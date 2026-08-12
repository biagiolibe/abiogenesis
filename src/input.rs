// Maps keys to player intents (TECH_DESIGN.md §2): input only decides
// *when* the player wants something to happen, era mechanics stay owned by
// sim.rs and world.rs.

use bevy::camera::Camera;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;

use abiogenesis::config::SimConfig;
use abiogenesis::objectives::{apply_tick_outcome, ObjectiveOutcomeParams};
#[cfg(test)]
use abiogenesis::objectives::{
    CurrentObjective, CurrentWorldOutcome, GraceProgress, ObjectiveAdvanced, ObjectiveProgress,
};
use abiogenesis::run::{MetaProgress, RunProgress};
use abiogenesis::sim::{
    tick_and_complete_era, ActionBudget, AdjacencyObserved, EraCompleted, EraProgress,
    OrganismBorn, OrganismDied, SpeciesExtinct, TerrainGateObserved, TerrainRevealed,
};
use abiogenesis::state::{EraState, GameState};
use abiogenesis::world::{draw_species_name, net_self_interaction, Organism, SimWorld, SpeciesId};
use abiogenesis::worldgen::era_ticks_for;

use crate::notebook::{EverSeeded, LogEntry, ObservationLog, PlayerPlacedCells};
use crate::render::{species_label, world_to_cell, GridCamera, MapViewMode, PlacementIndicator};
use crate::run_flow::{start_world, WorldResetParams};
use crate::text;
use crate::ui::{
    ActionMode, HudControlIntents, IsolationHint, SelectedAction, SelectedSpecies, SpliceDraft,
    SpliceEditChoice,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                start_era,
                single_tick,
                reseed_world,
                seed_organism_on_click,
                stress_on_click,
                cull_on_click,
                apply_splice,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, quit);
    }
}

/// Resolves the current left-click, if any, to a grid cell — the
/// window-cursor → camera → world-position → grid-coordinate pipeline task
/// 017 worked out for `Seed`, factored out so every click action (task 023
/// onward) reuses it instead of re-deriving the same edge cases. Returns
/// `None` for anything from "no click this frame" to "clicked outside the
/// grid" — callers don't need to distinguish why. Also returns `None`
/// whenever egui wants this frame's pointer input (task 091): the one
/// place every map-click action funnels through, so a click on a notebook
/// widget never also registers as a `Seed`/`Stress`/`Cull` click on the
/// cell underneath it.
fn clicked_cell(
    buttons: &ButtonInput<MouseButton>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    width: usize,
    height: usize,
    egui_wants_input: &EguiWantsInput,
) -> Option<(usize, usize)> {
    if !buttons.just_pressed(MouseButton::Left) {
        return None;
    }
    if egui_wants_input.wants_pointer_input() {
        return None;
    }
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let world_pos = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
    world_to_cell(world_pos, width, height)
}

/// `space` (or the HUD's Era button, task 094 — `HudControlIntents::
/// advance_era`, consumed here so the button and the shortcut share this one
/// implementation): starts (or resumes auto-playing) an era, unless one is
/// already advancing (acceptance criterion: advancement inputs are ignored
/// during `Advancing`). Only resets `EraProgress` to a full `era_ticks` if no
/// era is currently in flight — if the player already stepped some ticks
/// manually via `n` (`single_tick`), `space` auto-plays whatever ticks
/// remain rather than discarding that progress and restarting the era from
/// zero.
#[allow(clippy::too_many_arguments)]
fn start_era(
    keys: Res<ButtonInput<KeyCode>>,
    era_state: Res<State<EraState>>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    config: Res<SimConfig>,
    run_progress: Res<RunProgress>,
    world: Res<SimWorld>,
    mut intents: ResMut<HudControlIntents>,
) {
    if *era_state.get() == EraState::Advancing {
        return;
    }
    let triggered = keys.just_pressed(KeyCode::Space) || intents.advance_era;
    intents.advance_era = false;
    if triggered {
        if progress.remaining() == 0 {
            progress.start(era_ticks_for(run_progress.world_index, world.era, &config));
        }
        next_state.set(EraState::Advancing);
    }
}

