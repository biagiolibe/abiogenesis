// Anti-degeneration properties of Phase 0's single-species scenario (GDD
// §5.8): a photolithic bloom must grow then stabilise instead of exploding
// or going extinct, and the light gradient must carve out a real niche.
// Together with task 007 this is Phase 0's exit gate (GDD §13).

use abiogenesis::config::SimConfig;
use abiogenesis::sim::step;
use abiogenesis::world::{seed_starting_palette, SimWorld};

/// Long enough for the bloom to grow, saturate, and settle (suggested
/// implementation: 300-500 ticks).
const RUN_TICKS: usize = 500;
/// Trailing window used to judge whether the population has settled.
const STABILITY_WINDOW: usize = 50;
/// Coefficients are declared tunable (GDD §14): check the *shape* of the
/// curve (relative amplitude), not an absolute population count.
const STABILITY_TOLERANCE: f32 = 0.10;
/// Light below which `light * photolithic_metabolism_gain * env_fit` can't
/// cover `base_upkeep` even at perfect thermal fitness (GDD §5.9).
const LIGHT_SURVIVAL_THRESHOLD: f32 = 0.25;

fn population(world: &SimWorld) -> usize {
    world.cells.iter().filter(|c| c.organism.is_some()).count()
}

/// Runs the Phase 0 nominal scenario (one photolithic organism seeded in the
/// lit band) for `ticks`, returning the final world and the population
/// sampled after every tick.
fn run_nominal_scenario(seed: u64, ticks: usize) -> (SimWorld, Vec<usize>) {
    let config = SimConfig::default();
    let mut world = SimWorld::new(seed, &config);
    seed_starting_palette(&mut world, &config);

    let mut history = Vec::with_capacity(ticks);
    for _ in 0..ticks {
        step(&mut world, &config);
        history.push(population(&world));
    }
    (world, history)
}

#[test]
fn population_never_reaches_zero() {
    let (_, history) = run_nominal_scenario(42, RUN_TICKS);

    assert!(
        history.iter().all(|&pop| pop > 0),
        "population must never hit extinction in the nominal scenario"
    );
}

#[test]
fn bloom_grows_then_stabilises() {
    let (_, history) = run_nominal_scenario(42, RUN_TICKS);

    let start = history[0];
    let peak = *history.iter().max().expect("non-empty history");
    assert!(
        peak > start,
        "population should grow from its seeded state, got start={start} peak={peak}"
    );

    let tail = &history[RUN_TICKS - STABILITY_WINDOW..];
    let tail_min = *tail.iter().min().expect("non-empty tail") as f32;
    let tail_max = *tail.iter().max().expect("non-empty tail") as f32;
    let relative_amplitude = (tail_max - tail_min) / tail_max;

    assert!(
        relative_amplitude <= STABILITY_TOLERANCE,
        "population should settle within a narrow band over the last \
         {STABILITY_WINDOW} ticks instead of still swinging or exploding, \
         got min={tail_min} max={tail_max} (relative amplitude {relative_amplitude:.3})"
    );
}

#[test]
fn dark_rows_stay_uninhabited() {
    let (world, _) = run_nominal_scenario(42, RUN_TICKS);

    for y in 0..world.height {
        // Light varies only by row (GDD §5.2): sampling column 0 is
        // representative of the whole row.
        let row_light = world.get(0, y).light;
        if row_light >= LIGHT_SURVIVAL_THRESHOLD {
            continue;
        }
        for x in 0..world.width {
            assert!(
                world.get(x, y).organism.is_none(),
                "row {y} has light {row_light:.3} < {LIGHT_SURVIVAL_THRESHOLD}, so it should \
                 be uninhabitable, but found an organism at ({x}, {y})"
            );
        }
    }
}
