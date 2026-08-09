# Task 072 — Terrain sea/land balance correction (playtest correction to 069)

> **ID**: `072`
> **Category**: Bugfix / balance (worldgen)
> **Priority**: 🟢 P3
> **Estimate**: ~1h
> **Assigned to**: Claude
> **Session**: 2026-08-09, raised directly by the user right after 069 shipped, comparing generated worlds against `redesign/terrain-map-elevation.svg`

---

## 🎯 Objective

Task 069 (multi-octave terrain noise) shipped with `sea_threshold: 0.36` and
`min_placeable_fraction: 0.55`. In practice this produced worlds where Sea
was rare (measured ~8% of cells over 30 seeds) and often nearly absent
entirely, because `generate_terrain`'s bounded-resample loop only rejects a
draw for having *too little* placeable land — it never rejects a draw for
being almost all land, so the accepted ensemble is systematically biased
toward land-heavy worlds. The user compared this against
`redesign/abiogenesis-terrain-map.md`'s reference mockup
(`terrain-map-elevation.svg`), which shows a compact, organic landmass
surrounded by a much larger void/Sea — "il mare adesso compare pochissimo"
(the sea barely shows up now).

A first retuning pass (pushing `sea_threshold` up, `min_placeable_fraction`
down) revealed a second, deeper problem: with only 3 low-frequency continent
waves, each world's raw elevation amplitude varies a lot by chance, so a
fixed threshold against the raw `[0, 1]` output gave wildly inconsistent
sea/land ratios across seeds — some worlds ended up almost all land, others
almost all sea, rather than a consistent "compact landmass in a larger sea"
read.

---

## 📋 What changed

- **`normalize_elevations` (`src/world.rs`)**: a new step in
  `generate_terrain`, applied after computing the raw elevation field and
  before classification. Rescales the field in place so each world's own
  min/max spans the full `[0, 1]` range. This makes `TerrainConfig`'s
  thresholds land in the same *relative* place in every world's own
  distribution regardless of the random draw's raw amplitude, fixing the
  seed-to-seed inconsistency the first retuning pass exposed. A degenerate
  all-equal field is left untouched (guards a divide-by-zero).
- **`TerrainConfig::default()` retuned** (`src/config.rs`): `sea_threshold`
  0.36→0.42, `hill_threshold` 0.53→0.62, `mountain_threshold` 0.7→0.8,
  `peak_elevation_threshold` 0.8→0.88, `min_placeable_fraction` 0.4→0.4
  (unchanged from the first pass), `continent_wave_count` kept at 3. Over a
  30-seed histogram this gives Sea ≈ 33% of cells on average with a much
  narrower per-seed spread (roughly 18–42% across the sample, vs. the prior
  near-0%-to-70%+ swings), Mountain ≈ 10%, and a healthy peak count.
- **`world_with_one_organism` test helper (`src/sim.rs`)**: force every
  cell's terrain to `Plain`/non-peak right after `SimWorld::new`. This test
  helper backs 12 pure energy-formula unit tests that have nothing to do
  with terrain; one of them
  (`photolithic_in_the_dark_eventually_dies`) started failing under the
  rebalanced terrain because task 067's reproduction placement gating now
  incidentally depended on whether seed 42's generated neighbours were
  placeable, changing the test's reproduction/energy dynamics. Forcing flat
  terrain for this helper removes the incidental dependency, matching task
  066's original intent that `Cell::terrain` default to `Plain` so existing
  tests keep working.

---

## 📋 Acceptance Criteria

- [x] `cargo clippy -- -D warnings` clean, `cargo fmt -- --check` clean.
- [x] `cargo test` clean (91 lib tests + all integration tests), including
      the previously-regressed `photolithic_in_the_dark_eventually_dies`.
- [x] `terrain_generation_is_deterministic_for_a_given_seed`,
      `terrain_varies_with_seed`, `placeable_land_fraction_floor_holds_across_seeds`,
      `peaks_only_occur_within_the_mountain_band` all still pass unmodified.
- [x] Verified visually via a real `cargo run` window on the user's machine
      (driven headlessly with `cliclick`/`screencapture`, several reseeds
      via the `r` key): generated worlds now show a clearly visible black
      Sea covering a substantial share of the frame, with an organic
      landmass/lake shape carved out of it — much closer to
      `terrain-map-elevation.svg`'s reference proportions than 069's initial
      defaults.
- [x] No changes to classification logic itself
      (`classify_elevation`, `is_placeable_kind`), task 067's placement
      gating, or rendering (`render.rs`, task 068) — only `TerrainConfig`
      threshold values, a new elevation-normalization step, and an
      unrelated test helper's terrain override.

---

## 🔗 Dependencies

- **Depends on**: 069 (this is a direct correction to its shipped defaults).
- **Blocks**: none.
