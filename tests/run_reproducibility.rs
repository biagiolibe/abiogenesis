// End-to-end reproducibility of a *run* across world transitions (task 045).
// `determinism.rs` proves a single world replays bit-for-bit from its seed;
// this extends the same guarantee across `worldgen::build_world`'s seed
// chaining — world N+1's seed comes from world N's own RNG
// (`SimWorld::next_seed`), the same source `screens.rs::world_cleared_screen_ui`
// draws from, never `run_seed` again past world 0. No Bevy `App` involved,
// same rationale as `determinism.rs`: the chain doesn't need one.

use abiogenesis::config::SimConfig;
use abiogenesis::objectives::Objective;
use abiogenesis::sim::step;
use abiogenesis::worldgen::build_world;

const WORLDS_TO_PLAY: u32 = 3;
const TICKS_PER_WORLD: usize = 30;

struct WorldSnapshot {
    active_tags: Vec<abiogenesis::world::TagId>,
    matrix: abiogenesis::world::TagMatrix,
    cells: Vec<abiogenesis::world::Cell>,
    objective: Objective,
}

/// Plays `run_seed` through `WORLDS_TO_PLAY` consecutive worlds exactly as
/// the real "world cleared, continue" path does: simulate some ticks, draw
/// the next seed from the finishing world's own RNG, build the next world
/// at `world_index + 1`.
fn play_run(run_seed: u64, config: &SimConfig) -> Vec<WorldSnapshot> {
    let mut snapshots = Vec::new();
    let mut seed = run_seed;

    for world_index in 0..WORLDS_TO_PLAY {
        let (mut world, objective) = build_world(seed, world_index, config);
        for _ in 0..TICKS_PER_WORLD {
            step(&mut world, config);
        }
        seed = world.next_seed();
        snapshots.push(WorldSnapshot {
            active_tags: world.active_tags.clone(),
            matrix: world.matrix.clone(),
            cells: world.cells.clone(),
            objective,
        });
    }

    snapshots
}

#[test]
fn run_reproduces_the_same_world_sequence_from_the_same_run_seed() {
    let config = SimConfig::default();
    let a = play_run(123, &config);
    let b = play_run(123, &config);

    assert_eq!(a.len(), WORLDS_TO_PLAY as usize);
    for (world_index, (world_a, world_b)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            world_a.active_tags, world_b.active_tags,
            "world {world_index}'s active tags must match"
        );
        assert_eq!(
            world_a.matrix, world_b.matrix,
            "world {world_index}'s hidden matrix must match"
        );
        assert_eq!(
            world_a.cells, world_b.cells,
            "world {world_index}'s grid state after {TICKS_PER_WORLD} ticks must match"
        );
        assert_eq!(
            world_a.objective, world_b.objective,
            "world {world_index}'s generated objective must match"
        );
    }
}

#[test]
fn a_different_run_seed_diverges_the_world_sequence() {
    let config = SimConfig::default();
    let a = play_run(123, &config);
    let b = play_run(124, &config);

    let ever_diverged = a
        .iter()
        .zip(b.iter())
        .any(|(world_a, world_b)| world_a.cells != world_b.cells);

    assert!(
        ever_diverged,
        "different run seeds must diverge at some point across the world sequence"
    );
}
