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
    pub cluster: ClusterConfig,
    pub environment: EnvironmentConfig,
    pub time: TimeConfig,
    pub energy: EnergyConfig,
    pub tags: TagConfig,
    pub notebook: NotebookConfig,
    pub difficulty: DifficultyConfig,
    pub worldgen: WorldgenConfig,
    pub objectives: ObjectiveConfig,
    pub terrain: TerrainConfig,
    pub source: SourceConfig,
    pub biome: BiomeConfig,
    pub evolution: EvolutionConfig,
    pub hydrology: HydrologyConfig,
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
    /// Keyboard pan speed, in grid cells per second at `scale == 1.0` (task
    /// 087). Scaled by the camera's current `scale` at use, so panning
    /// covers the same screen-space distance per second at any zoom level,
    /// the same "feels equally responsive" intent as `zoom_speed`.
    pub pan_speed: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            zoom_min: 0.1,
            zoom_max: 1.0,
            zoom_threshold: 0.4,
            zoom_speed: 0.9,
            pan_speed: 120.0,
        }
    }
}

/// Overview mode's per-species cluster heatmap (task 076,
/// `redesign/abiogenesis-two-tier-view.md`, `cluster::compute_cluster_render`;
/// blob shape corrected by task 078).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Cluster cell count at which a blob's density (and therefore
    /// brightness) saturates to `1.0`; population above this reads as
    /// equally maximal, not brighter still. Chosen so a real, established
    /// colony (tens of cells) reads as a clear hot spot while a lone
    /// organism — a one-cell cluster — sits well below saturation and stays
    /// visibly dimmer, the opposite of scoring it as "fully dense" the way
    /// a compactness-only formula (occupied cells / bounding-box area)
    /// would.
    pub density_saturation: f32,
    /// Task 078: a cluster's filled (holes-included) footprint must reach
    /// at least this many cells before erosion touches it at all — below
    /// this, a cluster is already small enough to read as "abstracted"
    /// without shrinking further, and eroding it risks making a lone or
    /// near-lone organism (task 076's own "stays visibly distinct"
    /// acceptance criterion) disappear or read as fainter than it should.
    pub blob_erosion_min_size: u32,
    /// Task 078: how many morphological-erosion passes shrink a large
    /// cluster's filled blob — each pass removes every cell touching the
    /// shape's edge (or the grid's edge). More passes read as more
    /// abstracted/smaller; an iteration that would erode a blob away to
    /// nothing is skipped rather than applied (`compute_blob`'s own guard),
    /// so this can be tuned generously without risking a cluster vanishing.
    pub blob_erosion_iterations: u32,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            density_saturation: 20.0,
            blob_erosion_min_size: 8,
            blob_erosion_iterations: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environmental diffusion rate, fraction per tick (Phase 1+, GDD §5.9).
    pub diffusion_rate: f32,
    /// Light value at the sun-facing end of the per-world directional
    /// falloff, [0,1] (task 085; replaces the old fixed top-bottom gradient
    /// high end — direction now comes from `SimWorld`'s per-world sun draw,
    /// not a fixed axis).
    pub light_high: f32,
    /// Light value at the far end of the directional falloff, [0,1].
    pub light_low: f32,
    /// Fixed temperature at a heat source's center, [0,1] (task 085;
    /// replaces the old left-edge gradient value). `SimWorld::
    /// reinject_environment_sources` pulls source cells back toward this
    /// every tick, so it stays a standing feature despite diffusion.
    pub source_temperature: f32,
    /// Baseline temperature far from any heat source, [0,1] (task 085; the
    /// far end of the per-source falloff, replacing the old fixed-gradient
    /// endpoints).
    pub ambient_temperature: f32,
    /// Toxicity value task 113 imposes on the noise-gated toxic sub-region
    /// of `Biome::Swamp` cells (GDD §5.9; renamed from `toxic_zone_value`
    /// when task 113 removed the old standalone placed-rectangle toxic
    /// zone — this is now the only generation-time source of nonzero
    /// `Cell::toxicity`). Elsewhere it's 0.0.
    pub swamp_toxicity_value: f32,
    /// How much the `Stress` action (GDD §6) shifts a clicked cell's
    /// temperature, before clamping to `[0,1]`. Temperature, not toxicity:
    /// `sim::step`'s `env_fit` reads temperature every tick, so a stressed
    /// cell has an observable, deducible effect on organisms sitting on it —
    /// toxicity is currently written (world generation, diffusion) but read
    /// by nothing in the tick, so stressing it would be an inert action.
    /// Stressing a heat-source cell specifically is overridden the very next
    /// tick by `reinject_environment_sources`' pull back toward
    /// `source_temperature` (task 085) — the same "diffusion erodes it"
    /// caveat as any other cell, just enforced faster and pinned exactly on
    /// source cells; not worth special-casing, since a source cell's
    /// temperature was never a meaningful place to `Stress` in the first
    /// place (it's already the hottest point on the map).
    pub stress_delta: f32,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            diffusion_rate: 0.05,
            light_high: 0.9,
            light_low: 0.2,
            source_temperature: 0.85,
            ambient_temperature: 0.25,
            swamp_toxicity_value: 0.7,
            stress_delta: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    /// Pulses per season (task 135, `redesign/processed/abiogenesis-time-scale-reveal.md`
    /// §1) — the player's unit of decision. Replaces the old `era_ticks` as
    /// the primary knob: the era is now derived from this, not stored
    /// directly (see `era_ticks`).
    pub season_pulses: u32,
    /// Seasons per era (task 135, same doc §1-2). The era is the unit of
    /// narration — much longer and rarer than the season, and derived from
    /// it (`era_ticks`) rather than an independent length.
    pub seasons_per_era: u32,
    /// Era budget for the world at the start of the run (GDD §5.9). Counted
    /// in eras, so task 135 divided it by roughly `seasons_per_era` to keep
    /// the run's total pulse count in the same order as before the split.
    pub era_budget_early: u32,
    /// Era budget for the world at the end of the run (GDD §5.9). See
    /// `era_budget_early`.
    pub era_budget_late: u32,
    /// Action points granted per season (task 135; GDD §5.9 originally
    /// specified this per era — the season is now the refill boundary so the
    /// number of decisions per world stays in the same order of magnitude).
    pub point_budget_per_season: u32,
    pub action_costs: ActionCosts,
    /// Ticks per second during season animation (GDD §4). Presentation
    /// only — changing it must never change simulation outcomes (invariant
    /// 1); it only changes how fast the same `season_pulses` play back.
    pub era_tick_hz: f32,
    /// Onboarding grace period (task 079, GDD §8; unit moved to seasons by
    /// task 135, since the season is the granularity the player actually
    /// experiences): total-extinction failure is suppressed while
    /// `world.season < grace_seasons`, and adaptively extended past that
    /// until the player has kept a population alive for a full season at
    /// least once — see `objectives::is_grace_active`. Deliberately far
    /// smaller than the era budgets, since it only ever gates
    /// total-extinction, never era-budget exhaustion.
    pub grace_seasons: u32,
    /// Onboarding pacing (task 082, unit moved to seasons by task 135):
    /// world 0's seasons `0..onboarding_seasons` use
    /// `onboarding_season_pulses` instead of `season_pulses`, shortening the
    /// wait between checkpoints while the player is still learning the
    /// system. Every other season (any season in any other world, or world 0
    /// past this threshold) keeps the standard `season_pulses`.
    pub onboarding_seasons: u32,
    /// Shortened tick count for world 0's opening seasons (task 082). See
    /// `onboarding_seasons`.
    pub onboarding_season_pulses: u32,
}

