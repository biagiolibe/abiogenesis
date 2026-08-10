# Task 085 — Source-driven temperature and light

> **ID**: `085`
> **Category**: Architecture (worldgen)
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-10, scoped from `redesign/abiogenesis-environment-sources.md`

---

## 🎯 Objective

Replace `SimWorld::apply_environment_gradients`'s fixed left→right temperature lerp and top→bottom light lerp with a source-driven model:

- **Temperature**: N point heat sources placed via a dedicated seed-offset RNG stream at world creation, plus a per-world wind direction that biases distance-to-source anisotropically, plus a per-tick reinjection step that pulls source cells back toward their fixed source temperature so `diffuse_environment`'s blur doesn't erode the field to a flat mean over a long run.
- **Light**: a per-world sun direction replacing the top→bottom lerp; light per cell derives from a directional falloff (projection of the cell's position onto the sun direction). A linear directional field is already a fixed point of the Moore-blur (see Technical Context), so **light needs no reinjection**.
- `TerrainKind::Sea` cells act as a passive heat sink during diffusion's blend; `TerrainKind::Mountain`/peak cells dim nearby light.

This is worldgen/generation only: `env_fit` and the `Photolithic` energy-gain formula in `sim.rs` are unaffected — they just read `Cell.temperature`/`Cell.light`, agnostic to how those fields are populated. Full design rationale, open questions, and blast-radius analysis: `redesign/abiogenesis-environment-sources.md`.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors; `cargo clippy -- -D warnings` and `cargo fmt` clean.
- [ ] New seed-offset constant(s) added (e.g. `TEMPERATURE_SOURCE_SEED_OFFSET`, and a sun-direction offset — decide during implementation whether sun shares temperature's RNG draw or gets its own), same style as `TERRAIN_SEED_OFFSET`/`TOXIC_ZONE_SEED_OFFSET` (`src/world.rs:526-533`).
- [ ] Source placement is bounded-retry generation against `SimWorld::is_placeable` (no sources on Sea/peaks), mirroring `place_toxic_zone`'s attempt-loop/keep-best-seen pattern (`src/world.rs:345-403`) — reuse the pattern, don't invent a new one.
- [ ] Reinjection is a **distinct `SimWorld` method from `diffuse_environment`**, called from `sim::step` alongside it, not folded into the blend loop — this is what keeps `diffuse_environment`'s existing protected tests passing unmodified.
- [ ] Light falloff is linear in cell position (a directional projection); a new test proves a sun-direction-only field (with peaks present) is unchanged by one `diffuse_environment` tick, empirically confirming "no reinjection needed for light."
- [ ] `WorldParams` gains replacement fields for whatever ramps (source count, radius, wind/sun strength — implementer's call on exact shape); `world_params` stays a pure function (no RNG, no `SimWorld` — TECH_DESIGN.md invariant 2).
- [ ] `EnvironmentConfig::temperature_gradient_left/right`/`light_gradient_high/low` and `DifficultyConfig::temperature_spread_late` are removed or repurposed, not left dead. `assets/config/sim_config.ron` hand-updated to match every config.rs change.
- [ ] Decide and document how `EnvironmentConfig::stress_delta`'s temperature-nudge interacts with reinjection pinning source cells back to their fixed value every tick (it currently targets temperature because that's the only tick-loop-read scalar; reinjection risks silently no-op'ing Stress on exactly the source cells).
- [ ] New invariant test (style of task 060's ambient-trickle-vs-decay check) that reinjection strength stays compatible with `diffusion_rate` (0.05), so the field stays structured rather than either static or homogenized.
- [ ] `gradients_match_gdd_extremes` (`src/world.rs:848`, hardcodes fixed-gradient corner values) rewritten for the new model.
- [ ] `dark_rows_stay_uninhabited_across_seeds` (`tests/balance.rs:211-232`, assumes light varies only by row) rewritten for the new model.
- [ ] `tests/run_reproducibility.rs`'s `a_different_run_seed_diverges_the_world_sequence` (~line 76) updated: it currently asserts `cells` are identical across different run seeds at the same `world_index` because today's gradient is seed-independent; source/wind/sun positions drawing from `self.seed` breaks that premise, so the test and its explanatory comment must start asserting `cells` diverge too.
- [ ] These tests keep passing **unmodified**: `environment_scalars_stay_in_unit_range`, `uniform_field_is_a_fixed_point_of_diffusion`, `diffusion_smooths_a_single_perturbed_cell`, `diffusion_keeps_scalars_in_unit_range_over_many_ticks`, `diffusion_does_not_touch_the_rng_and_stays_deterministic` (`src/world.rs:997-1080`).
- [ ] `env_fit` (`sim.rs:396-400`), the `Photolithic` gain line (`sim.rs:~250`), and toxic zone generation/placement (`world.rs:345-403`) are **not modified**.
- [ ] `cargo test` passes in full.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `apply_environment_gradients` (replace), new reinjection method, new seed offset(s), `is_placeable`/`place_toxic_zone` as patterns to reuse. |
| `src/sim.rs` | `sim::step` call site for the new reinjection method (add call next to `diffuse_environment`); `env_fit`/`Photolithic` gain — read-only reference, not modified. |
| `src/worldgen.rs` | `WorldParams`, `world_params` — add new difficulty-ramped fields, remove `temperature_spread`. |
| `src/config.rs` | `EnvironmentConfig`, `DifficultyConfig`, likely a new source/sun config struct mirroring `TerrainConfig`'s shape (wave/threshold/attempt-count params). |
| `assets/config/sim_config.ron` | Must be hand-mirrored to every config.rs struct change. |
| `tests/run_reproducibility.rs` | Update seed-divergence assertion to include `cells`. |
| `tests/balance.rs` | Rewrite `dark_rows_stay_uninhabited_across_seeds`. |

---

## 🧩 Technical Context

**Current behavior**: `apply_environment_gradients` (`src/world.rs:413-429`, private, pure function of `config`/`params`, no RNG) sets `cell.light` via a top→bottom lerp (`light_gradient_high`→`light_gradient_low`) and `cell.temperature` via a left→right lerp (`temperature_gradient_left`→`temperature_gradient_left + params.temperature_spread`). Called once from `new_for_world`, after `generate_terrain`, before `place_toxic_zone`. `diffuse_environment` (`world.rs:431-457`, `pub`) runs every tick from `sim::step`, blending `temperature`/`light`/`toxicity` toward the Moore-neighbor mean at `config.environment.diffusion_rate` (0.05), via `self.cells` → `self.scratch` double buffering. It is explicitly RNG-free (`diffusion_does_not_touch_the_rng_and_stays_deterministic` checks this).

Key derived fact: the blend `new = c + rate * (mean(8 neighbours) - c)` leaves any field **linear in x/y** as an exact fixed point (the 8 symmetric neighbour offsets cancel). That's why the current lerps have never needed reinjection. A **sun-direction light field (a directional projection) stays linear**, so it inherits this property — no reinjection needed. A **radial point-source temperature field is not linear** — without reinjection it homogenizes toward the field mean within a few hundred ticks, so reinjection is a correctness requirement of this task, not deferred tuning.

RNG-stream pattern to follow: `TERRAIN_SEED_OFFSET`/`TOXIC_ZONE_SEED_OFFSET` (`world.rs:526-533`) are arbitrary `u64` XOR salts; each generation step seeds its own local `StdRng::seed_from_u64(self.seed ^ OFFSET)`, keeping generation phases order-independent regardless of which steps are added/removed. Follow this exactly for source/wind/sun placement.

`place_toxic_zone` (`world.rs:345-403`) is the style reference for bounded-retry placement generation: derive counts from `params: &WorldParams` (difficulty-ramped), seed local RNG, loop up to `max_..._attempts` scoring candidates, accept the first above a floor else keep the best seen, never panic on an unlucky draw.

`TerrainKind` (`world.rs:118-125`): `Sea`, `Plain` (default), `Hill`, `Mountain`. Peaks are `Mountain` cells with a separate `Cell.is_peak: bool`. Centralized occupiability check: `SimWorld::is_placeable(x,y)`/`is_placeable_index(idx)` (`world.rs:~479/487`) — no other call site should check `terrain`/`is_peak` directly.

`worldgen::world_params(world_index, config)` (`worldgen.rs:53-83`) is a pure function (no RNG, no `SimWorld`) that lerps early/late config endpoints by `ramp_fraction(world_index, ramp_worlds)`. The module doc comment states the project convention: generation methods consume `WorldParams` for ramped counts/spreads, not `SimConfig` difficulty endpoints directly.

**Desired behavior**: `apply_environment_gradients` (or its replacement) derives temperature from distance-to-nearest-heat-source (wind-biased) plus continuous reinjection, and light from a per-world sun-direction projection with Mountain shading — both still respecting the `[0,1]` scalar range and Sea/peak placement constraints, both still deterministic per world seed.

---

## 🔨 Suggested Implementation

1. Add new seed-offset constant(s) next to `TERRAIN_SEED_OFFSET`/`TOXIC_ZONE_SEED_OFFSET`.
2. Design the new config struct(s) (source count, radius, wind/sun strength, reinjection strength — early/late pairs where difficulty-ramped) and wire early/late endpoints through `world_params`.
3. Implement heat-source placement (bounded-retry against `is_placeable`) and per-cell temperature-from-distance with wind bias, replacing the temperature half of `apply_environment_gradients`.
4. Implement sun-direction light falloff with Mountain/peak dimming, replacing the light half.
5. Implement Sea-as-coolant inside (or alongside) `diffuse_environment`'s temperature blend.
6. Implement the reinjection method; call it from `sim::step` next to `diffuse_environment`.
7. Update `sim_config.ron`, rewrite/add the tests listed in Acceptance Criteria, run `cargo test`/`clippy`/`fmt`.

```
// No prescribed snippet — falloff function and exact constants are
// explicitly deferred to playtest per the design doc; pick a first-pass
// shape (e.g. smoothstep or inverse-square over a configured radius) and
// tune visually via task 086.
```

---

## ⚠️ Constraints and Caveats

- **Style**: Follow `TECH_DESIGN.md` conventions — deterministic, headless-testable sim/world/config, no `rand::rng()`, no `HashMap` iteration, `SimConfig` coefficients only (no magic numbers).
- **Explicitly deferred to playtest, not decided here**: exact falloff function shape for both temperature and light, exact source-count/radius/wind-strength difficulty curve values, exact reinjection strength constant, GDD §5.2 rewrite (only after implementation + playtest, so it documents what's real).
- **Scope guard**: do not modify `env_fit`, the `Photolithic` gain formula, or toxic zone generation/placement — this is a worldgen change, not a tick-formula change.
- Do not pre-plan a Sea/Mountain coupling task — fold both into this task's acceptance criteria; only open a follow-up if playtest (task 086) reveals the two effects visibly conflict.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: 086

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/085-source-driven-temperature-and-light.md)"$'\n\nExecute this task in the current project.'
```
