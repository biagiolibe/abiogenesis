// Fields are read by the systems added in later tasks (003+); the config
// itself is complete and correct before it has any readers.
#![allow(dead_code)]

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};

/// Every simulation coefficient in one place (GDD §5.9). Loaded from
/// `assets/config/sim_config.ron` (task 073) with hot-reload: editing that
/// file while `cargo run` is active updates the live resource without a
/// restart, via `sync_sim_config_on_reload` below. `impl Default` still
/// hand-mirrors the RON file's values, so tests can build a `SimConfig`
/// without spinning up Bevy's asset machinery — keep the two in sync by
/// hand when tuning either one.
#[derive(Asset, Resource, TypePath, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimConfig {
    pub grid: GridConfig,
    pub camera: CameraConfig,
    pub environment: EnvironmentConfig,
    pub time: TimeConfig,
    pub energy: EnergyConfig,
    pub tags: TagConfig,
    pub notebook: NotebookConfig,
    pub difficulty: DifficultyConfig,
    pub worldgen: WorldgenConfig,
    pub objectives: ObjectiveConfig,
    pub terrain: TerrainConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            width: 128,
            height: 80,
            neighborhood_size: 8,
        }
    }
}

/// Zoom camera tuning (task 075, `redesign/abiogenesis-two-tier-view.md`).
/// Applies to `OrthographicProjection::scale`: `1.0` is the un-zoomed
/// `ScalingMode::AutoMin` framing that exactly fits the whole grid (task
/// 074's zoomed-out floor), smaller values zoom in. Kept here rather than a
/// `render.rs` constant (unlike `CELL_SIZE`) because these need iterative
/// playtesting to feel right, the exact reason `SimConfig` moved to a
/// hot-reloadable RON asset in task 073.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    /// Smallest allowed `scale` (furthest zoom-in, most detail).
    pub zoom_min: f32,
    /// Largest allowed `scale` (furthest zoom-out); `1.0` matches the
    /// whole-grid `AutoMin` framing, so this also caps zoom-out at "see the
    /// whole grid," never past it into empty space.
    pub zoom_max: f32,
    /// `scale` threshold below which `MapViewMode` switches to `Detail`
    /// (individual organism rendering); at or above it, `Overview` (cluster
    /// heatmap, task 076) is active. A hard cutoff, not a blend
    /// (`redesign/abiogenesis-two-tier-view.md`).
    pub zoom_threshold: f32,
    /// Multiplicative zoom step per unit of mouse-wheel scroll: each scroll
    /// unit multiplies `scale` by `zoom_speed` (scrolling in) or divides by
    /// it (scrolling out), so zoom feels equally responsive at any current
    /// zoom level rather than a fixed additive step.
    pub zoom_speed: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            zoom_min: 0.1,
            zoom_max: 1.0,
            zoom_threshold: 0.4,
            zoom_speed: 0.9,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            // Scaled with grid size (task 074, 48x32 -> 128x80): kept the
            // same relative footprint as the original 8x6 (~3.1% of cells).
            toxic_zone_width: 21,
            toxic_zone_height: 15,
            stress_delta: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            era_budget_early: 60,
            era_budget_late: 45,
            point_budget_per_era: 3,
            action_costs: ActionCosts::default(),
            era_tick_hz: 8.0,
        }
    }
}

