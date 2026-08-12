# Task 112 — Biome rendering (flat color, dithering, borders, tree overlay)

> **ID**: `112`
> **Category**: Feature (rendering)
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: done
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-biomes.md`),
> implemented 2026-08-12.

---

## ✅ Implementation notes (2026-08-12)

- `terrain_color(TerrainKind)` replaced by `biome_color(Biome) -> Color`
  (all 15 in-scope biomes; Geyser stays out per task 114). `CrystalField`
  is the deliberate alien-hue exception (`hsl(280, 0.55, 0.35)`) — every
  other biome stays in the existing low-lightness console/lab family.
- `cell_color`'s empty-cell branch now calls `dithered_biome_color`
  (`biome_color` + a fixed `(x + y) % 2` checkerboard lightness offset,
  `DITHER_LIGHTNESS_DELTA = 0.015`) instead of `terrain_color(cell.terrain)`.
- `draw_boundaries`/`draw_edge` read `cell.biome`; `is_coastline` now
  checks a new `is_water(biome)` helper (`DeepWater`/`ShallowWater` only —
  Lago deliberately excluded, it's a standalone inland feature).
- `draw_peaks` untouched, exactly as specified (still reads `is_peak`
  directly).
- New tree overlay (`draw_trees`, in the `terrain_overlay` egui-painter
  pass): sparse on Pianura/Collina/Montagna/Palude, dense on Foresta,
  none elsewhere, monochrome `♣` glyph (not emoji — avoids the tofu-box
  issue `ui.rs`'s `DEJAVU_SANS` doc comment already flags for
  `ACTION_GLYPHS`). Deterministic per-cell via a pure `tree_hash(seed, x,
  y)` bit-mixer (SplitMix64-style), no RNG. Skips occupied cells so a tree
  glyph never fights an organism sprite for the same pixels.
- `draw_toxic_zone` left untouched — task 113's own acceptance criteria
  claims its removal, avoiding the double-removal/neither-removes-it
  ambiguity the task description flagged.
- Rewrote the render-test suite's terrain-band tests as biome-equivalents
  (`each_biome_has_a_distinct_flat_color`,
  `deep_water_reads_as_a_dark_blue_not_a_void`), added dithering/tree
  coverage tests.
- Verified live via `cargo run` + screenshot at the real 128×80 grid:
  flat dithered biome colors, thin internal borders vs. a thicker
  coastline, sparse/dense tree glyphs, and the `CrystalField` alien-color
  patch all read correctly together.

---

## 🎯 Objective

Render the 16 biomes (areal from task 110, feature from task 111) using the same
visual language already established for `TerrainKind` — flat color per cell, thin
grid-aligned borders, no gradients — extended with two things `TerrainKind`
rendering doesn't have yet: two-tone dithering within a biome, and a tree overlay.

No new rendering *technique* — this task applies patterns that already exist in
`render.rs` to a bigger enum, plus dithering (new) and trees (new).

---

## 📋 Acceptance Criteria

- [x] `terrain_color(kind: TerrainKind)` (`render.rs:1377-1384`) replaced/extended by
      a `biome_color(biome: Biome) -> Color` covering all in-scope biomes, same flat
      HSL-tone style and same "desaturated console/lab palette" family — Distesa di
      cristalli is the deliberate exception (doc: "tonalità visivamente aliena, fuori
      dalla palette naturale").
- [x] `cell_color` (`render.rs:1311-1358`) reads `cell.biome` instead of
      `cell.terrain` for the empty-cell fallback branch; the existing
      organism/residue branches and the `toxicity_tint` composite
      (`render.rs:1397-1400`) are unchanged — this task does not touch how
      organisms/residue are drawn.
- [x] **Dithering**: two-tone checkerboard pattern within a single biome's cells
      (new — `TerrainKind` rendering doesn't have this today), per the design doc's
      rendering-style section — a fixed pattern, not noise, and never blended across
      a biome boundary.
- [x] `draw_boundaries` (`render.rs:508-534`) updated to compare `cell.biome` instead
      of `cell.terrain`; coastline detection (`is_coastline`, `render.rs:547`) keeps
      using "either side is water" but now checks the biome's landform category
      (Acqua profonda/bassa) rather than `TerrainKind::Sea` directly.
- [x] `draw_peaks` (`render.rs:558-580`) unchanged — Vetta is still read via `is_peak`
      directly, not reclassified through `Biome`.
- [x] Tree overlay: new render pass, separate from `biome_color` (per the design
      doc's "trees are not a biome, they're an independent decoration layer").
      Sparse density on Pianura/Collina/Montagna/Palude, dense on Foresta, absent
      elsewhere. Deterministic per-cell placement from the world seed (no
      `rand::rng()`, consistent with `TECH_DESIGN.md`'s determinism invariant even
      though this is pure rendering, not sim state — keeps re-renders and screenshots
      stable across frames).
- [x] `draw_toxic_zone` (`render.rs:582+`) either removed here or left for task 113
      to remove, whichever lands second — coordinate with 113 so exactly one of the
      two tasks deletes it, not both redundantly and not neither.
- [x] `cargo clippy -- -D warnings` and `cargo fmt` clean; visual check via `cargo run`
      (per this repo's UI-change convention) confirms flat color, dithering, and tree
      overlay read correctly at the actual 128×80 grid size, not just in isolation.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs:1311-1358` | `cell_color` — biome read happens here. |
| `src/render.rs:1377-1384` | `terrain_color` — replaced by `biome_color`. |
| `src/render.rs:1397-1400` | `toxicity_tint` — unchanged, reused as-is (already matches the doc's "flat tint overlay, not blend" rule). |
| `src/render.rs:508-534` | `draw_boundaries` — biome-based edges instead of terrain-based. |
| `src/render.rs:582+` | `draw_toxic_zone` — coordinate removal with task 113. |
| `redesign/abiogenesis-biomes.md` | Rendering-style constraints (flat color, dithering, borders, overlay-as-tint) and the reference/example SVGs. |

---

## 🔗 Dependencies

- **Depends on**: 110, 111 (every biome must already be assigned on `Cell.biome`
  before it can be colored).
- **Blocks**: none.
