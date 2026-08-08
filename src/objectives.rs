// Per-world objectives (task 040, GDD §8): explicit requirements a world
// poses, checked tick by tick against `SimWorld`'s own observable state.
// Procedural generation of *which* objective a given world gets is task
// 042 — this module only defines the type and the evaluation engine, and
// is deliberately testable against a hand-built `SimWorld`, independent of
// worldgen.

use bevy::ecs::system::SystemParam;
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

/// The current world's objective sequence (task 059, GDD §8/§9): worlds pose
/// 2-3 objectives in order, not just one — clearing `objectives[index]`
/// advances `index` rather than ending the world, until the last one clears.
/// Empty until task 042/059's worldgen assigns a sequence (or a test sets one
/// directly) — the driving system below is a no-op while empty.
#[derive(Resource, Debug, Clone, Default)]
pub struct CurrentObjective {
    pub objectives: Vec<Objective>,
    pub index: usize,
}

impl CurrentObjective {
    pub fn new(objectives: Vec<Objective>) -> Self {
        Self {
            objectives,
            index: 0,
        }
    }

    /// The objective currently being evaluated, `None` once every objective
    /// in the sequence has cleared (shouldn't normally be observed — the
    /// world transitions to `WorldCleared` the same tick the last one does —
    /// but kept total rather than panicking on an out-of-range index).
    pub fn current(&self) -> Option<&Objective> {
        self.objectives.get(self.index)
    }

    /// Whether `index` points at the last objective in the sequence — the
    /// only one whose clearing should end the world instead of advancing.
    pub fn is_last(&self) -> bool {
        self.index + 1 >= self.objectives.len()
    }

    pub fn total(&self) -> usize {
        self.objectives.len()
    }
}

/// A world advanced from one objective to the next in its sequence (task
/// 059) without ending — `index` is the newly-current objective's position.
/// Consumed by `notebook.rs::record_events` to log the transition, the same
/// way it already consumes `sim.rs`'s `SpeciesExtinct`/`OrganismDied`, rather
/// than `objectives.rs` reaching into `ObservationLog` directly.
#[derive(Debug, Clone, Copy, Message)]
pub struct ObjectiveAdvanced {
    pub index: usize,
    pub objective: Objective,
}

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