impl TimeConfig {
    /// The era's length in pulses, derived from `season_pulses *
    /// seasons_per_era` (task 135) rather than stored directly — the era
    /// stopped being an independently-tunable length once the season became
    /// the unit of decision.
    pub fn era_ticks(&self) -> u32 {
        self.season_pulses * self.seasons_per_era
    }
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            // Kept equal to the pre-135 `era_ticks` (25): every duration
            // tuned against the old era (grace, onboarding, objective tick
            // bases) carries over unchanged onto the season, which now plays
            // the role the era used to.
            season_pulses: 25,
            // GDD's own suggested ratio (redesign doc §2): "if an era is
            // ~4 seasons".
            seasons_per_era: 4,
            era_budget_early: 15,
            era_budget_late: 11,
            point_budget_per_season: 3,
            action_costs: ActionCosts::default(),
            era_tick_hz: 8.0,
            grace_seasons: 3,
            onboarding_seasons: 3,
            onboarding_season_pulses: 8,
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
    /// `Splice`'s action-point cost once `ObjectiveConfig::
    /// splice_upgrade_energy_threshold` is banked (task 109,
    /// `RunProgress::splice_cost`) — cheaper than `splice`, never gates the
    /// action's base availability (unchanged from world 0 onward).
    pub splice_upgraded: u32,
}

impl Default for ActionCosts {
    fn default() -> Self {
        Self {
            seed: 1,
            stress: 1,
            cull: 1,
            splice: 2,
            splice_upgraded: 1,
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
    /// Carrying-capacity penalty per occupied neighbour of a *different*
    /// species (task 136 narrowed this from "any occupied neighbour" — see
    /// `sim::step`'s cost section for why). Same-species density is capped
    /// by the future per-cell carrying capacity (task 137), not this.
    pub crowd_factor: f32,
    /// Energy threshold at which an organism can reproduce.
    pub repro_threshold: f32,
    /// Energy cost passed to the child on reproduction.
    pub repro_cost: f32,
    /// Task 137: the maximum individuals a single cell's population can hold
    /// before the excess must break out to a neighbouring cell (or, with no
    /// valid outlet, feed local selection pressure instead). A new knob, not
    /// a retune of an existing one (task 136's coefficients stay as measured)
    /// — picked, not swept: `repro_threshold / repro_cost = 2` growth events
    /// consume one `repro_threshold`'s worth of aggregate energy each, so a
    /// capacity of `6` lets a population visibly grow through a handful of
    /// crossings before breakout pressure kicks in, without letting a single
    /// cell silently absorb the whole grid's carrying capacity.
    pub cell_carrying_capacity: u32,
    /// Photolithic species: energy gained per tick from metabolism
    /// (`gain = light × metabolism_gain × env_fit`, GDD §5.6). Retuned from
    /// `2.0` by task 136: at `2.0`, an isolated organism in optimal
    /// conditions (`light 0.7`, `env_fit ≈ 1`) nets `0.7 × 2.0 - 0.5 ≈
    /// +0.9`/tick — reproduction threshold in ~6 ticks on environment alone,
    /// making the hidden matrix optional for basic survival.
    ///
    /// The design doc's starting proposal was `0.8` (net `≈ +0.05`/tick).
    /// Measured against `tests/balance.rs`, that margin could not survive
    /// contact with `diffuse_environment`, which keeps eroding a cell's
    /// temperature toward its Moore-neighbourhood mean every tick regardless
    /// of any placement decision: at `0.8` the breakeven `env_fit` for
    /// `light 0.7` is `0.5 / (0.7 × 0.8) ≈ 0.89`, tolerating only `≈0.07` of
    /// temperature drift (`default_temp_tolerance = 0.15`) away from the
    /// exact placement optimum before an organism goes net-negative — and a
    /// 500-tick run measurably drifts further than that even in a
    /// stable-looking toxic patch (observed ≈0.09 drift). Isolated organisms
    /// were dying from ambient drift alone, not surviving-but-not-
    /// reproducing as the design intends: `chemolithotroph_survives_
    /// reasonably_in_its_toxic_zone_across_seeds` lost 62% of seeds, well
    /// past its 30% budget. `1.4` was the smallest value in a `0.8`-to-`2.0`
    /// sweep that brought every `tests/balance.rs` property back under
    /// budget (`1.2`/`1.3` still failed at 32%): breakeven-fit tolerance
    /// `≈0.19`, comfortably past the observed drift, while net `≈
    /// +0.48`/tick is still far short of `2.0`'s effectively-unlimited
    /// margin — a positive matrix relation (see `interaction_scale`) still
    /// meaningfully speeds reproduction up, it just no longer has to rescue
    /// the organism from dying of drift first. The other three metabolisms
    /// are scaled by the same ratio (`÷1.43`) so none of them becomes
    /// disproportionately strong once this one drops.
    pub photolithic_metabolism_gain: f32,
    /// Scale applied to a raw matrix intensity (`TagMatrix::get`, `{-2..2}`)
    /// before it enters the energy update (task 136,
    /// `redesign/processed/abiogenesis-matrix-necessity-balance.md`). Without
    /// this, a `±2` entry was worth `±2.0`/tick — four times `base_upkeep` —
    /// so the hidden matrix could dominate or vanish an organism's energy
    /// balance in a single tick regardless of how well the player read the
    /// visible environment. At `0.15`, one confirmed `+2` relation
    /// (`2 × 0.15 = 0.30`) is worth about the same order of magnitude as one
    /// occupied neighbour's `crowd_factor` cost — a real but not
    /// overwhelming margin, see `photolithic_metabolism_gain`'s own doc
    /// comment for the isolated-baseline math this is calibrated against.
    pub interaction_scale: f32,
    /// Predator species: maximum energy drained from prey per tick. Scaled
    /// by the same ÷1.43 ratio as `photolithic_metabolism_gain` by task 136.
    pub predator_drain_cap: f32,
    /// Predator species: base upkeep per tick.
    pub predator_upkeep: f32,
    /// Decomposer species: energy extracted from residue per tick. Scaled by
    /// the same ÷1.43 ratio as `photolithic_metabolism_gain` by task 136.
    pub decomposer_extract_rate: f32,
    /// Decomposer species: base upkeep per tick.
    pub decomposer_upkeep: f32,
    /// Chemolithotroph species (task 108): energy gained per tick from
    /// `Cell.toxicity`, the same role `photolithic_metabolism_gain` plays
    /// for `light`.
    pub chemolithotroph_metabolism_gain: f32,
    /// Chemolithotroph species: base upkeep per tick.
    pub chemolithotroph_upkeep: f32,
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
            cell_carrying_capacity: 6,
            photolithic_metabolism_gain: 1.4,
            interaction_scale: 0.15,
            predator_drain_cap: 1.4,
            predator_upkeep: 0.7,
            decomposer_extract_rate: 1.05,
            decomposer_upkeep: 0.5,
            // First-pass, tunable (task 108): mirrors `photolithic_metabolism_gain`/
            // `base_upkeep` exactly. `EnvironmentConfig::swamp_toxicity_value`
            // defaults to 0.7 — the same order of magnitude as the ~0.7
            // `light` a photolithic organism sees in a bright cell — so
            // starting from Photolithic's own numbers is the natural
            // balance-comparable baseline until playtesting says otherwise.
            // Scaled by the same ÷1.43 ratio as `photolithic_metabolism_gain`
            // by task 136.
            chemolithotroph_metabolism_gain: 1.4,
            chemolithotroph_upkeep: 0.5,
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
    /// Minimum number of tags a species can carry — a floor on
    /// `draw_species_tags`'s *initial roll* only, not a guarantee on the
    /// tag set it returns: if no zero-net combination exists at this size
    /// in the active pool, the search keeps descending below this minimum,
    /// down to 1 tag if necessary, since the zero-self-interaction
    /// invariant takes precedence (task 088). Invisible under the shipped
    /// default (`1`).
    pub tags_per_species_min: u32,
    /// Maximum number of tags a species can carry.
    pub tags_per_species_max: u32,
    /// Minimum effect intensity for a tag-pair adjacency, e.g. -2.
    pub effect_intensity_min: i8,
    /// Maximum effect intensity for a tag-pair adjacency, e.g. +2.
    pub effect_intensity_max: i8,
    /// Fraction of tag-pair cells in the hidden matrix that are non-zero (~40%).
    pub matrix_density: f32,
    /// How many `TagId`s, out of `global_tag_pool`, are terrain-conditional
    /// in every world (task 096, `redesign/abiogenesis-living-world.md`
    /// §1) — a small, fixed structural fact learned from the manual, not
    /// decoded per world. Keyed on `TagId` (pool-wide identity): the first
    /// `conditional_tag_count` `TagId`s by convention (`TagId(0)`, `TagId(1)`,
    /// ...) are always the conditional ones. *Which* terrain triggers each
    /// one, and whether it's `Mode::Inducible` or `Mode::Repressible`, is
    /// still rolled fresh per world — see `SimWorld::conditional_tags`.
    pub conditional_tag_count: u32,
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
            // Tuned to 4 in task 106 (2026-08-12) — `sim_config.ron` was
            // updated at the time, this hand-written default wasn't; found
            // and fixed 2026-08-19 by `config_ron_sync.rs`'s new drift test.
            conditional_tag_count: 4,
        }
    }
}

