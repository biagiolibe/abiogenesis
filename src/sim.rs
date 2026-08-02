// Advances the simulation by one tick (GDD §5.6). Pure Rust, no Bevy App
// required, so determinism and balance can be tested headless.

use bevy::prelude::*;
use rand::RngExt;

use crate::config::SimConfig;
use crate::state::EraState;
use crate::world::{Metabolism, Organism, SimWorld};

/// Advances the simulation by one tick: for each occupied cell, computes
/// metabolic gain, applies costs, resolves death and reproduction. Reads
/// from `world.cells` (the previous tick's snapshot) and writes into
/// `world.scratch`, then swaps the two — no "acted this tick" guard needed,
/// no dependency on iteration order, newborns don't act this tick by
/// construction (TECH_DESIGN.md §6).
pub fn step(world: &mut SimWorld, config: &SimConfig) {
    let energy = &config.energy;

    // Environment scalars are static in Phase 0, so start the write side as
    // a copy of the snapshot; organism/residue fields get overwritten below.
    world.scratch.copy_from_slice(&world.cells);

    // Residue decays every tick unless a death overwrites it further down.
    for cell in world.scratch.iter_mut() {
        cell.residue = (cell.residue - energy.residue_decay).max(0.0);
    }

    for idx in 0..world.cells.len() {
        let cell = world.cells[idx];
        let Some(organism) = cell.organism else {
            continue;
        };
        let species = &world.species[organism.species.0 as usize];

        // 1-2. Environmental fitness and metabolic gain.
        let fit = env_fit(
            cell.temperature,
            species.temp_optimum,
            species.temp_tolerance,
        );
        let gain = match species.metabolism {
            Metabolism::Photolithic => cell.light * energy.photolithic_metabolism_gain * fit,
            // Predation and decomposition are Phase 1 (GDD §5.4).
            Metabolism::Predator | Metabolism::Decomposer => 0.0,
        };

        // 3. Hidden matrix effect: always 0 in Phase 0 (no active tags yet).
        // Kept as a sum, not folded away, so Phase 1 only has to fill it in
        // (invariant 4).
        let interaction_delta = 0.0;

        // 4. Costs: base upkeep plus a carrying-capacity penalty per
        // occupied neighbour, read from the snapshot so the tick stays
        // order-independent.
        let (x, y) = (idx % world.width, idx / world.width);
        let occupied_neighbours = world
            .moore_neighbours(x, y)
            .filter(|&n| world.cells[n].organism.is_some())
            .count();
        let upkeep = match species.metabolism {
            Metabolism::Photolithic => energy.base_upkeep,
            Metabolism::Predator => energy.predator_upkeep,
            Metabolism::Decomposer => energy.decomposer_upkeep,
        };
        let crowding_penalty = energy.crowd_factor * occupied_neighbours as f32;

        // 5. Energy update.
        let new_energy = organism.energy + gain + interaction_delta - upkeep - crowding_penalty;

        // 6. Death.
        if new_energy <= 0.0 {
            world.scratch[idx].organism = None;
            // Fixed on death, not decayed leftover + this value (invariant
            // in task spec): a death this tick overwrites the residue.
            world.scratch[idx].residue = energy.residue_on_death;
            continue;
        }

        world.scratch[idx].organism = Some(Organism {
            energy: new_energy,
            ..organism
        });

        // 7. Reproduction: only if there's still an empty neighbour once the
        // birth cell is picked. Empty neighbours are collected from the
        // snapshot in index order, so the RNG draw is reproducible; the
        // scratch buffer is re-checked to resolve contention between two
        // parents claiming the same cell this tick.
        if new_energy >= species.repro_threshold {
            let empty_neighbours: Vec<usize> = world
                .moore_neighbours(x, y)
                .filter(|&n| world.cells[n].organism.is_none())
                .collect();
            if !empty_neighbours.is_empty() {
                let pick = world.rng_mut().random_range(0..empty_neighbours.len());
                let target = empty_neighbours[pick];
                if world.scratch[target].organism.is_none() {
                    world.scratch[target].organism = Some(Organism {
                        species: organism.species,
                        energy: energy.repro_cost,
                    });
                    world.scratch[idx].organism = Some(Organism {
                        energy: new_energy - energy.repro_cost,
                        ..organism
                    });
                }
            }
        }
    }

    std::mem::swap(&mut world.cells, &mut world.scratch);
    world.tick += 1;
}

/// Gaussian environmental fitness around the species' thermal optimum (GDD §5.9).
fn env_fit(temperature: f32, optimum: f32, tolerance: f32) -> f32 {
    let d = temperature - optimum;
    (-(d * d) / (2.0 * tolerance * tolerance)).exp()
}

/// Groups the per-era simulation advancement (TECH_DESIGN.md §3.4).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    Advance,
}

/// Ticks left in the era currently being animated (TECH_DESIGN.md §3.4).
#[derive(Resource, Default)]
pub struct EraProgress {
    pub(crate) remaining: u32,
}

impl EraProgress {
    pub fn start(&mut self, ticks: u32) {
        self.remaining = ticks;
    }

    pub fn remaining(&self) -> u32 {
        self.remaining
    }

