// Species/organism fields are read by later tasks (004+); the domain types
// are complete and correct before the tick algorithm consumes them.
#![allow(dead_code)]

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::RngExt;
use rand::SeedableRng;

use crate::config::{
    BiomeConfig, EnvironmentConfig, SimConfig, SourceConfig, TagConfig, TerrainConfig,
};
use crate::worldgen::{world_params, WorldParams};

/// How a species derives energy (GDD §5.4). Only `Photolithic` is active in
/// Phase 0; the other variants exist now to avoid a refactor in Phase 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metabolism {
    Photolithic,
    Predator,
    Decomposer,
    /// Draws energy from `Cell.toxicity` (task 108, GDD §5.4's deferred
    /// item) the same way `Photolithic` draws from `light` — a per-cell
    /// scalar read directly, no shared-resource pre-pass needed.
    Chemolithotroph,
}

/// Identity of a tag in the *global* pool of `TagConfig::global_tag_pool`
/// tags (GDD §5.5). Opaque to the player. Stable across worlds — used only
/// where a tag's name/color/glyph is resolved (`text.rs`/`ui.rs`/`render.rs`),
/// never to index `TagMatrix`/`MatrixKnowledge` directly (see `TagSlot`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagId(pub u8);

/// Position of a tag within *this world's* active subset (task 036),
/// distinct from `TagId`'s pool-wide identity. `SimWorld::active_tags: Vec<TagId>`
/// is the only slot→identity map — `active_tags[slot.0 as usize]` recovers the
/// `TagId` for display. Every matrix/evidence lookup (`TagMatrix::get`,
/// `MatrixKnowledge`) is keyed by `TagSlot`, not `TagId`, so a world can pick
/// a non-contiguous subset of the global pool (Phase 3 world generation,
/// task 038) without breaking the array indexing below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagSlot(pub u8);

/// Index into `SimWorld::species`. Kept small: species are few and never removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeciesId(pub u8);

/// How a terrain-conditional tag's default participation switches (task
/// 096, operon regulation as the biochemical grounding: *lac* = inducible,
/// *trp* = repressible). `Inducible` is silent unless the carrying
/// organism's cell matches the trigger terrain; `Repressible` is the
/// mirror — active everywhere except the trigger terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Inducible,
    Repressible,
}

/// One terrain-conditional `TagId`'s per-world roll (task 096): which
/// terrain triggers it and which `Mode` it rolled. *Which* `TagId`s are
/// conditional at all is a fixed, structural, cross-world constant (the
/// first `TagConfig::conditional_tag_count` `TagId`s by convention) — only
/// this struct's `terrain`/`mode` vary per world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConditionalTag {
    pub tag: TagId,
    pub terrain: TerrainKind,
    pub mode: Mode,
}

/// The secret tag x tag adjacency matrix (GDD §5.5): `get(exerter, receiver)`
/// is the energy delta `receiver` gets from being adjacent to `exerter`. The
/// diagonal is always `0` — a tag has no effect on itself, not stated
/// explicitly in the GDD but implied by every worked example (§16.1).
///
/// Indexed by `TagSlot`, this world's active-subset position, not `TagId`
/// (task 036) — the matrix only ever has as many rows/columns as this world
/// has active tags, regardless of which slice of the global pool they are.
#[derive(Debug, Clone, PartialEq)]
pub struct TagMatrix {
    // `pub(crate)`, not private: sim.rs's tests build hand-crafted matrices
    // to verify the adjacency effect (task 012) against known values.
    pub(crate) size: usize,
    pub(crate) values: Vec<i8>,
}

impl TagMatrix {
    pub fn get(&self, exerter: TagSlot, receiver: TagSlot) -> i8 {
        self.values[exerter.0 as usize * self.size + receiver.0 as usize]
    }

    /// Builds a matrix from a row-major `size x size` grid of effect values.
    /// Public so cross-crate callers (e.g. the binary's `input.rs`, which
    /// can't see `TagMatrix`'s `pub(crate)` fields) can hand-craft a known
    /// matrix for tests, the same way this crate's own tests do via the
    /// fields directly.
    pub fn from_values(size: usize, values: Vec<i8>) -> Self {
        assert_eq!(values.len(), size * size, "matrix values must be size*size");
        Self { size, values }
    }
}

/// A species' genome (GDD §5.3): metabolism and environmental range are
/// player-readable, tags are opaque and drive matrix interactions.
#[derive(Debug, Clone, PartialEq)]
pub struct Species {
    /// Display name (task 095), drawn once from the world's own seeded RNG
    /// at construction (`draw_species_name`) and stored — never recomputed
    /// from `SpeciesId`, so it stays stable for the species' whole lifetime
    /// and varies per world/seed the way tags/matrix/terrain already do. A
    /// `Splice`-derived child draws its own independent name, not a copy of
    /// its parent's.
    pub name: String,
    pub metabolism: Metabolism,
    pub temp_optimum: f32,
    pub temp_tolerance: f32,
    pub repro_threshold: f32,
    /// 1..=3 tags (GDD §5.3), as positions in the owning world's active
    /// subset (`TagSlot`), not global pool identities.
    pub tags: Vec<TagSlot>,
}

/// A living instance of a species occupying a cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Organism {
    pub species: SpeciesId,
    pub energy: f32,
    /// The season this organism was born in (task 083; moved from era to
    /// season by task 135, since this gates decision-cadence timing, not
    /// narration). Reproduction requires `born_season < world.season` — an
    /// organism must survive into a later season than its own birth before
    /// it can reproduce.
    pub born_season: u32,
}

/// A cell's elevation band (task 066, `redesign/abiogenesis-terrain-map.md`):
/// real per-cell simulation data, not a decorative visual value — a
/// possible future factor in evolution alongside others TBD. Deliberately
/// doesn't encode "can a species live here" itself (that's placement
/// gating, task 067, via a single centralized check) so a future aquatic
/// species doesn't require touching this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TerrainKind {
    Sea,
    #[default]
    Plain,
    Hill,
    Mountain,
}

/// Number of `TerrainKind` variants — the width of any per-terrain array
/// (`TerrainOccupancy`'s bitmask, `notebook.rs`'s `TerrainKnowledge` rows,
/// task 106's `SelectionPressure::terrain_mismatch`). One definition so the
/// variant-to-index mapping can't drift between them.
pub const TERRAIN_KIND_COUNT: usize = 4;

impl TerrainKind {
    /// This variant's position in any `[T; TERRAIN_KIND_COUNT]` array.
    pub fn index(self) -> usize {
        match self {
            TerrainKind::Sea => 0,
            TerrainKind::Plain => 1,
            TerrainKind::Hill => 2,
            TerrainKind::Mountain => 3,
        }
    }

    /// Inverse of `index` — panics on an out-of-range index, which should
    /// never happen since every caller derives `index` from this same enum.
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => TerrainKind::Sea,
            1 => TerrainKind::Plain,
            2 => TerrainKind::Hill,
            3 => TerrainKind::Mountain,
            _ => panic!("terrain index out of range: {index}"),
        }
    }
}

/// A cell's discrete biome (task 110, `redesign/abiogenesis-biomes.md`):
/// refines the base `TerrainKind` landform with the ambient scalars
/// (`temperature`/`light`/`toxicity`) into one of 16 design-doc biomes.
/// The 11 "areal" biomes emerge from elevation + scalars
/// (`SimWorld::classify_biomes`, task 110); the 4 "feature" biomes below
/// (Cratere profondo, Distesa di cristalli, Lago, Bocca vulcanica) are
/// placed explicitly instead (`SimWorld::place_feature_biomes`, task 111),
/// overriding whatever the areal pass produced there. Geyser (task 114)
/// stays blocked. Decided once at generation time and stored here, never
/// recomputed from live `Cell` scalars — the same reasoning
/// `objectives.rs`'s `SurviveIn` gives for why it checks `Cell::biome`
/// instead of live `toxicity` (task 113): `diffuse_environment` blends
/// scalars toward neighbours every tick, so a threshold read at query time
/// would drift away from the biome's actual shape over a long-running
/// world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Biome {
    DeepWater,
    ShallowWater,
    #[default]
    Plain,
    Hill,
    Mountain,
    Peak,
    Desert,
    Tundra,
    BareRock,
    /// Ghiacciaio (task 130): `TerrainKind::Mountain`'s cold-temperature
    /// sub-band — reachable only within Mountain terrain, alongside
    /// `AlpineMeadow`/`BareRock`/reused-`Forest`.
    Glacier,
    /// Prateria alpina (task 130): `TerrainKind::Mountain`'s
    /// moderate-temperature sub-band, between `Glacier`'s cold end and the
    /// warmer range `Forest`/`BareRock` compete in.
    AlpineMeadow,
    Forest,
    Swamp,
    /// Cratere profondo (task 111): bounded-retry rectangle placement,
    /// same pattern as the toxic zone.
    Crater,
    /// Distesa di cristalli (task 111): bounded-retry rectangle placement
    /// — the design doc's "visually alien" biome, a candidate hook for a
    /// rare genetic tag in a future task, not resolved here.
    CrystalField,
    /// Lago (task 111): bounded-retry rectangle placement — a standalone
    /// inland body of still water, distinct from `DeepWater`/`ShallowWater`
    /// (which come from `TerrainKind::Sea`).
    Lake,
    /// Bocca vulcanica (task 111): no placement search — directly hooks
    /// into `SimWorld::heat_sources` (task 085), which already exist by
    /// generation time.
    VolcanicVent,
}

/// Which `TerrainKind` bands a species' lineage has ever occupied this run
/// (task 099) — a tiny bitmask over `TerrainKind`'s 4 variants, one entry
/// per `SpeciesId` in `SimWorld::terrain_occupancy`. Deliberately named and
/// shaped generically (not `RevealTracker`/reveal-specific): task 106's
/// speciation trigger is expected to reuse this same structure for its own
/// "has this lineage been exposed to a terrain" check, not build a parallel
/// one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TerrainOccupancy(u8);

impl TerrainOccupancy {
    fn bit(kind: TerrainKind) -> u8 {
        match kind {
            TerrainKind::Sea => 0b0001,
            TerrainKind::Plain => 0b0010,
            TerrainKind::Hill => 0b0100,
            TerrainKind::Mountain => 0b1000,
        }
    }

    pub fn has(&self, kind: TerrainKind) -> bool {
        self.0 & Self::bit(kind) != 0
    }

    /// Sets `kind`'s bit, returning whether this call newly set it (i.e.
    /// this is the first time this terrain has been marked occupied).
    pub fn mark(&mut self, kind: TerrainKind) -> bool {
        let bit = Self::bit(kind);
        let newly_set = self.0 & bit == 0;
        self.0 |= bit;
        newly_set
    }
}

/// Per-species accumulated selection pressure (task 106,
/// `redesign/abiogenesis-evolution-xenotypes.md`), one entry per `SpeciesId`
/// in `SimWorld::selection_pressure`, grown lazily the same way as
/// `terrain_occupancy`. Lineage-scoped (per `SpeciesId`, shared by every
/// individual of that species), not per-organism: `Organism` stays a small
/// `Copy` snapshot value (`sim.rs`'s tick relies on cheaply copying it out of
/// the grid every iteration) and the doc's own framing talks about a
/// *lineage* being repeatedly exposed to a stimulus, not one individual's
/// luck.
///
/// Deliberately a new, separate accumulator, not a `MatrixKnowledge` reuse —
/// that one tracks tag-pair evidence, this tracks organism/lineage stimulus
/// exposure. Also deliberately separate from `TerrainOccupancy`: that one is
/// a "has this lineage ever set foot here" bitmask with no magnitude, so it
/// can't carry a pressure value — this one *is* the magnitude, keyed the
/// same way (`TerrainKind::index`) so the two stay parallel without
/// duplicating tracking.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SelectionPressure {
    /// Accumulated harm from negative `interaction_delta` (GDD §5.6 step 3)
    /// — only the harmful (negative) share counts, so a lineage that's
    /// mostly helped by adjacency never accrues pressure from this stimulus.
    pub interaction_harm: f32,
    /// Accumulated temperature mismatch (`1.0 - env_fit`), bucketed by the
    /// `TerrainKind` occupied at the time — task 107 needs to know not just
    /// how much mismatch pressure built up, but *where*, to shift
    /// `temp_optimum` toward the terrain actually occupied.
    pub terrain_mismatch: [f32; TERRAIN_KIND_COUNT],
    /// Accumulated `Cell::toxicity` exposure.
    pub toxicity: f32,
    /// Set once this species' total pressure has crossed
    /// `EvolutionConfig::selection_pressure_threshold` — mirrors
    /// `MatrixKnowledge::record`'s `was_confirmed` guard: once true, this
    /// species' crossing never fires `SelectionThresholdCrossed` again, even
    /// though the tallies above keep accumulating.
    pub crossed: bool,
}

impl SelectionPressure {
    pub fn total(&self) -> f32 {
        self.interaction_harm + self.terrain_mismatch.iter().sum::<f32>() + self.toxicity
    }
}

/// Per-cell adjacency-evidence onset tracking (task 136b,
/// `SimWorld::adjacency_exposure`, one entry per grid cell, sized once at
/// construction like `cells` — the grid never resizes). `sim::step` used to
/// emit `AdjacencyObserved` for every occupied Moore neighbour, every tick,
/// for as long as the adjacency held — a blob sitting still for 200 ticks
/// counted as 200 independent observations of the same fact, the opposite of
/// GDD §7's "the isolated observation is the valuable one." This tracks which
/// exerter tags were already adjacent as of the last tick this cell's
/// organism was processed, so evidence only accrues on a tag's *onset* —
/// the transition into adjacency, not its continued presence.
///
/// Deliberately keyed by cell, not folded into `Organism` (which stays a
/// small `Copy` snapshot, see `SelectionPressure`'s own doc comment): an
/// organism never moves, so "this cell's history" and "this organism's
/// history" are the same thing for as long as the same organism occupies it.
/// `owner_born_season` is how a stale history gets discarded when a
/// different organism (a different birth) later takes the same cell — this
/// task's own note that task 137's per-cell population model will give this
/// a cleaner home; this is the simplest correct version for today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AdjacencyExposure {
    /// `Organism::born_season` of whichever organism this exposure state
    /// belongs to. `None` before any organism has ever occupied this cell.
    /// A mismatch against the current organism's `born_season` means a
    /// different organism now holds the cell, so the stored mask is stale
    /// and must be treated as empty rather than reused.
    pub owner_born_season: Option<u32>,
    /// Bitmask over `TagSlot` indices: bit `n` set means `TagSlot(n)` was
    /// adjacent to this cell's organism as of the last tick it was
    /// processed. `u32` comfortably covers `TagConfig::global_tag_pool`
    /// (`10` by default) with room to spare.
    pub exerter_tags: u32,
}

/// One grid cell. Single occupancy (GDD §5.1): `organism` is never a collection.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Cell {
    pub temperature: f32,
    pub light: f32,
    pub toxicity: f32,
    pub organism: Option<Organism>,
    /// Dead matter left behind, feeds decomposers from Phase 1 (GDD §5.6 step 6).
    pub residue: f32,
    /// This cell's elevation band (task 066). Defaults to `Plain` — the
    /// ordinary placeable band — so every existing call site building cells
    /// via `Cell::default()` before terrain generation runs stays placeable.
    pub terrain: TerrainKind,
    /// Whether this cell is a local-maximum summit within `Mountain` (task
    /// 066) — decided once at generation time and stored here, not
    /// re-derived later by rendering or gameplay code. Peaks are
    /// unplaceable regardless of `terrain` (task 067).
    pub is_peak: bool,
    /// Raw normalized elevation (task 110), kept alongside the coarser
    /// `terrain` band it was classified from — `classify_elevation`
    /// otherwise discards this value once it picks a `TerrainKind`, leaving
    /// no way to split `TerrainKind::Sea` into Acqua profonda/Acqua bassa by
    /// depth. Mirrors `is_peak`: one extra bit of generation-time data kept
    /// around instead of derived from distance-to-land at query time.
    pub elevation: f32,
    /// This cell's discrete biome (task 110), classified once at generation
    /// time. See `Biome`'s own doc comment for why it isn't re-derived live.
    pub biome: Biome,
    /// Local elevation-gradient magnitude (task 124), normalized to
    /// `[0, 1]` via `TerrainConfig::slope_normalization` — a plain
    /// "how steep is this cell" number. Computed once at generation time
    /// by `SimWorld::compute_slope`, right after `generate_terrain` (pure
    /// function of `elevation`, no further dependency) — early enough that
    /// `classify_biomes` reads this field directly (task 132, resolved
    /// 2026-08-19; originally it computed its own local copy, since task
    /// 124 first placed this computation after `place_feature_biomes`).
    pub slope: f32,
    /// Grid (Moore-neighbour-step) distance to the nearest water cell
    /// (`Biome::DeepWater`/`ShallowWater`/`Lake`) — task 124's
    /// generalization of `SimWorld::sea_distance_field`'s Sea-only BFS to
    /// every water source, including inland lakes. Kept as a separate field
    /// rather than folded into `sea_distance_field` itself: that field
    /// already feeds `apply_environment_sources`' coastal-cooling model,
    /// and widening its source set to lakes would be an unreviewed change
    /// to the temperature balance, out of scope here. Computed once at
    /// generation time by `SimWorld::compute_water_distance`, after
    /// `place_feature_biomes` (its BFS needs `Biome::Lake` cells to exist
    /// as a source). **Still read nowhere** as this persisted field:
    /// `classify_biomes` and `compute_rainfall` both run *before* Lake
    /// exists, so they each use a locally-computed Sea-only proxy instead
    /// (task 132's resolved decision — unlike `slope`, this field's
    /// dependency on Lake genuinely can't move earlier without a bigger
    /// restructure, see `tasks/done/132-*.md`).
    pub water_distance: f32,
    /// Task 126: `[0, 1]` precipitation proxy — ocean-proximity moisture,
    /// orographic lift (terrain rising into the wind), and rain shadow
    /// (moisture depleted by a ridge crossed upwind), combined into one
    /// deterministic single-pass estimate (`SimWorld::compute_rainfall`).
    /// Read nowhere yet — feeding it into biome classification (replacing
    /// `light` as the aridity proxy) is an explicit future follow-up, not
    /// this task's scope.
    pub rainfall: f32,
    /// Task 127: this cell's single steepest-descent Moore neighbour — the
    /// cell index `flow_accumulation` drains into, computed by
    /// `SimWorld::compute_hydrology`. `None` for a sink (every `Sea` cell,
    /// and any non-`Sea` cell no Moore neighbour scores strictly lower than
    /// under the deterministic total order `compute_hydrology` uses — see
    /// its own doc comment for why that can never cycle).
    pub flow_direction: Option<usize>,
    /// Task 127: accumulated flow reaching this cell — starts at its own
    /// `rainfall` and gains every upstream cell's accumulation along the
    /// `flow_direction` tree. Monotonically non-decreasing downstream.
    pub flow_accumulation: f32,
    /// Task 127: whether `flow_accumulation` clears this world's adaptive
    /// river threshold (`HydrologyConfig::river_top_fraction`). Read
    /// nowhere yet — rendering rivers and feeding them into biome
    /// classification are explicit future follow-ups, not this task's
    /// scope (see its own Non-Goals).
    pub is_river: bool,
    /// Task 131: `[0, 1]` climate-grounded wetness estimate, refining task
    /// 125's `slope`/`water_distance` drainage proxy with a real formula
    /// (spec §9.4): `rainfall` retained against `slope` runoff, a proximity
    /// bonus toward rivers (`is_river`, task 127) and toward
    /// depression-derived Lake footprints (task 129's `lake_depressions`,
    /// known *before* `Biome::Lake` is actually painted — see
    /// `SimWorld::compute_soil_moisture`'s doc comment for why that's safe
    /// to use here unlike `water_distance`), minus evaporation
    /// (`temperature`) and drainage (`slope` again, a separate term from
    /// retention — see the formula in `compute_soil_moisture`). Computed
    /// once at generation time, read by `swamp_score` in place of the old
    /// `slope`/`water_distance` pair, which this field now subsumes.
    pub soil_moisture: f32,
}

