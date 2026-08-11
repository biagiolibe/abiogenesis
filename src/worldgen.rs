// Procedural world generation (task 038+, GDD §9). This module starts with
// the difficulty curve alone (task 037): a pure function from "which world
// in the run is this" to concrete generation parameters. Tag/environment/
// species/objective generation (later Phase 3 tasks) consume `WorldParams`
// instead of reading `SimConfig`'s early/late endpoints directly, so the
// curve has exactly one source of truth.

use rand::RngExt;

use crate::config::SimConfig;
use crate::objectives::{Objective, ZoneKind};
use crate::world::{
    draw_species_name, draw_species_tags, Metabolism, Organism, SimWorld, Species, SpeciesId,
};

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
    /// How many eras this world's run may take before failing (GDD §8: 60 ->
    /// 45 across the curve — raised from the original 40 -> 25 by task 059 to
    /// compensate for `objective_count` growing from 1 to 2-3 per world).
    pub era_budget: u32,
    /// Toxic zone width in cells (GDD §9: "larger toxic zones").
    pub toxic_zone_width: u32,
    /// Toxic zone height in cells.
    pub toxic_zone_height: u32,
    /// Heat sources placed at world generation (task 085, GDD §9: "harsher
    /// thermal gradients" — more, sharper hotspots at later worlds).
    pub heat_source_count: u32,
    /// Heat source falloff radius in cells; *smaller* at later worlds, so
    /// the same source count reads as more concentrated/harsher.
    pub heat_source_radius: f32,
    /// Per-world wind bias magnitude in cells.
    pub wind_strength: f32,
    /// Fraction of tag-pair matrix cells that are non-zero (GDD §9: "meaner
    /// matrix").
    pub matrix_density: f32,
    /// Multiplier task 042 applies to objective thresholds (GDD §9:
    /// "stricter objectives").
    pub objective_severity: f32,
    /// How many objectives (task 059) this world poses in sequence, 2 -> 3
    /// across the curve. A world only clears once every one of them has, in
    /// order — the lever that makes a world last longer without needing an
    /// even harsher single objective.
    pub objective_count: u32,
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
    let source = &config.source;

    WorldParams {
        active_tag_count: lerp_u32(tags.active_tags_early, tags.active_tags_late, t),
        era_budget: lerp_u32(time.era_budget_early, time.era_budget_late, t),
        toxic_zone_width: lerp_u32(env.toxic_zone_width, difficulty.toxic_zone_width_late, t),
        toxic_zone_height: lerp_u32(env.toxic_zone_height, difficulty.toxic_zone_height_late, t),
        heat_source_count: lerp_u32(
            source.heat_source_count_early,
            source.heat_source_count_late,
            t,
        ),
        heat_source_radius: lerp_f32(
            source.heat_source_radius_early,
            source.heat_source_radius_late,
            t,
        ),
        wind_strength: lerp_f32(source.wind_strength_early, source.wind_strength_late, t),
        matrix_density: lerp_f32(tags.matrix_density, difficulty.matrix_density_late, t),
        objective_severity: lerp_f32(
            difficulty.objective_severity_early,
            difficulty.objective_severity_late,
            t,
        ),
        objective_count: lerp_u32(
            difficulty.objective_count_early,
            difficulty.objective_count_late,
            t,
        ),
    }
}

