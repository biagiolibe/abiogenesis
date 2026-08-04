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

- [ ] New `src/objectives.rs` module with `enum Objective`, at least the three variants from the GDD examples: `Coexistence { min_species: u32, ticks: u32 }`, `SurviveIn { zone: ZoneKind, ticks: u32 }` (where `ZoneKind` covers at least the toxic zone), `TriggerBloom { .. }` (define the minimal parameters needed to recognize a "bloom" — check whether a bloom concept already exists in the code, otherwise define it here in observable terms, e.g. a species' population above a threshold in an area).
- [ ] `ObjectiveProgress` resource that tracks progress toward the current objective (e.g. a counter of consecutive ticks in which the condition is true).
- [ ] Pure function `pub fn evaluate(objective: &Objective, world: &SimWorld, progress: &mut ObjectiveProgress) -> WorldOutcome` (or similar — see task 041 for the shared `WorldOutcome`), called by a system in `src/sim.rs` after `advance_tick`.
- [ ] **"≥3 species for 50 ticks" uses a consecutive count**: if the condition is interrupted (e.g. a species goes extinct), the counter resets to zero, it does not decrement — this must be handled explicitly at the era boundary (50 ticks = 2 eras at `ERA_TICKS=25`), not implicitly assumed.
- [ ] Objectives are expressed **only on quantities observable by the player** (species count, population, zone occupancy, bloom events) — **never on cells of the hidden biochemical matrix**: an objective that indirectly revealed a matrix value would break the deduction pillar (GDD §11).
- [ ] `evaluate` is headless-testable: no dependency on `bevy::render`/`bevy_egui` (invariant 2), testable with a hand-built `SimWorld`, without waiting for the worldgen of tasks 038/039.
- [ ] Tests: at least one test per `Objective` variant, including a test that verifies the consecutive count resets when the condition is interrupted.
- [ ] `cargo clippy -- -D warnings` clean, `cargo test` green.

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
