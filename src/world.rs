// Species/organism fields are read by later tasks (004+); the domain types
// are complete and correct before the tick algorithm consumes them.
#![allow(dead_code)]

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::RngExt;
use rand::SeedableRng;

use crate::config::{SimConfig, TagConfig, TerrainConfig};
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
}

/// A species' genome (GDD §5.3): metabolism and environmental range are
/// player-readable, tags are opaque and drive matrix interactions.
#[derive(Debug, Clone, PartialEq)]
pub struct Species {
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
    /// The toxic zone's fixed bounds (task 047) — see `ToxicZoneBounds`'s
    /// own docs for why this exists separately from the diffusing
    /// `Cell::toxicity` scalar.
    pub toxic_zone: ToxicZoneBounds,
    /// Whether any organism has ever occupied a cell in this world (task
    /// 050): set by `sim::step` the first time its population scan finds
    /// one. Worlds start with nothing placed (the player seeds them via
    /// `Seed`, GDD §6) — `objectives::is_total_extinction` reads this so a
    /// world that hasn't been seeded yet doesn't fail on its very first
    /// evaluated tick.
    pub ever_populated: bool,
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
            toxic_zone: ToxicZoneBounds::default(),
            ever_populated: false,
            rng,
        };
        world.generate_terrain(config);
        world.apply_environment_gradients(config, &params);
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
            let waves = terrain_waves(&mut rng, terrain_cfg.noise_wave_count as usize);
            let elevations: Vec<f32> = (0..self.height)
                .flat_map(|y| (0..self.width).map(move |x| (x, y)))
                .map(|(x, y)| {
                    let nx = x as f32 / (self.width - 1).max(1) as f32;
                    let ny = y as f32 / (self.height - 1).max(1) as f32;
                    terrain_elevation(&waves, nx, ny)
                })
                .collect();
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

    /// Light falls top→bottom, temperature rises left→right (the two axes
    /// differ on purpose: their crossing is what creates 2D niches, GDD
    /// §5.2); the temperature gradient's spread comes from `params` (task
    /// 038), so later, harder worlds get harsher gradients (GDD §9) without
    /// changing the gradient shape itself. Doesn't touch the RNG, so it's
    /// deterministic independently of the seed, given the same `params`.
    /// Toxicity is no longer set here (task 066: `place_toxic_zone` owns
    /// it, since the zone's position now depends on generated terrain).
    fn apply_environment_gradients(&mut self, config: &SimConfig, params: &WorldParams) {
        let env = &config.environment;
        let temperature_left = env.temperature_gradient_left;
        let temperature_right = (temperature_left + params.temperature_spread).min(1.0);

        for y in 0..self.height {
            let ty = y as f32 / (self.height - 1).max(1) as f32;
            let light = lerp(env.light_gradient_high, env.light_gradient_low, ty);
            for x in 0..self.width {
                let tx = x as f32 / (self.width - 1).max(1) as f32;
                let temperature = lerp(temperature_left, temperature_right, tx);
                let idx = self.index(x, y);
                self.cells[idx].light = light;
                self.cells[idx].temperature = temperature;
            }
        }
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

    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> &Cell {
        &self.cells[self.index(x, y)]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        let idx = self.index(x, y);
        &mut self.cells[idx]
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

/// Draws `count` waves with random direction, a low frequency (so the
/// field stays smooth and organic, not noisy), and random phase, all from
/// `rng` — the sole source of randomness, so the same seed always produces
/// the same terrain.
fn terrain_waves(rng: &mut StdRng, count: usize) -> Vec<TerrainWave> {
    (0..count.max(1))
        .map(|_| {
            let angle = rng.random_range(0.0..TAU);
            TerrainWave {
                dir_x: angle.cos(),
                dir_y: angle.sin(),
                freq: rng.random_range(1.0..3.5),
                phase: rng.random_range(0.0..TAU),
            }
        })
        .collect()
}

/// Sums `waves` at normalized coordinates `(nx, ny)` into an elevation
/// value in `[0, 1]`.
fn terrain_elevation(waves: &[TerrainWave], nx: f32, ny: f32) -> f32 {
    let sum: f32 = waves
        .iter()
        .map(|wave| (wave.freq * (nx * wave.dir_x + ny * wave.dir_y) + wave.phase).sin())
        .sum();
    (sum / waves.len() as f32 + 1.0) * 0.5
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
/// Rejects a candidate whose tags net-drain *or* net-reinforce each other
/// and redraws, up to `max_self_conflict_draws` times, keeping the
/// closest-to-zero candidate seen if none lands exactly on it — this only
/// ever narrows the outcome space, so it stays deterministic for a given
/// seed.
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
/// grid before this fix. Landing on exactly `0` is what keeps a same-species
/// neighbour perfectly neutral, leaving the crowding penalty as the only
/// thing that caps local density — the assumption every carrying-capacity
/// test already relies on.
pub fn draw_species_tags(world: &mut SimWorld, config: &SimConfig) -> Vec<TagSlot> {
    let n = world
        .rng
        .random_range(config.tags.tags_per_species_min..=config.tags.tags_per_species_max)
        as usize;
    let slot_count = world.active_tags.len() as u8;
    let slots: Vec<TagSlot> = (0..slot_count).map(TagSlot).collect();

    let mut best: Option<Vec<TagSlot>> = None;
    let mut best_abs_self_interaction = i32::MAX;
    for _ in 0..config.tags.max_self_conflict_draws.max(1) {
        let candidate: Vec<TagSlot> = slots.sample(&mut world.rng, n).copied().collect();
        let self_interaction = net_self_interaction(&world.matrix, &candidate);
        if self_interaction == 0 {
            return candidate;
        }
        if self_interaction.abs() < best_abs_self_interaction {
            best_abs_self_interaction = self_interaction.abs();
            best = Some(candidate);
        }
    }
    best.expect("the loop above runs at least once")
}

/// Sum of the matrix effect every tag in `tags` exerts on every other tag in
/// `tags` — what a species feels from a same-species neighbour carrying an
/// identical tag set, the case reproduction always produces.
fn net_self_interaction(matrix: &TagMatrix, tags: &[TagSlot]) -> i32 {
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

    #[test]
    fn gradients_match_gdd_extremes() {
        let config = test_config();
        let world = SimWorld::new(42, &config);
        let env = &config.environment;

        assert_eq!(world.get(0, 0).light, env.light_gradient_high);
        assert_eq!(world.get(0, world.height - 1).light, env.light_gradient_low);
        assert_eq!(world.get(0, 0).temperature, env.temperature_gradient_left);
        assert_eq!(
            world.get(world.width - 1, 0).temperature,
            env.temperature_gradient_right
        );
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
        assert!(
            successes >= trials * 9 / 10,
            "expected the redraw to find the safe pair almost every time, got {successes}/{trials}"
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
        assert!(
            successes >= trials * 9 / 10,
            "expected the redraw to find the neutral pair almost every time, got {successes}/{trials}"
        );
    }

    #[test]
    fn draw_species_tags_never_panics_when_every_combination_is_self_destructive() {
        let mut config = test_config();
        config.tags.active_tags_early = 2;
        config.tags.tags_per_species_min = 2;
        config.tags.tags_per_species_max = 2;
        config.tags.max_self_conflict_draws = 5;

        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, -1, 0],
        };

        let mut world = SimWorld::new(7, &config);
        world.matrix = matrix;
        let tags = draw_species_tags(&mut world, &config);

        assert_eq!(tags.len(), 2);
        assert_eq!(net_self_interaction(&world.matrix, &tags), -3);
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
