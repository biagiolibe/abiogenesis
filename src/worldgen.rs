// Procedural world generation (task 038+, GDD §9). This module starts with
// the difficulty curve alone (task 037): a pure function from "which world
// in the run is this" to concrete generation parameters. Tag/environment/
// species/objective generation (later Phase 3 tasks) consume `WorldParams`
// instead of reading `SimConfig`'s early/late endpoints directly, so the
// curve has exactly one source of truth.

use crate::config::SimConfig;

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
}
