// Shared "how a world (re)starts" logic (task 045). `reseed_world`
// (`input.rs`, the `r` key) was the first place this reset was written; the
// world-cleared transition (`screens.rs`) needs the exact same reset, just
// with a different `world_index`/seed source — `start_world` is the one
// function both call, so there's a single source of truth instead of two
// copies that can drift apart.
//
// Deliberate exception to "one module = one `Plugin`" (TECH_DESIGN.md §3.2,
// same rationale as `state.rs`): plain functions only, called from systems
// that live in `input.rs`/`screens.rs`, no `Plugin` or systems of its own.

use abiogenesis::config::SimConfig;
use abiogenesis::objectives::{CurrentObjective, CurrentWorldOutcome, ObjectiveProgress};
#[cfg(test)]
use abiogenesis::objectives::{FailureReason, WorldOutcome};
use abiogenesis::run::RunProgress;
use abiogenesis::sim::{ActionBudget, EraProgress};
use abiogenesis::state::EraState;
use abiogenesis::world::{SimWorld, SpeciesId};
use abiogenesis::worldgen::build_world;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::notebook::{
    MatrixKnowledge, NotebookHasUnseenConfirmation, ObservationLog, PlayerPlacedCells,
};
use crate::ui::{SelectedSpecies, SpliceDraft};

/// Every piece of per-world state a world (re)start resets, bundled into one
/// `SystemParam` (task 054) — `start_world`'s individual `&mut` arguments
/// pushed its callers (`reseed_world`/`screens.rs`'s two functions) past
/// Bevy's ~16-parameter ceiling for a system function the moment a 10th
/// piece of state (`NotebookHasUnseenConfirmation`) joined the list. One
/// bundled param keeps every caller well under that ceiling regardless of
/// how many more reset targets future tasks add.
#[derive(SystemParam)]
pub struct WorldResetParams<'w> {
    pub knowledge: ResMut<'w, MatrixKnowledge>,
    pub log: ResMut<'w, ObservationLog>,
    pub budget: ResMut<'w, ActionBudget>,
    pub selected: ResMut<'w, SelectedSpecies>,
    pub splice_draft: ResMut<'w, SpliceDraft>,
    pub placed: ResMut<'w, PlayerPlacedCells>,
    pub unseen_confirmation: ResMut<'w, NotebookHasUnseenConfirmation>,
    pub objective: ResMut<'w, CurrentObjective>,
    pub objective_progress: ResMut<'w, ObjectiveProgress>,
    pub outcome: ResMut<'w, CurrentWorldOutcome>,
}

/// Rebuilds `world` in place as world `world_index` seeded with `seed`
/// (`worldgen::build_world`, which also generates the new world's
/// `Objective`), and resets every piece of per-world state that would
/// otherwise leak from the world just left: notebook knowledge/log, the
/// action budget, UI selection/draft state, and the objective/outcome
/// tracking (task 040/041). `knowledge` is resized to the *new* world's
/// `active_tags.len()` rather than any fixed constant — worldgen (task 038)
/// makes the active tag count vary with `world_index`, so a stale size would
/// go out of bounds the first time a bigger world is generated.
#[allow(clippy::too_many_arguments)]
pub fn start_world(
    world: &mut SimWorld,
    world_index: u32,
    seed: u64,
    config: &SimConfig,
    bonus_available_species: u32,
    era_progress: &mut EraProgress,
    era_next_state: &mut NextState<EraState>,
    reset: &mut WorldResetParams,
) {
    let (new_world, new_objective) =
        build_world(seed, world_index, config, bonus_available_species);
    *world = new_world;

    era_progress.cancel();
    era_next_state.set(EraState::Observing);

    *reset.knowledge = MatrixKnowledge::new(
        world.active_tags.len(),
        config.notebook.confirmation_threshold,
    );
    reset.log.entries.clear();
    reset.budget.refill(config.time.point_budget_per_era);
    reset.selected.0 = SpeciesId(0);
    *reset.splice_draft = SpliceDraft::default();
    reset.placed.0.clear();
    reset.unseen_confirmation.0 = false;

    *reset.objective = CurrentObjective(Some(new_objective));
    *reset.objective_progress = ObjectiveProgress::default();
    *reset.outcome = CurrentWorldOutcome::default();
}

