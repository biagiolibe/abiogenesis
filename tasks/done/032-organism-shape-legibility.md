# Task 032 — Distinguish organisms by shape (metabolism), not just color

> **ID**: `032`
> **Category**: UX
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-03 playtest

---

## 🎯 Objective

A 2026-08-03 playtest found the world grid "anonymous": every cell renders as a flat colored square (`render.rs::cell_color`), differentiated only by hue (species) and lightness (energy). A predator and a photolithic organism with similar hues are nearly indistinguishable at a glance — the player has to cross-reference the HUD's population list to know what they're looking at, rather than reading it directly off the board. This task adds a second visual dimension — **shape**, keyed off `Metabolism` — so the three metabolisms (`Photolithic`, `Predator`, `Decomposer`) are distinguishable independent of color.

---

## 📋 Acceptance Criteria

- [x] Occupied cells render with a distinct shape per `Metabolism` variant (e.g. circle for `Photolithic`, triangle for `Predator`, diamond/square for `Decomposer`) — species hue and energy-based lightness (task 006/existing `cell_color` logic) still apply, shape is an additional dimension, not a replacement.
- [x] Empty and residue-only cells are visually unaffected — this only changes how *occupied* cells render.
- [x] Shape updates correctly as organisms move/die/spawn across cells (a cell that held a predator and now holds a photolithic organism must show the new shape next frame, not a stale one) — verify this holds given `sync_grid_colors` already re-evaluates every cell every frame.
- [x] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `spawn_grid`, `sync_grid_colors`, `cell_color` — all three likely need changes depending on the chosen approach |
| `src/world.rs` | `Metabolism` enum — read-only, already public |

---

## 🧩 Technical Context

**This is the open technical decision for this task**: today each grid cell is one entity with a `Sprite` component, updated per-frame only via `Sprite::color` (a solid-colored square, `Sprite::from_color`). There's no existing mechanism for varying *shape* per cell at runtime. Two viable approaches, pick one and document the choice:

1. **Procedural shape textures.** At `Startup`, generate a small set of `Image`/texture handles (one per `Metabolism` variant, e.g. a filled circle/triangle/diamond rendered into a small bitmap or via `bevy::render`'s primitive-shape helpers if available in this Bevy version), and swap which texture a cell's `Sprite` uses (`Sprite::image`) based on the occupying organism's metabolism, alongside the existing per-frame `Sprite::color` tint. Keeps the current "one `Sprite` entity per cell" architecture — likely the smaller change.
2. **Mesh2d per shape.** Switch occupied cells to `Mesh2d`/`MeshMaterial2d` bundles using Bevy's built-in 2D primitive meshes (circle, triangle, etc.) instead of `Sprite`. More invasive — would need per-cell mesh/material swapping logic in `sync_grid_colors` (today it only ever touches `Sprite::color`), and empty/residue cells would need to stay on the simpler `Sprite` path or also migrate. Only worth it if approach 1 turns out to have real limitations (e.g. texture generation proving awkward in this Bevy version).

Check what primitive-shape/procedural-texture helpers this project's pinned Bevy version (`TECH_DESIGN.md` §12) actually exposes before committing to an approach — don't assume an API without verifying it compiles against the pinned version.

---

## 🔨 Suggested Implementation

1. Decide and document the shape-rendering approach (see Technical Context).
2. Generate/prepare the three metabolism shapes once (`Startup`, alongside `spawn_camera`/`spawn_grid`).
3. Extend `sync_grid_colors` (or a small helper it calls) to select the right shape per occupied cell based on `world.species[organism.species.0 as usize].metabolism`, in addition to its existing color logic.
4. Manual verification via the `run` skill: seed one organism of each metabolism (photolithic already seedable via the starting palette; predator/decomposer need a `Splice`-created or manually-tested species, or temporarily adjust `seed_starting_palette` locally for testing, reverted before considering the task done), confirm each renders with a visibly distinct shape, confirm shape updates correctly when an organism dies/is replaced.

---

## ⚠️ Constraints and Caveats

- Keep rendering strictly read-only against `SimWorld` (TECH_DESIGN.md §3.1/§3.3) — this is presentation-only, no simulation changes.
- Don't add an external image-asset pipeline (loading files from disk) if procedural generation or built-in mesh primitives can cover it — keep the project's zero-asset-dependency footprint for now unless there's a strong reason not to.

---

## 🔗 Dependencies

- **Depends on**: 006 (grid rendering), 010-012 (tags/matrix, for why species need to stay distinguishable by more than just shape)
- **Blocks**: none (independent of 033)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/032-organism-shape-legibility.md)"$'\n\nExecute this task in the current project.'
```
