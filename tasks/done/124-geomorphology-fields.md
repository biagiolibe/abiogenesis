# Task 124 — Derived geomorphology fields (slope, water distance)

> **ID**: `124`
> **Category**: Feature (worldgen)
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: done
> **Session**: 2026-08-13 (Phase 2 of the worldgen pipeline reassessment
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §8.3/§1.1 —
> see task 123 for Phase 1 and the session's overall diagnosis), implemented
> 2026-08-19.

---

## ✅ Implementation notes (2026-08-19)

- `Cell` gains `slope: f32` and `water_distance: f32` (both default `0.0`
  via `Cell`'s existing `#[derive(Default)]`).
- `slope`: `SimWorld::elevation_slope` computes a 4-neighbour central
  difference over `Cell.elevation` (one-sided at grid edges, no
  wraparound), then `compute_geomorphology` divides by the new
  `TerrainConfig::slope_normalization` (`0.05`, calibrated numerically
  against a few sample seeds — raw gradient magnitude runs roughly
  `0.0-0.05`, so this spreads the normalized value across most of `[0,1]`
  without excessive clamping) and clamps to `[0, 1]`.
- `water_distance`: generalized the existing `sea_distance_field`'s
  multi-source-BFS pattern into a shared `bfs_distance_from(is_source)`
  helper, then added `water_distance_field` on top of it, sourced from
  `Biome::DeepWater`/`ShallowWater`/`Lake` (not just `TerrainKind::Sea`).
  `sea_distance_field` itself is unchanged in behavior — still Sea-only,
  still feeds `apply_environment_sources`' coastal-cooling model — per the
  task's explicit caution against an unreviewed change to that balance.
- Both computed in the new `compute_geomorphology` step, called from
  `new_for_world` right after `place_feature_biomes` (needs `Biome::Lake`
  cells to exist as a BFS source) — nothing else in the pipeline reads
  either field yet.
- `slope_normalization` added to `TerrainConfig`, mirrored in
  `assets/config/sim_config.ron`.
- New tests: `slope_and_water_distance_do_not_affect_biome_classification`
  (calls `classify_biomes` twice on the same cell state, once with the two
  new fields corrupted to `999.0`, and asserts the areal output is
  unchanged — this is the actual scope-creep guard, stronger than a
  before/after snapshot since a hidden read of these fields would show up
  even though both runs used the same underlying data),
  `slope_stays_within_the_normalized_unit_range`,
  `water_distance_is_zero_on_every_water_cell_and_finite_elsewhere`,
  `water_distance_is_deterministic_for_a_given_seed`.
- `cargo build`/`clippy -D warnings`/`fmt`/`test` all clean.

---

## 🎯 Objective

`classify_biomes` currently reasons only from `TerrainKind` (a coarse
elevation band) plus the ambient scalars. The spec's §1.1 diagnosis: going
straight from elevation to a discrete band throws away slope and distance
to water, which are what later phases (125's score-based classification,
127's hydrology) need to place BareRock/Palude/rivers causally instead of
by an arbitrary threshold on an unrelated scalar.

This task is purely additive: compute and store two new per-cell fields.
It changes no classification behavior yet — `classify_biomes`'s output
must be unchanged after this task (verified by a snapshot-style test, see
below). Task 125 is what actually uses these fields to reclassify.

---

## 📋 Acceptance Criteria

- [x] `Cell` gains `slope: f32` — local elevation gradient magnitude,
      computed from `Cell.elevation` (already stored per task 110) via a
      finite-difference gradient over the Moore neighbourhood (or a
      cheaper 4-neighbour central difference — either is fine, document
      the choice). Normalize to a sane range so `BiomeConfig` thresholds
      added in task 125 can be tuned as plain `[0, ~1]` numbers, not raw
      per-cell elevation differences.
- [x] `Cell` gains `water_distance: f32` — generalizes the existing
      `sea_distance_field` (`world.rs:1079`, currently Sea-only) to a
      multi-source BFS from every cell whose `Biome` is `DeepWater`,
      `ShallowWater`, or `Lake` (not just `TerrainKind::Sea`), so a cell
      near a lake reads as "near water" the same way a coastal cell does.
      Decide and document whether this replaces `sea_distance_field`
      outright (`apply_environment_sources`'s coastal-cooling use, and the
      private `SimWorld::sea_distance` field, both currently sea-only —
      confirm whether folding lakes into coastal cooling too is desired or
      would change existing balance; if unsure, keep `sea_distance_field`
      as-is for temperature and add `water_distance` as a separate field
      used only by biome classification, rather than risk an unreviewed
      change to `apply_environment_sources`'s temperature model).
- [x] Both fields computed in `SimWorld::new_for_world`'s generation
      pipeline **after** `place_feature_biomes` (task 111) — `water_distance`
      needs `Lake` cells to already exist as a source, and `slope` needs
      final `elevation`, which is set by `generate_terrain` and never
      touched afterward, so ordering relative to `slope` alone is more
      flexible, but keep both in the same new pipeline step for clarity.
- [x] New fields are **read nowhere yet** in this task — `classify_biomes`,
      rendering, and every existing system stay untouched. A test asserting
      that biome/terrain output is byte-identical before and after this
      change (e.g. reusing an existing seed-determinism test's world and
      comparing `Vec<Biome>` snapshots) confirms this task is additive only.
- [x] New config knobs (if any — e.g. a normalization constant for `slope`)
      go in `SimConfig`/`TerrainConfig`, mirrored in
      `assets/config/sim_config.ron`. No inline magic numbers.
- [x] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:329-357` | `Cell` struct — add `slope`, `water_distance` here, alongside `elevation`/`is_peak`'s existing precedent for generation-time-only derived fields. |
| `src/world.rs:1079` | `sea_distance_field` — the BFS pattern to generalize (or parallel) for `water_distance`. |
| `src/world.rs:568-653` | `generate_terrain`/`write_terrain` — where `elevation` is finalized; `slope` must be computed from the same, final data. |
| `src/world.rs:837` (`place_feature_biomes`) | Must run before the new `water_distance` computation (needs `Lake` cells to exist). |

---

## ⚠️ Constraints and Caveats

- **Determinism**: both fields are pure functions of already-deterministic
  data (`elevation`, `Biome`) — no new RNG stream needed.
- **No magic numbers**: any normalization/scaling constant lives in config.
- Don't let this task's scope creep into reclassification — that's 125.
  The acceptance criteria's "byte-identical before/after" test is the guard
  against scope creep here.

---

## 🔗 Dependencies

- **Depends on**: 110 (`Cell.elevation`), 111 (`Biome::Lake` must exist as
  a `water_distance` source).
- **Blocks**: 125 (score-based classification needs `slope`/
  `water_distance` as inputs).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/124-geomorphology-fields.md)"$'\n\nExecute this task in the current project.'
```
