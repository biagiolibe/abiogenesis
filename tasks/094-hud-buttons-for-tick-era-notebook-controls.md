# Task 094 — On-screen buttons for tick/era/notebook controls

> **ID**: `094`
> **Category**: Feature / UX
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-10, user-requested during a debugging/planning
> discussion — every time-control action is currently keyboard-only, with
> no on-screen affordance at all.

---

## 🎯 Objective

Every control that advances the simulation or toggles the notebook is
keyboard-only today (confirmed by a full-repo grep of `KeyCode::` usage):
`Space` starts/resumes an era, `N` advances a single tick, `Tab` toggles
the notebook, `R` reseeds the world. None of these have an equivalent
on-screen button — a player who doesn't know or forgets the shortcut has no
way to discover or trigger them from the HUD itself, unlike the four
`ActionMode` options (`Seed`/`Stress`/`Cull`/`Splice`), which already have
both a keyboard-free icon row (`action_icon_row`, task 030) *and* whatever
shortcuts exist for them.

Add HUD buttons for the tick/era/notebook controls, **coexisting** with
their keyboard shortcuts (both must keep working, neither replaces the
other) — same pattern the action icon row already establishes for
`ActionMode`.

---

## 📋 Acceptance Criteria

- [ ] The HUD (`hud_panel`, `ui.rs`) gains buttons for: advance one tick
      (`N`), start/resume an era (`Space`), toggle the notebook (`Tab`).
      Whether "reseed" (`R`) also gets a button is the implementer's call —
      it's destructive (throws away the current world) and easy to
      misclick, so consider a confirmation affordance or leaving it
      keyboard-only if a button reads as too easy to trigger by accident;
      document whichever choice is made and why.
- [ ] Each button calls the *same* underlying logic its keyboard shortcut
      already triggers — no duplicated action logic between the button
      handler and `start_era`/`single_tick`/`toggle_notebook`. Prefer
      extracting the shared body into a plain function both the key-check
      system and the button click call, over two independent code paths
      that could drift.
- [ ] Buttons respect the same guards their keyboard equivalents already
      have (e.g. `single_tick`/`start_era` both no-op while
      `EraState::Advancing` — the button must too, ideally disabled/greyed
      out during that state rather than silently doing nothing on click).
- [ ] Keyboard shortcuts keep working unchanged — this is additive, not a
      replacement. No existing `input.rs`/`notebook.rs` key-handling test
      changes behavior.
- [ ] Buttons show their keyboard shortcut on hover (tooltip), matching
      `action_icon_row`'s existing pattern (if it already does this) or
      establishing it consistently across both rows if not — a player
      using the buttons should still discover the shortcut.
- [ ] `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test` clean.
- [ ] Verified live via `cargo run`: each new button produces the exact
      same effect as its keyboard shortcut, and pressing the shortcut still
      works normally alongside the buttons.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `hud_panel` (`~301+`), `action_icon_row` (`~742`, the existing pattern to mirror) — where the new buttons live. |
| `src/input.rs` | `start_era` (`~81`), `single_tick` (`~111`), `KeyCode::KeyR` reseed handler (`~165`) — the logic to expose as buttons, factored so both key-check and button-click call the same function. |
| `src/notebook.rs` | `toggle_notebook` (`~269`) — same factoring for the notebook toggle. |

---

## 🧩 Technical Context

**Current behavior**: `start_era`/`single_tick`/`toggle_notebook` each read
`Res<ButtonInput<KeyCode>>` directly and inline their own effect — there's
no separate "perform the action" function independent of "was the key
pressed," so a button click has nowhere to call into without either
duplicating the body or refactoring it out first.

`action_icon_row` (`ui.rs:742`) is the established reference pattern: a row
of egui buttons that set `SelectedAction`, already reading `SimConfig` for
per-action costs/tooltips. New buttons should follow its visual/layout
conventions (same row style, same panel section) rather than introducing a
new widget pattern.

---

## 🔨 Suggested Implementation

1. Extract each shortcut's effect into a plain function (e.g. `fn
   do_single_tick(...)`, `fn do_start_era(...)`) that both the existing
   `keys.just_pressed(...)` check and a new button's `.clicked()` check can
   call.
2. Add a small button row to `hud_panel`, near the existing era/tick label
   (`text::era_tick_line`) and state line, mirroring `action_icon_row`'s
   layout style.
3. Wire each button to its extracted function; add hover tooltips showing
   the keyboard shortcut.
4. `cargo test`/`clippy`/`fmt`, then a live `cargo run` check.

---

## ⚠️ Constraints and Caveats

- **Style**: presentation/input wiring in `ui.rs`/`input.rs`/`notebook.rs`,
  no `sim`/`world`/`config` changes needed — this task exposes existing
  logic through a second input path, it doesn't add new simulation
  behavior.
- Don't let button-driven and key-driven paths diverge in behavior (see
  acceptance criteria) — that's the concrete failure mode this task must
  avoid, not just a nice-to-have.

---

## 🔗 Dependencies

- **Depends on**: 030 (`action_icon_row`, the pattern this mirrors).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/094-hud-buttons-for-tick-era-notebook-controls.md)"$'\n\nExecute this task in the current project.'
```
