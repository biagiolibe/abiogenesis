// Overview mode's per-species cluster heatmap (task 076,
// `redesign/abiogenesis-two-tier-view.md`; blob shape corrected by task
// 078). Pure and headless (TECH_DESIGN.md §5): only reads `world::SimWorld`,
// no `bevy::render`/`bevy_egui`, so it's unit-testable without spinning up a
// Bevy `App`, matching `sim`/`world`'s own testing pattern.

use crate::config::{ClusterConfig, SimConfig};
use crate::world::{SimWorld, SpeciesId};

/// The result of clustering: for every grid cell, which species' Overview
/// blob (if any) claims it for rendering, and that blob's shared density.
/// `species`/`density` are always in lockstep — `species[idx].is_some()`
/// iff `density[idx] > 0.0` — kept as two parallel `Vec`s rather than one
/// `Vec<Option<(SpeciesId, f32)>>` since `render.rs` reads them from two
/// separate `Sprite` fields (hue, lightness) and a struct-of-arrays avoids
/// an `Option::unwrap` at every read site.
///
/// Task 078: a blob's cells are no longer identical to a cluster's literal
/// occupied-cell set. `species`/`density` cover the cluster's *rendered*
/// footprint (real cells, interior holes filled in, then eroded smaller —
/// see `compute_cluster_render`'s doc comment), so a cell can be part of a
/// blob without holding an `Organism`, and a real edge-of-cluster organism
/// can render as plain terrain if erosion abstracted its cell away — Overview
/// was never meant to show individual organisms precisely, only the
/// population's coarse shape.
pub struct ClusterRender {
    pub species: Vec<Option<SpeciesId>>,
    pub density: Vec<f32>,
}

/// Clusters same-species occupied cells (8-connected, Moore,
/// `SimWorld::moore_neighbours` — `GridConfig::neighborhood_size`'s
/// fixed-by-design value) and derives each cluster's *rendered* blob shape
/// and density.
///
/// **Density** (unchanged since task 076): a population-mass reading, not a
/// compactness one — a cluster's literal occupied-cell count normalized
/// against `ClusterConfig::density_saturation`, clamped to `[0, 1]`. A
/// large, established colony saturates toward `1.0` (brightest) while a
/// lone organism sits near the bottom of the range, clearly visible but
/// never mistaken for the grid's real hot spot. Computed from the literal
/// member count, not the blob's filled/eroded size — task 078 corrects the
/// blob's *shape*, not this formula (its own acceptance criterion).
///
/// **Blob shape** (task 078, correcting task 076's 1:1 recoloring): a
/// cluster's real occupied-cell footprint often has interior gaps (cells
/// without an organism surrounded by ones that do), and task 076 rendered
/// those gaps as-is by only ever recoloring literally-occupied cells — the
/// user's playtest feedback was that a blob should read as a smaller,
/// abstracted shape, not a pixel-perfect trace including its holes. Two
/// passes per cluster, confined to its bounding box:
/// 1. **Fill**: flood-fill "exterior" from the bounding box's border
///    inward, stopping at member cells; any non-member cell the flood never
///    reaches is an enclosed hole and joins the blob.
/// 2. **Erode**: `ClusterConfig::blob_erosion_iterations` passes remove any
///    blob cell touching the shape's edge (a Moore neighbour outside the
///    blob, *or* outside the grid entirely — an edge cell is always a
///    boundary cell under normal morphological erosion) — this is what
///    makes the blob read as smaller than the real footprint, while an
///    elongated population still eroding down to a smaller *elongated*
///    shape (the general silhouette survives; only its edges shrink).
///    Skipped entirely below `ClusterConfig::blob_erosion_min_size`
///    filled cells, and aborted early if an iteration would erode a
///    cluster away to nothing — small clusters (a lone organism is the
///    extreme case, task 076's own "stays visibly distinct" acceptance
///    criterion) must never disappear.
///
/// Clusters are processed in cell-scan order (deterministic, same order
/// `SimWorld::cells` is stored in) and a cell already claimed by an
/// earlier-processed cluster's blob is never reclaimed — two different-
/// species clusters can never literally share a cell (`Cell::organism` is
/// single-occupancy), but their *filled/eroded* blobs, being derived from a
/// bounding box rather than exact occupancy, can otherwise overlap near
/// each other. First-claimed-wins keeps every cell's Overview blob
/// membership unambiguous and, since processing order never depends on
/// iteration over an unordered collection, still fully deterministic.
pub fn compute_cluster_render(world: &SimWorld, config: &SimConfig) -> ClusterRender {
    let mut species = vec![None; world.cells.len()];
    let mut density = vec![0.0_f32; world.cells.len()];
    let mut claimed = vec![false; world.cells.len()];
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
        let blob = compute_blob(world, &members, &config.cluster);
        for &idx in &blob {
            if claimed[idx] {
                continue;
            }
            claimed[idx] = true;
            species[idx] = Some(organism.species);
            density[idx] = cluster_density;
        }
    }

    ClusterRender { species, density }
}

