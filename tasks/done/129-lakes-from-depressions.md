# Task 129 — Lakes derived from terrain depressions

> **ID**: `129`
> **Category**: Refactor (worldgen)
> **Priority**: 🟢 P3
> **Estimate**: ~3h
> **Assigned to**: done
> **Session**: 2026-08-13 (Phase 7 of the worldgen pipeline reassessment,
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §10.4)
> **Implemented**: 2026-08-19

---

## 🎯 Objective

Lago is currently placed by an organic-masked but still context-blind
search (task 123): a random center position, accepted once it clears a
placeable-land floor, with no relationship to the terrain's actual
drainage. Once task 127 computes flow accumulation and fills depressions to
route rivers, those depressions are real, terrain-grounded low points that
water would actually pool in — promoting suitably large/deep ones to
`Biome::Lake` is more credible than an independent random search, and lets
rivers terminate into lakes that make geographic sense instead of two
unrelated systems coexisting on the same map.

---

## 📋 Acceptance Criteria

- [x] During task 127's depression-fill pass, depressions above a
      configurable size/depth threshold are recorded (not just filled and
      discarded) — position, footprint, fill depth.
- [x] `place_feature_biomes` (or a new step run alongside it) promotes
      qualifying recorded depressions to `Biome::Lake`, writing the same
      target scalars task 111 already defined for Lago, over the
      depression's actual footprint (not a synthetic organic-disk mask —
      the depression's shape *is* the natural footprint here, no need for
      123's angular-distortion technique on top of it).
- [x] Task 123's organic-masked search (`place_feature_organic` applied to
      Lago) becomes a **fallback**: run only if depression-derived lakes
      don't reach a configurable minimum count/coverage for this world
      (mirrors the keep-best-seen/validation-retry pattern already used
      throughout worldgen, e.g. `generate_terrain`'s placeable-fraction
      floor). Document this explicitly — the fallback exists so a
      low-relief world (few real depressions) doesn't end up with zero
      lakes.
- [x] Crater and Distesa di cristalli keep using task 123's organic-mask
      search unchanged — this task only changes Lago's placement source.
- [x] Test: on a sample of seeds, depression-derived lakes correlate with
      locally low `flow_accumulation`-adjacent terrain (a lake cell should
      not be, e.g., on a local elevation maximum) — a sanity check that the
      derivation is actually terrain-grounded, not coincidentally similar
      to the old random search.
- [x] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| Task 127's depression-fill implementation | Source of candidate depressions — read its own file for where filled-but-discarded depressions would need to start being recorded instead. |
| `src/world.rs:837-906` | `place_feature_biomes` — where Lago's placement call is swapped for the depression-derived path (with 123's search as fallback). |
| `src/config.rs` (`BiomeConfig`) | New depression-to-lake threshold fields; existing `lake_*` fields become fallback-only parameters. |

---

## ⚠️ Constraints and Caveats

- **Ordering**: this task must run after 127's flow-accumulation/
  depression-fill pass produces real depression data, and after 123's
  organic-mask mechanism exists to serve as fallback.
- **No magic numbers**: promotion thresholds in config.
- **Determinism**: depression selection is derived from already-
  deterministic terrain data; only the fallback path (if triggered) draws
  from `LAKE_SEED_OFFSET`, unchanged from task 123.

---

## 🔗 Dependencies

- **Depends on**: 123 (fallback search), 127 (depression data to promote).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/129-lakes-from-depressions.md)"$'\n\nExecute this task in the current project.'
```

---

## ✅ Implementation notes (2026-08-19)

- **`record_significant_depressions`** (new method): a Moore-adjacency
  connected-component flood fill over `fill_depressions()`'s output,
  identifying basin cells via `filled[idx] - elevation[idx] > BASIN_EPSILON`
  (`1e-4`, to exclude float noise from cells that were never really
  filled). Each qualifying component's footprint is kept as-is — no
  synthetic mask — filtered by `lake_depression_min_size`/`_max_size`
  (cell count) and `lake_depression_min_depth` (max fill depth in the
  component).
- **Pipeline reorder**: `fill_depressions` is now computed once in
  `new_for_world` and shared by `record_significant_depressions` (Lake
  candidates) and `compute_hydrology` (flow routing, signature changed to
  take `filled: &[f32]` instead of recomputing it internally). Both moved
  ahead of `classify_biomes`/`place_feature_biomes`, since Lake placement
  now needs depression data before the feature pass runs. Neither
  `fill_depressions` nor `compute_hydrology` ever depended on biome data,
  so the reorder is safe — see the doc comment on the `new_for_world` call
  site.
- **Promotion-then-fallback**: `place_feature_biomes` now takes
  `lake_depressions: &[Vec<usize>]`. Each qualifying footprint (skipped
  wholesale if any cell is already `reserved`) is promoted directly to
  `Biome::Lake` with task 111's scalars (`lake_temperature`/`lake_light`/
  `lake_toxicity`). If fewer than `lake_min_depression_count` depressions
  were promoted, the old `place_feature_organic` organic-mask search runs
  as a fallback, unchanged from task 123's implementation.
- **Threshold calibration**: `lake_depression_min_size=10`,
  `max_size=100`, `min_depth=0.01` were chosen from a temporary
  `#[ignore]`d scratch diagnostic (added, measured across 25 seeds, then
  removed before committing) that histogrammed `fill_depressions()`'s
  connected-component sizes/depths — components ranged from single-digit
  float-noise up to 600+ cell whole drainage basins; the chosen band
  captures lake-sized basins while excluding both noise and valley-scale
  components.
- **Acceptance criterion's correlation test**: added
  `lake_cells_are_usually_not_local_elevation_maxima` (checks that under
  5% of Lake cells across 40 seeds are local elevation maxima among their
  Moore neighbours) plus `lake_footprint_is_never_absurdly_small` (a
  minimum-coverage sanity check). `feature_biomes_never_overlap_each_other`
  had Lake removed from its old fixed-radius disk-area checks, since a
  depression footprint has no relation to `lake_radius` any more.
- **`is_river`/Lake interior overlap checked post-hoc** (advisor review):
  since flow accumulation now runs before Lake exists, there was a
  concern rivers might route through the interior of a Lake footprint
  rather than just reaching its edge. Measured empirically across 40
  seeds: only 0.3% of Lake cells (8/2557) carry `is_river == true` —
  `fill_depressions` already routes flow across the filled basin surface
  rather than accumulating through the depression interior, so this is
  not a problem in practice.
- **GDD §5.10 updated** to reflect that `Lake` is now derived, not
  explicitly placed like `Crater`/`CrystalField` — the two-stage
  description previously implied all feature biomes were placement-based.
- Notes added to tasks 130 and 131 flagging two forward-looking couplings
  this reorder/refactor creates: 130 can now read `Cell.is_river`/
  `flow_accumulation` inside `classify_biomes` for the first time; 131, if
  it changes `swamp_score`'s inputs to `soil_moisture`, must also update
  `compute_macro_regions`'s region-aggregate call to the same function
  (task 128 added a second call site there).
- All acceptance criteria met; `cargo build --all-targets`,
  `cargo test` (182 lib tests + all integration binaries, all green),
  `cargo clippy --all-targets -- -D warnings`, and `cargo fmt -- --check`
  all clean.
