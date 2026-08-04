// Reproducibility of the simulation (GDD §5.7): the RNG lives in `SimWorld`,
// never in a global or the system clock, so the same seed must replay
// bit-for-bit and a different seed must diverge. No Bevy `App` involved —
// `step` is called directly, proving the sim doesn't need one (invariant 2).

use abiogenesis::config::SimConfig;
use abiogenesis::sim::step;
use abiogenesis::world::{seed_starting_palette, SimWorld};

const RUN_TICKS: usize = 200;

fn seeded_world(seed: u64, config: &SimConfig) -> SimWorld {
    let mut world = SimWorld::new(seed, config);
    seed_starting_palette(&mut world, config);
    world
}

#[test]
fn same_seed_yields_identical_history() {
    let config = SimConfig::default();
    let mut a = seeded_world(42, &config);
    let mut b = seeded_world(42, &config);

    for _ in 0..RUN_TICKS {
        step(&mut a, &config);
        step(&mut b, &config);
    }

    assert_eq!(a.tick, b.tick);
    assert_eq!(
        a.cells, b.cells,
        "same seed must reproduce the exact same grid state, cell by cell"
    );
}

#[test]
fn different_seeds_diverge() {
    // Checks divergence at any point during the run, not just the final
    // state (task 038 made active-tag selection procedural, so it's a real,
    // if rare, possibility that two arbitrary seeds both run their
    // population to total extinction and their grids re-converge once every
    // organism is gone and residue has fully decayed — that coincidence
    // would say nothing about whether `step` actually used the RNG).
    let config = SimConfig::default();
    let mut a = seeded_world(42, &config);
    let mut b = seeded_world(43, &config);

    let mut ever_diverged = false;
    for _ in 0..RUN_TICKS {
        step(&mut a, &config);
        step(&mut b, &config);
        if a.cells != b.cells {
            ever_diverged = true;
        }
    }

    assert!(
        ever_diverged,
        "different seeds must diverge at some point during the run \
         (a `step` that never draws from the RNG would pass the determinism \
         test above for the wrong reason)"
    );
}
