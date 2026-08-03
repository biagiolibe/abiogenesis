// Fields are read by the systems added in later tasks (003+); the config
// itself is complete and correct before it has any readers.
#![allow(dead_code)]

use bevy::prelude::*;

/// Every simulation coefficient in one place (GDD §5.9). Read-only at runtime;
/// hot-reload is a later-phase concern (TECH_DESIGN.md §4).
#[derive(Resource, Debug, Clone, Default)]
pub struct SimConfig {
    pub grid: GridConfig,
    pub environment: EnvironmentConfig,
    pub time: TimeConfig,
    pub energy: EnergyConfig,
    pub tags: TagConfig,
    pub notebook: NotebookConfig,
}

#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Grid width in cells (GDD §5.9).
    pub width: u32,
    /// Grid height in cells (GDD §5.9).
    pub height: u32,
    /// Moore neighborhood size (8-connected), fixed by design.
    pub neighborhood_size: u8,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            width: 48,
            height: 32,
            neighborhood_size: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    /// Environmental diffusion rate, fraction per tick (Phase 1+, GDD §5.9).
    pub diffusion_rate: f32,
    /// Light gradient value at the high end, [0,1] (GDD §5.9).
    pub light_gradient_high: f32,
    /// Light gradient value at the low end, [0,1] (GDD §5.9).
    pub light_gradient_low: f32,
    /// Temperature gradient value on the left edge, [0,1] (GDD §5.9).
    pub temperature_gradient_left: f32,
    /// Temperature gradient value on the right edge, [0,1] (GDD §5.9).
    pub temperature_gradient_right: f32,
    /// Toxicity value inside the toxic zone, [0,1] (GDD §5.9). Elsewhere it's 0.0.
    pub toxic_zone_value: f32,
    /// Width in cells of the Phase 0 toxic zone (bottom-right corner). Not in
    /// the GDD baseline table; kept here rather than hand-written in `world.rs`.
    pub toxic_zone_width: u32,
    /// Height in cells of the Phase 0 toxic zone.
    pub toxic_zone_height: u32,
    /// How much the `Stress` action (GDD §6) shifts a clicked cell's
    /// temperature, before clamping to `[0,1]`. Temperature, not toxicity:
    /// `sim::step`'s `env_fit` reads temperature every tick, so a stressed
    /// cell has an observable, deducible effect on organisms sitting on it —
    /// toxicity is currently written (world generation, diffusion) but read
    /// by nothing in the tick, so stressing it would be an inert action.
    pub stress_delta: f32,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            diffusion_rate: 0.05,
            light_gradient_high: 0.9,
            light_gradient_low: 0.2,
            temperature_gradient_left: 0.2,
            temperature_gradient_right: 0.8,
            toxic_zone_value: 0.7,
            toxic_zone_width: 8,
            toxic_zone_height: 6,
            stress_delta: 0.3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeConfig {
    /// Ticks per era (GDD §5.9).
    pub era_ticks: u32,
    /// Era budget for the world at the start of the run (GDD §5.9).
    pub era_budget_early: u32,
    /// Era budget for the world at the end of the run (GDD §5.9).
    pub era_budget_late: u32,
    /// Action points granted per era (GDD §5.9).
    pub point_budget_per_era: u32,
    pub action_costs: ActionCosts,
    /// Ticks per second during era animation (GDD §4). Presentation only —
    /// changing it must never change simulation outcomes (invariant 1); it
    /// only changes how fast the same `era_ticks` play back.
    pub era_tick_hz: f32,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            era_ticks: 25,
            era_budget_early: 40,
            era_budget_late: 25,
            point_budget_per_era: 3,
            action_costs: ActionCosts::default(),
            era_tick_hz: 20.0,
        }
    }
}

/// Action point cost per player action (GDD §5.9).
#[derive(Debug, Clone)]
pub struct ActionCosts {
    pub seed: u32,
    pub stress: u32,
    pub cull: u32,
    pub splice: u32,
}

impl Default for ActionCosts {
    fn default() -> Self {
        Self {
            seed: 1,
            stress: 1,
            cull: 1,
            splice: 2,
        }
    }
}

/// Per-organism energy and metabolism coefficients (GDD §5.9).
#[derive(Debug, Clone)]
pub struct EnergyConfig {
    /// Energy an organism starts with when seeded.
    pub seed_energy: f32,
    /// Base maintenance cost per tick.
    pub base_upkeep: f32,
    /// Carrying-capacity penalty per occupied neighbour.
    pub crowd_factor: f32,
    /// Energy threshold at which an organism can reproduce.
    pub repro_threshold: f32,
    /// Energy cost passed to the child on reproduction.
    pub repro_cost: f32,
    /// Photolithic species: energy gained per tick from metabolism.
    pub photolithic_metabolism_gain: f32,
    /// Predator species: maximum energy drained from prey per tick.
    pub predator_drain_cap: f32,
    /// Predator species: base upkeep per tick.
    pub predator_upkeep: f32,
    /// Decomposer species: energy extracted from residue per tick.
    pub decomposer_extract_rate: f32,
    /// Decomposer species: base upkeep per tick.
    pub decomposer_upkeep: f32,
    /// Residue energy deposited when an organism dies.
    pub residue_on_death: f32,
    /// Residue decay rate per tick.
    pub residue_decay: f32,
    /// Default temperature tolerance (σ) for species without an explicit override.
    pub default_temp_tolerance: f32,
}

impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            seed_energy: 5.0,
            base_upkeep: 0.5,
            crowd_factor: 0.15,
            repro_threshold: 10.0,
            repro_cost: 5.0,
            photolithic_metabolism_gain: 2.0,
            predator_drain_cap: 2.0,
            predator_upkeep: 0.7,
            decomposer_extract_rate: 1.5,
            decomposer_upkeep: 0.5,
            residue_on_death: 3.0,
            residue_decay: 0.2,
            default_temp_tolerance: 0.15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TagConfig {
    /// Total number of tags available across the whole game (GDD §5.9).
    pub global_tag_pool: u32,
    /// Active tags in the world at the start of the run.
    pub active_tags_early: u32,
    /// Active tags in the world at the end of the run.
    pub active_tags_late: u32,
    /// Minimum number of tags a species can carry.
    pub tags_per_species_min: u32,
    /// Maximum number of tags a species can carry.
    pub tags_per_species_max: u32,
    /// Minimum effect intensity for a tag-pair adjacency, e.g. -2.
    pub effect_intensity_min: i8,
    /// Maximum effect intensity for a tag-pair adjacency, e.g. +2.
    pub effect_intensity_max: i8,
    /// Fraction of tag-pair cells in the hidden matrix that are non-zero (~40%).
    pub matrix_density: f32,
}

impl Default for TagConfig {
    fn default() -> Self {
        Self {
            global_tag_pool: 10,
            active_tags_early: 5,
            active_tags_late: 8,
            tags_per_species_min: 1,
            tags_per_species_max: 3,
            effect_intensity_min: -2,
            effect_intensity_max: 2,
            matrix_density: 0.4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NotebookConfig {
    /// Cumulative evidence needed to confirm a matrix cell (GDD §7).
    pub confirmation_threshold: f32,
    /// Numerator of the observation-weight formula: weight = numerator / (1 + n_adjacent_confounders).
    pub observation_weight_numerator: f32,
}

impl Default for NotebookConfig {
    fn default() -> Self {
        Self {
            confirmation_threshold: 3.0,
            observation_weight_numerator: 1.0,
        }
    }
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimConfig>();
    }
}
