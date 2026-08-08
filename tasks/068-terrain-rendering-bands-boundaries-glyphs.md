# Task 068 — Terrain rendering: elevation bands, boundaries, peak glyphs, toxic-zone overlay

> **ID**: `068`
> **Category**: UI
> **Priority**: 🟡 P2
> **Estimate**: ~4-6h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (from `redesign/abiogenesis-terrain-map.md`, follow-up to tasks 066/067)

---

## 🎯 Objective

This is the visual half of `redesign/abiogenesis-terrain-map.md` — read that
file for the full rationale, the reference mockup
`redesign/terrain-map-elevation.svg`, and the concrete rules in its "Regole
concrete" section (1-6) — now backed by real per-cell terrain data from task
066 (and gated placement from 067) instead of a demonstrative shape
function.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors; `cargo clippy -- -D warnings` is clean.
- [ ] `cell_color`'s empty-cell branch (`render.rs:727-743`) is replaced/extended with flat per-terrain-band colors — desaturated tones consistent with the existing console/lab palette, no new palette family introduced.
- [ ] Sea renders near-black, close to the grid's background — not as a blue "water" fill — preserving the original doc's "void" reading even though Sea is now real simulation data, not an absence of cell.
- [ ] Boundary lines are drawn between differently-classed adjacent cells: thin/dark for internal band transitions (e.g. Plain/Hill), thicker/lighter for Sea↔land transitions (coastline reading). Implemented via an egui immediate-mode painter (`ctx.layer_painter(egui::LayerId::background())`, same pattern as `draw_energy_overlay`, `render.rs:325-338`) drawn over the grid sprites — not per-edge sprite entities.
- [ ] Peak glyphs (`^` or equivalent) are drawn via the same painter, one per stored peak cell from task 066 — no render-time local-maximum re-derivation.
- [ ] The toxic zone gets a dashed border around its (now variable) bounds, drawn via the same painter, reusing the dash-segment approach/constants from `draw_dashed_ring` (`notebook.rs:679`) adapted to a rectangle outline, for visual consistency with the existing "unconfirmed/hazardous" treatment.
- [ ] `apply_environment_overlay` (`render.rs:130-146`, T/L heatmap toggles) makes a deliberate decision about Sea/unplaceable cells — at minimum, doesn't silently erase the terrain read the player just learned; implementer's call on the exact treatment, but it must be explicit, not an oversight.
- [ ] Verified visually via `cargo run`: seed a world, confirm bands/boundaries/peaks/toxic-zone outline render distinctly from organisms/residue, in both a normal view and with T/L overlays toggled.
- [ ] `render.rs`'s existing `cell_color`-adjacent unit tests (around line 727+) are updated/extended for the new terrain-based branch.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `cell_color`, `apply_environment_overlay`, `draw_energy_overlay` (painter precedent) — all the rendering changes land here. |
| `src/notebook.rs` | `draw_dashed_ring` (line ~679) — dash-segment approach to adapt for the toxic-zone rectangle outline. |
| `redesign/abiogenesis-terrain-map.md` | Full design rationale and concrete rules. |
| `redesign/terrain-map-elevation.svg` | Reference mockup (demonstrative shape only — real shape now comes from task 066's generator). |

---

## 🧩 Technical Context

- **Current behavior**: empty cells shade continuously by `light` (`Color::hsl(0.0, 0.0, 0.03 + cell.light * 0.12)`); the toxic zone is only a color-tint blend (`toxicity_tint`) with no outline at all; the grid is one flat `Sprite` per cell with no edge-drawing or text-glyph mechanism.
- **Desired behavior**: flat per-terrain-band colors with thin/thick boundary lines, sparse peak glyphs, and a dashed toxic-zone outline — all painted over the grid sprites via egui, per the `draw_energy_overlay` precedent.

---

## 🔨 Suggested Implementation

1. Add a terrain→color mapping function, replacing the empty-cell branch in `cell_color`.
2. Add a new egui-painter system (mirroring `draw_energy_overlay`'s registration) that, each frame, compares each cell's terrain class to its right/below neighbour and draws a boundary line where they differ, styled by whether either side is Sea.
3. In the same painter, draw the peak glyph for each stored peak cell.
4. In the same painter, draw a dashed rectangle around the toxic zone's current bounds, adapting `draw_dashed_ring`'s dash-segment logic.
5. Update `apply_environment_overlay` to make a deliberate choice for unplaceable cells.
6. Update/extend `render.rs`'s `cell_color` tests.
7. Run `cargo run`, seed a world, visually confirm against the mockup's intent (not pixel-identical — the shape is now procedural, not the mockup's demo function).

---

## ⚠️ Constraints and Caveats

- **No texture/gradient/illustrative asset** — flat color and line only, per the original doc's pillar-3 constraint (this task is *not* an exception to pillar 3, unlike task 062's decorative background layer — terrain color must keep mapping to real data).
- **Don't touch task 062's background layer** — it stays a separate, decorative, behind-the-grid layer; this task's boundaries/glyphs render in front of the grid sprites, a different concern entirely.
- **Don't change placement rules** — 066/067 own that; this task is presentation only.

---

## 🔗 Dependencies

- **Depends on**: 066, 067.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/068-terrain-rendering-bands-boundaries-glyphs.md)"$'\n\nExecute this task in the current project.'
```
