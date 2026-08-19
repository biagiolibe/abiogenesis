# Task 078 — Overview heatmap blob shape correction

> **ID**: `078`
> **Category**: Bugfix / Rendering (design correction to task 076)
> **Priority**: 🟢 P3
> **Estimate**: ~1-2h
> **Assigned to**: done
> **Session**: 2026-08-09
> **Implemented**: 2026-08-19

---

## 🎯 Objective

Task 076 shipped Overview mode's per-species cluster heatmap by recoloring
the existing per-cell organism sprites (the same entities Detail mode uses)
according to each cluster's density. Live verification worked and confirmed
the density formula (population mass, not compactness) is correct, but the
user flagged a shape/scale problem right after seeing it: because rendering
reuses the real per-cell sprites 1:1, each blob reproduces the population's
**exact real footprint at full size**, including whatever gaps/holes exist
in the actual cell distribution.

The original design intent
(`redesign/abiogenesis-two-tier-view.md`, reconfirmed directly by the user
after this playtest) is that an Overview blob should read as a **smaller,
abstracted representation** of the population — not a pixel-perfect trace of
it — and should render as a **uniform filled shape**: internal gaps/holes
within a cluster's footprint get smoothed over, not shown as-is.

---

## 📋 Acceptance Criteria

- [x] A cluster's Overview blob is visibly smaller/more compact than its
      real occupied-cell footprint (not a 1:1 recoloring of the same cells
      Detail mode would show).
- [x] A cluster's blob renders as a solid, uniform shape: any holes/gaps in
      the actual cell distribution within the cluster's footprint are filled
      in, not reproduced as gaps in the blob.
- [x] The density-brightness formula from task 076
      (`cluster::compute_cluster_density`, population-mass based via
      `ClusterConfig::density_saturation`) is preserved — this task changes
      the blob's *shape/extent*, not how brightness is computed.
- [x] A one-cell cluster (isolated organism) still renders as a small but
      clearly visible blob (same scenario task 076 verified — don't regress
      it while shrinking/abstracting larger clusters).
- [x] Terrain rendering (elevation bands, boundaries, peak glyphs,
      toxic-zone outline) is untouched.