/// Era tick count for a given `(world_index, era)` (task 082): world 0's
/// opening eras (`era < config.time.onboarding_eras`) run shorter, so a
/// player still learning the system hits checkpoints sooner. Every other
/// era — any era in any other world, or world 0 past the threshold — uses
/// the standard `config.time.era_ticks`.
pub fn era_ticks_for(world_index: u32, era: u32, config: &SimConfig) -> u32 {
    let time = &config.time;
    if world_index == 0 && era < time.onboarding_eras {
        time.onboarding_era_ticks
    } else {
        time.era_ticks
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
/// Placeable cells' temperatures, sorted ascending (playtest fix,
/// 2026-08-11) — what generated species' `temp_optimum` should actually
/// draw from. Replaces the old `world.get(0, 0)`/`world.get(width - 1, 0)`
/// corner-sampling bug: those two fixed corners assumed a left-right
/// temperature gradient that hasn't existed since the heat-source model
/// landed (tasks 085/086 — temperature now depends on distance to
/// randomly-placed sources plus sea-coolant blending, not grid position).
/// Sampling two arbitrary corners meant they were very often both close to
/// `ambient`/`sea_coolant_value` (frequently the same `Sea` cell's
/// neighbourhood), so almost every generated species' `temp_optimum` landed
/// in `notebook.rs`'s "cold" band regardless of the world's real range.
/// Restricted to `is_placeable_index` cells (excludes `Sea`/peaks, where
/// nothing can ever stand) — a naive whole-grid min/max instead pulls in
/// `Sea`'s coolant floor and a single heat-source cell's ceiling, handing
/// some species a `temp_optimum` viable almost nowhere any organism could
/// actually be placed.
fn placeable_temperature_distribution(world: &SimWorld) -> Vec<f32> {
    let mut temps: Vec<f32> = world
        .cells
        .iter()
        .enumerate()
        .filter(|&(idx, _)| world.is_placeable_index(idx))
        .map(|(_, cell)| cell.temperature)
        .collect();
    temps.sort_by(f32::total_cmp);
    temps
}

/// Maps a `[0, 1]` weight to a `temp_optimum` drawn from `distribution`'s
/// interior 10th-90th percentile band, not its literal extremes (playtest
/// fix, 2026-08-11): the single coldest/hottest placeable cells are rare
/// outliers (often one cell right at a heat source's edge or the coast) —
/// handing a whole species' `temp_optimum` exactly there makes `env_fit`'s
/// Gaussian falloff (`default_temp_tolerance`) leave it non-viable almost
/// everywhere else on the map. Squeezing into the interior band keeps every
/// generated species viable somewhere non-trivial while still spreading
/// them across the world's real thermal range, not clustering them all near
/// `ambient` the way the old corner-sampling bug did.
fn temp_optimum_at_percentile(distribution: &[f32], weight: f32) -> f32 {
    let squeezed = 0.1 + weight.clamp(0.0, 1.0) * 0.8;
    let idx = ((distribution.len() - 1) as f32 * squeezed).round() as usize;
    distribution[idx]
}

pub fn generate_starting_palette(world: &mut SimWorld, config: &SimConfig) {
    let distribution = placeable_temperature_distribution(world);
    let starting_count = config.worldgen.starting_species_count as usize;

    for i in 0..starting_count {
        let weight = if starting_count <= 1 {
            0.5
        } else {
            i as f32 / (starting_count - 1) as f32
        };
        let temp_optimum = temp_optimum_at_percentile(&distribution, weight);
        let tags = draw_species_tags(world, config);
        let name = draw_species_name(world);
        world.species.push(Species {
            name,
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
    let distribution = placeable_temperature_distribution(world);
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
        let temp_optimum = temp_optimum_at_percentile(&distribution, weight);
        let tags = draw_species_tags(world, config);
        let name = draw_species_name(world);
        world.species.push(Species {
            name,
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
/// (`add_bonus_species`, task 046), and its objective sequence
/// (`generate_objectives`) — in the one order each step requires (bonus
/// species join the pool objective generation can pick from; objective
/// generation itself reads whatever species the world ended up with). The
/// single entry point task 044's main menu (first world) and task 045's
/// world transition (every later world) both call, so "how a world comes
/// into being" has exactly one definition.
pub fn build_world(
    seed: u64,
    world_index: u32,
    config: &SimConfig,
    bonus_available_species: u32,
) -> (SimWorld, Vec<Objective>) {
    let mut world = SimWorld::new_for_world(seed, world_index, config);
    generate_starting_palette(&mut world, config);
    add_bonus_species(&mut world, config, bonus_available_species);
    let params = world_params(world_index, config);
    let objectives = generate_objectives(&mut world, &params, config, world_index);
    // Placed after objective generation (task 098) so wild populations
    // never enter `Coexistence`'s `min_species` clamp or become a
    // `SurviveIn`/`TriggerBloom` target — those draws only ever see the
    // player-seedable pool that existed at generation time.
    place_wild_species(&mut world, config);
    (world, objectives)
}

/// Places `WorldgenConfig::wild_species_count` wild, pre-existing
/// populations directly onto the grid (task 098,
/// `redesign/abiogenesis-living-world.md` §2a) — one organism each, hidden
/// away from the grid's center where the player is most likely to look
/// first. A narrow, documented exception to task 050's "nothing
/// auto-placed" rule: unlike the player-seedable pool, these already have a
/// living organism on the grid before the player acts. Tracked via
/// `SimWorld::wild_species` (not a flag on `Species` itself) so every
/// existing `Species { .. }` construction site in the codebase stays
/// untouched.
///
/// Called after `generate_objectives` (see `build_world`), so these species
/// never influence objective generation, and their organisms are placed
/// directly on `world.cells` without touching `world.ever_populated` —
/// `sim::step`'s population scan excludes wild species from that flip (task
/// 098), preserving task 050's "hasn't been seeded yet" semantics for the
/// player's own lineages.
fn place_wild_species(world: &mut SimWorld, config: &SimConfig) {
    let count = config.worldgen.wild_species_count as usize;
    if count == 0 {
        return;
    }
    let distribution = placeable_temperature_distribution(world);
    let center = (world.width as f32 / 2.0, world.height as f32 / 2.0);
    let min_distance = config.worldgen.wild_species_min_distance_from_center;
    let attempts = config.worldgen.wild_species_placement_attempts;

    for _ in 0..count {
        let tags = draw_species_tags(world, config);
        let name = draw_species_name(world);
        let weight: f32 = world.rng_mut().random_range(0.0..=1.0);
        let temp_optimum = temp_optimum_at_percentile(&distribution, weight);
        let species_id = SpeciesId(world.species.len() as u8);
        world.species.push(Species {
            name,
            metabolism: Metabolism::Photolithic,
            temp_optimum,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags,
        });
        world.wild_species.push(species_id);

        let idx = find_wild_placement(world, center, min_distance, attempts);
        world.cells[idx].organism = Some(Organism {
            species: species_id,
            energy: config.energy.seed_energy,
            born_era: world.era,
        });
    }
}

/// Finds a placeable, unoccupied cell for one wild population (task 098):
/// bounded-resamples random cells (same defensive-generation pattern as
/// `SimWorld::place_toxic_zone`), preferring one at least `min_distance`
/// from `center`, falling back to the farthest placeable candidate seen if
/// none clears that floor within `attempts` draws — placement itself can
/// never fail outright as long as the world has at least one placeable
/// cell, which terrain generation already guarantees.
fn find_wild_placement(
    world: &mut SimWorld,
    center: (f32, f32),
    min_distance: f32,
    attempts: u32,
) -> usize {
    let (width, height) = (world.width, world.height);
    let mut best: Option<(usize, f32)> = None;
    for _ in 0..attempts.max(1) {
        let x = world.rng_mut().random_range(0..width);
        let y = world.rng_mut().random_range(0..height);
        let idx = world.index(x, y);
        if !world.is_placeable_index(idx) || world.get(x, y).organism.is_some() {
            continue;
        }
        let (dx, dy) = (x as f32 - center.0, y as f32 - center.1);
        let distance = (dx * dx + dy * dy).sqrt();
        if distance >= min_distance {
            return idx;
        }
        if best.is_none_or(|(_, best_distance)| distance > best_distance) {
            best = Some((idx, distance));
        }
    }
    best.map(|(idx, _)| idx).unwrap_or_else(|| {
        (0..world.cells.len())
            .find(|&idx| world.is_placeable_index(idx) && world.cells[idx].organism.is_none())
            .expect("a world always has at least one placeable, unoccupied cell")
    })
}

/// Chooses and parametrizes this world's objective sequence (task 040/059,
/// GDD §8/§9): `params.objective_count` objectives (2 -> 3 across the
/// difficulty curve), cleared in order — a world only clears once every one
/// of them has. Deterministic from the world's own seeded RNG — same seed,
/// same sequence. Must run after `generate_starting_palette` (task 039):
/// `Coexistence`'s `min_species` cap and `SurviveIn`/`TriggerBloom`'s
/// `species` pick both read the species this world actually ended up with.
///
/// No two *consecutive* objectives share an `ObjectiveKind` — each slot
/// excludes the previous slot's pick from its own candidates, so a run of
/// e.g. `TriggerBloom, TriggerBloom, TriggerBloom` can't happen even though
/// the RNG alone would occasionally produce it. When only one kind is
/// actually available (e.g. fewer than 2 species and no toxic zone, so only
/// `TriggerBloom` is coherent at all), the exclusion is skipped rather than
/// leaving an empty candidate list.
///
/// `world_index == 0`'s very first slot (task 079) skips the random draw
/// entirely: a brand-new run's opening objective is always the gentlest
/// possible `Coexistence{min_species: 2}`, rather than whatever the RNG
/// picks — which can otherwise land on `SurviveIn` (survive in a hostile
/// zone the player hasn't even seen yet) or a `Coexistence` requiring every
/// generated species including a harder-to-keep-alive Decomposer.
pub fn generate_objectives(
    world: &mut SimWorld,
    params: &WorldParams,
    config: &SimConfig,
    world_index: u32,
) -> Vec<Objective> {
    let mut objectives = Vec::with_capacity(params.objective_count as usize);
    let mut previous_kind = None;
    for i in 0..params.objective_count {
        let (objective, kind) = if i == 0 && world_index == 0 && world.species.len() as u32 >= 2 {
            opening_world_objective(params, config)
        } else {
            generate_one_objective(world, params, config, previous_kind)
        };
        objectives.push(objective);
        previous_kind = Some(kind);
    }
    objectives
}

/// The forced opening objective for `world_index == 0` (task 079): a gentle
/// 2-species `Coexistence`, deterministic (no RNG draw) so it's identical
/// across every seed's very first world. `ticks` still scales with
/// `objective_severity` like any other objective — only `min_species` is
/// hardcoded, overriding the usual `scale_severity(...).clamp(2,
/// species_count)` which can reach 3 at world 0's own severity/species pool.
fn opening_world_objective(params: &WorldParams, config: &SimConfig) -> (Objective, ObjectiveKind) {
    (
        Objective::Coexistence {
            min_species: 2,
            ticks: scale_severity(
                config.objectives.coexistence_ticks_base,
                params.objective_severity,
            ),
        },
        ObjectiveKind::Coexistence,
    )
}

/// Coherence (task 042's acceptance criteria) is enforced by construction,
/// not by rejecting a bad draw after the fact: `SurviveIn`'s toxic zone is
/// only ever a candidate when `params` says this world actually has one,
/// and `Coexistence`'s `min_species` is only ever a candidate — then
/// clamped — against the species pool this exact world generated, so it can
/// never ask for more coexisting species than exist to place.
fn generate_one_objective(
    world: &mut SimWorld,
    params: &WorldParams,
    config: &SimConfig,
    exclude: Option<ObjectiveKind>,
) -> (Objective, ObjectiveKind) {
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
    if candidates.len() > 1 {
        candidates.retain(|&kind| Some(kind) != exclude);
    }

    let pick = candidates[world.rng_mut().random_range(0..candidates.len())];
    let obj = &config.objectives;
    let objective = match pick {
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
    };
    (objective, pick)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn era_ticks_for_shortens_world_zeros_opening_eras() {
        let config = SimConfig::default();

        for era in 0..config.time.onboarding_eras {
            assert_eq!(
                era_ticks_for(0, era, &config),
                config.time.onboarding_era_ticks
            );
        }
    }

    #[test]
    fn era_ticks_for_uses_standard_length_past_the_onboarding_threshold() {
        let config = SimConfig::default();

        assert_eq!(
            era_ticks_for(0, config.time.onboarding_eras, &config),
            config.time.era_ticks
        );
        assert_eq!(
            era_ticks_for(0, config.time.onboarding_eras + 5, &config),
            config.time.era_ticks
        );
    }

    #[test]
    fn era_ticks_for_uses_standard_length_for_other_worlds_at_any_era() {
        let config = SimConfig::default();

        assert_eq!(era_ticks_for(1, 0, &config), config.time.era_ticks);
        assert_eq!(era_ticks_for(2, 1, &config), config.time.era_ticks);
    }

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
            params.heat_source_count,
            config.source.heat_source_count_early
        );
        assert_eq!(
            params.heat_source_radius,
            config.source.heat_source_radius_early
        );
        assert_eq!(params.wind_strength, config.source.wind_strength_early);
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
            at_ramp_end.heat_source_count,
            config.source.heat_source_count_late
        );
        assert_eq!(
            at_ramp_end.heat_source_radius,
            config.source.heat_source_radius_late
        );
        assert_eq!(at_ramp_end.wind_strength, config.source.wind_strength_late);
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

    /// Task 095: species names are drawn from the world's own seeded RNG,
    /// not derived from `SpeciesId` alone — two different seeds must be able
    /// to produce different names for "species 0" (the old id-indexed
    /// scheme always picked the same name here, every world, every seed).
    #[test]
    fn starting_species_names_vary_across_seeds() {
        let config = SimConfig::default();
        let mut saw_a_difference = false;
        for seed in 0..20u64 {
            let mut a = SimWorld::new(seed, &config);
            let mut b = SimWorld::new(seed + 1000, &config);
            generate_starting_palette(&mut a, &config);
            generate_starting_palette(&mut b, &config);
            if a.species[0].name != b.species[0].name {
                saw_a_difference = true;
                break;
            }
        }
        assert!(
            saw_a_difference,
            "species 0's name should vary across at least some seeds"
        );
    }

    /// Task 050: no world starts with an organism already on the grid — the
    /// player seeds it themselves via `Seed`.
    #[test]
    fn place_wild_species_puts_exactly_configured_count_of_organisms_on_the_grid() {
        let config = SimConfig::default();
        let (world, _) = build_world(42, 0, &config, 0);

        let placed_wild = world
            .cells
            .iter()
            .filter(|cell| cell.organism.is_some_and(|o| world.is_wild(o.species)))
            .count();
        assert_eq!(placed_wild, config.worldgen.wild_species_count as usize);
        assert_eq!(
            world.wild_species.len(),
            config.worldgen.wild_species_count as usize
        );
    }

    #[test]
    fn wild_species_never_land_on_non_placeable_terrain() {
        let config = SimConfig::default();
        for seed in 0..20u64 {
            let (world, _) = build_world(seed, 0, &config, 0);
            for &species in &world.wild_species {
                let idx = world
                    .cells
                    .iter()
                    .position(|cell| cell.organism.is_some_and(|o| o.species == species))
                    .expect("every wild species has exactly one placed organism");
                assert!(
                    world.is_placeable_index(idx),
                    "seed {seed}: wild species {species:?} landed on non-placeable terrain"
                );
            }
        }
    }

    #[test]
    fn wild_species_placement_is_deterministic_for_the_same_seed() {
        let config = SimConfig::default();
        let (a, _) = build_world(42, 0, &config, 0);
        let (b, _) = build_world(42, 0, &config, 0);

        assert_eq!(a.wild_species, b.wild_species);
        let wild_cells_of = |world: &SimWorld| -> Vec<usize> {
            world
                .cells
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.organism.is_some_and(|o| world.is_wild(o.species)))
                .map(|(idx, _)| idx)
                .collect()
        };
        assert_eq!(wild_cells_of(&a), wild_cells_of(&b));
        for (id_a, id_b) in a.wild_species.iter().zip(b.wild_species.iter()) {
            assert_eq!(world_species(&a, *id_a).tags, world_species(&b, *id_b).tags);
            assert_eq!(
                world_species(&a, *id_a).temp_optimum,
                world_species(&b, *id_b).temp_optimum
            );
        }
    }

    fn world_species(world: &SimWorld, id: SpeciesId) -> &Species {
        &world.species[id.0 as usize]
    }

    #[test]
    fn wild_species_do_not_flip_ever_populated() {
        let config = SimConfig::default();
        let (world, _) = build_world(42, 0, &config, 0);

        assert!(
            !world.wild_species.is_empty(),
            "the default config places at least one wild species"
        );
        assert!(
            !world.ever_populated,
            "wild placement alone must not flip ever_populated (task 050's semantics)"
        );
    }

    #[test]
    fn wild_species_are_excluded_from_the_coexistence_pool_at_generation_time() {
        // Wild species are placed after `generate_objectives` (see
        // `build_world`), so they must never appear as the species count
        // that decision saw — verified indirectly here by confirming the
        // world's non-wild species count still matches the pre-098
        // palette size the objective generation actually used.
        let config = SimConfig::default();
        let (world, _) = build_world(42, 0, &config, 0);
        let non_wild = world.species.len() - world.wild_species.len();
        assert_eq!(
            non_wild,
            (config.worldgen.starting_species_count + config.worldgen.extra_available_species_count)
                as usize,
            "wild species must be additional to, not counted within, the objective-generation pool"
        );
    }

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
        let objectives_a = generate_objectives(&mut a, &params, &config, 0);

        let mut b = SimWorld::new(42, &config);
        generate_starting_palette(&mut b, &config);
        let objectives_b = generate_objectives(&mut b, &params, &config, 0);

        assert_eq!(objectives_a, objectives_b);
    }

    #[test]
    fn objective_count_ramps_from_two_to_three() {
        let config = SimConfig::default();
        assert_eq!(
            world_params(0, &config).objective_count,
            config.difficulty.objective_count_early
        );
        assert_eq!(
            world_params(config.difficulty.ramp_worlds, &config).objective_count,
            config.difficulty.objective_count_late
        );
    }

    #[test]
    fn generated_objectives_never_repeat_the_same_kind_consecutively() {
        let config = SimConfig::default();
        for seed in 0..50u64 {
            let mut world = SimWorld::new(seed, &config);
            generate_starting_palette(&mut world, &config);
            // A late-endpoint world both has a toxic zone and enough species
            // for every `ObjectiveKind` to be a live candidate every slot —
            // otherwise a run of repeats could be the *only* coherent
            // sequence rather than evidence of a missing exclusion.
            let params = world_params(config.difficulty.ramp_worlds, &config);
            let objectives =
                generate_objectives(&mut world, &params, &config, config.difficulty.ramp_worlds);

            for pair in objectives.windows(2) {
                assert_ne!(
                    std::mem::discriminant(&pair[0]),
                    std::mem::discriminant(&pair[1]),
                    "seed {seed}: consecutive objectives share a kind: {pair:?}"
                );
            }
        }
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
                name: "Test".to_string(),
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
            generate_one_objective(&mut world, &params, &config, None).0
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

            for objective in generate_objectives(&mut world, &params, &config, 0) {
                if let Objective::Coexistence { min_species, .. } = objective {
                    assert!(
                        min_species <= species_count,
                        "seed {seed}: min_species {min_species} exceeds the pool of {species_count}"
                    );
                }
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

            for objective in generate_objectives(&mut world, &params, &config, 0) {
                assert!(
                    !matches!(objective, Objective::SurviveIn { .. }),
                    "seed {seed}: SurviveIn picked despite no toxic zone: {objective:?}"
                );
            }
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

            for objective in generate_objectives(&mut world, &params, &config, 0) {
                match objective {
                    Objective::SurviveIn { species, .. }
                    | Objective::TriggerBloom { species, .. } => {
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

    /// Task 079: world 0's opening objective is always the gentlest possible
    /// `Coexistence`, and — unlike every other slot — deterministic across
    /// every seed, since the forced branch consumes no RNG.
    #[test]
    fn world_zero_first_objective_is_always_gentle_coexistence() {
        let config = SimConfig::default();
        let expected_ticks = scale_severity(
            config.objectives.coexistence_ticks_base,
            config.difficulty.objective_severity_early,
        );

        for seed in 0..30u64 {
            let (_, objectives) = build_world(seed, 0, &config, 0);
            assert_eq!(
                objectives[0],
                Objective::Coexistence {
                    min_species: 2,
                    ticks: expected_ticks,
                },
                "seed {seed}: world 0's first objective must always be a gentle 2-species coexistence"
            );
        }
    }
}
