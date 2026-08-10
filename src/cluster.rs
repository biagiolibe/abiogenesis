// Overview mode's per-species cluster heatmap (task 076,
// `redesign/abiogenesis-two-tier-view.md`). Pure and headless (TECH_DESIGN.md
// §5): only reads `world::SimWorld`, no `bevy::render`/`bevy_egui`, so it's
// unit-testable without spinning up a Bevy `App`, matching `sim`/`world`'s
// own testing pattern.

use crate::config::SimConfig;
use crate::world::SimWorld;

/// Local population density for each occupied cell (0.0 for empty cells):
/// the 8-connected (Moore, `SimWorld::moore_neighbours` — `GridConfig::
/// neighborhood_size`'s fixed-by-design value) connected component of
/// same-species occupied cells a cell belongs to is its cluster; a cluster's
/// density is its occupied-cell count normalized against
/// `ClusterConfig::density_saturation`, clamped to `[0, 1]`. This is a
/// population-mass reading, not a compactness one: a large, established
/// colony saturates toward `1.0` (brightest) while a lone organism — a
/// one-cell cluster — sits near the bottom of the range, clearly visible
/// (task doc: "an isolated organism ... stays visibly distinct") but never
/// mistaken for the grid's real hot spot. A compactness formula (occupied
/// cells / bounding-box area) was considered and rejected: it would score a
/// single organism as maximally "dense" (a 1x1 box is always 100% full),
/// making a stray read brighter than a sprawling thriving colony — the
/// opposite of what a population heatmap should show.
///
/// Every cell in a cluster shares that cluster's density value; combined with
/// `render.rs`'s existing per-cell sprite (one sprite per grid cell, unchanged
/// from Detail mode), coloring each occupied cell by its cluster's density
/// makes the sprite union read as a single blob whose shape is the cluster's
/// real footprint, not a bounding box or a fixed-size tile.
///
/// Cell occupancy is single-organism (`Cell::organism: Option<Organism>`), so
/// two clusters of different species never need to agree on one cell's
/// color: each cell always renders as whichever species (if any) actually
/// occupies it, exactly like Detail mode. That's this task's z-order/blend
/// rule for overlapping/adjacent clusters — there is no blend, because
/// there's never a genuine per-cell conflict to resolve.
pub fn compute_cluster_density(world: &SimWorld, config: &SimConfig) -> Vec<f32> {
    let mut density = vec![0.0_f32; world.cells.len()];
    let mut visited = vec![false; world.cells.len()];
    let mut stack = Vec::new();
    let mut members = Vec::new();

    for start in 0..world.cells.len() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let Some(organism) = world.cells[start].organism else {
            continue;
        };

        members.clear();
        stack.clear();
        stack.push(start);
        members.push(start);
        while let Some(idx) = stack.pop() {
            let x = idx % world.width;
            let y = idx / world.width;
            for neighbour in world.moore_neighbours(x, y) {
                if visited[neighbour] {
                    continue;
                }
                if world.cells[neighbour]
                    .organism
                    .is_some_and(|o| o.species == organism.species)
                {
                    visited[neighbour] = true;
                    members.push(neighbour);
                    stack.push(neighbour);
                }
            }
        }

        let cluster_density =
            (members.len() as f32 / config.cluster.density_saturation).clamp(0.0, 1.0);
        for &idx in &members {
            density[idx] = cluster_density;
        }
    }

    density
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Organism, SpeciesId};

    fn place(world: &mut SimWorld, x: usize, y: usize, species: u8) {
        let idx = world.index(x, y);
        world.cells[idx].organism = Some(Organism {
            species: SpeciesId(species),
            energy: 5.0,
            born_era: 0,
        });
    }

    #[test]
    fn empty_grid_has_no_density() {
        let config = SimConfig::default();
        let world = SimWorld::new(1, &config);
        let density = compute_cluster_density(&world, &config);
        assert!(density.iter().all(|&d| d == 0.0));
    }

    #[test]
    fn isolated_organism_is_visible_but_far_from_saturated() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(1, &config);
        place(&mut world, 5, 5, 0);
        let density = compute_cluster_density(&world, &config);
        let isolated = density[world.index(5, 5)];
        assert!(isolated > 0.0, "a lone organism must still be visible");
        assert!(
            isolated < 1.0,
            "a lone organism must not read as a saturated hot spot"
        );
    }

    #[test]
    fn a_large_cluster_reads_brighter_than_a_lone_organism() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(1, &config);
        place(&mut world, 0, 0, 0);
        let lone = compute_cluster_density(&world, &config)[world.index(0, 0)];

        // A solid block comfortably past `density_saturation` (20.0 by
        // default) so it's pinned at the ceiling regardless of exact tuning.
        for y in 10..15 {
            for x in 10..15 {
                place(&mut world, x, y, 1);
            }
        }
        let colony = compute_cluster_density(&world, &config)[world.index(12, 12)];

        assert!(
            colony > lone,
            "an established colony must render brighter than a single stray organism"
        );
        assert_eq!(colony, 1.0);
    }

    #[test]
    fn cluster_density_saturates_at_the_configured_cell_count() {
        let mut config = SimConfig::default();
        config.cluster.density_saturation = 4.0;
        let mut world = SimWorld::new(1, &config);
        place(&mut world, 5, 5, 0);
        place(&mut world, 6, 5, 0);
        let density = compute_cluster_density(&world, &config);
        assert_eq!(density[world.index(5, 5)], 0.5);
    }

    #[test]
    fn different_species_never_merge_into_one_cluster() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(1, &config);
        place(&mut world, 5, 5, 0);
        place(&mut world, 6, 5, 1);
        let density = compute_cluster_density(&world, &config);
        // Each is its own one-cell cluster of a different species — same
        // (small) density, but tracked independently, not merged.
        assert_eq!(density[world.index(5, 5)], density[world.index(6, 5)]);
    }
}
