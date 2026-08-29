# Task 176 — Continuous-advance needs its own, slower cadence

> **ID**: `176`
> **Category**: Bug fix / feel
> **Priority**: 🟢 P3
> **Estimate**: ~30min
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

Playtest (`playtest_outcome.md`, issue I.11) reported continuous-advance
(task 152) feeling too fast — "uno scorrere lento" was expected, not a
sense of racing past events. Confirmed in `src/input.rs:405-459`:
`continuous_advance` fires one season-pulse per `FixedUpdate` step, reusing
`TimeConfig::era_tick_hz` (default `8.0`, `src/config.rs:191-194,241`) —
the exact same rate used for the manual space-bar season-advance animation.
There is no separate, slower cadence for the toggle; task 152's own doc
comment states this sharing was deliberate at the time.

Design source: `playtest_outcome.md` issue I.11.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] New config constant (e.g. `TimeConfig::continuous_advance_tick_hz`,
      lower than `era_tick_hz`) drives `continuous_advance`'s pulse rate
      independently of the manual-advance animation's `Time<Fixed>` hz.
      No magic numbers — value lives in `SimConfig`, mirrored in
      `sim_config.ron`.
- [ ] Manual space-bar advance animation speed is unaffected.
- [ ] Manual check: toggling continuous advance visibly reads as a slow,
      followable drift rather than a race through seasons.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `toggle_continuous_advance`/`continuous_advance` (405-459). |
| `src/config.rs` | `TimeConfig` (`era_tick_hz` 191-194,241) — add the new field. |
| `sim_config.ron` | Mirror the new config field. |

---

## 🔗 Dependencies

- **Depends on**: 152 (continuous-advance toggle, already shipped).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/176-continuous-advance-dedicated-cadence.md)"$'\n\nExecute this task in the current project.'
```
