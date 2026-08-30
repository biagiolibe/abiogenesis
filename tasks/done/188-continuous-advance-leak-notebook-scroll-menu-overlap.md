# Task 188 — Continuous-advance state leak, notebook outer scroll, pause/confirm overlap

> **ID**: `188`
> **Category**: Bugfix (one gameplay-logic, two UI)
> **Priority**: 🔴 P1 (continuous-advance leak is a real gameplay bug, not cosmetic)
> **Estimate**: ~45min
> **Assigned to**: Claude CLI
> **Session**: 2026-08-31

---

## 🎯 Objective

User report after rebuilding post-186/187: the Notebook still clips content
(no way to reach anything below the fold, e.g. the Chronicle section), the
pause menu and confirmation dialog visually overlap, and — most seriously —
**after enabling auto-advance, `space` starts advancing whole eras instead
of seasons, and this persists across returning to the main menu and
starting a new world.**

Root causes, all confirmed by reading the code (not guessed):

1. **`ContinuousAdvance`/`ContinuousAdvancePulseCounter` are app-global
   resources never reset by `run_flow::start_world`** (the function
   `menu::start_run`/`advance_to_next_world`/`retry_world` all funnel
   through). Every *other* piece of per-world UI/input state
   (`PauseMenuOpen`, `WorldTouched`, `SelectedAction`, …) is bundled into
   `WorldResetParams` and reset there — these two were simply never added
   to that list. Leaving auto-advance on and starting a fresh world (any
   path: New run, Continue, Retry) leaves the background
   `input::continuous_advance` system silently auto-playing the new world.
   Since `input::start_era` and the `shift+space` full-era system both gate
   on `!continuous.0`, the player's own `space` presses do nothing directly
   while the stuck auto-pilot keeps ripping through seasons/eras — reading
   exactly like "space now advances whole eras."
2. **`notebook_window`'s content had no outer scroll.** Only the
   Observation log had its own internal, height-capped `ScrollArea` — the
   Hypothesis grid (fixed-size canvas), Catalog (unbounded species list),
   and Chronicle (unbounded history) below it had no way to become visible
   once the panel's total content exceeded the viewport height. Task 187's
   `.wrap()` fixes stopped text from being clipped *horizontally* but did
   nothing for this *vertical* overflow — a different bug.
3. **`pause_menu` and `confirmation_dialog` are both `egui::Window`s
   anchored near screen-center** with only a 40px vertical offset between
   them; `pause_menu` had no guard against rendering while a confirmation
   is pending, so triggering "Abandon"/"Save and exit" showed both windows
   simultaneously, overlapping.

---

## 📋 Acceptance Criteria

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] `WorldResetParams` gains `continuous: ResMut<ContinuousAdvance>` and
      `continuous_pulse_counter: ResMut<ContinuousAdvancePulseCounter>`;
      `start_world` resets both to their `Default` (off, counter zeroed).
      Regression test added
      (`advancing_to_the_next_world_turns_off_a_stale_continuous_advance`).
      Test-only `resource_world` harness and `input.rs`'s two `reseed_world`
      tests updated to insert the two new resources `WorldResetParams` now
      requires.
- [x] `notebook_window`'s body (everything below the title-bar clearance)
      wrapped in one outer `egui::ScrollArea::vertical()`, so the
      Hypothesis grid/Catalog/Chronicle are always reachable regardless of
      total content height. The Observation log keeps its own nested,
      height-capped, stick-to-bottom `ScrollArea` unchanged.
- [x] `pause_menu` returns early (renders nothing) while
      `PendingConfirmation::kind` is `Some(..)`, so the confirmation dialog
      never has to share the screen with the pause menu underneath it.
- [-] Manual check — skipped per this session's standing "no live
      verification" instruction; ask the user to rebuild and re-verify all
      three, especially the continuous-advance fix (toggle Auto on, return
      to menu, start a new world, confirm `space` advances one season as
      normal).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/run_flow.rs` | `WorldResetParams`, `start_world` — the continuous-advance reset. |
| `src/notebook.rs` | `notebook_window` — outer `ScrollArea`. |
| `src/ui.rs` | `pause_menu` — pending-confirmation guard. |
| `src/input.rs` | Two `reseed_world` tests — new resources inserted into their scratch `App`. |

---

## ⚠️ Constraints and Caveats

- `sim`/`world`/`config` untouched — this is UI/input/reset-glue only.
- Didn't touch `input.rs`'s `start_era`/full-era/`continuous_advance`
  systems themselves — their `!continuous.0` gating logic was always
  correct; the bug was purely that nothing ever turned `continuous.0` back
  off between worlds.

---

## 🔗 Dependencies

- **Depends on**: 152 (`ContinuousAdvance` itself), 045 (`WorldResetParams`/
  `start_world`), 150 (pause menu/confirmation dialog), 186/187 (notebook
  chrome work this task's outer-scroll fix builds on).
- **Blocks**: none.