/// The simulated world: a dense grid, the species registry, and the seeded
/// RNG that drives every random choice in the simulation (GDD §5.7).
///
/// The grid is a plain `Vec<Cell>`, not ECS entities (TECH_DESIGN.md §3.1):
/// Bevy entities exist only for rendering (task 006).
#[derive(Resource)]
pub struct SimWorld {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub species: Vec<Species>,
    pub tick: u64,
    /// The rare, narrative time unit (task 135): advances once every
    /// `TimeConfig::seasons_per_era` seasons. Gates the world's era budget
    /// (`WorldParams::era_budget`) and the per-species curated notebook
    /// entries (`EraCompleted`) — not decision-cadence mechanics, which key
    /// off `season` instead.
    pub era: u32,
    /// The player's unit of decision (task 135): advances every
    /// `TimeConfig::season_pulses` ticks, refills `ActionBudget`, and gates
    /// reproduction eligibility (`Organism::born_season`) and the onboarding
    /// grace period — everything that used to key off `era` before the
    /// season/era split.
    pub season: u32,
    pub seed: u64,
    /// The world's active tag subset (GDD §5.5): `active_tags[slot.0 as usize]`
    /// is the `TagId` a given `TagSlot` refers to in this world. Drawn
    /// procedurally from the global pool at construction (task 038,
    /// `select_active_tags`) — not necessarily a contiguous range of
    /// `TagId`s, unlike Phase 1's fixed `TagId(0..active_tags_early)`.
    pub active_tags: Vec<TagId>,
    /// The world's secret matrix (GDD §5.5), generated once at construction.
    pub matrix: TagMatrix,
    /// This world's terrain-conditional tags (task 096): only conditional
    /// `TagId`s that actually landed in `active_tags` get an entry — a
    /// conditional `TagId` not drawn into this world's active subset simply
    /// doesn't exist here this run, no gating needed. Tiny (~1-2 entries by
    /// `TagConfig::conditional_tag_count`), so a linear scan
    /// (`conditional_gate`) is used instead of a `HashMap`.
    pub conditional_tags: Vec<ConditionalTag>,
    /// Cell indices of this world's heat sources (task 085), placed once at
    /// generation time. `reinject_environment_sources` reads this every
    /// tick to pull those cells' temperature back toward
    /// `EnvironmentConfig::source_temperature`, countering
    /// `diffuse_environment`'s erosion — the same reason `objectives.rs`'s
    /// `SurviveIn` reads the stable `Cell::biome` classification (task 113)
    /// rather than the diffusing scalar it seeded.
    pub heat_sources: Vec<usize>,
    /// Cell indices of the Swamp cells `classify_biomes`'s toxicity-
    /// imposition pass (task 125 §12.4) marked toxic, set once at
    /// generation time (task 122). Same role as `heat_sources`:
    /// `reinject_environment_sources` reads this every tick to pull those
    /// cells' `toxicity` back toward `EnvironmentConfig::
    /// swamp_toxicity_value`, countering `diffuse_environment`'s erosion —
    /// without this, a chemolithotroph (or a `SurviveIn` objective) left
    /// alone for a few hundred ticks would find the "toxic" ground it was
    /// counting on had quietly faded toward ambient.
    pub toxic_swamp_cells: Vec<usize>,
    /// Grid (Chebyshev) distance from each cell to its nearest `Sea` cell
    /// (task 086 playtest follow-up), computed once at generation time via
    /// multi-source BFS and reused both to bake the initial coastal-cooling
    /// falloff in `apply_environment_sources` and to weight
    /// `reinject_environment_sources`' ongoing pull toward
    /// `SourceConfig::sea_coolant_value` — a single source of truth for "how
    /// close is this cell to the sea" instead of two independent
    /// computations drifting apart.
    sea_distance: Vec<f32>,
    /// Whether any organism has ever occupied a cell in this world (task
    /// 050): set by `sim::step` the first time its population scan finds
    /// one. Worlds start with nothing placed (the player seeds them via
    /// `Seed`, GDD §6) — `objectives::is_total_extinction` reads this so a
    /// world that hasn't been seeded yet doesn't fail on its very first
    /// evaluated tick.
    pub ever_populated: bool,
    /// `SpeciesId`s of wild, pre-existing populations placed directly onto
    /// the grid at world generation (task 098) — a narrow, documented
    /// exception to task 050's "nothing auto-placed" rule. Tracked
    /// separately from `Species` itself (rather than a bool field on it) to
    /// avoid touching every existing `Species { .. }` literal in the
    /// codebase; tiny (`WorldgenConfig::wild_species_count` entries), so a
    /// `Vec` scan (`is_wild`) is fine, no `HashMap` needed.
    pub wild_species: Vec<SpeciesId>,
    /// `SpeciesId`s granted tolerance to `Sea` cells by a speciation event
    /// (task 107, "toxicity"-dominant stimulus edit) — a small side list,
    /// same shape as `wild_species` above and for the same reason (avoid
    /// touching every `Species { .. }` literal in the codebase for a flag
    /// only a handful of species will ever carry). **Naming note**: the
    /// source doc calls this edit "toxic-zone placement tolerance," but
    /// `is_placeable`/`is_placeable_kind` has never actually gated on
    /// `Cell::toxicity` — only on `TerrainKind::Sea` and mountain peaks —
    /// and the GDD (§5.2, v0.5 changelog) is explicit that `toxicity` is a
    /// "declared-but-currently-inert scalar," not read anywhere in the
    /// tick loop. There is no toxicity hazard to grant tolerance *from*
    /// yet. This bypasses the one placement gate that actually exists and
    /// is thematically closest (an environmental-hazard exemption), the
    /// same kind of doc/code grounding correction task 105 made explicit
    /// rather than silently building on a false premise. Revisit the name
    /// and behavior once toxicity gets real mechanical teeth (task 108 or
    /// later). Peaks are never bypassed — that's a structural
    /// impassability, not a hazard-tolerance question.
    pub sea_tolerant_species: Vec<SpeciesId>,
    /// Per-`SpeciesId` terrain-occupancy history (task 099), indexed by
    /// `SpeciesId.0`, grown lazily as `species` grows — see
    /// `TerrainOccupancy`'s own doc comment for why this is shaped
    /// generically rather than reveal-specific.
    pub terrain_occupancy: Vec<TerrainOccupancy>,
    /// Per-`SpeciesId` accumulated selection pressure (task 106), indexed by
    /// `SpeciesId.0`, grown lazily as `species` grows — see
    /// `SelectionPressure`'s own doc comment for the lineage-scoping
    /// rationale.
    pub selection_pressure: Vec<SelectionPressure>,
    /// Per-`SpeciesId` origin era (task 103) — `species_origin_era[id.0]` is
    /// the `era` this world was on when that species was first pushed onto
    /// `species` (i.e. when it entered the registry: world generation for
    /// the starting roster, or the moment of a `Splice`). Distinct from
    /// `species_seeded_era` below: a species can sit in the registry for
    /// several eras before the player ever places one — this field alone
    /// is *not* "when it was seeded," a bug task 103's original catalog
    /// row conflated (fixed as a follow-up, see `species_seeded_era`).
    /// Can't be reconstructed after the fact, so every `species.push` call
    /// site goes through `push_species` instead, which records both in the
    /// same step.
    pub species_origin_era: Vec<u32>,
    /// Per-`SpeciesId` "era this species was first actually placed on the
    /// grid" (task 103 follow-up), indexed by `SpeciesId.0`, grown lazily
    /// and set by `sim::step`'s per-tick population scan the first tick a
    /// species' population goes from `0` to `> 0` — unlike
    /// `species_origin_era` (registry-creation time), this tracks real
    /// placement, so a species seeded mid-run shows that era, not `0`.
    /// `None` for a species still sitting in the available roster with
    /// nothing placed yet. Once set, never cleared — the seeded era stays
    /// a true historical fact even after the species later goes fully
    /// extinct.
    pub species_seeded_era: Vec<Option<u32>>,
    /// Whether `sim::speciate` has successfully created a descendant
    /// species in this world (task 109) — set once, never cleared for the
    /// rest of this world's life, mirroring `ever_populated`'s shape.
    /// Backs `Objective::Speciation`, the long-term objective's default
    /// content: a per-world flag, not a run-level one, since the objective
    /// itself resets fresh with every new world like the short-term tier.
    pub has_speciated: bool,
    rng: StdRng,
    /// Write-side double buffer for the tick (TECH_DESIGN.md §6). `pub(crate)`
    /// so `sim::step` can read/write it directly without a cell-by-cell API.
    pub(crate) scratch: Vec<Cell>,
    /// Per-cell adjacency-evidence onset tracking (task 136b). See
    /// `AdjacencyExposure`'s own doc comment. Sized once, like `cells` —
    /// the grid never resizes, so this needs no lazy-grow logic.
    pub adjacency_exposure: Vec<AdjacencyExposure>,
}

impl SimWorld {
    /// Builds the first world of a run (`world_index = 0`) — a convenience
    /// so call sites that don't yet think in terms of run progress (most
    /// tests, the `r`-key reseed, `spawn_world`) don't have to. Behaves
    /// exactly like `new_for_world(seed, 0, config)`, which per task 037's
    /// `world_params(0, ..)` guarantee reproduces Phase 0-2's "early"
    /// generation parameters exactly.
    pub fn new(seed: u64, config: &SimConfig) -> Self {
        Self::new_for_world(seed, 0, config)
    }

    /// Builds one world of a run at `world_index` (GDD §9): the active tag
    /// subset, matrix, and environment all scale with the difficulty curve
    /// (`worldgen::world_params`, task 037) instead of always using Phase
    /// 1's fixed "early" values.
    pub fn new_for_world(seed: u64, world_index: u32, config: &SimConfig) -> Self {
        let width = config.grid.width as usize;
        let height = config.grid.height as usize;
        let cells = vec![Cell::default(); width * height];
        let mut rng = StdRng::seed_from_u64(seed);
        let params = world_params(world_index, config);
        let active_tags = select_active_tags(&params, &config.tags, &mut rng);
        let matrix = generate_matrix(
            active_tags.len(),
            params.matrix_density,
            &config.tags,
            &mut rng,
        );
        let conditional_tags = roll_conditional_tags(&active_tags, &config.tags, &mut rng);
        let mut world = Self {
            width,
            height,
            scratch: cells.clone(),
            adjacency_exposure: vec![AdjacencyExposure::default(); cells.len()],
            cells,
            species: Vec::new(),
            tick: 0,
            era: 0,
            season: 0,
            seed,
            active_tags,
            matrix,
            conditional_tags,
            heat_sources: Vec::new(),
            toxic_swamp_cells: Vec::new(),
            sea_distance: Vec::new(),
            ever_populated: false,
            wild_species: Vec::new(),
            sea_tolerant_species: Vec::new(),
            terrain_occupancy: Vec::new(),
            selection_pressure: Vec::new(),
            species_origin_era: Vec::new(),
            species_seeded_era: Vec::new(),
            has_speciated: false,
            rng,
        };
        world.generate_terrain(config);
        world.compute_slope(config);
        world.apply_environment_sources(config, &params);
        world.compute_rainfall(config, &params);
        // Task 129: `fill_depressions` computed once, shared by
        // `record_significant_depressions` (which `place_feature_biomes`
        // needs Lake candidates from) and `compute_hydrology` (flow
        // routing) — moved ahead of `classify_biomes`/`place_feature_biomes`
        // so depression-derived Lake footprints exist before the feature
        // pass needs them. Neither `fill_depressions` nor
        // `compute_hydrology` ever depended on biome data, so this
        // reordering doesn't disturb anything downstream.
        let filled = world.fill_depressions();
        let lake_depressions = world.record_significant_depressions(&filled, config);
        world.compute_hydrology(config, &filled);
        // Task 131: needs `rainfall`/`is_river`/`lake_depressions`, all
        // already available here, and must itself finish before
        // `classify_biomes` reads `Cell.soil_moisture`.
        world.compute_soil_moisture(config, &lake_depressions);
        world.classify_biomes(config, &params);
        world.place_feature_biomes(config, &lake_depressions);
        world.compute_water_distance();
        world
    }

    /// Generates this world's terrain (task 066): a per-cell classification
    /// (`TerrainKind` + a sparse `is_peak` flag) into organic sea/plain/
    /// hill/mountain regions, from a summed-plane-wave elevation field —
    /// the same dependency-free noise technique as the decorative
    /// background layer (`render.rs`'s `background_field`), except this
    /// field is real simulation data, so it lives here and never touches
    /// `self.rng` (which drives tag/species/reproduction draws): it derives
    /// its own stream via `TERRAIN_SEED_OFFSET`, so adding this generation
    /// step doesn't shift any of those existing draws for a given seed.
    ///
    /// Bounded-resamples the *whole* elevation field (not per-cell — a
    /// per-cell resample would destroy the organic shape) up to
    /// `TerrainConfig::max_generation_attempts` times if the placeable
    /// fraction falls short of `TerrainConfig::min_placeable_fraction`,
    /// keeping the best draw seen if none clears the floor — same
    /// defensive-generation spirit as tasks 047/048 (an unplayable world
    /// must be caught here, not left as a rare-seed bug). Must run before
    /// `place_feature_biomes`, which needs real terrain to search against.
    fn generate_terrain(&mut self, config: &SimConfig) {
        let terrain_cfg = &config.terrain;
        let mut rng = StdRng::seed_from_u64(self.seed ^ TERRAIN_SEED_OFFSET);
        let cell_count = self.width * self.height;

        // (kinds, peaks, elevations, placeable fraction) — the best draw
        // seen so far, kept in case no attempt clears `min_placeable_fraction`.
        type TerrainDraw = (Vec<TerrainKind>, Vec<bool>, Vec<f32>, f32);
        let mut best: Option<TerrainDraw> = None;
        for _ in 0..terrain_cfg.max_generation_attempts.max(1) {
            let continent_waves = terrain_waves(
                &mut rng,
                terrain_cfg.continent_wave_count as usize,
                terrain_cfg.continent_freq_min,
                terrain_cfg.continent_freq_max,
            );
            let island_waves = terrain_waves(
                &mut rng,
                terrain_cfg.island_wave_count as usize,
                terrain_cfg.island_freq_min,
                terrain_cfg.island_freq_max,
            );
            let mut elevations: Vec<f32> = (0..self.height)
                .flat_map(|y| (0..self.width).map(move |x| (x, y)))
                .map(|(x, y)| {
                    let nx = x as f32 / (self.width - 1).max(1) as f32;
                    let ny = y as f32 / (self.height - 1).max(1) as f32;
                    terrain_elevation(
                        &continent_waves,
                        &island_waves,
                        terrain_cfg.island_blend_weight,
                        nx,
                        ny,
                    )
                })
                .collect();
            normalize_elevations(&mut elevations);
            let kinds: Vec<TerrainKind> = elevations
                .iter()
                .map(|&e| classify_elevation(e, terrain_cfg))
                .collect();

            let mut peaks = vec![false; cell_count];
            for y in 0..self.height {
                for x in 0..self.width {
                    let idx = self.index(x, y);
                    if kinds[idx] != TerrainKind::Mountain
                        || elevations[idx] < terrain_cfg.peak_elevation_threshold
                    {
                        continue;
                    }
                    peaks[idx] = self
                        .moore_neighbours(x, y)
                        .all(|n| elevations[n] <= elevations[idx]);
                }
            }

            let placeable = kinds
                .iter()
                .zip(&peaks)
                .filter(|&(&kind, &peak)| is_placeable_kind(kind, peak))
                .count();
            let fraction = placeable as f32 / cell_count as f32;

            if fraction >= terrain_cfg.min_placeable_fraction {
                self.write_terrain(&kinds, &peaks, &elevations);
                return;
            }
            if best
                .as_ref()
                .is_none_or(|&(_, _, _, best_fraction)| fraction > best_fraction)
            {
                best = Some((kinds, peaks, elevations, fraction));
            }
        }
        let (kinds, peaks, elevations, _) = best.expect("the loop above runs at least once");
        self.write_terrain(&kinds, &peaks, &elevations);
    }

    fn write_terrain(&mut self, kinds: &[TerrainKind], peaks: &[bool], elevations: &[f32]) {
        for (idx, cell) in self.cells.iter_mut().enumerate() {
            cell.terrain = kinds[idx];
            cell.is_peak = peaks[idx];
            cell.elevation = elevations[idx];
        }
    }

    /// Task 128 (`redesign/procedural_biome_generation_spec_v2.md` §11.1):
    /// a coarse `BiomeConfig::macro_region_count`-point Voronoi partition
    /// of the grid, on its own dedicated RNG stream
    /// (`MACRO_REGION_SEED_OFFSET`, never `self.rng`/`BIOME_SEED_OFFSET`).
    /// Each region's dominant biome is decided once from its aggregate
    /// climate — mean `temperature`/`light`/`soil_moisture` (task 131:
    /// subsumes the original `slope`/`water_distance` pair, matching
    /// `swamp_score`'s own per-cell switch) over only its
    /// `TerrainKind::Plain` cells, since those are the only candidates the
    /// per-cell `Plain` branch ever picks among — using the same
    /// `*_score` functions the per-cell pass uses, with `noise = 0.0`
    /// (patch-noise is per-cell texture, meaningless as a region average).
    /// A region with no `Plain` cells at all (e.g. entirely `Sea`/
    /// `Mountain`) defaults to `Biome::Plain`; no cell ever looks it up,
    /// since a cell only queries its *own* region and a `Plain`-kind cell
    /// can't belong to an all-non-Plain region under a Voronoi partition
    /// only if literally zero `Plain` cells fell in it — a real but
    /// harmless edge case, not one worth a panic over.
    ///
    /// Returns `(region_id_per_cell, dominant_biome_per_region)` — kept
    /// local to the `classify_biomes` call that consumes it, not stored on
    /// `SimWorld`, since nothing else needs it once the per-cell bias pass
    /// is done.
    fn compute_macro_regions(&self, config: &SimConfig) -> (Vec<usize>, Vec<Biome>) {
        let biome_cfg = &config.biome;
        let region_count = (biome_cfg.macro_region_count as usize).max(1);
        let mut rng = StdRng::seed_from_u64(self.seed ^ MACRO_REGION_SEED_OFFSET);
        let seed_points: Vec<(f32, f32)> = (0..region_count)
            .map(|_| (rng.random_range(0.0..1.0), rng.random_range(0.0..1.0)))
            .collect();

        let mut region_id = vec![0usize; self.cells.len()];
        // (sum_temperature, sum_light, sum_soil_moisture, count).
        let mut aggregate = vec![(0.0f32, 0.0f32, 0.0f32, 0u32); region_count];
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let nx = x as f32 / (self.width - 1).max(1) as f32;
                let ny = y as f32 / (self.height - 1).max(1) as f32;
                let nearest = seed_points
                    .iter()
                    .enumerate()
                    .map(|(i, &(sx, sy))| (i, (nx - sx).powi(2) + (ny - sy).powi(2)))
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(i, _)| i)
                    .expect("region_count is at least 1, so seed_points is non-empty");
                region_id[idx] = nearest;

