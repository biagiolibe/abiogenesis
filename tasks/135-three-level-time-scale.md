# Task 135 — Three-level time scale: Pulse → Season → Era

> **ID**: `135`
> **Category**: Architecture / Balance
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-27 redesign adoption planning

---

## 🎯 Objective

Introduce **Season** as a level between Pulse and Era, and move the player's
unit of decision onto it.

Today an era is 25 pulses and the action budget refills per era. The redesign
makes the era **much longer** and **rare** (the unit of *narration*, where the
end-of-era reveal lands — task 140) while the season becomes the unit the player
actually plays in.

**This task comes before task 136 (energy coefficients), reversing the order in
`redesign/abiogenesis-INDEX.md`.** Both documents state they depend on the
other; the clock is the independent variable, so tuning coefficients first
means tuning them twice.

Design source: `redesign/processed/abiogenesis-time-scale-reveal.md` §1, §2, §6.

---

## 📋 Acceptance Criteria

- [ ] `TimeConfig` gains a season length (in pulses) and expresses the era in
      **seasons**, not directly in pulses. `era_ticks` stops being the primary
      knob.
- [ ] The action point budget refills **per season**, not per era
      (`point_budget_per_era` → per-season equivalent). The number of decisions
      per world stays in the same order of magnitude as today.
- [ ] The per-world era budget is lowered accordingly. Doc's starting point: if
      an era is ~4 seasons, the current `era_budget_early`/`era_budget_late`
      of `60`/`45` become roughly `15`/`11`. Total run length in pulses stays
      comparable — the rhythm changes, not the duration.
- [ ] `EvolutionConfig::selection_pressure_threshold` is **retuned** for the
      longer era. Left at `20.0` (tuned against a 25-pulse era) speciations
      would fire many times per era, trivialising both the `Speciation`
      objective and (later) Emersione.
- [ ] Objective durations currently expressed in eras are re-expressed in
      seasons where that is the natural unit — case by case, not a mechanical
      conversion. (Full objective rework is task 154; here only the unit.)
- [ ] `assets/sim_config.ron` updated in the same commit for every new/renamed
      field — `tests/config_ron_sync.rs` fails otherwise.
- [ ] Tests that assume `era_ticks = 25` or era-multiple tick counts are
      updated, not deleted (see `config.rs:613` and the era-relative time
      readout from task 117).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/config.rs` | `TimeConfig` (`era_ticks`, `era_budget_early/late`, `point_budget_per_era`, `onboarding_era_ticks`), `EvolutionConfig::selection_pressure_threshold`, `DifficultyConfig`. |
| `assets/sim_config.ron` | Must stay in sync (task 128's `tests/config_ron_sync.rs`). |
| `src/state.rs`, `src/run.rs`, `src/run_flow.rs` | Era/run lifecycle and budget refill. |
| `src/input.rs` | `ActionBudget` refill point; the advance-era keybinds (full control scheme is task 150). |
| `src/ui.rs` | Time readout and "moves remaining" — both now refer to the season. |
| `src/objectives.rs` | Duration-typed objectives. |
| `tests/balance.rs`, `tests/determinism.rs` | Fixed tick horizons keyed to `era_ticks`. |

---

## 🧩 Technical Context

- **Current behavior**: `era_ticks: 25`, `era_budget_early: 60`,
  `era_budget_late: 45`, `point_budget_per_era: 3`. `onboarding_era_ticks: 8`
  shortens world 0's first eras (task 079's grace period).
- **Desired behavior**: pulse → season → era, budget on the season, far fewer
  and much heavier eras.

`onboarding_era_ticks` needs an explicit decision: with a longer era, the
onboarding shortening probably belongs on the **season**, not the era. Whichever
way it goes, task 079's adaptive grace period must keep working — it gates
total-extinction failure, never era-budget exhaustion.

---

## 🔨 Suggested Implementation

1. Add season length to `TimeConfig`; derive era length as `seasons_per_era ×
   season_pulses` rather than keeping an independent `era_ticks`.
2. Move `ActionBudget` refill from the era boundary to the season boundary.
3. Lower the era budgets; recompute what the run's total pulse count becomes and
   record it in the Resolution section (it should stay near today's).
4. Retune `selection_pressure_threshold`: measure how many
   `SelectionThresholdCrossed` events fire per era under the new length across a
   handful of seeds, and pick a threshold that keeps speciation a notable event
   rather than a per-era routine. Record the measurement.
5. Update `sim_config.ron`, the HUD readout unit, and the affected tests.

---

## ⚠️ Constraints and Caveats

- **No magic numbers** — every new coefficient lives in `SimConfig`.
- Do **not** implement the end-of-era reveal beat here; that is task 140.
- Do **not** retune the energy coefficients here; that is task 136, which
  depends on this task's final numbers.
- Continuous advancement (`P`) and "advance to next notable event" (`G`) are
  part of the control scheme, task 150 — not this task.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: 136, 140, 154

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/135-three-level-time-scale.md)"$'\n\nExecute this task in the current project.'
```
