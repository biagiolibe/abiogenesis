# Task 070 — Remove task 062's decorative background layer (superseded by terrain colors)

> **ID**: `070`
> **Category**: Bugfix / UI
> **Priority**: 🔴 P1
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-09, raised directly by the user from a screenshot taken right after task 068 shipped

---

## 🎯 Objective

Every grid cell is a single `Sprite` entity (`spawn_grid`, `src/render.rs`).
For an occupied cell, `sync_grid_colors` swaps `Sprite::image` to a
metabolism shape mask (task 032 — circle/triangle/diamond, mostly
transparent outside the glyph) and tints it with the species color.
Everywhere that mask is transparent, the renderer shows whatever sits
*behind* that sprite — which is task 062's procedural decorative background
sprite, parked at `BACKGROUND_Z` strictly behind every grid cell, not the
cell's own terrain color.

Before terrain (066-068), this was invisible: empty cells were also
near-black (`Color::hsl(0.0, 0.0, 0.03 + cell.light * 0.12)`), so the
decorative background blended right in. Now that terrain has distinct flat
per-band colors (Sea/Plain/Hill/Mountain), any occupied cell — and any cell
a species reproduces into — shows a visibly wrong dark/off-color square
where the surrounding terrain color should show through, because task 062's
background leaks through the transparent parts of the organism's shape
mask.

The user's screenshot showed exactly this: point 1, patches showing task
062's old background color/texture instead of the terrain color underneath;
point 2, placed organisms keeping their correct metabolism glyph shape but
sitting on a black background square instead of their cell's actual terrain
color. Both are the same root cause, made obvious once population grows
after the first era advance (more occupied cells = more visible leaks).

**Decided fix** (explicit user call, not a compositing fix): remove task
062's decorative background layer entirely. It was added to solve "the grid
reads as an empty black void" — task 068's terrain colors now solve that
same problem with real per-cell data instead of decorative noise, so the
decorative layer is both redundant and actively wrong now that it leaks
through organism shape masks.

---

## 📋 Acceptance Criteria

- [x] The code compiles without errors; `cargo clippy -- -D warnings` is clean.
- [x] `spawn_background`, `sync_background`, the `BackgroundTexture` resource, `background_image`, `background_waves`, `background_field`, `BackgroundWave`, and every `BACKGROUND_*` constant are removed from `src/render.rs`.
- [x] `GridRenderPlugin` no longer registers `spawn_background` (Startup) or `sync_background` (Update) — no dangling system references.
- [x] `render.rs`'s unit tests referencing the removed background helpers (`background_image_is_deterministic_and_varies_with_seed`, `background_image_stays_dim_and_low_saturation`, or similar) are removed along with the code they test.
- [x] `task 068`'s terrain rendering — `cell_color`'s terrain branch, the `terrain_overlay` module (boundaries/peaks/toxic-zone outline) — is untouched by this change.
- [x] `MetabolismShapes`/`cell_shape`'s shape-mask system (task 032) is untouched — organisms keep their metabolism-specific glyph shape (circle/triangle/diamond); this task only removes what was showing through their transparent gaps.
- [x] Full `cargo test` is clean.
- [x] Verified via `cargo run` and directly by the user: after removing the decorative background layer, occupied/newly-reproduced-into cells show their correct terrain color around the organism's glyph shape — no stray dark/off-color square, no leftover decorative texture anywhere on the grid. (A follow-up two-sprite-layer redesign was attempted mid-session to further decouple the organism glyph from the terrain base, but the user confirmed the simple removal alone already fixed the reported bug, so that redesign was reverted — this task's scope stayed exactly "remove task 062's layer.")

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `spawn_background`, `sync_background`, `BackgroundTexture`, `background_image`/`background_waves`/`background_field`/`BackgroundWave`, `BACKGROUND_*` constants, and their registration in `GridRenderPlugin` — all removed here. |
| `tasks/done/062-procedural-background-layer.md` | Original task this reverts — background context only, not to be re-implemented. |

---

## 🧩 Technical Context

- **Current behavior**: a large dim procedurally-generated sprite sits behind the entire grid at `BACKGROUND_Z = -1.0`, regenerated per world seed by `sync_background`. It shows through any transparent pixel of any sprite in front of it — including the transparent gaps of an occupied cell's metabolism shape mask.
- **Desired behavior**: no decorative background layer. Transparent gaps in an organism's shape mask should reveal nothing but the cell's own opaque terrain-colored `Sprite` — since each grid cell is a single sprite, removing the layer behind it means there's nothing left to leak through inconsistently with the terrain color the player already learned to read.

---

## 🔨 Suggested Implementation

1. Delete `spawn_background`, `sync_background`, `BackgroundTexture`, `background_image`, `background_waves`, `background_field`, `BackgroundWave`, and the `BACKGROUND_*` constants from `src/render.rs`.
2. Remove their registration from `GridRenderPlugin`'s `Startup`/`Update` system additions.
3. Remove the now-dead unit tests covering the deleted functions.
4. Run `cargo clippy -- -D warnings` and `cargo test` to catch any remaining references.
5. Run `cargo run`, seed a world, advance an era, and visually confirm the fix (terrain color shows consistently, no stray dark squares, organism glyph shapes unaffected).

---

## ⚠️ Constraints and Caveats

- **Don't touch terrain rendering (068)** — `cell_color`'s terrain branch and the `terrain_overlay` module stay exactly as they are; this task only removes what sat behind them.
- **Don't touch the shape-mask system (032)** — `MetabolismShapes`/`cell_shape` and the metabolism glyph shapes (circle/triangle/diamond) are correct and untouched; only what leaked through their transparent gaps changes.
- **This is a deliberate revert, not a compositing fix** — no attempt to make organism sprites paint the terrain color into their own gaps; removing the layer behind them is the whole fix, per the user's explicit decision.

---

## 🔗 Dependencies

- **Depends on**: 062, 066, 067, 068.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/070-remove-decorative-background-layer.md)"$'\n\nExecute this task in the current project.'
```
