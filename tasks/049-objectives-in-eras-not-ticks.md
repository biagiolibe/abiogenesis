# Task 049 — Retune sustained objectives to era scale, show eras not ticks in the HUD

> **ID**: `049`
> **Category**: Balance / UI
> **Priority**: 🟡 P2
> **Estimate**: ~1-2h
> **Assigned to**: unassigned
> **Session**: 2026-08-06 playtest session

---

## 🎯 Objective

`Objective::Coexistence`/`SurviveIn`'s sustained-duration fields (`ticks: u32`) are tuned in a unit the player never consciously operates in. GDD §16.4's loop is Plan → Advance (one era = `SimConfig::time.era_ticks`, default `25`) → Observe — the player presses "advance era" (space), not "advance one tick." With `coexistence_ticks_base = 50` and `survive_in_ticks_base = 20`, a world's objective clears in **2 era-presses or less**, with no meaningful decision space in between. The `043` objective HUD panel also displays the raw tick count (`sustained_progress_bar_text`, "`consecutive_ticks / ticks`"), surfacing a unit GDD §11's whole design stance says the player shouldn't need to think in.

Internal evaluation stays tick-granular (`evaluate_sustained` already runs every tick, `SimSet::Advance`-ordered — this matters for task 041's total-extinction check reacting mid-era, not just objectives) — only the *tuning* and the *display* change.

---

## 📋 Acceptance Criteria

- [ ] `SimConfig`'s `coexistence_ticks_base`/`survive_in_ticks_base` raised to a sensible multiple of `era_ticks` (e.g. 3-6 eras' worth — concretely land on a number, don't leave it exactly on an era boundary if that reads as a coincidence; document the reasoning in a comment near the config fields).
- [ ] The HUD's objective progress bar (`ui.rs::objective_panel`, `text::sustained_progress_bar_text`) shows progress in whole/partial eras (e.g. "2 / 4 eras"), not raw ticks — `consecutive_ticks` still drives the underlying fraction, just formatted via `era_ticks`.
- [ ] The objective's own description line (`text::coexistence_objective_line`/`survive_in_objective_line`) reads in terms a player would recognize (era count), not "N ticks."
- [ ] `TriggerBloom` is unaffected (it's not a sustained-duration objective, no tick count involved) — don't touch it.
- [ ] `cargo clippy -- -D warnings` clean, `cargo test` green — update/add tests for the new config defaults and the era-formatted display.
- [ ] Manual verification: start a run, confirm the objective panel reads in eras and takes a plausible number of era-advances to clear (not 1-2 button presses).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/config.rs` | `ObjectivesConfig::coexistence_ticks_base`/`survive_in_ticks_base` (lines ~342-355) — the values to retune. |
| `src/objectives.rs` | `evaluate_sustained` (unchanged logic, still tick-granular), `Objective::Coexistence`/`SurviveIn`'s `ticks` field (keep the field name/type — only its *display* changes, per the "keep it simple" spirit; renaming to `eras` is out of scope unless the ratio to `era_ticks` isn't exact, see caveats). |
| `src/text.rs` | `sustained_progress_bar_text`, `coexistence_objective_line`, `survive_in_objective_line` — need era-aware formatting. |
| `src/ui.rs` | `objective_panel` — passes `config.time.era_ticks` through to the formatting call. |

---

## 🧩 Technical Context

<!-- TODO: add relevant code snippets and file paths -->

**Current behavior**: `ObjectiveProgress::consecutive_ticks` counts ticks; the HUD shows `"{consecutive_ticks} / {ticks} ticks"` verbatim (`text.rs::sustained_progress_bar_text`).

**Desired behavior**: the player-facing number is in eras. `consecutive_ticks / era_ticks` (integer or fractional, your call) for "eras held so far," `ticks / era_ticks` for "eras required" — both derived from existing data, no new resource needed beyond `SimConfig` (already available in `ui.rs::hud_panel`'s `Res<SimConfig>`).

---

## 🔨 Suggested Implementation

1. Pick new base values for `coexistence_ticks_base`/`survive_in_ticks_base` as exact multiples of `era_ticks` (25) — e.g. 100 (4 eras) and 75 (3 eras) — or intentionally non-exact if a "close but must actually finish the era" texture is wanted; document whichever you pick.
2. Update `text::sustained_progress_bar_text` (or add a new function) to take era-based numbers instead of raw ticks, computed by the caller (`ui.rs::objective_panel`) from `consecutive_ticks`/`ticks` and `config.time.era_ticks` — consistent with `text.rs`'s existing constraint (module doesn't touch `SimWorld`/derived state directly, `ui.rs` computes concrete values).
3. Update the objective description lines similarly if they currently name a tick count anywhere (check `worldgen.rs`'s generation too, in case severity scaling produces a decimal era count worth rounding intentionally).
4. Update/add config-default and formatting tests.
5. Manual verification with `cargo run`.

---

## ⚠️ Constraints and Caveats

- **Don't change evaluation granularity**: `evaluate_sustained`/`evaluate_world` must keep running every tick — only tuning and display change. Switching to era-boundary-only evaluation is explicitly the "bigger" alternative this task rejected in favor of the simpler retune-and-reformat approach (see the 2026-08-06 planning discussion).
- **`objective_severity` scaling** (`worldgen::scale_severity`) multiplies the new base values by a per-world severity factor — check that later/harder worlds still land on plausible era counts after scaling, not tiny fractions that round oddly.
- **No magic numbers**: any new constant goes in `SimConfig` (CLAUDE.md convention) — don't hardcode `era_ticks` conversions inline without going through the config value.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/049-objectives-in-eras-not-ticks.md)"$'\n\nExecute this task in the current project.'
```