                let cell = self.cells[idx];
                if cell.terrain == TerrainKind::Plain {
                    let entry = &mut aggregate[nearest];
                    entry.0 += cell.temperature;
                    entry.1 += cell.light;
                    entry.2 += cell.soil_moisture;
                    entry.3 += 1;
                }
            }
        }

        let dominant_biome: Vec<Biome> = aggregate
            .into_iter()
            .map(|(sum_temperature, sum_light, sum_soil_moisture, count)| {
                if count == 0 {
                    return Biome::Plain;
                }
                let n = count as f32;
                let (mean_temperature, mean_light, mean_soil_moisture) =
                    (sum_temperature / n, sum_light / n, sum_soil_moisture / n);
                let scores = [
                    (
                        Biome::Swamp,
                        swamp_score(mean_soil_moisture, biome_cfg, 0.0),
                    ),
                    (
                        Biome::Desert,
                        desert_score(mean_temperature, mean_light, biome_cfg),
                    ),
                    (Biome::Tundra, tundra_score(mean_temperature, biome_cfg)),
                    (
                        Biome::Forest,
                        forest_score(mean_temperature, mean_light, biome_cfg, 0.0),
                    ),
                    (Biome::Plain, biome_cfg.plain_baseline_score),
                ];
                argmax_biome(&scores)
            })
            .collect();

        (region_id, dominant_biome)
    }

    /// Classifies every cell's `Biome` (task 110,
    /// `redesign/abiogenesis-biomes.md`) — the two-stage architecture the
    /// design doc calls for: Stage A (`TerrainKind` + `is_peak` + the raw
    /// `elevation` this task adds) gives the base landform, Stage B refines
    /// it with the ambient scalars and geomorphology that
    /// `apply_environment_sources`/`generate_terrain` have already written.
    /// Runs before `place_feature_biomes`, which
    /// overrides whatever this produces for its own footprints (task 111) —
    /// so this method must never assume it's writing the *final* biome for
    /// every cell.
    ///
    /// Stage B picks each `TerrainKind::Plain` cell's biome by continuous
    /// score, not a priority chain of hard comparisons (task 125,
    /// `redesign/procedural_biome_generation_spec_v2.md` §1.5): every
    /// candidate (Swamp, Desert, Tundra, Forest, Plain) gets a `[0, 1]`
    /// fitness from smooth curves (`smoothstep`/`smooth_band`, the same
    /// idiom `sim.rs`'s `env_fit` Gaussian serves for organism fitness),
    /// and the arg-max wins — ties break by this fixed priority order:
    /// Swamp, Desert, Tundra, Forest, Plain. **Honest caveat, found
    /// 2026-08-19**: unlike a Gaussian, `smoothstep`/`smooth_band`
    /// *plateau* at exactly `1.0` past their transition (a cell far past
    /// `tundra_temperature_max - width` scores `1.0`, not a fading value),
    /// so exact ties are not the "vanishingly rare" edge case a purely
    /// continuous score would produce — a cold, flat, water-adjacent cell
    /// routinely saturates both Tundra's and Swamp's score to `1.0`
    /// simultaneously, and the fixed order (Swamp first) decides every
    /// such cell the same way regardless of *how* cold or *how* well-drained
    /// it is. This still fixed the original bug (no more hard
    /// discontinuity *within* a single candidate's own threshold), but the
    /// priority order still does real work at multi-candidate saturation
    /// overlaps — a known, documented limitation, not silently pretended
    /// away. A future refinement could replace the plateaus with a slowly
    /// decaying tail so ties stay genuinely rare. Forest/Palude's
    /// patch-noise masks
    /// (`forest_waves`/`swamp_waves`) are a small additive term on the
    /// score now, not the primary gate — same dependent-noise-sum
    /// technique as `terrain_elevation`, on their own derived RNG stream
    /// (`BIOME_SEED_OFFSET`, never `self.rng` or `generate_terrain`'s local
    /// stream — sharing a stream with a bounded-resample loop would make
    /// the draw depend on how many resample attempts that loop happened to
    /// take).
    ///
    /// Palude's drainage inputs (task 124's `slope`/`water_distance`,
    /// resolved as task 132 2026-08-19): `Cell.slope` is read directly —
    /// `compute_slope` now runs right after `generate_terrain`, before this
    /// method, since slope is a pure function of `elevation` with no
    /// further dependency. `water_distance` is still computed **locally**
    /// here via `sea_distance_field` (Sea-only), not read from the
    /// persisted `Cell.water_distance`: that field's BFS needs `Biome::Lake`
    /// cells, which `place_feature_biomes` only creates *after* this method
    /// runs (a genuine pipeline ordering constraint, not an oversight — see
    /// `compute_water_distance`'s doc comment). One consequence: Swamp's
    /// water-proximity score only "sees" the sea coastline here, not a
    /// newly-placed inland lake — acceptable, since the sea coastline is
    /// the dominant "near water" signal for a whole world and lakes are a
    /// small, rare feature.
    ///
    /// `params` (task 133) supplies `swamp_toxicity_min`'s per-world scaled
    /// value for the toxicity-imposition pass at the end of this method —
    /// everything else here still reads from `config` alone.
    ///
    /// Task 128: before the per-cell pass, `compute_macro_regions` derives
    /// a coarse Voronoi partition with one dominant biome per region and
    /// applies it as a **multiplicative** bias on each `Plain`-kind cell's
    /// own score for its region's dominant biome — deliberately
    /// multiplicative, not additive, because of the plateau caveat two
    /// paragraphs up: an additive bias does nothing to two scores already
    /// saturated at `1.0`, exactly the terrain where incoherent noise is
    /// worst, but a multiplicative one still separates them. The bias is a
    /// nudge, not an override: a cell whose own unboosted score for a
    /// *different* biome is high enough still wins (spec §11.4's "Swamp
    /// nei bacini" inside a Forest-dominant region).
    fn classify_biomes(&mut self, config: &SimConfig, params: &WorldParams) {
        let biome_cfg = &config.biome;
        let mut rng = StdRng::seed_from_u64(self.seed ^ BIOME_SEED_OFFSET);
        let forest_waves = terrain_waves(
            &mut rng,
            biome_cfg.patch_wave_count as usize,
            biome_cfg.patch_freq_min,
            biome_cfg.patch_freq_max,
        );
        let swamp_waves = terrain_waves(
            &mut rng,
            biome_cfg.patch_wave_count as usize,
            biome_cfg.patch_freq_min,
            biome_cfg.patch_freq_max,
        );
        let toxic_patch_waves = terrain_waves(
            &mut rng,
            biome_cfg.patch_wave_count as usize,
            biome_cfg.patch_freq_min,
            biome_cfg.patch_freq_max,
        );

        let (region_id, region_dominant_biome) = self.compute_macro_regions(config);

        let mut biomes = vec![Biome::default(); self.cells.len()];
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let cell = self.cells[idx];
                let nx = x as f32 / (self.width - 1).max(1) as f32;
                let ny = y as f32 / (self.height - 1).max(1) as f32;
                biomes[idx] = match cell.terrain {
                    TerrainKind::Sea => {
                        if cell.elevation < biome_cfg.deep_water_elevation_max {
                            Biome::DeepWater
                        } else {
                            Biome::ShallowWater
                        }
                    }
                    TerrainKind::Mountain => {
                        if cell.is_peak {
                            Biome::Peak
                        } else {
                            // Task 130 (spec §12.5): sub-bands a non-Peak
                            // Mountain cell into Ghiacciaio/Prateria
                            // alpina/Roccia nuda/reused-Foresta via the
                            // same score-based arg-max idiom task 125
                            // introduced for Plain. "Alta quota" is already
                            // guaranteed by `TerrainKind::Mountain` itself,
                            // so temperature (Glacier/AlpineMeadow) and
                            // slope+light (BareRock) are what actually
                            // differentiate the candidates. Forest is
                            // deliberately reused rather than a new
                            // MountainForest variant (task 130's own
                            // acceptance criterion): its score is already
                            // temperature/light based, with no dependency
                            // on `TerrainKind`, so a Mountain cell that
                            // clears Foresta's climate band reads as
                            // Foresta the same way a Plain one would.
                            let scores = [
                                (Biome::Mountain, biome_cfg.mountain_baseline_score),
                                (
                                    Biome::BareRock,
                                    bare_rock_score(cell.light, cell.slope, biome_cfg),
                                ),
                                (Biome::Glacier, glacier_score(cell.temperature, biome_cfg)),
                                (
                                    Biome::AlpineMeadow,
                                    alpine_meadow_score(cell.temperature, biome_cfg),
                                ),
                                (
                                    Biome::Forest,
                                    forest_score(
                                        cell.temperature,
                                        cell.light,
                                        biome_cfg,
                                        wave_band_sum(&forest_waves, nx, ny),
                                    ),
                                ),
                            ];
                            argmax_biome(&scores)
                        }
                    }
                    TerrainKind::Hill => {
                        // Task 125 §12.5, made score-based in task 130:
                        // steep + low light reads as BareRock more readily
                        // than shallow + low light — a slope-raised
                        // effective light ceiling, now a smooth arg-max
                        // (`bare_rock_score`) instead of a hard cutoff, and
                        // shared with `TerrainKind::Mountain`'s own BareRock
                        // candidate above rather than reimplemented here.
                        let scores = [
                            (Biome::Hill, biome_cfg.hill_baseline_score),
                            (
                                Biome::BareRock,
                                bare_rock_score(cell.light, cell.slope, biome_cfg),
                            ),
                        ];
                        argmax_biome(&scores)
                    }
                    TerrainKind::Plain => {
                        let scores = [
                            (
                                Biome::Swamp,
                                swamp_score(
                                    cell.soil_moisture,
                                    biome_cfg,
                                    wave_band_sum(&swamp_waves, nx, ny),
                                ),
                            ),
                            (
                                Biome::Desert,
                                desert_score(cell.temperature, cell.light, biome_cfg),
                            ),
                            (Biome::Tundra, tundra_score(cell.temperature, biome_cfg)),
                            (
                                Biome::Forest,
                                forest_score(
                                    cell.temperature,
                                    cell.light,
                                    biome_cfg,
                                    wave_band_sum(&forest_waves, nx, ny),
                                ),
                            ),
                            (Biome::Plain, biome_cfg.plain_baseline_score),
                        ];
                        let region_dominant = region_dominant_biome[region_id[idx]];
                        let biased = |candidate: (Biome, f32)| -> f32 {
                            if candidate.0 == region_dominant {
                                candidate.1 * (1.0 + biome_cfg.macro_region_bias_weight)
                            } else {
                                candidate.1
                            }
                        };
                        let mut best = scores[0];
                        let mut best_biased = biased(scores[0]);
                        for &candidate in &scores[1..] {
                            let candidate_biased = biased(candidate);
                            if candidate_biased > best_biased {
                                best = candidate;
                                best_biased = candidate_biased;
                            }
                        }
                        best.0
                    }
                };
            }
        }
        for (idx, cell) in self.cells.iter_mut().enumerate() {
            cell.biome = biomes[idx];
        }

        // Task 125 §12.4: toxicity is a modifier on a sub-region of the
        // Swamp cells just classified, not part of what makes a cell
        // Swamp in the first place — `swamp_toxicity_min` is repurposed
        // from "toxicity level a cell must already have" to "patch-noise
        // threshold selecting which fraction of Swamp reads as toxic."
        // This is now the *only* generation-time source of nonzero
        // `Cell::toxicity` (task 113 removed the old standalone
        // `place_toxic_zone` rectangle): `objectives.rs`'s `SurviveIn`
        // reads `Cell::biome == Swamp` directly, not this scalar. Task 133:
        // reads `params.swamp_toxicity_min` (the per-world difficulty-curve
        // scaled value), not `biome_cfg.swamp_toxicity_min` directly, so
        // the toxic fraction of Swamp grows across the run the way the old
        // `toxic_zone` rectangle's size used to (GDD §9's "larger toxic
        // zones"). Task 122: also records which cells got marked, in
        // `toxic_swamp_cells`, so `reinject_environment_sources` has a
        // stable list to counter `diffuse_environment`'s erosion against —
        // `toxicity > 0.0` alone can't be that list, since diffusion
        // spreads it to neighbouring cells over time and reinjecting
        // against *that* would expand the toxic footprint every tick
        // instead of holding it steady. Cleared first since this method
        // can run more than once per world (tests re-run it after
        // corrupting a field).
        self.toxic_swamp_cells.clear();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                if self.cells[idx].biome != Biome::Swamp {
                    continue;
                }
                let nx = x as f32 / (self.width - 1).max(1) as f32;
                let ny = y as f32 / (self.height - 1).max(1) as f32;
                if wave_band_sum(&toxic_patch_waves, nx, ny) > params.swamp_toxicity_min {
                    self.cells[idx].toxicity = config.environment.swamp_toxicity_value;
                    self.toxic_swamp_cells.push(idx);
                }
            }
        }
    }

    /// Places the four "feature" biomes task 111 covers
    /// (`redesign/abiogenesis-biomes.md`): Cratere profondo and Distesa di
    /// cristalli via bounded-retry rectangle search (same attempt-loop/
    /// keep-best-seen pattern as `generate_terrain`), Bocca vulcanica by
    /// hooking directly into the point heat sources
    /// `apply_environment_sources` already placed (no search needed
    /// there), and Lago from `lake_depressions` (task 129: terrain-grounded
    /// depressions `record_significant_depressions` already qualified),
    /// falling back to the same organic-mask search Crater/CrystalField
    /// use only if too few depressions qualified this world. Must run
    /// after `classify_biomes`: every cell touched here overrides whatever
    /// Stage A/B produced, not the other way round (the design doc's
    /// explicit "placed, not derived" distinction for these four biomes).
    ///
    /// `reserved` tracks every cell already claimed by an earlier feature
    /// in this same call so the searched/promoted footprints (and the vent
    /// footprint placed first) never overlap each other — each search
    /// requires zero overlap with `reserved` before accepting a candidate,
    /// falling back to the lowest-overlap candidate seen if none reaches
    /// zero within its attempt budget (vanishingly unlikely given these
    /// footprints are tiny relative to the grid); depression promotion
    /// uses the same zero-overlap rule, skipping (not partially promoting)
    /// any depression that overlaps an already-reserved cell.
    fn place_feature_biomes(&mut self, config: &SimConfig, lake_depressions: &[Vec<usize>]) {
        let biome_cfg = &config.biome;
        let cell_count = self.cells.len();
        let mut reserved = vec![false; cell_count];

        // Bocca vulcanica: a small radius around each heat source,
        // deliberately independent from `SourceConfig::heat_source_radius`
        // (that's the temperature falloff footprint, not a biome-sized
        // patch — task 111's own acceptance criteria calls out not
        // perturbing it). No *temperature* override:
        // `apply_environment_sources`/`reinject_environment_sources`
        // already keep these cells hot. `toxicity` *is* reset to ambient
        // here (task 113 fix, found by broadening
        // `cell_toxicity_matches_its_biome_across_seeds` past a single
        // seed): a cell `classify_biomes` scored as toxic Swamp can sit
        // within a heat source's radius and get overridden to
        // VolcanicVent here, and without this reset it would silently keep
        // its stale Swamp toxicity — a real cell whose biome says "not
        // toxic" but whose scalar disagrees.
        for &source_idx in &self.heat_sources.clone() {
            let (sx, sy) = (source_idx % self.width, source_idx / self.width);
            for y in 0..self.height {
                for x in 0..self.width {
                    let dx = x as f32 - sx as f32;
                    let dy = y as f32 - sy as f32;
                    if (dx * dx + dy * dy).sqrt() <= biome_cfg.volcanic_vent_radius {
                        let idx = self.index(x, y);
                        self.cells[idx].biome = Biome::VolcanicVent;
                        self.cells[idx].toxicity = 0.0;
                        reserved[idx] = true;
                    }
                }
            }
        }

        self.place_feature_organic(
            &mut reserved,
            &FeaturePlacement {
                seed_offset: CRATER_SEED_OFFSET,
                radius: biome_cfg.crater_radius,
                min_placeable_fraction: biome_cfg.crater_min_placeable_fraction,
                max_attempts: biome_cfg.crater_max_placement_attempts,
                biome: Biome::Crater,
                temperature: biome_cfg.crater_temperature,
                light: biome_cfg.crater_light,
                toxicity: biome_cfg.crater_toxicity,
            },
            biome_cfg,
        );
        self.place_feature_organic(
            &mut reserved,
            &FeaturePlacement {
                seed_offset: CRYSTAL_FIELD_SEED_OFFSET,
                radius: biome_cfg.crystal_field_radius,
                min_placeable_fraction: biome_cfg.crystal_field_min_placeable_fraction,
                max_attempts: biome_cfg.crystal_field_max_placement_attempts,
                biome: Biome::CrystalField,
                temperature: biome_cfg.crystal_field_temperature,
                light: biome_cfg.crystal_field_light,
                toxicity: biome_cfg.crystal_field_toxicity,
            },
            biome_cfg,
        );
        // Task 129: promote qualifying terrain depressions to Lago first —
        // real basins, not a synthetic organic-disk mask, so their
        // footprint is used exactly as `record_significant_depressions`
        // found it. A depression overlapping an already-reserved cell
        // (Bocca vulcanica/Crater/CrystalField) is skipped whole, never
        // partially promoted.
        let mut promoted = 0u32;
        for footprint in lake_depressions {
            if footprint.iter().any(|&idx| reserved[idx]) {
                continue;
            }
            for &idx in footprint {
                self.cells[idx].biome = Biome::Lake;
                self.cells[idx].temperature = biome_cfg.lake_temperature;
                self.cells[idx].light = biome_cfg.lake_light;
                self.cells[idx].toxicity = biome_cfg.lake_toxicity;
                reserved[idx] = true;
            }
            promoted += 1;
        }
        // Fallback: only if this world didn't produce enough real
        // depressions (e.g. low-relief terrain) does the old organic-mask
        // random search run at all — a world with enough depression-derived
        // lakes never touches `LAKE_SEED_OFFSET`.
        if promoted < biome_cfg.lake_min_depression_count {
            self.place_feature_organic(
                &mut reserved,
                &FeaturePlacement {
                    seed_offset: LAKE_SEED_OFFSET,
                    radius: biome_cfg.lake_radius,
                    min_placeable_fraction: biome_cfg.lake_min_placeable_fraction,
                    max_attempts: biome_cfg.lake_max_placement_attempts,
                    biome: Biome::Lake,
                    temperature: biome_cfg.lake_temperature,
                    light: biome_cfg.lake_light,
                    toxicity: biome_cfg.lake_toxicity,
                },
                biome_cfg,
            );
        }
    }

    /// Computes and stores `Cell.slope` (local elevation-gradient
    /// magnitude, normalized), a pure function of `elevation` alone
    /// (task 124). No new RNG stream is needed, and unlike
    /// `compute_water_distance` (see its own doc comment, task 132) this
    /// has no dependency on `Biome::Lake` existing yet, so it runs right
    /// after `generate_terrain`. Read by `classify_biomes` (task 125).
    fn compute_slope(&mut self, config: &SimConfig) {
        let terrain_cfg = &config.terrain;
        let elevations: Vec<f32> = self.cells.iter().map(|c| c.elevation).collect();
        let mut slope = vec![0.0f32; self.cells.len()];
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let raw_slope = self.elevation_slope(x, y, &elevations);
                slope[idx] =
                    (raw_slope / terrain_cfg.slope_normalization.max(f32::EPSILON)).clamp(0.0, 1.0);
            }
        }
        for (idx, cell) in self.cells.iter_mut().enumerate() {
            cell.slope = slope[idx];
        }
    }

    /// Task 132 (resolved 2026-08-19): `water_distance` stays a separate
    /// step from `compute_slope`, run after `place_feature_biomes` — its
    /// BFS needs `Biome::Lake` cells to exist as a source, and Lake is only
    /// placed there. `slope` has no such dependency (pure function of
    /// `elevation`, final right after `generate_terrain`), so splitting the
    /// two lets `classify_biomes` read the real, persisted `Cell.slope`
    /// instead of recomputing its own local copy (task 125's original
    /// workaround) — `water_distance` still can't be read that early, so
    /// Swamp's water-proximity term keeps its own local Sea-only proxy;
    /// see `classify_biomes`'s doc comment.
    fn compute_water_distance(&mut self) {
        let water_distance = self.water_distance_field();
        for (idx, cell) in self.cells.iter_mut().enumerate() {
            cell.water_distance = water_distance[idx];
        }
    }

    /// Priority-flood depression filling (task 127; Barnes et al. 2014,
    /// simplified — no separate flat-resolution pass, see
    /// `compute_hydrology`'s doc comment for how flats are still routed
    /// deterministically without one). Every `TerrainKind::Sea` cell starts
    /// already-resolved at its own elevation; expansion always resolves
    /// the lowest-filled unresolved cell next, raising it to at least its
    /// resolving neighbour's filled elevation. This guarantees every
    /// non-`Sea` cell ends up with at least one neighbour whose filled
    /// elevation is `<=` its own, so routing downhill from *filled*
    /// elevation can never trap in a closed local minimum the way routing
    /// from raw `elevation` could.
    ///
    /// Uses `elevation.to_bits()` as the priority queue's ordering key
    /// instead of a custom `Ord` wrapper for `f32` — sound because every
    /// `Cell.elevation` is normalized to `[0, 1]` (non-negative), and IEEE
    /// 754's bit-pattern order matches numeric order for non-negative
    /// floats. If a world somehow has no `Sea` cells at all (degenerate,
    /// blocked upstream by `TerrainConfig::min_placeable_fraction` in
    /// practice), filling is a no-op and raw `elevation` is returned
    /// unfilled — a graceful fallback, not a panic.
    fn fill_depressions(&self) -> Vec<f32> {
        let mut filled: Vec<f32> = self.cells.iter().map(|c| c.elevation).collect();
        let mut resolved = vec![false; self.cells.len()];
        let mut heap: std::collections::BinaryHeap<std::cmp::Reverse<(u32, usize)>> =
            std::collections::BinaryHeap::new();
        for (idx, cell) in self.cells.iter().enumerate() {
            if cell.terrain == TerrainKind::Sea {
                resolved[idx] = true;
                heap.push(std::cmp::Reverse((filled[idx].to_bits(), idx)));
            }
        }
        while let Some(std::cmp::Reverse((_, idx))) = heap.pop() {
            let (x, y) = (idx % self.width, idx / self.width);
            for n in self.moore_neighbours(x, y) {
                if resolved[n] {
                    continue;
                }
                resolved[n] = true;
                filled[n] = filled[n].max(filled[idx]);
                heap.push(std::cmp::Reverse((filled[n].to_bits(), n)));
            }
        }
        filled
    }

    /// Task 129 (spec §10.4): groups `fill_depressions`' output into
    /// connected components (Moore-adjacency flood fill) of cells that
    /// actually needed filling (`filled > elevation`, i.e. a real local
    /// basin, not already at its own resolved level) and returns the
    /// footprints of the ones that qualify as *Lago-sized* — big enough to
    /// read as a lake (`lake_depression_min_size`), not so big they're
    /// really a whole drainage basin (`lake_depression_max_size` — a
    /// 25-seed scratch measurement found raw components span from
    /// single-digit noise to 600+ cells), and deep enough
    /// (`lake_depression_min_depth`) to not just be float noise in the
    /// priority-flood fill. Deterministic and RNG-free: purely a function
    /// of already-seed-derived `elevation`/`filled`, so component
    /// discovery order (cell index ascending) doesn't affect the result,
    /// only which cell each component happens to be *found from* — the
    /// footprint itself is the same regardless.
    fn record_significant_depressions(
        &self,
        filled: &[f32],
        config: &SimConfig,
    ) -> Vec<Vec<usize>> {
        let biome_cfg = &config.biome;
        // `1e-4`, not `f32::EPSILON`: a much smaller threshold picks up
        // floating-point noise from the priority-flood fill's repeated
        // `max()` chaining as spurious single-cell "basins," inflating the
        // component count without representing a real dip in the terrain.
        const BASIN_EPSILON: f32 = 1e-4;
        let is_basin_cell = |idx: usize| filled[idx] - self.cells[idx].elevation > BASIN_EPSILON;

        let mut visited = vec![false; self.cells.len()];
        let mut qualifying = Vec::new();
        for start in 0..self.cells.len() {
            if visited[start] || !is_basin_cell(start) {
                visited[start] = true;
                continue;
            }
            let mut stack = vec![start];
            visited[start] = true;
            let mut footprint = Vec::new();
            let mut max_depth = 0.0f32;
            while let Some(idx) = stack.pop() {
                footprint.push(idx);
                max_depth = max_depth.max(filled[idx] - self.cells[idx].elevation);
                let (x, y) = (idx % self.width, idx / self.width);
                for n in self.moore_neighbours(x, y) {
                    if !visited[n] && is_basin_cell(n) {
                        visited[n] = true;
                        stack.push(n);
                    }
                }
            }
            let size = footprint.len() as u32;
            if (biome_cfg.lake_depression_min_size..=biome_cfg.lake_depression_max_size)
                .contains(&size)
                && max_depth >= biome_cfg.lake_depression_min_depth
            {
                qualifying.push(footprint);
            }
        }
        qualifying
    }

    /// Computes deterministic flow accumulation (task 127,
    /// `redesign/procedural_biome_generation_spec_v2.md` §10): `rainfall`
    /// (task 126) routed downhill along a single steepest-descent Moore
    /// neighbour per cell, then marks the top `river_top_fraction` of
    /// non-`Sea` cells by accumulation as `is_river`. Must run after
    /// `compute_rainfall`, the flow source — order relative to everything
    /// else is otherwise free (task 129: takes `filled` as a parameter,
    /// shared with `record_significant_depressions`, rather than
    /// recomputing `fill_depressions` a second time; the stale claim that
    /// this must also run after `compute_water_distance` is corrected
    /// below — it never actually depended on that).
    ///
    /// **Determinism is the primary risk in this task, not the hydrology
    /// model's fidelity** — every cell's downhill target is chosen by a
    /// fixed total order over `(filled_elevation, sea_distance, cell_index)`
    /// tuples (all three already-deterministic, seed-derived fields; no new
    /// RNG draw needed anywhere in this method). A cell only ever routes to
    /// a Moore neighbour whose tuple is *strictly* less than its own — so
    /// any chain of `flow_direction`s is strictly decreasing in a finite
    /// total order, which makes a routing cycle mathematically impossible
    /// by construction, regardless of how a tie is actually broken. The
    /// `sea_distance`/`cell_index` tail of the tuple only matters on a flat
    /// filled plateau (`fill_depressions` does no separate flat-resolution
    /// pass): preferring the neighbour closer to the sea nudges flow the
    /// geometrically sensible way on most flats, but isn't a rigorous
    /// guarantee — an occasional flat cell may end up routed to a
    /// less-than-ideal neighbour (or, in a truly pathological tie, to none,
    /// becoming a premature sink) without ever breaking determinism or
    /// correctness elsewhere. Accepted per this task's own scope note: "a
    /// simple priority-flood fill is sufficient, a full breaching algorithm
    /// is not required for a first version."
    fn compute_hydrology(&mut self, config: &SimConfig, filled: &[f32]) {
        let hydro_cfg = &config.hydrology;
        let sea_proximity = self.sea_distance_field();
        let key = |idx: usize| (filled[idx].to_bits(), sea_proximity[idx].to_bits(), idx);

        // Descending order by `key` (highest filled elevation first): every
        // upstream contribution to a cell is added to `accumulation` before
        // that cell drains further downstream, in a single pass.
        let mut order: Vec<usize> = (0..self.cells.len()).collect();
        order.sort_by_key(|&idx| std::cmp::Reverse(key(idx)));

        let mut flow_direction: Vec<Option<usize>> = vec![None; self.cells.len()];
        for &idx in &order {
            if self.cells[idx].terrain == TerrainKind::Sea {
                continue; // Sea is always a sink, never routes further.
            }
            let (x, y) = (idx % self.width, idx / self.width);
            let mut best: Option<usize> = None;
            let mut best_key = key(idx);
            for n in self.moore_neighbours(x, y) {
                let n_key = key(n);
                if n_key < best_key {
                    best_key = n_key;
                    best = Some(n);
                }
            }
            flow_direction[idx] = best;
        }

        let mut accumulation: Vec<f32> = self.cells.iter().map(|c| c.rainfall).collect();
        for &idx in &order {
            if let Some(target) = flow_direction[idx] {
                accumulation[target] += accumulation[idx];
            }
        }

        let threshold = {
            let mut non_sea: Vec<f32> = self
                .cells
                .iter()
                .zip(&accumulation)
                .filter(|(cell, _)| cell.terrain != TerrainKind::Sea)
                .map(|(_, &acc)| acc)
                .collect();
            non_sea.sort_by(f32::total_cmp);
            if non_sea.is_empty() {
                f32::INFINITY
            } else {
                let rank = ((non_sea.len() as f32) * (1.0 - hydro_cfg.river_top_fraction))
                    .clamp(0.0, (non_sea.len() - 1) as f32) as usize;
                non_sea[rank]
            }
        };

        for (idx, cell) in self.cells.iter_mut().enumerate() {
            cell.flow_direction = flow_direction[idx];
            cell.flow_accumulation = accumulation[idx];
            cell.is_river = cell.terrain != TerrainKind::Sea && accumulation[idx] >= threshold;
        }
    }

    /// Local elevation-gradient magnitude at `(x, y)` (task 124): a
    /// 4-neighbour central difference, cheaper than a full Moore gradient
    /// and accurate enough for a coarse per-cell terrain-roughness proxy.
    /// Edge cells fall back to a one-sided difference (no wraparound).
    fn elevation_slope(&self, x: usize, y: usize, elevations: &[f32]) -> f32 {
        let (dx, dy) = self.elevation_gradient(x, y, elevations);
        (dx * dx + dy * dy).sqrt()
    }

    /// Signed elevation gradient `(dx, dy)` at `(x, y)`: a 4-neighbour
    /// central difference (one-sided at grid edges, no wraparound) —
    /// `elevation_slope`'s magnitude-only reading factored out so task
    /// 126's rainfall step can also read the gradient's *direction*
    /// (projected onto wind, for orographic lift), not just how steep the
    /// terrain is.
    fn elevation_gradient(&self, x: usize, y: usize, elevations: &[f32]) -> (f32, f32) {
        let e = |xx: usize, yy: usize| elevations[self.index(xx, yy)];
        let dx = if x == 0 {
            e(x + 1, y) - e(x, y)
        } else if x == self.width - 1 {
            e(x, y) - e(x - 1, y)
        } else {
            (e(x + 1, y) - e(x - 1, y)) / 2.0
        };
        let dy = if y == 0 {
            e(x, y + 1) - e(x, y)
        } else if y == self.height - 1 {
            e(x, y) - e(x, y - 1)
        } else {
            (e(x, y + 1) - e(x, y - 1)) / 2.0
        };
        (dx, dy)
    }

    /// One bounded-retry deformed-disk placement for `place_feature_biomes`
    /// (task 111; organic footprint since task 123) — the same
    /// attempt-loop/keep-best-seen shape the old rectangle version used,
    /// extended with a hard zero-overlap requirement against `reserved`
    /// (other already-placed feature biomes). A candidate is accepted
    /// immediately once it clears both zero overlap and
    /// `spec.min_placeable_fraction`; otherwise the best candidate seen —
    /// ranked by lowest overlap first, then highest placeable fraction — is
    /// kept once the attempt budget runs out. The mask's angular-distortion
    /// waves are drawn once, up front, from the feature's own seed stream —
    /// same silhouette for every candidate center tried this call.
    fn place_feature_organic(
        &mut self,
        reserved: &mut [bool],
        spec: &FeaturePlacement,
        biome_cfg: &BiomeConfig,
    ) {
        if spec.radius <= 0.0 || self.width == 0 || self.height == 0 {
            return;
        }
        let mut rng = StdRng::seed_from_u64(self.seed ^ spec.seed_offset);
        let waves = angle_waves(&mut rng, biome_cfg.feature_mask_wave_count as usize);
        let mask = FeatureMask::new(spec.radius, biome_cfg.feature_mask_distortion, &waves);

        // (cx, cy, overlap with `reserved`, placeable fraction).
        let mut best: Option<(usize, usize, usize, f32)> = None;
        for _ in 0..spec.max_attempts.max(1) {
            let cx = rng.random_range(0..self.width);
            let cy = rng.random_range(0..self.height);
            let overlap = self.reserved_overlap_in_mask(cx, cy, &mask, reserved);
            let fraction = self.placeable_fraction_in_mask(cx, cy, &mask);
            if overlap == 0 && fraction >= spec.min_placeable_fraction {
                self.set_feature_biome_mask(reserved, cx, cy, &mask, spec);
                return;
            }
            let is_better = match best {
                None => true,
                Some((_, _, best_overlap, best_fraction)) => {
                    overlap < best_overlap || (overlap == best_overlap && fraction > best_fraction)
                }
            };
            if is_better {
                best = Some((cx, cy, overlap, fraction));
            }
        }
        let (cx, cy, _, _) = best.expect("the loop above runs at least once");
        self.set_feature_biome_mask(reserved, cx, cy, &mask, spec);
    }

    /// Bounding box (`[x0, x1) x [y0, y1)`) of `mask` centered at
    /// `(cx, cy)`, clamped to grid bounds.
    fn feature_mask_bounds(
        &self,
        cx: usize,
        cy: usize,
        mask: &FeatureMask,
    ) -> (usize, usize, usize, usize) {
        let x0 = cx.saturating_sub(mask.extent);
        let y0 = cy.saturating_sub(mask.extent);
        let x1 = (cx + mask.extent + 1).min(self.width);
        let y1 = (cy + mask.extent + 1).min(self.height);
        (x0, y0, x1, y1)
    }

    /// Count of `reserved` cells within `mask` centered at `(cx, cy)` — the
    /// overlap metric `place_feature_organic` searches against.
    fn reserved_overlap_in_mask(
        &self,
        cx: usize,
        cy: usize,
        mask: &FeatureMask,
        reserved: &[bool],
    ) -> usize {
        let (x0, y0, x1, y1) = self.feature_mask_bounds(cx, cy, mask);
        let mut count = 0;
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = (x as f32 - cx as f32, y as f32 - cy as f32);
                if mask.contains(dx, dy) && reserved[self.index(x, y)] {
                    count += 1;
                }
            }
        }
        count
    }

    /// Fraction of `mask`'s own cells (not its bounding box) that are
    /// placeable terrain — the metric `place_feature_organic` searches
    /// against. `0.0` for a mask with no cells at all (fully off-grid).
    fn placeable_fraction_in_mask(&self, cx: usize, cy: usize, mask: &FeatureMask) -> f32 {
        let (x0, y0, x1, y1) = self.feature_mask_bounds(cx, cy, mask);
        let mut total = 0usize;
        let mut placeable = 0usize;
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = (x as f32 - cx as f32, y as f32 - cy as f32);
                if mask.contains(dx, dy) {
                    total += 1;
                    let cell = self.get(x, y);
                    if is_placeable_kind(cell.terrain, cell.is_peak) {
                        placeable += 1;
                    }
                }
            }
        }
        if total == 0 {
            0.0
        } else {
            placeable as f32 / total as f32
        }
    }

    /// Writes `spec`'s biome and target scalars onto every cell inside
    /// `mask` centered at `(cx, cy)`, and marks them `reserved` so a later
    /// `place_feature_organic` call never overlaps this footprint.
    fn set_feature_biome_mask(
        &mut self,
        reserved: &mut [bool],
        cx: usize,
        cy: usize,
        mask: &FeatureMask,
        spec: &FeaturePlacement,
    ) {
        let (x0, y0, x1, y1) = self.feature_mask_bounds(cx, cy, mask);
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = (x as f32 - cx as f32, y as f32 - cy as f32);
                if mask.contains(dx, dy) {
                    let idx = self.index(x, y);
                    self.cells[idx].biome = spec.biome;
                    self.cells[idx].temperature = spec.temperature;
                    self.cells[idx].light = spec.light;
                    self.cells[idx].toxicity = spec.toxicity;
                    reserved[idx] = true;
                }
            }
        }
    }

    /// Replaces the old fixed left-right/top-bottom lerps (task 085,
    /// `redesign/abiogenesis-environment-sources.md`): temperature comes
    /// from distance to the nearest of `params.heat_source_count` wind-biased
    /// point sources, then blended toward `SourceConfig::sea_coolant_value`
    /// by proximity to `TerrainKind::Sea` (task 086 playtest follow-up —
    /// baking a real coastal falloff in here, not just a per-tick nudge on
    /// the coastline itself, is what makes the sea's cooling legible at
    /// world start rather than only after many ticks of diffusion). Light
    /// comes from a per-world sun-direction projection dimmed near
    /// `Mountain` peaks. Draws two independent RNG streams
    /// (`TEMPERATURE_SOURCE_SEED_OFFSET`, `SUN_DIRECTION_SEED_OFFSET`) —
    /// never `self.rng` — so this generation step doesn't shift any other
    /// draw for a given seed, same discipline as `generate_terrain`. Must
    /// run after `generate_terrain` (heat source placement, and the
    /// sea-distance field, both need real terrain to search against).
    /// Toxicity is not set here (task 113: `classify_biomes` owns it, as a
    /// post-classification modifier on Swamp cells).
    fn apply_environment_sources(&mut self, config: &SimConfig, params: &WorldParams) {
        let env = &config.environment;
        let source_cfg = &config.source;

        let mut temp_rng = StdRng::seed_from_u64(self.seed ^ TEMPERATURE_SOURCE_SEED_OFFSET);
        let wind_angle = temp_rng.random_range(0.0..TAU);
        let wind = Self::wind_from_angle(wind_angle, params.wind_strength);
        let heat_sources = self.place_heat_sources(&mut temp_rng, source_cfg, params);

        let mut sun_rng = StdRng::seed_from_u64(self.seed ^ SUN_DIRECTION_SEED_OFFSET);
        let sun_angle = sun_rng.random_range(0.0..TAU);
        let sun_dir = (sun_angle.cos(), sun_angle.sin());
        let sun_bounds = (
            sun_dir.0.min(0.0) + sun_dir.1.min(0.0),
            sun_dir.0.max(0.0) + sun_dir.1.max(0.0),
        );

        let peaks: Vec<(usize, usize)> = self
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.is_peak)
            .map(|(idx, _)| (idx % self.width, idx / self.width))
            .collect();

        let sea_distance = self.sea_distance_field();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let heat_temperature =
                    self.temperature_at(x, y, &heat_sources, wind, env, params.heat_source_radius);
                let base_light = self.sun_light_at(x, y, sun_dir, sun_bounds, env);
                // A heat source cell itself stays pinned to the heat model
                // even right at the coast: `reinject_environment_sources`
                // guarantees this every tick afterward anyway (its own pull
                // outweighs `sea_coolant_strength`), so blending it toward
                // `sea_coolant_value` here would only produce a cold-then-
                // reheating flicker on the very first ticks.
                self.cells[idx].temperature = if heat_sources.contains(&idx) {
                    heat_temperature
                } else {
                    let sea_t = (sea_distance[idx]
                        / source_cfg.sea_coolant_radius.max(f32::EPSILON))
                    .clamp(0.0, 1.0);
                    lerp(source_cfg.sea_coolant_value, heat_temperature, sea_t)
                };
                self.cells[idx].light = mountain_shaded_light(base_light, x, y, &peaks, source_cfg);
            }
        }
        self.heat_sources = heat_sources;
        self.sea_distance = sea_distance;
    }

    /// This world's wind vector from a drawn angle and `params.wind_strength`
    /// — factored out of `apply_environment_sources` purely so the formula
    /// has one definition; doesn't change that method's own draw sequence
    /// at all (still draws `wind_angle` from its own live `temp_rng` before
    /// heat source placement, exactly as before).
    fn wind_from_angle(angle: f32, strength: f32) -> (f32, f32) {
        (angle.cos() * strength, angle.sin() * strength)
    }

    /// Re-derives this world's wind vector (task 126) from a **fresh**
    /// `TEMPERATURE_SOURCE_SEED_OFFSET` stream — `wind_angle` is that
    /// stream's very first draw both here and in `apply_environment_sources`,
    /// so the two always agree bit-for-bit without `compute_rainfall`
    /// needing `apply_environment_sources`' own already-consumed `temp_rng`
    /// (which no longer exists by the time this runs) or a new `SimWorld`
    /// field to carry the vector across generation steps.
    fn wind_vector(&self, params: &WorldParams) -> (f32, f32) {
        let mut rng = StdRng::seed_from_u64(self.seed ^ TEMPERATURE_SOURCE_SEED_OFFSET);
        let wind_angle = rng.random_range(0.0..TAU);
        Self::wind_from_angle(wind_angle, params.wind_strength)
    }

    /// Computes and stores `Cell.rainfall` (task 126): a deterministic,
    /// single-pass approximation of orographic-lift/rain-shadow
    /// precipitation. Purely additive — read nowhere yet, same scope
    /// discipline as task 124. Three `[0, ~1]` terms combined and clamped:
    ///
    /// - **ocean moisture**: falls off linearly with a Sea-only water
    ///   distance (`sea_distance_field`, not the persisted `Cell.water_distance`
    ///   — Lake cells don't exist yet at this point in the pipeline, same
    ///   ordering constraint task 125's Swamp score hit).
    /// - **orographic lift**: the elevation gradient (`elevation_gradient`)
    ///   projected onto the wind direction, positive half only — terrain
    ///   rising into the wind forces air upward, condensing more moisture.
    /// - **rain shadow**: a bounded single-pass ray march *upwind* (the
    ///   `-wind` direction — where the air now at this cell came from) for
    ///   `rain_shadow_ray_steps` steps of `rain_shadow_step_length` cells
    ///   each, tracking the highest elevation crossed above this cell's
    ///   own. A ridge upwind depletes the moisture that would otherwise
    ///   reach here. Deliberately **not** the spec's iterative
    ///   per-tick-of-generation advection loop (§9.3) — a single bounded
    ///   pass per cell keeps generation time trivial and the field easy to
    ///   reason about; a documented simplification, not a placeholder.
    fn compute_rainfall(&mut self, config: &SimConfig, params: &WorldParams) {
        let source_cfg = &config.source;
        let wind = self.wind_vector(params);
        let wind_len = (wind.0 * wind.0 + wind.1 * wind.1).sqrt();
        let wind_dir = if wind_len > f32::EPSILON {
            (wind.0 / wind_len, wind.1 / wind_len)
        } else {
            (1.0, 0.0)
        };

        let elevations: Vec<f32> = self.cells.iter().map(|c| c.elevation).collect();
        let ocean_distance = self.sea_distance_field();

        let mut rainfall = vec![0.0f32; self.cells.len()];
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let ocean_moisture = 1.0
                    - (ocean_distance[idx]
                        / source_cfg.rain_ocean_moisture_radius.max(f32::EPSILON))
                    .clamp(0.0, 1.0);

                let (dx, dy) = self.elevation_gradient(x, y, &elevations);
                let lift = (dx * wind_dir.0 + dy * wind_dir.1).max(0.0)
                    * source_cfg.rain_orographic_lift_strength;

                let mut ridge: f32 = 0.0;
                for step in 1..=source_cfg.rain_shadow_ray_steps {
                    let dist = step as f32 * source_cfg.rain_shadow_step_length;
                    let sx = x as f32 - wind_dir.0 * dist;
                    let sy = y as f32 - wind_dir.1 * dist;
                    if sx < 0.0
                        || sy < 0.0
                        || sx > (self.width - 1) as f32
                        || sy > (self.height - 1) as f32
                    {
                        break;
                    }
                    let sample = elevations[self.index(sx.round() as usize, sy.round() as usize)];
                    ridge = ridge.max(sample - elevations[idx]);
                }
                let shadow = (ridge * source_cfg.rain_shadow_strength).clamp(0.0, 1.0);

                rainfall[idx] = (ocean_moisture + lift - shadow).clamp(0.0, 1.0);
            }
        }
        for (idx, cell) in self.cells.iter_mut().enumerate() {
            cell.rainfall = rainfall[idx];
        }
    }

    /// Grid distance from every cell to the nearest cell satisfying
    /// `is_source`, via multi-source BFS seeded from every matching cell at
    /// once — `O(cells)`, not `O(cells * sources)` a point-by-point
    /// nearest-search would cost with a large source set. Distance is in
    /// Moore-neighbour steps (matching `diffuse_environment`'s own
    /// 8-connectivity). Shared by `sea_distance_field` and
    /// `water_distance_field` (task 124), which differ only in which cells
    /// count as a source.
    fn bfs_distance_from(&self, is_source: impl Fn(&Cell) -> bool) -> Vec<f32> {
        self.bfs_distance_from_indices(
            self.cells
                .iter()
                .enumerate()
                .filter(|(_, cell)| is_source(cell))
                .map(|(idx, _)| idx),
        )
    }

    /// The same multi-source BFS as `bfs_distance_from`, seeded from
    /// explicit cell indices instead of a per-`Cell` predicate (task 131)
    /// — needed for sources that aren't yet a queryable `Cell` field, like
    /// `record_significant_depressions`' `lake_depressions` footprints,
    /// known before `Biome::Lake` is actually painted onto any cell.
    fn bfs_distance_from_indices(&self, sources: impl Iterator<Item = usize>) -> Vec<f32> {
        let mut distance = vec![f32::INFINITY; self.cells.len()];
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        for idx in sources {
            distance[idx] = 0.0;
            queue.push_back(idx);
        }
        while let Some(idx) = queue.pop_front() {
            let (x, y) = (idx % self.width, idx / self.width);
            let next = distance[idx] + 1.0;
            for n in self.moore_neighbours(x, y) {
                if next < distance[n] {
                    distance[n] = next;
                    queue.push_back(n);
                }
            }
        }
        distance
    }

    /// Task 131 (spec §9.4): `Cell.soil_moisture`, a climate-grounded
    /// refinement of task 125's `slope`/`water_distance` drainage proxy —
    ///
    /// ```text
    /// soil_moisture =
    ///     rainfall * retention(slope)
    ///     + river_bonus(river_distance)
    ///     + lake_bonus(lake_distance)
    ///     - evaporation(temperature)
    ///     - drainage(slope)
    /// ```
    ///
    /// (`curvature` is out of scope, same simplification task 124 already
    /// made — `slope` alone stands in for the drainage term, one of the
    /// spec's own listed alternatives.)
    ///
    /// `river_distance` and `lake_distance` are both real, not stubbed: task
    /// 127's rivers had already landed by the time this task was picked up,
    /// so `river_bonus` reads `Cell.is_river` directly via a fresh BFS
    /// (`bfs_distance_from`) instead of folding into `water_distance` as
    /// the task file's own fallback plan allowed. `lake_bonus` goes further
    /// still: task 129's `record_significant_depressions` already knows
    /// *where* Lake will be placed before `place_feature_biomes` actually
    /// paints `Biome::Lake` onto any cell, so `lake_depressions`' raw
    /// footprints (flattened into one seed set) can BFS a real distance
    /// field here — unlike the persisted `Cell.water_distance` field
    /// (task 132's finding), this doesn't need to wait for Lake to exist as
    /// a queryable `Biome`.
    ///
    /// Must run after `compute_rainfall` (`rainfall`), `compute_hydrology`
    /// (`is_river`), and `record_significant_depressions` (`lake_depressions`)
    /// — but before `classify_biomes`, which reads this field via
    /// `swamp_score` in place of the old proxy.
    fn compute_soil_moisture(&mut self, config: &SimConfig, lake_depressions: &[Vec<usize>]) {
        let cfg = &config.biome;
        let river_distance = self.bfs_distance_from(|cell| cell.is_river);
        let lake_distance =
            self.bfs_distance_from_indices(lake_depressions.iter().flatten().copied());

        let proximity = |distance: f32, max: f32, falloff: f32| -> f32 {
            smoothstep(max, max - falloff, distance)
        };

        for (idx, cell) in self.cells.iter_mut().enumerate() {
            let retention = (1.0 - cfg.soil_moisture_retention_slope_weight * cell.slope).max(0.0);
            let river_bonus = cfg.soil_moisture_river_bonus
                * proximity(
                    river_distance[idx],
                    cfg.soil_moisture_river_proximity_max,
                    cfg.soil_moisture_river_proximity_falloff,
                );
            let lake_bonus = cfg.soil_moisture_lake_bonus
                * proximity(
                    lake_distance[idx],
                    cfg.soil_moisture_lake_proximity_max,
                    cfg.soil_moisture_lake_proximity_falloff,
                );
            let evaporation = cfg.soil_moisture_evaporation_weight * cell.temperature;
            let drainage = cfg.soil_moisture_drainage_slope_weight * cell.slope;
            cell.soil_moisture =
                (cell.rainfall * retention + river_bonus + lake_bonus - evaporation - drainage)
                    .clamp(0.0, 1.0);
        }
    }

    /// Grid distance from every cell to its nearest `TerrainKind::Sea` cell
    /// (task 086 playtest follow-up) — feeds `apply_environment_sources`'
    /// coastal-cooling falloff, so it composes directly with
    /// `SourceConfig::sea_coolant_radius`, a cell count like
    /// `heat_source_radius`. Deliberately Sea-only, not folded into
    /// `water_distance_field`'s wider water set: widening this field's
    /// source set to lakes would be an unreviewed change to the
    /// temperature balance (task 124 note).
    fn sea_distance_field(&self) -> Vec<f32> {
        self.bfs_distance_from(|cell| cell.terrain == TerrainKind::Sea)
    }

    /// Grid distance from every cell to its nearest water cell — task 124's
    /// generalization of `sea_distance_field` to every `Biome` that reads as
    /// water (`DeepWater`, `ShallowWater`, `Lake`), not just
    /// `TerrainKind::Sea`, so a cell near an inland lake reads as "near
    /// water" the same way a coastal cell does. Must run after
    /// `place_feature_biomes`, which is what actually places `Biome::Lake`.
    fn water_distance_field(&self) -> Vec<f32> {
        self.bfs_distance_from(|cell| {
            matches!(
                cell.biome,
                Biome::DeepWater | Biome::ShallowWater | Biome::Lake
            )
        })
    }

    /// Places `params.heat_source_count` point sources via bounded-retry
    /// generation against `is_placeable` (no sources on `Sea`/peaks),
    /// the same attempt-loop/keep-best-seen pattern `generate_terrain`
    /// uses for the grid as a whole: each source retries up to
    /// `max_heat_source_placement_attempts` random cells, accepting the
    /// first that's both placeable and at least `heat_source_min_distance`
    /// from every already-placed source, else keeping the best-scoring
    /// placeable candidate seen (highest distance to the nearest existing
    /// source). Never panics on an unlucky draw: a source that finds no
    /// placeable cell at all in its attempt budget is simply skipped.
    fn place_heat_sources(
        &self,
        rng: &mut StdRng,
        source_cfg: &SourceConfig,
        params: &WorldParams,
    ) -> Vec<usize> {
        let mut sources: Vec<(usize, usize)> = Vec::new();
        for _ in 0..params.heat_source_count {
            let mut best: Option<(usize, usize, f32)> = None;
            for _ in 0..source_cfg.max_heat_source_placement_attempts.max(1) {
                let x = rng.random_range(0..self.width);
                let y = rng.random_range(0..self.height);
                if !self.is_placeable(x, y) {
                    continue;
                }
                let min_dist = sources
                    .iter()
                    .map(|&(sx, sy)| {
                        (((x as f32) - sx as f32).powi(2) + ((y as f32) - sy as f32).powi(2)).sqrt()
                    })
                    .fold(f32::INFINITY, f32::min);
                if min_dist >= source_cfg.heat_source_min_distance {
                    best = Some((x, y, min_dist));
                    break;
                }
                if best
                    .as_ref()
                    .is_none_or(|&(_, _, best_dist)| min_dist > best_dist)
                {
                    best = Some((x, y, min_dist));
                }
            }
            if let Some((x, y, _)) = best {
                sources.push((x, y));
            }
        }
        sources.iter().map(|&(x, y)| self.index(x, y)).collect()
    }

    /// Temperature at `(x, y)`: `EnvironmentConfig::source_temperature` at
    /// the nearest heat source, blending linearly down to `ambient_temperature`
    /// at `radius` cells away and beyond (first-pass falloff shape, tuned
    /// visually per task 086). `wind` shifts each source's effective
    /// position downwind by `params.wind_strength` cells before measuring
    /// distance, so cells downwind read hotter than the same distance
    /// upwind. Not linear in `(x, y)` (a `min` over several distances, then
    /// clamped) — this is why heat sources need `reinject_environment_sources`
    /// and a sun-direction light field doesn't.
    fn temperature_at(
        &self,
        x: usize,
        y: usize,
        heat_sources: &[usize],
        wind: (f32, f32),
        env: &EnvironmentConfig,
        radius: f32,
    ) -> f32 {
        if heat_sources.is_empty() {
            return env.ambient_temperature;
        }
        let (px, py) = (x as f32, y as f32);
        let min_dist = heat_sources
            .iter()
            .map(|&idx| {
                let (sx, sy) = (idx % self.width, idx / self.width);
                let (esx, esy) = (sx as f32 + wind.0, sy as f32 + wind.1);
                ((px - esx).powi(2) + (py - esy).powi(2)).sqrt()
            })
            .fold(f32::INFINITY, f32::min);
        let t = (min_dist / radius.max(f32::EPSILON)).clamp(0.0, 1.0);
        lerp(env.source_temperature, env.ambient_temperature, t)
    }

    /// Light at `(x, y)` from the per-world sun direction alone, before
    /// mountain shading: a linear projection of the cell's normalized
    /// position onto `sun_dir`, rescaled by `sun_bounds` (the projection's
    /// min/max across the grid for this direction) so the field spans
    /// exactly `[light_low, light_high]` regardless of which direction the
    /// sun landed on. Deliberately linear in `(x, y)` — an affine transform
    /// of a linear function stays linear, so this field alone is an exact
    /// fixed point of `diffuse_environment`'s Moore-blur, unlike temperature
    /// (see `sun_direction_light_field_is_a_fixed_point_of_diffusion` test).
    fn sun_light_at(
        &self,
        x: usize,
        y: usize,
        sun_dir: (f32, f32),
        sun_bounds: (f32, f32),
        env: &EnvironmentConfig,
    ) -> f32 {
        let nx = x as f32 / (self.width - 1).max(1) as f32;
        let ny = y as f32 / (self.height - 1).max(1) as f32;
        let proj = nx * sun_dir.0 + ny * sun_dir.1;
        let (min_p, max_p) = sun_bounds;
        let t = if (max_p - min_p) > f32::EPSILON {
            (proj - min_p) / (max_p - min_p)
        } else {
            0.5
        };
        lerp(env.light_low, env.light_high, t)
    }

    /// Blends each of `temperature`, `light`, `toxicity` toward the mean of
    /// its Moore neighbours, at `diffusion_rate` per tick (GDD §5.2). Reads
    /// neighbours from the snapshot (`self.cells`) and writes into
    /// `self.scratch`, the same double-buffering discipline `sim::step` uses
    /// for organism energy and residue, so the result never depends on
    /// iteration order (TECH_DESIGN.md invariant 1). Doesn't touch
    /// `residue`: that's a separate mechanic with its own decay (task 005),
    /// not an "environmental scalar" in the GDD §5.2 sense.
    pub fn diffuse_environment(&mut self, config: &SimConfig) {
        let rate = config.environment.diffusion_rate;
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let neighbours: Vec<usize> = self.moore_neighbours(x, y).collect();
                let n = neighbours.len() as f32;
                let mean = |get: fn(&Cell) -> f32| {
                    neighbours.iter().map(|&i| get(&self.cells[i])).sum::<f32>() / n
                };
                let cell = &self.cells[idx];
                self.scratch[idx].temperature =
                    cell.temperature + rate * (mean(|c| c.temperature) - cell.temperature);
                self.scratch[idx].light = cell.light + rate * (mean(|c| c.light) - cell.light);
                self.scratch[idx].toxicity =
                    cell.toxicity + rate * (mean(|c| c.toxicity) - cell.toxicity);
            }
        }
    }

    /// Counters `diffuse_environment`'s erosion of the standing
    /// temperature/toxicity features it doesn't otherwise preserve (task
    /// 085, extended by task 122): `heat_sources` are pulled back toward
    /// `source_temperature`, every cell within `sea_coolant_radius` of the
    /// sea gets a pull toward `sea_coolant_value` weighted by
    /// `self.sea_distance` (task 086: widened from a single Moore-ring
    /// nudge to the same radius-based falloff `apply_environment_sources`
    /// bakes in at generation, so the coastal band both starts and stays
    /// visible instead of only the literal coastline), and
    /// `toxic_swamp_cells` are pulled back toward `swamp_toxicity_value`
    /// (task 122 — without this, a chemolithotroph or `SurviveIn` objective
    /// relying on toxic ground would find it fading toward ambient over a
    /// long-running world, the same problem heat sources solved for
    /// temperature in task 085). Deliberately a separate method from
    /// `diffuse_environment`, called right after it from `sim::step` and
    /// operating on `self.scratch` (the write side diffusion just
    /// populated) — folding this into the blend loop would perturb
    /// `diffuse_environment`'s own fixed-point tests, which build a
    /// hand-crafted uniform field and expect diffusion alone to leave it
    /// untouched.
    pub fn reinject_environment_sources(&mut self, config: &SimConfig) {
        let source_cfg = &config.source;
        debug_assert!(
            source_cfg.reinjection_strength > config.environment.diffusion_rate,
            "reinjection_strength must exceed diffusion_rate, or diffusion erodes the source \
             faster than reinjection restores it"
        );
        debug_assert!(
            source_cfg.toxic_reinjection_strength > config.environment.diffusion_rate,
            "toxic_reinjection_strength must exceed diffusion_rate, or diffusion erodes toxic \
             Swamp cells faster than reinjection restores them"
        );
        for &idx in &self.heat_sources {
            let current = self.scratch[idx].temperature;
            self.scratch[idx].temperature +=
                source_cfg.reinjection_strength * (config.environment.source_temperature - current);
        }

        let radius = source_cfg.sea_coolant_radius.max(f32::EPSILON);
        for (idx, &distance) in self.sea_distance.iter().enumerate() {
            let weight = (1.0 - distance / radius).clamp(0.0, 1.0);
            if weight <= 0.0 {
                continue;
            }
            let current = self.scratch[idx].temperature;
            self.scratch[idx].temperature +=
                source_cfg.sea_coolant_strength * weight * (source_cfg.sea_coolant_value - current);
        }

        // `place_feature_biomes` (Crater/CrystalField/Lake/VolcanicVent)
        // runs after `classify_biomes` and can override a cell this list
        // was built from — the `biome == Swamp` guard keeps this self-
        // healing against that instead of ever reinjecting toxicity into a
        // feature biome that no longer wants it.
        for &idx in &self.toxic_swamp_cells {
            if self.cells[idx].biome != Biome::Swamp {
                continue;
            }
            let current = self.scratch[idx].toxicity;
            self.scratch[idx].toxicity += source_cfg.toxic_reinjection_strength
                * (config.environment.swamp_toxicity_value - current);
        }
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// This world's conditional-tag entry for `tag`, if any (task 096). A
    /// linear scan over `conditional_tags` — the set is tiny (~1-2 entries),
    /// so this avoids reaching for a `HashMap` for convenience (CLAUDE.md's
    /// ban on `HashMap` iteration in sim logic).
    pub fn conditional_gate(&self, tag: TagId) -> Option<&ConditionalTag> {
        self.conditional_tags.iter().find(|c| c.tag == tag)
    }

    /// Whether `species` is one of this world's wild, pre-existing
    /// populations (task 098) rather than a player-seedable one.
    pub fn is_wild(&self, species: SpeciesId) -> bool {
        self.wild_species.contains(&species)
    }

    /// Pushes `species` onto `self.species` and records the current `era`
    /// as its origin in `species_origin_era` (task 103) — the only place
    /// that era is captured, since it can't be reconstructed after the
    /// fact. Every call site that adds a new species (worldgen, `Splice`,
    /// tests) should go through this instead of `species.push` directly.
    pub fn push_species(&mut self, species: Species) -> SpeciesId {
        let id = SpeciesId(self.species.len() as u8);
        self.species.push(species);
        self.species_origin_era.push(self.era);
        id
    }

    /// Whether `species`'s lineage has ever occupied `terrain` this run
    /// (task 099). `false` for a species index past the tracker's current
    /// length — it simply hasn't had anything recorded yet, not an error.
    pub fn has_occupied_terrain(&self, species: SpeciesId, terrain: TerrainKind) -> bool {
        self.terrain_occupancy
            .get(species.0 as usize)
            .is_some_and(|occupancy| occupancy.has(terrain))
    }

    pub fn get(&self, x: usize, y: usize) -> &Cell {
        &self.cells[self.index(x, y)]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        let idx = self.index(x, y);
        &mut self.cells[idx]
    }

    /// Whether any species could occupy `(x, y)` today (task 067): the one
    /// centralized check the player's `Seed` action (`input.rs`) and
    /// in-tick reproduction (`sim.rs`) both gate placement through, instead
    /// of duplicating "is this Sea or a peak" inline at each call site. A
    /// future aquatic species only needs to extend `is_placeable_kind`
    /// (task 066) here — no other call site should ever check `terrain` or
    /// `is_peak` directly.
    pub fn is_placeable(&self, x: usize, y: usize) -> bool {
        let cell = self.get(x, y);
        is_placeable_kind(cell.terrain, cell.is_peak)
    }

    /// Same as `is_placeable`, taking a flat cell index (task 067) — lets
    /// `sim::step`'s reproduction filter Moore-neighbour indices (what
    /// `moore_neighbours` yields) without converting back to `(x, y)` first.
    pub fn is_placeable_index(&self, idx: usize) -> bool {
        let cell = &self.cells[idx];
        is_placeable_kind(cell.terrain, cell.is_peak)
    }

    /// Whether `species` has been granted `Sea` tolerance by a speciation
    /// event (task 107) — see `sea_tolerant_species`'s own doc comment.
    pub fn is_sea_tolerant(&self, species: SpeciesId) -> bool {
        self.sea_tolerant_species.contains(&species)
    }

    /// Species-aware variant of `is_placeable` (task 107): identical except
    /// a `Sea` cell is placeable for a species in `sea_tolerant_species`.
    /// Peaks stay unplaceable for every species regardless — that gate is
    /// structural, not a hazard tolerance question.
    pub fn is_placeable_for(&self, x: usize, y: usize, species: SpeciesId) -> bool {
        let cell = self.get(x, y);
        if cell.is_peak {
            return false;
        }
        cell.terrain != TerrainKind::Sea || self.is_sea_tolerant(species)
    }

    /// Species-aware variant of `is_placeable_index` (task 107), mirroring
    /// `is_placeable_for`'s `Sea`-tolerance exception.
    pub fn is_placeable_index_for(&self, idx: usize, species: SpeciesId) -> bool {
        let cell = &self.cells[idx];
        if cell.is_peak {
            return false;
        }
        cell.terrain != TerrainKind::Sea || self.is_sea_tolerant(species)
    }

    /// The seeded RNG, exposed only through `&mut self` so nobody can clone
    /// it out from under the world and break determinism (invariant 1).
    pub fn rng_mut(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    /// Draws the next seed from this world's own RNG stream, so reseeding
    /// (the `r` key) is derived from the run's own state, never the system
    /// clock (invariant 1: no non-determinism).
    pub fn next_seed(&mut self) -> u64 {
        self.rng.random()
    }

    /// Moore neighbourhood (8 cells), clipped at the grid borders (GDD §5.1).
    /// No wrap-around: the grid has real edges, so a corner cell has 3 neighbours.
    pub fn moore_neighbours(&self, x: usize, y: usize) -> impl Iterator<Item = usize> + '_ {
        let (x, y) = (x as isize, y as isize);
        (-1..=1).flat_map(move |dy| {
            (-1..=1).filter_map(move |dx| {
                if dx == 0 && dy == 0 {
                    return None;
                }
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx as usize >= self.width || ny as usize >= self.height {
                    return None;
                }
                Some(self.index(nx as usize, ny as usize))
            })
        })
    }
}