/// A cluster's rendered blob (task 078): `members`' interior holes filled,
/// then eroded smaller — see `compute_cluster_render`'s doc comment. Both
/// passes work in a dense local grid over `members`' bounding box, not the
/// whole world grid, since clusters are typically small relative to it.
fn compute_blob(world: &SimWorld, members: &[usize], cfg: &ClusterConfig) -> Vec<usize> {
    let (min_x, max_x, min_y, max_y) = bounding_box(world, members);
    let box_w = max_x - min_x + 1;
    let box_h = max_y - min_y + 1;
    let local = |x: usize, y: usize| (y - min_y) * box_w + (x - min_x);

    let mut shape = vec![false; box_w * box_h];
    for &idx in members {
        shape[local(idx % world.width, idx / world.width)] = true;
    }

    fill_interior_holes(world, &mut shape, min_x, max_x, min_y, max_y, box_w, local);

    let filled_count = shape.iter().filter(|&&b| b).count();
    if filled_count as u32 >= cfg.blob_erosion_min_size {
        for _ in 0..cfg.blob_erosion_iterations {
            let eroded = erode_once(world, &shape, min_x, max_x, min_y, max_y, local);
            if !eroded.iter().any(|&b| b) {
                // Would erode the whole blob away — keep the previous
                // iteration's shape instead (never disappear entirely).
                break;
            }
            shape = eroded;
        }
    }

    let mut blob = Vec::new();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if shape[local(x, y)] {
                blob.push(world.index(x, y));
            }
        }
    }
    blob
}

fn bounding_box(world: &SimWorld, members: &[usize]) -> (usize, usize, usize, usize) {
    let mut min_x = usize::MAX;
    let mut max_x = 0;
    let mut min_y = usize::MAX;
    let mut max_y = 0;
    for &idx in members {
        let (x, y) = (idx % world.width, idx / world.width);
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    (min_x, max_x, min_y, max_y)
}

/// Flood-fills `shape`'s complement from the bounding box's border inward
/// ("exterior"); any cell never reached is an enclosed hole, folded into
/// `shape` in place.
#[allow(clippy::too_many_arguments)]
fn fill_interior_holes(
    world: &SimWorld,
    shape: &mut [bool],
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
    box_w: usize,
    local: impl Fn(usize, usize) -> usize,
) {
    let mut exterior = vec![false; box_w * (max_y - min_y + 1)];
    let mut stack = Vec::new();
    let seed = |x: usize, y: usize, exterior: &mut Vec<bool>, stack: &mut Vec<(usize, usize)>| {
        let li = local(x, y);
        if !shape[li] && !exterior[li] {
            exterior[li] = true;
            stack.push((x, y));
        }
    };
    for x in min_x..=max_x {
        seed(x, min_y, &mut exterior, &mut stack);
        seed(x, max_y, &mut exterior, &mut stack);
    }
    for y in min_y..=max_y {
        seed(min_x, y, &mut exterior, &mut stack);
        seed(max_x, y, &mut exterior, &mut stack);
    }
    while let Some((x, y)) = stack.pop() {
        for n in world.moore_neighbours(x, y) {
            let (nx, ny) = (n % world.width, n / world.width);
            if nx < min_x || nx > max_x || ny < min_y || ny > max_y {
                continue;
            }
            let li = local(nx, ny);
            if shape[li] || exterior[li] {
                continue;
            }
            exterior[li] = true;
            stack.push((nx, ny));
        }
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let li = local(x, y);
            if !shape[li] && !exterior[li] {
                shape[li] = true;
            }
        }
    }
}

