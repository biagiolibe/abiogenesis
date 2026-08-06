// Shared "how a world (re)starts" logic (task 045). `reseed_world`
// (`input.rs`, the `r` key) was the first place this reset was written; the
// world-cleared transition (`screens.rs`) needs the exact same reset, just
// with a different `world_index`/seed source — `start_world` is the one
// function both call, so there's a single source of truth instead of two
// copies that can drift apart.

use abiogenesis::config::SimConfig;
use abiogenesis::objectives::{CurrentObjective, CurrentWorldOutcome, ObjectiveProgress};
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
