# Task 132 — [DECISION] `Cell.slope`/`Cell.water_distance` are computed every generation and read by nothing

> **ID**: `132`
> **Category**: Decision / correction (worldgen)
> **Priority**: 🟢 P3
> **Estimate**: ~30min to decide, 0-3h to act depending on the choice
> **Assigned to**: unassigned
> **Session**: 2026-08-19 (found during advisor review after tasks 123-126 shipped)

---

## 🎯 What happened

Task 124 added `Cell.slope` and `Cell.water_distance`, explicitly so task 125
(biome scoring) and later hydrology work could read them. Task 124's own
ordering was: `compute_geomorphology` runs *after* `place_feature_biomes`,
because `water_distance`'s BFS needs `Biome::Lake` cells to exist as a
source.

Task 125 (Swamp's drainage score) needed the *same kind of data* — but
needed it *inside* `classify_biomes`, which runs *before*
`place_feature_biomes` (by design: `place_feature_biomes` overrides
whatever Stage A/B produced, so it has to run after). That's a genuine
ordering conflict: `classify_biomes` cannot read `Cell.water_distance`
(Lake-aware) because it isn't populated yet at that point in the pipeline.

Resolution taken (documented in `classify_biomes`'s own doc comment and in
task 125/126's `tasks/done/` notes): both task 125 (Swamp score) and task
126 (rainfall) compute their **own local, Sea-only proxy** for
slope/water-distance inline, via `elevation_slope`/`sea_distance_field`,
instead of reading the persisted `Cell.slope`/`Cell.water_distance` fields.

**Net result:** `Cell.slope` and `Cell.water_distance` are computed once
per world generation (a real, if small, cost) and read by **nothing**.
Every consumer that exists so far needed the data earlier in the pipeline
than these fields are populated, and worked around it locally instead.

## ⚠️ Why this matters for later tasks

`tasks/127-hydrology-rivers.md`, `128-macro-region-biomes.md`,
`130-mountain-sub-banding.md`, and `131-soil-moisture.md` are all scoped
assuming `Cell.slope`/`Cell.water_distance` are *the* place later worldgen
code reads this data from. If they're scoped to read the persisted fields
without re-checking pipeline order first, they may hit the exact same
conflict task 125 did — or worse, silently read stale/zero values if they
run *before* `compute_geomorphology` without realizing it.

## 🔀 Options (not decided yet — pick one before 127/128/130/131 start)

1. **Leave as-is.** The persisted fields remain correct and available for
   any *future* consumer that genuinely runs after `place_feature_biomes`
   (e.g. task 127's hydrology, if it turns out to run late enough). Every
   task that needs the data earlier keeps computing its own local proxy,
   documented each time. Cheapest, but repeats the same small
   recomputation cost (`O(cells)`, currently negligible) each time, and
   requires every future task author to re-derive this same ordering
   analysis instead of finding a settled answer.
2. **Move slope computation earlier.** `slope` only depends on `elevation`
   (final right after `generate_terrain`), no Lake dependency — split
   `compute_geomorphology` into `compute_slope` (runs before
   `classify_biomes`) and `compute_water_distance` (unchanged, after
   `place_feature_biomes`). This would let task 125's `classify_biomes`
   (and any future Stage-B consumer) read `Cell.slope` directly instead of
   recomputing it locally — removes one of the two local-proxy
   workarounds. `water_distance`'s Lake-awareness conflict remains either
   way.
3. **Drop Lake from `water_distance`, reorder placement.** If Lake's
   contribution to "near water" is judged not worth the ordering
   complexity, `water_distance_field` could go back to Sea-only (same as
   `sea_distance_field`, arguably redundant with it at that point) and run
   before `classify_biomes` — full availability, no local proxies anywhere,
   but loses the "swamps form near lakes too" credibility this field was
   built for.
4. **Re-order feature placement.** Have `place_feature_biomes` place *only*
   `Biome::Lake` before `classify_biomes` (Crater/CrystalField/VolcanicVent
   can still come after), so Lake exists early enough for a
   pre-`classify_biomes` `water_distance` pass. Bigger structural change,
   touches `place_feature_biomes`'s "must run after `classify_biomes`,
   overrides Stage A/B" invariant for one biome only — needs its own
   careful review, not a quick fix.

## 📋 Acceptance Criteria (once a direction is picked)

- [ ] Decision recorded here (which option, and why).
- [ ] If option 2/3/4: implemented, with `classify_biomes`'s local-proxy
      workaround removed in favor of reading the now-earlier-available
      persisted field(s).
- [ ] Tasks 127/128/130/131 re-checked against whatever ordering this
      lands on, before any of them start — update their own files if the
      ordering assumption they were scoped under has changed.
- [ ] `cargo clippy -- -D warnings` / `cargo fmt` / `cargo test` clean if
      code changes.

---

## 🔗 Dependencies

- **Depends on**: 124, 125, 126 (done — this task documents a gap found
  after they shipped).
- **Blocks**: nothing directly, but 127/128/130/131 should have this
  resolved (or explicitly re-confirmed against option 1) before they start,
  since they're the tasks whose scoping assumed the persisted fields were
  the consumption point.