/// One morphological-erosion pass: a blob cell survives only if every one of
/// its 8 Moore-neighbour positions is both inside the grid *and* itself
/// still part of the blob — a cell at the grid's edge always has fewer than
/// 8 in-grid neighbours, so it always erodes, matching standard erosion
/// against an implicit background outside the canvas.
#[allow(clippy::too_many_arguments)]
fn erode_once(
    world: &SimWorld,
    shape: &[bool],
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
    local: impl Fn(usize, usize) -> usize,
) -> Vec<bool> {
    let mut eroded = shape.to_vec();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let li = local(x, y);
            if !shape[li] {
                continue;
            }
            let neighbour_count = world.moore_neighbours(x, y).count();
            let touches_grid_edge = neighbour_count < 8;
            let touches_outside_shape = world.moore_neighbours(x, y).any(|n| {
                let (nx, ny) = (n % world.width, n / world.width);
                nx < min_x || nx > max_x || ny < min_y || ny > max_y || !shape[local(nx, ny)]
            });
            if touches_grid_edge || touches_outside_shape {
                eroded[li] = false;
            }
        }
    }
    eroded
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

    fn no_erosion_config() -> SimConfig {
        // Most tests below want to isolate fill-then-density behaviour from
        // erosion's own size/shape effects; a huge `blob_erosion_min_size`
        // disables erosion without touching the fill pass.
        let mut config = SimConfig::default();
        config.cluster.blob_erosion_min_size = u32::MAX;
        config
    }

    #[test]
    fn empty_grid_has_no_density() {
        let config = SimConfig::default();
        let world = SimWorld::new(1, &config);
        let render = compute_cluster_render(&world, &config);
        assert!(render.density.iter().all(|&d| d == 0.0));
        assert!(render.species.iter().all(|s| s.is_none()));
    }

    #[test]
    fn isolated_organism_is_visible_but_far_from_saturated() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(1, &config);
        place(&mut world, 5, 5, 0);
        let render = compute_cluster_render(&world, &config);
        let isolated = render.density[world.index(5, 5)];
        assert!(isolated > 0.0, "a lone organism must still be visible");
        assert!(
            isolated < 1.0,
            "a lone organism must not read as a saturated hot spot"
        );
        assert_eq!(render.species[world.index(5, 5)], Some(SpeciesId(0)));
    }

    #[test]
    fn a_large_cluster_reads_brighter_than_a_lone_organism() {
        let config = no_erosion_config();
        let mut world = SimWorld::new(1, &config);
        place(&mut world, 0, 0, 0);
        let lone = compute_cluster_render(&world, &config).density[world.index(0, 0)];

        // A solid block comfortably past `density_saturation` (20.0 by
        // default) so it's pinned at the ceiling regardless of exact tuning.
        for y in 10..15 {
            for x in 10..15 {
                place(&mut world, x, y, 1);
            }
        }
        let colony = compute_cluster_render(&world, &config).density[world.index(12, 12)];

        assert!(
            colony > lone,
            "an established colony must render brighter than a single stray organism"
        );
        assert_eq!(colony, 1.0);
    }

    #[test]
    fn cluster_density_saturates_at_the_configured_cell_count() {
        let mut config = no_erosion_config();
        config.cluster.density_saturation = 4.0;
        let mut world = SimWorld::new(1, &config);
        place(&mut world, 5, 5, 0);
        place(&mut world, 6, 5, 0);
        let render = compute_cluster_render(&world, &config);
        assert_eq!(render.density[world.index(5, 5)], 0.5);
    }

    #[test]
    fn different_species_never_merge_into_one_cluster() {
        let config = no_erosion_config();
        let mut world = SimWorld::new(1, &config);
        place(&mut world, 5, 5, 0);
        place(&mut world, 6, 5, 1);
        let render = compute_cluster_render(&world, &config);
        // Each is its own one-cell cluster of a different species — same
        // (small) density, but tracked independently, not merged.
        assert_eq!(
            render.density[world.index(5, 5)],
            render.density[world.index(6, 5)]
        );
        assert_ne!(
            render.species[world.index(5, 5)],
            render.species[world.index(6, 5)]
        );
    }

    /// Task 078's core acceptance criterion: an interior gap inside a
    /// cluster's occupied-cell footprint renders as filled (part of the
    /// blob), not as a hole. A 3x3 ring with an empty center is the
    /// simplest shape with a genuine enclosed hole.
    #[test]
    fn interior_gaps_within_a_cluster_are_filled() {
        let config = no_erosion_config();
        let mut world = SimWorld::new(1, &config);
        for (x, y) in [
            (4, 4),
            (5, 4),
            (6, 4),
            (4, 5),
            /* (5, 5) left empty: the hole */
            (6, 5),
            (4, 6),
            (5, 6),
            (6, 6),
        ] {
            place(&mut world, x, y, 0);
        }
        let render = compute_cluster_render(&world, &config);
        let hole_idx = world.index(5, 5);
        assert!(
            render.species[hole_idx].is_some(),
            "the ring's enclosed center should be filled, not left as a gap"
        );
        assert_eq!(render.density[hole_idx], render.density[world.index(4, 4)]);
    }

    /// The mirror case: a gap that is *not* enclosed (open to the
    /// bounding box's exterior) must stay empty — only genuinely enclosed
    /// holes get filled, not every non-member cell inside the bounding box.
    #[test]
    fn non_enclosed_gaps_next_to_a_cluster_are_not_filled() {
        let config = no_erosion_config();
        let mut world = SimWorld::new(1, &config);
        // An L-shape: (5, 5) is inside the bounding box but has a clear
        // path to the box border (e.g. straight down), so it's exterior,
        // not a hole.
        for (x, y) in [(4, 4), (5, 4), (6, 4), (4, 5), (4, 6), (6, 6)] {
            place(&mut world, x, y, 0);
        }
        let render = compute_cluster_render(&world, &config);
        assert!(
            render.species[world.index(5, 5)].is_none(),
            "a gap with a clear path to the bounding box border must not be filled"
        );
    }

    /// Task 078's acceptance criterion: erosion makes a large cluster's blob
    /// visibly smaller than its real footprint.
    #[test]
    fn erosion_shrinks_a_large_solid_cluster() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(1, &config);
        for y in 0..10 {
            for x in 0..10 {
                place(&mut world, x, y, 0);
            }
        }
        let render = compute_cluster_render(&world, &config);
        let blob_size = render.species.iter().filter(|s| s.is_some()).count();
        assert!(
            blob_size < 100,
            "a large solid cluster's blob should be smaller than its real \
             100-cell footprint, got {blob_size}"
        );
        assert!(blob_size > 0, "erosion must not erase the whole cluster");
    }

    /// Task 078's explicit non-regression criterion (AC4): a one-cell
    /// cluster (isolated organism) must still render, not be eroded away.
    #[test]
    fn isolated_organism_survives_erosion() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(1, &config);
        place(&mut world, 5, 5, 0);
        let render = compute_cluster_render(&world, &config);
        assert_eq!(render.species[world.index(5, 5)], Some(SpeciesId(0)));
    }

    /// Erosion must never erode a blob down to nothing, even for a
    /// thin/small shape that clears `blob_erosion_min_size` but has no true
    /// interior cell to survive a full pass.
    #[test]
    fn erosion_never_erases_a_cluster_entirely() {
        let mut config = SimConfig::default();
        config.cluster.blob_erosion_min_size = 1;
        config.cluster.blob_erosion_iterations = 5;
        let mut world = SimWorld::new(1, &config);
        // A thin horizontal line: every cell touches the shape's edge, so a
        // single erosion pass would erase it entirely without the
        // never-erase-to-nothing guard.
        for x in 4..9 {
            place(&mut world, x, 5, 0);
        }
        let render = compute_cluster_render(&world, &config);
        assert!(
            render.species.iter().any(|s| s.is_some()),
            "a thin cluster must never be eroded away to nothing"
        );
    }
}
