# Task 076 — Overview mode: per-species cluster heatmap rendering

> **ID**: `076`
> **Category**: Feature / Rendering
> **Priority**: 🟡 P2
> **Estimate**: ~2-3h
> **Assigned to**: unassigned
> **Session**: 2026-08-09

---

## 🎯 Objective

With task 075's `MapViewMode` in place, this task implements what Overview
mode actually draws instead of individual organism sprites: per-species
**connected-component clusters** rendered as density-heatmap blobs. Full
rationale in `redesign/abiogenesis-two-tier-view.md` — short version: instead
of downsampling the grid into a fixed checkerboard of equal-size blocks
(which would risk losing small/isolated populations inside mostly-empty
tiles), group contiguous same-species occupied cells into clusters and render
each as a blob whose shape/extent follows the cluster's real footprint, hue
is the species color, and brightness/intensity reflects local population
density. An isolated single organism is simply its own one-cell cluster —
still visible, not absorbed into anything.

---

## 📋 Acceptance Criteria

- [x] When `MapViewMode::Overview` is active, occupied cells are not drawn as
      individual organism sprites; instead, per-species clusters (connected
      components of contiguous same-species occupied cells, standard
      4- or 8-connected flood-fill — decide against `neighborhood_size`
      already used for the sim's own Moore neighborhood, `GridConfig`) are
      computed and each rendered as a single blob covering its footprint.
      Implemented as `cluster::compute_cluster_density` (8-connected, reusing
      `SimWorld::moore_neighbours`), consumed by the existing per-cell
      sprites rather than new blob entities — see "Technical decisions"
      below.
- [x] Blob color: the cluster's species hue (reuse `species_color`,
      `src/render.rs:54`, for consistency with the sidebar/notebook). Blob
      brightness/intensity: reflects local population density within the
      cluster (e.g. occupied-cell fraction of the cluster's bounding
      region, or a per-cell density falloff — pick something legible and
      testable, document the formula chosen).
      Formula: cluster cell count normalized against `ClusterConfig::
      density_saturation` (default `20.0`), clamped to `[0, 1]` — a
      population-mass reading, not compactness. A compactness formula
      (occupied cells / bounding-box area) was tried first and rejected
      after `advisor` review: it scored a lone organism (1x1 bbox, always
      100% full) as *maximally* dense, brighter than a real sprawling
      colony — backwards for a population heatmap.
- [x] A one-cell cluster (isolated organism) renders as a small but clearly
      visible blob — verify this specifically, it's the scenario task 074's
      follow-up discussion called out by name.
      Verified live (see below): a lone organism renders as a single bright
      dot at a `0.20` lightness floor (above every terrain band's own
      lightness), clearly visible but dimmer than a saturated colony.
- [x] Terrain rendering (elevation bands, boundaries, peak glyphs, toxic-zone
      outline — tasks 066-072) is untouched by this task; only the organism
      layer's representation changes in Overview mode.
- [x] Two adjacent/overlapping clusters of different species (interleaved
      cells in the same area) render without one silently hiding the other —
      decide and document a simple z-order or blend rule.
      Rule: none needed. `Cell::organism` is single-occupancy, so no cell
      ever has to choose between two species' colors — each cell always
      renders whichever species (if any) actually occupies it, exactly like
      Detail mode. Documented in `cluster.rs`.
- [x] Cluster computation does not run every render frame — recompute only
      on population-changing events (tick advance, action resolution),
      matching the note in `redesign/abiogenesis-two-tier-view.md` about
      not re-running connected-component analysis unconditionally on a
      ~10000+ cell grid every frame.
      `update_cluster_density` gates on `Res<SimWorld>::is_changed()` (true
      exactly on ticks/actions that mutate the world) OR `MapViewMode`
      switching into `Overview`, and only while `Overview` is active.
