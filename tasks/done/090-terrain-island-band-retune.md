# Task 090 — Terrain island-band retune: organic coastlines, less sea

> **ID**: `090`
> **Category**: Worldgen / Balance
> **Priority**: 🟢 P3
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-10 (user-reported live, playing with task 085/086's
> sea-coolant fix: `Sea` was generated "spesso in quantità eccessiva e in
> forme non troppo credibili" — a screenshot showed several same-sized,
> perfectly isolated near-circular `Sea` blobs scattered inland, "polka-dot
> lakes" rather than a credible connected coastline. The excess sea also
> diluted task 085's heat-source visibility, since more sea meant more of
> the map running cold via `sea_coolant_radius`.)

---

## 🎯 Objective

Retune `TerrainConfig`'s island wave band (task 069) and `sea_threshold`
so generated worlds read as organic connected coastlines instead of a
regular field of same-sized isolated lakes, and so `Sea` covers a more
plausible fraction of the grid.

---

## 🔍 Diagnosis

A throwaway diagnostic (ASCII terrain dump + a 30-seed Sea/Plain/Hill/
Mountain/peak histogram, same technique task 069 itself used, removed
before closing this task) isolated the cause: setting
`island_blend_weight` to `0.0` made the polka-dot pattern vanish entirely,
leaving a single smooth diagonal tilt from the continent band alone.
Restoring the default `island_blend_weight: 0.45` at `island_wave_count: 6`
reproduced the pattern immediately. Root cause: `wave_band_sum` sums `N`
plane waves of random direction/phase and divides by `N` — with only 6
waves, the sum is dominated by low-order interference between a handful of
sinusoids, which reads as a fairly regular, periodic pattern of lobes
(closed contours of similar size, roughly evenly spaced) rather than
organic noise. The continent band (task 069) uses the same technique at
`continent_wave_count: 3`, but is unaffected by this complaint since it's
never thresholded on its own to produce isolated blobs — it only sets the
one broad tilt the island band overlays detail onto.

Separately, `sea_threshold: 0.42` was measured (via the same histogram) to
classify ~33% of cells as `Sea` across a 30-seed sample — high enough that
task 085's `sea_coolant_radius` (12 cells) reached a large fraction of the
map, which is what made the heat-source model read as "washed out" on top
of the shape complaint.

---

## 📋 Acceptance Criteria

- [x] `TerrainConfig::island_wave_count` raised (`6` → `16`): more summed
      waves average toward a less periodic, more organic-looking field
      (the central-limit argument `wave_band_sum`'s own `/waves.len()`
      normalization relies on).
- [x] `TerrainConfig::island_blend_weight` raised (`0.45` → `0.55`) to
      compensate for the smaller typical per-wave contribution at a higher
      wave count, keeping the island band's visual impact comparable to
      before rather than fading out.
- [x] `TerrainConfig::sea_threshold` lowered (`0.42` → `0.34`) to bring
      total `Sea` coverage down from ~33% to ~24% across a 30-seed sample —
      re-verified the retuned values don't regress `min_placeable_fraction`
      (placeable land rose, not fell) or collapse `Mountain`/peak reachability
      (peaks: 78 vs 81 baseline over the same 30 seeds — no collapse, the
      exact failure mode task 069 warned about when retuning wave counts
      without re-checking the histogram).
- [x] `assets/config/sim_config.ron` hand-mirrored to match every
      `TerrainConfig` default change.
- [x] `cargo test` passes in full, including `tests/balance.rs`'s seed-survey
      tests (empirically tuned against the old land/sea distribution — less
      sea means less coastal cooling means a warmer map on average, a real
      risk to `env_fit`-driven population dynamics, not just cosmetic).
- [x] `cargo clippy -- -D warnings` and `cargo fmt` clean.
- [x] No changes to `generate_terrain`'s algorithm, `classify_elevation`,
      `is_placeable_kind`, or any other threshold (`hill_threshold`,
      `mountain_threshold`, `peak_elevation_threshold`, `continent_*`) —
      this task retunes island-band + sea-threshold *values* only.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/config.rs` | `TerrainConfig::default()` — the three retuned values. |
| `assets/config/sim_config.ron` | Hand-mirrored to match. |

---

## 🧩 Technical Context

`generate_terrain` (`src/world.rs`) sums a continent band (task 066) and an
island band (task 069) into one elevation field, normalizes it to `[0,1]`
per world (`normalize_elevations`), then classifies each cell by threshold
(`classify_elevation`). `wave_band_sum(waves, nx, ny) = mean(sin(freq *
(nx*dir_x + ny*dir_y) + phase))` over the band's own waves. This task
changes only the island band's wave count/weight and `sea_threshold` —
`generate_terrain`'s bounded-retry loop (`TerrainConfig::
max_generation_attempts`, `min_placeable_fraction`) and every other
threshold are untouched.

---

## ⚠️ Constraints and Caveats

- **Verification is empirical, not analytical**: task 069's own history
  records a near-identical trap (raising wave count silently collapsed
  Mountain/peak cells to near-zero, caught only by a before/after
  histogram, not by eyeballing one seed's map). This task's histogram check
  is not optional scaffolding — it's the actual acceptance evidence.
- The live visual read (does this actually look like organic coastlines
  now, not just "different numbers in a histogram") is the user's own
  `cargo run` check, not something automatable from this session — the
  agent has no working `screencapture` permission in this environment.

---

## 🔗 Dependencies

- **Depends on**: 069 (island-band technique this retunes), 085/086
  (surfaced the complaint during their own live playtest).
- **Blocks**: none.
