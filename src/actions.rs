// The world-level effect of the player's actions (GDD §6), separated from
// the Bevy input systems that trigger them (`input.rs`).
//
// `input.rs` owns everything about *how* an action is invoked — mouse
// picking, the view-mode gate, egui focus, the transient placement indicator
// — and this module owns *what it does to the world*. The split exists so
// the effect is reachable without a window: `tests/` and `examples/` see only
// this crate, never the binary's modules, so the headless two-bot survey
// (`examples/two_bot_survey.rs`, task 134) could not otherwise model a player
// acting at all.
//
// Task 171 extends this to `Cull` and `Splice` (mirroring `attempt_seed`'s
// task-134 extraction) so the bot-vs-bot harness in `examples/` can act with
// all three player tools rather than `Seed` alone. `Stress` stays in
// `input.rs` — no bot policy needs it yet.

use crate::config::SimConfig;
use crate::sim::{cull_knockout_observations, ActionBudget, AdjacencyObserved};
use crate::world::{
    draw_species_name, net_self_interaction, Population, SimWorld, SpeciesId, TagSlot,
};

/// Every rejection rule `seed_organism_on_click` enforces once a clicked
/// cell is known: affordable, placeable (task 067 — not Sea or a mountain
/// peak), and empty. Pulled out of the system so it's unit-testable without
/// the mouse/window/camera harness `clicked_cell` needs. Silent no-op on
/// every rejection, matching the rest of the action gates (no
/// rejected-action feedback mechanism exists yet); returns the cell index it
/// placed on, or `None` if it placed nothing — the index is what `input.rs`
/// needs for `PlayerPlacedCells`, which stays up there since it is notebook
/// bookkeeping rather than simulation state.
pub fn attempt_seed(
    world: &mut SimWorld,
    config: &SimConfig,
    budget: &mut ActionBudget,
    species: SpeciesId,
    x: usize,
    y: usize,
) -> Option<usize> {
    if budget.points_remaining < config.time.action_costs.seed {
        return None;
    }
    if !world.is_placeable_for(x, y, species) {
        return None;
    }
    let index = world.index(x, y);
    let season = world.season;
    let cell = world.get_mut(x, y);
    if cell.population.is_some() {
        return None;
    }
    cell.population = Some(Population {
        species,
        count: 1,
        energy: config.energy.seed_energy,
        born_season: season,
        blocked: false,
    });
    budget.points_remaining -= config.time.action_costs.seed;
    Some(index)
}

/// The world-level effect of a `Cull` click (GDD §6), mirroring
/// `input.rs`'s `cull_on_click` minus the Bevy-input picking and the
/// `PlayerPlacedCells`/`world_touched` bookkeeping a caller with access to
/// those resources still owns. Occupancy is checked before the budget, same
/// asymmetry as the original: clicking an empty cell costs nothing. Returns
/// the knockout observations (task 146) `cull_knockout_observations`
/// computes before the organism is removed, or `None` on a rejected cull —
/// the caller emits them as `AdjacencyObserved` messages (in `input.rs`) or
/// feeds them straight into `MatrixKnowledge` (in a headless caller like
/// `examples/two_bot_survey.rs`).
pub fn attempt_cull(
    world: &mut SimWorld,
    config: &SimConfig,
    budget: &mut ActionBudget,
    x: usize,
    y: usize,
) -> Option<Vec<AdjacencyObserved>> {
    let index = world.index(x, y);
    world.cells[index].population?;
    if budget.points_remaining < config.time.action_costs.cull {
        return None;
    }
    let observations = cull_knockout_observations(world, config, x, y);
    world.get_mut(x, y).population = None;
    budget.points_remaining -= config.time.action_costs.cull;
    Some(observations)
}

/// The edit a `Splice` applies to a cloned source species — the crate-level
/// (no `bevy_egui`) counterpart to `ui::SpliceEditChoice`, which additionally
/// tracks in-progress/incomplete UI selections this module has no use for.
/// Mirrors `apply_splice`'s three branches exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpliceEdit {
    SwapTag { old: TagSlot, new: TagSlot },
    AddTag { tag: TagSlot },
    ShiftTempOptimum { warmer: bool },
}

