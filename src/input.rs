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
use abiogenesis::world::{seed_starting_palette, Organism, SimWorld};

use crate::notebook::{MatrixKnowledge, ObservationLog};
use crate::render::{world_to_cell, GridCamera};
use crate::ui::{ActionMode, SelectedAction, SelectedSpecies};

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
    }
}

/// Left-click while `ActionMode::Seed` is selected: places an organism of
/// the currently-selected species (GDD §6 "Seed") in the clicked cell, if
/// it's empty, affordable, and `EraState::Observing` — the same "ignored
/// mid-`Advancing`" rule the other player-driven systems in this file
/// follow. Clicks outside the grid (the HUD panel, letterboxed margins) are
/// silently ignored via `world_to_cell`. Costs `config.time.action_costs.seed`
/// action points (task 022); an unaffordable click does nothing at all, not
/// even the empty-cell check.
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

    let cell = world.get_mut(x, y);
    if cell.organism.is_some() {
        return;
    }
    cell.organism = Some(Organism {
        species: selected.0,
        energy: config.energy.seed_energy,
    });
    budget.points_remaining -= config.time.action_costs.seed;
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
/// starving or being predated.
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

    let cell = world.get_mut(x, y);
    if cell.organism.is_none() {
        return;
    }
    if budget.points_remaining < config.time.action_costs.cull {
        return;
    }
    cell.organism = None;
    budget.points_remaining -= config.time.action_costs.cull;
}

/// `Esc` quits. `q` was planned in GDD v0.3 but removed in v0.4, kept free
/// for future text input.
fn quit(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
