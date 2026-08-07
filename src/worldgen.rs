// Procedural world generation (task 038+, GDD §9). This module starts with
// the difficulty curve alone (task 037): a pure function from "which world
// in the run is this" to concrete generation parameters. Tag/environment/
// species/objective generation (later Phase 3 tasks) consume `WorldParams`
// instead of reading `SimConfig`'s early/late endpoints directly, so the
// curve has exactly one source of truth.

use rand::RngExt;

use crate::config::SimConfig;
use crate::objectives::{Objective, ZoneKind};
use crate::world::{draw_species_tags, Metabolism, SimWorld, Species, SpeciesId};

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
/// Replaces Phase 1's `seed_starting_palette` placeholder (task 013, always
/// exactly 2 fixed photolithic species) with a real generator (task 038's
/// world already has its own tag subset/matrix/environment by the time this
/// runs — see `SimWorld::new_for_world`).
///
/// **No organism is placed on the grid** (task 050, 2026-08-06 playtest):
/// every world starts empty, and the player seeds it themselves via `Seed`
/// (GDD §6) — closer to the game's own premise than the world arriving
/// pre-populated. `WorldgenConfig::starting_species_count` species are
/// still generated as `Metabolism::Photolithic` (the only metabolism
/// self-sustaining from light alone with no prey/residue already present,
/// GDD §5.4 — a reasonable "first thing to seed"), with `temp_optimum`
/// spread across the world's actual temperature range the same way
/// `add_bonus_species`'s extras already are, since there's no placement
/// site left to read a real temperature from.
/// `WorldgenConfig::extra_available_species_count` further species are
/// added the same way `add_bonus_species` always worked — giving the
/// player metabolism variety to seed deliberately.
///
/// Every species draws its tags from the world's own active subset
/// (`draw_species_tags`, task 010/036) and its RNG from `world`'s own seeded
/// stream (never an external RNG), so the whole palette stays deterministic
/// given the same world seed.
pub fn generate_starting_palette(world: &mut SimWorld, config: &SimConfig) {
    let cold = world.get(0, 0).temperature;
    let hot = world.get(world.width - 1, 0).temperature;
    let starting_count = config.worldgen.starting_species_count as usize;

    for i in 0..starting_count {
        let weight = if starting_count <= 1 {
            0.5
        } else {
            i as f32 / (starting_count - 1) as f32
        };
        let temp_optimum = cold + (hot - cold) * weight;
        let tags = draw_species_tags(world, config);
        world.species.push(Species {
            metabolism: Metabolism::Photolithic,
            temp_optimum,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags,
        });
    }

    add_bonus_species(world, config, config.worldgen.extra_available_species_count);
}