/// The world-level effect of applying a `Splice` (GDD §6), mirroring
/// `input.rs`'s `apply_splice` minus the `SpliceDraft`/`ObservationLog`/
/// `world_touched` bookkeeping a caller with access to those resources still
/// owns. Silently rejects (returns `None`, no partial application) on: an
/// unknown `source`, a `SwapTag` naming a tag the source doesn't carry, an
/// `AddTag` past the 3-tag cap, a resulting tag set with nonzero
/// `net_self_interaction`, or insufficient budget — same guards
/// `apply_splice` enforces, checked in the same order so a caller that
/// reimplements the UI-side pre-checks (task 147's confirmed-traits gate,
/// for instance) never disagrees with what this actually allows.
pub fn attempt_splice(
    world: &mut SimWorld,
    config: &SimConfig,
    budget: &mut ActionBudget,
    splice_cost: u32,
    source: SpeciesId,
    edit: SpliceEdit,
) -> Option<SpeciesId> {
    let source_species = world.species.get(source.0 as usize)?;
    let mut new_species = source_species.clone();
    new_species.name = draw_species_name(world);
    match edit {
        SpliceEdit::SwapTag { old, new } => {
            let pos = new_species.tags.iter().position(|&tag| tag == old)?;
            new_species.tags[pos] = new;
            if net_self_interaction(&world.matrix, &new_species.tags) != 0 {
                return None;
            }
        }
        SpliceEdit::AddTag { tag } => {
            if new_species.tags.len() >= 3 {
                return None;
            }
            new_species.tags.push(tag);
            if net_self_interaction(&world.matrix, &new_species.tags) != 0 {
                return None;
            }
        }
        SpliceEdit::ShiftTempOptimum { warmer } => {
            let delta = if warmer {
                config.energy.splice_temp_shift
            } else {
                -config.energy.splice_temp_shift
            };
            new_species.temp_optimum = (new_species.temp_optimum + delta).clamp(0.0, 1.0);
        }
    }
    if budget.points_remaining < splice_cost {
        return None;
    }
    let new_species_id = world.push_species(new_species);
    world.spliced_species.push(new_species_id);
    budget.points_remaining -= splice_cost;
    Some(new_species_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Metabolism, Species, TagId, TagMatrix, TerrainKind};

    #[test]
    fn attempt_cull_removes_the_organism_and_reports_knockout_observations() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.active_tags = vec![TagId(0), TagId(1)];
        world.matrix = TagMatrix::from_values(2, vec![0, 1, 0, 0]);
        world.conditional_tags = Vec::new();
        world.push_species(Species {
            name: "Exerter".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagSlot(0)],
        });
        world.push_species(Species {
            name: "Receiver".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagSlot(1)],
        });
        let mut budget = ActionBudget {
            points_remaining: 3,
        };
        let mut seed = |x: usize, y: usize, species: SpeciesId| {
            attempt_seed(&mut world, &config, &mut budget, species, x, y).unwrap();
        };
        seed(5, 5, SpeciesId(0));
        seed(5, 6, SpeciesId(1));

        let observations = attempt_cull(&mut world, &config, &mut budget, 5, 5)
            .expect("occupied cell should cull");

        assert!(world.get(5, 5).population.is_none());
        assert!(
            observations
                .iter()
                .any(|o| o.exerter_tag == TagSlot(0) && o.receiver_tag == TagSlot(1)),
            "the culled organism's tag-0 -> tag-1 effect on its living neighbour must be reported"
        );
        assert_eq!(
            budget.points_remaining,
            3 - 2 * config.time.action_costs.seed - config.time.action_costs.cull
        );
    }

    #[test]
    fn attempt_cull_is_a_free_no_op_on_an_empty_cell() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let mut budget = ActionBudget {
            points_remaining: 3,
        };

        let result = attempt_cull(&mut world, &config, &mut budget, 5, 5);

        assert!(result.is_none());
        assert_eq!(
            budget.points_remaining, 3,
            "an empty-cell cull must cost nothing"
        );
    }

    fn world_with_one_taggable_species() -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.active_tags = vec![TagId(0), TagId(1)];
        world.matrix = TagMatrix::from_values(2, vec![0, 0, 0, 0]);
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagSlot(0)],
        });
        (world, config)
    }

    #[test]
    fn attempt_splice_appends_a_new_species_and_leaves_the_source_untouched() {
        let (mut world, config) = world_with_one_taggable_species();
        let mut budget = ActionBudget {
            points_remaining: 3,
        };

        let new_id = attempt_splice(
            &mut world,
            &config,
            &mut budget,
            2,
            SpeciesId(0),
            SpliceEdit::SwapTag {
                old: TagSlot(0),
                new: TagSlot(1),
            },
        )
        .expect("neutral matrix should allow the swap");

        assert_eq!(world.species.len(), 2);
        assert_eq!(world.species[0].tags, vec![TagSlot(0)]);
        assert_eq!(world.species[new_id.0 as usize].tags, vec![TagSlot(1)]);
        assert_eq!(budget.points_remaining, 1);
    }

    #[test]
    fn attempt_splice_rejects_a_self_reinforcing_result_without_spending_budget() {
        let (mut world, mut config) = world_with_one_taggable_species();
        world.matrix = TagMatrix::from_values(2, vec![0, 1, 1, 0]);
        world.active_tags = vec![TagId(0), TagId(1)];
        config.energy.splice_temp_shift = 0.1;
        let mut budget = ActionBudget {
            points_remaining: 3,
        };

        let result = attempt_splice(
            &mut world,
            &config,
            &mut budget,
            2,
            SpeciesId(0),
            SpliceEdit::AddTag { tag: TagSlot(1) },
        );

        assert!(result.is_none());
        assert_eq!(world.species.len(), 1);
        assert_eq!(budget.points_remaining, 3);
    }

    #[test]
    fn attempt_seed_rejects_an_unplaceable_cell() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.get_mut(5, 5).terrain = TerrainKind::Sea;
        let mut budget = ActionBudget {
            points_remaining: 3,
        };

        let placed = attempt_seed(&mut world, &config, &mut budget, SpeciesId(0), 5, 5);

        assert!(placed.is_none());
        assert!(world.get(5, 5).population.is_none());
        assert_eq!(
            budget.points_remaining, 3,
            "budget must not be spent on a rejected placement"
        );
    }

    #[test]
    fn attempt_seed_rejects_an_unplaceable_peak_even_though_the_terrain_is_mountain() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let cell = world.get_mut(5, 5);
        cell.terrain = TerrainKind::Mountain;
        cell.is_peak = true;
        let mut budget = ActionBudget {
            points_remaining: 3,
        };

        let placed = attempt_seed(&mut world, &config, &mut budget, SpeciesId(0), 5, 5);

        assert!(placed.is_none());
        assert!(world.get(5, 5).population.is_none());
    }

    #[test]
    fn attempt_seed_succeeds_on_ordinary_placeable_terrain() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.get_mut(5, 5).terrain = TerrainKind::Plain;
        let mut budget = ActionBudget {
            points_remaining: 3,
        };

        let placed = attempt_seed(&mut world, &config, &mut budget, SpeciesId(0), 5, 5);

        assert_eq!(placed, Some(world.index(5, 5)));
        assert!(world.get(5, 5).population.is_some());
        assert_eq!(budget.points_remaining, 2);
    }
}