/// Task 106's selection-pressure accumulator (`redesign/abiogenesis-evolution-xenotypes.md`):
/// per-stimulus weights and the crossing threshold. First-pass, tunable
/// values — not derived from any GDD formula yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    /// Total accumulated pressure (sum of all three weighted stimuli) needed
    /// to fire `SelectionThresholdCrossed` for a species. Retuned from `20.0`
    /// by task 135: at the old value, measuring `SelectionThresholdCrossed`
    /// events per (now 4x longer) era across 12 greedily-seeded worlds gave a
    /// mean of 0.71 crossings/era with bursts up to 5 in a single era —
    /// several speciations landing in one narrative beat, trivializing both
    /// the `Speciation` objective and the eventual end-of-era reveal (task
    /// 140). `80.0` brings that to a mean of 0.25/era, max 1 in the same
    /// sample, with 75% of eras seeing none at all — speciation reads as a
    /// notable event again, not a per-era routine.
    pub selection_pressure_threshold: f32,
    /// Weight applied to a tick's harmful (negative) `interaction_delta`
    /// share before accumulating.
    pub interaction_harm_weight: f32,
    /// Weight applied to a tick's temperature mismatch (`1.0 - env_fit`)
    /// before accumulating.
    pub terrain_mismatch_weight: f32,
    /// Weight applied to a tick's `Cell::toxicity` exposure before
    /// accumulating.
    pub toxicity_weight: f32,
    /// Hard cap on `world.species.len()` a speciation event (task 107) will
    /// ever push past — a correctness requirement, not merely a balance
    /// knob: `SpeciesId` wraps a `u8` (`world.rs`'s doc comment: "species
    /// are few and never removed"), and a simulation-driven creator has no
    /// `ActionBudget` gate the way player `Splice` does, so nothing else
    /// stops unbounded growth from wrapping/aliasing `SpeciesId`s past
    /// `u8::MAX`. Kept well under that ceiling.
    pub max_species: usize,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            selection_pressure_threshold: 80.0,
            interaction_harm_weight: 1.0,
            terrain_mismatch_weight: 1.0,
            toxicity_weight: 1.0,
            max_species: 40,
        }
    }
}

/// Flow accumulation and rivers (task 127,
/// `redesign/procedural_biome_generation_spec_v2.md` §10): computed once at
/// generation time from `Cell.elevation` + `Cell.rainfall` (task 126).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrologyConfig {
    /// Fraction of non-`Sea` cells marked `Cell.is_river`, by
    /// `flow_accumulation` rank — an **adaptive per-world percentile**, not
    /// a fixed absolute accumulation value: worlds vary a lot in total
    /// rainfall/accumulation scale (a wetter world's median cell can carry
    /// more flow than a drier world's principal river), so a fixed
    /// threshold would either flood a wet world with "rivers" or leave a
    /// dry world with none. Calibrated so a 128x80 grid typically lands in
    /// the spec's own credibility bounds (§10.3: roughly 1-3 principal
    /// rivers of 20-40 cells of path length), checked statistically across
    /// a sample of seeds (`tests`), not guaranteed exactly per seed.
    pub river_top_fraction: f32,
}

impl Default for HydrologyConfig {
    fn default() -> Self {
        Self {
            river_top_fraction: 0.004,
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
            // Tuned to 1.0 in task 106 (2026-08-12) — `sim_config.ron` was
            // updated at the time, this hand-written default wasn't; found
            // and fixed 2026-08-19 by `config_ron_sync.rs`'s new drift test.
            confirmation_threshold: 1.0,
            observation_weight_numerator: 1.0,
        }
    }
}

