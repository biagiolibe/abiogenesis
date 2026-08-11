# Task 113 — Palude replaces `toxic_zone`

> **ID**: `113`
> **Category**: Refactor (worldgen + objectives)
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-biomes.md`)

---

## 🎯 Objective

`redesign/abiogenesis-biomes.md` proposes the Palude biome as a generalization of the
existing isolated `toxic_zone` rectangle — diffuse toxicity over a whole biome instead
of one ad-hoc rectangle. This task removes `toxic_zone` and re-points everything that
depended on it (most importantly the `SurviveIn`/`ZoneKind::Toxic` objective) at the
Palude biome from task 110.

**This is more than a cosmetic swap.** `ZoneKind::Toxic` (`objectives.rs:320-336`) is
a real game objective, checked as `world.toxic_zone.contains(x, y)` — a fixed
rectangle with bounds. Palude can be multi-patch and irregularly shaped (task 110's
Stage B uses patch-gated noise, not a rectangle), so the check becomes cell-membership
in a biome, not containment in a rectangle.

---

## 📋 Acceptance Criteria

- [ ] `toxic_zone: ToxicZoneBounds` field, `ToxicZoneBounds` struct, and
      `place_toxic_zone` (`world.rs:345-403`) removed from `SimWorld`.
      `EnvironmentConfig`'s `toxic_zone_*` fields and `TerrainConfig`'s
      `min_toxic_zone_placeable_fraction`/`max_toxic_zone_placement_attempts`
      (`config.rs:141-146, 572-580`) removed or repurposed for Palude's own placement
      config from task 110, not left dead.
- [ ] `cell_in_zone` (`objectives.rs:333-335`) rewritten: `ZoneKind::Toxic =>` checks
      `world.get(x, y).biome == Biome::Palude` (or equivalent), not a rectangle
      `.contains`.
- [ ] The comment on `species_present_in_zone` (`objectives.rs:318-320`) — explaining
      *why* the check uses fixed geometry instead of live `Cell.toxicity` (diffusion
      erodes the meaning of "is in the zone" over time) — updated to explain the new
      invariant instead: `Cell.biome` is a stable per-cell classification set at
      generation time (task 110), not recomputed from `toxicity` each tick, so it
      doesn't have the same erosion problem. Don't lose this reasoning, it's the
      reason the check is correct.
- [ ] Every test that constructs `ToxicZoneBounds` by hand
      (`objectives.rs:482, 591-593, 623, 627, 676-683, 710` and any others found by
      `grep -n ToxicZoneBounds src/*.rs`) rewritten to set `Cell.biome = Biome::Palude`
      on the relevant cells instead.
- [ ] `draw_toxic_zone` (`render.rs:582+`) removed — coordinate with task 112 so it's
      deleted exactly once (whichever of 112/113 lands second does the deletion; the
      other leaves a one-line note instead of duplicating the change).
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test` passes,
      including the `SurviveIn`/toxic-zone objective tests under their new Palude-based
      setup.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:345-403` | `place_toxic_zone`, `ToxicZoneBounds` — removed. |
| `src/objectives.rs:318-336` | `species_present_in_zone`, `cell_in_zone`, `ZoneKind::Toxic` — rewritten to check biome membership. |
| `src/objectives.rs:482, 591-593, 623, 627, 676-683, 710` | Tests constructing `ToxicZoneBounds` by hand — rewritten. |
| `src/config.rs:141-146, 572-580` | `toxic_zone_*` config fields — removed/repurposed. |
| `src/render.rs:582+` | `draw_toxic_zone` — removed (coordinate with 112). |

---

## 🔗 Dependencies

- **Depends on**: 110 (Palude must exist as a `Biome` variant, stably assigned on
  `Cell.biome`).
- **Blocks**: none.
