# Task 112 — Biome rendering (flat color, dithering, borders, tree overlay)

> **ID**: `112`
> **Category**: Feature (rendering)
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-biomes.md`)

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

- [ ] `terrain_color(kind: TerrainKind)` (`render.rs:1377-1384`) replaced/extended by
      a `biome_color(biome: Biome) -> Color` covering all in-scope biomes, same flat
      HSL-tone style and same "desaturated console/lab palette" family — Distesa di
      cristalli is the deliberate exception (doc: "tonalità visivamente aliena, fuori
      dalla palette naturale").
- [ ] `cell_color` (`render.rs:1311-1358`) reads `cell.biome` instead of
      `cell.terrain` for the empty-cell fallback branch; the existing
      organism/residue branches and the `toxicity_tint` composite
      (`render.rs:1397-1400`) are unchanged — this task does not touch how
      organisms/residue are drawn.
- [ ] **Dithering**: two-tone checkerboard pattern within a single biome's cells
      (new — `TerrainKind` rendering doesn't have this today), per the design doc's
      rendering-style section — a fixed pattern, not noise, and never blended across
      a biome boundary.
- [ ] `draw_boundaries` (`render.rs:508-534`) updated to compare `cell.biome` instead
      of `cell.terrain`; coastline detection (`is_coastline`, `render.rs:547`) keeps
      using "either side is water" but now checks the biome's landform category
      (Acqua profonda/bassa) rather than `TerrainKind::Sea` directly.
- [ ] `draw_peaks` (`render.rs:558-580`) unchanged — Vetta is still read via `is_peak`
      directly, not reclassified through `Biome`.
- [ ] Tree overlay: new render pass, separate from `biome_color` (per the design
      doc's "trees are not a biome, they're an independent decoration layer").
      Sparse density on Pianura/Collina/Montagna/Palude, dense on Foresta, absent
      elsewhere. Deterministic per-cell placement from the world seed (no
      `rand::rng()`, consistent with `TECH_DESIGN.md`'s determinism invariant even
      though this is pure rendering, not sim state — keeps re-renders and screenshots
      stable across frames).
- [ ] `draw_toxic_zone` (`render.rs:582+`) either removed here or left for task 113
      to remove, whichever lands second — coordinate with 113 so exactly one of the
      two tasks deletes it, not both redundantly and not neither.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; visual check via `cargo run`
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
