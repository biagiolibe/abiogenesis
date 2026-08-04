# Task 037 — `WorldParams` and difficulty curve

> **ID**: `037`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Phase 3 planning session

---

## 🎯 Objective

GDD §9 describes a difficulty curve: the first worlds have 5 active tags and a mild environment, gradually rising to ~8 active tags, nastier matrices, more extreme environments, and shorter era budgets. The example playthrough (§16) shows World 2 (second world) with **6** active tags — not a direct jump from 5 to 8, so the curve is a ramp across multiple worlds, not just two "early"/"late" levels.

`SimConfig` already has `active_tags_early/late` and `era_budget_early/late`, but no consumer uses them to interpolate — they're meant as the two endpoints of a curve that was never written. This task introduces that curve as a pure, headless-testable function, which the worldgen tasks (038, 039, 042) will consume.

---

## 📋 Acceptance Criteria

- [x] New `src/worldgen.rs` module with a pure function `pub fn world_params(world_index: u32, config: &SimConfig) -> WorldParams`.
- [x] `WorldParams` includes: `active_tag_count: u32`, `era_budget: u32`, `toxic_zone_width: u32`, `toxic_zone_height: u32`, `temperature_spread: f32`, `matrix_density: f32`, `objective_severity: f32`.
- [x] New `DifficultyConfig` in `SimConfig` (field `difficulty`) with `ramp_worlds`, `toxic_zone_width_late`, `toxic_zone_height_late`, `temperature_spread_late`, `matrix_density_late`, `objective_severity_early/late`. No magic numbers outside `SimConfig`.
- [x] **Literal criterion from GDD §16**: `world_params(1, &config).active_tag_count == 6` — verified by the `world_index_one_has_six_active_tags` test.
- [x] `world_params(0, &config)` produces exactly the current "early" values — verified by the `world_index_zero_matches_the_early_endpoints_exactly` test.
- [x] The ramp saturates at the `*_late` values after `ramp_worlds` worlds — verified by the `the_curve_saturates_at_the_late_endpoints_past_ramp_worlds` test (even 50 worlds past the ramp produce the same result, no overflow).
- [x] `world_params` is testable without constructing a `SimWorld`: it depends only on `crate::config::SimConfig`.
- [x] Unit tests: value at `world_index=0`, value at `world_index=1` (the 6-tag constraint), saturation past `ramp_worlds`, plus an additional test for the decreasing monotonicity of the era budget along the ramp.
- [x] `cargo clippy --all-targets -- -D warnings` clean, `cargo test` green (72 tests total, no regressions).

## Implementation summary

- `src/config.rs`: new `DifficultyConfig` (field `SimConfig::difficulty`) with `ramp_worlds: u32 = 3` (the minimum value that exactly reproduces the GDD §16 constraint of 6 tags at World 2) and the `*_late` endpoints for toxic zone, thermal spread, matrix density, objective severity.
- `src/worldgen.rs` (new): `WorldParams` and `world_params()`, linear interpolation (`ramp_fraction`/`lerp_u32`/`lerp_f32`) from the "early" endpoint (read from the existing `TagConfig`/`TimeConfig`/`EnvironmentConfig` configs) to the "late" endpoint (new fields in `DifficultyConfig`), saturated past `ramp_worlds`.
- `src/lib.rs`: exported `pub mod worldgen;`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/worldgen.rs` (new) | `WorldParams`, `world_params()`, unit tests. |
| `src/config.rs` | New endpoint fields for the environmental curve, `difficulty_ramp_worlds`. |
| `src/main.rs` | Registration of the new module (`mod worldgen;`) — no Plugin needed for this task, it's just a pure function. |

---

## 🧩 Technical Context

**Current `TimeConfig`** (`src/config.rs`, lines ~84-111):
```rust
pub struct TimeConfig {
    pub era_ticks: u32,               // 25
    pub era_budget_early: u32,        // 40
    pub era_budget_late: u32,         // 25
    pub point_budget_per_era: u32,    // 3
    pub action_costs: ActionCosts,
    pub era_tick_hz: f32,             // 20.0
}
```

**Current `TagConfig`** (`src/config.rs`, lines ~188-221):
```rust
pub struct TagConfig {
    pub global_tag_pool: u32,       // 10
    pub active_tags_early: u32,     // 5
    pub active_tags_late: u32,      // 8
    pub tags_per_species_min: u32,  // 1
    pub tags_per_species_max: u32,  // 3
    pub effect_intensity_min: i8,   // -2
    pub effect_intensity_max: i8,   // 2
    pub matrix_density: f32,        // 0.4
}
```
Both have `early`/`late` endpoints but **no consumer** interpolates them today — `SimWorld::new` only uses `active_tags_early`, `advance_tick` never reads `era_budget_*`.

`EnvironmentConfig` (not yet read in detail for this task — verify in `config.rs`) currently only has static values (fixed toxic zone size/position, fixed gradients), with no early/late endpoints: this task introduces them.

- **Current behavior**: no function converts "which world we're in" into concrete parameters — the very concept of "current world index" doesn't exist yet in the code before this task (it arrives with `RunProgress.world_index` from task 035).
- **Desired behavior**: `world_params(world_index, config)` is the single source of truth for "how hard is world N" — every difficulty axis (tags, budget, environment, matrix density, objective severity) goes through here, instead of being recomputed ad hoc at different points of worldgen.

---

## 🔨 Suggested Implementation

1. Read `EnvironmentConfig` in `config.rs` to understand the exact names of the existing environmental fields (toxic zone, thermal gradient) before adding the `*_late` endpoints.
2. Define `WorldParams` in `src/worldgen.rs` with the fields listed above.
3. Write `world_params` as a clamped linear interpolation: for each axis, `value(world_index) = early + (late - early) * min(world_index, ramp_worlds) / ramp_worlds`, rounded/truncated to the field's type (`u32` for counts/budgets, `f32` for spreads).
4. Verify the literal constraint: with `active_tags_early=5`, `active_tags_late=8`, which `difficulty_ramp_worlds` produces `active_tag_count(1) == 6`? With a linear ramp over 3 worlds (`ramp_worlds=3`), `world_index=1` → `5 + 3*1/3 = 6`. Set `difficulty_ramp_worlds`'s default to a value that satisfies this constraint for the tag axis, and reuse it for the other axes (a single ramp constant, not one per axis, unless a need emerges during implementation).
5. Add the unit tests listed in the acceptance criteria.

---

## ⚠️ Constraints and Caveats

- **Pure function**: `world_params` must not read `SimWorld`, RNG, or mutable state — only `world_index` and `config`. This makes it testable without a Bevy bootstrap.
- **No magic numbers**: every endpoint/ramp constant lives in `SimConfig`, not as a literal in `worldgen.rs`.
- **Don't generate anything procedural here yet**: this task only produces *parameters*, it doesn't select concrete tags/environment/species/objectives — that's the job of tasks 038, 039, 042, which consume `WorldParams`.
- **Consistency with the endless-until-failure model**: the curve must remain sensible even for arbitrarily high `world_index` (saturation, not unbounded extrapolation).

---

## 🔗 Dependencies

- **Depends on**: none (parallel to task 036).
- **Blocks**: 038 (worldgen consumes `WorldParams` for tags/environment), 042 (consumes `objective_severity`).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/037-world-params-difficulty-curve.md)"$'\n\nExecute this task in the current project.'
```
