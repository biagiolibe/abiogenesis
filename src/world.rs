// Species/organism fields are read by later tasks (004+); the domain types
// are complete and correct before the tick algorithm consumes them.
#![allow(dead_code)]

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::RngExt;
use rand::SeedableRng;

use crate::config::{EnvironmentConfig, SimConfig, SourceConfig, TagConfig, TerrainConfig};
use crate::worldgen::{world_params, WorldParams};

/// How a species derives energy (GDD §5.4). Only `Photolithic` is active in
/// Phase 0; the other variants exist now to avoid a refactor in Phase 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metabolism {
    Photolithic,
    Predator,
    Decomposer,
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
    /// The era this organism was born in (task 083). Reproduction requires
    /// `born_era < world.era` — an organism must survive into a later era
    /// than its own birth before it can reproduce.
    pub born_era: u32,
}

/// The toxic zone's geometry (GDD §5.2): every cell within the rectangle
/// `[x0, x0+width) x [y0, y0+height)` is in the zone. Set once at world
/// construction (task 066: position and size now vary per world, chosen to
/// overlap enough placeable terrain — no longer always anchored to the
/// grid's bottom-right corner) and never touched again — unlike the
/// per-cell `toxicity` scalar, which `diffuse_environment` blends toward
/// neighbours every tick and so drifts away from the zone's actual shape
/// over time (task 047: `objectives.rs`'s `SurviveIn` zone check reads
/// this, not `toxicity`, precisely because the scalar isn't a reliable
/// proxy for "in the zone" once diffusion has run for a while).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToxicZoneBounds {
    pub x0: usize,
    pub y0: usize,
    pub width: usize,
    pub height: usize,
}