/// `n` (or the HUD's Tick button, task 094 — `HudControlIntents::
/// advance_tick`, consumed here so the button and the shortcut share this
/// one implementation): advances a single tick directly, with no `EraState`
/// transition — useful for fine observation and debugging (GDD §11). Starts
/// a fresh era (`EraProgress`) if none is in flight, and shares
/// `advance_tick`'s era-completion bookkeeping and objective evaluation via
/// `tick_and_complete_era`/`apply_tick_outcome` — a player who only ever
/// presses `n` must still see eras complete, the action budget refill, and
/// objectives/failure conditions evaluate exactly as they would under
/// `space`, since both paths advance the same ticks. Bound to `n` rather
/// than `s` (task 087 follow-up) so `WASD` is free for camera pan.
#[allow(clippy::too_many_arguments)]
fn single_tick(
    keys: Res<ButtonInput<KeyCode>>,
    era_state: Res<State<EraState>>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut progress: ResMut<EraProgress>,
    mut budget: ResMut<ActionBudget>,
    mut died: MessageWriter<OrganismDied>,
    mut extinct: MessageWriter<SpeciesExtinct>,
    mut adjacencies: MessageWriter<AdjacencyObserved>,
    mut era_completed: MessageWriter<EraCompleted>,
    mut born: MessageWriter<OrganismBorn>,
    mut revealed: MessageWriter<TerrainRevealed>,
    mut terrain_gates: MessageWriter<TerrainGateObserved>,
    mut objective_outcome: ObjectiveOutcomeParams,
    run_progress: Res<RunProgress>,
    mut intents: ResMut<HudControlIntents>,
) {
    if *era_state.get() == EraState::Advancing {
        return;
    }
    let triggered = keys.just_pressed(KeyCode::KeyN) || intents.advance_tick;
    intents.advance_tick = false;
    if !triggered {
        return;
    }
    if progress.remaining() == 0 {
        progress.start(era_ticks_for(run_progress.world_index, world.era, &config));
    }
    let events = tick_and_complete_era(
        &mut world,
        &config,
        &mut progress,
        &mut budget,
        &mut era_completed,
    );
    died.write_batch(events.deaths);
    extinct.write_batch(events.extinctions);
    adjacencies.write_batch(events.adjacencies);
    born.write_batch(events.births);
    revealed.write_batch(events.reveals);
    terrain_gates.write_batch(events.terrain_gates);
    apply_tick_outcome(&world, &config, &mut objective_outcome);
}

/// `r`: reseeds the *current* world (same `world_index`, so the same
/// difficulty parameters — this doesn't advance the run) deterministically
/// from its own RNG (never the system clock, invariant 1), through the same
/// `start_world` (task 045) the world-cleared transition uses — one reset
/// implementation, not two that can drift apart. Allowed even
/// mid-`Advancing`: a full world reset legitimately invalidates whatever
/// animation was playing.
///
/// Deliberately keyboard-only, no HUD button (task 094 considered and
/// rejected one): this discards the entire current world — every organism,
/// every era of progress — with no confirmation step. A stray click is a
/// much easier accident than a stray keypress on a dedicated letter key, and
/// this codebase has no "are you sure?" affordance to add a safety net with.
/// If a button is ever added here, it should come with a confirmation
/// dialog, not reuse the bare-button pattern the other three time controls
/// use.
fn reseed_world(
    keys: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut run_progress: ResMut<RunProgress>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    mut reset: WorldResetParams,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        let new_seed = world.next_seed();
        start_world(
            &mut world,
            run_progress.world_index,
            new_seed,
            &config,
            run_progress.unlocks.bonus_available_species,
            &mut progress,
            &mut next_state,
            &mut reset,
        );
        run_progress.world_seed = new_seed;
    }
}

/// Left-click while `ActionMode::Seed` is selected: places an organism of
/// the currently-selected species (GDD §6 "Seed") in the clicked cell, if
/// it's placeable (task 067 — not Sea or a mountain peak), empty,
/// affordable, and `EraState::Observing` — the same "ignored mid-`Advancing`"
/// rule the other player-driven systems in this file follow. Clicks outside
/// the grid (the HUD panel, letterboxed margins) are silently ignored via
/// `world_to_cell`. Costs `config.time.action_costs.seed` action points
/// (task 022); an unaffordable click does nothing at all, not even the
/// placeable/empty-cell checks — same silent-no-op convention an
/// unplaceable-cell click now also follows, this codebase has no rejected-
/// action feedback mechanism yet. Records the cell into `PlayerPlacedCells`
/// (task 026) so the Notebook can log a salient entry if this organism
/// later dies, and latches `EverSeeded` (task 053's onboarding hint) since
/// `PlayerPlacedCells` alone empties back out on death. On the very first
/// such placement of the player's very first run (`!ever_seeded.0` still
/// true when this runs, gated by `MetaProgress::seen_isolation_hint`),
/// checks whether the placement was isolated and sets `IsolationHint` (task
/// 055) accordingly — informational only, never a gate on the placement
/// itself (task 050 deliberately removed placement constraints).
/// Every rejection rule `seed_organism_on_click` enforces once a clicked
/// cell is known: affordable, placeable (task 067 — not Sea or a mountain
/// peak), and empty. Pulled out of the system so it's unit-testable without
/// the mouse/window/camera harness `clicked_cell` needs. Silent no-op on
/// every rejection, matching the rest of this file's action gates (no
/// rejected-action feedback mechanism exists yet); returns whether the
/// placement actually happened.
#[allow(clippy::too_many_arguments)]
fn attempt_seed(
    world: &mut SimWorld,
    config: &SimConfig,
    budget: &mut ActionBudget,
    placed: &mut PlayerPlacedCells,
    species: SpeciesId,
    x: usize,
    y: usize,
) -> bool {
    if budget.points_remaining < config.time.action_costs.seed {
        return false;
    }
    if !world.is_placeable(x, y) {
        return false;
    }
    let index = world.index(x, y);
    let era = world.era;
    let cell = world.get_mut(x, y);
    if cell.organism.is_some() {
        return false;
    }
    cell.organism = Some(Organism {
        species,
        energy: config.energy.seed_energy,
        born_era: era,
    });
    budget.points_remaining -= config.time.action_costs.seed;
    placed.0.insert(index);
    true
}

