# Task 131 — Soil moisture (refines Swamp/Forest beyond the slope/water-distance proxy)

> **ID**: `131`
> **Category**: Feature (worldgen)
> **Priority**: 🟢 P3
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-13 (Phase 9 of the worldgen pipeline reassessment,
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §9.4)

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

- [ ] `Cell` gains `soil_moisture: f32` (`[0, 1]`), computed once at
      generation time from `rainfall` (task 126), `slope`/`water_distance`
      (task 124), and `temperature` (existing), following the formula
      above (simplified: no `curvature` term, `river_proximity` folded
      into `water_distance` if task 127's rivers haven't landed yet — flag
      in the implementation which terms are stubbed vs real, don't silently
      approximate without noting it).
- [ ] Task 125's Swamp score is updated to use `soil_moisture` instead of
      (or as a refinement on top of) the `slope`/`water_distance` proxy it
      shipped with — the proxy's shape shouldn't be thrown away wholesale,
      `soil_moisture` should subsume it (it already includes both as
      inputs), not duplicate the same reasoning twice under two different
      names.
- [ ] Forest's score (also in task 125) may optionally gain a
      `soil_moisture` term instead of/alongside `light` for its moisture
      condition — evaluate whether this improves or just shuffles the
      existing Forest distribution; not a hard requirement if `light`
      already does the job adequately once rainfall exists.
- [ ] New config knobs (retention/evaporation/drainage coefficients) in
      `SimConfig`, mirrored in `assets/config/sim_config.ron`.
- [ ] Test: multi-seed check that mean `soil_moisture` correlates
      positively with `rainfall` and negatively with `temperature`/`slope`
      — a relational sanity check (spec §18.5 style), not just "the field
      is populated."
- [ ] Re-run task 125's biome-distribution histogram test on the same
      seeds — confirm the Swamp/Forest shift (if any) from switching to
      `soil_moisture` is a deliberate improvement, not an unreviewed
      regression.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
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