/// Adds `count` species to `world.species`, available for `Seed` but not
/// placed on the grid — the same generation rule `generate_starting_palette`
/// uses for its own `extra_available_species_count` (`Predator`/`Decomposer`
/// each drawn with equal probability per slot, temperature optimum spread
/// across the world's actual range), factored out so `build_world` can reuse
/// it to apply
/// `MetaProgress::bonus_available_species` (task 046) on top: more species
/// to *choose from*, never anything about the hidden matrix.
pub fn add_bonus_species(world: &mut SimWorld, config: &SimConfig, count: u32) {
    let cold = world.get(0, 0).temperature;
    let hot = world.get(world.width - 1, 0).temperature;
    for _ in 0..count as usize {
        // A per-slot coin flip, not `i % 2` parity: `add_bonus_species` is
        // called independently from `generate_starting_palette` (fixed
        // `extra_available_species_count`) and again from `build_world`
        // (the separate meta-progression bonus, `MetaProgress::
        // bonus_available_species`) — each call restarted its own `i` at 0,
        // so a lone slot (the shipped default `extra_available_species_count
        // = 1`) always landed on index 0 and was deterministically always
        // `Predator`, making `Decomposer` structurally unreachable for an
        // entire run (playtest finding: 4 worlds cleared, never seen).
        let metabolism = if world.rng_mut().random_bool(0.5) {
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
}

/// Which `Objective` variant a candidate draw picked — kept separate from
/// `Objective` itself since the candidate is chosen before its parameters
/// are known (task 042).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectiveKind {
    Coexistence,
    SurviveIn,
    TriggerBloom,
}

/// Scales an `ObjectiveConfig` base value by this world's severity (task
/// 037's `WorldParams::objective_severity`), rounding to the nearest whole
/// unit and never below `1` — a `0`-tick or `0`-population objective would
/// be satisfied instantly, which isn't a requirement at all.
fn scale_severity(base: u32, severity: f32) -> u32 {
    ((base as f32 * severity).round() as u32).max(1)
}

/// Builds one full world of a run — grid, species, matrix, environment
/// (`SimWorld::new_for_world`), starting palette (`generate_starting_palette`),
/// `bonus_available_species` extra species earned via meta-progression
/// (`add_bonus_species`, task 046), and its `Objective` (`generate_objective`)
/// — in the one order each step requires (bonus species join the pool
/// `Objective` generation can pick from; objective generation itself reads
/// whatever species the world ended up with). The single entry point task
/// 044's main menu (first world) and task 045's world transition (every
/// later world) both call, so "how a world comes into being" has exactly
/// one definition.
pub fn build_world(
    seed: u64,
    world_index: u32,
    config: &SimConfig,
    bonus_available_species: u32,
) -> (SimWorld, Objective) {
    let mut world = SimWorld::new_for_world(seed, world_index, config);
    generate_starting_palette(&mut world, config);
    add_bonus_species(&mut world, config, bonus_available_species);
    let params = world_params(world_index, config);
    let objective = generate_objective(&mut world, &params, config);
    (world, objective)
}

/// Chooses and parametrizes this world's `Objective` (task 040, GDD §8/§9),
/// deterministic from the world's own seeded RNG — same seed, same
/// objective. Must run after `generate_starting_palette` (task 039):
/// `Coexistence`'s `min_species` cap and `SurviveIn`/`TriggerBloom`'s
/// `species` pick both read the species this world actually ended up with.
///
/// Coherence (task 042's acceptance criteria) is enforced by construction,
/// not by rejecting a bad draw after the fact: `SurviveIn`'s toxic zone is
/// only ever a candidate when `params` says this world actually has one,
/// and `Coexistence`'s `min_species` is only ever a candidate — then
/// clamped — against the species pool this exact world generated, so it can
/// never ask for more coexisting species than exist to place.
pub fn generate_objective(
    world: &mut SimWorld,
    params: &WorldParams,
    config: &SimConfig,
) -> Objective {
    let severity = params.objective_severity;
    let species_count = world.species.len() as u32;
    let has_toxic_zone = params.toxic_zone_width > 0 && params.toxic_zone_height > 0;

    let mut candidates = vec![ObjectiveKind::TriggerBloom];
    if species_count >= 2 {
        candidates.push(ObjectiveKind::Coexistence);
    }
    if has_toxic_zone {
        candidates.push(ObjectiveKind::SurviveIn);
    }

    let pick = candidates[world.rng_mut().random_range(0..candidates.len())];
    let obj = &config.objectives;
    match pick {
        ObjectiveKind::Coexistence => Objective::Coexistence {
            min_species: scale_severity(obj.coexistence_min_species_base, severity)
                .clamp(2, species_count),
            ticks: scale_severity(obj.coexistence_ticks_base, severity),
        },
        ObjectiveKind::SurviveIn => Objective::SurviveIn {
            species: SpeciesId(world.rng_mut().random_range(0..species_count) as u8),
            zone: ZoneKind::Toxic,
            ticks: scale_severity(obj.survive_in_ticks_base, severity),
        },
        ObjectiveKind::TriggerBloom => Objective::TriggerBloom {
            species: SpeciesId(world.rng_mut().random_range(0..species_count) as u8),
            population_threshold: scale_severity(
                obj.trigger_bloom_population_threshold_base,
                severity,
            ),
        },
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

        generate_starting_palette(&mut a, &config);
        generate_starting_palette(&mut b, &config);

        assert_eq!(a.species, b.species);
    }

    /// Task 050: no world starts with an organism already on the grid — the
    /// player seeds it themselves via `Seed`.
    #[test]
    fn generate_starting_palette_places_no_organisms() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        generate_starting_palette(&mut world, &config);

        assert!(
            world.cells.iter().all(|cell| cell.organism.is_none()),
            "a freshly generated world must start with an empty grid"
        );
        assert!(
            !world.species.is_empty(),
            "species must still be generated, just not placed"
        );
    }

    #[test]
    fn add_bonus_species_extends_the_available_pool_without_placing_any() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        generate_starting_palette(&mut world, &config);
        let species_before = world.species.len();

        add_bonus_species(&mut world, &config, 3);

        assert_eq!(world.species.len(), species_before + 3);
    }

    #[test]
    fn starting_species_are_photolithic_and_carry_valid_tags() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        generate_starting_palette(&mut world, &config);

        let starting_count = config.worldgen.starting_species_count as usize;
        for species in &world.species[..starting_count] {
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
    fn available_pool_size_and_metabolism_variety() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        generate_starting_palette(&mut world, &config);

        assert_eq!(
            world.species.len(),
            (config.worldgen.starting_species_count + config.worldgen.extra_available_species_count)
                as usize
        );
        assert!(
            world
                .species
                .iter()
                .any(|species| species.metabolism != Metabolism::Photolithic),
            "the available pool should include non-photolithic metabolisms for variety"
        );
    }

    #[test]
    fn add_bonus_species_can_produce_both_predator_and_decomposer() {
        // Playtest finding: with the old `i % 2 == 0` rule, `add_bonus_species`
        // always restarted `i` at 0 on every independent call (once from
        // `generate_starting_palette`'s fixed slot, once more from
        // `build_world`'s meta-progression bonus), so a lone slot always
        // landed on `Predator` and `Decomposer` was mathematically
        // unreachable for an entire run. A per-slot coin flip fixes that;
        // sampling many seeds with several slots each pins that both
        // metabolisms are actually reachable, so this can't silently
        // regress back to a fixed pattern.
        let config = SimConfig::default();
        let mut saw_predator = false;
        let mut saw_decomposer = false;
        for seed in 0..50u64 {
            let mut world = SimWorld::new(seed, &config);
            add_bonus_species(&mut world, &config, 8);
            for species in &world.species {
                match species.metabolism {
                    Metabolism::Predator => saw_predator = true,
                    Metabolism::Decomposer => saw_decomposer = true,
                    Metabolism::Photolithic => {}
                }
            }
        }
        assert!(saw_predator, "Predator should be reachable");
        assert!(saw_decomposer, "Decomposer should be reachable");
    }

    #[test]
    fn objective_generation_is_deterministic_for_the_same_seed() {
        let config = SimConfig::default();
        let params = world_params(0, &config);

        let mut a = SimWorld::new(42, &config);
        generate_starting_palette(&mut a, &config);
        let objective_a = generate_objective(&mut a, &params, &config);

        let mut b = SimWorld::new(42, &config);
        generate_starting_palette(&mut b, &config);
        let objective_b = generate_objective(&mut b, &params, &config);

        assert_eq!(objective_a, objective_b);
    }

    /// Task 049: the HUD displays sustained-objective progress in whole
    /// eras (`ui.rs::eras_progress`), so the base tick counts at severity
    /// 1.0 must land exactly on an era boundary — otherwise the very first
    /// world a player sees would show an odd fractional-looking requirement
    /// (e.g. "3.2 eras") that isn't a rounding artifact of severity scaling,
    /// but baked into the defaults themselves.
    #[test]
    fn objective_tick_bases_are_exact_era_multiples_at_base_severity() {
        let config = SimConfig::default();
        assert_eq!(
            config.objectives.coexistence_ticks_base % config.time.era_ticks,
            0,
            "coexistence_ticks_base should be an exact multiple of era_ticks"
        );
        assert_eq!(
            config.objectives.survive_in_ticks_base % config.time.era_ticks,
            0,
            "survive_in_ticks_base should be an exact multiple of era_ticks"
        );
    }

    #[test]
    fn objective_thresholds_grow_with_world_severity() {
        let config = SimConfig::default();

        // A single species and no toxic zone leaves `TriggerBloom` as the
        // only possible candidate, so the comparison below isn't confounded
        // by which variant the RNG happened to draw.
        let build = |severity: f32| {
            let mut world = SimWorld::new(42, &config);
            world.species.push(Species {
                metabolism: Metabolism::Photolithic,
                temp_optimum: 0.5,
                temp_tolerance: config.energy.default_temp_tolerance,
                repro_threshold: config.energy.repro_threshold,
                tags: Vec::new(),
            });
            let mut params = world_params(0, &config);
            params.toxic_zone_width = 0;
            params.toxic_zone_height = 0;
            params.objective_severity = severity;
            generate_objective(&mut world, &params, &config)
        };

        let early = build(config.difficulty.objective_severity_early);
        let late = build(config.difficulty.objective_severity_late);

        let Objective::TriggerBloom {
            population_threshold: early_threshold,
            ..
        } = early
        else {
            panic!("expected TriggerBloom (the only coherent candidate), got {early:?}");
        };
        let Objective::TriggerBloom {
            population_threshold: late_threshold,
            ..
        } = late
        else {
            panic!("expected TriggerBloom (the only coherent candidate), got {late:?}");
        };
        assert!(
            late_threshold > early_threshold,
            "higher severity should raise the threshold: {early_threshold} -> {late_threshold}"
        );
    }

    #[test]
    fn coexistence_min_species_never_exceeds_the_available_species_pool() {
        let config = SimConfig::default();
        for seed in 0..30u64 {
            let mut world = SimWorld::new(seed, &config);
            generate_starting_palette(&mut world, &config);
            let params = world_params(0, &config);
            let species_count = world.species.len() as u32;

            if let Objective::Coexistence { min_species, .. } =
                generate_objective(&mut world, &params, &config)
            {
                assert!(
                    min_species <= species_count,
                    "seed {seed}: min_species {min_species} exceeds the pool of {species_count}"
                );
            }
        }
    }

    #[test]
    fn survive_in_toxic_zone_is_never_chosen_without_a_toxic_zone() {
        let config = SimConfig::default();
        for seed in 0..30u64 {
            let mut world = SimWorld::new(seed, &config);
            generate_starting_palette(&mut world, &config);
            let mut params = world_params(0, &config);
            params.toxic_zone_width = 0;
            params.toxic_zone_height = 0;

            let objective = generate_objective(&mut world, &params, &config);
            assert!(
                !matches!(objective, Objective::SurviveIn { .. }),
                "seed {seed}: SurviveIn picked despite no toxic zone: {objective:?}"
            );
        }
    }

    #[test]
    fn objective_species_reference_is_always_within_the_generated_pool() {
        let config = SimConfig::default();
        for seed in 0..30u64 {
            let mut world = SimWorld::new(seed, &config);
            generate_starting_palette(&mut world, &config);
            let params = world_params(0, &config);
            let species_count = world.species.len() as u32;

            let objective = generate_objective(&mut world, &params, &config);
            match objective {
                Objective::SurviveIn { species, .. } | Objective::TriggerBloom { species, .. } => {
                    assert!(
                        (species.0 as u32) < species_count,
                        "seed {seed}: species {species:?} out of bounds for pool of {species_count}"
                    );
                }
                Objective::Coexistence { .. } => {}
            }
        }
    }
}
