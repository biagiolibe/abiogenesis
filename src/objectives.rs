// Per-world objectives (task 040, GDD §8): explicit requirements a world
// poses, checked tick by tick against `SimWorld`'s own observable state.
// Procedural generation of *which* objective a given world gets is task
// 042 — this module only defines the type and the evaluation engine, and
// is deliberately testable against a hand-built `SimWorld`, independent of
// worldgen.

use bevy::prelude::*;

use crate::sim::SimSet;
use crate::state::EraState;
use crate::world::{SimWorld, SpeciesId};

/// A region of the grid an objective can reference. Currently only the
/// toxic zone (GDD §8's "survives in the toxic zone" example) — extend here
/// if a future objective needs another named region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneKind {
    Toxic,
}

/// One world's explicit requirement (GDD §8). Expressed only in terms of
/// quantities the player can observe (species counts, population, zone
/// occupancy) — never a hidden-matrix cell, or satisfying the objective
/// would leak information the notebook is supposed to make the player earn
/// (GDD §11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Objective {
    /// "Achieve a biosphere with `min_species` coexisting species for
    /// `ticks` consecutive ticks."
    Coexistence { min_species: u32, ticks: u32 },
    /// "Grow a species that survives in `zone` for `ticks` consecutive
    /// ticks."
    SurviveIn {
        species: SpeciesId,
        zone: ZoneKind,
        ticks: u32,
    },
    /// "Trigger a bloom": `species`'s population reaches
    /// `population_threshold` in a single tick. Unlike the other two
    /// variants this isn't a sustained condition — a bloom is a triggering
    /// event, not a state to hold (mirrors the bloom-detection direction
    /// sketched as a TODO in `notebook.rs`'s salient-event log).
    TriggerBloom {
        species: SpeciesId,
        population_threshold: u32,
    },
}

/// Where a world currently stands relative to its objective (and, once task
/// 041 wires failure conditions in, its failure conditions too — this is
/// the type task 041's acceptance criteria call out as shared).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorldOutcome {
    #[default]
    Ongoing,
    Cleared,
    Failed,
}

/// Tracks progress toward the current `Objective`. `consecutive_ticks`
/// counts ticks in a row the objective's condition has held (reset to `0`,
/// never decremented, the moment it doesn't — GDD §8 doesn't reward partial
/// credit); it keeps counting across era boundaries by construction, since
/// nothing here resets it when an era ends, only `evaluate` itself. Once
/// `satisfied` is set it stays set — `evaluate` short-circuits back to
/// `WorldOutcome::Cleared` without re-checking the condition.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ObjectiveProgress {
    pub consecutive_ticks: u32,
    pub satisfied: bool,
}

/// The objective assigned to the current world. `None` until task 042's
/// worldgen assigns one (or a test sets one directly) — the driving system
/// below is a no-op while empty.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentObjective(pub Option<Objective>);

/// The current world's outcome, as last computed by `evaluate`. A plain
/// resource (not an event) since UI (task 043) and run-flow systems (task
/// 041, 045) need to read "where do things stand right now," not react to
/// a one-shot transition.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentWorldOutcome(pub WorldOutcome);

/// Checks `objective` against `world`'s current state and updates
/// `progress` accordingly. Pure function of its arguments — no RNG, no
/// `SimWorld` mutation, no Bevy dependency — so it's callable from a plain
/// unit test with a hand-built `SimWorld`, independent of the Bevy `App`
/// and of worldgen (invariant 2, TECH_DESIGN.md §5).
pub fn evaluate(
    objective: &Objective,
    world: &SimWorld,
    progress: &mut ObjectiveProgress,
) -> WorldOutcome {
    if progress.satisfied {
        return WorldOutcome::Cleared;
    }

    match *objective {
        Objective::Coexistence { min_species, ticks } => {
            let holds = count_coexisting_species(world) >= min_species;
            evaluate_sustained(holds, ticks, progress)
        }
        Objective::SurviveIn {
            species,
            zone,
            ticks,
        } => {
            let holds = species_present_in_zone(world, species, zone);
            evaluate_sustained(holds, ticks, progress)
        }
        Objective::TriggerBloom {
            species,
            population_threshold,
        } => {
            if population_of(world, species) >= population_threshold {
                progress.satisfied = true;
                WorldOutcome::Cleared
            } else {
                WorldOutcome::Ongoing
            }
        }
    }
}