- [x] Switching between `Overview` and `Detail` (task 075's threshold) shows
      the right representation immediately, no stale frame from the other
      mode.
      Verified live via zoom in/out over a grown cluster.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [x] Verified live via `cargo run`: seed a few species in different
      configurations (clustered, scattered, one isolated organism), zoom out
      past the threshold, confirm each population reads clearly as a
      distinct, appropriately-shaped/colored blob.
      Live-tested the release binary with screenshots: a 13-cell Nyx
      population rendered as a bright, irregularly-shaped red blob
      (footprint, not a bounding box) next to a single-cell Kael organism
      rendered as a small dim green dot — the colony read unambiguously
      brighter. Zooming past the Detail threshold and back confirmed no
      stale frame either direction; terrain (bands/boundaries) unaffected
      throughout.

### Technical decisions (beyond the suggested implementation)

- **No new blob entities.** The suggested implementation sketch proposed
  spawning/despawning blob entities; instead, `Overview` mode reuses the
  same per-cell sprites `spawn_grid` already creates (one per grid cell),
  recoloring occupied cells by their cluster's density in `cell_color`.
  Coloring every cell in a cluster the same value makes the *union* of
  those sprites read as one blob whose shape is the cluster's exact
  footprint — simpler than bbox math, and gets click/viewport handling
  (`world_to_cell`, the zoom camera, HUD-viewport clipping) for free since
  it's the same entities Detail mode already uses.
- **`ClusterConfig::density_saturation`** (new `SimConfig` field, `20.0`
  default) is the only new tunable, mirrored in
  `assets/config/sim_config.ron` per the hot-reload convention (task 073).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `sync_grid_colors`, `cell_color`, `species_color`, `spawn_grid` — current per-cell sprite rendering this task adds an alternate path alongside, gated by `MapViewMode`. |
| `src/world.rs` | `SimWorld::cells`, `moore_neighbours` — organism occupancy and neighbor iteration this task's clustering pass will read from. |
| `redesign/abiogenesis-two-tier-view.md` | Design record — the clustering approach, its rationale, and why it was chosen over a fixed-grid block scheme. |

---

## 🧩 Technical Context

- **Current behavior**: every occupied cell gets its own sprite
  (`sync_grid_colors`), colored by `cell_color`/`species_color` and shaped by
  `MetabolismShapes`. This is what Detail mode keeps doing unchanged.
- **Desired behavior in Overview**: instead of per-cell sprites for
  organisms, one blob entity (or a small number of them) per per-species
  connected component, positioned/sized to match the cluster's cell
  footprint, colored per the density formula chosen above.
- This task only needs to read `world.cells` (occupancy + species) — it
  doesn't change simulation state, matching the render/sim boundary
  (TECH_DESIGN.md §5).

---

## 🔨 Suggested Implementation

1. Read `redesign/abiogenesis-two-tier-view.md` in full first, and confirm
   task 075 has landed (`MapViewMode` resource available).
2. Write a pure clustering function (testable independent of Bevy, similar
   in spirit to how `sim`/`world` logic is kept headless-testable) that
   takes occupied cells + species and returns per-species connected
   components with their bounding footprint and cell count.
3. Decide and implement the density formula (blob brightness/intensity).
4. Add the Bevy-side rendering: spawn/update blob entities from the
   clustering output when in `Overview` mode, hide/despawn the normal
   per-cell organism sprites while in that mode (terrain sprites stay).
5. Wire recomputation to population-changing events rather than every
   frame.
6. Handle the overlapping-cluster case (pick and document a simple rule).

---

## ⚠️ Constraints and Caveats

- **Don't touch terrain rendering** — this is scoped to the organism layer
  only.
- **No magic numbers**: any density-formula constants belong in `SimConfig`.
- Keep the clustering algorithm itself free of `bevy::render`/`bevy_egui`
  dependencies if it's pure enough to unit-test headlessly — matches the
  project's existing pattern of keeping sim-adjacent logic testable without
  spinning up a Bevy `App`.

---

## 🔗 Dependencies

- **Depends on**: 075 (`MapViewMode` resource and zoom camera must exist
  first).
- **Blocks**: none directly.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/076-overview-cluster-heatmap-rendering.md)"$'\n\nExecute this task in the current project.'
```
