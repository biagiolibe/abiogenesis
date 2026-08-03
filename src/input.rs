// Maps keys to player intents (TECH_DESIGN.md §2): input only decides
// *when* the player wants something to happen, era mechanics stay owned by
// sim.rs and world.rs.

use bevy::prelude::*;

use abiogenesis::config::SimConfig;
use abiogenesis::sim::{step, EraProgress};
use abiogenesis::state::EraState;
use abiogenesis::world::{seed_starting_palette, SimWorld};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (start_era, single_tick, reseed_world, quit));
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
) {
    if *era_state.get() == EraState::Advancing {
        return;
    }
    if keys.just_pressed(KeyCode::KeyS) {
        step(&mut world, &config);
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
) {
    if keys.just_pressed(KeyCode::KeyR) {
        let new_seed = world.next_seed();
        *world = SimWorld::new(new_seed, &config);
        seed_starting_palette(&mut world, &config);
        progress.cancel();
        next_state.set(EraState::Observing);
    }
}

/// `Esc` quits. `q` was planned in GDD v0.3 but removed in v0.4, kept free
/// for future text input.
fn quit(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
