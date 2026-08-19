# Task 128 — Macro-regions before per-cell biome classification

> **ID**: `128`
> **Category**: Refactor (worldgen)
> **Priority**: 🟡 P2
> **Estimate**: ~5h
> **Assigned to**: done
> **Session**: 2026-08-13 (Phase 6 of the worldgen pipeline reassessment,
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §3.2/§11.1/
> §11.4 — the single highest-value gap identified after tasks 123-127 were
> scoped: those tasks make per-cell classification more *causal*, but stay
> per-cell. This task adds the macro-region layer the spec insists on:
> "non assegnare ogni bioma con probabilità indipendente per cella".)
> Implemented 2026-08-19.

---

## 🎯 Objective

Even with task 125's continuous scores, two adjacent cells with near-equal
climate inputs can still land in different biomes on a small noise tie —
the result is a locally-plausible but globally unstructured mosaic, not the
large, readable regions the spec's §2.3 minimum widths (15-40 cells) call
for. The fix is a coarse macro-region layer computed *before* per-cell
classification, each with a dominant biome; per-cell scoring (125) then
becomes a **bias toward the region's dominant biome**, with local
variation (temperature/light/slope/noise) still free to produce secondary
biomes and edges — but no longer free to flip the whole region's identity
cell by cell.

---

## 📋 Acceptance Criteria

- [x] A coarse macro-region grid is derived — either a low-resolution
      climate grid (e.g. 32×20, spec §5.1) or a small-`k` Voronoi partition
      (4-8 regions, spec §11.1) over the full 128×80 grid. Pick whichever
      integrates more simply with the existing per-cell wave-noise
      generation style already in `world.rs` (a Voronoi partition seeded
      from `k` random points is likely the smaller change — no new
      resolution/interpolation machinery needed — but a low-res climate
      grid is closer to the spec's own recommendation; document the choice
      and why).
- [x] Each macro-region gets a **dominant biome**, decided from the
      region's aggregate climate (mean temperature/light/rainfall if task
      126 has landed, elevation) using the same `biome_score` functions
      task 125 introduced, evaluated once per region instead of per cell.
- [x] Task 125's per-cell `biome_score` gains a bias term: a cell's score
      for its macro-region's dominant biome is boosted (a configurable
      weight), so the region's identity wins ties and small noise
      fluctuations, while a cell whose *local* conditions strongly favor a
      different biome (e.g. a real swamp pocket in a Forest-dominant
      region) can still override it — bias, not override.
- [x] `MacroRegion`-equivalent data (id, dominant biome, aggregate stats)
      computed once at generation time on its own seed offset stream (new
      `MACRO_REGION_SEED_OFFSET`, following the existing per-stage stream
      convention), stored only as long as needed to compute the bias — no
      requirement to keep it in `SimWorld` afterward unless another system
      needs it (none does yet).
- [x] Config: number of regions/region-grid resolution, bias weight, in
      `SimConfig`, mirrored in `assets/config/sim_config.ron`.
- [x] Test: measure biome-transition count between adjacent cells
      (spec §18.6's continuity metric) before/after this task on the same
      seeds used in 125's histogram test — the number of transitions should
      drop measurably, confirming regions read as more contiguous, not
      just differently distributed.
- [x] Visual check across several seeds: biomes should read as a handful of
      large, readable regions with local texture inside them, not a
      finer-grained mosaic than before 125. **Confirmed 2026-08-19** via a
      live `cargo run` session, 3 seeds: each shows one dominant Forest
      region as a single large contiguous mass (dithered tree texture, no
      speckling), a second large gray/rocky region forming its own
      coherent blob, smooth-edged lakes, and small isolated feature-biome
      patches (Crater/CrystalField, task 111/123) sitting as local
      exceptions inside the dominant region — the "local texture inside
      large regions" target, not classification noise.
- [x] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes.

---

## ✅ Implementation notes (2026-08-19)

- **Chose Voronoi**, as the task file's own reasoning anticipated: `k`
  random seed points (`BiomeConfig::macro_region_count`, default 6) on a
  dedicated `MACRO_REGION_SEED_OFFSET` stream, nearest-point assignment
  per cell (`compute_macro_regions`, `src/world.rs`) — no new resolution/
  interpolation machinery, reuses the grid's existing normalized `[0,1]`
  coordinate space `classify_biomes` already computes per cell.
- **Bias is multiplicative, not additive** — a design decision the task
  file left open ("boosted (a configurable weight)") and worth flagging
  explicitly: task 125's own doc comment (found 2026-08-19, same session)
  records that `smoothstep`/`smooth_band` *plateau* at exactly `1.0`, so a
  cold flat water-adjacent cell routinely saturates both Tundra's and
  Swamp's score to `1.0` simultaneously. An additive bias does nothing to
  two scores already at the ceiling — exactly the terrain where
  region-level structure is most needed. `score * (1.0 +
  macro_region_bias_weight)` still separates two plateaued `1.0`s from
  each other. A 5-seed scratch measurement (`cargo run --example`,
  discarded) confirmed this isn't a no-op: 1.7-3.9% of `Plain` cells
  changed biome under the default `0.5` weight vs. the bias switched off.
- **Region-level scoring reuses task 125's exact score functions**
  (`swamp_score`/`desert_score`/`tundra_score`/`forest_score`), evaluated
  once per region against the mean temperature/light/slope/water_distance
  over the region's own `TerrainKind::Plain` cells (`noise = 0.0` — patch
  noise is per-cell texture, meaningless as a region average). A region
  with zero `Plain` cells defaults to `Biome::Plain`; harmless, since no
  cell ever queries an all-non-`Plain` region under a Voronoi partition
  (a `Plain` cell's own region is, by construction, a region containing at
  least that one `Plain` cell).
