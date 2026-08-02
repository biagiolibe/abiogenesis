// Species/organism fields are read by later tasks (004+); the domain types
// are complete and correct before the tick algorithm consumes them.
#![allow(dead_code)]

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::config::SimConfig;

/// How a species derives energy (GDD §5.4). Only `Photolithic` is active in
/// Phase 0; the other variants exist now to avoid a refactor in Phase 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metabolism {
    Photolithic,
    Predator,
    Decomposer,
}

/// Index into a tag pool (GDD §5.5). Opaque to the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagId(pub u8);

/// Index into `SimWorld::species`. Kept small: species are few and never removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeciesId(pub u8);

/// A species' genome (GDD §5.3): metabolism and environmental range are
/// player-readable, tags are opaque and drive matrix interactions.
#[derive(Debug, Clone, PartialEq)]
pub struct Species {
    pub metabolism: Metabolism,
    pub temp_optimum: f32,
    pub temp_tolerance: f32,
    pub repro_threshold: f32,
    /// 1..=3 tags (GDD §5.3).
    pub tags: Vec<TagId>,
}

/// A living instance of a species occupying a cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Organism {
    pub species: SpeciesId,
    pub energy: f32,
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
    rng: StdRng,
    /// Write-side double buffer for the tick (TECH_DESIGN.md §6). `pub(crate)`
    /// so `sim::step` can read/write it directly without a cell-by-cell API.
    pub(crate) scratch: Vec<Cell>,
}

impl SimWorld {
    pub fn new(seed: u64, config: &SimConfig) -> Self {
        let width = config.grid.width as usize;
        let height = config.grid.height as usize;
        let cells = vec![Cell::default(); width * height];
        let mut world = Self {
            width,
            height,
            scratch: cells.clone(),
            cells,
            species: Vec::new(),
            tick: 0,
            era: 0,
            seed,
            rng: StdRng::seed_from_u64(seed),
        };
        world.apply_gradients(config);
        world
    }

    /// Static Phase 0 gradients: light falls top→bottom, temperature rises
    /// left→right. The two axes differ on purpose: their crossing is what
    /// creates 2D niches (GDD §5.2). Doesn't touch the RNG, so it's
    /// deterministic independently of the seed.
    fn apply_gradients(&mut self, config: &SimConfig) {
        let env = &config.environment;
        let zone_x0 = self.width.saturating_sub(env.toxic_zone_width as usize);
        let zone_y0 = self.height.saturating_sub(env.toxic_zone_height as usize);

        for y in 0..self.height {
            let ty = y as f32 / (self.height - 1).max(1) as f32;
            let light = lerp(env.light_gradient_high, env.light_gradient_low, ty);
            for x in 0..self.width {
                let tx = x as f32 / (self.width - 1).max(1) as f32;
                let temperature = lerp(
                    env.temperature_gradient_left,
                    env.temperature_gradient_right,
                    tx,
                );
                let toxicity = if x >= zone_x0 && y >= zone_y0 {
                    env.toxic_zone_value
                } else {
                    0.0
                };
                let idx = self.index(x, y);
                self.cells[idx].light = light;
                self.cells[idx].temperature = temperature;
                self.cells[idx].toxicity = toxicity;
            }
        }
    }

    /// Phase 1+: blend each scalar toward its neighbours' mean at
    /// `diffusion_rate` per tick (GDD §5.2). Not active in Phase 0: gradients
    /// are static, so the environment stays a fixed target while the tick
    /// algorithm is tuned.
    fn diffuse_environment(&mut self, _config: &SimConfig) {
        // Intentionally empty in Phase 0.
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

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_world);
    }
}

fn spawn_world(mut commands: Commands, config: Res<SimConfig>) {
    // Fixed seed in Phase 0; interactive reseeding is task 007.
    commands.insert_resource(SimWorld::new(42, &config));
}

/// Exact at `t = 0.0` and `t = 1.0` (unlike `from + (to - from) * t`), which
/// matters here: tests assert the grid extremes equal the GDD values exactly.
fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from * (1.0 - t) + to * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

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

    #[test]
    fn toxic_zone_is_isolated_to_its_corner() {
        let config = test_config();
        let world = SimWorld::new(42, &config);
        let env = &config.environment;

        assert_eq!(
            world.get(world.width - 1, world.height - 1).toxicity,
            env.toxic_zone_value
        );
        assert_eq!(world.get(0, 0).toxicity, 0.0);
    }

    #[test]
    fn same_seed_produces_identical_environment() {
        let config = test_config();
        let a = SimWorld::new(42, &config);
        let b = SimWorld::new(42, &config);

        assert_eq!(a.cells, b.cells);
    }
}