/// Shared "condition must hold for `required_ticks` in a row" logic behind
/// `Coexistence` and `SurviveIn`.
fn evaluate_sustained(
    condition_holds: bool,
    required_ticks: u32,
    progress: &mut ObjectiveProgress,
) -> WorldOutcome {
    if condition_holds {
        progress.consecutive_ticks += 1;
    } else {
        progress.consecutive_ticks = 0;
    }

    if progress.consecutive_ticks >= required_ticks {
        progress.satisfied = true;
        WorldOutcome::Cleared
    } else {
        WorldOutcome::Ongoing
    }
}

/// Number of distinct species with at least one living organism on the
/// grid right now.
fn count_coexisting_species(world: &SimWorld) -> u32 {
    let mut population = vec![0u32; world.species.len()];
    for cell in &world.cells {
        if let Some(organism) = cell.organism {
            population[organism.species.0 as usize] += 1;
        }
    }
    population.iter().filter(|&&count| count > 0).count() as u32
}

/// Whether any living organism of `species` currently occupies a cell in
/// `zone`.
fn species_present_in_zone(world: &SimWorld, species: SpeciesId, zone: ZoneKind) -> bool {
    world.cells.iter().any(|cell| {
        cell.organism
            .is_some_and(|organism| organism.species == species)
            && cell_in_zone(cell, zone)
    })
}

fn cell_in_zone(cell: &crate::world::Cell, zone: ZoneKind) -> bool {
    match zone {
        ZoneKind::Toxic => cell.toxicity > 0.0,
    }
}

/// Living population of `species` across the whole grid.
fn population_of(world: &SimWorld, species: SpeciesId) -> u32 {
    world
        .cells
        .iter()
        .filter(|cell| {
            cell.organism
                .is_some_and(|organism| organism.species == species)
        })
        .count() as u32
}

pub struct ObjectivesPlugin;

impl Plugin for ObjectivesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentObjective>()
            .init_resource::<ObjectiveProgress>()
            .init_resource::<CurrentWorldOutcome>()
            .add_systems(
                FixedUpdate,
                evaluate_current_objective
                    .after(SimSet::Advance)
                    .run_if(in_state(EraState::Advancing)),
            );
    }
}