- **`(region_id, dominant_biome)` kept local** to the `classify_biomes`
  call, not stored on `SimWorld` — matches the task's own "no requirement
  to keep it unless another system needs it" criterion; nothing does yet.
- **Config**: `BiomeConfig::macro_region_count: u32 = 6`,
  `macro_region_bias_weight: f32 = 0.5`, mirrored in `sim_config.ron`.
- **Test**: `macro_region_bias_reduces_plain_biome_transitions_on_average`
  (spec §18.6's continuity metric) — compares the shipped default against
  the same 20 seeds with `macro_region_bias_weight` forced to `0.0` (a
  no-op multiplier), counting adjacent `Plain`-kind cell pairs with
  differing biomes. Passes; aggregate, not per-seed, since the bias nudges
  probabilities rather than guaranteeing a drop on every draw.
- **Found and fixed in the same pass, unrelated to this task's own scope**:
  added `tests/config_ron_sync.rs` (deserializes `sim_config.ron`,
  compares its `Debug` output against `SimConfig::default()`) per an
  advisor review — this immediately caught two more pre-existing drifts
  from task 106 (2026-08-12, six days before this session):
  `TagConfig::conditional_tag_count` (RON `4`, stale default `1`) and
  `NotebookConfig::confirmation_threshold` (RON `1.0`, stale default
  `3.0`). Both hand-written defaults corrected to match the RON (the
  deliberately tuned values); one notebook test
  (`an_unconfirmed_observation_logs_nothing_but_still_accumulates_evidence`)
  needed a real confounder in its fixture to stay genuinely unconfirmed
  under the corrected, lower threshold.
- **Live verification**: confirmed 2026-08-19, see the visual-check box
  above.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:743-818` | `classify_biomes` — where the macro-region bias plugs into task 125's score function. |
| `src/world.rs:1508-1530` | `terrain_waves`/`wave_band_sum` — reusable pattern if the climate-grid route is chosen; a Voronoi route would instead need `k` random seed points and a nearest-point assignment, a new small helper. |
| `src/config.rs` (`BiomeConfig`) | New region-count/bias-weight fields. |

---

## 🧩 Technical Context

- **Current behavior (after task 125)**: every cell picks its biome from
  its own local score, independent of its neighbours beyond the small
  patch-noise term.
- **Desired behavior**: cells within the same macro-region share a
  dominant-biome bias, producing large coherent regions with plausible
  local exceptions (a river-adjacent Swamp inside a Forest region, a rocky
  outcrop inside a Plain region) rather than noise-driven speckle.

---

## ⚠️ Constraints and Caveats

- **Don't let the bias become an override.** The spec is explicit that
  local causal facts (a real wetland pocket, a real desert microclimate)
  should still be able to produce a secondary biome inside a
  differently-dominant region (§11.4's worked example: "Swamp nei bacini"
  inside a Forest-dominant region). A bias weight strong enough to make
  local conditions irrelevant defeats the purpose of task 125's causal
  scoring.
- **No magic numbers**: region count/bias weight in config.
- **Determinism**: dedicated seed offset stream, no shared RNG.

---

## 🔗 Dependencies

- **Depends on**: 125 (needs the score-based classification this biases).
- **Blocks**: none directly, but is the natural place to later hook a
  world-profile system (deferred to `VISION.md`, not scoped as a task) if
  that's picked up — profiles would set macro-region *targets*, not
  per-cell rules.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/128-macro-region-biomes.md)"$'\n\nExecute this task in the current project.'
```
