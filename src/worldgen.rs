// Procedural world generation (task 038+, GDD §9). This module starts with
// the difficulty curve alone (task 037): a pure function from "which world
// in the run is this" to concrete generation parameters. Tag/environment/
// species/objective generation (later Phase 3 tasks) consume `WorldParams`
// instead of reading `SimConfig`'s early/late endpoints directly, so the
// curve has exactly one source of truth.

use rand::RngExt;

use crate::config::SimConfig;
use crate::world::{draw_species_tags, Metabolism, Organism, SimWorld, Species, SpeciesId};

/// Concrete generation parameters for one world, derived from its position
/// in the run (`world_index`, 0-based: the first world is `0`). Every field
/// ramps linearly from its "early" endpoint to its "late" endpoint over
/// `DifficultyConfig::ramp_worlds` worlds, then holds steady — GDD §8's run
/// is endless-until-failure, so there is no final world to design a hard
/// ceiling for.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldParams {
    /// How many tags from the global pool are active in this world (GDD §9:
    /// 5 -> ~8 across the curve).
    pub active_tag_count: u32,
    /// How many eras this world's run may take before failing (GDD §8: 40 ->
    /// 25 across the curve).
    pub era_budget: u32,
    /// Toxic zone width in cells (GDD §9: "larger toxic zones").
    pub toxic_zone_width: u32,
    /// Toxic zone height in cells.
    pub toxic_zone_height: u32,
    /// Temperature gradient spread (`right - left`, GDD §9: "harsher
    /// thermal gradients").
    pub temperature_spread: f32,
    /// Fraction of tag-pair matrix cells that are non-zero (GDD §9: "meaner
    /// matrix").
    pub matrix_density: f32,
    /// Multiplier task 042 applies to objective thresholds (GDD §9:
    /// "stricter objectives").
    pub objective_severity: f32,
}

/// Computes `WorldParams` for `world_index` (the run's `RunProgress::world_index`,
/// task 035). Pure function of `world_index` and `config` — no `SimWorld`,
/// no RNG, headless-testable (TECH_DESIGN.md invariant 2), so later tasks
/// can call it before a world even exists.
pub fn world_params(world_index: u32, config: &SimConfig) -> WorldParams {
    let t = ramp_fraction(world_index, config.difficulty.ramp_worlds);
    let env = &config.environment;
    let tags = &config.tags;
    let time = &config.time;
    let difficulty = &config.difficulty;
    let early_temperature_spread = env.temperature_gradient_right - env.temperature_gradient_left;

    WorldParams {
        active_tag_count: lerp_u32(tags.active_tags_early, tags.active_tags_late, t),
        era_budget: lerp_u32(time.era_budget_early, time.era_budget_late, t),
        toxic_zone_width: lerp_u32(env.toxic_zone_width, difficulty.toxic_zone_width_late, t),
        toxic_zone_height: lerp_u32(env.toxic_zone_height, difficulty.toxic_zone_height_late, t),
        temperature_spread: lerp_f32(
            early_temperature_spread,
            difficulty.temperature_spread_late,
            t,
        ),
        matrix_density: lerp_f32(tags.matrix_density, difficulty.matrix_density_late, t),
        objective_severity: lerp_f32(
            difficulty.objective_severity_early,
            difficulty.objective_severity_late,
            t,
        ),
    }
}

/// `0.0` at `world_index = 0`, ramping linearly to `1.0` at `world_index =
/// ramp_worlds`, then staying at `1.0` — the curve saturates rather than
/// extrapolating past its late endpoint, since a run can outlast any fixed
/// number of worlds. `ramp_worlds = 0` saturates immediately (every world
/// uses the late endpoint), rather than dividing by zero.
fn ramp_fraction(world_index: u32, ramp_worlds: u32) -> f32 {
    if ramp_worlds == 0 {
        return 1.0;
    }
    (world_index as f32 / ramp_worlds as f32).min(1.0)
}

fn lerp_u32(early: u32, late: u32, t: f32) -> u32 {
    (early as f32 + (late as f32 - early as f32) * t).round() as u32
}

fn lerp_f32(early: f32, late: f32, t: f32) -> f32 {
    early + (late - early) * t
}