/// XOR salt decorrelating terrain generation's RNG stream (task 066) from
/// `SimWorld::rng` (tag/species/reproduction draws) — an arbitrary
/// constant, chosen only to not collide with the other salts below.
const TERRAIN_SEED_OFFSET: u64 = 0x9E37_79B9_7F4A_7C15;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for heat source placement and the
/// per-world wind direction draw (task 085) — a different constant so this
/// stream doesn't start in lockstep with the others.
const TEMPERATURE_SOURCE_SEED_OFFSET: u64 = 0x1656_67B1_9E37_79F9;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for the per-world sun direction
/// draw (task 085). Kept independent from `TEMPERATURE_SOURCE_SEED_OFFSET`
/// rather than sharing its stream, so hot/bright regions don't always
/// correlate spatially — an open question the design doc left to scoping.
const SUN_DIRECTION_SEED_OFFSET: u64 = 0x9E97_79B9_7F4A_1656;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for the biome patch-noise masks
/// (task 110) — a different constant so this stream doesn't start in
/// lockstep with the others.
const BIOME_SEED_OFFSET: u64 = 0x2545_F491_4F6C_DD1D;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for Cratere profondo's placement
/// search (task 111) — a different constant so this stream doesn't start
/// in lockstep with the others.
const CRATER_SEED_OFFSET: u64 = 0x27D4_EB2F_1656_67B1;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for Distesa di cristalli's
/// placement search (task 111).
const CRYSTAL_FIELD_SEED_OFFSET: u64 = 0x9E37_79B1_2545_F491;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for Lago's placement search
/// (task 111).
const LAKE_SEED_OFFSET: u64 = 0x4F6C_DD1D_C2B2_AE3D;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for the macro-region Voronoi seed
/// points (task 128) — a different constant so this stream doesn't start
/// in lockstep with `BIOME_SEED_OFFSET`'s own forest/swamp/toxic-patch
/// draws, even though both feed the same `classify_biomes` call.
const MACRO_REGION_SEED_OFFSET: u64 = 0x7F4A_7C15_9E37_79B9;

