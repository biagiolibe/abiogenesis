# Task 069 — Multi-octave terrain noise (macro-continents + small islands)

> **ID**: `069`
> **Category**: Feature (worldgen)
> **Priority**: 🟢 P3
> **Estimate**: ~2-3h
> **Assigned to**: unassigned
> **Session**: 2026-08-09, raised directly by the user after reviewing task 068's rendering

---

## 🎯 Objective

Task 066's elevation field sums `noise_wave_count` (4) plane waves all drawn
from the same frequency range (`1.0..3.5`) — effectively a single-octave
noise. That produces one dominant blob shape per world: either a few large
landmasses or scattered small islands, never both at once, because there's
no frequency separation between "shapes the continents" and "adds small
detail on top of them."

Split the field into two frequency bands — a small set of low-frequency
waves that shape macro-continents, plus a small set of higher-frequency
waves layered on top with a smaller weight that add small islands/coastline
detail — so a single generated world can show both large continents and
scattered small islands, closer to what "un continente visibile con qualche
isola piccola" actually looks like. This is a follow-up refinement to the
terrain map redesign (066-068), not part of the original design doc's
scope — it was raised directly by the user while discussing task 068's
rendering, not a pre-planned task.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors; `cargo clippy -- -D warnings` is clean.
- [ ] `terrain_waves`/`terrain_elevation` (or their replacements) draw two frequency bands from the same derived RNG stream (`self.seed ^ TERRAIN_SEED_OFFSET`, unchanged) — a low-frequency band shaping continent-scale shape, a higher-frequency band layered on top with a smaller blend weight for island/detail scale.
- [ ] Band wave counts, frequency ranges, and the detail-band blend weight are new fields on `TerrainConfig` (`src/config.rs`), not hardcoded — no magic numbers, per project convention.
- [ ] Generation stays fully deterministic for a given seed (existing `terrain_generation_is_deterministic_for_a_given_seed` test, and any new tests, must pass) and `min_placeable_fraction` resampling (`generate_terrain`'s bounded-attempts loop) still works against the new field shape.
- [ ] `TerrainConfig::default()` values are chosen so a generated world visibly shows both a large connected landmass and one or more small (a few cells) separate islands — verified visually via `cargo run`, not just by the classifier thresholds.
- [ ] Existing terrain/worldgen unit tests (`world.rs`'s `terrain_*` tests, `worldgen.rs` tests referencing terrain) are updated if the field's internal shape changed, and still pass; full `cargo test` is clean.
- [ ] No changes to `sea_threshold`/`hill_threshold`/`mountain_threshold`/`peak_elevation_threshold` classification logic itself, `is_placeable_kind`, or task 067's placement gating — this task only changes how the elevation *field* is generated, not how it's classified or gated.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `TerrainWave`, `terrain_waves`, `terrain_elevation`, `generate_terrain` — all the field-generation changes land here. |
| `src/config.rs` | `TerrainConfig` — new tunable fields for the two bands. |
| `redesign/abiogenesis-terrain-map.md` | Original design rationale (background only — this task's scope isn't in it). |

---

## 🧩 Technical Context

- **Current behavior**: `terrain_waves(rng, count)` draws `count` waves (default 4) with random direction, frequency uniformly in `1.0..3.5`, and phase, all from one RNG stream. `terrain_elevation` sums all waves' contributions with equal weight and normalizes to `[0, 1]`. Single scale → single dominant shape per world.
- **Desired behavior**: two wave groups — e.g. a "continent" band (fewer waves, lower frequency range, full weight) and a "detail" band (a few waves, higher frequency range, smaller weight) — summed together (weighted) before normalizing, so continent-scale shape and island-scale detail coexist in the same field.

---

## 🔨 Suggested Implementation

1. Add `TerrainConfig` fields for the two bands: e.g. `continent_wave_count`, `continent_freq_min`/`continent_freq_max`, `island_wave_count`, `island_freq_min`/`island_freq_max`, `island_blend_weight`.
2. Extend `terrain_waves` (or split into two calls) to draw each band's waves from the same `rng` stream, in a fixed order, so determinism is preserved.
3. Extend `terrain_elevation` to sum both bands, weighting the island band by `island_blend_weight` before the final normalization.
4. Re-tune `TerrainConfig::default()`'s band parameters and, if needed, the existing `sea_threshold`/`hill_threshold`/`mountain_threshold` to get a visually convincing mix of continent + islands — this will need iteration via `cargo run`, not just picking numbers analytically.
5. Update/add unit tests in `world.rs` for the new field shape (determinism, seed variation, placeable-fraction resampling still converging).
6. Run `cargo run`, seed a few worlds with different seeds, visually confirm the "one continent + a few small islands" read holds across seeds, not just one lucky draw.

---

## ⚠️ Constraints and Caveats

- **Don't touch classification or gating** — `classify_elevation`, `is_placeable_kind`, and task 067's placement checks are out of scope; only the elevation field itself changes.
- **Determinism is non-negotiable**: both bands must draw from the single derived RNG stream (`TERRAIN_SEED_OFFSET`), in a fixed, seed-independent order — no `rand::rng()`, no HashMap iteration in the generation path.
- **No new palette/rendering changes** — task 068 already owns rendering; this task only changes the underlying data the renderer reads.

---

## 🔗 Dependencies

- **Depends on**: 066, 067, 068.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/069-multi-octave-terrain-noise.md)"$'\n\nExecute this task in the current project.'
```
