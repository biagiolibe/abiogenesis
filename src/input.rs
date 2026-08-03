// Maps keys to player intents (TECH_DESIGN.md §2): input only decides
// *when* the player wants something to happen, era mechanics stay owned by
// sim.rs and world.rs.

use bevy::camera::Camera;
use bevy::prelude::*;

use abiogenesis::config::SimConfig;
use abiogenesis::sim::{
    step, ActionBudget, AdjacencyObserved, EraProgress, OrganismDied, SpeciesExtinct,
};
use abiogenesis::state::EraState;
use abiogenesis::world::{seed_starting_palette, Organism, SimWorld, SpeciesId};

use crate::notebook::{MatrixKnowledge, ObservationLog, PlayerPlacedCells};
use crate::render::{world_to_cell, GridCamera};
use crate::ui::{ActionMode, SelectedAction, SelectedSpecies, SpliceDraft, SpliceEditChoice};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                start_era,
                single_tick,
                reseed_world,
                quit,
                seed_organism_on_click,
                stress_on_click,
                cull_on_click,
                apply_splice,
            ),
        );
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

/// `space`: starts an era, unless one is already advancing (acceptance
/// criterion: advancement inputs are ignored during `Advancing`).
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
        progress.start(config.time.era_ticks);
        next_state.set(EraState::Advancing);
    }
}

/// `s`: advances a single tick directly, with no state transition — useful
/// for fine observation and debugging (GDD §11).
fn single_tick(
    keys: Res<ButtonInput<KeyCode>>,
    era_state: Res<State<EraState>>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut died: MessageWriter<OrganismDied>,
    mut extinct: MessageWriter<SpeciesExtinct>,
    mut adjacencies: MessageWriter<AdjacencyObserved>,
) {
    if *era_state.get() == EraState::Advancing {
        return;
    }
    if keys.just_pressed(KeyCode::KeyS) {
        let events = step(&mut world, &config);
        died.write_batch(events.deaths);
        extinct.write_batch(events.extinctions);
        adjacencies.write_batch(events.adjacencies);
    }
}

/// `r`: reseeds the world deterministically from the current RNG (never the
/// system clock, invariant 1), cancelling any era in progress. Allowed even
/// mid-`Advancing`: a full world reset legitimately invalidates whatever
/// animation was playing.
#[allow(clippy::too_many_arguments)]
fn reseed_world(
    keys: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    mut knowledge: ResMut<MatrixKnowledge>,
    mut log: ResMut<ObservationLog>,
    mut budget: ResMut<ActionBudget>,
    mut selected: ResMut<SelectedSpecies>,
    mut splice_draft: ResMut<SpliceDraft>,
    mut placed: ResMut<PlayerPlacedCells>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        let new_seed = world.next_seed();
        *world = SimWorld::new(new_seed, &config);
        seed_starting_palette(&mut world, &config);
        progress.cancel();
        next_state.set(EraState::Observing);
        // A new seed means a new hidden matrix (task 011): stale confirmed
        // evidence from the previous run must not carry over.
        *knowledge = MatrixKnowledge::new(
            config.tags.active_tags_early as usize,
            config.notebook.confirmation_threshold,
        );
        // `world.era` resets to 0 above; stale entries would otherwise show
        // era numbers higher than the fresh run's current era.
        log.entries.clear();
        budget.refill(config.time.point_budget_per_era);
        // `Splice` (task 025) can grow `world.species` past the fresh
        // world's starting count; a `SelectedSpecies` left pointing at an
        // index that no longer exists would panic the next time anything
        // indexes `world.species` by it (e.g. `ui.rs::species_stats`).
        selected.0 = SpeciesId(0);
        *splice_draft = SpliceDraft::default();
        // Cell indices from the previous world mean nothing in the fresh
        // one — a stale entry could wrongly mark whatever ends up in that
        // cell next as "player-placed".
        placed.0.clear();
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
/// later dies.
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
    use abiogenesis::world::{Species, SpeciesId, TagId};
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
            tags: vec![TagId(0)],
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
                    old: Some(TagId(0)),
                    new: Some(TagId(1)),
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
            vec![TagId(0)],
            "the source species must stay unchanged"
        );
        assert_eq!(
            world.species[1].tags,
            vec![TagId(1)],
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
                    tag: Some(TagId(1)),
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
            vec![TagId(0)],
            "the source species must stay unchanged"
        );
        assert_eq!(
            world.species[1].tags,
            vec![TagId(0), TagId(1)],
            "the new species should carry the original tag plus the added one"
        );
    }

    #[test]
    fn add_tag_splice_does_nothing_on_a_species_already_at_the_cap() {
        let (mut world, config) = world_with_one_taggable_species();
        world.species[0].tags = vec![TagId(0), TagId(1), TagId(0)];
        let mut app = app_with(
            world,
            config,
            SpliceDraft {
                source: Some(SpeciesId(0)),
                edit: SpliceEditChoice::AddTag {
                    tag: Some(TagId(1)),
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
                    old: Some(TagId(0)),
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
        // species, then pressing `r`. The fresh world only ever starts with
        // 2 species (`seed_starting_palette`), so `SelectedSpecies` must be
        // pulled back in range or the next seed click indexes out of bounds.
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        seed_starting_palette(&mut world, &config);
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
        app.insert_resource(SelectedSpecies(SpeciesId(2)));
        app.insert_resource(SpliceDraft {
            source: Some(SpeciesId(2)),
            ..SpliceDraft::default()
        });
        app.insert_resource(PlayerPlacedCells(std::collections::HashSet::from([1, 5])));
        app.add_systems(Update, reseed_world);
        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.species.len(),
            2,
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
}