/// One feature biome's placement parameters (task 111) — bundled so
/// `place_feature_organic`'s call sites don't need a growing-argument
/// function signature per feature. `radius` is the mask's base radius
/// (task 123: a deformed disk, not a rectangle) — see `FeatureMask`.
struct FeaturePlacement {
    seed_offset: u64,
    radius: f32,
    min_placeable_fraction: f32,
    max_attempts: u32,
    biome: Biome,
    temperature: f32,
    light: f32,
    toxicity: f32,
}

/// One term of the deformed-disk angular-distortion field used by
/// `place_feature_organic` (task 123): a periodic wave over angle with an
/// integer harmonic number (so the field is smooth and closes up cleanly
/// at `2*PI`) and a random phase — same summed-sines technique as
/// `TerrainWave`/`wave_band_sum`, in angle-space instead of position-space.
struct AngleWave {
    harmonic: f32,
    phase: f32,
}

/// Draws one angular wave per harmonic `1..=count` (each harmonic used
/// exactly once, so the sum can't degenerate into one dominant term) with a
/// random phase, from `rng` — the sole source of randomness, so a given
/// feature's seed stream always produces the same silhouette.
fn angle_waves(rng: &mut StdRng, count: usize) -> Vec<AngleWave> {
    (1..=count.max(1))
        .map(|harmonic| AngleWave {
            harmonic: harmonic as f32,
            phase: rng.random_range(0.0..TAU),
        })
        .collect()
}

/// Sums `waves` at `angle` (radians), averaged over the wave count, into a
/// value in `[-1, 1]` — the angle-space equivalent of `wave_band_sum`.
fn angle_wave_sum(waves: &[AngleWave], angle: f32) -> f32 {
    let sum: f32 = waves
        .iter()
        .map(|wave| (wave.harmonic * angle + wave.phase).sin())
        .sum();
    sum / waves.len() as f32
}

/// A feature's deformed-disk mask (task 123), bundled once per placement
/// call so the mask-aware search/write helpers below share one small
/// parameter instead of a growing list. The disk's radius at a given angle
/// is `base_radius * (1 + distortion * angle_wave_sum(waves, angle))`.
struct FeatureMask<'a> {
    base_radius: f32,
    distortion: f32,
    waves: &'a [AngleWave],
    /// Half-width (cells) of the mask's worst-case bounding box around its
    /// center — `ceil(base_radius * (1 + distortion))`.
    extent: usize,
}

impl FeatureMask<'_> {
    fn new(base_radius: f32, distortion: f32, waves: &[AngleWave]) -> FeatureMask<'_> {
        let extent = (base_radius * (1.0 + distortion)).max(0.0).ceil() as usize;
        FeatureMask {
            base_radius,
            distortion,
            waves,
            extent,
        }
    }

    /// Whether the cell at offset `(dx, dy)` from the mask's center falls
    /// inside the deformed disk.
    fn contains(&self, dx: f32, dy: f32) -> bool {
        let dist = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx);
        let local_radius =
            self.base_radius * (1.0 + self.distortion * angle_wave_sum(self.waves, angle));
        dist <= local_radius.max(0.0)
    }
}

/// Dims `light` near `Mountain` peaks (task 085's "mountain shading"):
/// linear falloff from `mountain_shade_strength` at a peak's own cell to `0`
/// at `mountain_shade_radius` cells away, taking the strongest (nearest)
/// peak's contribution per cell. A separate, deliberately non-linear pass
/// on top of `sun_light_at`'s pure linear field — see that method's doc
/// comment for why the *unshaded* field is what's required to stay a
/// diffusion fixed point, not this.
fn mountain_shaded_light(
    base_light: f32,
    x: usize,
    y: usize,
    peaks: &[(usize, usize)],
    source_cfg: &SourceConfig,
) -> f32 {
    if peaks.is_empty() {
        return base_light;
    }
    let (px, py) = (x as f32, y as f32);
    let min_dist = peaks
        .iter()
        .map(|&(qx, qy)| (((px - qx as f32).powi(2)) + ((py - qy as f32).powi(2))).sqrt())
        .fold(f32::INFINITY, f32::min);
    let t = (min_dist / source_cfg.mountain_shade_radius.max(f32::EPSILON)).clamp(0.0, 1.0);
    let shade = source_cfg.mountain_shade_strength * (1.0 - t);
    (base_light - shade).max(0.0)
}

/// One term of the terrain elevation field: a plane wave traveling in
/// direction `(dir_x, dir_y)` with the given spatial frequency and phase —
/// the same dependency-free noise technique as `render.rs`'s decorative
/// `BackgroundWave`/`background_field`, except this field is real
/// simulation data (task 066), so it lives here instead.
struct TerrainWave {
    dir_x: f32,
    dir_y: f32,
    freq: f32,
    phase: f32,
}

/// Draws `count` waves with random direction, frequency uniform in
/// `freq_min..freq_max`, and random phase, all from `rng` — the sole
/// source of randomness, so the same seed always produces the same
/// terrain. Called once per frequency band (task 069): a low-frequency
/// range shapes macro-continents, a higher-frequency range layered on top
/// adds island/coastline detail.
fn terrain_waves(rng: &mut StdRng, count: usize, freq_min: f32, freq_max: f32) -> Vec<TerrainWave> {
    (0..count.max(1))
        .map(|_| {
            let angle = rng.random_range(0.0..TAU);
            TerrainWave {
                dir_x: angle.cos(),
                dir_y: angle.sin(),
                freq: rng.random_range(freq_min..freq_max),
                phase: rng.random_range(0.0..TAU),
            }
        })
        .collect()
}

/// Sums a wave band at normalized coordinates `(nx, ny)`, averaged over the
/// band's wave count, into a value in `[-1, 1]`.
fn wave_band_sum(waves: &[TerrainWave], nx: f32, ny: f32) -> f32 {
    let sum: f32 = waves
        .iter()
        .map(|wave| (wave.freq * (nx * wave.dir_x + ny * wave.dir_y) + wave.phase).sin())
        .sum();
    sum / waves.len() as f32
}

/// Blends the continent band (macro-continent shape) with the island band
/// (small island/coastline detail, weighted down by `island_weight`) at
/// normalized coordinates `(nx, ny)` into an elevation value in `[0, 1]`.
fn terrain_elevation(
    continent_waves: &[TerrainWave],
    island_waves: &[TerrainWave],
    island_weight: f32,
    nx: f32,
    ny: f32,
) -> f32 {
    let continent = wave_band_sum(continent_waves, nx, ny);
    let island = wave_band_sum(island_waves, nx, ny);
    let blended = continent + island * island_weight;
    let max_amplitude = 1.0 + island_weight;
    (blended / max_amplitude + 1.0) * 0.5
}

/// Rescales `elevations` in place so its own min and max span the full
/// `[0, 1]` range (task 069's follow-up correction, 2026-08-09): a low
/// wave count on the continent band means each world's raw amplitude
/// varies a lot by chance, so a fixed threshold against the raw `[0, 1]`
/// output produced wildly inconsistent sea/land ratios across seeds — some
/// worlds almost all land, others almost all sea, instead of the "compact
/// organic landmass in a much larger sea" read the terrain-map redesign
/// calls for. Stretching every world's own elevation range to fill `[0, 1]`
/// makes `TerrainConfig`'s thresholds land in the same *relative* place in
/// every world's distribution, regardless of the random draw's raw
/// amplitude. A degenerate all-equal field (max == min) is left untouched.
fn normalize_elevations(elevations: &mut [f32]) {
    let min = elevations.iter().copied().fold(f32::INFINITY, f32::min);
    let max = elevations.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let range = max - min;
    if range <= f32::EPSILON {
        return;
    }
    for e in elevations.iter_mut() {
        *e = (*e - min) / range;
    }
}

/// Elevation → `TerrainKind` band, per `TerrainConfig`'s thresholds.
fn classify_elevation(elevation: f32, terrain: &TerrainConfig) -> TerrainKind {
    if elevation < terrain.sea_threshold {
        TerrainKind::Sea
    } else if elevation < terrain.hill_threshold {
        TerrainKind::Plain
    } else if elevation < terrain.mountain_threshold {
        TerrainKind::Hill
    } else {
        TerrainKind::Mountain
    }
}

/// Whether a cell of terrain `kind`, with peak status `is_peak`, could ever
/// hold a species today — `Sea` and mountain peaks are off-limits to every
/// species that exists right now (task 067's placement gating and this
/// module's own generation-time viability checks both go through this one
/// function, so a future aquatic species only needs to extend it here).
fn is_placeable_kind(kind: TerrainKind, is_peak: bool) -> bool {
    kind != TerrainKind::Sea && !is_peak
}

/// Smooth 0->1 rise between `edge0` and `edge1` (cubic Hermite
/// smoothstep) — task 125's replacement for `classify_biomes`'
/// old hard threshold comparisons, so a biome's fitness for a cell
/// doesn't jump discontinuously as a scalar crosses a boundary.
/// `edge0 > edge1` flips the direction into a smooth *fall* instead
/// of a rise (used by every "lower is better" score below).
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if (edge1 - edge0).abs() < f32::EPSILON {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smooth "is `x` within `[min, max]`" band (task 125): rises from 0 to 1
/// over `width` below `min`, plateaus at 1 inside the band, falls back to
/// 0 over `width` above `max`. The banded counterpart of `smoothstep`'s
/// single-edge transition, for scores that want a preferred *range*
/// (Forest's temperature/light bands) rather than a one-sided threshold.
fn smooth_band(min: f32, max: f32, width: f32, x: f32) -> f32 {
    smoothstep(min - width, min, x) * (1.0 - smoothstep(max, max + width, x))
}

/// Desert's `[0, 1]` fitness score (task 125): high temperature and high
/// light, both smooth rises from `BiomeConfig`'s existing threshold pair.
fn desert_score(temperature: f32, light: f32, cfg: &BiomeConfig) -> f32 {
    let w = cfg.biome_score_transition_width;
    smoothstep(
        cfg.desert_temperature_min,
        cfg.desert_temperature_min + w,
        temperature,
    ) * smoothstep(cfg.desert_light_min, cfg.desert_light_min + w, light)
}

/// Tundra's `[0, 1]` fitness score (task 125): a smooth *fall* as
/// temperature rises past `tundra_temperature_max` — the mirror of
/// Desert's rise.
fn tundra_score(temperature: f32, cfg: &BiomeConfig) -> f32 {
    let w = cfg.biome_score_transition_width;
    smoothstep(
        cfg.tundra_temperature_max,
        cfg.tundra_temperature_max - w,
        temperature,
    )
}

/// Forest's `[0, 1]` fitness score (task 125): a climate term (smooth
/// temperature/light bands) plus a small additive patch-noise term — the
/// noise is no longer the primary gate (`redesign/procedural_biome_generation_spec_v2.md`
/// §1.4), just texture on top of the climate fit, so it can nudge which
/// side of a close climate call a cell lands on without being able to
/// conjure Forest somewhere climatically hostile to it.
fn forest_score(temperature: f32, light: f32, cfg: &BiomeConfig, noise: f32) -> f32 {
    let w = cfg.biome_score_transition_width;
    let climate = smooth_band(
        cfg.forest_temperature_min,
        cfg.forest_temperature_max,
        w,
        temperature,
    ) * smooth_band(cfg.forest_light_min, cfg.forest_light_max, w, light);
    (climate + noise.max(0.0) * cfg.patch_noise_weight).min(1.0)
}

/// Swamp's `[0, 1]` fitness score (task 125, refined by task 131): a
/// smooth rise past `swamp_soil_moisture_min` on `Cell.soil_moisture`
/// (task 131's climate-grounded wetness estimate), plus the same small
/// additive patch-noise term Forest uses — instead of the old
/// `toxicity` gate task 125 already replaced. Palude is a wetland/drainage
/// fact now, not a toxicity readout; toxicity is imposed as a separate
/// post-classification modifier on a sub-region of the cells this score
/// selects (see `classify_biomes`). `soil_moisture` subsumes task 125's
/// original `slope`/`water_distance` drainage proxy (it's already one of
/// `soil_moisture`'s own inputs) rather than sitting alongside it — see
/// `compute_soil_moisture`'s doc comment for the full formula.
fn swamp_score(soil_moisture: f32, cfg: &BiomeConfig, noise: f32) -> f32 {
    let w = cfg.biome_score_transition_width;
    let moisture_term = smoothstep(
        cfg.swamp_soil_moisture_min,
        cfg.swamp_soil_moisture_min + w,
        soil_moisture,
    );
    (moisture_term + noise.max(0.0) * cfg.patch_noise_weight).min(1.0)
}

/// Unbiased arg-max over a candidate `(Biome, score)` list (task 130) —
/// the same tie-breaking shape `classify_biomes`' Plain branch uses for
/// its macro-region-biased version, extracted here for the Hill/Mountain
/// branches that have no macro-region bias term to apply. `scores` must be
/// non-empty.
fn argmax_biome(scores: &[(Biome, f32)]) -> Biome {
    let mut best = scores[0];
    for &candidate in &scores[1..] {
        if candidate.1 > best.1 {
            best = candidate;
        }
    }
    best.0
}

/// Roccia nuda's `[0, 1]` fitness score (task 130, extends task 125's
/// `bare_rock_light_max`/`bare_rock_slope_light_bonus` gate to a smooth
/// score): a smooth *fall* as `light` rises past the same
/// slope-raised effective ceiling the original `TerrainKind::Hill` branch
/// used as a hard cutoff. Shared by both `TerrainKind::Hill` and
/// `TerrainKind::Mountain`'s arg-max — "what makes a cell Roccia nuda" has
/// one implementation, not two (this task's own acceptance criterion).
fn bare_rock_score(light: f32, slope: f32, cfg: &BiomeConfig) -> f32 {
    let w = cfg.biome_score_transition_width;
    let effective_light_max = cfg.bare_rock_light_max + cfg.bare_rock_slope_light_bonus * slope;
    smoothstep(effective_light_max + w, effective_light_max - w, light)
}

/// Ghiacciaio's `[0, 1]` fitness score (task 130): a smooth fall as
/// `temperature` rises past `glacier_temperature_max` — the same shape as
/// `tundra_score`, one sub-band down. Only ever compared within
/// `TerrainKind::Mountain`'s arg-max, where "alta quota" (spec §12.5) is
/// already guaranteed by the terrain kind itself, so no separate elevation
/// term is needed here.
fn glacier_score(temperature: f32, cfg: &BiomeConfig) -> f32 {
    let w = cfg.biome_score_transition_width;
    smoothstep(
        cfg.glacier_temperature_max,
        cfg.glacier_temperature_max - w,
        temperature,
    )
}

/// Prateria alpina's `[0, 1]` fitness score (task 130): a smooth "is
/// `temperature` within the moderate band above Glacier's cold end"
/// check, the same `smooth_band` shape Foresta's climate term uses. Only
/// ever compared within `TerrainKind::Mountain`'s arg-max.
fn alpine_meadow_score(temperature: f32, cfg: &BiomeConfig) -> f32 {
    smooth_band(
        cfg.alpine_meadow_temperature_min,
        cfg.alpine_meadow_temperature_max,
        cfg.biome_score_transition_width,
        temperature,
    )
}

/// No longer spawns a `SimWorld` at `Startup` (task 044): the first world
/// now comes into being when the player presses "New run" at the main menu
/// (`menu.rs::start_run`), so `Res<SimWorld>` genuinely doesn't exist until
/// `GameState::Playing` is entered — every system that reads it must be
/// gated to that state (or a substate of it, like `EraState`).
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, _app: &mut App) {}
}

/// Draws `params.active_tag_count` distinct tags from the global pool
/// (`TagConfig::global_tag_pool`) using the world's own RNG (task 038, GDD
/// §9) — not necessarily a contiguous range, unlike Phase 1's fixed
/// `TagId(0..active_tags_early)`. The draw order becomes each tag's
/// `TagSlot` in this world (position in the returned `Vec`).
fn select_active_tags(params: &WorldParams, config: &TagConfig, rng: &mut StdRng) -> Vec<TagId> {
    let pool: Vec<TagId> = (0..config.global_tag_pool as u8).map(TagId).collect();
    let count = (params.active_tag_count as usize).min(pool.len());
    pool.sample(rng, count).copied().collect()
}

