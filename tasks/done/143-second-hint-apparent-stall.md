# Task 143 — Second contextual hint on the apparent stall

> **ID**: `143`
> **Category**: Feature
> **Priority**: 🟡 Media (Phase 1b)
> **Estimate**: ~1h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-28

---

## 🎯 Objective

An isolated organism sitting near energy break-even shows no visible change
for many ticks. A new player can't tell "this is the expected behaviour"
from "I made a mistake." Add a second, discreet, non-blocking onboarding
hint, shown once per process session, once a population's per-capita net
energy has sat within a narrow band around zero for enough consecutive
ticks.

Design source: `redesign/processed/culture-shock-friction-fixes.md`,
Intervento 1.

---

## 📋 Acceptance Criteria

- [x] `cargo build`/`clippy -- -D warnings` clean.
- [x] `SimWorld::stall_ticks` (grid-sized `Vec<u32>`, same lifecycle as
      `adjacency_exposure`): incremented in `sim::step` when a population's
      per-capita net energy stays within `STALL_BAND` (`0.1`, indicative
      per the design doc), reset to `0` otherwise or on death.
- [x] `sim::any_population_stalled` — true once any cell reaches
      `STALL_TICKS_THRESHOLD` (`15`) consecutive stalled ticks.
- [x] `ui::StallHint` + `check_stall_hint` latch the hint once per process
      session (`MetaProgress::seen_stall_hint`), era-derived
      `duration_ticks` exactly like `IsolationHint`.
- [x] `viewport_hint` shows it below both task-053 milestone hints and the
      guided first-isolation hint — never overlapping them.
- [x] Reset on every world (re)start via `WorldResetParams` (`stall_hint.active`).
- [x] Unit tests: `any_population_stalled` threshold read, and `step`
      increments `stall_ticks` only for a near-equilibrium population, not
      a healthy-margin one.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `SimWorld::stall_ticks` field. |
| `src/sim.rs` | `STALL_BAND`/`STALL_TICKS_THRESHOLD` consts, increment/reset in `step`, `any_population_stalled`. |
| `src/ui.rs` | `StallHint` resource, `check_stall_hint` system, `viewport_hint`'s priority chain. |
| `src/run.rs` | `MetaProgress::seen_stall_hint`. |
| `src/run_flow.rs` | `WorldResetParams::stall_hint` reset. |
| `src/menu.rs` | `start_run`'s fresh-resource insertion. |
| `src/text.rs` | `HINT_APPARENT_STALL`. |

---

## 🔗 Dependencies

- **Depends on**: 137 (per-cell population model), 055 (`IsolationHint`, the pattern this mirrors).
- **Blocks**: none.
