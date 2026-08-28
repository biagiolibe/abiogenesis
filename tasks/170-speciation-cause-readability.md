# Task 170 — Speciation genome before/after diff

> **ID**: `170`
> **Category**: Feature (UI / notebook)
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-28

---

## 🎯 Objective

Task 142 (done) already names the **dominant pressure stimulus** in the
end-of-era reveal card when a speciation fires. What's still missing: a
concrete **before/after diff** of what actually changed in the new species'
genome (trait gained, tolerance shifted, or metabolism coefficient changed) —
today the reveal explains *why* in general terms ("interaction harm caused
this") but not *what* changed mechanically.

Design source: `abiogenesis-gdd.md` §5.11.

Risk being addressed: R9 from the external design-review pass (2026-08-28) —
"after a speciation, a player should be able to say the change was caused by
X *and see what actually changed*." The cause is now named (142); the effect
still isn't shown.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors; `cargo clippy -- -D warnings` clean.
- [ ] The reveal entry (or notebook/Chronicle entry, per task 153) for a
      speciation event shows a concrete before/after: which trait, tolerance,
      or metabolism coefficient changed on the new species relative to its
      parent, and to what value/state.
- [ ] Reuses the existing `EraEvolutionReveal`/reveal-card plumbing (140, 142)
      rather than adding a parallel notification system.
- [ ] Synthesised (`Splice`) and evolved (speciation) species remain visually
      and narratively distinguishable, per §5.11.
- [ ] No new magic numbers — any new thresholds/config live in `SimConfig`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/evolution.rs` (or equivalent, per current module layout) | Speciation logic, `SelectionThresholdCrossed` event, dominant-stimulus tagging |
| notebook/Chronicle module (task 153) | Where the surfaced entry should likely land |
| `src/config.rs` | `EvolutionConfig` weights already live here |

---

## 🧩 Technical Context

- **Current behavior**: `SelectionThresholdCrossed` fires, `speciate` applies a
  genome edit, and `EraEvolutionReveal`/the reveal card names the dominant
  stimulus (task 142). The actual genome edit (trait/tolerance/metabolism
  change) is applied but never surfaced.
- **Desired behavior**: the reveal (or Chronicle entry) also shows the concrete
  before/after — e.g. "gained tolerance to toxicity" or "temperature tolerance
  widened" — derived from diffing the parent and child genome.

---

## 🔨 Suggested Implementation

1. In `speciate` (`src/sim.rs`, per task 142's file map), capture the parent
   genome state before the edit and the child genome state after.
2. Extend `EraEvolutionReveal` with the diff (or a summarizable field), and
   extend `text::era_reveal_evolution_line` (or add a sibling function) to
   render it as a short natural-language clause — consistent with how the
   dominant-stimulus clause is already rendered (no raw numbers).
3. Wire it through `screens.rs`'s reveal card, same as task 142 did.
4. Verify visual/narrative distinction between evolved and synthesised species
   still holds once this entry is added.

---

## ⚠️ Constraints and Caveats

- **Style**: follow `TECH_DESIGN.md` conventions — sim/world logic stays
  deterministic and headless; presentation stays out of `sim`/`world`/`config`.
- Reuse existing reveal/notebook plumbing (tasks 140, 153) rather than adding a
  parallel notification system.

---

## 🔗 Dependencies

- **Depends on**: 106-109 (evolution by speciation, done), 140 (reveal beat, done), 142 (dominant-stimulus naming, done), 153 (notebook Chronicle, done)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/170-speciation-cause-readability.md)"$'\n\nExecute this task in the current project.'
```
