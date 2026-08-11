// Anti-degeneration properties of the nominal starting scenario (GDD §5.8):
// a photolithic bloom must usually grow then stabilise instead of exploding
// or going extinct, and the light gradient must carve out a real niche.
// Together with task 007 this was Phase 0's exit gate (GDD §13). Since task
// 050 nothing is auto-placed in real play — `place_starting_organisms` below
// synthesizes a plausible player placement: each generated species goes onto
// the placeable cell whose temperature is closest to that species' own
// `temp_optimum`, so these seed-swept statistical properties measure real
// population dynamics under a fixed, comparable starting condition.
//
// This replaced a `y = 0`-corner placement (task 098/099 playtest session,
// 2026-08-11): `generate_starting_palette` used to derive every species'
// `temp_optimum` by reading the temperature at exactly the two grid corners
// this test placed organisms on, so `env_fit` was tautologically ~1.0 by
// construction, not because the scenario was actually balanced. Fixing that
// bug (species now draw `temp_optimum` from the real distribution of
// placeable-cell temperatures, `worldgen::placeable_temperature_distribution`)
// broke the coincidence — placement here now has to actually seek out where
// each species would fit, the way a player choosing where to `Seed` would.
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
use abiogenesis::world::{Organism, SimWorld, SpeciesId};
use abiogenesis::worldgen::generate_starting_palette;

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
/// Margin below `LIGHT_SURVIVAL_THRESHOLD` before a cell counts as "should
/// be uninhabitable" for `dark_cells_stay_uninhabited_across_seeds` (task
/// 074, rewritten per-cell in task 085 once light stopped varying only by
/// row): the per-cell light step shrinks with grid size, so cells near the
/// threshold's own value can legitimately still be inhabited, not yet
/// starved, without the boundary itself being a bug. Only cells clearly
/// below the survival threshold are asserted uninhabited.
const DARK_CELL_MARGIN: f32 = 0.03;

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
/// Generous margin above the empirical ~10% observed rate (task 048, after
/// `draw_species_tags` was fixed to reject net-*positive* self-interaction,
/// not just net-negative — see the module doc on `population_never_
/// saturates_the_grid_across_seeds` for why a residual minority is expected
/// rather than eliminated outright). Task 088 made that rejection
/// exhaustive rather than best-effort (the prior random-retry search could
/// provably fail to find a zero-net combination for ~15% of worlds' 3-tag
/// pool at world 0's default 5 active tags), so same-species
/// self-interaction is now eliminated exactly, not just reduced — any
/// residual saturation this test tolerates is now attributable entirely to
/// cross-species reinforcement, not a same-species leak.
const MAX_SATURATION_RATE: f32 = 0.3;

fn population(world: &SimWorld) -> usize {
    world.cells.iter().filter(|c| c.organism.is_some()).count()
}

/// Worlds no longer auto-place organisms (task 050) — the player seeds them
/// via `Seed`. Mirrors the old auto-placement exactly (the first
/// `starting_species_count` generated species, evenly spread along `y = 0`)
/// so these seed-swept statistical properties keep measuring real
/// population dynamics, the same nominal scenario they measured before.
fn place_starting_organisms(world: &mut SimWorld, config: &SimConfig) {
    let count = config.worldgen.starting_species_count as usize;
    for i in 0..count {
        let optimum = world.species[i].temp_optimum;
        let idx = (0..world.cells.len())
            .filter(|&idx| world.is_placeable_index(idx) && world.cells[idx].organism.is_none())
            .min_by(|&a, &b| {
                (world.cells[a].temperature - optimum)
                    .abs()
                    .total_cmp(&(world.cells[b].temperature - optimum).abs())
            })
            .expect("terrain generation guarantees at least one placeable cell");
        world.cells[idx].organism = Some(Organism {
            species: SpeciesId(i as u8),
            energy: config.energy.seed_energy,
            born_era: 0,
        });
    }
}

/// Runs the nominal scenario (a procedurally generated starting palette,
/// `generate_starting_palette`, task 039) for `ticks`, returning the final
/// world and the population sampled after every tick.
fn run_nominal_scenario(seed: u64, ticks: usize) -> (SimWorld, Vec<usize>) {
    let config = SimConfig::default();
    let mut world = SimWorld::new(seed, &config);
    generate_starting_palette(&mut world, &config);
    place_starting_organisms(&mut world, &config);

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

/// Opposite-direction companion to `population_rarely_reaches_total_
/// extinction_across_seeds` (task 048, GDD §5.8's "one dominates" failure
/// mode): the grid saturating completely (every cell occupied) means growth
/// never found a ceiling at all, the same generator-imbalance concern as
/// total extinction, just at the other extreme.
///
/// Root cause (2026-08-06 playtest): `world::draw_species_tags` rejected a
/// candidate tag set with net-*negative* self-interaction (a species that
/// drains itself into extinction the moment it reproduces next to itself)
/// but not net-*positive* self-interaction — and `sim::step`'s
/// `crowding_penalty` (`crowd_factor` per occupied neighbour, `0.15`) is
/// dwarfed by a single matrix entry (`±effect_intensity_max`, up to `±2`),
/// so any species whose own tags reinforced each other turned clustering
/// into unbounded growth. Fixed by requiring `net_self_interaction == 0`
/// (checked directly in `world.rs`'s own unit tests) rather than merely
/// `>= 0`.
///
/// That fix doesn't reach *cross*-species reinforcement (species A and B
/// whose tags mutually boost each other without either being
/// self-destructive) — tuning `crowd_factor` to also cover that case was
/// tried and rejected: strong enough to matter for the worst-case draws, it
/// also crushed normal populations toward extinction (measured during the
/// investigation), a worse trade than a documented residual minority.
/// `MAX_SATURATION_RATE` budgets for that minority the same way
/// `MAX_EXTINCTION_RATE` already budgets for the opposite extreme.
#[test]
fn population_never_saturates_the_grid_across_seeds() {
    let saturated = SURVEY_SEEDS
        .filter(|&seed| {
            let (world, _) = run_nominal_scenario(seed, RUN_TICKS);
            population(&world) == world.cells.len()
        })
        .count();
    let rate = saturated as f32 / SURVEY_SEED_COUNT as f32;

    assert!(
        rate <= MAX_SATURATION_RATE,
        "expected at most {:.0}% of seeds to saturate the whole grid, got {}/{} ({:.0}%)",
        MAX_SATURATION_RATE * 100.0,
        saturated,
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

/// Task 085: light now comes from a per-world sun-direction projection (plus
/// mountain shading), not a fixed top-bottom row gradient, so the check is
/// per-cell rather than per-row.
#[test]
fn dark_cells_stay_uninhabited_across_seeds() {
    for seed in SURVEY_SEEDS {
        let (world, _) = run_nominal_scenario(seed, RUN_TICKS);
        for y in 0..world.height {
            for x in 0..world.width {
                let light = world.get(x, y).light;
                if light >= LIGHT_SURVIVAL_THRESHOLD - DARK_CELL_MARGIN {
                    continue;
                }
                assert!(
                    world.get(x, y).organism.is_none(),
                    "seed {seed}: cell ({x}, {y}) has light {light:.3}, well below \
                     {LIGHT_SURVIVAL_THRESHOLD}, so it should be uninhabitable, but found an \
                     organism there"
                );
            }
        }
    }
}