/// Drives `evaluate` once per simulated tick (same `FixedUpdate` cadence as
/// `sim::advance_tick`, right after it — GDD §8's "N ticks" objectives need
/// tick-granularity, not era-granularity). A no-op while no objective has
/// been assigned yet.
fn evaluate_current_objective(
    world: Res<SimWorld>,
    objective: Res<CurrentObjective>,
    mut progress: ResMut<ObjectiveProgress>,
    mut outcome: ResMut<CurrentWorldOutcome>,
) {
    let Some(objective) = objective.0 else {
        return;
    };
    outcome.0 = evaluate(&objective, &world, &mut progress);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimConfig;
    use crate::world::{Cell, Metabolism, Organism, Species};

    fn world_with_species(count: usize) -> SimWorld {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        for _ in 0..count {
            world.species.push(Species {
                metabolism: Metabolism::Photolithic,
                temp_optimum: 0.5,
                temp_tolerance: config.energy.default_temp_tolerance,
                repro_threshold: config.energy.repro_threshold,
                tags: Vec::new(),
            });
        }
        world
    }

    fn place(world: &mut SimWorld, x: usize, y: usize, species: SpeciesId) {
        let idx = world.index(x, y);
        world.cells[idx] = Cell {
            organism: Some(Organism {
                species,
                energy: 5.0,
            }),
            ..world.cells[idx]
        };
    }

    #[test]
    fn coexistence_requires_the_configured_species_count() {
        let mut world = world_with_species(3);
        place(&mut world, 0, 0, SpeciesId(0));
        place(&mut world, 1, 0, SpeciesId(1));
        // Species 2 never placed: only 2 species coexist.

        let objective = Objective::Coexistence {
            min_species: 3,
            ticks: 1,
        };
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Ongoing
        );

        place(&mut world, 2, 0, SpeciesId(2));
        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Cleared
        );
    }

    #[test]
    fn coexistence_resets_the_consecutive_count_on_interruption_not_decrements() {
        let mut world = world_with_species(3);
        place(&mut world, 0, 0, SpeciesId(0));
        place(&mut world, 1, 0, SpeciesId(1));
        place(&mut world, 2, 0, SpeciesId(2));

        let objective = Objective::Coexistence {
            min_species: 3,
            ticks: 5,
        };
        let mut progress = ObjectiveProgress::default();

        for _ in 0..3 {
            assert_eq!(
                evaluate(&objective, &world, &mut progress),
                WorldOutcome::Ongoing
            );
        }
        assert_eq!(progress.consecutive_ticks, 3);

        // Species 2 drops out for one tick: the count must reset to 0, not
        // to 2.
        let idx = world.index(2, 0);
        world.cells[idx].organism = None;
        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Ongoing
        );
        assert_eq!(
            progress.consecutive_ticks, 0,
            "an interrupted streak must reset to zero, not decrement"
        );

        // Species 2 comes back: needs 5 fresh consecutive ticks from here.
        place(&mut world, 2, 0, SpeciesId(2));
        for tick in 1..5 {
            assert_eq!(
                evaluate(&objective, &world, &mut progress),
                WorldOutcome::Ongoing,
                "tick {tick} of the fresh streak should not satisfy yet"
            );
        }
        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Cleared
        );
    }

    #[test]
    fn survive_in_toxic_zone_requires_sustained_presence() {
        let mut world = world_with_species(1);
        let idx = world.index(0, 0);
        world.cells[idx].toxicity = 0.7;
        place(&mut world, 0, 0, SpeciesId(0));

        let objective = Objective::SurviveIn {
            species: SpeciesId(0),
            zone: ZoneKind::Toxic,
            ticks: 3,
        };
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Ongoing
        );
        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Ongoing
        );
        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Cleared
        );
    }

    #[test]
    fn survive_in_toxic_zone_is_not_satisfied_by_a_clean_cell() {
        let mut world = world_with_species(1);
        // toxicity stays at its default 0.0.
        place(&mut world, 0, 0, SpeciesId(0));

        let objective = Objective::SurviveIn {
            species: SpeciesId(0),
            zone: ZoneKind::Toxic,
            ticks: 1,
        };
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Ongoing
        );
    }

    #[test]
    fn trigger_bloom_is_satisfied_the_instant_the_threshold_is_reached() {
        let mut world = world_with_species(1);
        for x in 0..4 {
            place(&mut world, x, 0, SpeciesId(0));
        }

        let objective = Objective::TriggerBloom {
            species: SpeciesId(0),
            population_threshold: 5,
        };
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Ongoing,
            "only 4 organisms placed, threshold is 5"
        );

        place(&mut world, 4, 0, SpeciesId(0));
        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Cleared
        );
    }

    #[test]
    fn a_satisfied_objective_stays_cleared_even_if_the_condition_later_breaks() {
        let mut world = world_with_species(1);
        for x in 0..5 {
            place(&mut world, x, 0, SpeciesId(0));
        }
        let objective = Objective::TriggerBloom {
            species: SpeciesId(0),
            population_threshold: 5,
        };
        let mut progress = ObjectiveProgress::default();
        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Cleared
        );

        // Population collapses afterwards: already-cleared objectives don't
        // un-clear.
        for x in 0..5 {
            let idx = world.index(x, 0);
            world.cells[idx].organism = None;
        }
        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Cleared
        );
    }
}
