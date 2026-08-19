# Task 131 — Soil moisture (refines Swamp/Forest beyond the slope/water-distance proxy)

> **ID**: `131`
> **Category**: Feature (worldgen)
> **Priority**: 🟢 P3
> **Estimate**: ~3h
> **Assigned to**: done
> **Session**: 2026-08-13 (Phase 9 of the worldgen pipeline reassessment,
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §9.4)
> **Implemented**: 2026-08-19

---

## 🎯 Objective

Task 125 gives Swamp a causal basis using `slope` and `water_distance`
(task 124) as a stand-in for real saturation — reasonable as a first pass,
but a proxy: it has no notion of rainfall, evaporation, or drainage rate,
so a low-slope cell far from any water but in a very wet climate scores the
same as a low-slope cell in a dry one. Once task 126 adds `rainfall`, a
proper `soil_moisture` field (spec's §9.4 formula) replaces the proxy with
something that actually accounts for climate, not just geometry.

```text
soil_moisture =
    precipitation * rainfall_retention(slope)
    + river_proximity * river_moisture_bonus
    + lake_proximity * lake_moisture_bonus
    - evaporation(temperature)
    - drainage(slope, curvature)
```

(`curvature` isn't in scope — task 124 deliberately didn't add it; use
`slope` alone for the drainage term, a reasonable simplification the spec
itself allows by listing slope-only alternatives elsewhere.)

---

## 📋 Acceptance Criteria

- [x] `Cell` gains `soil_moisture: f32` (`[0, 1]`), computed once at
      generation time from `rainfall` (task 126), `slope`/`water_distance`
      (task 124), and `temperature` (existing), following the formula
      above (simplified: no `curvature` term, `river_proximity` folded
      into `water_distance` if task 127's rivers haven't landed yet — flag
      in the implementation which terms are stubbed vs real, don't silently
      approximate without noting it).
- [x] Task 125's Swamp score is updated to use `soil_moisture` instead of
      (or as a refinement on top of) the `slope`/`water_distance` proxy it
      shipped with — the proxy's shape shouldn't be thrown away wholesale,
      `soil_moisture` should subsume it (it already includes both as
      inputs), not duplicate the same reasoning twice under two different
      names.
- [x] Forest's score (also in task 125) may optionally gain a
      `soil_moisture` term instead of/alongside `light` for its moisture
      condition — evaluate whether this improves or just shuffles the
      existing Forest distribution; not a hard requirement if `light`
      already does the job adequately once rainfall exists.
- [x] New config knobs (retention/evaporation/drainage coefficients) in
      `SimConfig`, mirrored in `assets/config/sim_config.ron`.
- [x] Test: multi-seed check that mean `soil_moisture` correlates
      positively with `rainfall` and negatively with `temperature`/`slope`
      — a relational sanity check (spec §18.5 style), not just "the field
      is populated."
- [x] Re-run task 125's biome-distribution histogram test on the same
      seeds — confirm the Swamp/Forest shift (if any) from switching to
      `soil_moisture` is a deliberate improvement, not an unreviewed
      regression.
- [x] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:329-357` | `Cell` struct — add `soil_moisture`. |
| `src/world.rs:743-818` | `classify_biomes` — Swamp/Forest scores updated to consume it. |
| Task 124/126's fields | Direct inputs (`slope`, `water_distance`, `rainfall`). |

---

## ⚠️ Constraints and Caveats

- **No magic numbers**: retention/evaporation/drainage coefficients in
  config.
- **Determinism**: pure function of already-deterministic fields, no new
  RNG stream needed.
- Don't reintroduce a second, parallel "is this cell wet" concept — this
  field should be the single source of truth Swamp's score reads, replacing
  (not sitting alongside) the cruder proxy from task 125.
- **`swamp_score` has two call sites since task 128**: the per-cell pass in
  `classify_biomes`, and `compute_macro_regions`'s region-aggregate pass
  (which evaluates the same score once per macro-region using
  `mean_slope`/`mean_water_distance`, noise fixed at `0.0`). If this task
  swaps `slope`/`water_distance` for `soil_moisture` as `swamp_score`'s
  input, `compute_macro_regions`'s aggregate must accumulate a matching
  `mean_soil_moisture` (not keep passing `mean_slope`/`mean_water_distance`
  into a signature that no longer wants them) — otherwise the region-bias
  pass silently drifts from the per-cell pass it's supposed to reinforce.

---

## 🔗 Dependencies

- **Depends on**: 124 (`slope`/`water_distance`), 125 (the Swamp/Forest
  scores this refines), 126 (`rainfall` as the core input — without it,
  this task has nothing new to add over 125's existing proxy).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/131-soil-moisture.md)"$'\n\nExecute this task in the current project.'
```

---

## ✅ Implementation notes (2026-08-19)

- **Both river and lake proximity are real, not stubbed.** Task 127's
  rivers had already landed, so `river_bonus` reads `Cell.is_river`
  directly via a fresh multi-source BFS (`bfs_distance_from`). Lake
  proximity goes further than the task file's own fallback plan
  anticipated: task 129's `record_significant_depressions` already knows
  *where* Lake will be placed before `place_feature_biomes` paints
  `Biome::Lake` onto any cell, so `lake_depressions`' raw footprints
  (flattened into one seed set) feed a real BFS distance field
  (`bfs_distance_from_indices`, a new sibling of `bfs_distance_from`
  seeded from explicit indices instead of a `Cell` predicate) — unlike the
  persisted `Cell.water_distance` field (task 132's finding), this doesn't
  need to wait for Lake to exist as a queryable `Biome`.
- **Pipeline placement**: `compute_soil_moisture` runs right after
  `compute_hydrology` and before `classify_biomes` in `new_for_world` —
  all its inputs (`rainfall`, `is_river`, `lake_depressions`) are ready by
  then, and `classify_biomes` needs the finished field.
- **Swamp score replaced, not extended**: `swamp_score` now takes
  `soil_moisture: f32` directly (was `slope`/`water_distance`) — a single
  smooth rise past `swamp_soil_moisture_min`, since `soil_moisture` already
  incorporates both original inputs. `compute_macro_regions`'s
  region-aggregate call (task 128's second `swamp_score` call site, flagged
  in this file before implementation) was updated in the same pass:
  `mean_soil_moisture` replaces the old `mean_slope`/`mean_water_distance`
  pair, and `sea_proximity` is no longer threaded through
  `compute_macro_regions`/`classify_biomes` at all, since nothing else
  needed it.
- **Forest decision**: left unchanged, using `light`. `light` already
  drives Forest's moisture condition adequately once `rainfall`/
  `soil_moisture` feed Swamp instead — evaluated but not worth the
  duplication risk the task file itself flagged as optional.
- **Threshold calibration** (`swamp_soil_moisture_min`): the naive first
  guess (`0.35`, roughly the sampled median) produced 41% Swamp coverage
  of `Plain` cells — Swamp *dominating* instead of reading as a minority
  wetland texture. Raising to `0.65` fixed the fraction (8.2%, close to
  Forest's own 7.0%) but dropped `some_swamp_cells_are_toxic_across_seeds`'
  existing balance guarantee (≥75% of seeds need a toxic Swamp cell) to
  36/60 (60%) — Swamp got too rare *per seed* even though the aggregate
  fraction looked right. Settled on `0.5`: 19% Swamp coverage, 47/60 (78%)
  seeds clearing the toxic-swamp balance test. All three values were
  measured via a temporary scratch example, removed before committing.
- **New relational test**:
  `soil_moisture_correlates_with_rainfall_temperature_and_slope_as_designed`
  (20 seeds, above/below-median bucket comparison — a correlation-sign
  check, not a linearity assumption).
- **Biome-distribution re-check**: no dedicated histogram test exists in
  the codebase for task 125 to "re-run" (searched; none found) — this
  criterion was instead satisfied by the calibration measurements above
  (Swamp 19% / Forest 7.0% of Plain cells, a deliberate, reviewed choice,
  not a silent drift).
- **`classify_biomes_reads_the_persisted_slope_field` split into two
  tests**: its original Swamp-via-slope assertion no longer holds (Swamp
  reads `soil_moisture`, not `slope`, inside `classify_biomes` — forcing
  `cell.slope` after generation has no effect on it). Narrowed to its
  still-true BareRock claim, plus a new
  `classify_biomes_reads_the_persisted_soil_moisture_field` covering the
  positive/negative Swamp case directly.
- All acceptance criteria met; `cargo build --all-targets`, `cargo test`
  (185 lib tests + all integration binaries, all green), `cargo clippy
  --all-targets -- -D warnings`, and `cargo fmt -- --check` all clean.