/// Bundled into one `SystemParam` (mirrors `objectives.rs`'s
/// `ObjectiveOutcomeParams`, `run_flow.rs`'s `WorldResetParams`) purely to
/// stay under Bevy's per-function system-param limit — `seed_organism_on_click`
/// was already at the edge of it before task 092 added `run_progress` for
/// `IsolationHint`'s new era-derived `duration_ticks`.
#[derive(SystemParam)]
struct IsolationHintParams<'w> {
    isolation_hint: ResMut<'w, IsolationHint>,
    meta: ResMut<'w, MetaProgress>,
    run_progress: Res<'w, RunProgress>,
}

#[allow(clippy::too_many_arguments)]
fn seed_organism_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    era_state: Res<State<EraState>>,
    selected_action: Res<SelectedAction>,
    selected: Res<SelectedSpecies>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut budget: ResMut<ActionBudget>,
    mut placed: ResMut<PlayerPlacedCells>,
    mut ever_seeded: ResMut<EverSeeded>,
    mut hint_params: IsolationHintParams,
    mode: Res<MapViewMode>,
    mut placement_indicator: ResMut<PlacementIndicator>,
    egui_wants_input: Res<EguiWantsInput>,
) {
    if selected_action.0 != ActionMode::Seed {
        return;
    }
    if *era_state.get() == EraState::Advancing {
        return;
    }
    let Some((x, y)) = clicked_cell(
        &buttons,
        &windows,
        &cameras,
        world.width,
        world.height,
        &egui_wants_input,
    ) else {
        return;
    };
    if !attempt_seed(
        &mut world,
        &config,
        &mut budget,
        &mut placed,
        selected.0,
        x,
        y,
    ) {
        return;
    }

    // Overview's cluster-heatmap aggregation (task 076) doesn't show
    // individual cells, so a placement there gets a transient ring marking
    // exactly which cell it landed on (task 077) — Detail already shows the
    // organism sprite directly, no indicator needed.
    if *mode == MapViewMode::Overview {
        placement_indicator.show(x, y);
    }

    if !ever_seeded.0 && !hint_params.meta.seen_isolation_hint {
        hint_params.isolation_hint.text = Some(if is_isolated_placement(&world, x, y) {
            text::HINT_ISOLATED_FIRST_PLACEMENT
        } else {
            text::HINT_CLUSTERED_FIRST_PLACEMENT
        });
        hint_params.isolation_hint.shown_at_tick = world.tick;
        // Task 092: pinned to the era length *at the moment the hint is
        // shown*, not re-derived later — see `IsolationHint`'s own doc
        // comment for why a later, longer era shouldn't retroactively
        // extend an already-showing hint's lifetime.
        hint_params.isolation_hint.duration_ticks =
            era_ticks_for(hint_params.run_progress.world_index, world.era, &config) as u64;
        hint_params.meta.seen_isolation_hint = true;
    }
    ever_seeded.0 = true;
}

/// Whether the cell at `(x, y)` has no occupied Moore neighbour (task 055) —
/// the condition the confounder-weight formula (`sim.rs`'s
/// `weight = 1/(1+confounders)`) rewards with full-weight evidence. Pure so
/// it's unit-testable without a running `App`.
fn is_isolated_placement(world: &SimWorld, x: usize, y: usize) -> bool {
    world
        .moore_neighbours(x, y)
        .all(|idx| world.cells[idx].organism.is_none())
}

/// Left-click while `ActionMode::Stress` is selected (GDD §6 "Stress"):
/// shifts the clicked cell's temperature by `config.environment.stress_delta`,
/// clamped to `[0,1]` (the existing scalar-range invariant) — temperature,
/// not toxicity, since `sim::step`'s `env_fit` reads temperature every tick
/// and toxicity currently isn't read anywhere, so a stressed cell has an
/// observable effect on any organism sitting on it (subject to environmental
/// diffusion smearing it back toward neighbours over subsequent ticks, same
/// as any other scalar). Unlike `Seed`, occupancy isn't a precondition —
/// Stress targets the environment, so it works on empty and occupied cells
/// alike. Same budget-check-then-decrement pattern as `Seed`.
#[allow(clippy::too_many_arguments)]
fn stress_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    era_state: Res<State<EraState>>,
    selected_action: Res<SelectedAction>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut budget: ResMut<ActionBudget>,
    mode: Res<MapViewMode>,
    egui_wants_input: Res<EguiWantsInput>,
) {
    if selected_action.0 != ActionMode::Stress {
        return;
    }
    if *era_state.get() == EraState::Advancing {
        return;
    }
    // Stress needs per-cell precision Overview's cluster-heatmap aggregation
    // (task 076) doesn't preserve — gated to Detail (task 077). The HUD
    // (`ui.rs`) also disables the Stress button in Overview so the player
    // never reaches a click that silently does nothing; this check is
    // defense-in-depth against a `SelectedAction` left on Stress from before
    // zooming out.
    if *mode != MapViewMode::Detail {
        return;
    }
    let Some((x, y)) = clicked_cell(
        &buttons,
        &windows,
        &cameras,
        world.width,
        world.height,
        &egui_wants_input,
    ) else {
        return;
    };
    if budget.points_remaining < config.time.action_costs.stress {
        return;
    }

    let cell = world.get_mut(x, y);
    cell.temperature = (cell.temperature + config.environment.stress_delta).clamp(0.0, 1.0);
    budget.points_remaining -= config.time.action_costs.stress;
}

