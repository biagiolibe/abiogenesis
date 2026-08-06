# Task 047 — Fix `SurviveIn`'s toxic-zone membership check

> **ID**: `047`
> **Category**: Bugfix
> **Priority**: 🔴 P1
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-06 playtest session

---

## 🎯 Objective

`Objective::SurviveIn { zone: ZoneKind::Toxic, .. }` is satisfiable by an organism that was never anywhere near the toxic zone. `objectives.rs::cell_in_zone` checks `cell.toxicity > 0.0`, but `world.rs::diffuse_environment` blends every environmental scalar (including `toxicity`) toward the mean of its Moore neighbours every tick, with no floor pinning non-source cells back to `0.0`. Given enough ticks, trace toxicity diffuses out of the original corner and eventually contaminates the whole grid, so "is this cell in the toxic zone" — checked against the *live, diffused* scalar — stops meaning what it's supposed to mean.

The zone must be checked against its original geometry (a fixed region of the grid), not against a scalar that drifts over time.

---

## 📋 Acceptance Criteria

- [ ] `species_present_in_zone`/`cell_in_zone` (or their replacement) determine zone membership from the zone's original `(x, y)` bounds, not from `cell.toxicity`'s current value.
- [ ] A regression test: an organism that never enters the toxic zone's original bounds must not satisfy `SurviveIn`, even after many ticks of diffusion have raised `toxicity` above `0.0` outside the zone.
- [ ] Existing behavior preserved: an organism actually inside the zone's original bounds still satisfies the condition (don't break `objectives.rs`'s existing `survive_in_toxic_zone_requires_sustained_presence`-style tests).
- [ ] `cargo clippy -- -D warnings` clean, `cargo test` green.
- [ ] Manual/automated verification that the specific playtest scenario (organism far from the toxic corner, many ticks elapsed) no longer satisfies the objective.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` | `cell_in_zone` (checks `cell.toxicity > 0.0`), `species_present_in_zone`. |
| `src/world.rs` | `apply_gradients` (computes `zone_x0`/`zone_y0` from `WorldParams` at construction — the geometry that should be the source of truth), `diffuse_environment` (the diffusion that invalidates the current scalar-based check). `SimWorld` does not currently store the zone's bounds anywhere. |
| `src/worldgen.rs` | `WorldParams::toxic_zone_width`/`toxic_zone_height`, already available at world-construction time. |

---

## 🧩 Technical Context

<!-- TODO: add relevant code snippets and file paths -->

**Current behavior**: `apply_gradients` (`world.rs:177`) computes `zone_x0 = width - params.toxic_zone_width`, `zone_y0 = height - params.toxic_zone_height` once, at world construction, to paint the initial `toxicity` scalar. Those bounds aren't stored anywhere on `SimWorld` — only the resulting per-cell `toxicity` values are, and those drift via `diffuse_environment`. `objectives.rs::cell_in_zone` re-derives "is this cell in the zone" from the drifted scalar instead of the original bounds.

**Desired behavior**: zone membership is a fixed, geometric fact about the world (like the grid dimensions), unaffected by how far toxicity has diffused by the time the check runs.

- **Current behavior**: `cell_in_zone` returns `true` for any cell with `toxicity > 0.0`, which becomes true almost everywhere after enough diffusion ticks.
- **Desired behavior**: `cell_in_zone` (or its replacement) returns `true` only for cells within the zone's original `(x, y)` rectangle, regardless of the live `toxicity` scalar.

---

## 🔨 Suggested Implementation

1. Decide where the zone's bounds live: likely a new field on `SimWorld` (e.g. `toxic_zone: (usize, usize, usize, usize)` or a small struct), set once in `apply_gradients`/`new_for_world` from `WorldParams`, alongside `active_tags`/`matrix`.
2. Update `objectives.rs::cell_in_zone` to check the cell's `(x, y)` against those stored bounds instead of `cell.toxicity`.
3. Add the regression test described above (build a world, place an organism outside the original zone, run enough ticks for diffusion to raise `toxicity` outside the zone, assert `SurviveIn` still doesn't clear).
4. Verify existing objective tests still pass.

---

## ⚠️ Constraints and Caveats

- **Determinism**: whatever storage you add to `SimWorld` must not introduce any new RNG use or non-determinism (invariant 1, TECH_DESIGN.md §5).
- **`ZoneKind` is meant to generalize** (currently only `Toxic` exists) — keep the fix general enough that a future zone kind isn't tied to a specific environmental scalar either.

---

## 🔗 Dependencies

- **Depends on**: none (bugfix against already-shipped Phase 3 code).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/047-fix-toxic-zone-membership-check.md)"$'\n\nExecute this task in the current project.'
```