impl ToxicZoneBounds {
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x0 && x < self.x0 + self.width && y >= self.y0 && y < self.y0 + self.height
    }
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
    pub era: u32,
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
    /// The toxic zone's fixed bounds (task 047) — see `ToxicZoneBounds`'s
    /// own docs for why this exists separately from the diffusing
    /// `Cell::toxicity` scalar.
    pub toxic_zone: ToxicZoneBounds,
    /// Cell indices of this world's heat sources (task 085), placed once at
    /// generation time. `reinject_environment_sources` reads this every
    /// tick to pull those cells' temperature back toward
    /// `EnvironmentConfig::source_temperature`, countering
    /// `diffuse_environment`'s erosion — the same reason `toxic_zone` is
    /// kept as a stable reference alongside the diffusing scalar it seeded.
    pub heat_sources: Vec<usize>,
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
    /// `species`. Can't be reconstructed after the fact, so every
    /// `species.push` call site goes through `push_species` instead, which
    /// records both in the same step.
    pub species_origin_era: Vec<u32>,
    rng: StdRng,
    /// Write-side double buffer for the tick (TECH_DESIGN.md §6). `pub(crate)`
    /// so `sim::step` can read/write it directly without a cell-by-cell API.
    pub(crate) scratch: Vec<Cell>,
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
            cells,
            species: Vec::new(),
            tick: 0,
            era: 0,
            seed,
            active_tags,
            matrix,
            conditional_tags,
            toxic_zone: ToxicZoneBounds::default(),
            heat_sources: Vec::new(),
            sea_distance: Vec::new(),
            ever_populated: false,
            wild_species: Vec::new(),
            terrain_occupancy: Vec::new(),
            selection_pressure: Vec::new(),
            species_origin_era: Vec::new(),
            rng,
        };
        world.generate_terrain(config);
        world.apply_environment_sources(config, &params);
        world.place_toxic_zone(config, &params);
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
    /// `place_toxic_zone`, which needs real terrain to search against.
    fn generate_terrain(&mut self, config: &SimConfig) {
        let terrain_cfg = &config.terrain;
        let mut rng = StdRng::seed_from_u64(self.seed ^ TERRAIN_SEED_OFFSET);
        let cell_count = self.width * self.height;

        let mut best: Option<(Vec<TerrainKind>, Vec<bool>, f32)> = None;
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
                self.write_terrain(&kinds, &peaks);
                return;
            }
            if best
                .as_ref()
                .is_none_or(|&(_, _, best_fraction)| fraction > best_fraction)
            {
                best = Some((kinds, peaks, fraction));
            }
        }
        let (kinds, peaks, _) = best.expect("the loop above runs at least once");
        self.write_terrain(&kinds, &peaks);
    }

    fn write_terrain(&mut self, kinds: &[TerrainKind], peaks: &[bool]) {
        for (idx, cell) in self.cells.iter_mut().enumerate() {
            cell.terrain = kinds[idx];
            cell.is_peak = peaks[idx];
        }
    }

    /// Places the toxic zone (task 066): searches random rectangle
    /// positions of `params`'s size (its own derived RNG stream, via
    /// `TOXIC_ZONE_SEED_OFFSET`, never `self.rng`) for one overlapping
    /// enough placeable terrain (`TerrainConfig::min_toxic_zone_placeable_fraction`),
    /// bounded by `max_toxic_zone_placement_attempts` and keeping the best
    /// position seen otherwise — the same guarantee `generate_terrain` makes
    /// for the grid as a whole, applied to the zone's own footprint, so
    /// `objectives.rs`'s `SurviveIn` can never land on an unwinnable
    /// all-sea/all-peak zone. Must run after `generate_terrain`.
    fn place_toxic_zone(&mut self, config: &SimConfig, params: &WorldParams) {
        let env = &config.environment;
        let width = (params.toxic_zone_width as usize).min(self.width);
        let height = (params.toxic_zone_height as usize).min(self.height);
        if width == 0 || height == 0 {
            self.toxic_zone = ToxicZoneBounds::default();
            return;
        }
        let terrain_cfg = &config.terrain;
        let mut rng = StdRng::seed_from_u64(self.seed ^ TOXIC_ZONE_SEED_OFFSET);
        let max_x0 = self.width - width;
        let max_y0 = self.height - height;

        let mut best: Option<(usize, usize, f32)> = None;
        for _ in 0..terrain_cfg.max_toxic_zone_placement_attempts.max(1) {
            let x0 = rng.random_range(0..=max_x0);
            let y0 = rng.random_range(0..=max_y0);
            let fraction = self.placeable_fraction_in(x0, y0, width, height);
            if fraction >= terrain_cfg.min_toxic_zone_placeable_fraction {
                self.set_toxic_zone(x0, y0, width, height, env.toxic_zone_value);
                return;
            }
            if best.is_none_or(|(_, _, best_fraction)| fraction > best_fraction) {
                best = Some((x0, y0, fraction));
            }
        }
        let (x0, y0, _) = best.expect("the loop above runs at least once");
        self.set_toxic_zone(x0, y0, width, height, env.toxic_zone_value);
    }

    /// Fraction of `[x0, x0+width) x [y0, y0+height)` that's placeable
    /// terrain — the metric `place_toxic_zone` searches against.
    fn placeable_fraction_in(&self, x0: usize, y0: usize, width: usize, height: usize) -> f32 {
        let mut placeable = 0usize;
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                let cell = self.get(x, y);
                if is_placeable_kind(cell.terrain, cell.is_peak) {
                    placeable += 1;
                }
            }
        }
        placeable as f32 / (width * height) as f32
    }

    fn set_toxic_zone(&mut self, x0: usize, y0: usize, width: usize, height: usize, value: f32) {
        self.toxic_zone = ToxicZoneBounds {
            x0,
            y0,
            width,
            height,
        };
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                let idx = self.index(x, y);
                self.cells[idx].toxicity = value;
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
    /// draw for a given seed, same discipline as `generate_terrain`/
    /// `place_toxic_zone`. Must run after `generate_terrain` (heat source
    /// placement, and the sea-distance field, both need real terrain to
    /// search against) and before `place_toxic_zone` (unchanged ordering
    /// requirement). Toxicity is not set here (task 066: `place_toxic_zone`
    /// owns it).
    fn apply_environment_sources(&mut self, config: &SimConfig, params: &WorldParams) {
        let env = &config.environment;
        let source_cfg = &config.source;

        let mut temp_rng = StdRng::seed_from_u64(self.seed ^ TEMPERATURE_SOURCE_SEED_OFFSET);
        let wind_angle = temp_rng.random_range(0.0..TAU);
        let wind = (
            wind_angle.cos() * params.wind_strength,
            wind_angle.sin() * params.wind_strength,
        );
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

    /// Grid distance from every cell to its nearest `TerrainKind::Sea` cell
    /// (task 086 playtest follow-up), via multi-source BFS seeded from every
    /// `Sea` cell at once — `O(cells)`, not `O(cells * sea_cells)` a
    /// point-by-point nearest-search would cost with a large sea. Distance
    /// is in Moore-neighbour steps (matching `diffuse_environment`'s own
    /// 8-connectivity), so it composes directly with
    /// `SourceConfig::sea_coolant_radius`, a cell count like
    /// `heat_source_radius`.
    fn sea_distance_field(&self) -> Vec<f32> {
        let mut distance = vec![f32::INFINITY; self.cells.len()];
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        for (idx, cell) in self.cells.iter().enumerate() {
            if cell.terrain == TerrainKind::Sea {
                distance[idx] = 0.0;
                queue.push_back(idx);
            }
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

    /// Places `params.heat_source_count` point sources via bounded-retry
    /// generation against `is_placeable` (no sources on `Sea`/peaks),
    /// mirroring `place_toxic_zone`'s attempt-loop/keep-best-seen pattern
    /// (`world.rs:359-387`): each source retries up to
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

    /// Counters `diffuse_environment`'s erosion of the two standing
    /// temperature features it doesn't otherwise preserve (task 085):
    /// `heat_sources` are pulled back toward `source_temperature`, and every
    /// cell within `sea_coolant_radius` of the sea gets a pull toward
    /// `sea_coolant_value` weighted by `self.sea_distance` (task 086: widened
    /// from a single Moore-ring nudge to the same radius-based falloff
    /// `apply_environment_sources` bakes in at generation, so the coastal
    /// band both starts and stays visible instead of only the literal
    /// coastline). Deliberately a separate method from `diffuse_environment`,
    /// called right after it from `sim::step` and operating on
    /// `self.scratch` (the write side diffusion just populated) — folding
    /// this into the blend loop would perturb `diffuse_environment`'s own
    /// fixed-point tests, which build a hand-crafted uniform field and
    /// expect diffusion alone to leave it untouched.
    pub fn reinject_environment_sources(&mut self, config: &SimConfig) {
        let source_cfg = &config.source;
        debug_assert!(
            source_cfg.reinjection_strength > config.environment.diffusion_rate,
            "reinjection_strength must exceed diffusion_rate, or diffusion erodes the source \
             faster than reinjection restores it"
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
/// `SimWorld::rng` (tag/species/reproduction draws) and from
/// `TOXIC_ZONE_SEED_OFFSET` below — an arbitrary constant, chosen only to
/// not collide with the other salt.
const TERRAIN_SEED_OFFSET: u64 = 0x9E37_79B9_7F4A_7C15;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for the toxic zone's placement
/// search (task 066) — a different constant so the two derived streams
/// don't start in lockstep.
const TOXIC_ZONE_SEED_OFFSET: u64 = 0xC2B2_AE3D_27D4_EB4F;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for heat source placement and the
/// per-world wind direction draw (task 085) — a different constant so this
/// stream doesn't start in lockstep with the others.
const TEMPERATURE_SOURCE_SEED_OFFSET: u64 = 0x1656_67B1_9E37_79F9;

/// Same purpose as `TERRAIN_SEED_OFFSET`, for the per-world sun direction
/// draw (task 085). Kept independent from `TEMPERATURE_SOURCE_SEED_OFFSET`
/// rather than sharing its stream, so hot/bright regions don't always
/// correlate spatially — an open question the design doc left to scoping.
const SUN_DIRECTION_SEED_OFFSET: u64 = 0x9E97_79B9_7F4A_1656;

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

    /// Task 066: the zone's position is no longer fixed to the grid's
    /// bottom-right corner, so this checks the general invariant instead —
    /// every cell's toxicity matches `toxic_zone_value` exactly where
    /// `ToxicZoneBounds::contains` says it should, and `0.0` everywhere else.
    #[test]
    fn toxic_zone_matches_its_own_bounds() {
        let config = test_config();
        let world = SimWorld::new(42, &config);
        let env = &config.environment;

        assert!(
            world.toxic_zone.width > 0 && world.toxic_zone.height > 0,
            "world_index 0's default config always has a non-empty toxic zone"
        );
        for y in 0..world.height {
            for x in 0..world.width {
                let expected = if world.toxic_zone.contains(x, y) {
                    env.toxic_zone_value
                } else {
                    0.0
                };
                assert_eq!(world.get(x, y).toxicity, expected, "cell ({x}, {y})");
            }
        }
    }

    /// Task 066: the toxic zone's position now depends on generated
    /// terrain, but its footprint must still always contain enough
    /// placeable land for `SurviveIn` to remain satisfiable — checked
    /// across a sample of seeds, not just one.
    #[test]
    fn toxic_zone_always_overlaps_enough_placeable_land() {
        let config = test_config();
        for seed in 0..30u64 {
            let world = SimWorld::new(seed, &config);
            let zone = world.toxic_zone;
            if zone.width == 0 || zone.height == 0 {
                continue;
            }
            let placeable = (zone.y0..zone.y0 + zone.height)
                .flat_map(|y| (zone.x0..zone.x0 + zone.width).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    let cell = world.get(x, y);
                    cell.terrain != TerrainKind::Sea && !cell.is_peak
                })
                .count();
            let fraction = placeable as f32 / (zone.width * zone.height) as f32;
            assert!(
                fraction >= config.terrain.min_toxic_zone_placeable_fraction,
                "seed {seed}: toxic zone placeable fraction {fraction} below the configured floor"
            );
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
}