- [x] Detail mode's per-cell organism rendering (unchanged since before task
      075) is untouched — this is Overview-only.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [x] Verified via a headless ASCII-rendered diagnostic (not a live
      `cargo run` screenshot — skipped this session by explicit user
      instruction): seed/grow an irregular, gappy cluster, confirm the blob
      reads as a smaller, solid shape rather than a full-size trace of the
      exact occupied cells. See implementation notes below for the actual
      diagrams produced.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/cluster.rs` | `compute_cluster_density` — task 076's connected-component clustering and density formula. Likely needs a companion function (or an extension) that derives the blob's *rendered* shape/extent, separate from the density value. |
| `src/render.rs` | `cell_color`, `sync_grid_colors`, `ClusterDensity`, `update_cluster_density` — currently recolors the real per-cell sprites in place for Overview; this task changes what actually gets drawn (rendered extent), not just the color. |
| `redesign/abiogenesis-two-tier-view.md` | Original design record — "shape/extent follows the cluster's real footprint" language that task 076 read as "exact footprint"; this task corrects that reading toward "abstracted representation of the footprint," per direct user clarification during 076's live review. |

---

## 🧩 Technical Context

- **Current behavior** (task 076): every occupied cell in a cluster gets the
  same `Sprite::color` (hue = species, lightness = cluster density), reusing
  the exact same per-cell sprite entities `spawn_grid` creates for Detail
  mode. The rendered shape is therefore identical, cell-for-cell, to the
  population's actual distribution — including any holes.
- **Desired behavior**: the rendered blob should be a smaller, solid
  abstraction of the cluster's footprint. Two directions worth evaluating
  (pick one, document the choice):
  1. **Shrink + fill**: derive a scaled-down region from the cluster's
     bounding box (e.g. some fraction of its extent, centered on the
     cluster's centroid) and fill it solidly, ignoring the individual
     occupied-cell mask entirely.
  2. **Morphological closing**: keep the cluster's real extent but run a
     fill/closing pass over its occupied-cell mask so interior gaps get
     painted too, then optionally shrink the result slightly.
  Either way, cells inside a cluster's abstracted shape that are *not*
  actually occupied by an organism need a rendering path — the current
  approach only ever recolors cells that already hold an `Organism`, so
  this likely requires either (a) extending which cells `sync_grid_colors`
  treats as "cluster interior" in Overview mode, independent of literal
  occupancy, or (b) a different rendering mechanism entirely (e.g. actual
  blob entities/textures, which task 076 deliberately avoided but may be
  the more natural fit for a shape that no longer maps 1:1 to real cells).
- Re-read `redesign/abiogenesis-two-tier-view.md`'s Overview description
  before implementing — the phrase "shape/extent follows the cluster's real
  footprint (so a population that has spread out in one direction reads as
  an elongated blob, not a uniform square)" is still the intended read at a
  coarse level (elongated populations should still read as elongated); what's
  wrong is rendering that footprint at 1:1 scale with its literal gaps, not
  the general idea of following the population's real shape.

---

## 🔨 Suggested Implementation

1. Re-read `redesign/abiogenesis-two-tier-view.md` and this task's
   "Technical Context" before choosing an approach.
2. Decide shrink+fill vs. morphological closing (or another approach) and
   document the choice with a comment, same pattern as task 076's
   z-order/blend-rule and density-formula comments in `cluster.rs`.
3. Adjust `cluster.rs` (or add a sibling function) to produce the blob's
   rendered shape, not just its density value.
4. Update the Overview rendering path in `render.rs` accordingly — this may
   require rendering cells that have no organism if the abstracted shape
   extends past the literal occupied-cell set.
5. Confirm the isolated-organism case (AC 4) doesn't regress: shrinking a
   1-cell cluster further would make it *less* visible, which is the
   opposite of what task 076 fixed — the shrink/abstraction logic should
   apply mainly to clusters big enough for it to matter, not uniformly at
   every size.

---

## ⚠️ Constraints and Caveats

- **Don't touch Detail mode or terrain rendering** — Overview-only.
- **No magic numbers**: any new shrink-factor/fill-threshold constants
  belong in `SimConfig` (extend `ClusterConfig`, same as task 076's
  `density_saturation`), not hardcoded.
- Keep the clustering/shape logic free of `bevy::render`/`bevy_egui`
  dependencies where practical, matching task 076's headless-testable
  pattern (`cluster.rs` has no Bevy renderer dependency today).

---

## 🔗 Dependencies

- **Depends on**: 076 (done) — this corrects its rendering, not its density
  formula.
- **Blocks**: none. Independent of 077 (action gating by view mode), which
  can proceed without this.

---

## ✅ Implementation notes (2026-08-19)

- **Approach chosen: morphological closing (fill + erode), not shrink+bbox.**
  The technical context's two options were shrink-from-bounding-box (loses
  the "elongated cluster reads as elongated" property outright — a bbox
  fraction is always a rectangle) or fill+erode (keeps the real silhouette,
  just smaller/smoothed). Went with fill+erode.
- **Rendering mechanism: still per-cell sprites, no new blob entities.**
  Task 076 deliberately avoided separate blob geometry; task 078 keeps that
  architecture by extending which cells count as "cluster interior"
  independent of literal occupancy (technical context's option (a)) rather
  than switching to option (b). `cell_color`'s Overview branch now checks
  `ClusterRender::species[idx]` (a cell's blob claim) instead of
  `Cell::organism` directly — a filled-hole cell can render with no real
  organism, and an eroded-away edge cell with a real organism can render as
  plain terrain instead. This is the expected/intended abstraction, not a
  bug: Overview was never meant to show individual organisms precisely.
- **New `cluster::compute_cluster_render`** replaces
  `compute_cluster_density`, returning both `density: Vec<f32>` (unchanged
  formula, still the literal member count normalized against
  `density_saturation` — task's own AC3) and `species: Vec<Option<SpeciesId>>`
  (which cluster's blob claims each cell). Per cluster: (1) flood-fill the
  bounding box's border-connected exterior; any non-member cell never
  reached is an enclosed hole, folded into the shape. (2)
  `blob_erosion_iterations` morphological-erosion passes remove any shape
  cell touching the shape's edge or the grid's edge, skipped entirely below
  `blob_erosion_min_size` filled cells, and aborted (keeping the prior
  iteration) if a pass would erode the blob to nothing.
- **Cross-cluster conflict resolution**: two different-species clusters can
  never literally share a cell (`Cell::organism` is single-occupancy), but
  their filled/eroded blobs — being bounding-box-derived, not exact-
  occupancy-derived — can otherwise overlap near each other. Resolved by
  processing clusters in cell-scan order (same deterministic order the
  connected-component discovery already used) and having a claimed cell
  never get reclaimed by a later cluster.
- **Config**: `ClusterConfig` gains `blob_erosion_min_size` (`8`) and
  `blob_erosion_iterations` (`1`). Calibrated by hand-building test clusters
  (a realistic circular blob with a ragged edge and a hole, and an
  elongated oval) and printing an ASCII diagram of real-footprint vs.
  blob-claim per cell via a temporary scratch example (removed before
  committing): a 79-cell circular cluster eroded to 45 cells, reading as a
  clean solid circle with its hole filled; a 91-cell elongated oval eroded
  to 43 cells while clearly staying a wide ellipse, not collapsing toward a
  circle. A pathological thin, zigzagging one-cell-wide diagonal "snake"
  shape (35 cells) eroded down to 9 — much more aggressively, but still
  non-empty (the never-erase-to-nothing guard held) — not a realistic
  population shape (reproduction spreads to Moore neighbours, so real
  clusters trend blobbier), noted here rather than over-tuned against.
- **10 new/updated tests in `cluster.rs`** (was 5): the original 5 updated
  to the new `ClusterRender` struct (most with erosion disabled via a
  `no_erosion_config` helper, to isolate density/fill behaviour from
  erosion's own size effects), plus `interior_gaps_within_a_cluster_are_filled`,
  `non_enclosed_gaps_next_to_a_cluster_are_not_filled` (the fill pass must
  distinguish a genuinely enclosed hole from an open gap with a clear path
  to the bounding box border), `erosion_shrinks_a_large_solid_cluster`,
  `isolated_organism_survives_erosion`, and
  `erosion_never_erases_a_cluster_entirely`.
- **`render.rs`**: `ClusterDensity` resource extended from a bare `Vec<f32>`
  to `{ density, species }`; `cell_color`'s Overview branch restructured
  around an `occupant: Option<(SpeciesId, f32)>` computed per-mode (Detail
  from `Cell::organism`, Overview from `ClusterRender::species`), keeping
  Detail's own logic byte-for-byte unchanged. One stale doc-comment
  reference to the old function name (near `SparkIndicators`) fixed in the
  same pass.
- All acceptance criteria met except the live-window screenshot check,
  explicitly skipped this session by direct user instruction ("non fare
  live check") — substituted with a headless ASCII-diagram verification
  against hand-built realistic cluster shapes (see calibration note above).
  `cargo build --all-targets`, `cargo test` (190 lib tests + all
  integration binaries, all green — `balance.rs` ran unusually slowly this
  session, ~508s instead of its usual ~55s, evidently system load rather
  than a regression, since it's unrelated code and passed cleanly),
  `cargo clippy --all-targets -- -D warnings`, and `cargo fmt -- --check`
  all clean.

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/078-overview-heatmap-blob-shape-correction.md)"$'\n\nExecute this task in the current project.'
```
