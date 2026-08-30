# Task 183 — Pause menu + confirmation dialog: chrome and state-colored actions

> **ID**: `183`
> **Category**: UI / Bugfix
> **Priority**: 🟡 P2 (corrective — Phase 2 residual)
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-29 scoping

---

## 🎯 Objective

`pause_menu` and `confirmation_dialog` (`src/ui.rs`) are both modal
`egui::Window`s that have never been restyled — the pause menu's own doc
comment (`ui.rs:1144-1146`) explicitly chose to keep egui's default window
chrome rather than a custom frame. Fix both, and give the confirmation
dialog's Confirm/Cancel buttons the state-color distinction the design
already implies (a destructive vs. safe binary choice) but never applied.

Read `VISUAL_STYLE_GUIDE.md` first — §3 (color tokens), §6 (chrome/button
registers). This task reuses the `STATE_POSITIVE`/`STATE_NEGATIVE`/
`PANEL_BG`/`OUTLINE_STROKE` constants task 182 introduces — check they
exist before adding a second copy.

---

## 📋 Acceptance Criteria

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] **Window chrome**: `pause_menu`'s and `confirmation_dialog`'s
      `egui::Window`s both get an explicit `Frame` (`egui::Frame::window`)
      with `PANEL_BG` fill and `OUTLINE_STROKE` border instead of egui's
      default window fill/stroke, plus `apply_monospace` on their `Ui`.
- [x] **Pause menu buttons** (Resume/Settings/Save-and-exit) become
      outline-chrome buttons via `outline_button_auto`. Settings keeps its
      disabled/hint behavior (`on_hover_text`, since it's a custom-painted
      control, not `on_disabled_hover_text`). "Abandon" uses the new
      `outline_button_auto_colored` helper with `STATE_NEGATIVE` for both
      box stroke and label — chrome, not just text color, reads as
      destructive.
- [x] **Confirmation dialog's Confirm/Cancel get state-colored chrome**:
      Confirm uses `outline_button_auto_colored(..., STATE_NEGATIVE, ...)`
      (both existing `ConfirmationKind`s are destructive/irreversible),
      Cancel uses plain `outline_button_auto` (`OUTLINE_STROKE`). A comment
      at the call site flags that a future non-destructive kind should
      switch Confirm's color per-kind rather than hardcoding red.
- [-] Live visual check — skipped per explicit user instruction for this
      task; `cargo build`/`clippy`/`fmt`/`test` all clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `pause_menu` (`1152-1187`), `confirmation_dialog` (`1195-1223`), `ALERT_COLOR` (`1140`, becomes `STATE_NEGATIVE` per task 182). |

---

## 🧩 Technical Context

- **Current behavior**: both dialogs use egui's default `Window` chrome
  (gray fill, default stroke, default corner radius zeroed only by the
  global 151 pass); all buttons are plain `ui.button(...)`; Confirm/Cancel
  are visually identical regardless of what they confirm.
- **Desired behavior**: dark panel chrome consistent with the rest of the
  game, outline-box buttons, and a clear color-coded distinction between
  the destructive Confirm action and the safe Cancel action.

---

## 🔨 Suggested Implementation

1. Confirm task 182's constants exist; if not yet landed, add the same
   `PANEL_BG`/`STATE_NEGATIVE`/`OUTLINE_STROKE` constants locally and note
   the duplication for a follow-up merge.
2. Wrap both `Window::new(...)` calls with an explicit `.frame(...)`
   override (fill + stroke) instead of relying on defaults.
3. Apply the monospace-family override to each window's `Ui`.
4. Swap `ui.button` calls for the outline-button helper (182/180's, or a
   local one if landing first).
5. Confirmation dialog: branch button chrome by outcome polarity (Confirm
   = negative/destructive styling, Cancel = neutral outline) rather than by
   `ConfirmationKind` variant — both current kinds happen to be
   destructive, but the chrome rule should be about the button's role, not
   an enum match that'll need updating per new kind.
6. Live-check both dialogs.

---

## ⚠️ Constraints and Caveats

- Both `ConfirmationKind` variants are currently equally destructive — if a
  future non-destructive confirmation kind is added, its Confirm button
  should probably use `STATE_POSITIVE` instead; this task only needs to get
  today's two kinds right, but structure the fix so that's an easy future
  branch, not a hardcoded assumption that Confirm is always red.
- Don't touch `Esc`-cascade logic or any confirmation *behavior* — this is
  presentation-only, same boundary every other pixel-grain task in this
  queue has kept.
- Keep `sim`/`world`/`config` untouched.

---

## 🔗 Dependencies

- **Depends on**: 182 (reuses its shared state-color/panel constants and
  outline-button helper — land 182 first where practical, though not a
  hard blocker per 182's own dependency note).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/183-pause-menu-confirmation-chrome.md)"$'\n\nExecute this task in the current project.'
```
