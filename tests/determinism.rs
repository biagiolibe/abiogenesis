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
    let config = SimConfig::default();
    let mut a = seeded_world(42, &config);
    let mut b = seeded_world(43, &config);

    for _ in 0..RUN_TICKS {
        step(&mut a, &config);
        step(&mut b, &config);
    }

    assert_ne!(
        a.cells, b.cells,
        "different seeds must not converge to the same grid state \
         (a `step` that never draws from the RNG would pass the determinism \
         test above for the wrong reason)"
    );
}