/// Action point cost per player action (GDD §5.9).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Ambient background residue gained by every cell each tick,
    /// independent of organism deaths. Must stay strictly below
    /// `residue_decay` so residue reaches a small stable equilibrium
    /// instead of growing unboundedly. Keeps an isolated Decomposer from
    /// starving out immediately, without making it self-sufficient.
    pub residue_ambient_trickle: f32,
    /// Default temperature tolerance (σ) for species without an explicit override.
    pub default_temp_tolerance: f32,
    /// How much the `Splice` action (GDD §6) shifts a new species'
    /// `temp_optimum` away from its source's, before clamping to `[0,1]`.
    pub splice_temp_shift: f32,
    /// Minimum change in a species' average energy between two consecutive
    /// eras for the HUD's per-era population trend indicator (task 063) to
    /// call it `Rising`/`Falling` rather than `Stable` — presentation-only,
    /// doesn't affect simulation behavior, but stays a named constant here
    /// rather than a UI-side literal (no magic numbers, TECH_DESIGN.md
    /// invariant 3).
    pub trend_epsilon: f32,
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
            residue_ambient_trickle: 0.05,
            default_temp_tolerance: 0.15,
            splice_temp_shift: 0.15,
            trend_epsilon: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// How many times `draw_species_tags` redraws a candidate tag set that
    /// nets a negative self-interaction (playtest finding: a species whose
    /// own tags drain each other dies as soon as it reproduces, since a
    /// child always spawns adjacent to a parent with the identical genome)
    /// before giving up and accepting the least-bad draw seen.
    pub max_self_conflict_draws: u32,
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
            max_self_conflict_draws: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// The Phase 3 difficulty curve (task 037, GDD §9): every axis of "how hard
/// is world N" ramps linearly from its early endpoint (read off the
/// existing per-domain config: `TagConfig::active_tags_early`,
/// `TimeConfig::era_budget_early`, `TagConfig::matrix_density`,
/// `EnvironmentConfig`'s toxic-zone size and temperature gradient) to a late
/// endpoint declared here, over `ramp_worlds` worlds, then holds steady —
/// the run is endless-until-failure (GDD §8), so there is no "final" world
/// to hit an exact late value at.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyConfig {
    /// Number of worlds over which every axis ramps from its early to its
    /// late endpoint. `3` is the smallest value that reproduces GDD §16's
    /// worked example exactly: World 2 (`world_index = 1`) has 6 active
    /// tags, i.e. `active_tags_early(5) + (active_tags_late(8) -
    /// active_tags_early(5)) * 1/3 = 6`.
    pub ramp_worlds: u32,
    /// Toxic zone width at the late endpoint (early end is
    /// `EnvironmentConfig::toxic_zone_width`).
    pub toxic_zone_width_late: u32,
    /// Toxic zone height at the late endpoint (early end is
    /// `EnvironmentConfig::toxic_zone_height`).
    pub toxic_zone_height_late: u32,
    /// Temperature gradient spread (`right - left`) at the late endpoint;
    /// the early spread is derived from `EnvironmentConfig`'s existing
    /// `temperature_gradient_left`/`temperature_gradient_right`.
    pub temperature_spread_late: f32,
    /// Matrix density at the late endpoint (early end is
    /// `TagConfig::matrix_density`).
    pub matrix_density_late: f32,
    /// Objective severity multiplier at the run's start (task 042 scales
    /// objective thresholds by this; task 037 only produces the number).
    pub objective_severity_early: f32,
    /// Objective severity multiplier at the late endpoint.
    pub objective_severity_late: f32,
    /// How many objectives a world poses in sequence at the run's start
    /// (task 059) — a world only clears once every one of them has, in
    /// order.
    pub objective_count_early: u32,
    /// How many objectives a world poses in sequence at the late endpoint.
    pub objective_count_late: u32,
}

impl Default for DifficultyConfig {
    fn default() -> Self {
        Self {
            ramp_worlds: 3,
            // Scaled with grid size (task 074, 48x32 -> 128x80), same
            // relative footprint as the original 16x12 (~12.5% of cells).
            toxic_zone_width_late: 43,
            toxic_zone_height_late: 30,
            temperature_spread_late: 1.0,
            matrix_density_late: 0.6,
            objective_severity_early: 1.0,
            objective_severity_late: 2.0,
            objective_count_early: 2,
            objective_count_late: 3,
        }
    }
}

/// Starting-species-pool generation (task 039, GDD §9 "available starting
/// species"). Kept separate from `TagConfig`/`EnergyConfig` since it governs
/// *how many* species `worldgen::generate_starting_palette` creates, not a
/// per-species genome coefficient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldgenConfig {
    /// Species pre-placed on the grid when a world starts. Always generated
    /// as `Metabolism::Photolithic` — the only metabolism that's
    /// self-sustaining from light alone, with no prey or residue required
    /// to already exist (GDD §5.4). Matches Phase 1's `seed_starting_palette`
    /// placeholder count.
    pub starting_species_count: u32,
    /// Extra species added to the "available" pool (selectable via `Seed`,
    /// GDD §6) but not pre-placed on the grid — gives the player
    /// non-photolithic metabolism variety to seed deliberately, without
    /// every starting organism needing prey/residue to survive its first
    /// ticks.
    pub extra_available_species_count: u32,
}

