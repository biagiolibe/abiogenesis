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

    // Predation pre-pass (GDD §5.4): a shared-resource drain computed from
    // the immutable snapshot, into per-cell accumulators, before the main
    // loop — see TECH_DESIGN.md §6 "Shared resource drain". This keeps the
    // tick order-independent: a predator never writes directly into a prey's
    // scratch entry while iterating.
    let mut predation_gain = vec![0.0f32; world.cells.len()];
    let mut predation_loss = vec![0.0f32; world.cells.len()];
    for (idx, cell) in world.cells.iter().enumerate() {
        let Some(organism) = cell.organism else {
            continue;
        };
        let species = &world.species[organism.species.0 as usize];
        if species.metabolism != Metabolism::Predator {
            continue;
        }
        let (x, y) = (idx % world.width, idx / world.width);
        let prey: Vec<usize> = world
            .moore_neighbours(x, y)
            .filter(|&n| world.cells[n].organism.is_some())
            .collect();
        if prey.is_empty() {
            continue;
        }
        let fit = env_fit(
            cell.temperature,
            species.temp_optimum,
            species.temp_tolerance,
        );
        let available: f32 = prey
            .iter()
            .map(|&n| world.cells[n].organism.unwrap().energy)
            .sum();
        let drawn = (energy.predator_drain_cap * fit).min(available);
        predation_gain[idx] = drawn;
        // Split evenly across prey neighbours (GDD doesn't pin down a
        // species-specific targeting rule for Phase 1).
        let share = drawn / prey.len() as f32;
        for &n in &prey {
            predation_loss[n] += share;
        }
    }

    // Decomposition pre-pass (GDD §5.4): extends the shared-resource drain
    // pattern above to residue. Draws from the decomposer's own cell plus
    // its Moore neighbours, reading `world.scratch`'s residue — which has
    // already been decayed above — so decay and extraction compose in a
    // fixed, documented order (decay first) rather than one overwriting the
    // other (TECH_DESIGN.md §6 "Shared resource drain").
    let mut decomposition_gain = vec![0.0f32; world.cells.len()];
    let mut residue_loss = vec![0.0f32; world.cells.len()];
    for (idx, cell) in world.cells.iter().enumerate() {
        let Some(organism) = cell.organism else {
            continue;
        };
        let species = &world.species[organism.species.0 as usize];
        if species.metabolism != Metabolism::Decomposer {
            continue;
        }
        let (x, y) = (idx % world.width, idx / world.width);
        let sources: Vec<usize> = std::iter::once(idx)
            .chain(world.moore_neighbours(x, y))
            .filter(|&n| world.scratch[n].residue > 0.0)
            .collect();
        if sources.is_empty() {
            continue;
        }
        let fit = env_fit(
            cell.temperature,
            species.temp_optimum,
            species.temp_tolerance,
        );
        let available: f32 = sources.iter().map(|&n| world.scratch[n].residue).sum();
        let drawn = (energy.decomposer_extract_rate * fit).min(available);
        decomposition_gain[idx] = drawn;
        // Distribute proportionally to each source's residue share, capped
        // at that source's own residue so a single decomposer's draw can
        // never overdraw one of its sources.
        for &n in &sources {
            let share = drawn * world.scratch[n].residue / available;
            residue_loss[n] += share.min(world.scratch[n].residue);
        }
    }

    // Apply the accumulated extraction. A final clamp to 0.0 is the
    // multi-decomposer safety net: two decomposers competing for the same
    // residue each compute their share against the same pre-extraction
    // snapshot, so their combined draw can exceed what's actually there —
    // residue must never go negative regardless.
    for (cell, loss) in world.scratch.iter_mut().zip(residue_loss.iter()) {
        cell.residue = (cell.residue - loss).max(0.0);
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
            Metabolism::Predator => predation_gain[idx],
            Metabolism::Decomposer => decomposition_gain[idx],
        };

        // 3. Hidden matrix effect (GDD §5.6 step 3, §5.5): additive and
        // linear (invariant 4), read only from the snapshot like everything
        // else here, so the tick stays order-independent. For every
        // occupied Moore neighbour, for every (their tag, my tag) pair, sum
        // the matrix entry — row = exerting tag, column = receiving tag.
        let (x, y) = (idx % world.width, idx / world.width);
        let mut interaction_delta = 0.0;
        for neighbour_idx in world.moore_neighbours(x, y) {
            let Some(neighbour) = world.cells[neighbour_idx].organism else {
                continue;
            };
            let neighbour_species = &world.species[neighbour.species.0 as usize];
            for &their_tag in &neighbour_species.tags {
                for &my_tag in &species.tags {
                    interaction_delta += world.matrix.get(their_tag, my_tag) as f32;
                }
            }
        }

        // 4. Costs: base upkeep plus a carrying-capacity penalty per
        // occupied neighbour, read from the snapshot so the tick stays
        // order-independent.
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
        let new_energy = organism.energy + gain + interaction_delta
            - upkeep
            - crowding_penalty
            - predation_loss[idx];

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
    use crate::world::{Cell, Species, SpeciesId, TagId, TagMatrix};

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

    /// Two adjacent photolithic organisms (species 0 at `(cx, cy)`, species 1
    /// at `(cx + 1, cy)`), sharing `light`/`temperature` so the photolithic
    /// gain term is identical to `world_with_one_organism`'s, and overriding
    /// `world.matrix` with a hand-built one so the adjacency effect (task
    /// 012) is exactly known.
    fn world_with_two_neighbours(
        matrix: TagMatrix,
        tags_a: Vec<TagId>,
        tags_b: Vec<TagId>,
        light: f32,
        temperature: f32,
        energy_a: f32,
        energy_b: f32,
    ) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.matrix = matrix;
        for tags in [tags_a, tags_b] {
            world.species.push(Species {
                metabolism: Metabolism::Photolithic,
                temp_optimum: temperature,
                temp_tolerance: config.energy.default_temp_tolerance,
                repro_threshold: config.energy.repro_threshold,
                tags,
            });
        }
        let (cx, cy) = (world.width / 2, world.height / 2);
        for (dx, species, energy) in [(0, SpeciesId(0), energy_a), (1, SpeciesId(1), energy_b)] {
            let idx = world.index(cx + dx, cy);
            world.cells[idx] = Cell {
                light,
                temperature,
                organism: Some(Organism { species, energy }),
                ..world.cells[idx]
            };
        }
        (world, config)
    }

    #[test]
    fn negative_adjacency_effect_subtracts_energy() {
        // 2-tag matrix: tag 0 (species A) harms tag 1 (species B) by -2;
        // the reverse direction stays 0.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };
        let (mut world, config) =
            world_with_two_neighbours(matrix, vec![TagId(0)], vec![TagId(1)], 0.7, 0.5, 5.0, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        // gain 1.4, upkeep 0.5, 1 occupied neighbour -> crowding 0.15,
        // interaction_delta -2: net 5.0 + 1.4 - 0.5 - 0.15 - 2.0 = 3.75.
        assert!((b.energy - 3.75).abs() < TOLERANCE, "got {}", b.energy);
    }

    #[test]
    fn positive_adjacency_effect_adds_energy() {
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, 2, 0, 0],
        };
        let (mut world, config) =
            world_with_two_neighbours(matrix, vec![TagId(0)], vec![TagId(1)], 0.7, 0.5, 5.0, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        // net 5.0 + 1.4 - 0.5 - 0.15 + 2.0 = 7.75.
        assert!((b.energy - 7.75).abs() < TOLERANCE, "got {}", b.energy);
    }

    #[test]
    fn no_matching_tags_yields_zero_adjacency_effect() {
        // Non-zero matrix entries exist, but B carries no tags at all, so
        // the (their tag x my tag) double loop contributes nothing.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, -2, 0],
        };
        let (mut world, config) =
            world_with_two_neighbours(matrix, vec![TagId(0)], Vec::new(), 0.7, 0.5, 5.0, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        // net 5.0 + 1.4 - 0.5 - 0.15 + 0.0 = 5.75.
        assert!((b.energy - 5.75).abs() < TOLERANCE, "got {}", b.energy);
    }

    #[test]
    fn adjacency_effect_sums_over_multiple_neighbours() {
        // Two A neighbours (west and east of B), each carrying tag 0, each
        // contributing -1 to B (tag 1): total interaction_delta -2, and
        // crowding now counts 2 occupied neighbours.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -1, 0, 0],
        };
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.matrix = matrix;
        world.species.push(Species {
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagId(0)],
        });
        world.species.push(Species {
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagId(1)],
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        for (dx, species, energy) in [
            (-1i32, SpeciesId(0), 5.0),
            (0, SpeciesId(1), 5.0),
            (1, SpeciesId(0), 5.0),
        ] {
            let idx = world.index((cx as i32 + dx) as usize, cy);
            world.cells[idx] = Cell {
                light: 0.7,
                temperature: 0.5,
                organism: Some(Organism { species, energy }),
                ..world.cells[idx]
            };
        }

        step(&mut world, &config);

        let b = world.get(cx, cy).organism.expect("B survives");
        // net 5.0 + 1.4 - 0.5 - 0.15*2 + (-1 - 1) = 3.6.
        assert!((b.energy - 3.6).abs() < TOLERANCE, "got {}", b.energy);
    }

    fn world_with_one_predator(temperature: f32, energy: f32) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.species.push(Species {
            metabolism: Metabolism::Predator,
            temp_optimum: temperature,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        world.cells[idx] = Cell {
            light: 0.0,
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
    fn isolated_predator_collapses_with_no_gain() {
        // GDD §5.9: seed_energy 5.0, predator_upkeep 0.7, no prey ⇒ dies
        // after ceil(5.0 / 0.7) = 8 ticks, with zero gain every tick.
        let (mut world, config) = world_with_one_predator(0.5, config_seed_energy());
        let (cx, cy) = (world.width / 2, world.height / 2);

        for tick in 1..=8 {
            step(&mut world, &config);
            let alive = world.get(cx, cy).organism.is_some();
            if tick < 8 {
                assert!(alive, "predator should still be alive at tick {tick}");
            } else {
                assert!(!alive, "predator should have died by tick {tick}");
            }
        }
    }

    fn config_seed_energy() -> f32 {
        SimConfig::default().energy.seed_energy
    }

    #[test]
    fn predator_with_abundant_prey_nets_positive_energy() {
        let (mut world, config) = world_with_one_predator(0.5, 5.0);
        let (cx, cy) = (world.width / 2, world.height / 2);

        world.species.push(Species {
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let neighbours: Vec<usize> = world.moore_neighbours(cx, cy).collect();
        for &idx in &neighbours {
            world.cells[idx].organism = Some(Organism {
                species: SpeciesId(1),
                energy: 20.0,
            });
        }

        step(&mut world, &config);

        let predator = world.get(cx, cy).organism.expect("predator survives");
        // drain = min(predator_drain_cap, available) * fit = 2.0 (fit=1.0,
        // available huge), upkeep 0.7, 8 occupied neighbours -> crowding
        // 0.15*8=1.2: net 5.0 + 2.0 - 0.7 - 1.2 = 5.1.
        assert!(
            (predator.energy - 5.1).abs() < TOLERANCE,
            "got {}",
            predator.energy
        );
    }

    #[test]
    fn two_predators_sharing_one_prey_split_deterministically() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.species.push(Species {
            metabolism: Metabolism::Predator,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        world.species.push(Species {
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        // Prey at (cx, cy), predators at (cx - 1, cy) and (cx + 1, cy), both
        // Moore-adjacent to the prey and sharing it as their only neighbour.
        let (cx, cy) = (world.width / 2, world.height / 2);
        let prey_idx = world.index(cx, cy);
        world.cells[prey_idx] = Cell {
            light: 0.0,
            temperature: 0.5,
            organism: Some(Organism {
                species: SpeciesId(1),
                energy: 20.0,
            }),
            ..world.cells[prey_idx]
        };
        for dx in [-1i32, 1] {
            let idx = world.index((cx as i32 + dx) as usize, cy);
            world.cells[idx] = Cell {
                light: 0.0,
                temperature: 0.5,
                organism: Some(Organism {
                    species: SpeciesId(0),
                    energy: 5.0,
                }),
                ..world.cells[idx]
            };
        }

        step(&mut world, &config);

        let left = world
            .get(cx - 1, cy)
            .organism
            .expect("left predator survives");
        let right = world
            .get(cx + 1, cy)
            .organism
            .expect("right predator survives");
        assert!(
            (left.energy - right.energy).abs() < TOLERANCE,
            "left {} vs right {}",
            left.energy,
            right.energy
        );
        // Each predator's only prey neighbour is the shared one, so each
        // draws its full drain_cap (fit=1.0, prey has plenty of energy):
        // net 5.0 + 2.0 - 0.7 - 0.15 = 6.15.
        assert!(
            (left.energy - 6.15).abs() < TOLERANCE,
            "got {}",
            left.energy
        );
    }

    fn world_with_one_decomposer(temperature: f32, energy: f32) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.species.push(Species {
            metabolism: Metabolism::Decomposer,
            temp_optimum: temperature,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        world.cells[idx] = Cell {
            light: 0.0,
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
    fn decomposer_with_no_residue_behaves_like_dark_photolithic() {
        // No residue anywhere ⇒ gain 0, loses decomposer_upkeep (0.5) per
        // tick, same shape as photolithic_in_the_dark_eventually_dies.
        let (mut world, config) = world_with_one_decomposer(0.5, config_seed_energy());
        let (cx, cy) = (world.width / 2, world.height / 2);

        step(&mut world, &config);
        let organism = world.get(cx, cy).organism.expect("survives the first tick");
        assert!(
            (organism.energy - (config_seed_energy() - config.energy.decomposer_upkeep)).abs()
                < TOLERANCE,
            "expected net -upkeep/tick with no residue, got {}",
            organism.energy - config_seed_energy()
        );

        for _ in 0..200 {
            if world.get(cx, cy).organism.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).organism.is_none(),
            "decomposer with no residue in range should not survive long-term"
        );
    }

    #[test]
    fn decomposer_adjacent_to_residue_gains_and_residue_shrinks() {
        let (mut world, config) = world_with_one_decomposer(0.5, 5.0);
        let (cx, cy) = (world.width / 2, world.height / 2);
        let neighbour_idx = world
            .moore_neighbours(cx, cy)
            .next()
            .expect("has a neighbour");
        world.cells[neighbour_idx].residue = 10.0;

        step(&mut world, &config);

        let decomposer = world.get(cx, cy).organism.expect("decomposer survives");
        // decay first: 10.0 - residue_decay(0.2) = 9.8 available; drawn =
        // min(decomposer_extract_rate(1.5) * fit(1.0), 9.8) = 1.5; net
        // 5.0 + 1.5 - decomposer_upkeep(0.5) = 6.0.
        assert!(
            (decomposer.energy - 6.0).abs() < TOLERANCE,
            "got {}",
            decomposer.energy
        );
        let (nx, ny) = (neighbour_idx % world.width, neighbour_idx / world.width);
        assert!(
            (world.get(nx, ny).residue - 8.3).abs() < TOLERANCE,
            "expected residue reduced by the drawn amount, got {}",
            world.get(nx, ny).residue
        );
    }

    #[test]
    fn residue_never_goes_negative_under_competing_decomposers() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.species.push(Species {
            metabolism: Metabolism::Decomposer,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        // Residue cell at (cx, cy); two decomposers straddle it on the same
        // row so each is Moore-adjacent only to the residue cell, not to
        // each other, and each independently computes the full residue as
        // available — their combined draw exceeds what's actually there.
        let (cx, cy) = (world.width / 2, world.height / 2);
        let residue_idx = world.index(cx, cy);
        world.cells[residue_idx].residue = 2.0;
        for dx in [-1i32, 1] {
            let idx = world.index((cx as i32 + dx) as usize, cy);
            world.cells[idx] = Cell {
                light: 0.0,
                temperature: 0.5,
                organism: Some(Organism {
                    species: SpeciesId(0),
                    energy: 5.0,
                }),
                ..world.cells[idx]
            };
        }

        step(&mut world, &config);

        assert!(
            world.get(cx, cy).residue >= 0.0,
            "residue must never go negative, got {}",
            world.get(cx, cy).residue
        );
        assert!(
            world.get(cx, cy).residue.abs() < TOLERANCE,
            "expected residue fully depleted (over-drawn to 0), got {}",
            world.get(cx, cy).residue
        );
    }

    #[test]
    fn phase_0_single_species_worlds_still_have_zero_interaction_delta() {
        // Regression guard: existing single-species tests build worlds with
        // no tags at all, so the matrix wiring must not change their result.
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let organism = world.get(cx, cy).organism.expect("organism survives");
        assert!(
            (organism.energy - 5.9).abs() < TOLERANCE,
            "got {}",
            organism.energy
        );
    }
}
