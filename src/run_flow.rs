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
use abiogenesis::run::RunProgress;
use abiogenesis::sim::{ActionBudget, EraProgress};
use abiogenesis::state::EraState;
use abiogenesis::world::{SimWorld, SpeciesId};
use abiogenesis::worldgen::build_world;
use bevy::prelude::*;

use crate::notebook::{MatrixKnowledge, ObservationLog, PlayerPlacedCells};
use crate::ui::{SelectedSpecies, SpliceDraft};

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
    knowledge: &mut MatrixKnowledge,
    log: &mut ObservationLog,
    budget: &mut ActionBudget,
    selected: &mut SelectedSpecies,
    splice_draft: &mut SpliceDraft,
    placed: &mut PlayerPlacedCells,
    objective: &mut CurrentObjective,
    objective_progress: &mut ObjectiveProgress,
    outcome: &mut CurrentWorldOutcome,
) {
    let (new_world, new_objective) =
        build_world(seed, world_index, config, bonus_available_species);
    *world = new_world;

    era_progress.cancel();
    era_next_state.set(EraState::Observing);

    *knowledge = MatrixKnowledge::new(
        world.active_tags.len(),
        config.notebook.confirmation_threshold,
    );
    log.entries.clear();
    budget.refill(config.time.point_budget_per_era);
    selected.0 = SpeciesId(0);
    *splice_draft = SpliceDraft::default();
    placed.0.clear();

    *objective = CurrentObjective(Some(new_objective));
    *objective_progress = ObjectiveProgress::default();
    *outcome = CurrentWorldOutcome::default();
}

/// The concrete effect of "World cleared → Continue" (task 045): advances
/// `run_progress` to `world_index + 1`, seeded from the *finishing* world's
/// own RNG (`SimWorld::next_seed`, drawn before `start_world` overwrites
/// `world`), and rebuilds through the same `start_world` every other
/// transition uses. Factored out of `screens.rs::world_cleared_screen_ui` so
/// it's testable without egui or a running `Bevy` `App` — same rationale
/// `menu.rs::start_run` gives for splitting UI from effect.
#[allow(clippy::too_many_arguments)]
pub fn advance_to_next_world(
    world: &mut SimWorld,
    run_progress: &mut RunProgress,
    config: &SimConfig,
    era_progress: &mut EraProgress,
    era_next_state: &mut NextState<EraState>,
    knowledge: &mut MatrixKnowledge,
    log: &mut ObservationLog,
    budget: &mut ActionBudget,
    selected: &mut SelectedSpecies,
    splice_draft: &mut SpliceDraft,
    placed: &mut PlayerPlacedCells,
    objective: &mut CurrentObjective,
    objective_progress: &mut ObjectiveProgress,
    outcome: &mut CurrentWorldOutcome,
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
        knowledge,
        log,
        budget,
        selected,
        splice_draft,
        placed,
        objective,
        objective_progress,
        outcome,
    );
    run_progress.world_index = next_world_index;
    run_progress.world_seed = next_seed;
    run_progress.worlds_cleared += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use abiogenesis::config::SimConfig;
    use abiogenesis::worldgen::{build_world, world_params};

    fn fresh_resources() -> (
        EraProgress,
        NextState<EraState>,
        MatrixKnowledge,
        ObservationLog,
        ActionBudget,
        SelectedSpecies,
        SpliceDraft,
        PlayerPlacedCells,
    ) {
        (
            EraProgress::default(),
            NextState::default(),
            MatrixKnowledge::new(5, 3.0),
            ObservationLog::default(),
            ActionBudget::default(),
            SelectedSpecies(SpeciesId(0)),
            SpliceDraft::default(),
            PlayerPlacedCells::default(),
        )
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
        let (
            mut era_progress,
            mut era_next_state,
            mut knowledge,
            mut log,
            mut budget,
            mut selected,
            mut splice_draft,
            mut placed,
        ) = fresh_resources();
        let mut current_objective = CurrentObjective(Some(objective));
        let mut objective_progress = ObjectiveProgress::default();
        let mut outcome = CurrentWorldOutcome::default();

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
            &mut knowledge,
            &mut log,
            &mut budget,
            &mut selected,
            &mut splice_draft,
            &mut placed,
            &mut current_objective,
            &mut objective_progress,
            &mut outcome,
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
        assert_eq!(current_objective.0, Some(expected_objective_1));
        assert_eq!(objective_progress.consecutive_ticks, 0);
        assert!(!objective_progress.satisfied);
        assert_eq!(outcome.0, abiogenesis::objectives::WorldOutcome::Ongoing);

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
            &mut knowledge,
            &mut log,
            &mut budget,
            &mut selected,
            &mut splice_draft,
            &mut placed,
            &mut current_objective,
            &mut objective_progress,
            &mut outcome,
        );

        assert_eq!(run_progress.world_index, 2);
        assert_eq!(run_progress.worlds_cleared, 2);
        assert_eq!(world.active_tags, expected_world_2.active_tags);
        assert_eq!(world.matrix, expected_world_2.matrix);
        assert_eq!(current_objective.0, Some(expected_objective_2));
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
        let (
            mut era_progress,
            mut era_next_state,
            mut knowledge,
            mut log,
            mut budget,
            mut selected,
            mut splice_draft,
            mut placed,
        ) = fresh_resources();
        let mut current_objective = CurrentObjective(Some(objective));
        let mut objective_progress = ObjectiveProgress::default();
        let mut outcome = CurrentWorldOutcome::default();

        advance_to_next_world(
            &mut world,
            &mut run_progress,
            &config,
            &mut era_progress,
            &mut era_next_state,
            &mut knowledge,
            &mut log,
            &mut budget,
            &mut selected,
            &mut splice_draft,
            &mut placed,
            &mut current_objective,
            &mut objective_progress,
            &mut outcome,
        );

        let expected_tags = world_params(1, &config).active_tag_count as usize;
        // `record`/`evidence` index `size * size` — the real out-of-bounds
        // failure mode if `knowledge` were left at the stale size 5.
        knowledge.record(
            abiogenesis::world::TagSlot((expected_tags - 1) as u8),
            abiogenesis::world::TagSlot((expected_tags - 1) as u8),
            1.0,
        );
    }
}
