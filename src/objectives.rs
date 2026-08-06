// Per-world objectives (task 040, GDD §8): explicit requirements a world
// poses, checked tick by tick against `SimWorld`'s own observable state.
// Procedural generation of *which* objective a given world gets is task
// 042 — this module only defines the type and the evaluation engine, and
// is deliberately testable against a hand-built `SimWorld`, independent of
// worldgen.

use bevy::prelude::*;

use crate::config::SimConfig;
use crate::run::{MetaProgress, RunProgress};
use crate::sim::SimSet;
use crate::state::{EraState, GameState};
use crate::world::{SimWorld, SpeciesId};
use crate::worldgen::world_params;

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

/// Why a world's outcome became `WorldOutcome::Failed` (GDD §8): the two
/// failure conditions are independent, but only one can be the *first* to
/// trigger — `evaluate_world` reports whichever it detects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureReason {
    /// No living organism remains on the grid (GDD §8's "obvious floor").
    TotalExtinction,
    /// `world.era` reached the world's `WorldParams::era_budget` without the
    /// objective having been satisfied (GDD §8's "generous but finite" clock).
    EraBudgetExhausted,
}

/// Where a world currently stands relative to its objective and failure
/// conditions (task 041, GDD §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorldOutcome {
    #[default]
    Ongoing,
    Cleared,
    Failed(FailureReason),
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

/// Checks `objective` (if any has been assigned yet) and both failure
/// conditions against `world`'s current state, and returns whichever
/// `WorldOutcome` applies. Pure function (no RNG, no `SimWorld` mutation, no
/// Bevy dependency), like `evaluate`, so it's unit-testable against
/// hand-built worlds independent of worldgen and the running `App`.
///
/// Total extinction is checked unconditionally, every call — the caller
/// (task 041's driving system) runs this once per simulated tick, not just
/// at era boundaries, so a mid-era wipeout fails the world in the same tick
/// it happens rather than up to `era_ticks` ticks later. The era-budget
/// check only matters once the objective hasn't already cleared it, since
/// clearing on the exact tick the budget would otherwise expire is still a
/// win, not a loss.
pub fn evaluate_world(
    objective: Option<&Objective>,
    world: &SimWorld,
    progress: &mut ObjectiveProgress,
    era_budget: u32,
) -> WorldOutcome {
    if is_total_extinction(world) {
        return WorldOutcome::Failed(FailureReason::TotalExtinction);
    }

    let outcome = match objective {
        Some(objective) => evaluate(objective, world, progress),
        None => WorldOutcome::Ongoing,
    };
    if outcome == WorldOutcome::Cleared {
        return outcome;
    }

    if is_era_budget_exhausted(world, era_budget) {
        return WorldOutcome::Failed(FailureReason::EraBudgetExhausted);
    }

    outcome
}

/// True once every living organism on the grid has died. Guarded against
/// the false positive of the instant right after `SimWorld` construction,
/// before the starting species have been placed: `world.species` is empty
/// only in that window (in practice, seeding the starting palette pushes
/// species and places their organisms in the same synchronous call, so no
/// real tick ever observes "species exist, none are placed yet") — treating
/// an empty registry as extinction would fail a world before it began.
fn is_total_extinction(world: &SimWorld) -> bool {
    !world.species.is_empty() && world.cells.iter().all(|cell| cell.organism.is_none())
}