/// The Phase 3 difficulty curve (task 037, GDD §9): every axis of "how hard
/// is world N" ramps linearly from its early endpoint (read off the
/// existing per-domain config: `TagConfig::active_tags_early`,
/// `TimeConfig::era_budget_early`, `TagConfig::matrix_density`,
/// `SourceConfig`'s temperature gradient) to a late endpoint declared here,
/// over `ramp_worlds` worlds, then holds steady — the run is
/// endless-until-failure (GDD §8), so there is no "final" world to hit an
/// exact late value at.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyConfig {
    /// Number of worlds over which every axis ramps from its early to its
    /// late endpoint. `3` is the smallest value that reproduces GDD §16's
    /// worked example exactly: World 2 (`world_index = 1`) has 6 active
    /// tags, i.e. `active_tags_early(5) + (active_tags_late(8) -
    /// active_tags_early(5)) * 1/3 = 6`.
    pub ramp_worlds: u32,
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
    /// `BiomeConfig::swamp_toxicity_min` at the late endpoint (early end is
    /// `BiomeConfig::swamp_toxicity_min` itself) — task 133: revives GDD
    /// §9's "larger toxic zones" difficulty axis, lost when task 113
    /// removed the old sized `toxic_zone` rectangle. Lower is more toxic:
    /// this is a threshold `wave_band_sum` must exceed, not a size. A
    /// 20-seed measurement found the early default `0.3` yields ~21% of
    /// Swamp cells toxic; `-0.2` (chosen here) yields ~78% — a real
    /// "later worlds are more hostile" step without going fully uniform
    /// (which `-0.4`'s ~92% starts to look like, losing the sub-region
    /// visual variety toxicity imposition was designed to have).
    pub swamp_toxicity_min_late: f32,
}

