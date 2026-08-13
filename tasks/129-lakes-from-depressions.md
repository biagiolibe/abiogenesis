# Task 129 — Lakes derived from terrain depressions

> **ID**: `129`
> **Category**: Refactor (worldgen)
> **Priority**: 🟢 P3
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-13 (Phase 7 of the worldgen pipeline reassessment,
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §10.4)

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

- [ ] During task 127's depression-fill pass, depressions above a
      configurable size/depth threshold are recorded (not just filled and
      discarded) — position, footprint, fill depth.
- [ ] `place_feature_biomes` (or a new step run alongside it) promotes
      qualifying recorded depressions to `Biome::Lake`, writing the same
      target scalars task 111 already defined for Lago, over the
      depression's actual footprint (not a synthetic organic-disk mask —
      the depression's shape *is* the natural footprint here, no need for
      123's angular-distortion technique on top of it).
- [ ] Task 123's organic-masked search (`place_feature_organic` applied to
      Lago) becomes a **fallback**: run only if depression-derived lakes
      don't reach a configurable minimum count/coverage for this world
      (mirrors the keep-best-seen/validation-retry pattern already used
      throughout worldgen, e.g. `generate_terrain`'s placeable-fraction
      floor). Document this explicitly — the fallback exists so a
      low-relief world (few real depressions) doesn't end up with zero
      lakes.
- [ ] Crater and Distesa di cristalli keep using task 123's organic-mask
      search unchanged — this task only changes Lago's placement source.
- [ ] Test: on a sample of seeds, depression-derived lakes correlate with
      locally low `flow_accumulation`-adjacent terrain (a lake cell should
      not be, e.g., on a local elevation maximum) — a sanity check that the
      derivation is actually terrain-grounded, not coincidentally similar
      to the old random search.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
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