/// Left-click while `ActionMode::Cull` is selected (GDD §6 "Cull"): removes
/// the organism at the clicked cell, if any — the empty/occupied asymmetry
/// mirrors `Seed`'s but inverted (`Seed` needs an empty cell, `Cull` needs
/// an occupied one), so occupancy is checked *before* spending the budget:
/// clicking an empty cell costs nothing and does nothing. Deliberately
/// deposits no residue — GDD §5.6 step 6 ties residue to *death* by the
/// tick algorithm (energy `<= 0`), not to an organism's removal by any
/// means, and a player-culled organism is removed by fiat rather than
/// starving or being predated. Also clears the cell from `PlayerPlacedCells`
/// (task 026) if present — a culled organism generates no `OrganismDied`
/// (it never goes through `sim::step`), so nothing would otherwise remove
/// the stale entry, and a *later* organism placed at the same cell must not
/// inherit a "player-placed" marker that was really about a different,
/// already-culled individual. This does mean a culled player-placed
/// organism itself never gets a Notebook entry — a known gap, not fixed
/// here (see task 024's own note on `Cull` emitting no event at all).
#[allow(clippy::too_many_arguments)]
fn cull_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    era_state: Res<State<EraState>>,
    selected_action: Res<SelectedAction>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut budget: ResMut<ActionBudget>,
    mut placed: ResMut<PlayerPlacedCells>,
    mode: Res<MapViewMode>,
    egui_wants_input: Res<EguiWantsInput>,
) {
    if selected_action.0 != ActionMode::Cull {
        return;
    }
    if *era_state.get() == EraState::Advancing {
        return;
    }
    // Same Detail-only gating as `stress_on_click` (task 077) — culling a
    // specific organism needs per-cell precision Overview's aggregated
    // heatmap doesn't offer.
    if *mode != MapViewMode::Detail {
        return;
    }
    let Some((x, y)) = clicked_cell(
        &buttons,
        &windows,
        &cameras,
        world.width,
        world.height,
        &egui_wants_input,
    ) else {
        return;
    };

    let index = world.index(x, y);
    let cell = world.get_mut(x, y);
    if cell.organism.is_none() {
        return;
    }
    if budget.points_remaining < config.time.action_costs.cull {
        return;
    }
    cell.organism = None;
    budget.points_remaining -= config.time.action_costs.cull;
    placed.0.remove(&index);
}

/// Applies a `Splice` (GDD §6) once the HUD's editor panel (`ui.rs`) sets
/// `SpliceDraft::apply_requested` — the one action whose trigger is an egui
/// button rather than a grid click, so it has no `clicked_cell` call, but
/// otherwise follows the same `Observing`-only, budget-check-then-decrement
/// pattern as the other three. Creates a **new** species (`world.species`
/// append) with the edit applied, rather than mutating the source species
/// in place: mutating in place would retroactively change every
/// already-alive organism of that species, but "modify a species' genome"
/// (GDD §6) reads as introducing a variant, not rewriting history — species
/// identity otherwise stays stable for a run (GDD §5.6 step 7 only floats
/// child-genome mutation as a *future* idea). Silently does nothing if the
/// draft is incomplete (no source picked, or a `SwapTag` missing either
/// tag), or if the resulting tag set has a nonzero `net_self_interaction`
/// (task 089) — an unseen matrix combination that would make the spliced
/// species self-reinforce or self-drain, the exact invariant
/// `draw_species_tags` already enforces for worldgen-created species (task
/// 088) — rather than partially applying it.
fn apply_splice(
    era_state: Res<State<EraState>>,
    mut draft: ResMut<SpliceDraft>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut budget: ResMut<ActionBudget>,
    mut log: ResMut<ObservationLog>,
) {
    if !draft.apply_requested {
        return;
    }
    draft.apply_requested = false;
    if *era_state.get() == EraState::Advancing {
        return;
    }
    let Some(source) = draft.source else {
        return;
    };
    let Some(source_species) = world.species.get(source.0 as usize) else {
        return;
    };
    let mut new_species = source_species.clone();
    // Task 095: a spliced child draws its own independent name — cloning
    // `source_species` would otherwise leave it sharing its parent's.
    new_species.name = draw_species_name(&mut world);
    match draft.edit {
        SpliceEditChoice::SwapTag {
            old: Some(old),
            new: Some(new),
        } => {
            let Some(pos) = new_species.tags.iter().position(|&tag| tag == old) else {
                return;
            };
            new_species.tags[pos] = new;
            if net_self_interaction(&world.matrix, &new_species.tags) != 0 {
                return;
            }
        }
        SpliceEditChoice::AddTag { tag: Some(tag) } => {
            // Defense-in-depth against a stale draft (e.g. picked before
            // switching to a source that's already at the cap): the UI
            // gates this too, but a leftover selection must still no-op
            // here rather than push past GDD §5.3's 3-tag cap.
            if new_species.tags.len() >= 3 {
                return;
            }
            new_species.tags.push(tag);
            if net_self_interaction(&world.matrix, &new_species.tags) != 0 {
                return;
            }
        }
        SpliceEditChoice::ShiftTempOptimum { warmer } => {
            let delta = if warmer {
                config.energy.splice_temp_shift
            } else {
                -config.energy.splice_temp_shift
            };
            new_species.temp_optimum = (new_species.temp_optimum + delta).clamp(0.0, 1.0);
        }
        // Incomplete SwapTag/AddTag selection (missing tag(s)).
        SpliceEditChoice::SwapTag { .. } | SpliceEditChoice::AddTag { tag: None } => return,
    }
    if budget.points_remaining < config.time.action_costs.splice {
        return;
    }
    let new_species_id = world.push_species(new_species);
    log.entries.push(LogEntry {
        era: world.era,
        species: Some(new_species_id),
        text: text::species_created_message(&species_label(&world, new_species_id)),
    });
    budget.points_remaining -= config.time.action_costs.splice;
    *draft = SpliceDraft::default();
}

