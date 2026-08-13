# Task 128 — Macro-regions before per-cell biome classification

> **ID**: `128`
> **Category**: Refactor (worldgen)
> **Priority**: 🟡 P2
> **Estimate**: ~5h
> **Assigned to**: unassigned
> **Session**: 2026-08-13 (Phase 6 of the worldgen pipeline reassessment,
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §3.2/§11.1/
> §11.4 — the single highest-value gap identified after tasks 123-127 were
> scoped: those tasks make per-cell classification more *causal*, but stay
> per-cell. This task adds the macro-region layer the spec insists on:
> "non assegnare ogni bioma con probabilità indipendente per cella".)

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

- [ ] A coarse macro-region grid is derived — either a low-resolution
      climate grid (e.g. 32×20, spec §5.1) or a small-`k` Voronoi partition
      (4-8 regions, spec §11.1) over the full 128×80 grid. Pick whichever
      integrates more simply with the existing per-cell wave-noise
      generation style already in `world.rs` (a Voronoi partition seeded
      from `k` random points is likely the smaller change — no new
      resolution/interpolation machinery needed — but a low-res climate
      grid is closer to the spec's own recommendation; document the choice
      and why).
- [ ] Each macro-region gets a **dominant biome**, decided from the
      region's aggregate climate (mean temperature/light/rainfall if task
      126 has landed, elevation) using the same `biome_score` functions
      task 125 introduced, evaluated once per region instead of per cell.
- [ ] Task 125's per-cell `biome_score` gains a bias term: a cell's score
      for its macro-region's dominant biome is boosted (a configurable
      weight), so the region's identity wins ties and small noise
      fluctuations, while a cell whose *local* conditions strongly favor a
      different biome (e.g. a real swamp pocket in a Forest-dominant
      region) can still override it — bias, not override.
- [ ] `MacroRegion`-equivalent data (id, dominant biome, aggregate stats)
      computed once at generation time on its own seed offset stream (new
      `MACRO_REGION_SEED_OFFSET`, following the existing per-stage stream
      convention), stored only as long as needed to compute the bias — no
      requirement to keep it in `SimWorld` afterward unless another system
      needs it (none does yet).
- [ ] Config: number of regions/region-grid resolution, bias weight, in
      `SimConfig`, mirrored in `assets/config/sim_config.ron`.
- [ ] Test: measure biome-transition count between adjacent cells
      (spec §18.6's continuity metric) before/after this task on the same
      seeds used in 125's histogram test — the number of transitions should
      drop measurably, confirming regions read as more contiguous, not
      just differently distributed.
- [ ] Visual check across several seeds: biomes should read as a handful of
      large, readable regions with local texture inside them, not a
      finer-grained mosaic than before 125.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes.

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
