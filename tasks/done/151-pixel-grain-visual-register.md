# Task 151 — Pixel-grain visual register across map, HUD and notebook

> **ID**: `151`
> **Category**: UI
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~3h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

Restyle the whole interface — map, HUD, notebook — into a consistent
pixel-grain register: procedural, no hand-drawn assets, replacing today's
smooth vector look with quantized shapes, noise-textured biomes, squared-off
chrome and stepped graph edges. This is a rendering refinement of decisions
already made and shipped (color=environment/shape=life, the slate notebook
material, per-cell population model) — it changes *how* those are drawn, not
what they mean.

Design source: `redesign/processed/culture-shock-population-model-aesthetic.md`,
"Rendering a grana pixel `[deciso]`" section (lines 88-121) and "Colore =
ambiente, forma = vita `[deciso]`" (77-86) for the palette this restyle must
preserve, not revise. Also `redesign/processed/abiogenesis-ui-redesign.md` for
any general chrome/typography rules beyond the pixel-grain doc's own scope.

**Deliberately sequenced last in Phase 2's UI work**: this task restyles
whatever HUD/notebook features exist at the point it runs. Tasks 149
(inspection tool), 150 (control scheme/pause menu), 152 (HUD gaps), 153
(notebook Chronicle) all add new UI surface this task must also cover — run
151 after those land, not before, per this project's own explicit rationale
for splitting the aesthetic document across two phases in the first place
("so the HUD isn't restyled and then rebuilt").

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] **Organism shapes quantized to a pixel grid.** `render.rs`'s
      `MetabolismShapes` (`~1408`) currently generates smooth anti-aliased
      masks (`circle_mask`, `triangle_mask`, `diamond_mask`, `cross_mask`) at
      `SHAPE_TEXTURE_SIZE = 20` texels. Replace the mask generation so edges
      snap to a coarse block grid (a handful of blocks per shape, not a
      smooth curve) while keeping the same four silhouettes and the same
      `handle_for`/blocked-marker composition machinery (task 141) — this is
      a rasterization change, not a new shape system.
- [ ] **Procedural noise texture on biomes**, replacing `dithered_biome_color`
      (`render.rs:~1781`)'s fixed two-tone checkerboard dither. Deterministic
      per-cell noise (seeded from world seed + cell coords, no `rand::rng()`
      per `TECH_DESIGN.md` §5) breaking up the flat fill — same biome-color
      inputs (`biome_color`), different texture generator.
- [ ] **HUD chrome squared off**: action-bar icons (Seed/Stress/Cull/Splice),
      tick/notch indicators (`dot_row`, per task 152's findings), and any
      rounded-corner panel borders in `ui.rs` move to the same blocky
      register — a style pass over existing widgets, not new widgets. Cover
      whatever 149/150/152 added by the time this task runs.
- [ ] **Notebook relationship-graph edges become stepped paths** (horizontal/
      vertical pixel segments) instead of smooth/diagonal lines, in whatever
      graph-drawing code task 153's Chronicle work leaves in place
      (`notebook.rs`'s `hypothesis_grid` per this session's findings).
- [ ] Text stays untouched — the doc explicitly excludes monospace text from
      the pixel treatment ("non aveva bisogno di alcun trattamento pixel").
- [ ] Organism ink stays neutral per the doc's warmer variant: full amber
      (`#e0c99a`) above the energy-critical threshold, dim amber below —
      confirm/replace whatever tint `sync_grid_colors` currently applies for
      organism state, without touching species-hue or biome coloring.
- [ ] Live visual check (`cargo run`, screenshot or interactive) — this is a
      purely visual task, type-checking and unit tests cannot verify it.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `MetabolismShapes`/mask generators (`~1408-1520`), `dithered_biome_color` (`~1781`), `cell_shape`/`cell_color`. All four rewrite targets live here. |
| `src/ui.rs` | Action-bar icons, notch/dot indicators, panel borders — style pass. |
| `src/notebook.rs` | `hypothesis_grid`'s edge-drawing — stepped-path rework. |

---

## 🧩 Technical Context

- **Current behavior**: organism shapes are smooth procedurally-generated
  masks (circle/triangle/diamond/cross) at a texture resolution meant to
  look clean, not blocky; biomes use a fixed-amplitude two-tone lightness
  checkerboard (`DITHER_LIGHTNESS_DELTA`); HUD panels/icons use rounded
  vector chrome; the notebook's relationship graph (already shipped, not
  this task's job to build) draws direct edges between nodes.
- **Desired behavior**: same shapes, same data, same layout — every edge
  quantized to a visible block grid, biome fill textured with procedural
  noise instead of a flat two-tone pattern, HUD chrome squared, graph edges
  stepped. Purely a rasterization/style layer over unchanged logic.

---

## 🔨 Suggested Implementation

1. Rework the four `*_mask` functions (or the way `shape_mask_image` samples
   them) to snap to a coarse grid — e.g. evaluate the mask at a lower
   effective resolution and nearest-neighbour-upscale, rather than
   `SHAPE_TEXTURE_SIZE`'s current smooth sampling.
2. Replace `dithered_biome_color`'s two-branch dither with a small
   deterministic noise function (hash of `(world_seed, x, y)` into a
   lightness offset with more than two discrete levels) — keep it cheap,
   this runs per visible cell per frame.
3. Style pass over `ui.rs`'s egui widgets: square corners, blocky notch
   glyphs — likely CSS-analog egui `Rounding`/`Stroke` tweaks rather than new
   geometry.
4. Rework `hypothesis_grid`'s edge line-drawing to route horizontal-then-
   vertical (or vice versa) instead of a direct line between two node
   positions.
5. Live-check in `cargo run`: map at both zoom levels, action bar, notebook's
   four sections.

---

## ⚠️ Constraints and Caveats

- **No hand-drawn assets** — every technique here must stay 100% procedural,
  per the doc's own explicit rationale (near-zero content cost for 15+10
  traits, 16 biomes, future xenotraits; a sprite-per-element pipeline would
  reintroduce exactly the cost the project has avoided).
- **Determinism**: `sim`/`world`/`config` stay untouched by this task —
  every change here lives in `render.rs`/`ui.rs`/`notebook.rs` (presentation
  layer), and any noise/texture generation must be a pure function of
  existing deterministic inputs (coords, seed), never real RNG.
- **No magic numbers**: new grid/noise constants into config or clearly
  named local consts, matching the existing `SHAPE_TEXTURE_SIZE`/
  `DITHER_LIGHTNESS_DELTA` pattern.
- Don't touch the color=environment/shape=life rule itself, or the slate
  notebook material — both are `[deciso]` and out of this task's scope,
  this task only changes rasterization technique.

---

## 🔗 Dependencies

- **Depends on**: 137 (per-cell population model — the data this renders),
  149, 150, 152, 153 (this task restyles their UI additions, so run after).
- **Blocks**: none in Phase 2; Phase 3's `tag-archetypes` work (task 155)
  will eventually add new trait-code glyphs that should already land in this
  same pixel register once introduced (not this task's job to anticipate).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/151-pixel-grain-visual-register.md)"$'\n\nExecute this task in the current project.'
```
