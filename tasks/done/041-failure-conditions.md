# Task 041 — Failure conditions

> **ID**: `041`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Phase 3 planning session

---

## 🎯 Objective

GDD §8, failure conditions [DECIDED]: total extinction → immediate failure (the obvious floor); a generous but finite per-world era budget (baseline: 40 eras in the initial worlds, moving toward 25 in later worlds) — a stuck player fails instead of grinding forever, and that's what gives roguelike tension.

Today neither check exists: `SpeciesExtinct` (existing event, task 018) is emitted per-species but no system checks whether *all* species are extinct simultaneously; `TimeConfig::era_budget_early/late` is defined in `SimConfig` but never read by `advance_tick`. This task wires both into the `WorldOutcome` type shared with task 040 and into `GameState::Defeat` (task 035).

---

## 📋 Acceptance Criteria

- [x] Type shared with 040, e.g. `enum WorldOutcome { Ongoing, Cleared, Failed(FailureReason) }` (or an equivalent form) — if 040 hasn't already introduced it, define it here in `objectives.rs` or in a shared module.
- [x] **Total extinction** check: if grid occupancy is 0 (no living organism), the outcome becomes `Failed` in the same tick it happens — with an explicit guard against the false positive of the frame before initial seeding (the world starts empty for an instant before the starting species are placed).
- [x] **Era budget exhausted** check: `advance_tick` (or a system immediately after it) compares `world.era` against `WorldParams.era_budget` (task 037/038) — if the budget is exhausted without the objective having been satisfied, the outcome becomes `Failed`.
- [x] When the outcome is `Failed`, the game transitions to `GameState::Defeat` (variants introduced in task 035).
- [x] Unit test: hand-built world that exhausts the era budget without satisfying the objective → `Failed`.
- [x] Unit test: hand-built world that reaches zero occupancy → `Failed` in the same tick, not one tick late.
- [x] No false positive of total extinction in the initialization tick (before `seed_starting_palette`/task 039's generator has placed the species).
- [x] `cargo clippy -- -D warnings` clean, `cargo test` green.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `advance_tick` — point to add the era-budget check; possible hook for total extinction. |
| `src/objectives.rs` | Shared `WorldOutcome` type, if not already present from task 040. |

---

## 🧩 Technical Context

**Current `advance_tick`** (`src/sim.rs`, lines ~427-450, see also task 035 for the extension with `EraCompleted`):
```rust
if progress.remaining() == 0 {
    world.era += 1;
    budget.refill(config.time.point_budget_per_era);
    next_state.set(EraState::Observing);
}
```
This is the natural point for the budget check: right after `world.era += 1`, compare against the current world's `WorldParams.era_budget`.

**`SpeciesExtinct`** (`src/sim.rs`, lines ~20-25): emitted per-species when the last organism of that species dies (detected via a pre/post-tick population diff, line ~286). It does not imply *total* extinction — a separate check on overall grid occupancy is needed.

- **Current behavior**: a world can proceed indefinitely even with zero living organisms or well past any reasonable era budget — there is no "game over" at all.
- **Desired behavior**: the player receives an explicit, timely failure in both cases, consistent with the "roguelike tension" design of GDD §8.

---

## 🔨 Suggested Implementation

1. Verify/define `WorldOutcome` in coordination with what task 040 produces (if executed first, reuse its type; if this task is executed first between the two, define it here and have 040 reuse it — decide based on the actual execution order).
2. Add a system (or extend task 040's) that, after `advance_tick`, checks total grid occupancy (`world.cells` — verify the exact field/counting-method name in `world.rs`).
3. In the same `if progress.remaining() == 0` branch of `advance_tick`, add the comparison `world.era >= era_budget_corrente` (from `WorldParams`, available only after task 038 — if this task is executed before 038/039 are completed on the development branch, use `WorldParams` with a fixed/test `world_index` until the real integration is available, but production code must read the real value).
4. Wire the `Failed` outcome to the `next_state.set(GameState::Defeat)` transition.
5. Write the tests with hand-built worlds (existing pattern in `sim.rs`).

---

## ⚠️ Constraints and Caveats

- **Determinism**: no check here introduces RNG or external state.
- **Evaluation order**: total extinction must be detectable even mid-way through an animated era (`EraState::Advancing`), not only at the era boundary — otherwise the player would see an empty grid for up to 25 ticks before failure.
- **Don't duplicate reset logic**: this task detects failure and transitions state; the actual *reset* (new world, new run) is task 045's responsibility (`start_world`), not this task's.
- **Anti-false-positive guard**: the tick immediately following `SimWorld`'s creation, before the starting species are placed, has zero occupancy by construction — it must not be interpreted as defeat.

---

## 🔗 Dependencies

- **Depends on**: 040 (shared `WorldOutcome`/`Objective` type).
- **Blocks**: 045 (world transition consumes `GameState::Defeat`).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/041-failure-conditions.md)"$'\n\nExecute this task in the current project.'
```
