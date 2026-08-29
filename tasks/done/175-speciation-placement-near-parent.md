# Task 175 — New species from speciation should place near where pressure actually accrued

> **ID**: `175`
> **Category**: Bug fix / feel
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

Playtest (`playtest_outcome.md`, gameplay note #10) observed a new species
born from speciation placed in a different, non-adjacent biome from the
parent species' cell the player was watching. Confirmed by reading
`src/sim.rs`:

- `SelectionThresholdCrossed::cell` (`src/sim.rs:193-198`) is explicitly
  documented as "a representative location, not necessarily where the
  pressure was mostly accrued" — it's whichever cell's tick happened to
  push the species-wide tally over threshold, not a spatially meaningful
  point.
- `accumulate_selection_pressure` (`src/sim.rs:634-682`) sums pressure
  **per species across the entire map**, not per cell.
- `speciate` (`src/sim.rs:755-813`) simply replaces the species id of
  whatever population already occupies `event.cell` — for a multi-biome
  species, that cell can be anywhere the species has a population, with no
  relation to where the player has been watching it.

This is working as coded, but the result reads as arbitrary/unearned to a
player tracking a specific population, and undercuts the "attachment to
placed species" feel `playtest_outcome.md`'s checkpoint evidence calls out
as missing.

Design source: `playtest_outcome.md` gameplay note #10. Note: task 170
(speciation cause readability, queued) is about naming *why* a speciation
happened in the reveal text — it does not touch *where* the new species
ends up. This task is a distinct, complementary fix; check 170's landed
state before starting to avoid `EraEvolutionReveal` merge conflicts.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] Track, per species, which cell has contributed most to its
      accumulated selection pressure (or the most recent/highest-pressure
      cell at threshold-crossing time), instead of an arbitrary
      "whichever tick tipped it over" cell — smallest change that gives
      `SelectionThresholdCrossed::cell` (or a new field) a spatially
      meaningful value without redesigning `accumulate_selection_pressure`'s
      per-species aggregation.
- [ ] `speciate` places/converts the new species at that meaningful cell
      (same replace-in-place mechanism as today, just fed a better cell).
- [ ] Determinism preserved: given a fixed seed, the chosen cell is
      reproducible (no `HashMap` iteration order dependency, no
      `rand::rng()` — per `TECH_DESIGN.md` §5).
- [ ] Unit test: a species occupying cells in two different biomes, with
      pressure concentrated in one, speciates into a cell within/adjacent
      to that biome, not the other.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `SelectionThresholdCrossed` (193-198), `accumulate_selection_pressure` (634-682), `speciate` (755-813). |

---

## ⚠️ Constraints and Caveats

- Deterministic: no `rand::rng()`, no `HashMap` iteration in tick logic —
  per `TECH_DESIGN.md` §5.
- Don't touch `speciate`'s three-branch edit logic (tag added / thermal
  shift / sea tolerance) — task 170's scope, not this task's.

---

## 🔗 Dependencies

- **Depends on**: none directly, but check task 170's landed state first
  (both touch `sim.rs` speciation code, low conflict risk if sequenced).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/175-speciation-placement-near-parent.md)"$'\n\nExecute this task in the current project.'
```
