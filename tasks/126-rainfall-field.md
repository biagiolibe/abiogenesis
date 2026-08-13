# Task 126 — Rainfall field (orographic lift, rain shadow)

> **ID**: `126`
> **Category**: Feature (worldgen)
> **Priority**: 🟢 P3
> **Estimate**: ~4h
> **Assigned to**: unassigned
> **Session**: 2026-08-13 (Phase 5a of the worldgen pipeline reassessment
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §9.2/§9.3 —
> see task 123 for Phase 1 and the session's overall diagnosis)

---

## 🎯 Objective

Nothing in the current pipeline represents precipitation. `light` currently
does double duty as both illumination and (via `desert_light_min`) an
aridity proxy — the spec's §1.2 complaint, still present even after task
125 moves Swamp off `toxicity`. A real `rainfall` field, driven by the
world's existing global wind vector and elevation (orographic lift/rain
shadow), gives later biome work (not scoped here — see Non-Goals) an actual
causal basis for aridity instead of overloading `light`.

**This task adds the field only.** It is deliberately scoped like task 124
(additive, not consumed by classification yet) — introducing both a new
physical field *and* rewiring what makes a cell a Desert/Forest in the same
task would make either change hard to review or roll back independently.

---

## 📋 Acceptance Criteria

- [ ] `Cell` gains `rainfall: f32` (`[0, 1]`, following the existing scalar
      convention).
- [ ] Computed once at generation time from:
      - **ocean proximity**: reuse `water_distance` (task 124) or
        `sea_distance_field` — moisture is highest near water, falls off
        with distance.
      - **orographic lift**: the existing global `wind` vector (already
        computed in `apply_environment_sources`, `world.rs:1018-1023` —
        reuse the same direction, don't redraw it from a new RNG call,
        since rainfall must be consistent with the same wind that already
        shapes temperature) crossed with the local elevation gradient
        (`slope`'s direction, not just magnitude — may need a signed
        gradient vector alongside/instead of `Cell.slope`'s scalar
        magnitude from task 124; if so, compute it locally here rather
        than changing `Cell.slope`'s meaning).
      - **rain shadow**: moisture depletes as the field crosses high
        ground along the wind direction, leaving less on the leeward side.
      Use a **single-pass approximation**, not the spec's iterative
      per-tick-of-generation advection loop (§9.3's `for _ in
      0..wind_steps`) — e.g. integrate accumulated upwind elevation gain
      along each cell's wind-aligned ray. A full iterative solver is a
      possible future refinement, not required for a first credible field,
      and keeps generation time bounded and easy to reason about.
- [ ] New `ClimateConfig`-style knobs (condensation rate, rain shadow
      strength, wind-alignment weighting) added to `SimConfig`, mirrored in
      `assets/config/sim_config.ron`. Reuse `EnvironmentConfig`/
      `SourceConfig` if a field fits naturally there instead of a new
      struct — don't create a new config struct for two or three fields
      when an existing one already groups climate-adjacent config.
- [ ] Field computed in the same generation step as (or immediately after)
      `apply_environment_sources`, since it needs that step's `wind`
      vector and finalized `temperature`/`light` are unaffected either way.
- [ ] Test: for a handful of seeds, the mean `rainfall` on the geometric
      leeward side of the tallest mountain range (found via `is_peak`
      cells' position relative to `wind`) is measurably lower than the
      windward side — the credibility check the spec's §18.5 calls for
      ("precipitazione minore oltre le montagne"), not just "the field is
      populated."
- [ ] `rainfall` is **read nowhere else yet** — `classify_biomes`,
      `Desert`/`Tundra`/`Forest` scoring (task 125), and rendering all stay
      untouched. Confirmed by the same before/after snapshot approach task
      124 used.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes.

---

## 🚫 Non-Goals

- Feeding `rainfall` into biome classification (replacing `light` as the
  aridity proxy in task 125's Desert/Forest/Tundra scores). That's a
  natural next step but a separate, reviewable task once this field exists
  and its output has been sanity-checked across seeds.
- Soil moisture / drainage refinement of Swamp using `rainfall` (spec
  §9.4) — task 125 already gives Swamp a slope/water-distance basis;
  folding rainfall in is a future refinement, not required here.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:1014-1066` | `apply_environment_sources` — source of the `wind` vector to reuse; the new rainfall step likely lives right after this. |
| `src/world.rs:329-357` | `Cell` struct — add `rainfall` here. |
| `src/world.rs:1079` | `sea_distance_field` / task 124's `water_distance` — ocean-proximity input. |

---

## ⚠️ Constraints and Caveats

- **Determinism**: no new RNG draws needed — `rainfall` is a deterministic
  function of already-generated fields (elevation, wind, water distance).
  If it does need its own randomness (e.g. a small noise term for
  variation), use a new dedicated seed offset, never reuse another stage's
  stream.
- **Performance**: a single-pass integration along wind-aligned rays over
  a 128×80 grid must stay cheap — no per-cell iterative solver in this
  task (see the single-pass note above).
- **No magic numbers**: every coefficient in `SimConfig`.

---

## 🔗 Dependencies

- **Depends on**: 124 (`water_distance`), and reads the existing `wind`
  vector from `apply_environment_sources` (no code dependency beyond
  ordering).
- **Blocks**: 127 (flow accumulation/rivers need a rainfall field to route).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/126-rainfall-field.md)"$'\n\nExecute this task in the current project.'
```