impl Default for DifficultyConfig {
    fn default() -> Self {
        Self {
            ramp_worlds: 3,
            matrix_density_late: 0.6,
            objective_severity_early: 1.0,
            objective_severity_late: 2.0,
            objective_count_early: 2,
            objective_count_late: 3,
            swamp_toxicity_min_late: -0.2,
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
    /// Wild, pre-existing populations placed directly on the grid at world
    /// generation (task 098, `redesign/abiogenesis-living-world.md` §2a) —
    /// a narrow, documented exception to task 050's "nothing auto-placed"
    /// rule: unlike the player-seedable pool above, these already have a
    /// living organism on the grid before the player acts, hidden away from
    /// the likely starting area for the player to discover. First-pass
    /// tune-by-playtest guess, not derived.
    pub wild_species_count: u32,
    /// Minimum Euclidean distance from the grid's center a wild
    /// population's placement cell must clear (task 098) — a first-pass
    /// stand-in for "not immediately reachable/visible from the player's
    /// likely starting area," since the codebase has no fixed player-start
    /// position to measure against. Tunable, not derived.
    pub wild_species_min_distance_from_center: f32,
    /// Bounded-resample attempts (task 098, same defensive-generation
    /// pattern as `TerrainConfig::max_generation_attempts`) for finding a
    /// placeable cell that also clears `wild_species_min_distance_from_center`; falls back
    /// to the best (farthest) placeable candidate seen if none clears it
    /// within this many draws, so wild placement can never fail outright.
    pub wild_species_placement_attempts: u32,
}

impl Default for WorldgenConfig {
    fn default() -> Self {
        Self {
            starting_species_count: 2,
            extra_available_species_count: 1,
            wild_species_count: 1,
            wild_species_min_distance_from_center: 30.0,
            wild_species_placement_attempts: 30,
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
/// but the player's actual unit of interaction is the season (task 135;
/// `space` advances one `TimeConfig::season_pulses`, `25` by default) — 50
/// ticks cleared in 2 season-presses or less, with no real decision space in
/// between. Both bases are now exact multiples of `season_pulses` (`100` = 4
/// seasons, `75` = 3 seasons) — intentionally exact, not a coincidence, so
/// the HUD's season-formatted progress (`ui.rs::seasons_progress`) never
/// shows an odd fractional-looking requirement at `objective_severity ==
/// 1.0`. Scaling by severity (up to `2.0`, see `DifficultyConfig`) can still
/// land on a non-exact season count for intermediate severities — expected,
/// `seasons_progress` ceils the requirement rather than truncating it.
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
    /// Energy granted to `RunProgress::energy` when any objective clears —
    /// short- or long-term tier alike (task 109,
    /// `redesign/abiogenesis-progression-pacing.md`). First-pass tunable
    /// amount, not derived — pure balance, left open by the design doc.
    pub objective_clear_energy_reward: f32,
    /// `RunProgress::energy` banked threshold at which `Splice`'s upgraded,
    /// cheaper action-point cost (`ActionCosts::splice_upgraded`) becomes
    /// available (task 109) — an unlocked capability, not a spent
    /// currency (GDD §10's "unlock capabilities, not answers"): crossing
    /// this never subtracts from `energy`.
    pub splice_upgrade_energy_threshold: f32,
}

impl Default for ObjectiveConfig {
    fn default() -> Self {
        Self {
            coexistence_min_species_base: 3,
            coexistence_ticks_base: 100,
            survive_in_ticks_base: 75,
            trigger_bloom_population_threshold_base: 8,
            objective_clear_energy_reward: 1.0,
            // 3 objective clears' worth of energy at the default reward —
            // reachable within a single world's short-term sequence plus
            // one more, not locked behind a whole extra world.
            splice_upgrade_energy_threshold: 3.0,
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
    /// detail on top of the continent band (task 069; raised 6→16 by a
    /// 2026-08-10 playtest follow-up — a small wave count makes a handful of
    /// sine waves interfere into a regular, periodic "polka-dot lakes"
    /// pattern instead of organic coastline detail; more waves average
    /// toward a proper noise-like field, per the central-limit argument
    /// `wave_band_sum`'s own `/waves.len()` normalization relies on).
    pub island_wave_count: u32,
    /// Lower bound of the island band's spatial frequency range.
    pub island_freq_min: f32,
    /// Upper bound of the island band's spatial frequency range.
    pub island_freq_max: f32,
    /// Weight applied to the island band's contribution before summing it
    /// with the continent band and normalizing — kept small so the island
    /// band adds detail without swamping the continent-scale shape. Raised
    /// alongside `island_wave_count` (task 085 sea-cooling playtest
    /// follow-up, 2026-08-10) to compensate for more waves' smaller typical
    /// per-band amplitude (each `wave_band_sum` averages over more terms),
    /// so the island band's visual weight stays comparable to before.
    pub island_blend_weight: f32,
    /// Elevation below this becomes `TerrainKind::Sea`. Lowered 0.42→0.34
    /// (2026-08-10 playtest follow-up) alongside the island-band retune
    /// above: the same fixed threshold captured excessive sea coverage
    /// (~33-35% of the grid) once real-world playtesting surfaced it as
    /// implausible, especially now that `Sea` is a real coastal-cooling
    /// source (task 085) — more sea than intended meant more of the map
    /// running cold. Re-verified against `min_placeable_fraction` (the
    /// generation floor, unaffected — placeable land actually rose) and a
    /// 30-seed Sea/Plain/Hill/Mountain/peak histogram (peaks unaffected:
    /// 78 vs 81 baseline over 30 seeds, no collapse).
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
    /// Task 124: divides the raw elevation-gradient magnitude
    /// (`SimWorld::elevation_slope`, a central difference over one grid
    /// step) before clamping to `[0, 1]`, so `Cell.slope` reads as a plain
    /// normalized number future thresholds (task 125) can be tuned against,
    /// instead of a raw per-cell elevation delta whose scale depends on the
    /// wave field's frequency mix.
    pub slope_normalization: f32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            continent_wave_count: 3,
            continent_freq_min: 0.8,
            continent_freq_max: 1.6,
            island_wave_count: 16,
            island_freq_min: 12.0,
            island_freq_max: 18.0,
            island_blend_weight: 0.55,
            sea_threshold: 0.34,
            hill_threshold: 0.62,
            mountain_threshold: 0.8,
            peak_elevation_threshold: 0.88,
            min_placeable_fraction: 0.4,
            max_generation_attempts: 8,
            slope_normalization: 0.05,
        }
    }
}

/// Biome classification (task 110, `redesign/abiogenesis-biomes.md`): a
/// two-stage refinement of `TerrainConfig`'s landform bands into the 11
/// "areal" biomes, using the ambient scalars generation already writes
/// (`temperature`/`light`/`toxicity`). Only covers the areal biomes — the
/// explicitly-placed "feature" biomes (Cratere profondo, Distesa di
/// cristalli, Lago, Bocca vulcanica) are task 111's scope, and their config
/// lives there, not here. Every threshold below is a first-pass baseline
/// from the design doc's target-value table, explicitly not a finished
/// balance pass (the doc itself flags `toxicity`-derived values, in
/// particular, as needing a joint retune with task 108's chemolithotroph
/// metabolism).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeConfig {
    /// Within `TerrainKind::Sea`, elevation below this becomes Acqua
    /// profonda; at/above it becomes Acqua bassa. An absolute elevation
    /// value, not a fraction of `TerrainConfig::sea_threshold` — the raw
    /// `Cell::elevation` field (task 110) this compares against is already
    /// normalized to `[0, 1]` per world by `normalize_elevations`.
    pub deep_water_elevation_max: f32,
    /// Within `TerrainKind::Hill`, `light` at/below this becomes Roccia
    /// nuda instead of Collina — a placeholder stand-in for "low organic
    /// viability" (the design doc's own framing) until task 108's
    /// chemolithotroph retune gives it a firmer basis. Task 125: the
    /// *effective* ceiling a cell is compared against also grows with
    /// `slope` — see `bare_rock_slope_light_bonus`.
    pub bare_rock_light_max: f32,
    /// Task 125 (§12.5): how much `bare_rock_light_max`'s effective
    /// ceiling rises per unit of normalized `slope` — steep terrain reads
    /// as Roccia nuda at a higher light level than flat terrain would need
    /// to. Kept as a small, contained addition to the existing `Hill`
    /// branch; full multi-band mountain classification is a separate,
    /// larger future pass, not this task's scope.
    pub bare_rock_slope_light_bonus: f32,
    /// Within `TerrainKind::Plain`, `temperature` at/above this (together
    /// with `desert_light_min`) becomes Deserto.
    pub desert_temperature_min: f32,
    /// Within `TerrainKind::Plain`, `light` at/above this (together with
    /// `desert_temperature_min`) becomes Deserto.
    pub desert_light_min: f32,
    /// Within `TerrainKind::Plain`, `temperature` at/below this becomes
    /// Tundra (checked after Deserto, so a cell can't satisfy both).
    pub tundra_temperature_max: f32,
    /// How many low-frequency plane waves make up each of Foresta's,
    /// Palude's, and Palude's toxic-sub-region patch mask (task 110/125) —
    /// same summed-plane-wave technique as `TerrainConfig::continent_wave_count`,
    /// reused so per-cell scalar thresholds don't produce checkerboard
    /// speckle instead of organic patches. Each mask draws its own
    /// independent wave set from this many waves, off the same
    /// `BIOME_SEED_OFFSET` stream.
    pub patch_wave_count: u32,
    /// Lower bound of the patch mask's spatial frequency range.
    pub patch_freq_min: f32,
    /// Upper bound of the patch mask's spatial frequency range.
    pub patch_freq_max: f32,
    /// Task 125: half-width of the smooth transition every biome score's
    /// `smoothstep`/`smooth_band` curve uses, in the same `[0, 1]` units as
    /// `temperature`/`light`/normalized `slope` — one shared knob rather
    /// than one per threshold, since there's no reason each biome boundary
    /// should read as sharper or softer than the others.
    pub biome_score_transition_width: f32,
    /// Task 125: amplitude of the small additive patch-noise term
    /// (`wave_band_sum`'s `[-1, 1]` output, positive half only) added to
    /// Forest's and Palude's climate/drainage score before picking an
    /// arg-max — texture on top of the climate fit, not the primary gate
    /// (`redesign/procedural_biome_generation_spec_v2.md` §1.4).
    pub patch_noise_weight: f32,
    /// Task 125: `TerrainKind::Plain`'s baseline score before any of the
    /// other four candidates' scores are compared against it — the
    /// "boring generic terrain" fallback only wins when nothing else fits
    /// well. Deliberately below `1.0` so a cell with even a middling
    /// Desert/Tundra/Forest/Swamp fit still wins the arg-max.
    pub plain_baseline_score: f32,
    /// Within `TerrainKind::Plain` and inside its patch mask, `temperature`
    /// must fall in `[forest_temperature_min, forest_temperature_max]` to
    /// become Foresta.
    pub forest_temperature_min: f32,
    /// See `forest_temperature_min`.
    pub forest_temperature_max: f32,
    /// Within `TerrainKind::Plain` and inside its patch mask, `light` must
    /// fall in `[forest_light_min, forest_light_max]` to become Foresta.
    pub forest_light_min: f32,
    /// See `forest_light_min`.
    pub forest_light_max: f32,
    /// Task 125, refined by task 131: Palude's fitness score is a smooth
    /// rise past this threshold on `Cell.soil_moisture` (task 131's
    /// climate-grounded wetness estimate) — replaces the original
    /// `swamp_slope_max`/`swamp_water_distance_max`/
    /// `swamp_water_distance_falloff` triple, which `soil_moisture`
    /// subsumes rather than sits alongside (both slope and water proximity
    /// are already inputs to it, see `SimWorld::compute_soil_moisture`).
    pub swamp_soil_moisture_min: f32,
    /// Task 131: how much `soil_moisture`'s rainfall-retention term falls
    /// per unit of normalized `slope` — `retention = (1 - this * slope)`,
    /// clamped to `[0, 1]`. Steep terrain sheds rainfall instead of
    /// retaining it.
    pub soil_moisture_retention_slope_weight: f32,
    /// Task 131: a second, independent slope penalty on `soil_moisture` —
    /// the formula's own `drainage(slope, curvature)` term (spec §9.4,
    /// `curvature` out of scope, same simplification task 124 already
    /// made). Distinct from `soil_moisture_retention_slope_weight`: that
    /// one scales how much of the *incoming* rainfall is kept, this one is
    /// a flat subtraction representing ongoing runoff loss — the spec
    /// keeps them as two separate terms, not one.
    pub soil_moisture_drainage_slope_weight: f32,
    /// Task 131: how much `soil_moisture` falls per unit of `temperature`
    /// — the formula's `evaporation(temperature)` term, linear for
    /// simplicity (no evaporation curve data to justify a nonlinear one).
    pub soil_moisture_evaporation_weight: f32,
    /// Task 131: `soil_moisture` bonus at zero distance from the nearest
    /// `Cell.is_river` cell (task 127), scaled by `soil_moisture_river_proximity_max`/
    /// `_falloff` the same way `swamp_water_distance_max`/`_falloff` used
    /// to scale the old drainage proxy's water term.
    pub soil_moisture_river_bonus: f32,
    /// Distance (grid cells) past which `soil_moisture_river_bonus` has
    /// fully fallen to `0`.
    pub soil_moisture_river_proximity_max: f32,
    /// Width, in the same cell units as `soil_moisture_river_proximity_max`,
    /// of the smooth transition on the river-proximity term.
    pub soil_moisture_river_proximity_falloff: f32,
    /// Task 131: `soil_moisture` bonus at zero distance from the nearest
    /// depression `record_significant_depressions` already qualified as a
    /// future `Biome::Lake` (task 129) — read *before* Lake is actually
    /// painted onto any cell, unlike the persisted `Cell.water_distance`
    /// field (see `SimWorld::compute_soil_moisture`'s doc comment for why
    /// that's safe here).
    pub soil_moisture_lake_bonus: f32,
    /// Distance (grid cells) past which `soil_moisture_lake_bonus` has
    /// fully fallen to `0`.
    pub soil_moisture_lake_proximity_max: f32,
    /// Width, in the same cell units as `soil_moisture_lake_proximity_max`,
    /// of the smooth transition on the lake-proximity term.
    pub soil_moisture_lake_proximity_falloff: f32,
    /// Task 125: repurposed from "toxicity level a cell must already have
    /// to read as Palude" (the old gate) to "`wave_band_sum` threshold
    /// selecting which fraction of the cells just classified as Palude get
    /// `toxicity` imposed as a post-classification modifier." Palude's
    /// *identity* no longer depends on toxicity at all
    /// (`swamp_soil_moisture_min` above is the real wetness-based gate);
    /// this only decides which sub-region of an already-classified Palude
    /// reads as toxic, same organic-sub-region idiom the design doc's
    /// §12.4 describes. `EnvironmentConfig::swamp_toxicity_value` (0.7 by
    /// default) is still the only *value* imposed — this field only
    /// selects *where*. Task 133: this is also `WorldParams::
    /// swamp_toxicity_min`'s difficulty-curve early endpoint (late end is
    /// `DifficultyConfig::swamp_toxicity_min_late`) — `classify_biomes`
    /// reads the per-world scaled value from `WorldParams`, not this field
    /// directly, once generation is running.
    pub swamp_toxicity_min: f32,
    /// Radius (cells, Euclidean) around each `SimWorld::heat_sources` cell
    /// that reads as Bocca vulcanica (task 111) — deliberately independent
    /// from `SourceConfig::heat_source_radius`/`heat_source_radius_late`
    /// (that's the temperature falloff footprint, load-bearing for task
    /// 085's balance and not to be perturbed here), and much smaller: a
    /// biome-sized patch, not a whole thermal gradient.
    pub volcanic_vent_radius: f32,
    /// Cratere profondo's placement disk base radius (task 111, organic
    /// shape since task 123), same bounded-retry pattern as
    /// `TerrainConfig`'s toxic-zone fields — now searching over candidate
    /// centers instead of rectangle corners.
    pub crater_radius: f32,
    /// Minimum placeable-land fraction a candidate Cratere profondo
    /// position must clear.
    pub crater_min_placeable_fraction: f32,
    /// Bounded resample attempts for Cratere profondo's placement search.
    pub crater_max_placement_attempts: u32,
    /// Cratere profondo's imposed `temperature` (design doc table).
    pub crater_temperature: f32,
    /// Cratere profondo's imposed `light`.
    pub crater_light: f32,
    /// Cratere profondo's imposed `toxicity`.
    pub crater_toxicity: f32,
    /// Distesa di cristalli's placement disk base radius (task 111, organic
    /// shape since task 123).
    pub crystal_field_radius: f32,
    /// Minimum placeable-land fraction a candidate Distesa di cristalli
    /// position must clear.
    pub crystal_field_min_placeable_fraction: f32,
    /// Bounded resample attempts for Distesa di cristalli's placement
    /// search.
    pub crystal_field_max_placement_attempts: u32,
    /// Distesa di cristalli's imposed `temperature` (design doc table).
    pub crystal_field_temperature: f32,
    /// Distesa di cristalli's imposed `light`.
    pub crystal_field_light: f32,
    /// Distesa di cristalli's imposed `toxicity`.
    pub crystal_field_toxicity: f32,
    /// Lago's placement disk base radius (task 111, organic shape since
    /// task 123). Task 129: this whole organic-search mechanism is now a
    /// **fallback**, used only when depression-derived lakes
    /// (`lake_depression_*` below) don't reach `lake_min_depression_count`
    /// — kept unchanged otherwise, since a low-relief world with too few
    /// real depressions still needs somewhere for Lago to come from.
    pub lake_radius: f32,
    /// Minimum placeable-land fraction a candidate Lago position must
    /// clear (fallback search only, see `lake_radius`).
    pub lake_min_placeable_fraction: f32,
    /// Bounded resample attempts for Lago's placement search (fallback
    /// search only, see `lake_radius`).
    pub lake_max_placement_attempts: u32,
    /// Lago's imposed `temperature` (design doc table).
    pub lake_temperature: f32,
    /// Lago's imposed `light`.
    pub lake_light: f32,
    /// Lago's imposed `toxicity`.
    pub lake_toxicity: f32,
    /// Task 129: minimum size (cells) a `fill_depressions` connected
    /// component must reach to qualify for promotion to `Biome::Lake`.
    /// Filters out single-cell/tiny noise — a real depression floor
    /// component, not a rounding artifact of the priority-flood fill.
    pub lake_depression_min_size: u32,
    /// Task 129: maximum size (cells) a depression component may have and
    /// still qualify as a *Lago-sized* feature, not a whole drainage
    /// basin. A 25-seed measurement found `fill_depressions` components
    /// span from single-digit noise up to 600+ cells (entire valleys
    /// resolved toward the sea) — without this ceiling, the biggest
    /// component on a seed would routinely dwarf `Crater`/`CrystalField`'s
    /// own scale (radius `~3.5-5`, `~40-80` cells) by an order of
    /// magnitude, reading as a flood, not a lake.
    pub lake_depression_max_size: u32,
    /// Task 129: minimum fill depth (`filled_elevation - elevation` at the
    /// component's deepest cell) a depression must reach to qualify —
    /// filters out near-flat "depressions" that are really just
    /// floating-point noise in the priority-flood fill, not a real basin.
    pub lake_depression_min_depth: f32,
    /// Task 129: how many depression-derived lakes a world needs before
    /// the organic-mask fallback search (`lake_radius` etc. above) is
    /// skipped entirely. `1`: any world with at least one qualifying
    /// depression gets its lakes from terrain alone; a low-relief world
    /// with none falls back to the old random search so Lago never goes
    /// missing outright.
    pub lake_min_depression_count: u32,
    /// Task 123: shared angular-distortion amplitude for the deformed-disk
    /// mask used by all three searched feature biomes (Crater/CrystalField/
    /// Lake) — how far the disk's radius at a given angle can stray from
    /// `base_radius`, as a fraction of it. One shared knob rather than three
    /// copies, since there's no reason the three features should look
    /// differently "organic" from each other.
    pub feature_mask_distortion: f32,
    /// Task 123: how many angular sine terms make up the distortion field
    /// (same summed-plane-wave technique as `patch_wave_count`, in
    /// angle-space instead of position-space) — more terms read as a
    /// lumpier, less simply-oval silhouette.
    pub feature_mask_wave_count: u32,
    /// Task 128 (`k` in a `k`-point Voronoi partition of the grid, spec
    /// §11.1): how many macro-regions `classify_biomes` derives before its
    /// per-cell pass, each with one dominant `TerrainKind::Plain`-kind
    /// biome. Small on purpose — this is a coarse bias layer, not a second
    /// classification pass; 4-8 regions read as "a handful of large
    /// climate zones," the spec's own framing.
    pub macro_region_count: u32,
    /// Task 128: multiplicative boost applied to a cell's score for its
    /// own macro-region's dominant biome before arg-max — `score * (1.0 +
    /// this)`. Multiplicative, not additive: task 125's own honest caveat
    /// (`classify_biomes`'s doc comment) is that `smoothstep`/`smooth_band`
    /// *plateau* at exactly `1.0`, so an additive bias would do nothing
    /// wherever a saturated tie is exactly the failure mode this task
    /// exists to fix — multiplying still separates two plateaued `1.0`
    /// scores from each other. Tuned so the region bias wins plausible
    /// ties without becoming an override: a cell whose *local* score for a
    /// different biome is strong enough (spec §11.4's "Swamp nei bacini"
    /// inside a Forest region) still wins on its own unboosted score.
    pub macro_region_bias_weight: f32,
    /// Task 130: `TerrainKind::Hill`'s baseline score before `bare_rock_score`
    /// is compared against it — mirrors `plain_baseline_score`'s role, now
    /// that Hill's BareRock gate is a smooth arg-max instead of a hard
    /// `light <= effective_max` cutoff.
    pub hill_baseline_score: f32,
    /// Task 130: `TerrainKind::Mountain`'s (non-`Peak`) baseline score
    /// before `Glacier`/`AlpineMeadow`/`BareRock`/reused-`Forest` candidate
    /// scores are compared against it — the "plain old Montagna" fallback,
    /// same role as `plain_baseline_score`/`hill_baseline_score`.
    pub mountain_baseline_score: f32,
    /// Task 130: within `TerrainKind::Mountain`, `temperature` at/below this
    /// gives `Glacier` a high score — a smooth fall, mirroring
    /// `tundra_score`'s shape. Elevation itself isn't a separate term here:
    /// every `Mountain` cell already clears `terrain.mountain_threshold`, so
    /// "alta quota" (spec §12.5) is already guaranteed by the terrain kind;
    /// temperature is what actually differentiates Glacier from the other
    /// three candidates within it.
    pub glacier_temperature_max: f32,
    /// Task 130: within `TerrainKind::Mountain`, `temperature` in
    /// `[alpine_meadow_temperature_min, alpine_meadow_temperature_max]`
    /// gives `AlpineMeadow` a high score — the "moderate" band between
    /// Glacier's cold end and Foresta/BareRock's warmer range, using
    /// `smooth_band` the same way Foresta's climate term does.
    pub alpine_meadow_temperature_min: f32,
    /// See `alpine_meadow_temperature_min`.
    pub alpine_meadow_temperature_max: f32,
}

impl Default for BiomeConfig {
    fn default() -> Self {
        Self {
            deep_water_elevation_max: 0.15,
            bare_rock_light_max: 0.4,
            bare_rock_slope_light_bonus: 0.2,
            desert_temperature_min: 0.75,
            desert_light_min: 0.75,
            tundra_temperature_max: 0.2,
            patch_wave_count: 6,
            patch_freq_min: 6.0,
            patch_freq_max: 10.0,
            biome_score_transition_width: 0.05,
            patch_noise_weight: 0.15,
            plain_baseline_score: 0.3,
            forest_temperature_min: 0.35,
            forest_temperature_max: 0.65,
            forest_light_min: 0.25,
            forest_light_max: 0.45,
            swamp_soil_moisture_min: 0.5,
            soil_moisture_retention_slope_weight: 0.6,
            soil_moisture_drainage_slope_weight: 0.2,
            soil_moisture_evaporation_weight: 0.3,
            soil_moisture_river_bonus: 0.3,
            soil_moisture_river_proximity_max: 8.0,
            soil_moisture_river_proximity_falloff: 4.0,
            soil_moisture_lake_bonus: 0.3,
            soil_moisture_lake_proximity_max: 10.0,
            soil_moisture_lake_proximity_falloff: 5.0,
            swamp_toxicity_min: 0.3,
            volcanic_vent_radius: 4.0,
            crater_radius: 4.5,
            crater_min_placeable_fraction: 0.5,
            crater_max_placement_attempts: 24,
            crater_temperature: 0.60,
            crater_light: 0.20,
            crater_toxicity: 0.60,
            crystal_field_radius: 3.5,
            crystal_field_min_placeable_fraction: 0.5,
            crystal_field_max_placement_attempts: 24,
            crystal_field_temperature: 0.40,
            crystal_field_light: 0.60,
            crystal_field_toxicity: 0.20,
            lake_radius: 5.0,
            lake_min_placeable_fraction: 0.5,
            lake_max_placement_attempts: 24,
            lake_temperature: 0.45,
            lake_light: 0.40,
            lake_toxicity: 0.05,
            lake_depression_min_size: 10,
            lake_depression_max_size: 100,
            lake_depression_min_depth: 0.01,
            lake_min_depression_count: 1,
            feature_mask_distortion: 0.35,
            feature_mask_wave_count: 4,
            macro_region_count: 6,
            macro_region_bias_weight: 0.5,
            hill_baseline_score: 0.3,
            mountain_baseline_score: 0.3,
            glacier_temperature_max: 0.28,
            alpine_meadow_temperature_min: 0.28,
            alpine_meadow_temperature_max: 0.45,
        }
    }
}

/// Source-driven temperature/light generation (task 085,
/// `redesign/abiogenesis-environment-sources.md`): replaces the old fixed
/// left-right/top-bottom lerps with point heat sources (+ wind bias, +
/// reinjection) and a per-world sun direction (+ mountain shading). Mirrors
/// `TerrainConfig`'s shape — placement/attempt-count mechanics live here,
/// the scalar endpoint *values* stay in `EnvironmentConfig` alongside the
/// other `[0,1]` environment values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Heat sources placed in a `world_index = 0` world (`WorldParams`'
    /// early endpoint).
    pub heat_source_count_early: u32,
    /// Heat sources placed at `DifficultyConfig::ramp_worlds` and beyond.
    pub heat_source_count_late: u32,
    /// Falloff radius (cells) within which a source's temperature blends
    /// down to `EnvironmentConfig::ambient_temperature`, at the early
    /// endpoint. A *smaller* late radius reads as a harsher, more
    /// concentrated hotspot (GDD §9 "harsher thermal gradients").
    pub heat_source_radius_early: f32,
    /// Heat source falloff radius (cells) at the late endpoint.
    pub heat_source_radius_late: f32,
    /// Per-world wind bias magnitude (cells) at the early endpoint: shifts
    /// each source's effective position downwind before computing distance.
    pub wind_strength_early: f32,
    /// Wind bias magnitude (cells) at the late endpoint.
    pub wind_strength_late: f32,
    /// Minimum cell distance enforced between placed heat sources, so `N`
    /// sources don't cluster into one big hotspot.
    pub heat_source_min_distance: f32,
    /// Bounded-retry attempts per source (the same attempt-loop/
    /// keep-best-seen pattern `generate_terrain` uses) before falling back
    /// to the best candidate seen.
    pub max_heat_source_placement_attempts: u32,
    /// Per-tick pull-back strength toward `EnvironmentConfig::
    /// source_temperature` for heat source cells, and (weighted by
    /// closeness within `sea_coolant_radius`) toward `sea_coolant_value` for
    /// cells near `TerrainKind::Sea` — counteracts `diffuse_environment`'s
    /// erosion so both features persist as standing terrain rather than
    /// fading to the field mean. Must exceed `EnvironmentConfig::
    /// diffusion_rate`, or the pull-back loses to diffusion's own erosion
    /// every tick.
    pub reinjection_strength: f32,
    /// Passive-coolant pull strength applied to a cell's temperature toward
    /// `sea_coolant_value`, weighted by how close the cell is to the nearest
    /// `TerrainKind::Sea` cell within `sea_coolant_radius` (task 085's "Sea
    /// as a passive heat sink", widened from a single-cell Moore ring to a
    /// real coastal band per task 086's playtest — a 1-cell-only pull read
    /// as "the sea isn't influencing anything" at the heatmap's scale).
    /// Deliberately weaker than `reinjection_strength`: this is ambient
    /// coastal cooling, not a pinned source.
    pub sea_coolant_strength: f32,
    /// Fixed cold target temperature Sea's coastal cooling pulls toward.
    pub sea_coolant_value: f32,
    /// Falloff radius (cells, grid/Chebyshev distance) within which
    /// `TerrainKind::Sea`'s coastal cooling blends into the heat-source
    /// field — same falloff shape as `heat_source_radius_*`, applied both at
    /// generation time (`apply_environment_sources`) and every tick
    /// (`reinject_environment_sources`) via `SimWorld::sea_distance`.
    pub sea_coolant_radius: f32,
    /// Mountain/peak light-dimming falloff radius (cells).
    pub mountain_shade_radius: f32,
    /// Maximum light reduction at a peak's own cell, fading to `0` at
    /// `mountain_shade_radius`.
    pub mountain_shade_strength: f32,
    /// Task 126: falloff radius (cells, Sea-only `sea_distance_field`
    /// units) within which `Cell.rainfall`'s ocean-moisture term blends
    /// from `1.0` at the coast down to `0.0` — the same falloff shape
    /// `heat_source_radius_*`/`sea_coolant_radius` already use elsewhere.
    pub rain_ocean_moisture_radius: f32,
    /// Task 126: how strongly `Cell.rainfall`'s orographic-lift term
    /// (the elevation gradient projected onto wind direction) boosts
    /// moisture where terrain rises into the wind.
    pub rain_orographic_lift_strength: f32,
    /// Task 126: number of steps `compute_rainfall`'s upwind ray march
    /// takes per cell when looking for a rain-shadowing ridge — bounds the
    /// single-pass approximation's cost (`O(cells * this)`), not an
    /// iterative solver.
    pub rain_shadow_ray_steps: u32,
    /// Task 126: grid-cell length of each upwind ray-march step.
    /// `rain_shadow_ray_steps * this` is the furthest upwind distance a
    /// ridge can be detected from.
    pub rain_shadow_step_length: f32,
    /// Task 126: how strongly the tallest ridge crossed by the upwind ray
    /// march depletes `Cell.rainfall` — a taller ridge (relative to this
    /// cell's own elevation) casts a stronger rain shadow.
    pub rain_shadow_strength: f32,
    /// Task 122: per-tick pull-back strength toward `EnvironmentConfig::
    /// swamp_toxicity_value` for `SimWorld::toxic_swamp_cells` (the Swamp
    /// cells `classify_biomes` marked toxic at generation time, task 125)
    /// — the same "counteracts `diffuse_environment`'s erosion" role
    /// `reinjection_strength` plays for heat sources, but a **separate**
    /// field: toxicity and temperature diffuse at the same
    /// `EnvironmentConfig::diffusion_rate` today, but there's no reason
    /// their reinjection strengths need to move together, so they stay
    /// independently tunable even though they start equal. Must exceed
    /// `diffusion_rate` for the same reason `reinjection_strength` must.
    pub toxic_reinjection_strength: f32,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            heat_source_count_early: 2,
            heat_source_count_late: 4,
            heat_source_radius_early: 40.0,
            heat_source_radius_late: 25.0,
            wind_strength_early: 5.0,
            wind_strength_late: 15.0,
            heat_source_min_distance: 20.0,
            max_heat_source_placement_attempts: 24,
            reinjection_strength: 0.15,
            sea_coolant_strength: 0.05,
            sea_coolant_value: 0.1,
            sea_coolant_radius: 12.0,
            mountain_shade_radius: 8.0,
            mountain_shade_strength: 0.3,
            rain_ocean_moisture_radius: 40.0,
            rain_orographic_lift_strength: 4.0,
            rain_shadow_ray_steps: 10,
            rain_shadow_step_length: 3.0,
            rain_shadow_strength: 3.0,
            toxic_reinjection_strength: 0.15,
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