/// Rolls this world's terrain-conditional tags (task 096): *which* `TagId`s
/// are conditional is a fixed, structural, cross-world constant — the first
/// `config.conditional_tag_count` `TagId`s by pool-wide convention — but the
/// trigger `TerrainKind` and `Mode` are rolled fresh here, from the world's
/// own seeded RNG, same moment as `select_active_tags`/`generate_matrix`.
/// Only conditional `TagId`s that actually landed in `active_tags` get an
/// entry — one not drawn into this world's active subset simply doesn't
/// exist here this run.
fn roll_conditional_tags(
    active_tags: &[TagId],
    config: &TagConfig,
    rng: &mut StdRng,
) -> Vec<ConditionalTag> {
    const TERRAIN_KINDS: [TerrainKind; 4] = [
        TerrainKind::Sea,
        TerrainKind::Plain,
        TerrainKind::Hill,
        TerrainKind::Mountain,
    ];
    (0..config.conditional_tag_count as u8)
        .map(TagId)
        .filter(|conditional_id| active_tags.contains(conditional_id))
        .map(|tag| {
            let terrain = *TERRAIN_KINDS
                .choose(rng)
                .expect("TERRAIN_KINDS is non-empty");
            let mode = if rng.random_bool(0.5) {
                Mode::Inducible
            } else {
                Mode::Repressible
            };
            ConditionalTag { tag, terrain, mode }
        })
        .collect()
}

/// Generates the world's secret tag matrix (GDD §5.5, §5.8), sized to
/// `slot_count` — the number of tags this world has active, regardless of
/// which `TagId`s from the global pool they are (task 036: the matrix is
/// indexed by `TagSlot`, so it only ever needs to know how many slots exist).
/// Each off-diagonal cell independently becomes non-zero with probability
/// `matrix_density` (task 038: taken from `WorldParams`, so it scales with
/// the difficulty curve instead of always reading `TagConfig::matrix_density`
/// directly); the diagonal always stays `0`. Afterwards, a negative 3-cycle
/// is forced among 3 distinct slots (overwriting whatever the random pass
/// produced there) to guarantee at least one coexistence-sustaining RPS
/// relationship exists (GDD §5.8, the worked example in §16.1) — there's no
/// closed-form way to sample a sparse asymmetric matrix that's guaranteed to
/// contain one, so forcing it after the fact is simpler than rejection
/// sampling and always terminates.
fn generate_matrix(
    slot_count: usize,
    matrix_density: f32,
    config: &TagConfig,
    rng: &mut StdRng,
) -> TagMatrix {
    let n = slot_count;
    let mut values = vec![0i8; n * n];

    for exerter in 0..n {
        for receiver in 0..n {
            if exerter == receiver {
                continue;
            }
            if rng.random_bool(matrix_density as f64) {
                values[exerter * n + receiver] = nonzero_intensity(config, rng);
            }
        }
    }

    if n >= 3 {
        let slots: Vec<usize> = (0..n).collect();
        let cycle: Vec<usize> = slots.sample(rng, 3).copied().collect();
        for &(exerter, receiver) in &[
            (cycle[0], cycle[1]),
            (cycle[1], cycle[2]),
            (cycle[2], cycle[0]),
        ] {
            values[exerter * n + receiver] = rng.random_range(config.effect_intensity_min..=-1);
        }
    }

    TagMatrix { size: n, values }
}

/// Draws a non-zero effect intensity in `[effect_intensity_min, effect_intensity_max]`
/// (retrying past `0`, so a "non-zero cell" from the density roll is never
/// silently downgraded to no effect).
fn nonzero_intensity(config: &TagConfig, rng: &mut StdRng) -> i8 {
    loop {
        let v = rng.random_range(config.effect_intensity_min..=config.effect_intensity_max);
        if v != 0 {
            return v;
        }
    }
}

/// Draws 1..=3 tags for a new species from the world's active pool (GDD
/// §5.5), without replacement, using the world's own RNG so species
/// composition stays deterministic given the same seed. Returns `TagSlot`s
/// (task 036), since `Species.tags` is keyed by position in the world's
/// active subset, not global pool identity.
///
/// Exhaustively searches for a tag set with exactly zero
/// `net_self_interaction`, starting at the rolled tag count and descending
/// to smaller sets if the pool has no safe combination at that size —
/// guaranteed to terminate at 1 tag, which is trivially always safe (a
/// single tag has no pairs to net against itself). Landing on exactly `0`
/// is what keeps a same-species neighbour perfectly neutral, leaving the
/// crowding penalty as the only thing that caps local density — the
/// assumption every carrying-capacity test relies on.
///
/// Net-negative self-interaction was the original playtest finding: such a
/// species dies as soon as it reproduces, because the child spawns adjacent
/// to the parent carrying the identical tag set. Net-*positive*
/// self-interaction (task 048, a later playtest) is the same mechanism in
/// the other direction and considerably more damaging: `sim::step`'s
/// `crowding_penalty` is a fixed `crowd_factor` (0.15) per occupied
/// neighbour, dwarfed by a matrix entry (`±effect_intensity_max`, up to
/// `±2`) — so *any* species whose own tags reinforce each other even
/// slightly turns clustering into unbounded growth the moment it
/// reproduces next to itself, the exact scenario that saturated the whole
/// grid before this fix.
///
/// This used to be a random-sample-with-retry search (up to
/// `max_self_conflict_draws` tries), which could fail to find an exact zero
/// even after every retry — not from bad luck, but because a small active
/// tag pool (5 tags at world 0) only has `C(5,3) = 10` possible 3-tag
/// combinations, and roughly 15% of randomly-generated matrices have *none*
/// of those 10 net exactly zero (task 088). Exhaustive search removes that
/// gap entirely: it either finds every zero-net candidate at the rolled
/// size, or proves none exists and tries a smaller size instead, so the
/// result is always exactly zero, never "closest to zero".
pub fn draw_species_tags(world: &mut SimWorld, config: &SimConfig) -> Vec<TagSlot> {
    let rolled_n = world
        .rng
        .random_range(config.tags.tags_per_species_min..=config.tags.tags_per_species_max)
        as usize;
    let slot_count = world.active_tags.len();
    let slots: Vec<TagSlot> = (0..slot_count as u8).map(TagSlot).collect();

    let mut n = rolled_n.min(slot_count);
    loop {
        let candidates: Vec<Vec<TagSlot>> = combinations(&slots, n)
            .into_iter()
            .filter(|candidate| net_self_interaction(&world.matrix, candidate) == 0)
            .collect();
        if let Some(chosen) = candidates.choose(&mut world.rng) {
            return chosen.clone();
        }
        if n == 0 {
            // Only reachable if the active tag pool itself is empty — not a
            // real world state today, but avoids an infinite loop if it
            // ever were.
            return Vec::new();
        }
        n -= 1;
    }
}

/// Fixed, unordered word list `draw_species_name` samples from (task 095,
/// replacing task 029's fixed `SpeciesId`-indexed scheme — see that
/// function's own doc comment for why). Species carry no design secrecy
/// constraint (unlike tags, GDD §11), so a generated name is purely a
/// legibility upgrade — wide enough that repeats stay rare across an
/// ordinary run, even with `Splice` adding species past the pool size.
const SPECIES_NAMES: &[&str] = &[
    "Nyx", "Kael", "Sable", "Rook", "Vesk", "Lira", "Thorn", "Onyx", "Fenn", "Skye", "Brakk",
    "Cass", "Drys", "Elm", "Fira", "Grix", "Vor", "Zeph", "Quil", "Wrey", "Sten", "Blythe",
    "Corvin", "Ashka", "Ryn", "Tavik", "Ember", "Lask", "Moth", "Juno", "Halo", "Reef", "Snow",
    "Dune", "Ashe", "Wraith", "Fable", "Glim", "Hollow", "Marrow", "Nettle", "Opal", "Pyre",
    "Quartz", "Rime", "Slate", "Thicket", "Umbra",
];

/// Draws this species' display name from the world's own seeded RNG
/// (task 095) — never `rand::rng()` (TECH_DESIGN.md invariant 1), same
/// discipline `draw_species_tags` follows. Called once per species at
/// construction and stored on `Species::name`, never recomputed later.
/// Names carry no GDD §11 secrecy constraint (unlike tags/matrix), so this
/// reads from the world's main RNG stream directly rather than a
/// decorrelated offset stream the way terrain/toxic-zone/heat-source
/// generation does.
pub fn draw_species_name(world: &mut SimWorld) -> String {
    SPECIES_NAMES
        .choose(&mut world.rng)
        .expect("SPECIES_NAMES is a non-empty fixed list")
        .to_string()
}

/// Every `k`-sized combination of `slots`, in lexicographic order. Hand-rolled
/// rather than pulling in `itertools` (not a direct dependency of this
/// project) — the search space here is tiny (at most `C(8,3) = 56` given the
/// game's own tag-pool/tags-per-species bounds), so a small recursive
/// generator is simpler than adding a crate for one call site.
fn combinations(slots: &[TagSlot], k: usize) -> Vec<Vec<TagSlot>> {
    if k == 0 {
        return vec![Vec::new()];
    }
    if k > slots.len() {
        return Vec::new();
    }
    let mut result = Vec::new();
    for i in 0..=slots.len() - k {
        for mut rest in combinations(&slots[i + 1..], k - 1) {
            rest.insert(0, slots[i]);
            result.push(rest);
        }
    }
    result
}

/// Sum of the matrix effect every tag in `tags` exerts on every other tag in
/// `tags` — what a species feels from a same-species neighbour carrying an
/// identical tag set, the case reproduction always produces.
pub fn net_self_interaction(matrix: &TagMatrix, tags: &[TagSlot]) -> i32 {
    let mut total = 0i32;
    for &a in tags {
        for &b in tags {
            if a != b {
                total += matrix.get(a, b) as i32;
            }
        }
    }
    total
}