/// True once every living organism on the grid has died. Guarded by
/// `SimWorld::ever_populated` (task 050): worlds start with nothing placed
/// — the player seeds them via `Seed` — so an empty grid on its own doesn't
/// mean extinction, only an empty grid *after* life has actually existed
/// here does. A player who never seeds anything doesn't dodge failure
/// forever this way — the world still fails once `WorldParams::era_budget`
/// runs out (`is_era_budget_exhausted`), just via that condition instead.
fn is_total_extinction(world: &SimWorld) -> bool {
    world.ever_populated && world.cells.iter().all(|cell| cell.organism.is_none())
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
/// `zone`. Checked against the zone's fixed geometry (`SimWorld::toxic_zone`,
/// task 047), not `Cell::toxicity` — that scalar diffuses toward neighbours
/// every tick (`SimWorld::diffuse_environment`) and, given enough ticks,
/// stops meaning "is in the zone" at all.
fn species_present_in_zone(world: &SimWorld, species: SpeciesId, zone: ZoneKind) -> bool {
    world.cells.iter().enumerate().any(|(idx, cell)| {
        let x = idx % world.width;
        let y = idx / world.width;
        cell.organism
            .is_some_and(|organism| organism.species == species)
            && cell_in_zone(world, x, y, zone)
    })
}

fn cell_in_zone(world: &SimWorld, x: usize, y: usize, zone: ZoneKind) -> bool {
    match zone {
        ZoneKind::Toxic => world.toxic_zone.contains(x, y),
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
            .add_message::<ObjectiveAdvanced>()
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
/// yet (an empty `CurrentObjective`, before task 042/059's worldgen wires a
/// sequence in): `evaluate_world` still checks failure conditions in that case.
/// Evaluates the current objective against `world`'s present state and
/// applies whatever state transition the result implies. Called once per
/// simulated tick — shared by `evaluate_current_objective` (the
/// `FixedUpdate`/`Advancing` system driving auto-play eras) and `input.rs`'s
/// `single_tick` (the `s` key's manual step) so a world can only clear or
/// fail on ticks that actually happened, regardless of which key drove them.
///
/// Bundled into one `SystemParam` (mirrors `run_flow.rs`'s `WorldResetParams`,
/// task 054): `single_tick` already sits close to Bevy's per-system parameter
/// ceiling on its own tick-advancement/message-writer parameters, and task
/// 059 adding `ObjectiveAdvanced`'s `MessageWriter` on top of the existing
/// five objective/outcome fields pushed it over — bundling here is what lets
/// both `evaluate_current_objective` and `single_tick` share this logic
/// without either exceeding the ceiling.
#[derive(SystemParam)]
pub struct ObjectiveOutcomeParams<'w> {
    pub objective: ResMut<'w, CurrentObjective>,
    pub progress: ResMut<'w, ObjectiveProgress>,
    pub outcome: ResMut<'w, CurrentWorldOutcome>,
    pub run_progress: Res<'w, RunProgress>,
    pub meta: ResMut<'w, MetaProgress>,
    pub next_game_state: ResMut<'w, NextState<GameState>>,
    pub advanced: MessageWriter<'w, ObjectiveAdvanced>,
}

pub fn apply_tick_outcome(
    world: &SimWorld,
    config: &SimConfig,
    params: &mut ObjectiveOutcomeParams,
) {
    let era_budget = world_params(params.run_progress.world_index, config).era_budget;
    let previous_outcome = params.outcome.0;
    let new_outcome = evaluate_world(
        params.objective.current(),
        world,
        &mut params.progress,
        era_budget,
    );
    params.outcome.0 = new_outcome;

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
        // Total extinction ends this world, not the run (task 051): a
        // player who loses their only organisms early shouldn't have the
        // whole run end over it — `WorldFailed` offers a retry of the exact
        // same world instead. `worlds_cleared` is untouched, so `meta`
        // doesn't absorb anything here; the run isn't over.
        WorldOutcome::Failed(FailureReason::TotalExtinction) => {
            params.next_game_state.set(GameState::WorldFailed);
        }
        // Running out the era budget without meeting the objective is a
        // real, skill-based run-ending failure — unchanged from before 051.
        WorldOutcome::Failed(FailureReason::EraBudgetExhausted) => {
            params.meta.absorb(params.run_progress.worlds_cleared);
            params.next_game_state.set(GameState::Defeat);
        }
        // Clearing the *last* objective in the sequence ends the world, same
        // as before task 059. Clearing an earlier one (task 059) advances to
        // the next objective with fresh progress instead, and resets
        // `outcome` back to `Ongoing` so the `previous_outcome` guard above
        // can still catch the *next* Ongoing -> {Failed, Cleared} edge —
        // without this the world would look permanently "already concluded"
        // and the final objective's own clearing would never fire.
        WorldOutcome::Cleared if params.objective.is_last() => {
            params.next_game_state.set(GameState::WorldCleared);
        }
        WorldOutcome::Cleared => {
            params.objective.index += 1;
            *params.progress = ObjectiveProgress::default();
            params.outcome.0 = WorldOutcome::Ongoing;
            let index = params.objective.index;
            params.advanced.write(ObjectiveAdvanced {
                index,
                objective: params.objective.objectives[index],
            });
        }
        WorldOutcome::Ongoing => {}
    }
}

fn evaluate_current_objective(
    world: Res<SimWorld>,
    config: Res<SimConfig>,
    mut params: ObjectiveOutcomeParams,
) {
    apply_tick_outcome(&world, &config, &mut params);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimConfig;
    use crate::world::{Cell, Metabolism, Organism, Species, ToxicZoneBounds};

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
        // Mirrors what a real placement (`Seed`, reproduction) does to
        // `SimWorld::ever_populated` in `sim::step` — hand-built test worlds
        // don't go through `step`, so this test helper sets it directly.
        world.ever_populated = true;
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
        world.toxic_zone = ToxicZoneBounds {
            x0: 0,
            y0: 0,
            width: 1,
            height: 1,
        };
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
        // The zone is a single cell in the bottom-right corner; the organism
        // sits at (0, 0), outside it.
        world.toxic_zone = ToxicZoneBounds {
            x0: world.width - 1,
            y0: world.height - 1,
            width: 1,
            height: 1,
        };
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

    /// Task 047's regression: `diffuse_environment` blends `toxicity`
    /// toward neighbours every tick with no floor pinning cells outside the
    /// zone back to `0.0` — given enough ticks it leaks well past the
    /// zone's actual bounds. `SurviveIn` must still not be satisfiable by an
    /// organism that only ever sat in the (now slightly toxic) clean corner,
    /// far from where the zone actually is.
    #[test]
    fn diffused_toxicity_outside_the_zone_does_not_satisfy_survive_in() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.species.push(Species {
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        place(&mut world, 0, 0, SpeciesId(0));

        // Task 066: world construction now generates terrain and places the
        // toxic zone somewhere derived from it, which could in principle put
        // real toxicity near (0, 0). This test isolates diffusion leakage
        // specifically, so it resets to a known, fixed 1x1 zone far from
        // (0, 0) rather than relying on incidental placement.
        for cell in world.cells.iter_mut() {
            cell.toxicity = 0.0;
        }
        world.toxic_zone = ToxicZoneBounds {
            x0: world.width - 1,
            y0: world.height - 1,
            width: 1,
            height: 1,
        };
        let corner = world.index(world.width - 1, world.height - 1);
        world.cells[corner].toxicity = config.environment.toxic_zone_value;

        // Diffusion only, not a full `sim::step` — this isolates the effect
        // under test (toxicity leaking via diffusion) from organism
        // dynamics (growth/reproduction) that would otherwise risk
        // legitimately placing a species-0 organism inside the real zone
        // and confounding the assertion below.
        for _ in 0..500 {
            world.scratch.copy_from_slice(&world.cells);
            world.diffuse_environment(&config);
            std::mem::swap(&mut world.cells, &mut world.scratch);
        }

        let idx = world.index(0, 0);
        assert!(
            world.cells[idx].toxicity > 0.0,
            "diffusion should have leaked some toxicity to (0, 0) after 500 ticks — \
             otherwise this test isn't actually exercising the bug it's meant to catch"
        );
        assert!(
            !world.toxic_zone.contains(0, 0),
            "(0, 0) must still be outside the zone's real geometry"
        );

        let objective = Objective::SurviveIn {
            species: SpeciesId(0),
            zone: ZoneKind::Toxic,
            ticks: 1,
        };
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate(&objective, &world, &mut progress),
            WorldOutcome::Ongoing,
            "an organism that never entered the zone's real bounds must not satisfy SurviveIn, \
             even though diffusion has raised its cell's toxicity above 0.0"
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

    /// Task 050's regression: species can be generated (`world.species`
    /// non-empty, e.g. right after `worldgen::generate_starting_palette`)
    /// without anything ever being placed on the grid — worlds no longer
    /// auto-place organisms, the player seeds them via `Seed`. The old
    /// guard (`!world.species.is_empty()`) would have failed the world
    /// instantly here; `ever_populated` must stay `false` until an organism
    /// actually lands.
    #[test]
    fn an_unseeded_world_with_species_defined_is_not_treated_as_extinction() {
        let world = world_with_species(2);
        assert!(
            !world.ever_populated,
            "precondition: nothing has been placed yet"
        );
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate_world(None, &world, &mut progress, 40),
            WorldOutcome::Ongoing,
            "species existing without any having ever been placed must not read as total extinction"
        );
    }

    /// Once `ever_populated` is `true` (life has actually existed), an
    /// empty grid goes back to failing the world normally — the guard only
    /// covers "hasn't started yet," not "started and then died out."
    #[test]
    fn a_world_that_was_populated_and_then_emptied_still_fails() {
        let mut world = world_with_species(1);
        world.ever_populated = true;
        let mut progress = ObjectiveProgress::default();

        assert_eq!(
            evaluate_world(None, &world, &mut progress, 40),
            WorldOutcome::Failed(FailureReason::TotalExtinction),
            "an empty grid after life has existed must still fail the world"
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
