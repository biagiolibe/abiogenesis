// Maps keys to player intents (TECH_DESIGN.md §2): input only decides
// *when* the player wants something to happen, era mechanics stay owned by
// sim.rs and world.rs.

use bevy::camera::Camera;
use bevy::prelude::*;

use abiogenesis::config::SimConfig;
use abiogenesis::sim::{step, AdjacencyObserved, EraProgress, OrganismDied, SpeciesExtinct};
use abiogenesis::state::EraState;
use abiogenesis::world::{seed_starting_palette, Organism, SimWorld};

use crate::notebook::{MatrixKnowledge, ObservationLog};
use crate::render::{world_to_cell, GridCamera};
use crate::ui::SelectedSpecies;

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
            ),
        );
    }
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
fn reseed_world(
    keys: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    mut knowledge: ResMut<MatrixKnowledge>,
    mut log: ResMut<ObservationLog>,
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
    }
}

/// Left-click: places an organism of the currently-selected species (GDD §6
/// "Seed", the only Phase 1 action) in the clicked cell, if it's empty and
/// `EraState::Observing` — the same "ignored mid-`Advancing`" rule the other
/// player-driven systems in this file follow. Clicks outside the grid (the
/// HUD panel, letterboxed margins) are silently ignored via `world_to_cell`.
/// No action-budget point is charged: that economy is Phase 2.
fn seed_organism_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    era_state: Res<State<EraState>>,
    selected: Res<SelectedSpecies>,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
) {
    if *era_state.get() == EraState::Advancing {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };
    let Some((x, y)) = world_to_cell(world_pos, world.width, world.height) else {
        return;
    };

    let cell = world.get_mut(x, y);
    if cell.organism.is_some() {
        return;
    }
    cell.organism = Some(Organism {
        species: selected.0,
        energy: config.energy.seed_energy,
    });
}

/// `Esc` quits. `q` was planned in GDD v0.3 but removed in v0.4, kept free
/// for future text input.
fn quit(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
