# Task 078 — Overview heatmap blob shape correction

> **ID**: `078`
> **Category**: Bugfix / Rendering (design correction to task 076)
> **Priority**: 🟢 P3
> **Estimate**: ~1-2h
> **Assigned to**: unassigned
> **Session**: 2026-08-09

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

- [ ] A cluster's Overview blob is visibly smaller/more compact than its
      real occupied-cell footprint (not a 1:1 recoloring of the same cells
      Detail mode would show).
- [ ] A cluster's blob renders as a solid, uniform shape: any holes/gaps in
      the actual cell distribution within the cluster's footprint are filled
      in, not reproduced as gaps in the blob.
- [ ] The density-brightness formula from task 076
      (`cluster::compute_cluster_density`, population-mass based via
      `ClusterConfig::density_saturation`) is preserved — this task changes
      the blob's *shape/extent*, not how brightness is computed.
- [ ] A one-cell cluster (isolated organism) still renders as a small but
      clearly visible blob (same scenario task 076 verified — don't regress
      it while shrinking/abstracting larger clusters).
- [ ] Terrain rendering (elevation bands, boundaries, peak glyphs,
      toxic-zone outline) is untouched.
- [ ] Detail mode's per-cell organism rendering (unchanged since before task
      075) is untouched — this is Overview-only.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: seed/grow an irregular, gappy cluster,
      zoom out past the Overview threshold, and confirm the blob reads as a
      smaller, solid shape rather than a full-size trace of the exact
      occupied cells.

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

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/078-overview-heatmap-blob-shape-correction.md)"$'\n\nExecute this task in the current project.'
```