/// A world's starting species (task 039, GDD §9/§10): every species the
/// player could choose to `Seed` in this world, plus which of them are
/// already on the grid when the world starts. `available` is the superset
/// task 046's meta-progression unlocks will extend; `placed` never grows on
/// its own — only pre-seeded organisms count.
#[derive(Debug, Clone, PartialEq)]
pub struct StartingPalette {
    /// Every generated species, indexable by the positions in `placed`.
    pub available: Vec<Species>,
    /// `(index into available, grid position)` for organisms present on the
    /// grid at world start.
    pub placed: Vec<(usize, (usize, usize))>,
}

/// Replaces Phase 1's `seed_starting_palette` placeholder (task 013, always
/// exactly 2 fixed photolithic species) with a real generator (task 038's
/// world already has its own tag subset/matrix/environment by the time this
/// runs — see `SimWorld::new_for_world`):
///
/// - `WorldgenConfig::starting_species_count` species are placed on the
///   grid, evenly spread along row `y = 0` (the highest-light row). Always
///   `Metabolism::Photolithic`, the only metabolism that's self-sustaining
///   from light alone with no prey or residue already present (GDD §5.4) —
///   `temp_optimum` is read directly from the generated environment at each
///   placement site, so it's always a good fit for where it's placed,
///   whatever this world's temperature spread (task 038) turned out to be.
/// - `WorldgenConfig::extra_available_species_count` further species are
///   added to `available` only, with `Metabolism::Predator`/`Decomposer` in
///   alternation — giving the player metabolism variety to seed
///   deliberately (GDD §6 `Seed`) without every starting organism needing
///   prey/residue to survive its first ticks.
///
/// Every species draws its tags from the world's own active subset
/// (`draw_species_tags`, task 010/036) and its RNG from `world`'s own seeded
/// stream (never an external RNG), so the whole palette stays deterministic
/// given the same world seed.
pub fn generate_starting_palette(world: &mut SimWorld, config: &SimConfig) -> StartingPalette {
    let placed_count = config.worldgen.starting_species_count as usize;
    let mut placed = Vec::with_capacity(placed_count);

    for i in 0..placed_count {
        let x = if placed_count <= 1 {
            world.width / 2
        } else {
            i * (world.width - 1) / (placed_count - 1)
        };
        let temp_optimum = world.get(x, 0).temperature;
        let tags = draw_species_tags(world, config);
        let species_index = world.species.len();
        world.species.push(Species {
            metabolism: Metabolism::Photolithic,
            temp_optimum,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags,
        });
        let idx = world.index(x, 0);
        world.cells[idx].organism = Some(Organism {
            species: SpeciesId(species_index as u8),
            energy: config.energy.seed_energy,
        });
        placed.push((species_index, (x, 0)));
    }

    let cold = world.get(0, 0).temperature;
    let hot = world.get(world.width - 1, 0).temperature;
    for i in 0..config.worldgen.extra_available_species_count as usize {
        let metabolism = if i % 2 == 0 {
            Metabolism::Predator
        } else {
            Metabolism::Decomposer
        };
        let weight: f32 = world.rng_mut().random_range(0.0..=1.0);
        let temp_optimum = cold + (hot - cold) * weight;
        let tags = draw_species_tags(world, config);
        world.species.push(Species {
            metabolism,
            temp_optimum,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags,
        });
    }

    StartingPalette {
        available: world.species.clone(),
        placed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_index_zero_matches_the_early_endpoints_exactly() {
        let config = SimConfig::default();
        let params = world_params(0, &config);

        assert_eq!(params.active_tag_count, config.tags.active_tags_early);
        assert_eq!(params.era_budget, config.time.era_budget_early);
        assert_eq!(params.toxic_zone_width, config.environment.toxic_zone_width);
        assert_eq!(
            params.toxic_zone_height,
            config.environment.toxic_zone_height
        );
        assert_eq!(
            params.temperature_spread,
            config.environment.temperature_gradient_right
                - config.environment.temperature_gradient_left
        );
        assert_eq!(params.matrix_density, config.tags.matrix_density);
        assert_eq!(
            params.objective_severity,
            config.difficulty.objective_severity_early
        );
    }

    /// GDD §16's worked example: World 2 (`world_index = 1`, the second
    /// world of the run) has 6 active tags, not a jump straight to 8 — the
    /// literal constraint the whole curve is built to satisfy.
    #[test]
    fn world_index_one_has_six_active_tags() {
        let config = SimConfig::default();
        let params = world_params(1, &config);

        assert_eq!(params.active_tag_count, 6);
    }

    #[test]
    fn the_curve_saturates_at_the_late_endpoints_past_ramp_worlds() {
        let config = SimConfig::default();
        let at_ramp_end = world_params(config.difficulty.ramp_worlds, &config);
        let well_past_ramp_end = world_params(config.difficulty.ramp_worlds + 50, &config);

        assert_eq!(at_ramp_end, well_past_ramp_end);
        assert_eq!(at_ramp_end.active_tag_count, config.tags.active_tags_late);
        assert_eq!(at_ramp_end.era_budget, config.time.era_budget_late);
        assert_eq!(
            at_ramp_end.toxic_zone_width,
            config.difficulty.toxic_zone_width_late
        );
        assert_eq!(
            at_ramp_end.toxic_zone_height,
            config.difficulty.toxic_zone_height_late
        );
        assert_eq!(
            at_ramp_end.temperature_spread,
            config.difficulty.temperature_spread_late
        );
        assert_eq!(
            at_ramp_end.matrix_density,
            config.difficulty.matrix_density_late
        );
        assert_eq!(
            at_ramp_end.objective_severity,
            config.difficulty.objective_severity_late
        );
    }

    #[test]
    fn era_budget_decreases_monotonically_across_the_ramp() {
        let config = SimConfig::default();
        let mut previous = world_params(0, &config).era_budget;
        for world_index in 1..=config.difficulty.ramp_worlds {
            let current = world_params(world_index, &config).era_budget;
            assert!(
                current <= previous,
                "era budget should never increase across the ramp: world {world_index} had {current}, previous had {previous}"
            );
            previous = current;
        }
    }

    #[test]
    fn starting_palette_is_deterministic_for_the_same_seed() {
        let config = SimConfig::default();
        let mut a = SimWorld::new(42, &config);
        let mut b = SimWorld::new(42, &config);

        let palette_a = generate_starting_palette(&mut a, &config);
        let palette_b = generate_starting_palette(&mut b, &config);

        assert_eq!(palette_a, palette_b);
    }

    #[test]
    fn placed_species_are_photolithic_and_carry_valid_tags() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let palette = generate_starting_palette(&mut world, &config);

        assert_eq!(
            palette.placed.len(),
            config.worldgen.starting_species_count as usize
        );
        for &(index, _) in &palette.placed {
            let species = &palette.available[index];
            assert_eq!(species.metabolism, Metabolism::Photolithic);
            assert!(
                (config.tags.tags_per_species_min as usize
                    ..=config.tags.tags_per_species_max as usize)
                    .contains(&species.tags.len()),
                "expected 1..=3 tags, got {}",
                species.tags.len()
            );
            for tag in &species.tags {
                assert!(
                    (tag.0 as usize) < world.active_tags.len(),
                    "tag slot {tag:?} out of bounds for this world's active tags"
                );
            }
        }
    }

    #[test]
    fn placed_organisms_land_on_the_grid_at_their_recorded_position() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let palette = generate_starting_palette(&mut world, &config);

        for &(index, (x, y)) in &palette.placed {
            let organism = world.get(x, y).organism.expect("an organism was placed");
            assert_eq!(organism.species, SpeciesId(index as u8));
        }
    }

    #[test]
    fn available_pool_is_larger_than_placed_and_adds_other_metabolisms() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let palette = generate_starting_palette(&mut world, &config);

        assert_eq!(
            palette.available.len(),
            (config.worldgen.starting_species_count + config.worldgen.extra_available_species_count)
                as usize
        );
        assert!(
            palette.available.len() > palette.placed.len(),
            "the available pool must offer more than what's pre-placed (task 046 hook)"
        );
        assert!(
            palette
                .available
                .iter()
                .any(|species| species.metabolism != Metabolism::Photolithic),
            "the available pool should include non-photolithic metabolisms for variety"
        );
    }
}