    /// Used by the `r` key (world reset) to cancel any era in progress.
    pub fn cancel(&mut self) {
        self.remaining = 0;
    }
}

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        let era_tick_hz = app.world().resource::<SimConfig>().time.era_tick_hz as f64;
        app.insert_resource(Time::<Fixed>::from_hz(era_tick_hz));
        app.init_resource::<EraProgress>();
        app.add_systems(
            FixedUpdate,
            advance_tick
                .in_set(SimSet::Advance)
                .run_if(in_state(EraState::Advancing)),
        );
    }
}

/// Advances one tick of the currently-animating era, then transitions back
/// to `Observing` once `era_ticks` have been played out. Guards against a
/// stray extra `FixedUpdate` execution landing before the state transition
/// takes effect, which would otherwise run one tick too many.
fn advance_tick(
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
) {
    if progress.remaining() == 0 {
        return;
    }
    step(&mut world, &config);
    progress.remaining -= 1;
    if progress.remaining() == 0 {
        world.era += 1;
        next_state.set(EraState::Observing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameState;
    use crate::world::{Cell, Species, SpeciesId};

    const TOLERANCE: f32 = 1e-4;

    fn world_with_one_organism(light: f32, temperature: f32, energy: f32) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.species.push(Species {
            metabolism: Metabolism::Photolithic,
            temp_optimum: temperature,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        world.cells[idx] = Cell {
            light,
            temperature,
            organism: Some(Organism {
                species: SpeciesId(0),
                energy,
            }),
            ..world.cells[idx]
        };
        (world, config)
    }

    #[test]
    fn isolated_photolithic_grows() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let organism = world.get(cx, cy).organism.expect("organism survives");
        assert!(
            (organism.energy - 5.9).abs() < TOLERANCE,
            "expected net +0.9, got {}",
            organism.energy - 5.0
        );
    }

    #[test]
    fn crowded_photolithic_stalls_at_carrying_capacity() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        let (cx, cy) = (world.width / 2, world.height / 2);

        // Fill 7 of the 8 Moore neighbours with organisms of the same species,
        // far enough from repro_threshold that they don't reproduce this tick.
        let neighbours: Vec<usize> = world.moore_neighbours(cx, cy).collect();
        for &idx in neighbours.iter().take(7) {
            world.cells[idx].organism = Some(Organism {
                species: SpeciesId(0),
                energy: 1.0,
            });
        }

        step(&mut world, &config);

        let organism = world.get(cx, cy).organism.expect("organism survives");
        assert!(
            (organism.energy - 4.85).abs() < TOLERANCE,
            "expected net -0.15, got {}",
            organism.energy - 5.0
        );
    }

    #[test]
    fn photolithic_in_the_dark_eventually_dies() {
        // GDD §5.9: gain 0.4 < upkeep 0.5 ⇒ net -0.1/tick, doesn't survive.
        // Starting energy is 5.0, so death takes ~50 ticks, not the first one.
        let (mut world, config) = world_with_one_organism(0.2, 0.5, 5.0);
        let (cx, cy) = (world.width / 2, world.height / 2);

        step(&mut world, &config);
        let organism = world.get(cx, cy).organism.expect("survives the first tick");
        assert!(
            (organism.energy - 4.9).abs() < TOLERANCE,
            "expected net -0.1/tick in the dark, got {}",
            organism.energy - 5.0
        );

        for _ in 0..200 {
            if world.get(cx, cy).organism.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).organism.is_none(),
            "light niche: should not survive long-term"
        );
        assert!((world.get(cx, cy).residue - config.energy.residue_on_death).abs() < TOLERANCE);
    }

    #[test]
    fn tick_counter_increments() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        assert_eq!(world.tick, 0);
        step(&mut world, &config);
        assert_eq!(world.tick, 1);
    }

    /// Drives `advance_tick` on `Update` (rather than `FixedUpdate`) so the
    /// count doesn't depend on wall-clock accumulation — this isolates the
    /// counting/guard logic from schedule timing, which task 007 requires to
    /// be exact regardless of frame-rate hitches.
    #[test]
    fn era_advances_exactly_era_ticks_then_stops_at_observing() {
        let config = SimConfig::default();
        let (world, _) = world_with_one_organism(0.7, 0.5, 5.0);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.insert_resource(config.clone());
        app.insert_resource(world);
        app.init_state::<GameState>();
        app.add_sub_state::<EraState>();
        app.init_resource::<EraProgress>();
        app.add_systems(Update, advance_tick.run_if(in_state(EraState::Advancing)));

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // apply GameState transition, establishing EraState::Observing

        app.world_mut()
            .resource_mut::<EraProgress>()
            .start(config.time.era_ticks);
        app.world_mut()
            .resource_mut::<NextState<EraState>>()
            .set(EraState::Advancing);

        for _ in 0..config.time.era_ticks + 10 {
            app.update();
        }

        let sim_world = app.world().resource::<SimWorld>();
        assert_eq!(sim_world.tick, config.time.era_ticks as u64);
        assert_eq!(sim_world.era, 1);
        assert_eq!(
            *app.world().resource::<State<EraState>>().get(),
            EraState::Observing
        );

        for _ in 0..10 {
            app.update();
        }
        let sim_world = app.world().resource::<SimWorld>();
        assert_eq!(
            sim_world.tick, config.time.era_ticks as u64,
            "no extra ticks should run once the era has ended"
        );
    }
}
