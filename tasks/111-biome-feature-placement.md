# Task 111 — Explicit placement for feature biomes

> **ID**: `111`
> **Category**: Feature (worldgen)
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-biomes.md`)

---

## 🎯 Objective

Some biomes proposed in `redesign/abiogenesis-biomes.md` can't be reliably derived
from thresholds on `temperature`/`light`/`toxicity` without risking collisions or
ambiguity with neighboring biomes — they need to be placed explicitly, the same way
`toxic_zone` already is. This task covers: **Cratere profondo**, **Distesa di
cristalli**, **Lago**, and **Bocca vulcanica**.

Bocca vulcanica needs no new placement logic — it hooks directly into the point heat
sources task 085 already generates.

---

## 📋 Acceptance Criteria

- [ ] Cratere profondo, Distesa di cristalli, Lago: each gets a bounded-retry
      placement pass, same pattern as `place_toxic_zone` (`world.rs:345-403` —
      candidate positions tried up to a config-bounded attempt count, best-seen kept
      if none clears the placeability floor). Reuse the pattern; don't invent a new
      one.
- [ ] Each placed feature **imposes** its target `temperature`/`light`/`toxicity`
      values (from the table in `redesign/abiogenesis-biomes.md`) on the cells it
      occupies and sets `Cell.biome` accordingly — it overrides whatever Stage
      A/B (task 110) would have produced there, it does not need to agree with it.
- [ ] Bocca vulcanica: cells in `SimWorld.heat_sources` (`world.rs:200`, populated by
      `place_heat_sources`, `world.rs:559-604`) get `Cell.biome = Biome::BoccaVulcanica`
      directly — no new placement search. Confirm the existing heat-source radius
      (`heat_source_radius`, `SimConfig`) is a sane footprint for a biome patch, not
      just a temperature falloff radius; if it's too broad for a biome-sized feature,
      cap the biome footprint independently from the temperature falloff footprint
      rather than changing `heat_source_radius` itself (that constant is load-bearing
      for task 085's balance, don't perturb it here).
- [ ] New config entries for the three new placements (size, placeable-fraction
      floor, max attempts), same style as `TerrainConfig`'s `*_toxic_zone_*` fields
      (`config.rs:572-580`).
- [ ] Test: no two feature biomes overlap the same cell in a single generation run;
      each placement's bounded-retry degrades gracefully (best-seen kept) the same
      way the existing `place_toxic_zone` tests verify.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test` passes.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:345-403` | `place_toxic_zone` — the bounded-retry pattern to replicate for the three new placements. |
| `src/world.rs:200, 559-604` | `heat_sources` / `place_heat_sources` — Bocca vulcanica hooks in here directly. |
| `src/config.rs:572-580` | `min_toxic_zone_placeable_fraction` / `max_toxic_zone_placement_attempts` — pattern for the new config fields. |

---

## 🔗 Dependencies

- **Depends on**: 110 (`Biome` enum and `Cell.biome` field must exist).
- **Blocks**: 112 (rendering needs every biome, areal and feature, already assigned).