impl Default for WorldgenConfig {
    fn default() -> Self {
        Self {
            starting_species_count: 2,
            extra_available_species_count: 1,
        }
    }
}

/// Base parameters for the three `Objective` variants (task 040, GDD §8),
/// at `WorldParams::objective_severity == 1.0` (the run's early worlds,
/// task 037) — `worldgen::generate_objective` (task 042) scales these by
/// the current world's severity, never reads them unscaled.
/// `WorldgenConfig`'s default species count (2 + 1 = 3) makes
/// `coexistence_min_species_base` achievable.
///
/// `coexistence_ticks_base`/`survive_in_ticks_base` (task 049, 2026-08-06
/// playtest): GDD §8's original worked example read "50 ticks" literally,
/// but the player's actual unit of interaction is the era (`space` advances
/// one `TimeConfig::era_ticks`, `25` by default) — 50 ticks cleared in 2
/// era-presses or less, with no real decision space in between. Both bases
/// are now exact multiples of `era_ticks` (`100` = 4 eras, `75` = 3 eras) —
/// intentionally exact, not a coincidence, so the HUD's era-formatted
/// progress (`ui.rs::eras_progress`) never shows an odd fractional-looking
/// requirement at `objective_severity == 1.0`. Scaling by severity (up to
/// `2.0`, see `DifficultyConfig`) can still land on a non-exact era count
/// for intermediate severities — expected, `eras_progress` ceils the
/// requirement rather than truncating it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveConfig {
    /// `Objective::Coexistence`'s `min_species` at severity 1.0.
    pub coexistence_min_species_base: u32,
    /// `Objective::Coexistence`'s `ticks` at severity 1.0 — 4 eras.
    pub coexistence_ticks_base: u32,
    /// `Objective::SurviveIn`'s `ticks` at severity 1.0 — 3 eras.
    pub survive_in_ticks_base: u32,
    /// `Objective::TriggerBloom`'s `population_threshold` at severity 1.0.
    pub trigger_bloom_population_threshold_base: u32,
}

impl Default for ObjectiveConfig {
    fn default() -> Self {
        Self {
            coexistence_min_species_base: 3,
            coexistence_ticks_base: 100,
            survive_in_ticks_base: 75,
            trigger_bloom_population_threshold_base: 8,
        }
    }
}

