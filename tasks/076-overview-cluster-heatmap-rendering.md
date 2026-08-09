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

- [ ] When `MapViewMode::Overview` is active, occupied cells are not drawn as
      individual organism sprites; instead, per-species clusters (connected
      components of contiguous same-species occupied cells, standard
      4- or 8-connected flood-fill — decide against `neighborhood_size`
      already used for the sim's own Moore neighborhood, `GridConfig`) are
      computed and each rendered as a single blob covering its footprint.
- [ ] Blob color: the cluster's species hue (reuse `species_color`,
      `src/render.rs:54`, for consistency with the sidebar/notebook). Blob
      brightness/intensity: reflects local population density within the
      cluster (e.g. occupied-cell fraction of the cluster's bounding
      region, or a per-cell density falloff — pick something legible and
      testable, document the formula chosen).
- [ ] A one-cell cluster (isolated organism) renders as a small but clearly
      visible blob — verify this specifically, it's the scenario task 074's
      follow-up discussion called out by name.
- [ ] Terrain rendering (elevation bands, boundaries, peak glyphs, toxic-zone
      outline — tasks 066-072) is untouched by this task; only the organism
      layer's representation changes in Overview mode.
- [ ] Two adjacent/overlapping clusters of different species (interleaved
      cells in the same area) render without one silently hiding the other —
      decide and document a simple z-order or blend rule.
- [ ] Cluster computation does not run every render frame — recompute only
      on population-changing events (tick advance, action resolution),
      matching the note in `redesign/abiogenesis-two-tier-view.md` about
      not re-running connected-component analysis unconditionally on a
      ~10000+ cell grid every frame.
- [ ] Switching between `Overview` and `Detail` (task 075's threshold) shows
      the right representation immediately, no stale frame from the other
      mode.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: seed a few species in different
      configurations (clustered, scattered, one isolated organism), zoom out
      past the threshold, confirm each population reads clearly as a
      distinct, appropriately-shaped/colored blob.

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