/// `Esc` quits. `q` was planned in GDD v0.3 but removed in v0.4, kept free
/// for future text input.
fn quit(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::{MatrixKnowledge, NotebookHasUnseenConfirmation};
    use abiogenesis::world::{Species, TagId, TagMatrix, TagSlot, TerrainKind};
    use abiogenesis::worldgen::generate_starting_palette;
    use bevy::state::state::State;

    /// A world with two active tags and one species carrying only tag 0,
    /// enough to exercise both `SpliceEditChoice` variants without needing
    /// the full RNG-driven world generation. The matrix is forced to all
    /// zeros (rather than whatever seed 42 happens to generate) so every
    /// splice test's outcome is deterministic for a stated reason — a
    /// neutral matrix — not by luck of the seed, now that `apply_splice`
    /// itself checks `net_self_interaction` (task 089).
    fn world_with_one_taggable_species() -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.active_tags = vec![TagId(0), TagId(1)];
        world.matrix = TagMatrix::from_values(2, vec![0, 0, 0, 0]);
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: abiogenesis::world::Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagSlot(0)],
        });
        (world, config)
    }

    fn app_with(world: SimWorld, config: SimConfig, draft: SpliceDraft) -> App {
        let mut app = App::new();
        app.insert_resource(config);
        app.insert_resource(world);
        app.insert_resource(State::new(EraState::Observing));
        app.insert_resource(ActionBudget {
            points_remaining: 3,
        });
        app.insert_resource(draft);
        app.insert_resource(ObservationLog::default());
        app.add_systems(Update, apply_splice);
        app
    }

    #[test]
    fn swap_tag_splice_appends_a_new_species_and_leaves_the_source_untouched() {
        let (world, config) = world_with_one_taggable_species();
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::SwapTag {
                    old: Some(TagSlot(0)),
                    new: Some(TagSlot(1)),
                },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.species.len(),
            2,
            "the splice should append, not replace"
        );
        assert_eq!(
            world.species[0].tags,
            vec![TagSlot(0)],
            "the source species must stay unchanged"
        );
        assert_eq!(
            world.species[1].tags,
            vec![TagSlot(1)],
            "the new species should carry the swapped-in tag"
        );

        let budget = app.world().resource::<ActionBudget>();
        assert_eq!(budget.points_remaining, 1, "splice costs 2 points");

        let draft = app.world().resource::<SpliceDraft>();
        assert!(!draft.apply_requested, "the intent must be consumed");
        assert_eq!(
            draft.source, None,
            "the draft resets after a successful splice"
        );
    }

    /// `SwapTag` only replaces one tag in place — a source with just one
    /// tag would always land on a still-single-tag (trivially self-neutral)
    /// result, so these three tests need a 3-tag active pool and a 2-tag
    /// source to make `SwapTag`'s self-interaction check meaningful:
    /// `[TagSlot(0), TagSlot(2)]`, swapping `TagSlot(2)` for `TagSlot(1)`,
    /// leaves `[TagSlot(0), TagSlot(1)]` — the pair the matrix override
    /// below actually tests.
    fn world_with_one_two_tagged_species() -> (SimWorld, SimConfig) {
        let (mut world, config) = world_with_one_taggable_species();
        world.active_tags.push(TagId(2));
        world.matrix = TagMatrix::from_values(3, vec![0; 9]);
        world.species[0].tags = vec![TagSlot(0), TagSlot(2)];
        (world, config)
    }

    /// Task 089: a `SwapTag` whose result nets a positive self-interaction
    /// (a hidden, unseen combination that would make the spliced species
    /// self-reinforce every time it reproduces next to itself) must be
    /// rejected outright, the same invariant `draw_species_tags` already
    /// enforces for worldgen-created species (task 088).
    #[test]
    fn swap_tag_splice_is_rejected_when_the_result_self_reinforces() {
        let (mut world, config) = world_with_one_two_tagged_species();
        #[rustfmt::skip]
        let values = vec![
            0, 1, 0,
            1, 0, 0,
            0, 0, 0,
        ];
        world.matrix = TagMatrix::from_values(3, values);
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::SwapTag {
                    old: Some(TagSlot(2)),
                    new: Some(TagSlot(1)),
                },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(world.species.len(), 1, "no species should be appended");
        assert_eq!(
            world.species[0].tags,
            vec![TagSlot(0), TagSlot(2)],
            "the source species must stay unchanged"
        );

        let budget = app.world().resource::<ActionBudget>();
        assert_eq!(
            budget.points_remaining, 3,
            "a rejected splice costs nothing"
        );

        let log = app.world().resource::<ObservationLog>();
        assert!(log.entries.is_empty(), "nothing should be logged");

        let draft = app.world().resource::<SpliceDraft>();
        assert!(
            !draft.apply_requested,
            "the intent is still consumed even on rejection"
        );
    }

    /// Regression guard for the design choice to reject `!= 0`, not just
    /// `> 0`: net-negative self-interaction is just as real a failure (the
    /// spliced species would die the instant it reproduces next to itself)
    /// and the player has no way to see the hidden matrix that caused it.
    #[test]
    fn swap_tag_splice_is_rejected_when_the_result_self_drains() {
        let (mut world, config) = world_with_one_two_tagged_species();
        #[rustfmt::skip]
        let values = vec![
            0, -1, 0,
            -1, 0, 0,
            0, 0, 0,
        ];
        world.matrix = TagMatrix::from_values(3, values);
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::SwapTag {
                    old: Some(TagSlot(2)),
                    new: Some(TagSlot(1)),
                },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(world.species.len(), 1, "no species should be appended");
    }

    /// The gate must only block a nonzero *net* self-interaction, not every
    /// matrix with nonzero individual entries.
    #[test]
    fn swap_tag_splice_is_applied_when_the_result_is_self_neutral() {
        let (mut world, config) = world_with_one_two_tagged_species();
        #[rustfmt::skip]
        let values = vec![
            0, 1, 0,
            -1, 0, 0,
            0, 0, 0,
        ];
        world.matrix = TagMatrix::from_values(3, values);
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::SwapTag {
                    old: Some(TagSlot(2)),
                    new: Some(TagSlot(1)),
                },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(world.species.len(), 2, "a self-neutral splice must apply");
        assert_eq!(world.species[1].tags, vec![TagSlot(0), TagSlot(1)]);

        let budget = app.world().resource::<ActionBudget>();
        assert_eq!(budget.points_remaining, 1, "splice costs 2 points");
    }

    /// Regression guard for gating only the tag-mutating arms: a
    /// `ShiftTempOptimum` splice off a source that is already
    /// self-reinforcing (for a reason unrelated to this edit) must still
    /// apply — the check must not fire for edits that never touch `tags`.
    #[test]
    fn shift_temp_splice_is_unaffected_by_a_self_reinforcing_source() {
        let (mut world, config) = world_with_one_taggable_species();
        world.matrix = TagMatrix::from_values(2, vec![0, 1, 1, 0]);
        world.species[0].tags = vec![TagSlot(0), TagSlot(1)];
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::ShiftTempOptimum { warmer: true },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(world.species.len(), 2, "the shift must still apply");
        assert_eq!(world.species[1].temp_optimum, 0.65);
    }

    #[test]
    fn a_successful_splice_logs_the_new_species() {
        let (world, config) = world_with_one_taggable_species();
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::SwapTag {
                    old: Some(TagSlot(0)),
                    new: Some(TagSlot(1)),
                },
                apply_requested: true,
            },
        );
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(log.entries.len(), 1, "the new species must be logged");
        assert_eq!(log.entries[0].species, Some(SpeciesId(1)));
    }

    /// Task 095: a spliced child draws its own independent name — cloning
    /// `source_species` (`apply_splice`'s starting point for the edit) must
    /// not leave it sharing its parent's name unchanged.
    #[test]
    fn spliced_species_draws_its_own_name_not_a_copy_of_its_parents() {
        let (mut world, config) = world_with_one_taggable_species();
        world.species[0].name = "ParentSentinel".to_string();
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::SwapTag {
                    old: Some(TagSlot(0)),
                    new: Some(TagSlot(1)),
                },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_ne!(
            world.species[1].name, "ParentSentinel",
            "the spliced child must draw its own name, not inherit its parent's"
        );
    }

    #[test]
    fn shift_temp_splice_clamps_to_the_unit_range() {
        let (mut world, config) = world_with_one_taggable_species();
        world.species[0].temp_optimum = 0.95;
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::ShiftTempOptimum { warmer: true },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(world.species.len(), 2);
        assert_eq!(
            world.species[0].temp_optimum, 0.95,
            "the source species must stay unchanged"
        );
        assert_eq!(
            world.species[1].temp_optimum, 1.0,
            "a warmer shift past 1.0 should clamp, not wrap or panic"
        );
    }

    #[test]
    fn add_tag_splice_appends_a_tag_without_removing_any() {
        let (world, config) = world_with_one_taggable_species();
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::AddTag {
                    tag: Some(TagSlot(1)),
                },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.species.len(),
            2,
            "the splice should append, not replace"
        );
        assert_eq!(
            world.species[0].tags,
            vec![TagSlot(0)],
            "the source species must stay unchanged"
        );
        assert_eq!(
            world.species[1].tags,
            vec![TagSlot(0), TagSlot(1)],
            "the new species should carry the original tag plus the added one"
        );
    }

    /// Mirrors `swap_tag_splice_is_rejected_when_the_result_self_reinforces`
    /// via the `AddTag` arm instead of `SwapTag`.
    #[test]
    fn add_tag_splice_is_rejected_when_the_result_self_reinforces() {
        let (mut world, config) = world_with_one_taggable_species();
        world.matrix = TagMatrix::from_values(2, vec![0, 1, 1, 0]);
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::AddTag {
                    tag: Some(TagSlot(1)),
                },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(world.species.len(), 1, "no species should be appended");
        assert_eq!(
            world.species[0].tags,
            vec![TagSlot(0)],
            "the source species must stay unchanged"
        );
    }

    #[test]
    fn add_tag_splice_does_nothing_on_a_species_already_at_the_cap() {
        let (mut world, config) = world_with_one_taggable_species();
        world.species[0].tags = vec![TagSlot(0), TagSlot(1), TagSlot(0)];
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::AddTag {
                    tag: Some(TagSlot(1)),
                },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.species.len(),
            1,
            "adding a tag to a species already at the 3-tag cap must not apply"
        );
    }

    #[test]
    fn incomplete_swap_tag_draft_does_nothing() {
        let (world, config) = world_with_one_taggable_species();
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                // Only `old` picked, no `new` tag yet.
                edit: SpliceEditChoice::SwapTag {
                    old: Some(TagSlot(0)),
                    new: None,
                },
                apply_requested: true,
            },
        );
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.species.len(),
            1,
            "an incomplete draft must not splice"
        );
        let budget = app.world().resource::<ActionBudget>();
        assert_eq!(budget.points_remaining, 3, "nothing should be spent");
    }

    #[test]
    fn splice_without_enough_budget_does_nothing() {
        let (world, config) = world_with_one_taggable_species();
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::ShiftTempOptimum { warmer: true },
                apply_requested: true,
            },
        );
        app.world_mut()
            .resource_mut::<ActionBudget>()
            .points_remaining = 1;
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.species.len(),
            1,
            "an unaffordable splice must not apply"
        );
    }

    #[test]
    fn reseed_resets_a_selected_species_left_pointing_past_the_fresh_worlds_registry() {
        // Simulates having spliced past the starting-palette species count
        // (task 025) and left the seed selector pointing at the spliced-in
        // species, then pressing `r`. A fresh world built via `reseed_world`
        // goes through `build_world` (unlike this test's own setup below,
        // which calls `generate_starting_palette` directly), so it always
        // starts with `starting_species_count + extra_available_species_count
        // + wild_species_count` species (task 039's `generate_starting_palette`
        // plus task 098's wild populations, 2 + 1 + 1 by default), so
        // `SelectedSpecies` must be pulled back in range or the next seed
        // click indexes out of bounds.
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        generate_starting_palette(&mut world, &config);
        world.push_species(world.species[0].clone()); // pretend a splice happened

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyR);

        let mut app = App::new();
        app.insert_resource(config);
        app.insert_resource(world);
        app.insert_resource(keys);
        app.insert_resource(EraProgress::default());
        app.insert_resource(NextState::<EraState>::default());
        app.insert_resource(MatrixKnowledge::new(5, 3.0));
        app.insert_resource(crate::notebook::TerrainKnowledge::new(5, 3.0));
        app.insert_resource(ObservationLog::default());
        app.insert_resource(ActionBudget::default());
        app.insert_resource(SelectedSpecies(SpeciesId(5)));
        app.insert_resource(SpliceDraft {
            source: Some(SpeciesId(5)),
            ..SpliceDraft::default()
        });
        app.insert_resource(PlayerPlacedCells(std::collections::HashSet::from([1, 5])));
        app.insert_resource(NotebookHasUnseenConfirmation::default());
        app.insert_resource(IsolationHint::default());
        app.insert_resource(RunProgress::default());
        app.insert_resource(CurrentObjective::default());
        app.insert_resource(ObjectiveProgress::default());
        app.insert_resource(CurrentWorldOutcome::default());
        app.insert_resource(GraceProgress::default());
        app.insert_resource(crate::ui::PopulationTrends::default());
        app.insert_resource(crate::notebook::BirthTally::default());
        app.insert_resource(crate::render::SeenRelations::new(5));
        app.add_systems(Update, reseed_world);
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.species.len(),
            4,
            "a fresh world starts with the starting palette's species count plus wild species"
        );
        let selected = app.world().resource::<SelectedSpecies>();
        assert_eq!(
            selected.0,
            SpeciesId(0),
            "a selection past the fresh world's species count must be reset, not left dangling"
        );
        let draft = app.world().resource::<SpliceDraft>();
        assert_eq!(draft.source, None, "the splice draft should reset too");
        let placed = app.world().resource::<PlayerPlacedCells>();
        assert!(
            placed.0.is_empty(),
            "stale player-placed cell indices from the old world must not carry over"
        );
    }

    /// A player who only ever presses `s` (never `space`) must still see
    /// eras complete: `world.era` advances, the action budget refills, and
    /// `EraCompleted` fires — the bug this test guards against had all three
    /// silently never happening because `single_tick` called `step()`
    /// directly, bypassing `advance_tick`'s era-boundary bookkeeping
    /// entirely.
    #[test]
    fn repeated_single_ticks_alone_complete_an_era_and_refill_the_budget() {
        let config = SimConfig::default();
        let world = SimWorld::new(42, &config);

        let mut app = App::new();
        app.insert_resource(config.clone());
        app.insert_resource(world);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(State::new(EraState::Observing));
        app.insert_resource(EraProgress::default());
        app.insert_resource(ActionBudget {
            points_remaining: 0,
        });
        app.add_message::<OrganismDied>();
        app.add_message::<SpeciesExtinct>();
        app.add_message::<AdjacencyObserved>();
        app.add_message::<EraCompleted>();
        app.add_message::<OrganismBorn>();
        app.add_message::<TerrainRevealed>();
        app.add_message::<TerrainGateObserved>();
        app.add_message::<ObjectiveAdvanced>();
        app.insert_resource(CurrentObjective::default());
        app.insert_resource(ObjectiveProgress::default());
        app.insert_resource(CurrentWorldOutcome::default());
        app.insert_resource(GraceProgress::default());
        // world_index: 1, not the default 0 — this test exercises
        // single_tick's generic era-completion bookkeeping, not task 082's
        // world-0 onboarding pacing exception, so it must use the standard
        // `era_ticks` length.
        app.insert_resource(RunProgress {
            world_index: 1,
            ..RunProgress::default()
        });
        app.insert_resource(MetaProgress::default());
        app.insert_resource(NextState::<GameState>::default());
        app.init_resource::<HudControlIntents>();
        app.add_systems(Update, single_tick);

        for _ in 0..config.time.era_ticks {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyN);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::KeyN);
        }

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.era, 1,
            "era_ticks worth of manual single-ticks must complete the era"
        );
        let budget = app.world().resource::<ActionBudget>();
        assert_eq!(
            budget.points_remaining, config.time.point_budget_per_era,
            "the action budget must refill on a manually-completed era, same as an auto-played one"
        );
    }

    #[test]
    fn is_isolated_placement_true_with_no_occupied_neighbours() {
        let config = SimConfig::default();
        let world = SimWorld::new(42, &config);
        assert!(is_isolated_placement(&world, 5, 5));
    }

    #[test]
    fn is_isolated_placement_false_with_one_occupied_moore_neighbour() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.get_mut(6, 5).organism = Some(Organism {
            species: SpeciesId(0),
            energy: 1.0,
            born_era: 0,
        });
        assert!(!is_isolated_placement(&world, 5, 5));
    }

    #[test]
    fn attempt_seed_rejects_an_unplaceable_cell() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.get_mut(5, 5).terrain = TerrainKind::Sea;
        let mut budget = ActionBudget {
            points_remaining: 3,
        };
        let mut placed = PlayerPlacedCells::default();

        let placed_ok = attempt_seed(
            &mut world,
            &config,
            &mut budget,
            &mut placed,
            SpeciesId(0),
            5,
            5,
        );

        assert!(!placed_ok);
        assert!(world.get(5, 5).organism.is_none());
        assert_eq!(
            budget.points_remaining, 3,
            "budget must not be spent on a rejected placement"
        );
        assert!(placed.0.is_empty());
    }

    #[test]
    fn attempt_seed_rejects_an_unplaceable_peak_even_though_the_terrain_is_mountain() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let cell = world.get_mut(5, 5);
        cell.terrain = TerrainKind::Mountain;
        cell.is_peak = true;
        let mut budget = ActionBudget {
            points_remaining: 3,
        };
        let mut placed = PlayerPlacedCells::default();

        let placed_ok = attempt_seed(
            &mut world,
            &config,
            &mut budget,
            &mut placed,
            SpeciesId(0),
            5,
            5,
        );

        assert!(!placed_ok);
        assert!(world.get(5, 5).organism.is_none());
    }

    #[test]
    fn attempt_seed_succeeds_on_ordinary_placeable_terrain() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.get_mut(5, 5).terrain = TerrainKind::Plain;
        let mut budget = ActionBudget {
            points_remaining: 3,
        };
        let mut placed = PlayerPlacedCells::default();

        let placed_ok = attempt_seed(
            &mut world,
            &config,
            &mut budget,
            &mut placed,
            SpeciesId(0),
            5,
            5,
        );

        assert!(placed_ok);
        assert!(world.get(5, 5).organism.is_some());
        assert_eq!(budget.points_remaining, 2);
    }
}
