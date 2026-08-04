# Task 040 — Objectives: type + evaluation engine

> **ID**: `040`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Phase 3 planning session

---

## 🎯 Objective

GDD §8: each world poses one or more explicit requirements. Literal examples from the GDD: "Achieve a biosphere with ≥3 coexisting species for 50 ticks", "Grow a species that survives in the toxic zone", "Trigger a bloom of a specific type". Success leads to the next world (task 045); failure ends the run (task 041).

This task introduces the `Objective` type and the engine that evaluates its satisfaction tick by tick, **independently of worldgen** — it's testable against the current hardcoded world (Phase 0-2), it doesn't need to wait for tasks 038/039 to exist. The procedural generation of *which* objective to assign to each world is task 042, which consumes the type defined here.

---

## 📋 Acceptance Criteria

- [x] New `src/objectives.rs` module with `enum Objective`: `Coexistence { min_species: u32, ticks: u32 }`, `SurviveIn { species: SpeciesId, zone: ZoneKind, ticks: u32 }` (`ZoneKind::Toxic` for now), `TriggerBloom { species: SpeciesId, population_threshold: u32 }` — a single-tick population threshold, matching the bloom-detection direction already sketched as a TODO in `notebook.rs`'s salient-event log.
- [x] `ObjectiveProgress` resource: `consecutive_ticks: u32` (streak counter) + `satisfied: bool` (sticky — once cleared, stays cleared even if the condition later breaks).
- [x] Pure function `pub fn evaluate(objective: &Objective, world: &SimWorld, progress: &mut ObjectiveProgress) -> WorldOutcome`. Driving system lives in `objectives.rs` itself (`ObjectivesPlugin`, `evaluate_current_objective`), scheduled `.after(SimSet::Advance)` in `FixedUpdate` — not literally inside `sim.rs`, to keep the "one module = one Plugin" convention; ordering against `advance_tick` is via the existing `SimSet`, not a hard dependency on `sim.rs`'s internals.
- [x] Consecutive-tick counting: `evaluate_sustained` increments on a true condition, resets to `0` (never decrements) the tick it's false. Runs every simulated tick (same `FixedUpdate` cadence as `advance_tick`), so a 50-tick/2-era objective accumulates correctly across era boundaries by construction — nothing resets the counter when an era ends, only `evaluate` itself does.
- [x] Objectives reference only observable quantities (species count, zone occupancy via `cell.toxicity`, population) — never `world.matrix`/`TagMatrix`.
- [x] `evaluate` is a pure function of `(&Objective, &SimWorld, &mut ObjectiveProgress)`, no Bevy dependency, no RNG — tested entirely with hand-built `SimWorld`s (`world_with_species`/`place` test helpers), independent of worldgen.
- [x] Tests: one (or more) per variant, plus a dedicated reset-on-interruption test and a "stays cleared after the condition later breaks" test.
- [x] `cargo clippy --all-targets -- -D warnings` clean, `cargo test` green (83 tests total).

## Implementation summary

- `src/objectives.rs` (new): `ZoneKind`, `Objective`, `WorldOutcome` (shared type task 041 will extend with its own `Failed` producer), `ObjectiveProgress`, `CurrentObjective` (resource, `Option<Objective>`, `None` until task 042 wires worldgen), `CurrentWorldOutcome` (resource, last `evaluate` result), `evaluate()`, `ObjectivesPlugin`.
- `src/lib.rs`: exported `pub mod objectives;`.
- `src/main.rs`: registered `ObjectivesPlugin` in the plugin tuple, after `SimPlugin`.
- Design note: `TriggerBloom` deliberately isn't a sustained condition like the other two — a bloom is a triggering *event* (population crosses a threshold in a single tick), not a state to hold, so it skips `evaluate_sustained` and sets `satisfied` directly.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` (new) | `Objective`, `ObjectiveProgress`, `evaluate()`. |
| `src/sim.rs` | Hook that calls `evaluate` after `advance_tick`; possible reading of `EraCompleted` (task 035) to handle the era boundary. |

---

## 🧩 Technical Context

**GDD §8, literal examples**:
> - "Achieve a biosphere with ≥3 coexisting species for 50 ticks."
> - "Grow a species that survives in the toxic zone."
> - "Trigger a bloom of a specific type."

**GDD §11**: the list of always-visible HUD panels includes "current objective" — this task doesn't touch the HUD (task 043), but the `ObjectiveProgress` resource it exposes is what 043 will read.

**`advance_tick`** (`src/sim.rs`, task 035 extends it with `EraCompleted`): this task adds a separate system (not necessarily inside `advance_tick` itself) that reads `SimWorld` state after each tick and updates `ObjectiveProgress`.

- **Current behavior**: no objective concept exists in the code — `ui.rs:243` has a literal placeholder comment (`// Placeholder: objective arrives in Phase 3 (GDD §8).`) marking where the UI will need to hook in (task 043).
- **Desired behavior**: given an assigned `Objective` (for now, in this task, even one hand-built in tests — procedural assignment is task 042), the engine correctly tracks progress tick by tick and produces an outcome when the condition is satisfied.

---

## 🔨 Suggested Implementation

1. Define `Objective` and `ZoneKind` (or reuse an existing zone type in `world.rs` if present — verify before introducing a new one).
2. Define `ObjectiveProgress` with the fields needed to track the consecutive count (e.g. `consecutive_ticks: u32`, `satisfied: bool`).
3. Write `evaluate`: for `Coexistence`, count species with population > 0 on the grid; for `SurviveIn`, check whether an organism of a designated species exists in the indicated zone; for `TriggerBloom`, define an observable threshold (e.g. a species' population above N in an area in a single tick) — avoid introducing new concepts not anchored to data already present in `SimWorld`.
4. Wire `evaluate` to a Bevy system that runs after `advance_tick`, reading `SimWorld` read-only (consistent with `SimSet::Sync`, TECH_DESIGN.md §3.3).
5. Write the tests with hand-built `SimWorld`s (pattern already used in the existing `sim.rs`/`world.rs` tests).

---

## ⚠️ Constraints and Caveats

- **Don't reveal the matrix**: no objective must depend on a `TagMatrix`/`MatrixKnowledge` value — only on observable state (population, position, events).
- **Determinism**: `evaluate` is a pure function on `&SimWorld`, no RNG of its own, no external state.
- **Don't generate objectives procedurally yet**: this task defines the type and the evaluation engine; *which* objective to assign to a world is task 042.
- **Don't touch the UI**: consuming `ObjectiveProgress` for the HUD is task 043.

---

## 🔗 Dependencies

- **Depends on**: none (uses existing Phase 2 events, testable against the current hardcoded world).
- **Blocks**: 041 (failure conditions shares `WorldOutcome`), 042 (worldgen generates `Objective` instances), 043 (HUD reads `ObjectiveProgress`).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/040-objectives-type-evaluation-engine.md)"$'\n\nExecute this task in the current project.'
```
