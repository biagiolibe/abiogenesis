# Task 059 — Objective pacing / multi-objective design

> **ID**: `059`
> **Category**: Feature (design)
> **Priority**: 🟡 P2
> **Estimate**: unknown — design discussion needed before scoping
> **Assigned to**: unassigned
> **Session**: 2026-08-07 (playtest finding, same session as tasks 057/058)

**Status**: `[?]` proposal — not yet approved. The problem is confirmed real, but
the fix requires a design decision (see "Open questions" below) before this can
become an actionable, scoped `[ ]` task.

---

## 🎯 Objective

A playtester reported that worlds end too quickly, feel under-explored, and some
objectives are too easy — they want longer, more substantial runs: more species
placed, more matrix dependencies discovered, before a world's objective clears.

This is a deliberate MVP scope cut, not a bug: task 042 explicitly deferred
multi-objective worlds ("Don't introduce bonus objectives: explicitly out of the
MVP", GDD §8: "planned in principle... but after the clean primary-objective→
advance core. Not in the minimal MVP"), and task 049 already retuned this once —
raising `coexistence_ticks_base`/`survive_in_ticks_base` after a prior playtest
found worlds cleared "in 2 era-presses or less, with no meaningful decision
space." That retune only touched two of the three objective kinds.

Concretely, at default config (`SimConfig::default()`), for `world_index = 0`
(`objective_severity = 1.0`):

| Objective | Base value | Eras to satisfy (`era_ticks = 25`) | Retuned by 049? |
|---|---|---|---|
| `Coexistence` | `coexistence_ticks_base = 100` | 4 eras sustained | Yes |
| `SurviveIn` | `survive_in_ticks_base = 75` | 3 eras sustained | Yes |
| `TriggerBloom` | `trigger_bloom_population_threshold_base = 8` | single-tick check, no sustain | No |

Early worlds get `era_budget_early = 40` eras of runway — an objective clearing
in 3-4 eras (or a single tick, for `TriggerBloom`) leaves most of that budget,
and most of the hidden matrix, unexplored before `WorldCleared` fires and a
brand-new world (fresh grid, species, matrix) replaces the current one. There is
currently no way to keep playing the same world once its objective is met
(`apply_tick_outcome` transitions to `GameState::WorldCleared` the same tick the
objective satisfies; the only action from there is "Continue" → an entirely new
world via `build_world`).

---

## 🧭 Open questions to resolve before approval

This task cannot be scoped into concrete acceptance criteria yet — pick a
direction (or a combination) before turning this into an approved `[ ]` task:

1. **Multiple/sequential objectives per world** — e.g. a world requires clearing
   2-3 objectives (possibly of different kinds) before `WorldCleared` fires.
   Bigger structural change: touches `CurrentObjective`'s `Option<Objective>`
   shape, `evaluate_current_objective`, worldgen's `generate_objective`, and the
   HUD's objective panel (would need to show progress across several).
2. **Retune `TriggerBloom` alone** — it's the one objective kind task 049 never
   touched, and single-tick population checks are inherently easier to satisfy
   opportunistically than a sustained condition. Smallest possible fix, but
   doesn't address "more species placed / more dependencies discovered" — a
   world could still clear via a single opportunistic species bloom.
3. **Gate the objective on matrix discovery** — e.g. don't let `WorldCleared`
   trigger until some minimum number of matrix cells are confirmed
   (`MatrixKnowledge`), regardless of the objective's own condition. Directly
   targets "more dependencies discovered" but changes what "objective" means
   (couples two previously independent systems: `objectives.rs` and
   `notebook.rs`'s `MatrixKnowledge`).
4. **Raise all three bases again, more aggressively** — cheapest, but task 049
   already tried "raise the numbers" once; diminishing returns if the real
   complaint is structural (one objective, not just an easy one).

Also worth deciding: does `era_budget` need to grow too, or is 40 eras already
enough runway once the objective itself takes longer to clear?

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` | `CurrentObjective`, `evaluate_current_objective`, `apply_tick_outcome` — where the single-objective structure and immediate `WorldCleared` transition live. |
| `src/worldgen.rs` | `generate_objective` — where objective kind/severity is chosen and parametrized. |
| `src/config.rs` | `ObjectiveConfig` (base thresholds), `DifficultyConfig` (severity ramp), `TimeConfig` (`era_budget_early`/`era_ticks`). |
| `tasks/done/042-worldgen-objective-generation.md` | Prior art: explicit MVP scope cut on bonus/multi-objectives. |
| `tasks/done/049-objectives-in-eras-not-ticks.md` | Prior art: the last pacing retune, and the playtest finding that motivated it. |

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

Not yet actionable — resolve the open questions above first (a design
discussion, not a coding session), then rewrite this file's Objective/Acceptance
Criteria sections and flip its status to `[ ]` before delegating:

```bash
claude "$(cat tasks/059-objective-pacing-design.md)"$'\n\nExecute this task in the current project.'
```