/// The concrete effect of "World cleared → Continue" (task 045): advances
/// `run_progress` to `world_index + 1`, seeded from the *finishing* world's
/// own RNG (`SimWorld::next_seed`, drawn before `start_world` overwrites
/// `world`), and rebuilds through the same `start_world` every other
/// transition uses. Factored out of `screens.rs::world_cleared_screen_ui` so
/// it's testable without egui or a running `Bevy` `App` — same rationale
/// `menu.rs::start_run` gives for splitting UI from effect.
pub fn advance_to_next_world(
    world: &mut SimWorld,
    run_progress: &mut RunProgress,
    config: &SimConfig,
    era_progress: &mut EraProgress,
    era_next_state: &mut NextState<EraState>,
    reset: &mut WorldResetParams,
) {
    let next_world_index = run_progress.world_index + 1;
    let next_seed = world.next_seed();
    start_world(
        world,
        next_world_index,
        next_seed,
        config,
        run_progress.unlocks.bonus_available_species,
        era_progress,
        era_next_state,
        reset,
    );
    run_progress.world_index = next_world_index;
    run_progress.world_seed = next_seed;
    run_progress.worlds_cleared += 1;
}

/// The concrete effect of "World failed → Retry" (task 051,
/// `FailureReason::TotalExtinction` only — see `objectives.rs`): rebuilds
/// the *same* `world_index` from the *same* `run_progress.world_seed`
/// through `start_world`, an exact do-over of the world just lost rather
/// than a new one. Unlike `advance_to_next_world`, `run_progress` itself
/// isn't mutated — `world_index`/`world_seed`/`worlds_cleared` all stay
/// put, since nothing about the run's progress actually changed.
pub fn retry_world(
    world: &mut SimWorld,
    run_progress: &RunProgress,
    config: &SimConfig,
    era_progress: &mut EraProgress,
    era_next_state: &mut NextState<EraState>,
    reset: &mut WorldResetParams,
) {
    start_world(
        world,
        run_progress.world_index,
        run_progress.world_seed,
        config,
        run_progress.unlocks.bonus_available_species,
        era_progress,
        era_next_state,
        reset,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use abiogenesis::config::SimConfig;
    use abiogenesis::worldgen::{build_world, world_params};
    use bevy::ecs::system::SystemState;

    /// Builds a scratch ECS `World` with every resource `WorldResetParams`
    /// bundles, so `start_world`/`advance_to_next_world`/`retry_world` can be
    /// exercised exactly as the real systems call them — `WorldResetParams`
    /// derives `SystemParam` (task 054, to stay under Bevy's per-system
    /// parameter ceiling), so its fields can no longer be assembled from
    /// plain owned locals the way the pre-054 tuple-of-resources helper did.
    fn resource_world(
        objective: Option<abiogenesis::objectives::Objective>,
        objective_progress: ObjectiveProgress,
        outcome: CurrentWorldOutcome,
    ) -> World {
        let mut ecs_world = World::new();
        ecs_world.insert_resource(MatrixKnowledge::new(5, 3.0));
        ecs_world.insert_resource(ObservationLog::default());
        ecs_world.insert_resource(ActionBudget::default());
        ecs_world.insert_resource(SelectedSpecies(SpeciesId(0)));
        ecs_world.insert_resource(SpliceDraft::default());
        ecs_world.insert_resource(PlayerPlacedCells::default());
        ecs_world.insert_resource(NotebookHasUnseenConfirmation::default());
        ecs_world.insert_resource(CurrentObjective(objective));
        ecs_world.insert_resource(objective_progress);
        ecs_world.insert_resource(outcome);
        ecs_world
    }

    /// Task 045's own acceptance criterion: "advances the run programmatically
    /// for two world-cleared cycles" and reproduces the same sequence a
    /// hand-rolled chain (`build_world` -> `next_seed` -> `build_world`)
    /// would. Exercises `advance_to_next_world` directly — the exact
    /// function the "Continue" button calls — rather than reimplementing the
    /// chain, so a bug in the real transition would actually fail this test.
    #[test]
    fn two_consecutive_world_cleared_transitions_advance_and_match_build_world() {
        let config = SimConfig::default();
        let (mut world, objective) = build_world(123, 0, &config, 0);
        let mut run_progress = RunProgress {
            run_seed: 123,
            world_index: 0,
            world_seed: 123,
            worlds_cleared: 0,
            unlocks: Default::default(),
        };
        let mut era_progress = EraProgress::default();
        let mut era_next_state = NextState::default();
        let mut ecs_world = resource_world(
            Some(objective),
            ObjectiveProgress::default(),
            CurrentWorldOutcome::default(),
        );
        let mut state = SystemState::<WorldResetParams>::new(&mut ecs_world);
        let mut reset = state.get_mut(&mut ecs_world).unwrap();

        // Independently reproduce what the transition *should* produce,
        // without going through `advance_to_next_world` at all.
        let mut reference_world = build_world(123, 0, &config, 0).0;
        let expected_seed_1 = reference_world.next_seed();
        let (expected_world_1, expected_objective_1) = build_world(expected_seed_1, 1, &config, 0);

        advance_to_next_world(
            &mut world,
            &mut run_progress,
            &config,
            &mut era_progress,
            &mut era_next_state,
            &mut reset,
        );

        assert_eq!(run_progress.world_index, 1);
        assert_eq!(run_progress.worlds_cleared, 1);
        assert_eq!(run_progress.world_seed, expected_seed_1);
        assert_eq!(world.active_tags, expected_world_1.active_tags);
        assert_eq!(world.matrix, expected_world_1.matrix);
        assert_eq!(
            world.active_tags.len(),
            world_params(1, &config).active_tag_count as usize,
            "world 1 must actually use world_index 1's difficulty parameters"
        );
        assert_eq!(reset.objective.0, Some(expected_objective_1));
        assert_eq!(reset.objective_progress.consecutive_ticks, 0);
        assert!(!reset.objective_progress.satisfied);
        assert_eq!(
            reset.outcome.0,
            abiogenesis::objectives::WorldOutcome::Ongoing
        );

        // Second consecutive transition, chained off the first — this is
        // the "two cycles" the acceptance criterion asks for.
        reference_world = expected_world_1;
        let expected_seed_2 = reference_world.next_seed();
        let (expected_world_2, expected_objective_2) = build_world(expected_seed_2, 2, &config, 0);

        advance_to_next_world(
            &mut world,
            &mut run_progress,
            &config,
            &mut era_progress,
            &mut era_next_state,
            &mut reset,
        );

        assert_eq!(run_progress.world_index, 2);
        assert_eq!(run_progress.worlds_cleared, 2);
        assert_eq!(world.active_tags, expected_world_2.active_tags);
        assert_eq!(world.matrix, expected_world_2.matrix);
        assert_eq!(reset.objective.0, Some(expected_objective_2));
    }

    /// Task 054's confirmation badge is a per-world notification: its
    /// referent (`MatrixKnowledge`) resets on every world (re)start, so a
    /// badge left lit from the world just abandoned would point at a
    /// hypothesis grid that's already been wiped back to zero evidence.
    /// `start_world` must clear it exactly like it clears `PlayerPlacedCells`.
    #[test]
    fn advancing_to_the_next_world_clears_a_stale_unseen_confirmation_badge() {
        let config = SimConfig::default();
        let (mut world, objective) = build_world(11, 0, &config, 0);
        let mut run_progress = RunProgress {
            run_seed: 11,
            world_index: 0,
            world_seed: 11,
            worlds_cleared: 0,
            unlocks: Default::default(),
        };
        let mut era_progress = EraProgress::default();
        let mut era_next_state = NextState::default();
        let mut ecs_world = resource_world(
            Some(objective),
            ObjectiveProgress::default(),
            CurrentWorldOutcome::default(),
        );
        ecs_world.resource_mut::<NotebookHasUnseenConfirmation>().0 = true;
        let mut state = SystemState::<WorldResetParams>::new(&mut ecs_world);
        let mut reset = state.get_mut(&mut ecs_world).unwrap();

        advance_to_next_world(
            &mut world,
            &mut run_progress,
            &config,
            &mut era_progress,
            &mut era_next_state,
            &mut reset,
        );

        assert!(!reset.unseen_confirmation.0);
    }

    /// `MatrixKnowledge` must be resized to the *new* world's active-tag
    /// count, not left at whatever size it had before — the exact
    /// out-of-bounds risk task 045's brief flagged.
    #[test]
    fn matrix_knowledge_is_resized_to_the_new_worlds_active_tag_count() {
        let config = SimConfig::default();
        let (mut world, objective) = build_world(7, 0, &config, 0);
        let mut run_progress = RunProgress {
            run_seed: 7,
            world_index: 0,
            world_seed: 7,
            worlds_cleared: 0,
            unlocks: Default::default(),
        };
        let mut era_progress = EraProgress::default();
        let mut era_next_state = NextState::default();
        let mut ecs_world = resource_world(
            Some(objective),
            ObjectiveProgress::default(),
            CurrentWorldOutcome::default(),
        );
        let mut state = SystemState::<WorldResetParams>::new(&mut ecs_world);
        let mut reset = state.get_mut(&mut ecs_world).unwrap();

        advance_to_next_world(
            &mut world,
            &mut run_progress,
            &config,
            &mut era_progress,
            &mut era_next_state,
            &mut reset,
        );

        let expected_tags = world_params(1, &config).active_tag_count as usize;
        // `record`/`evidence` index `size * size` — the real out-of-bounds
        // failure mode if `knowledge` were left at the stale size 5.
        reset.knowledge.record(
            abiogenesis::world::TagSlot((expected_tags - 1) as u8),
            abiogenesis::world::TagSlot((expected_tags - 1) as u8),
            1.0,
        );
    }

    /// Task 051's own acceptance criterion: `retry_world` reproduces the
    /// *exact same* world (active tags, matrix, objective) the player just
    /// lost — same `world_index`, same seed — and leaves `run_progress`
    /// untouched, unlike `advance_to_next_world`.
    #[test]
    fn retry_world_rebuilds_the_identical_world_and_leaves_run_progress_untouched() {
        let config = SimConfig::default();
        let (mut world, objective) = build_world(99, 1, &config, 0);
        let run_progress = RunProgress {
            run_seed: 42,
            world_index: 1,
            world_seed: 99,
            worlds_cleared: 3,
            unlocks: Default::default(),
        };
        let mut era_progress = EraProgress::default();
        let mut era_next_state = NextState::default();
        let mut ecs_world = resource_world(
            Some(objective),
            ObjectiveProgress {
                consecutive_ticks: 7,
                satisfied: false,
            },
            CurrentWorldOutcome(WorldOutcome::Failed(FailureReason::TotalExtinction)),
        );
        let mut state = SystemState::<WorldResetParams>::new(&mut ecs_world);
        let mut reset = state.get_mut(&mut ecs_world).unwrap();

        let (expected_world, expected_objective) = build_world(99, 1, &config, 0);

        retry_world(
            &mut world,
            &run_progress,
            &config,
            &mut era_progress,
            &mut era_next_state,
            &mut reset,
        );

        assert_eq!(run_progress.world_index, 1);
        assert_eq!(run_progress.world_seed, 99);
        assert_eq!(run_progress.worlds_cleared, 3);
        assert_eq!(world.active_tags, expected_world.active_tags);
        assert_eq!(world.matrix, expected_world.matrix);
        assert_eq!(reset.objective.0, Some(expected_objective));
        assert_eq!(reset.objective_progress.consecutive_ticks, 0);
        assert_eq!(reset.outcome.0, WorldOutcome::Ongoing);
    }
}
