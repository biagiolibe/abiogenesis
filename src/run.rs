// Run-level state (task 035, GDD §8-§10): where the player is within a
// sequence of worlds. Kept separate from `world.rs` (which owns a *single*
// world's simulation state) so it survives across the `SimWorld` rebuilds
// that world transitions (task 045) perform.

use bevy::prelude::*;

/// Tracks progress through a run (a sequence of worlds, GDD §8: "success ->
/// next world... failure -> the run ends"). `run_seed` is chosen once, at
/// the main menu (task 044) — the one legitimate point outside the
/// simulation where a run's variety originates; every subsequent
/// `world_seed` derives from the run's own seeded RNG, never a fresh source,
/// to preserve determinism (TECH_DESIGN.md §5).
#[derive(Resource, Debug, Clone, Default)]
pub struct RunProgress {
    pub run_seed: u64,
    pub world_index: u32,
    pub world_seed: u64,
    pub worlds_cleared: u32,
    pub unlocks: Unlocks,
}

/// Meta-progression unlocks accumulated across runs within the same process
/// session (GDD §10: capabilities, never matrix answers; no disk
/// persistence for the MVP). Empty placeholder — populated by task 046.
#[derive(Debug, Clone, Default)]
pub struct Unlocks;

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunProgress>();
    }
}