/// Procedural terrain generation (task 066, `redesign/abiogenesis-terrain-map.md`):
/// elevation is real per-cell simulation data, not a decorative value, so its
/// thresholds live here like every other simulation coefficient (no magic
/// numbers). `Sea` is deliberately not called out as "impassable" anywhere in
/// this config — that's a placement-gating decision (task 067), kept
/// separate so a future aquatic species doesn't require touching generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConfig {
    /// How many low-frequency plane waves shape the macro-continent scale
    /// of the elevation field (task 069) — same dependency-free noise
    /// technique as `render.rs`'s decorative background layer, but this
    /// field is real simulation data.
    pub continent_wave_count: u32,
    /// Lower bound of the continent band's spatial frequency range.
    pub continent_freq_min: f32,
    /// Upper bound of the continent band's spatial frequency range.
    pub continent_freq_max: f32,
    /// How many higher-frequency plane waves layer small island/coastline
    /// detail on top of the continent band (task 069).
    pub island_wave_count: u32,
    /// Lower bound of the island band's spatial frequency range.
    pub island_freq_min: f32,
    /// Upper bound of the island band's spatial frequency range.
    pub island_freq_max: f32,
    /// Weight applied to the island band's contribution before summing it
    /// with the continent band and normalizing — kept small so the island
    /// band adds detail without swamping the continent-scale shape.
    pub island_blend_weight: f32,
    /// Elevation below this becomes `TerrainKind::Sea`.
    pub sea_threshold: f32,
    /// Elevation at/above `sea_threshold` and below this becomes `Plain`.
    pub hill_threshold: f32,
    /// Elevation at/above `hill_threshold` and below this becomes `Hill`;
    /// at/above it becomes `Mountain`.
    pub mountain_threshold: f32,
    /// Minimum elevation, within `Mountain`, for a cell to even be eligible
    /// as a peak (in addition to being a local maximum among its Moore
    /// neighbours) — keeps peaks confined to a mountain's actual summit
    /// rather than marking every local wobble along its foot.
    pub peak_elevation_threshold: f32,
    /// Minimum fraction of the grid that must classify as placeable
    /// (`Plain`, `Hill`, or non-peak `Mountain`; `Sea` never counts) for a
    /// generated terrain to be accepted outright.
    pub min_placeable_fraction: f32,
    /// Bounded resample attempts (whole elevation field, not per-cell — a
    /// per-cell resample would destroy the organic shape) if a draw's
    /// placeable fraction falls short of `min_placeable_fraction`. The best
    /// draw seen is kept if none clears the floor within this many
    /// attempts, same defensive-generation spirit as tasks 047/048.
    pub max_generation_attempts: u32,
    /// Minimum fraction of the toxic zone's own footprint that must be
    /// placeable land, once a candidate position is tried — keeps the
    /// `SurviveIn` objective satisfiable regardless of where terrain
    /// generation put the sea/mountains.
    pub min_toxic_zone_placeable_fraction: f32,
    /// Bounded resample attempts when searching for a toxic zone position
    /// meeting `min_toxic_zone_placeable_fraction`. The best position seen
    /// is kept if none clears the floor within this many attempts.
    pub max_toxic_zone_placement_attempts: u32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            continent_wave_count: 3,
            continent_freq_min: 0.8,
            continent_freq_max: 1.6,
            island_wave_count: 6,
            island_freq_min: 12.0,
            island_freq_max: 18.0,
            island_blend_weight: 0.45,
            sea_threshold: 0.42,
            hill_threshold: 0.62,
            mountain_threshold: 0.8,
            peak_elevation_threshold: 0.88,
            min_placeable_fraction: 0.4,
            max_generation_attempts: 8,
            min_toxic_zone_placeable_fraction: 0.5,
            max_toxic_zone_placement_attempts: 24,
        }
    }
}

/// Holds the handle to the loaded `SimConfig` RON asset (task 073) so
/// `sync_sim_config_on_reload` can tell, on every `AssetEvent`, whether the
/// event is about *this* config file rather than some other asset.
#[derive(Resource)]
struct SimConfigHandle(Handle<SimConfig>);

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // `SimConfig::default()` hand-mirrors `assets/config/sim_config.ron`
        // (see the struct's doc comment), so the resource has a correct
        // value immediately — the async asset load below then overwrites it
        // once the file is actually read, and again on every hot-reload.
        app.init_resource::<SimConfig>()
            .add_plugins(RonAssetPlugin::<SimConfig>::new(&["ron"]))
            .add_systems(Startup, load_sim_config)
            .add_systems(Update, sync_sim_config_on_reload);
    }
}

fn load_sim_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("config/sim_config.ron");
    commands.insert_resource(SimConfigHandle(handle));
}

/// Keeps the live `SimConfig` resource in sync with the loaded RON asset:
/// fires once when the initial load completes, then again every time the
/// file changes on disk while `cargo run` is active (`file_watcher`
/// feature, enabled in `Cargo.toml`) — hot-reload with no restart needed.
fn sync_sim_config_on_reload(
    mut events: MessageReader<AssetEvent<SimConfig>>,
    handle: Option<Res<SimConfigHandle>>,
    assets: Res<Assets<SimConfig>>,
    mut config: ResMut<SimConfig>,
) {
    let Some(handle) = handle else {
        return;
    };
    for event in events.read() {
        let is_this_config = match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => *id == handle.0.id(),
            _ => false,
        };
        if !is_this_config {
            continue;
        }
        if let Some(loaded) = assets.get(handle.0.id()) {
            *config = loaded.clone();
        }
    }
}
