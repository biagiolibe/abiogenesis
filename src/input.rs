// Maps keys to player intents (TECH_DESIGN.md §2): input only decides
// *when* the player wants something to happen, era mechanics stay owned by
// sim.rs and world.rs.

use bevy::camera::Camera;
use bevy::prelude::*;

use abiogenesis::config::SimConfig;
use abiogenesis::objectives::{
    apply_tick_outcome, CurrentObjective, CurrentWorldOutcome, ObjectiveProgress,
};
use abiogenesis::run::{MetaProgress, RunProgress};
use abiogenesis::sim::{
    tick_and_complete_era, ActionBudget, AdjacencyObserved, EraCompleted, EraProgress,
    OrganismDied, SpeciesExtinct,
};
use abiogenesis::state::{EraState, GameState};
use abiogenesis::world::{Organism, SimWorld, SpeciesId};

use crate::notebook::{EverSeeded, LogEntry, ObservationLog, PlayerPlacedCells};
use crate::render::{species_label, world_to_cell, GridCamera};
use crate::run_flow::{start_world, WorldResetParams};
use crate::text;
use crate::ui::{
    ActionMode, IsolationHint, SelectedAction, SelectedSpecies, SpliceDraft, SpliceEditChoice,
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
/// grid" — callers don't need to distinguish why.
fn clicked_cell(
    buttons: &ButtonInput<MouseButton>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    width: usize,
    height: usize,
) -> Option<(usize, usize)> {
    if !buttons.just_pressed(MouseButton::Left) {
        return None;
    }
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let world_pos = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
    world_to_cell(world_pos, width, height)
}

/// `space`: starts (or resumes auto-playing) an era, unless one is already
/// advancing (acceptance criterion: advancement inputs are ignored during
/// `Advancing`). Only resets `EraProgress` to a full `era_ticks` if no era is
/// currently in flight — if the player already stepped some ticks manually
/// via `s` (`single_tick`), `space` auto-plays whatever ticks remain rather
/// than discarding that progress and restarting the era from zero.
fn start_era(
    keys: Res<ButtonInput<KeyCode>>,
    era_state: Res<State<EraState>>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    config: Res<SimConfig>,
) {
    if *era_state.get() == EraState::Advancing {
        return;
    }
    if keys.just_pressed(KeyCode::Space) {
        if progress.remaining() == 0 {
            progress.start(config.time.era_ticks);
        }
        next_state.set(EraState::Advancing);
    }
}

/// `s`: advances a single tick directly, with no `EraState` transition —
/// useful for fine observation and debugging (GDD §11). Starts a fresh era
/// (`EraProgress`) if none is in flight, and shares `advance_tick`'s
/// era-completion bookkeeping and objective evaluation via
/// `tick_and_complete_era`/`apply_tick_outcome` — a player who only ever
/// presses `s` must still see eras complete, the action budget refill, and
/// objectives/failure conditions evaluate exactly as they would under
/// `space`, since both paths advance the same ticks.
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
    objective: Res<CurrentObjective>,
    mut objective_progress: ResMut<ObjectiveProgress>,
    mut outcome: ResMut<CurrentWorldOutcome>,
    run_progress: Res<RunProgress>,
    mut meta: ResMut<MetaProgress>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if *era_state.get() == EraState::Advancing {
        return;
    }
    if !keys.just_pressed(KeyCode::KeyS) {
        return;
    }
    if progress.remaining() == 0 {
        progress.start(config.time.era_ticks);
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
    apply_tick_outcome(
        &world,
        objective.0.as_ref(),
        &mut objective_progress,
        &mut outcome,
        &run_progress,
        &mut meta,
        &config,
        &mut next_game_state,
    );
}

/// `r`: reseeds the *current* world (same `world_index`, so the same
/// difficulty parameters — this doesn't advance the run) deterministically
/// from its own RNG (never the system clock, invariant 1), through the same
/// `start_world` (task 045) the world-cleared transition uses — one reset
/// implementation, not two that can drift apart. Allowed even
/// mid-`Advancing`: a full world reset legitimately invalidates whatever
/// animation was playing.
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
/// it's empty, affordable, and `EraState::Observing` — the same "ignored
/// mid-`Advancing`" rule the other player-driven systems in this file
/// follow. Clicks outside the grid (the HUD panel, letterboxed margins) are
/// silently ignored via `world_to_cell`. Costs `config.time.action_costs.seed`
/// action points (task 022); an unaffordable click does nothing at all, not
/// even the empty-cell check. Records the cell into `PlayerPlacedCells`
/// (task 026) so the Notebook can log a salient entry if this organism
/// later dies, and latches `EverSeeded` (task 053's onboarding hint) since
/// `PlayerPlacedCells` alone empties back out on death. On the very first
/// such placement of the player's very first run (`!ever_seeded.0` still
/// true when this runs, gated by `MetaProgress::seen_isolation_hint`),
/// checks whether the placement was isolated and sets `IsolationHint` (task
/// 055) accordingly — informational only, never a gate on the placement
/// itself (task 050 deliberately removed placement constraints).
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
    mut meta: ResMut<MetaProgress>,
    mut isolation_hint: ResMut<IsolationHint>,
) {
    if selected_action.0 != ActionMode::Seed {
        return;
    }
    if *era_state.get() == EraState::Advancing {
        return;
    }
    let Some((x, y)) = clicked_cell(&buttons, &windows, &cameras, world.width, world.height) else {
        return;
    };
    if budget.points_remaining < config.time.action_costs.seed {
        return;
    }

    let index = world.index(x, y);
    let cell = world.get_mut(x, y);
    if cell.organism.is_some() {
        return;
    }
    cell.organism = Some(Organism {
        species: selected.0,
        energy: config.energy.seed_energy,
    });
    budget.points_remaining -= config.time.action_costs.seed;
    placed.0.insert(index);

    if !ever_seeded.0 && !meta.seen_isolation_hint {
        isolation_hint.text = Some(if is_isolated_placement(&world, x, y) {
            text::HINT_ISOLATED_FIRST_PLACEMENT
        } else {
            text::HINT_CLUSTERED_FIRST_PLACEMENT
        });
        isolation_hint.shown_at_tick = world.tick;
        meta.seen_isolation_hint = true;
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
) {
    if selected_action.0 != ActionMode::Stress {
        return;
    }
    if *era_state.get() == EraState::Advancing {
        return;
    }
    let Some((x, y)) = clicked_cell(&buttons, &windows, &cameras, world.width, world.height) else {
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
) {
    if selected_action.0 != ActionMode::Cull {
        return;
    }
    if *era_state.get() == EraState::Advancing {
        return;
    }
    let Some((x, y)) = clicked_cell(&buttons, &windows, &cameras, world.width, world.height) else {
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
/// tag) rather than partially applying it.
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
    match draft.edit {
        SpliceEditChoice::SwapTag {
            old: Some(old),
            new: Some(new),
        } => {
            let Some(pos) = new_species.tags.iter().position(|&tag| tag == old) else {
                return;
            };
            new_species.tags[pos] = new;
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
    world.species.push(new_species);
    let new_species_id = SpeciesId(world.species.len() as u8 - 1);
    log.entries.push(LogEntry {
        era: world.era,
        species: Some(new_species_id),
        text: text::species_created_message(&species_label(new_species_id)),
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
    use abiogenesis::world::{Species, TagId, TagSlot};
    use abiogenesis::worldgen::generate_starting_palette;
    use bevy::state::state::State;

    /// A world with two active tags and one species carrying only tag 0,
    /// enough to exercise both `SpliceEditChoice` variants without needing
    /// the full RNG-driven world generation.
    fn world_with_one_taggable_species() -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.active_tags = vec![TagId(0), TagId(1)];
        world.species.push(Species {
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
        // species, then pressing `r`. A fresh world always starts with
        // `starting_species_count + extra_available_species_count` species
        // (task 039's `generate_starting_palette`, 2 + 1 by default), so
        // `SelectedSpecies` must be pulled back in range or the next seed
        // click indexes out of bounds.
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        generate_starting_palette(&mut world, &config);
        world.species.push(world.species[0].clone()); // pretend a splice happened

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyR);

        let mut app = App::new();
        app.insert_resource(config);
        app.insert_resource(world);
        app.insert_resource(keys);
        app.insert_resource(EraProgress::default());
        app.insert_resource(NextState::<EraState>::default());
        app.insert_resource(MatrixKnowledge::new(5, 3.0));
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
        app.add_systems(Update, reseed_world);
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.species.len(),
            3,
            "a fresh world starts with the starting palette's species count"
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
        app.insert_resource(CurrentObjective::default());
        app.insert_resource(ObjectiveProgress::default());
        app.insert_resource(CurrentWorldOutcome::default());
        app.insert_resource(RunProgress::default());
        app.insert_resource(MetaProgress::default());
        app.insert_resource(NextState::<GameState>::default());
        app.add_systems(Update, single_tick);

        for _ in 0..config.time.era_ticks {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyS);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::KeyS);
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
        });
        assert!(!is_isolated_placement(&world, 5, 5));
    }
}
