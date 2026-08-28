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
// Only `Seed` lives here so far. Stress and Cull move down when tasks 145/146
// rework them; there is no reason to move them ahead of the work that changes
// them.

use crate::config::SimConfig;
use crate::sim::ActionBudget;
use crate::world::{Organism, SimWorld, SpeciesId};

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
    if cell.organism.is_some() {
        return None;
    }
    cell.organism = Some(Organism {
        species,
        energy: config.energy.seed_energy,
        born_season: season,
    });
    budget.points_remaining -= config.time.action_costs.seed;
    Some(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::TerrainKind;

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
        assert!(world.get(5, 5).organism.is_none());
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
        assert!(world.get(5, 5).organism.is_none());
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
        assert!(world.get(5, 5).organism.is_some());
        assert_eq!(budget.points_remaining, 2);
    }
}
