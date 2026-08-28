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

- [x] `TimeConfig` gains a season length (in pulses) and expresses the era in
      **seasons**, not directly in pulses. `era_ticks` stops being the primary
      knob.
- [x] The action point budget refills **per season**, not per era
      (`point_budget_per_era` → per-season equivalent). The number of decisions
      per world stays in the same order of magnitude as today.
- [x] The per-world era budget is lowered accordingly. Doc's starting point: if
      an era is ~4 seasons, the current `era_budget_early`/`era_budget_late`
      of `60`/`45` become roughly `15`/`11`. Total run length in pulses stays
      comparable — the rhythm changes, not the duration.
- [x] `EvolutionConfig::selection_pressure_threshold` is **retuned** for the
      longer era. Left at `20.0` (tuned against a 25-pulse era) speciations
      would fire many times per era, trivialising both the `Speciation`
      objective and (later) Emersione.
- [x] Objective durations currently expressed in eras are re-expressed in
      seasons where that is the natural unit — case by case, not a mechanical
      conversion. (Full objective rework is task 154; here only the unit.)
- [x] `assets/sim_config.ron` updated in the same commit for every new/renamed
      field — `tests/config_ron_sync.rs` fails otherwise.
- [x] Tests that assume `era_ticks = 25` or era-multiple tick counts are
      updated, not deleted (see `config.rs:613` and the era-relative time
      readout from task 117).
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.

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

## ✅ Resolution (2026-08-28)

**Numeric choice.** `season_pulses = 25` (identical to the old `era_ticks`)
and `seasons_per_era = 4` (the design doc's own suggested ratio), so
`era_ticks() = 100`. Keeping the season's length equal to the old era's meant
every duration already tuned against the 25-pulse era (`grace_eras`,
`onboarding_era_ticks`, the objective tick bases `100`/`75` = 4/3 "eras")
carries over unchanged onto the season, which now plays the role the era used
to. Only what's genuinely narrative moved to the new, rarer era: the world's
`era_budget` (`60/45` → `15/11`, matching the 4:1 ratio) and the per-species
notebook log cadence (`EraCompleted` now fires every 4 seasons).

**Mechanism.** `SimWorld` gained a `season: u32` field alongside `era`.
`tick_and_complete_season` (renamed from `tick_and_complete_era`) advances
`world.season` and refills `ActionBudget` every `season_pulses` ticks, and
only increments `world.era`/fires `EraCompleted` every `seasons_per_era`
seasons. Everything that gates the player's actual decision cadence moved
from era to season: `SeasonProgress` (renamed from `EraProgress`), the
onboarding shortening (`onboarding_seasons`/`onboarding_season_pulses`), the
grace period (`grace_seasons`, checked against `world.season`), and
reproduction eligibility (`Organism::born_season < world.season` — previously
`born_era < world.era`, which would otherwise have quietly made newborns wait
4x longer to reproduce, an unintended balance change disguised as a unit
rename). Purely narrative bookkeeping (`species_seeded_era`,
`species_origin_era`, the Notebook's log entries) stayed on `world.era`.

**`selection_pressure_threshold` retune.** Measured `SelectionThresholdCrossed`
crossings per (new, 100-tick) era across 12 greedily-seeded world-0 seeds, 15
eras each, at several candidate thresholds:

```text
threshold   20.0: 180 era-windows, mean 0.71/era, 104/180 eras with 0, max 5
threshold   40.0: 180 era-windows, mean 0.43/era, 123/180 eras with 0, max 3
threshold   60.0: 180 era-windows, mean 0.33/era, 129/180 eras with 0, max 3
threshold   80.0: 180 era-windows, mean 0.25/era, 135/180 eras with 0, max 1
threshold  100.0: 180 era-windows, mean 0.22/era, 141/180 eras with 0, max 2
```

Left at `20.0`, an era regularly bundles 2-5 speciation events (a burst, not a
beat). `80.0` was chosen: mean 0.25/era, max 1 in the sample, 75% of eras with
none at all — a speciation reads as a rare, single, notable event again,
matching the doc's "epocale, non di routine" framing.

**HUD.** `era_tick_line` → `season_tick_line`: `"Era {era} · Season {season} ·
pulse {current}/{total}"`. The sustained-objective progress indicator
(`objective_panel`) now reads in seasons (`eras_progress` → `seasons_progress`,
`ERA_PROGRESS_DOT_CAP` → `SEASON_PROGRESS_DOT_CAP`).

**Not done here (by design):** the end-of-era reveal beat (task 140), the
full objective unit rework (task 154), and `two_bot_survey.rs`'s harness now
tracks `short_term_seasons`/`full_seasons` instead of the old era-keyed
fields — an incidental but necessary fix, since the survey re-implements the
tick loop by hand rather than reusing `sim.rs`'s systems.

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/135-three-level-time-scale.md)"$'\n\nExecute this task in the current project.'
```
