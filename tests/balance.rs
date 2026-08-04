// Anti-degeneration properties of the nominal two-species scenario (GDD
// §5.8): a photolithic bloom must usually grow then stabilise instead of
// exploding or going extinct, and the light gradient must carve out a real
// niche. Together with task 007 this was Phase 0's exit gate (GDD §13).
//
// Task 038 made active-tag selection procedural (`select_active_tags`
// samples the RNG instead of always taking `TagId(0..5)`), so a fixed seed
// no longer pins one fixed matrix/species-tags outcome — the RNG stream a
// given seed produces shifts with every later task that adds another draw
// before or during world generation (039's species pool, 042's objective
// generation, ...). Pinning to one "lucky" seed would just mean re-shopping
// it each time. These tests instead assert the anti-degeneration property
// GDD §5.8 actually makes: most seeds should stay alive and settle, not
// that a single specific seed does. A minority reaching total extinction
// (two species with a negative interaction between them is a real possible
// draw, not a bug) is expected and budgeted for below; a majority failing
// would mean the generator itself is unbalanced (GDD §14's main risk),
// which is a Final Tuning concern, not something to chase here.

use abiogenesis::config::SimConfig;
use abiogenesis::sim::step;
use abiogenesis::world::{seed_starting_palette, SimWorld};

/// Long enough for a bloom to grow, saturate, and settle (suggested
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

/// Seeds surveyed for the statistical properties below. Empirically (task
/// 038's own investigation), 3/50 seeds in this range hit total extinction
/// and 4/50 fail to stabilise within `STABILITY_TOLERANCE` — a real minority
/// outcome of a negative species-pair interaction, not systemic imbalance.
const SURVEY_SEEDS: std::ops::Range<u64> = 0..50;
const SURVEY_SEED_COUNT: u64 = SURVEY_SEEDS.end - SURVEY_SEEDS.start;
/// Generous margins above the empirical ~6-8% observed rates: these tests
/// exist to catch the generator becoming *systemically* unbalanced (most
/// seeds failing), not to chase every individual unlucky draw.
const MAX_EXTINCTION_RATE: f32 = 0.3;
const MAX_UNSTABLE_RATE: f32 = 0.3;

fn population(world: &SimWorld) -> usize {
    world.cells.iter().filter(|c| c.organism.is_some()).count()
}

/// Runs the nominal scenario (two photolithic species seeded at opposite
/// ends of the temperature gradient, `seed_starting_palette`) for `ticks`,
/// returning the final world and the population sampled after every tick.
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

/// Relative swing of the trailing `STABILITY_WINDOW` ticks of `history`;
/// `f32::INFINITY` for a population that ended at zero, so total extinction
/// always counts as "not stabilised" without dividing by zero.
fn tail_relative_amplitude(history: &[usize]) -> f32 {
    let tail = &history[history.len() - STABILITY_WINDOW..];
    let tail_min = *tail.iter().min().expect("non-empty tail") as f32;
    let tail_max = *tail.iter().max().expect("non-empty tail") as f32;
    if tail_max == 0.0 {
        f32::INFINITY
    } else {
        (tail_max - tail_min) / tail_max
    }
}

#[test]
fn population_rarely_reaches_total_extinction_across_seeds() {
    let extinctions = SURVEY_SEEDS
        .filter(|&seed| {
            let (_, history) = run_nominal_scenario(seed, RUN_TICKS);
            history.contains(&0)
        })
        .count();
    let rate = extinctions as f32 / SURVEY_SEED_COUNT as f32;

    assert!(
        rate <= MAX_EXTINCTION_RATE,
        "expected at most {:.0}% of seeds to hit total extinction, got {}/{} ({:.0}%)",
        MAX_EXTINCTION_RATE * 100.0,
        extinctions,
        SURVEY_SEED_COUNT,
        rate * 100.0
    );
}

#[test]
fn bloom_usually_grows_then_stabilises_across_seeds() {
    let unstable = SURVEY_SEEDS
        .filter(|&seed| {
            let (_, history) = run_nominal_scenario(seed, RUN_TICKS);
            tail_relative_amplitude(&history) > STABILITY_TOLERANCE
        })
        .count();
    let rate = unstable as f32 / SURVEY_SEED_COUNT as f32;

    assert!(
        rate <= MAX_UNSTABLE_RATE,
        "expected at most {:.0}% of seeds to still be swinging or exploding over the last \
         {STABILITY_WINDOW} ticks, got {}/{} ({:.0}%)",
        MAX_UNSTABLE_RATE * 100.0,
        unstable,
        SURVEY_SEED_COUNT,
        rate * 100.0
    );
}

#[test]
fn dark_rows_stay_uninhabited_across_seeds() {
    for seed in SURVEY_SEEDS {
        let (world, _) = run_nominal_scenario(seed, RUN_TICKS);
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
                    "seed {seed}: row {y} has light {row_light:.3} < \
                     {LIGHT_SURVIVAL_THRESHOLD}, so it should be uninhabitable, but found an \
                     organism at ({x}, {y})"
                );
            }
        }
    }
}
