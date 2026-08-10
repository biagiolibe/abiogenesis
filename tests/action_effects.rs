// Verifies the Stress action (task 023, GDD §6) has a real, observable
// effect on the simulation — headless and numeric, since the effect is too
// small relative to environmental diffusion to reliably eyeball in a single
// manual playtest click.

use abiogenesis::config::SimConfig;
use abiogenesis::sim::step;
use abiogenesis::world::{Organism, SimWorld, SpeciesId};
use abiogenesis::worldgen::generate_starting_palette;

/// Worlds no longer auto-place organisms (task 050) — the player seeds them
/// via `Seed`. Mirrors the old auto-placement exactly (the first
/// `starting_species_count` generated species, evenly spread along `y = 0`)
/// so species 0 lands at `(0, 0)` as this test's comments below assume.
fn place_starting_organisms(world: &mut SimWorld, config: &SimConfig) {
    let count = config.worldgen.starting_species_count as usize;
    for i in 0..count {
        let x = if count <= 1 {
            world.width / 2
        } else {
            i * (world.width - 1) / (count - 1)
        };
        let idx = world.index(x, 0);
        world.cells[idx].organism = Some(Organism {
            species: SpeciesId(i as u8),
            energy: config.energy.seed_energy,
            born_era: 0,
        });
    }
}

/// Mean energy across all living organisms of species 0, or `None` if none
/// survived.
fn species_zero_energy(world: &SimWorld) -> Option<f32> {
    let (total, count) = world
        .cells
        .iter()
        .filter_map(|cell| cell.organism)
        .filter(|organism| organism.species.0 == 0)
        .fold((0.0, 0u32), |(total, count), organism| {
            (total + organism.energy, count + 1)
        });
    (count > 0).then_some(total / count as f32)
}

#[test]
fn stress_action_measurably_hurts_the_stressed_organism() {
    let config = SimConfig::default();

    // Baseline: nominal starting palette, one era untouched.
    let mut baseline = SimWorld::new(42, &config);
    generate_starting_palette(&mut baseline, &config);
    place_starting_organisms(&mut baseline, &config);
    for _ in 0..config.time.era_ticks {
        step(&mut baseline, &config);
    }
    let baseline_energy = species_zero_energy(&baseline).expect(
        "the nominal starting-palette scenario (tests/balance.rs's own baseline) should never \
         lose species 0 within a single era",
    );

    // Stressed: same seed, but species 0's cell (left edge, cold optimum,
    // task 013) gets the same temperature bump `input::stress_on_click`
    // applies, spent to the full action budget (task 022: 3 points at cost
    // 1 each) before the era runs.
    let mut stressed = SimWorld::new(42, &config);
    generate_starting_palette(&mut stressed, &config);
    place_starting_organisms(&mut stressed, &config);
    let idx = stressed.index(0, 0);
    let uses = config.time.point_budget_per_era / config.time.action_costs.stress;
    for _ in 0..uses {
        stressed.cells[idx].temperature =
            (stressed.cells[idx].temperature + config.environment.stress_delta).clamp(0.0, 1.0);
    }
    for _ in 0..config.time.era_ticks {
        step(&mut stressed, &config);
    }
    let stressed_energy = species_zero_energy(&stressed);

    match stressed_energy {
        // A dead stressed organism is even stronger evidence the action
        // works than a merely-lower average would be.
        None => {}
        Some(stressed_avg) => assert!(
            stressed_avg < baseline_energy - 1.0,
            "stressing the organism's temperature away from its optimum should measurably \
             reduce its energy within one era, got baseline={baseline_energy} stressed={stressed_avg}"
        ),
    }
}