/// Whether `world`'s era count has reached its per-world budget
/// (`WorldParams::era_budget`, task 037) without the objective having been
/// satisfied yet.
fn is_era_budget_exhausted(world: &SimWorld, era_budget: u32) -> bool {
    world.era >= era_budget
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

/// Drives `evaluate_world` once per simulated tick (same `FixedUpdate`
/// cadence as `sim::advance_tick`, right after it — GDD §8's "N ticks"
/// objectives need tick-granularity, not era-granularity; task 041's
/// total-extinction check needs it too, so a mid-era wipeout doesn't wait
/// for the era boundary). Runs even while no objective has been assigned
/// yet (`CurrentObjective(None)`, before task 042's worldgen wires one in):
/// `evaluate_world` still checks failure conditions in that case.
#[allow(clippy::too_many_arguments)]
fn evaluate_current_objective(
    world: Res<SimWorld>,
    objective: Res<CurrentObjective>,
    mut progress: ResMut<ObjectiveProgress>,
    mut outcome: ResMut<CurrentWorldOutcome>,
    run_progress: Res<RunProgress>,
    mut meta: ResMut<MetaProgress>,
    config: Res<SimConfig>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let era_budget = world_params(run_progress.world_index, &config).era_budget;
    let previous_outcome = outcome.0;
    let new_outcome = evaluate_world(objective.0.as_ref(), &world, &mut progress, era_budget);
    outcome.0 = new_outcome;

    // Only act on the `Ongoing -> {Failed, Cleared}` edge, not every tick
    // the world spends already concluded. `FixedUpdate` can run this system
    // more than once in a single frame (timestep catch-up after a hitch);
    // without this guard, a frame that both crosses the `Failed` threshold
    // *and* catches up an extra tick would call `meta.absorb` twice for the
    // same run (`evaluate_world` doesn't short-circuit `Failed` the way it
    // does `Cleared`, so this can't rely on that alone).
    if previous_outcome != WorldOutcome::Ongoing {
        return;
    }
    match new_outcome {
        WorldOutcome::Failed(_) => {
            meta.absorb(run_progress.worlds_cleared);
            next_game_state.set(GameState::Defeat);
        }
        WorldOutcome::Cleared => next_game_state.set(GameState::WorldCleared),
        WorldOutcome::Ongoing => {}
    }
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

    #[test]
    fn era_budget_exhausted_fails_the_world_when_the_objective_is_unmet() {
        let mut world = world_with_species(1);
        place(&mut world, 0, 0, SpeciesId(0));
        world.era = 40;

        let objective = Objective::Coexistence {
            min_species: 2, // never satisfied: only 1 species is placed.
            ticks: 1,
        };
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate_world(Some(&objective), &world, &mut progress, 40),
            WorldOutcome::Failed(FailureReason::EraBudgetExhausted)
        );
    }

    #[test]
    fn era_budget_not_yet_exhausted_stays_ongoing() {
        let mut world = world_with_species(1);
        place(&mut world, 0, 0, SpeciesId(0));
        world.era = 39;

        let objective = Objective::Coexistence {
            min_species: 2,
            ticks: 1,
        };
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate_world(Some(&objective), &world, &mut progress, 40),
            WorldOutcome::Ongoing
        );
    }

    #[test]
    fn clearing_the_objective_on_the_exhausting_tick_still_counts_as_cleared() {
        let mut world = world_with_species(2);
        place(&mut world, 0, 0, SpeciesId(0));
        place(&mut world, 1, 0, SpeciesId(1));
        world.era = 40;

        let objective = Objective::Coexistence {
            min_species: 2,
            ticks: 1,
        };
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate_world(Some(&objective), &world, &mut progress, 40),
            WorldOutcome::Cleared,
            "the objective clears on this very tick, so the world should not fail instead"
        );
    }

    #[test]
    fn total_extinction_fails_the_world_in_the_same_tick_it_happens() {
        let mut world = world_with_species(1);
        place(&mut world, 0, 0, SpeciesId(0));
        let objective = Objective::Coexistence {
            min_species: 1,
            ticks: 1,
        };
        let mut progress = ObjectiveProgress::default();

        // Still alive: ongoing (and would clear next call if it stayed alive).
        assert_eq!(
            evaluate_world(Some(&objective), &world, &mut progress, 40),
            WorldOutcome::Cleared
        );

        // The lone organism dies mid-era: must fail on this very call, not
        // one tick later.
        let idx = world.index(0, 0);
        world.cells[idx].organism = None;
        assert_eq!(
            evaluate_world(Some(&objective), &world, &mut progress, 40),
            WorldOutcome::Failed(FailureReason::TotalExtinction),
            "total extinction must override an already-cleared outcome, not be masked by it"
        );
    }

    #[test]
    fn empty_species_registry_before_seeding_is_not_treated_as_extinction() {
        let config = SimConfig::default();
        let world = SimWorld::new(42, &config);
        assert!(
            world.species.is_empty(),
            "precondition: no starting palette has been placed yet"
        );
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate_world(None, &world, &mut progress, 40),
            WorldOutcome::Ongoing,
            "an empty grid before any species exists must not read as total extinction"
        );
    }

    #[test]
    fn no_objective_assigned_yet_still_checks_failure_conditions() {
        let mut world = world_with_species(1);
        place(&mut world, 0, 0, SpeciesId(0));
        world.era = 40;
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate_world(None, &world, &mut progress, 40),
            WorldOutcome::Failed(FailureReason::EraBudgetExhausted),
            "failure conditions apply even while task 042's worldgen hasn't assigned an objective"
        );
    }
}