/// Exact at `t = 0.0` and `t = 1.0` (unlike `from + (to - from) * t`), which
/// matters here: tests assert the grid extremes equal the GDD values exactly.
fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from * (1.0 - t) + to * t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SimConfig {
        SimConfig::default()
    }

    #[test]
    fn same_seed_produces_identical_state() {
        let config = test_config();
        let mut a = SimWorld::new(42, &config);
        let mut b = SimWorld::new(42, &config);

        assert_eq!(a.width, b.width);
        assert_eq!(a.height, b.height);
        assert_eq!(a.cells, b.cells);
        assert_eq!(a.species, b.species);
        assert_eq!(a.tick, b.tick);
        assert_eq!(a.era, b.era);
        assert_eq!(a.season, b.season);
        assert_eq!(a.seed, b.seed);
        assert_eq!(a.rng_mut().random::<u64>(), b.rng_mut().random::<u64>());
    }

    #[test]
    fn different_seeds_produce_different_state() {
        let config = test_config();
        let mut a = SimWorld::new(42, &config);
        let mut b = SimWorld::new(43, &config);

        assert_ne!(a.seed, b.seed);
        assert_ne!(a.rng_mut().random::<u64>(), b.rng_mut().random::<u64>());
    }

    #[test]
    fn roll_conditional_tags_skips_ids_not_in_active_set() {
        let mut config = test_config();
        config.tags.conditional_tag_count = 1;
        let mut rng = StdRng::seed_from_u64(1);
        // TagId(0) is the conditional identity, but it's not in this
        // world's active subset — no entry should be produced, and this
        // must not panic.
        let active_tags = vec![TagId(1), TagId(2)];
        let rolled = roll_conditional_tags(&active_tags, &config.tags, &mut rng);
        assert!(rolled.is_empty());
    }

    #[test]
    fn roll_conditional_tags_includes_active_conditional_ids() {
        let mut config = test_config();
        config.tags.conditional_tag_count = 1;
        let mut rng = StdRng::seed_from_u64(1);
        let active_tags = vec![TagId(0), TagId(1)];
        let rolled = roll_conditional_tags(&active_tags, &config.tags, &mut rng);
        assert_eq!(rolled.len(), 1);
        assert_eq!(rolled[0].tag, TagId(0));
    }

    #[test]
    fn conditional_gate_finds_the_matching_entry_by_tag_id() {
        let config = test_config();
        let mut world = SimWorld::new(42, &config);
        world.conditional_tags = vec![ConditionalTag {
            tag: TagId(3),
            terrain: TerrainKind::Mountain,
            mode: Mode::Repressible,
        }];
        assert!(world.conditional_gate(TagId(3)).is_some());
        assert!(world.conditional_gate(TagId(4)).is_none());
    }

    #[test]
    fn moore_neighbours_respects_borders() {
        let config = test_config();
        let world = SimWorld::new(42, &config);

        assert_eq!(world.moore_neighbours(0, 0).count(), 3);
        assert_eq!(
            world.moore_neighbours(world.width / 2, 0).count(),
            5,
            "top edge, not a corner"
        );
        assert_eq!(
            world
                .moore_neighbours(world.width / 2, world.height / 2)
                .count(),
            8,
            "interior cell"
        );
    }

    /// Task 085: replaces the old fixed-corner-value check (no longer
    /// meaningful once temperature/light come from a randomized source/sun
    /// placement) with the source model's structural invariants — light
    /// spans its full configured range somewhere on the grid (mountain
    /// shading only ever *reduces* light, so the unshaded extremes are
    /// still upper/lower bounds), and heat source cells read hotter than
    /// the ambient baseline.
    #[test]
    fn source_model_spans_its_configured_light_range_and_heats_its_sources() {
        let config = test_config();
        let world = SimWorld::new(42, &config);
        let env = &config.environment;

        let max_light = world.cells.iter().map(|c| c.light).fold(f32::MIN, f32::max);
        let min_light = world.cells.iter().map(|c| c.light).fold(f32::MAX, f32::min);
        assert!(
            max_light <= env.light_high + f32::EPSILON,
            "light should never exceed light_high, got {max_light}"
        );
        assert!(
            min_light <= env.light_low + f32::EPSILON,
            "the grid corner farthest from the sun should reach light_low, got {min_light}"
        );
        assert!(
            max_light - min_light > 0.1,
            "the sun direction should produce a real gradient, not a near-flat field"
        );

        assert!(
            !world.heat_sources.is_empty(),
            "world_index 0's default config always places heat sources"
        );
        for &idx in &world.heat_sources {
            assert!(
                world.cells[idx].temperature > env.ambient_temperature,
                "a heat source cell should read hotter than the ambient baseline"
            );
        }
    }

    #[test]
    fn environment_scalars_stay_in_unit_range() {
        let config = test_config();
        let world = SimWorld::new(42, &config);

        for cell in &world.cells {
            assert!((0.0..=1.0).contains(&cell.light));
            assert!((0.0..=1.0).contains(&cell.temperature));
            assert!((0.0..=1.0).contains(&cell.toxicity));
        }
    }

    /// Task 113: since `place_toxic_zone`'s standalone rectangle is gone,
    /// every cell's toxicity now comes from exactly one of the placed
    /// feature biomes' fixed imposed value or Swamp's post-classification
    /// modifier (task 125 §12.4) — `0.0` everywhere else.
    #[test]
    fn cell_toxicity_matches_its_biome_across_seeds() {
        let config = test_config();
        for seed in 0..10u64 {
            let world = SimWorld::new(seed, &config);
            for y in 0..world.height {
                for x in 0..world.width {
                    let cell = world.get(x, y);
                    match cell.biome {
                        Biome::Crater => assert_eq!(
                            cell.toxicity, config.biome.crater_toxicity,
                            "seed {seed}, cell ({x}, {y})"
                        ),
                        Biome::CrystalField => assert_eq!(
                            cell.toxicity, config.biome.crystal_field_toxicity,
                            "seed {seed}, cell ({x}, {y})"
                        ),
                        Biome::Lake => assert_eq!(
                            cell.toxicity, config.biome.lake_toxicity,
                            "seed {seed}, cell ({x}, {y})"
                        ),
                        // Swamp's toxicity is a post-classification modifier
                        // on a noise-gated sub-region (task 125): either
                        // ambient 0.0 or the imposed `swamp_toxicity_value`.
                        Biome::Swamp => assert!(
                            cell.toxicity == 0.0
                                || cell.toxicity == config.environment.swamp_toxicity_value,
                            "seed {seed}, cell ({x}, {y}): unexpected Swamp toxicity {}",
                            cell.toxicity
                        ),
                        _ => assert_eq!(cell.toxicity, 0.0, "seed {seed}, cell ({x}, {y})"),
                    }
                }
            }
        }
    }

    /// Task 113: `SurviveIn`'s zone check (`objectives.rs::cell_in_zone`)
    /// now reads `Cell::biome == Swamp` directly rather than a placement
    /// search bounded against placeable terrain — this is only sound if
    /// Swamp can never land on `Sea` or a peak. `classify_biomes` only ever
    /// assigns `Biome::Swamp` from its `TerrainKind::Plain` branch, so this
    /// holds by construction; this test guards that invariant against a
    /// future change to the classification match arms.
    #[test]
    fn swamp_cells_are_always_placeable_land_across_seeds() {
        let config = test_config();
        for seed in 0..30u64 {
            let world = SimWorld::new(seed, &config);
            for (idx, cell) in world.cells.iter().enumerate() {
                if cell.biome != Biome::Swamp {
                    continue;
                }
                let (x, y) = (idx % world.width, idx / world.width);
                assert!(
                    cell.terrain != TerrainKind::Sea && !cell.is_peak,
                    "seed {seed}, cell ({x}, {y}): Swamp on unplaceable terrain"
                );
            }
        }
    }

    #[test]
    fn terrain_generation_is_deterministic_for_a_given_seed() {
        let config = test_config();
        let a = SimWorld::new(7, &config);
        let b = SimWorld::new(7, &config);

        let terrain_a: Vec<(TerrainKind, bool)> =
            a.cells.iter().map(|c| (c.terrain, c.is_peak)).collect();
        let terrain_b: Vec<(TerrainKind, bool)> =
            b.cells.iter().map(|c| (c.terrain, c.is_peak)).collect();
        assert_eq!(terrain_a, terrain_b);
    }

    #[test]
    fn terrain_varies_with_seed() {
        let config = test_config();
        let a = SimWorld::new(7, &config);
        let b = SimWorld::new(8, &config);

        let terrain_a: Vec<TerrainKind> = a.cells.iter().map(|c| c.terrain).collect();
        let terrain_b: Vec<TerrainKind> = b.cells.iter().map(|c| c.terrain).collect();
        assert_ne!(terrain_a, terrain_b);
    }

    #[test]
    fn biome_classification_is_deterministic_for_a_given_seed() {
        let config = test_config();
        let a = SimWorld::new(7, &config);
        let b = SimWorld::new(7, &config);

        let biomes_a: Vec<Biome> = a.cells.iter().map(|c| c.biome).collect();
        let biomes_b: Vec<Biome> = b.cells.iter().map(|c| c.biome).collect();
        assert_eq!(biomes_a, biomes_b);
    }

    /// Counts adjacent `TerrainKind::Plain` cell pairs (right/below only,
    /// same "each shared edge exactly once" trick `render.rs::draw_boundaries`
    /// uses) whose `Biome` differs — restricted to `Plain`-kind pairs since
    /// that's the only terrain the macro-region bias (task 128) touches;
    /// including Sea/Mountain/Hill boundaries would dilute the signal with
    /// terrain-driven transitions the bias was never meant to affect.
    fn plain_biome_transition_count(world: &SimWorld) -> usize {
        let mut transitions = 0;
        for y in 0..world.height {
            for x in 0..world.width {
                let here = world.get(x, y);
                if here.terrain != TerrainKind::Plain {
                    continue;
                }
                if x + 1 < world.width {
                    let right = world.get(x + 1, y);
                    if right.terrain == TerrainKind::Plain && right.biome != here.biome {
                        transitions += 1;
                    }
                }
                if y + 1 < world.height {
                    let below = world.get(x, y + 1);
                    if below.terrain == TerrainKind::Plain && below.biome != here.biome {
                        transitions += 1;
                    }
                }
            }
        }
        transitions
    }

    /// Task 128 (spec §18.6's continuity metric): the macro-region bias
    /// should measurably reduce noise-driven speckle between adjacent
    /// `Plain`-kind cells, not just redistribute which biome wins where.
    /// Compares the shipped default (`macro_region_bias_weight` from
    /// config) against the same seeds with the bias switched off
    /// (`macro_region_bias_weight: 0.0`, which makes `classify_biomes`'s
    /// multiplicative bias a no-op — `score * (1.0 + 0.0) == score`).
    /// Aggregate across seeds, not a hard per-seed assert: the bias nudges
    /// probabilities, it doesn't guarantee a monotonic drop on every draw.
    #[test]
    fn macro_region_bias_reduces_plain_biome_transitions_on_average() {
        let biased_config = test_config();
        let mut unbiased_config = test_config();
        unbiased_config.biome.macro_region_bias_weight = 0.0;

        let n_seeds = 20u64;
        let mut total_biased = 0usize;
        let mut total_unbiased = 0usize;
        for seed in 0..n_seeds {
            total_biased += plain_biome_transition_count(&SimWorld::new(seed, &biased_config));
            total_unbiased += plain_biome_transition_count(&SimWorld::new(seed, &unbiased_config));
        }
        assert!(
            total_biased < total_unbiased,
            "expected the macro-region bias to reduce Plain-biome transitions on average \
             across {n_seeds} seeds: biased {total_biased}, unbiased {total_unbiased}"
        );
    }

    #[test]
    fn every_areal_biome_is_reachable_across_seeds() {
        let config = test_config();
        let mut seen = std::collections::HashSet::new();
        for seed in 0..40u64 {
            let world = SimWorld::new(seed, &config);
            for cell in &world.cells {
                seen.insert(cell.biome);
            }
        }
        for biome in [
            Biome::DeepWater,
            Biome::ShallowWater,
            Biome::Plain,
            Biome::Hill,
            Biome::Mountain,
            Biome::Peak,
            Biome::Desert,
            Biome::Tundra,
            Biome::BareRock,
            Biome::Forest,
            Biome::Swamp,
        ] {
            assert!(
                seen.contains(&biome),
                "{biome:?} never reached across 40 seeds"
            );
        }
    }

    #[test]
    fn feature_biomes_never_overlap_each_other() {
        // `place_feature_biomes` places Crater/CrystalField (then Lake,
        // checked separately below since task 129) in that order (Bocca
        // vulcanica's vent footprint first), each requiring zero overlap
        // with every cell already claimed by an earlier one. Task 123
        // replaced the rectangle footprint with a deformed disk, so an
        // exact `width * height` cell count no longer applies: the radius
        // wobbles with the angular-distortion waves, and grid-edge
        // clipping can shrink it further, both by design. Instead, check
        // each feature's footprint lands in a generous band around its
        // ideal disk area `PI * radius^2` — still tight enough to catch the
        // shrinkage a real cross-feature overlap would cause (an
        // overlapping cell is repainted by whichever feature places
        // second, so the earlier feature's count would come in well under
        // this band).
        let config = test_config();
        let checks = [
            (Biome::Crater, config.biome.crater_radius),
            (Biome::CrystalField, config.biome.crystal_field_radius),
        ];
        for seed in 0..40u64 {
            let world = SimWorld::new(seed, &config);
            for &(biome, radius) in &checks {
                let count = world.cells.iter().filter(|c| c.biome == biome).count();
                let ideal_area = std::f32::consts::PI * radius * radius;
                assert!(count > 0, "seed {seed}: {biome:?} was never placed");
                assert!(
                    count as f32 >= ideal_area * 0.25,
                    "seed {seed}: {biome:?} footprint ({count} cells) far below its ideal \
                     disk area ({ideal_area}) — possible overlap shrinkage"
                );
            }
        }
    }

    /// Task 129: Lake no longer has one fixed-shape source, so it can't
    /// share the disk-area check above — a depression-derived footprint
    /// (10-100 cells per component, `BiomeConfig::lake_depression_*`, and
    /// possibly several components summed) has a completely different
    /// shape budget than the organic-mask fallback's `PI * lake_radius^2`
    /// disk. This checks the property that actually matters regardless of
    /// source: Lake exists, and its total footprint isn't a corrupted
    /// sliver (a real cross-feature overlap would still shrink it toward
    /// zero the same way it would for Crater/CrystalField).
    #[test]
    fn lake_footprint_is_never_absurdly_small() {
        let config = test_config();
        for seed in 0..40u64 {
            let world = SimWorld::new(seed, &config);
            let count = world
                .cells
                .iter()
                .filter(|c| c.biome == Biome::Lake)
                .count();
            assert!(count > 0, "seed {seed}: Lake was never placed");
            assert!(
                count as u32 >= config.biome.lake_depression_min_size / 2,
                "seed {seed}: Lake footprint ({count} cells) implausibly small — possible \
                 overlap shrinkage"
            );
        }
    }

    /// Task 129's own acceptance criterion: depression-derived lakes
    /// should be terrain-grounded, not coincidentally similar to the old
    /// context-blind random search — a lake cell should not, e.g., sit on
    /// a local elevation maximum. Aggregate across seeds and cells, not a
    /// hard per-cell assert: the organic-mask *fallback* (still elevation-
    /// blind, unchanged from task 123) can occasionally place a Lake cell
    /// that violates this on a low-relief world, which is exactly when the
    /// fallback triggers — this test would be flaky, not meaningfully
    /// stricter, if it demanded zero violations.
    #[test]
    fn lake_cells_are_usually_not_local_elevation_maxima() {
        let config = test_config();
        let mut lake_cells = 0u32;
        let mut local_maxima = 0u32;
        for seed in 0..40u64 {
            let world = SimWorld::new(seed, &config);
            for (idx, cell) in world.cells.iter().enumerate() {
                if cell.biome != Biome::Lake {
                    continue;
                }
                lake_cells += 1;
                let (x, y) = (idx % world.width, idx / world.width);
                let is_local_max = world
                    .moore_neighbours(x, y)
                    .all(|n| world.cells[n].elevation <= cell.elevation);
                if is_local_max {
                    local_maxima += 1;
                }
            }
        }
        assert!(lake_cells > 0, "no Lake cells found across 40 seeds");
        assert!(
            local_maxima * 20 < lake_cells,
            "expected well under 5% of Lake cells to be local elevation maxima, got \
             {local_maxima}/{lake_cells}"
        );
    }

    /// Task 131's own acceptance criterion (spec §18.5-style relational
    /// check): `Cell.soil_moisture` should correlate positively with
    /// `rainfall` and negatively with `temperature`/`slope`, matching the
    /// formula's own sign on each term (`compute_soil_moisture`'s doc
    /// comment). Checked by splitting all cells from 20 seeds into
    /// above-median/below-median buckets per driver and comparing mean
    /// `soil_moisture` between them — a simple, robust correlation-sign
    /// check (no assumption of linearity), same aggregate-over-seeds
    /// spirit as `leeward_side_of_the_tallest_peak_reads_drier_than_windward_on_average`.
    #[test]
    fn soil_moisture_correlates_with_rainfall_temperature_and_slope_as_designed() {
        let config = test_config();
        let mut rainfall = Vec::new();
        let mut temperature = Vec::new();
        let mut slope = Vec::new();
        let mut soil_moisture = Vec::new();
        for seed in 0..20u64 {
            let world = SimWorld::new(seed, &config);
            for cell in world.cells.iter() {
                rainfall.push(cell.rainfall);
                temperature.push(cell.temperature);
                slope.push(cell.slope);
                soil_moisture.push(cell.soil_moisture);
            }
        }

        let mean_soil_moisture_split = |driver: &[f32]| -> (f32, f32) {
            let mut sorted = driver.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = sorted[sorted.len() / 2];
            let (mut below_sum, mut below_n, mut above_sum, mut above_n) =
                (0.0f32, 0u32, 0.0f32, 0u32);
            for (d, m) in driver.iter().zip(soil_moisture.iter()) {
                if *d < median {
                    below_sum += m;
                    below_n += 1;
                } else {
                    above_sum += m;
                    above_n += 1;
                }
            }
            (
                below_sum / below_n.max(1) as f32,
                above_sum / above_n.max(1) as f32,
            )
        };

        let (rain_low, rain_high) = mean_soil_moisture_split(&rainfall);
        assert!(
            rain_high > rain_low,
            "soil_moisture should be higher with more rainfall: below-median mean {rain_low:.3}, \
             above-median mean {rain_high:.3}"
        );

        let (temp_low, temp_high) = mean_soil_moisture_split(&temperature);
        assert!(
            temp_high < temp_low,
            "soil_moisture should be lower with more temperature (evaporation): below-median \
             mean {temp_low:.3}, above-median mean {temp_high:.3}"
        );

        let (slope_low, slope_high) = mean_soil_moisture_split(&slope);
        assert!(
            slope_high < slope_low,
            "soil_moisture should be lower with more slope (drainage/lower retention): \
             below-median mean {slope_low:.3}, above-median mean {slope_high:.3}"
        );
    }

    /// Task 130's own acceptance criterion (spec §18.5-style relational
    /// check, same shape as task 126's windward/leeward test): within
    /// `TerrainKind::Mountain`, Glacier should only occur at low
    /// temperature, and BareRock/AlpineMeadow should show the
    /// slope/temperature correlation their scores are built on. Checked as
    /// aggregate means across a sample of seeds, not per-cell hard asserts
    /// — individual cells can land on a close arg-max tie against a
    /// neighbouring candidate, which is expected noise, not a bug.
    #[test]
    fn mountain_sub_bands_correlate_with_temperature_and_slope_as_designed() {
        let config = test_config();
        let mut glacier_temp = Vec::new();
        let mut alpine_meadow_temp = Vec::new();
        let mut bare_rock_temp = Vec::new();
        let mut mountain_temp = Vec::new();
        let mut glacier_slope = Vec::new();
        let mut bare_rock_slope = Vec::new();
        for seed in 0..30u64 {
            let world = SimWorld::new(seed, &config);
            for cell in world.cells.iter() {
                match cell.biome {
                    Biome::Glacier => {
                        glacier_temp.push(cell.temperature);
                        glacier_slope.push(cell.slope);
                    }
                    Biome::AlpineMeadow => alpine_meadow_temp.push(cell.temperature),
                    Biome::BareRock if cell.terrain == TerrainKind::Mountain => {
                        bare_rock_temp.push(cell.temperature);
                        bare_rock_slope.push(cell.slope);
                    }
                    Biome::Mountain => mountain_temp.push(cell.temperature),
                    _ => {}
                }
            }
        }
        let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len().max(1) as f32;

        assert!(
            !glacier_temp.is_empty(),
            "no Glacier cells found across 30 seeds"
        );
        assert!(
            !alpine_meadow_temp.is_empty(),
            "no AlpineMeadow cells found across 30 seeds"
        );
        assert!(
            !bare_rock_temp.is_empty(),
            "no Mountain-terrain BareRock cells found across 30 seeds"
        );

        // Glacier reads as the coldest of the four Mountain candidates.
        assert!(
            mean(&glacier_temp) < mean(&alpine_meadow_temp),
            "Glacier mean temperature ({:.3}) should be colder than AlpineMeadow's ({:.3})",
            mean(&glacier_temp),
            mean(&alpine_meadow_temp)
        );
        if !mountain_temp.is_empty() {
            assert!(
                mean(&glacier_temp) < mean(&mountain_temp),
                "Glacier mean temperature ({:.3}) should be colder than plain Mountain's ({:.3})",
                mean(&glacier_temp),
                mean(&mountain_temp)
            );
        }

        // BareRock reads as steeper, on average, than Glacier within Mountain terrain.
        assert!(
            mean(&bare_rock_slope) > mean(&glacier_slope),
            "Mountain BareRock mean slope ({:.3}) should exceed Glacier's ({:.3})",
            mean(&bare_rock_slope),
            mean(&glacier_slope)
        );
    }

    #[test]
    fn feature_biomes_impose_their_target_scalars() {
        let config = test_config();
        for seed in 0..20u64 {
            let world = SimWorld::new(seed, &config);
            for cell in &world.cells {
                match cell.biome {
                    Biome::Crater => {
                        assert_eq!(cell.temperature, config.biome.crater_temperature);
                        assert_eq!(cell.light, config.biome.crater_light);
                        assert_eq!(cell.toxicity, config.biome.crater_toxicity);
                    }
                    Biome::CrystalField => {
                        assert_eq!(cell.temperature, config.biome.crystal_field_temperature);
                        assert_eq!(cell.light, config.biome.crystal_field_light);
                        assert_eq!(cell.toxicity, config.biome.crystal_field_toxicity);
                    }
                    Biome::Lake => {
                        assert_eq!(cell.temperature, config.biome.lake_temperature);
                        assert_eq!(cell.light, config.biome.lake_light);
                        assert_eq!(cell.toxicity, config.biome.lake_toxicity);
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn volcanic_vent_biome_covers_every_heat_source() {
        let config = test_config();
        for seed in 0..20u64 {
            let world = SimWorld::new(seed, &config);
            for &idx in &world.heat_sources {
                assert_eq!(
                    world.cells[idx].biome,
                    Biome::VolcanicVent,
                    "seed {seed}: heat source cell must read as VolcanicVent"
                );
            }
        }
    }

    #[test]
    fn feature_biome_placement_is_deterministic_for_a_given_seed() {
        let config = test_config();
        let a = SimWorld::new(11, &config);
        let b = SimWorld::new(11, &config);
        let biomes_a: Vec<Biome> = a.cells.iter().map(|c| c.biome).collect();
        let biomes_b: Vec<Biome> = b.cells.iter().map(|c| c.biome).collect();
        assert_eq!(biomes_a, biomes_b);
    }

    #[test]
    fn placeable_land_fraction_floor_holds_across_seeds() {
        let config = test_config();
        for seed in 0..30u64 {
            let world = SimWorld::new(seed, &config);
            let placeable = world
                .cells
                .iter()
                .filter(|c| c.terrain != TerrainKind::Sea && !c.is_peak)
                .count();
            let fraction = placeable as f32 / world.cells.len() as f32;
            assert!(
                fraction >= config.terrain.min_placeable_fraction,
                "seed {seed}: placeable fraction {fraction} below the configured floor"
            );
        }
    }

    #[test]
    fn peaks_only_occur_within_the_mountain_band() {
        let config = test_config();
        let world = SimWorld::new(7, &config);
        assert!(world
            .cells
            .iter()
            .all(|c| !c.is_peak || c.terrain == TerrainKind::Mountain));
    }

    #[test]
    fn same_seed_produces_identical_environment() {
        let config = test_config();
        let a = SimWorld::new(42, &config);
        let b = SimWorld::new(42, &config);

        assert_eq!(a.cells, b.cells);
    }

    /// Mirrors the double-buffering discipline `sim::step` uses: copy the
    /// snapshot into `scratch`, diffuse from `cells` into `scratch`, swap.
    /// Lets these tests drive `diffuse_environment` over several ticks
    /// without going through the full tick algorithm.
    fn diffuse_tick(world: &mut SimWorld, config: &SimConfig) {
        world.scratch.copy_from_slice(&world.cells);
        world.diffuse_environment(config);
        std::mem::swap(&mut world.cells, &mut world.scratch);
    }

    #[test]
    fn uniform_field_is_a_fixed_point_of_diffusion() {
        let config = test_config();
        let mut world = SimWorld::new(42, &config);
        for cell in &mut world.cells {
            cell.light = 0.5;
            cell.temperature = 0.5;
            cell.toxicity = 0.5;
        }
        let before = world.cells.clone();

        diffuse_tick(&mut world, &config);

        assert_eq!(world.cells, before, "a uniform field must not drift");
    }

    #[test]
    fn diffusion_smooths_a_single_perturbed_cell() {
        let config = test_config();
        let mut world = SimWorld::new(42, &config);
        for cell in &mut world.cells {
            cell.light = 0.2;
            cell.temperature = 0.2;
            cell.toxicity = 0.2;
        }
        let (cx, cy) = (world.width / 2, world.height / 2);
        {
            let hot = world.get_mut(cx, cy);
            hot.light = 0.9;
            hot.temperature = 0.9;
            hot.toxicity = 0.9;
        }
        let (nx, ny) = (cx + 1, cy);

        let mut previous_center = world.get(cx, cy).temperature;
        for tick in 1..=5 {
            diffuse_tick(&mut world, &config);
            let center = world.get(cx, cy).temperature;
            let neighbour = world.get(nx, ny).temperature;

            assert!(
                center < previous_center,
                "tick {tick}: center should keep cooling toward its neighbours, got {center} >= {previous_center}"
            );
            assert!(
                neighbour > 0.2,
                "tick {tick}: neighbour should warm above the baseline, got {neighbour}"
            );
            assert!((0.0..=1.0).contains(&center));
            assert!((0.0..=1.0).contains(&neighbour));
            previous_center = center;
        }
    }

    #[test]
    fn diffusion_keeps_scalars_in_unit_range_over_many_ticks() {
        let config = test_config();
        let mut world = SimWorld::new(42, &config);

        for _ in 0..50 {
            diffuse_tick(&mut world, &config);
        }

        for cell in &world.cells {
            assert!((0.0..=1.0).contains(&cell.light));
            assert!((0.0..=1.0).contains(&cell.temperature));
            assert!((0.0..=1.0).contains(&cell.toxicity));
        }
    }

    #[test]
    fn diffusion_does_not_touch_the_rng_and_stays_deterministic() {
        let config = test_config();
        let mut a = SimWorld::new(42, &config);
        let mut b = SimWorld::new(42, &config);

        for _ in 0..10 {
            diffuse_tick(&mut a, &config);
            diffuse_tick(&mut b, &config);
        }

        assert_eq!(
            a.cells, b.cells,
            "same seed must yield the same diffusion trajectory"
        );
        assert_eq!(
            a.rng_mut().random::<u64>(),
            b.rng_mut().random::<u64>(),
            "diffusion must not consume any RNG state"
        );
    }

    /// Task 085's key derived fact: a field linear in `(x, y)` is an exact
    /// fixed point of `diffuse_environment`'s Moore-blur for any *interior*
    /// cell, because its 8 symmetric neighbour offsets cancel. Edge/corner
    /// cells (this grid has real borders, no wrap-around — GDD §5.1) see
    /// only 3-5 neighbours, so the cancellation is inexact there; the
    /// resulting residual is a tiny, non-compounding boundary bias, not the
    /// genuine unbounded erosion a radial source field suffers without
    /// reinjection — hence the generous tolerance below, chosen well above
    /// the observed boundary residual and well below any plausible erosion
    /// signal. `sun_light_at`'s pure directional projection (deliberately
    /// *not* mountain-shaded here) is linear, so it needs no reinjection —
    /// unlike temperature's radial source falloff. Uses a real generated
    /// world (seed 42 has peaks on the map) to prove the property holds
    /// even with terrain irregularity nearby, not just on an empty grid.
    #[test]
    fn sun_direction_light_field_is_a_fixed_point_of_diffusion() {
        let config = test_config();
        let mut world = SimWorld::new(42, &config);
        assert!(
            world.cells.iter().any(|c| c.is_peak),
            "seed 42 should generate at least one peak for this test to be meaningful"
        );

        let sun_dir: (f32, f32) = (0.6, 0.8);
        let sun_bounds = (
            sun_dir.0.min(0.0) + sun_dir.1.min(0.0),
            sun_dir.0.max(0.0) + sun_dir.1.max(0.0),
        );
        for y in 0..world.height {
            for x in 0..world.width {
                let idx = world.index(x, y);
                world.cells[idx].light =
                    world.sun_light_at(x, y, sun_dir, sun_bounds, &config.environment);
            }
        }
        let light_before: Vec<f32> = world.cells.iter().map(|c| c.light).collect();

        diffuse_tick(&mut world, &config);

        let light_after: Vec<f32> = world.cells.iter().map(|c| c.light).collect();
        for (before, after) in light_before.iter().zip(&light_after) {
            assert!(
                (before - after).abs() < 1e-2,
                "a pure sun-direction field must not drift (beyond a tiny boundary residual) \
                 under diffusion: {before} -> {after}"
            );
        }
    }

    /// Task 085: `reinject_environment_sources`' pull-back must outweigh
    /// `diffuse_environment`'s own erosion, or the source structure loses to
    /// diffusion every tick and homogenizes anyway — the same class of
    /// invariant task 060 enforces for `residue_ambient_trickle` vs
    /// `residue_decay` (`sim.rs`'s `debug_assert` in `step`).
    #[test]
    fn reinjection_strength_stays_compatible_with_diffusion_rate() {
        let config = test_config();
        assert!(
            config.source.reinjection_strength > config.environment.diffusion_rate,
            "reinjection_strength ({}) must exceed diffusion_rate ({}), or heat sources erode \
             to the field mean over a long run",
            config.source.reinjection_strength,
            config.environment.diffusion_rate
        );
    }

    /// Task 122: same invariant as the heat-source one above, for the new
    /// Swamp-toxicity reinjection.
    #[test]
    fn toxic_reinjection_strength_stays_compatible_with_diffusion_rate() {
        let config = test_config();
        assert!(
            config.source.toxic_reinjection_strength > config.environment.diffusion_rate,
            "toxic_reinjection_strength ({}) must exceed diffusion_rate ({}), or toxic Swamp \
             cells erode to the field mean over a long run",
            config.source.toxic_reinjection_strength,
            config.environment.diffusion_rate
        );
    }

    /// Task 122: unlike `heat_sources`, `toxic_swamp_cells` had no ongoing
    /// reinjection before this task — `toxicity` was set once at
    /// generation (task 125) and only ever diffused afterward, exactly the
    /// erosion `reinject_environment_sources` already prevented for
    /// temperature (task 085). Runs the real double-buffered
    /// diffuse-then-reinject tick sequence `sim::step` uses (not just
    /// `diffuse_environment` alone) for many ticks and confirms a toxic
    /// Swamp cell's toxicity stays close to `swamp_toxicity_value` instead
    /// of eroding toward the grid's ambient mean.
    #[test]
    fn toxic_swamp_cells_hold_steady_under_repeated_diffusion() {
        let config = test_config();
        let mut world = SimWorld::new(42, &config);
        assert!(
            !world.toxic_swamp_cells.is_empty(),
            "seed 42 under test_config should produce at least one toxic Swamp cell"
        );
        for _ in 0..500 {
            world.scratch.copy_from_slice(&world.cells);
            world.diffuse_environment(&config);
            world.reinject_environment_sources(&config);
            std::mem::swap(&mut world.cells, &mut world.scratch);
        }
        let target = config.environment.swamp_toxicity_value;
        for &idx in &world.toxic_swamp_cells {
            let toxicity = world.cells[idx].toxicity;
            assert!(
                (toxicity - target).abs() < 0.1,
                "toxic Swamp cell {idx} eroded to {toxicity}, expected to stay near {target}"
            );
        }
    }

    #[test]
    fn active_tags_match_the_configured_pool() {
        let config = test_config();
        let world = SimWorld::new(42, &config);

        assert_eq!(
            world.active_tags.len(),
            config.tags.active_tags_early as usize
        );
        // Task 038: the subset is drawn from the whole global pool, not
        // necessarily contiguous — every tag must be a valid pool member
        // and no tag may repeat.
        for (i, tag) in world.active_tags.iter().enumerate() {
            assert!(
                (tag.0 as usize) < config.tags.global_tag_pool as usize,
                "tag {tag:?} is outside the global pool"
            );
            assert!(
                !world.active_tags[..i].contains(tag),
                "active tags must be unique, got a repeat of {tag:?}"
            );
        }
    }

    /// Task 037/038 link: the number of active tags a generated world gets
    /// must follow `worldgen::world_params`'s curve, not a fixed constant —
    /// this is a wiring test, not a game-logic one (the curve's own
    /// behavior is tested in `worldgen::tests`).
    #[test]
    fn active_tag_count_follows_the_difficulty_curve() {
        let config = test_config();

        for world_index in [0, 1, config.difficulty.ramp_worlds, 10] {
            let world = SimWorld::new_for_world(42, world_index, &config);
            let expected = crate::worldgen::world_params(world_index, &config).active_tag_count;
            assert_eq!(
                world.active_tags.len(),
                expected as usize,
                "world_index {world_index}: expected {expected} active tags"
            );
        }
    }

    #[test]
    fn drawn_species_tags_stay_within_bounds_and_the_active_pool() {
        let config = test_config();
        let mut world = SimWorld::new(42, &config);

        for _ in 0..20 {
            let tags = draw_species_tags(&mut world, &config);
            assert!(
                (config.tags.tags_per_species_min as usize
                    ..=config.tags.tags_per_species_max as usize)
                    .contains(&tags.len()),
                "expected 1..=3 tags, got {}",
                tags.len()
            );
            for tag in &tags {
                assert!(
                    (tag.0 as usize) < world.active_tags.len(),
                    "drawn slot {:?} is out of bounds for the active pool",
                    tag
                );
            }
        }
    }

    #[test]
    fn draw_species_tags_avoids_a_self_destructive_pair_when_a_safe_one_exists() {
        let mut config = test_config();
        config.tags.active_tags_early = 3;
        config.tags.tags_per_species_min = 2;
        config.tags.tags_per_species_max = 2;

        // Every pair touching tag 2 nets negative; only {0,1} is safe.
        let matrix = TagMatrix {
            size: 3,
            #[rustfmt::skip]
            values: vec![
                0,  0, -2,
                0,  0, -2,
                -2, -2, 0,
            ],
        };

        let mut successes = 0;
        let trials = 30;
        for seed in 0..trials {
            let mut world = SimWorld::new(seed, &config);
            world.matrix = matrix.clone();
            let tags = draw_species_tags(&mut world, &config);
            if net_self_interaction(&world.matrix, &tags) >= 0 {
                successes += 1;
            }
        }
        assert_eq!(
            successes, trials,
            "exhaustive search must find the safe pair every time, got {successes}/{trials}"
        );
    }

    /// Task 048's regression: a candidate whose tags net-*reinforce* each
    /// other must be rejected too, not just a net-draining one — that's the
    /// exact mechanism that let same-species clustering turn into unbounded
    /// growth (`sim::step`'s `crowding_penalty` is far smaller than a
    /// single matrix entry, so any positive self-interaction overwhelms it).
    #[test]
    fn draw_species_tags_avoids_a_self_reinforcing_pair_when_a_safe_one_exists() {
        let mut config = test_config();
        config.tags.active_tags_early = 3;
        config.tags.tags_per_species_min = 2;
        config.tags.tags_per_species_max = 2;

        // Every pair touching tag 2 nets strongly positive; only {0,1} is
        // neutral.
        let matrix = TagMatrix {
            size: 3,
            #[rustfmt::skip]
            values: vec![
                0, 0, 2,
                0, 0, 2,
                2, 2, 0,
            ],
        };

        let mut successes = 0;
        let trials = 30;
        for seed in 0..trials {
            let mut world = SimWorld::new(seed, &config);
            world.matrix = matrix.clone();
            let tags = draw_species_tags(&mut world, &config);
            if net_self_interaction(&world.matrix, &tags) == 0 {
                successes += 1;
            }
        }
        assert_eq!(
            successes, trials,
            "exhaustive search must find the neutral pair every time, got {successes}/{trials}"
        );
    }

    /// Task 088: when no combination at the rolled tag count is self-neutral
    /// (here, the only possible 2-tag pair nets -3), the search must
    /// gracefully degrade to a smaller tag set rather than accepting a
    /// nonzero "closest to zero" result — a 1-tag set is always safe since
    /// it has no pairs to net against itself.
    #[test]
    fn draw_species_tags_falls_back_to_a_smaller_tag_set_when_no_combination_is_neutral() {
        let mut config = test_config();
        config.tags.active_tags_early = 2;
        config.tags.tags_per_species_min = 2;
        config.tags.tags_per_species_max = 2;

        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, -1, 0],
        };

        let mut world = SimWorld::new(7, &config);
        world.matrix = matrix;
        let tags = draw_species_tags(&mut world, &config);

        assert_eq!(tags.len(), 1);
        assert_eq!(net_self_interaction(&world.matrix, &tags), 0);
    }

    /// Task 088's direct regression test: under the REAL default config
    /// (`active_tags_early = 5`, matching world 0's actual starting-species
    /// generation), `draw_species_tags` must always return an exactly
    /// self-neutral tag set. Combinatorial analysis found that ~15% of
    /// randomly-generated matrices have no zero-net 3-tag combination among
    /// the only `C(5,3) = 10` possible — the old random-retry search could
    /// silently fail there; exhaustive search must not.
    #[test]
    fn draw_species_tags_is_always_exactly_self_neutral_under_default_config() {
        let config = test_config();
        for seed in 0..300u64 {
            let mut world = SimWorld::new(seed, &config);
            for _ in 0..5 {
                let tags = draw_species_tags(&mut world, &config);
                assert_eq!(
                    net_self_interaction(&world.matrix, &tags),
                    0,
                    "seed {seed}: expected exact self-neutrality, got tags {tags:?}"
                );
            }
        }
    }

    #[test]
    fn same_seed_draws_identical_species_tags() {
        let config = test_config();
        let mut a = SimWorld::new(42, &config);
        let mut b = SimWorld::new(42, &config);

        for _ in 0..5 {
            assert_eq!(
                draw_species_tags(&mut a, &config),
                draw_species_tags(&mut b, &config)
            );
        }
    }

    /// Task 095: `draw_species_name` must draw from `world.rng_mut()`, never
    /// `rand::rng()` (TECH_DESIGN.md invariant 1) — same seed, same draw.
    #[test]
    fn draw_species_name_is_deterministic_for_the_same_seed() {
        let config = test_config();
        let mut a = SimWorld::new(42, &config);
        let mut b = SimWorld::new(42, &config);

        for _ in 0..5 {
            assert_eq!(draw_species_name(&mut a), draw_species_name(&mut b));
        }
    }

    fn tag_pairs(world: &SimWorld) -> impl Iterator<Item = (TagSlot, TagSlot)> + '_ {
        let n = world.active_tags.len() as u8;
        (0..n).flat_map(move |a| (0..n).map(move |b| (TagSlot(a), TagSlot(b))))
    }

    #[test]
    fn matrix_values_are_in_configured_range() {
        let config = test_config();
        let world = SimWorld::new(42, &config);
        let range = config.tags.effect_intensity_min..=config.tags.effect_intensity_max;

        for (a, b) in tag_pairs(&world) {
            assert!(range.contains(&world.matrix.get(a, b)));
        }
    }

    #[test]
    fn matrix_diagonal_is_always_zero() {
        let config = test_config();
        let world = SimWorld::new(42, &config);

        for i in 0..world.active_tags.len() as u8 {
            let slot = TagSlot(i);
            assert_eq!(world.matrix.get(slot, slot), 0);
        }
    }

    #[test]
    fn matrix_density_is_close_to_configured_target() {
        let config = test_config();
        let world = SimWorld::new(42, &config);
        let n = world.active_tags.len();
        let off_diagonal = n * n - n;

        let non_zero = tag_pairs(&world)
            .filter(|&(a, b)| a != b)
            .filter(|&(a, b)| world.matrix.get(a, b) != 0)
            .count();
        let density = non_zero as f32 / off_diagonal as f32;

        assert!(
            (density - config.tags.matrix_density).abs() < 0.25,
            "expected density near {}, got {density} ({non_zero}/{off_diagonal})",
            config.tags.matrix_density
        );
    }

    #[test]
    fn matrix_is_asymmetric_in_general() {
        let config = test_config();
        let world = SimWorld::new(42, &config);

        let asymmetric = tag_pairs(&world)
            .filter(|&(a, b)| a != b)
            .any(|(a, b)| world.matrix.get(a, b) != world.matrix.get(b, a));
        assert!(asymmetric, "matrix should not be forced symmetric");
    }

    #[test]
    fn matrix_guarantees_a_negative_three_cycle() {
        let config = test_config();
        let world = SimWorld::new(42, &config);
        let n = world.active_tags.len() as u8;

        let has_cycle = (0..n).any(|a| {
            (0..n).any(|b| {
                (0..n).any(|c| {
                    a != b
                        && b != c
                        && a != c
                        && world.matrix.get(TagSlot(a), TagSlot(b)) < 0
                        && world.matrix.get(TagSlot(b), TagSlot(c)) < 0
                        && world.matrix.get(TagSlot(c), TagSlot(a)) < 0
                })
            })
        });
        assert!(
            has_cycle,
            "matrix must contain at least one negative RPS cycle"
        );
    }

    #[test]
    fn same_seed_produces_identical_matrix() {
        let config = test_config();
        let a = SimWorld::new(42, &config);
        let b = SimWorld::new(42, &config);

        assert_eq!(a.matrix, b.matrix);
    }

    /// Task 124: `slope`/`water_distance` are purely additive derived
    /// fields — `classify_biomes` must never read them. Corrupting both
    /// after generation and re-running classification directly (rather
    /// than just comparing two independent generation runs, which
    /// wouldn't catch a hidden read of these fields since they'd hold the
    /// same values both times) is the actual guard against scope creep
    /// into task 125's territory. Compares `classify_biomes`'s *own* areal
    /// output run twice on the same cell state (not the full-pipeline
    /// biome, which includes `place_feature_biomes`' overrides that
    /// `classify_biomes` would itself strip on a second call — that's a
    /// pre-existing property of re-running Stage A/B alone, unrelated to
    /// this task, and not what this test is checking).
    #[test]
    fn water_distance_does_not_affect_biome_classification() {
        // Task 132 (resolved 2026-08-19): `Cell.slope` is now read directly
        // by `classify_biomes` (it moved earlier in the pipeline), so it's
        // no longer part of this guard — see
        // `classify_biomes_reads_the_persisted_slope_field` below for the
        // positive confirmation of that change. `water_distance` still
        // can't move earlier (its BFS needs `Biome::Lake`, task 132's
        // Option 3/4, not taken), so it stays local-proxy-only and this
        // guard still applies to it.
        let config = test_config();
        let params = world_params(0, &config);
        for seed in 0..10u64 {
            let mut world = SimWorld::new(seed, &config);
            world.classify_biomes(&config, &params);
            let areal_before: Vec<Biome> = world.cells.iter().map(|c| c.biome).collect();
            for cell in world.cells.iter_mut() {
                cell.water_distance = 999.0;
            }
            world.classify_biomes(&config, &params);
            let areal_after: Vec<Biome> = world.cells.iter().map(|c| c.biome).collect();
            assert_eq!(
                areal_before, areal_after,
                "seed {seed}: biome classification changed"
            );
        }
    }

    /// Task 132: the positive counterpart of the guard above — confirms
    /// `classify_biomes` actually reads the persisted `Cell.slope` now
    /// (not a silently-never-exercised code path). A `Hill` cell forced to
    /// maximum slope should read as BareRock (`bare_rock_score`'s
    /// slope-raised effective light ceiling makes even moderate light read
    /// as BareRock at a saturated slope).
    #[test]
    fn classify_biomes_reads_the_persisted_slope_field() {
        let config = test_config();
        let params = world_params(0, &config);
        let mut world = SimWorld::new(7, &config);
        for cell in world.cells.iter_mut() {
            if cell.terrain == TerrainKind::Hill {
                cell.slope = 999.0;
            }
        }
        world.classify_biomes(&config, &params);
        assert!(
            world
                .cells
                .iter()
                .filter(|c| c.terrain == TerrainKind::Hill)
                .all(|c| c.biome == Biome::BareRock),
            "every Hill cell forced to maximum slope should read as BareRock"
        );
    }

    /// Task 131: the equivalent positive confirmation for
    /// `Cell.soil_moisture` — `classify_biomes` reads it directly via
    /// `swamp_score`, not a locally re-derived proxy. A cell forced to
    /// `0.0` soil moisture should lose Swamp eligibility everywhere; one
    /// forced to `1.0` should make Swamp win the arg-max wherever nothing
    /// else scores even higher (`TerrainKind::Plain` only, matching
    /// `swamp_score`'s own scope).
    #[test]
    fn classify_biomes_reads_the_persisted_soil_moisture_field() {
        let config = test_config();
        let params = world_params(0, &config);

        let mut dry_world = SimWorld::new(11, &config);
        for cell in dry_world.cells.iter_mut() {
            cell.soil_moisture = 0.0;
        }
        dry_world.classify_biomes(&config, &params);
        assert!(
            !dry_world.cells.iter().any(|c| c.biome == Biome::Swamp),
            "zero soil moisture everywhere should make Swamp unreachable"
        );

        let mut wet_world = SimWorld::new(11, &config);
        for cell in wet_world.cells.iter_mut() {
            cell.soil_moisture = 1.0;
        }
        wet_world.classify_biomes(&config, &params);
        assert!(
            wet_world
                .cells
                .iter()
                .any(|c| c.terrain == TerrainKind::Plain && c.biome == Biome::Swamp),
            "saturated soil moisture everywhere should make Swamp reachable somewhere on Plain"
        );
    }

    #[test]
    fn slope_stays_within_the_normalized_unit_range() {
        let config = test_config();
        for seed in 0..10u64 {
            let world = SimWorld::new(seed, &config);
            for cell in &world.cells {
                assert!(
                    (0.0..=1.0).contains(&cell.slope),
                    "seed {seed}: slope {} outside [0, 1]",
                    cell.slope
                );
            }
        }
    }

    #[test]
    fn water_distance_is_zero_on_every_water_cell_and_finite_elsewhere() {
        let config = test_config();
        for seed in 0..10u64 {
            let world = SimWorld::new(seed, &config);
            for cell in &world.cells {
                assert!(
                    cell.water_distance.is_finite(),
                    "seed {seed}: water_distance should always be finite on a world with sea"
                );
                if matches!(
                    cell.biome,
                    Biome::DeepWater | Biome::ShallowWater | Biome::Lake
                ) {
                    assert_eq!(
                        cell.water_distance, 0.0,
                        "seed {seed}: a water cell itself must have water_distance 0"
                    );
                }
            }
        }
    }

    #[test]
    fn water_distance_is_deterministic_for_a_given_seed() {
        let config = test_config();
        let a = SimWorld::new(11, &config);
        let b = SimWorld::new(11, &config);
        let distances_a: Vec<f32> = a.cells.iter().map(|c| c.water_distance).collect();
        let distances_b: Vec<f32> = b.cells.iter().map(|c| c.water_distance).collect();
        assert_eq!(distances_a, distances_b);
    }

    /// Task 125 (§12.4): the toxicity-imposition pass is load-bearing, not
    /// optional flavor — it's the *only* generation-time toxicity source
    /// for Swamp cells (task 113 removed the old standalone
    /// `place_toxic_zone` rectangle), so Swamp's own toxic sub-region must
    /// actually produce nonzero toxicity somewhere across a run of seeds.
    /// A single seed's Swamp region could in principle be
    /// too small/oddly-shaped for the noise mask to select any of it (same
    /// keep-best-seen spirit as other placement code, not a hard
    /// per-seed guarantee) — checked as a comfortable majority across a
    /// generous sample instead of every seed.
    #[test]
    fn some_swamp_cells_are_toxic_across_seeds() {
        let config = test_config();
        let n_seeds = 60u64;
        let mut seeds_with_toxic_swamp = 0u64;
        for seed in 0..n_seeds {
            let world = SimWorld::new(seed, &config);
            if world
                .cells
                .iter()
                .any(|c| c.biome == Biome::Swamp && c.toxicity > 0.0)
            {
                seeds_with_toxic_swamp += 1;
            }
        }
        assert!(
            seeds_with_toxic_swamp >= n_seeds * 3 / 4,
            "only {seeds_with_toxic_swamp}/{n_seeds} seeds produced a toxic Swamp cell"
        );
    }

    /// Task 133: GDD §9's "larger toxic zones" difficulty axis, revived
    /// after task 113 removed the old sized `toxic_zone` rectangle it used
    /// to drive — `WorldParams::swamp_toxicity_min` should make later
    /// worlds' Swamp read as toxic more often, not just have a
    /// differently-scaled `WorldParams` field nobody reads end to end.
    /// Aggregate fraction across seeds, not a per-seed hard assert, same
    /// idiom as `some_swamp_cells_are_toxic_across_seeds`.
    #[test]
    fn later_worlds_have_a_larger_toxic_fraction_of_swamp() {
        let config = test_config();
        let n_seeds = 20u64;
        let toxic_fraction = |world_index: u32| -> f32 {
            let mut swamp = 0u64;
            let mut toxic = 0u64;
            for seed in 0..n_seeds {
                let world = SimWorld::new_for_world(seed, world_index, &config);
                for cell in &world.cells {
                    if cell.biome == Biome::Swamp {
                        swamp += 1;
                        if cell.toxicity > 0.0 {
                            toxic += 1;
                        }
                    }
                }
            }
            toxic as f32 / swamp.max(1) as f32
        };
        let early = toxic_fraction(0);
        let late = toxic_fraction(config.difficulty.ramp_worlds);
        assert!(
            late > early,
            "expected a later world's Swamp to read toxic more often: early {early}, late {late}"
        );
    }

    /// Task 126: `rainfall` is purely additive — corrupting it and
    /// re-running `classify_biomes` (same guard shape task 124/125 already
    /// use for `slope`/`water_distance`) confirms nothing downstream reads
    /// it yet.
    #[test]
    fn rainfall_does_not_affect_biome_classification() {
        let config = test_config();
        let params = world_params(0, &config);
        for seed in 0..10u64 {
            let mut world = SimWorld::new(seed, &config);
            world.classify_biomes(&config, &params);
            let areal_before: Vec<Biome> = world.cells.iter().map(|c| c.biome).collect();
            for cell in world.cells.iter_mut() {
                cell.rainfall = 999.0;
            }
            world.classify_biomes(&config, &params);
            let areal_after: Vec<Biome> = world.cells.iter().map(|c| c.biome).collect();
            assert_eq!(
                areal_before, areal_after,
                "seed {seed}: biome classification changed"
            );
        }
    }

    #[test]
    fn rainfall_stays_within_the_unit_range() {
        let config = test_config();
        for seed in 0..10u64 {
            let world = SimWorld::new(seed, &config);
            for cell in &world.cells {
                assert!(
                    (0.0..=1.0).contains(&cell.rainfall),
                    "seed {seed}: rainfall {} outside [0, 1]",
                    cell.rainfall
                );
            }
        }
    }

    #[test]
    fn rainfall_is_deterministic_for_a_given_seed() {
        let config = test_config();
        let a = SimWorld::new(11, &config);
        let b = SimWorld::new(11, &config);
        let rainfall_a: Vec<f32> = a.cells.iter().map(|c| c.rainfall).collect();
        let rainfall_b: Vec<f32> = b.cells.iter().map(|c| c.rainfall).collect();
        assert_eq!(rainfall_a, rainfall_b);
    }

    /// Task 126 (§18.5's credibility check, "precipitazione minore oltre le
    /// montagne"): rainfall in a local ring around the tallest peak should
    /// read lower on the leeward side than the windward side. Checked as an
    /// aggregate mean across a sample of seeds, not per-seed — an
    /// individual seed's tallest peak can land near a grid edge or have an
    /// unrepresentative local ring (e.g. mostly ocean on one side for
    /// reasons unrelated to rain shadow), which is exactly the kind of
    /// single-sample noise an aggregate mean is meant to average out; the
    /// physical effect only needs to hold on balance, the same "usually"/
    /// "rarely" statistical spirit `tests/balance.rs` already uses.
    #[test]
    fn leeward_side_of_the_tallest_peak_reads_drier_than_windward_on_average() {
        let config = test_config();
        let local_radius = 25.0f32;
        let mut diffs: Vec<f32> = Vec::new();
        for seed in 0..30u64 {
            let world = SimWorld::new(seed, &config);
            let params = world_params(0, &config);
            let wind = world.wind_vector(&params);
            let wind_len = (wind.0 * wind.0 + wind.1 * wind.1).sqrt();
            if wind_len <= f32::EPSILON {
                continue;
            }
            let wind_dir = (wind.0 / wind_len, wind.1 / wind_len);

            let Some((px, py)) = world
                .cells
                .iter()
                .enumerate()
                .filter(|(_, c)| c.is_peak)
                .max_by(|a, b| a.1.elevation.total_cmp(&b.1.elevation))
                .map(|(idx, _)| (idx % world.width, idx / world.width))
            else {
                continue;
            };

            let mut windward_sum = 0.0;
            let mut windward_n = 0u32;
            let mut leeward_sum = 0.0;
            let mut leeward_n = 0u32;
            for y in 0..world.height {
                for x in 0..world.width {
                    let rel = (x as f32 - px as f32, y as f32 - py as f32);
                    let dist = (rel.0 * rel.0 + rel.1 * rel.1).sqrt();
                    if dist > local_radius || dist < 3.0 {
                        continue;
                    }
                    let along_wind = rel.0 * wind_dir.0 + rel.1 * wind_dir.1;
                    let r = world.get(x, y).rainfall;
                    if along_wind < 0.0 {
                        windward_sum += r;
                        windward_n += 1;
                    } else {
                        leeward_sum += r;
                        leeward_n += 1;
                    }
                }
            }
            if windward_n == 0 || leeward_n == 0 {
                continue;
            }
            diffs.push(windward_sum / windward_n as f32 - leeward_sum / leeward_n as f32);
        }
        assert!(
            diffs.len() >= 20,
            "too few seeds produced a usable tallest-peak sample: {}",
            diffs.len()
        );
        let mean_diff: f32 = diffs.iter().sum::<f32>() / diffs.len() as f32;
        assert!(
            mean_diff > 0.02,
            "windward rainfall isn't measurably higher than leeward on average: mean diff {mean_diff:.4}"
        );
    }

    /// Sizes of the connected components of the `is_river` subgraph, where
    /// two river cells are adjacent iff one's `flow_direction` is the
    /// other — i.e. the actual principal-river path lengths a player would
    /// see, not just a raw `is_river` cell count (which would also count
    /// disconnected single-cell tributary starts as "rivers"). Sorted
    /// largest-first.
    fn river_component_sizes(world: &SimWorld) -> Vec<usize> {
        let is_river: Vec<bool> = world.cells.iter().map(|c| c.is_river).collect();
        let mut visited = vec![false; world.cells.len()];
        let mut sizes = Vec::new();
        for start in 0..world.cells.len() {
            if !is_river[start] || visited[start] {
                continue;
            }
            let mut stack = vec![start];
            visited[start] = true;
            let mut count = 0;
            while let Some(cur) = stack.pop() {
                count += 1;
                if let Some(target) = world.cells[cur].flow_direction {
                    if is_river[target] && !visited[target] {
                        visited[target] = true;
                        stack.push(target);
                    }
                }
                let (x, y) = (cur % world.width, cur / world.width);
                for n in world.moore_neighbours(x, y) {
                    if is_river[n] && world.cells[n].flow_direction == Some(cur) && !visited[n] {
                        visited[n] = true;
                        stack.push(n);
                    }
                }
            }
            sizes.push(count);
        }
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        sizes
    }

    #[test]
    fn hydrology_is_deterministic_for_a_given_seed() {
        let config = test_config();
        let a = SimWorld::new(11, &config);
        let b = SimWorld::new(11, &config);
        let accum_a: Vec<f32> = a.cells.iter().map(|c| c.flow_accumulation).collect();
        let accum_b: Vec<f32> = b.cells.iter().map(|c| c.flow_accumulation).collect();
        assert_eq!(accum_a, accum_b);
        let dir_a: Vec<Option<usize>> = a.cells.iter().map(|c| c.flow_direction).collect();
        let dir_b: Vec<Option<usize>> = b.cells.iter().map(|c| c.flow_direction).collect();
        assert_eq!(dir_a, dir_b);
    }

    #[test]
    fn flow_accumulation_is_never_negative_and_sea_cells_never_route_further() {
        let config = test_config();
        for seed in 0..10u64 {
            let world = SimWorld::new(seed, &config);
            for cell in &world.cells {
                assert!(cell.flow_accumulation >= 0.0);
                if cell.terrain == TerrainKind::Sea {
                    assert_eq!(cell.flow_direction, None, "Sea must always be a sink");
                }
            }
        }
    }

    /// A routing cycle would be a determinism/correctness bug, not just an
    /// aesthetic one (an infinite loop for any future code that chases
    /// `flow_direction` chains, e.g. a river-rendering pass). Chases every
    /// cell's chain up to a generous step bound and fails if it doesn't
    /// reach a sink (`flow_direction == None`) — the strict-total-order
    /// argument in `compute_hydrology`'s doc comment guarantees this can
    /// never happen, this test is the empirical check that the argument
    /// actually holds in the implementation.
    #[test]
    fn flow_direction_chains_never_cycle() {
        let config = test_config();
        for seed in 0..10u64 {
            let world = SimWorld::new(seed, &config);
            for start in 0..world.cells.len() {
                let mut cur = start;
                let mut steps = 0;
                while let Some(next) = world.cells[cur].flow_direction {
                    cur = next;
                    steps += 1;
                    assert!(
                        steps <= world.cells.len(),
                        "seed {seed}: flow_direction chain from cell {start} did not reach a sink within {} steps — likely cycle",
                        world.cells.len()
                    );
                }
            }
        }
    }

    /// Task 127 (§10.3's credibility bounds): "roughly 1-3 principal rivers
    /// of 20-40 cells of path length," checked as a statistical sample
    /// across seeds — not a hard per-seed assert, per the task's own
    /// caution that a fixed bound could make some seeds unsatisfiable.
    #[test]
    fn rivers_usually_form_a_small_number_of_plausible_length_principal_paths() {
        let config = test_config();
        let mut principal_in_range = 0u32;
        let mut component_count_in_range = 0u32;
        let n_seeds = 30u64;
        for seed in 0..n_seeds {
            let world = SimWorld::new(seed, &config);
            let sizes = river_component_sizes(&world);
            let principal = sizes.first().copied().unwrap_or(0);
            let significant = sizes.iter().filter(|&&s| s >= 5).count();
            if (10..=45).contains(&principal) {
                principal_in_range += 1;
            }
            if (1..=4).contains(&significant) {
                component_count_in_range += 1;
            }
        }
        assert!(
            principal_in_range >= n_seeds as u32 * 2 / 3,
            "only {principal_in_range}/{n_seeds} seeds had a principal river path in a plausible length range"
        );
        assert!(
            component_count_in_range >= n_seeds as u32 * 2 / 3,
            "only {component_count_in_range}/{n_seeds} seeds had a plausible number of significant (>=5 cell) river components"
        );
    }

    /// Task 127's explicit acceptance criterion: a hand-constructed plateau
    /// (several cells at identical elevation) must route the same way on
    /// every run, not depend on iteration order breaking the tie — and
    /// (going further than the literal criterion) must actually reach the
    /// sea, not get trapped by the flat.
    ///
    /// Self-contained construction, entirely within a 20x20 corner block
    /// (not exposed to whatever real terrain this seed happens to generate
    /// elsewhere on the map, which would make the "reaches sea" half
    /// unpredictable): every cell in the block gets a strict
    /// distance-from-exit gradient elevation, *except* a small plateau
    /// sub-rectangle forced to one shared elevation (the gradient value at
    /// the plateau's farthest corner from the exit) — high enough that
    /// every real gradient cell just outside the plateau, on its
    /// exit-facing side, is still strictly lower, so a valid downhill exit
    /// always exists immediately adjacent to the flat.
    #[test]
    fn depression_filling_and_routing_are_deterministic_on_a_flat_plateau() {
        let config = test_config();
        let mut world_a = SimWorld::new(3, &config);
        let mut world_b = SimWorld::new(3, &config);

        let build = |world: &mut SimWorld| {
            let exit = (0i32, 0i32);
            for y in 0..20 {
                for x in 0..20 {
                    let dist = (x as i32 - exit.0).abs().max((y as i32 - exit.1).abs()) as f32;
                    let idx = world.index(x, y);
                    world.cells[idx].elevation = 0.02 * dist;
                    world.cells[idx].terrain = TerrainKind::Plain;
                }
            }
            let exit_idx = world.index(0, 0);
            world.cells[exit_idx].elevation = 0.0;
            world.cells[exit_idx].terrain = TerrainKind::Sea;

            // Plateau: (10..14, 10..14), all forced to the gradient value
            // its farthest corner (13, 13) would naturally have — several
            // cells sharing one identical elevation despite differing
            // position, the exact scenario the acceptance criterion calls
            // out.
            let plateau_elev = 0.02 * 13.0;
            for y in 10..14 {
                for x in 10..14 {
                    let idx = world.index(x, y);
                    world.cells[idx].elevation = plateau_elev;
                }
            }
        };
        build(&mut world_a);
        build(&mut world_b);
        let filled_a = world_a.fill_depressions();
        let filled_b = world_b.fill_depressions();
        world_a.compute_hydrology(&config, &filled_a);
        world_b.compute_hydrology(&config, &filled_b);

        let dir_a: Vec<Option<usize>> = world_a.cells.iter().map(|c| c.flow_direction).collect();
        let dir_b: Vec<Option<usize>> = world_b.cells.iter().map(|c| c.flow_direction).collect();
        assert_eq!(
            dir_a, dir_b,
            "identical plateau setups must route identically"
        );
        for y in 10..14 {
            for x in 10..14 {
                let idx = world_a.index(x, y);
                let mut cur = idx;
                let mut steps = 0;
                while let Some(next) = world_a.cells[cur].flow_direction {
                    cur = next;
                    steps += 1;
                    assert!(
                        steps <= world_a.cells.len(),
                        "plateau cell ({x},{y}) never reached a sink"
                    );
                }
                assert_eq!(
                    world_a.cells[cur].terrain,
                    TerrainKind::Sea,
                    "plateau cell ({x},{y}) drained to a non-Sea sink"
                );
            }
        }
    }
}
