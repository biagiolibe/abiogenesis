# Task 127 — Flow accumulation and rivers

> **ID**: `127`
> **Category**: Feature (worldgen)
> **Priority**: 🟢 P3
> **Estimate**: ~5h
> **Assigned to**: unassigned
> **Session**: 2026-08-13 (Phase 5b of the worldgen pipeline reassessment
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §10 — see
> task 123 for Phase 1 and the session's overall diagnosis)

---

## 🎯 Objective

The current world has no rivers, and no drainage relationship between
elevation and water at all beyond "Sea is low, Mountain is high." Compute
flow accumulation from elevation + `rainfall` (task 126) and mark
high-accumulation cells as rivers, giving the map the continuity the spec's
§1.6 diagnosis calls out as missing: `oceano -> costa umida -> foresta ->
montagne -> steppa/deserto` requires water to actually flow somewhere.

This is the highest-complexity, highest-risk phase in this reassessment —
larger in scope than tasks 123-126 combined. **Do not start implementation
without first re-reading this task's Non-Goals and confirming the scope
below still makes sense** — flow accumulation touches determinism
(sort-order tie-breaking) and generation performance in a way the earlier,
purely-local phases didn't.

---

## 📋 Acceptance Criteria

- [ ] `flow_direction: Option<Direction>` and `flow_accumulation: f32`
      (or equivalent) computed once at generation time: sort all cells by
      descending elevation, route each cell's accumulated flow (starting
      from its own `rainfall`) to its lowest downhill Moore neighbour,
      accumulating into that neighbour.
- [ ] **Deterministic tie-break required**: cells with exactly equal
      elevation (plausible on a normalized `f32` field, especially at
      `Sea` level where many cells may share the same clamped value) must
      sort in a fixed, reproducible order — e.g. break ties by `(y, x)`
      grid index — or the same seed can silently produce different rivers
      across runs/platforms. Add a dedicated test with a hand-constructed
      plateau (several cells at identical elevation) confirming the same
      routing every run.
- [ ] Depressions (local minima that aren't `Sea`) are either filled or
      explicitly breached before routing — an unhandled depression traps
      flow and produces a degenerate accumulation field. Pick one approach
      and document why; a simple priority-flood fill is sufficient, a full
      breaching algorithm is not required for a first version.
- [ ] `is_river: bool` (or a `Biome`/feature marker — decide and document
      which; seeE Non-Goals) set where `flow_accumulation` exceeds a
      configurable threshold. Constrain to the spec's own credibility
      bounds (§10.3): on a 128×80 grid, expect roughly 1-3 principal rivers
      of 20-40 cells of path length, not a dense dendritic network covering
      the map — tune the threshold to land in that range across a sample
      of seeds, and add a test asserting river count/length stay within a
      configured bound (a validation-and-retry check, spec §11.6 style, not
      a hard assert that could make some seeds unsatisfiable — keep-best-seen
      if no attempt lands in range, same pattern as `generate_terrain`).
- [ ] New `HydrologyConfig`-style knobs (river threshold, depression-fill
      limit) added to `SimConfig`, mirrored in `assets/config/sim_config.ron`.
- [ ] Own dedicated seed offset for any new randomness this step needs
      (if the depression-fill or threshold search needs bounded retries,
      it needs its own RNG stream, never reusing another stage's).
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes.

---

## 🚫 Non-Goals

- **Rendering rivers.** Once `is_river`/`flow_accumulation` exist as real
  per-cell data, drawing them is a separate, much smaller follow-up task
  (mirrors how task 112 was split out from task 110/111's data layer).
- **Deciding whether a river is a new `Biome` variant, a `Feature`
  overlay, or a standalone flag.** The spec's own model (§13.1) treats
  features as overlays independent from the climate biome underneath
  ("Mountain + VolcanicAsh + Toxic" without losing the base biome) — a
  river crossing a Forest should probably stay Forest-with-a-river, not
  become a distinct biome. This is a real design decision to make at
  implementation time, informed by how `Feature`/overlay-vs-biome is
  eventually resolved for the existing `VolcanicVent`-style features (task
  111 made those biome overrides, not overlays — a river may want
  different treatment since it's a thin line through many other biomes,
  not a patch). Flag this decision explicitly in the implementation PR,
  don't default to copying the override pattern without considering it.
- **Lakes-from-depressions.** Lakes today are explicitly placed (task 111,
  reworked organically by task 123) rather than derived from terrain.
  Replacing that with depression-derived lakes (spec §10.4) is a separable,
  larger change (it would need to reconcile with the existing placement
  search) — not folded into this task. Depressions this task fills for
  routing purposes stay internal to the flow-accumulation computation, not
  promoted to new `Lake` biome cells.
- **Feeding `flow_accumulation`/`is_river` back into biome classification**
  (e.g. a `RiverForest` variant, spec §11.4's worked example) — a further
  refinement once rivers exist and their placement has been visually
  validated across seeds, not assumed to be correct on the first pass.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:329-357` | `Cell` struct — add the new fields here. |
| `src/world.rs:568-653` | `generate_terrain` — the elevation field this consumes; runs after this in the pipeline. |
| Task 126's `rainfall` field | Direct input — this task should follow 126 in the same session/branch if picked up together, since flow accumulation without a real rainfall field would have to fall back to a uniform stand-in, weakening the result. |

---

## 🧩 Technical Context

- **Current behavior**: no drainage model exists; `Sea`/`Lake` cells are
  independent of any upstream terrain.
- **Desired behavior**: a deterministic flow-accumulation field derived
  from elevation + rainfall, with a small number of principal rivers
  marked from it — enough to make "water flows from mountains to the sea"
  a visible, inspectable fact about the generated world.

---

## ⚠️ Constraints and Caveats

- **Determinism is the main risk in this task**, not correctness of the
  hydrology model itself — see the tie-break acceptance criterion above.
  Treat it as the primary thing to test, not an afterthought.
- **Performance**: flow accumulation via elevation-sort is `O(n log n)`
  over ~10,240 cells — cheap in isolation, but this task adds a full extra
  sort-and-route pass to generation; if combined with tasks 124-126's
  passes, re-check total generation time stays acceptable (no hard budget
  documented today — establish a rough baseline before/after if generation
  time becomes user-visible).
- **No magic numbers**: threshold/config values in `SimConfig`.

---

## 🔗 Dependencies

- **Depends on**: 124 (`slope`/elevation-adjacent fields), 126 (`rainfall`
  as the flow input).
- **Blocks**: none directly (see Non-Goals for the natural follow-ups this
  unblocks but doesn't require).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/127-hydrology-rivers.md)"$'\n\nExecute this task in the current project.'
```
