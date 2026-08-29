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

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] **Window chrome**: `pause_menu`'s `egui::Window` (`ui.rs:1161`) and
      `confirmation_dialog`'s `egui::Window` (`ui.rs:1207`) both get an
      explicit `Frame` with `PANEL_BG` fill (task 182's constant) and
      `OUTLINE_STROKE` border instead of egui's default window
      fill/stroke, plus monospace font applied to their `Ui` (same
      technique as `hud_panel`, `ui.rs:666-668`).
- [ ] **Pause menu buttons** (`PAUSE_RESUME_BUTTON` `1167`,
      `PAUSE_SETTINGS_BUTTON` `1170`, `PAUSE_SAVE_AND_EXIT_BUTTON` `1173`)
      become outline-chrome buttons (task 182's/180's shared helper — reuse,
      don't reinvent). The "Abandon" action (`1177-1183`) already uses
      `ALERT_COLOR` for its text; once task 182 lands, this becomes
      `STATE_NEGATIVE` automatically (constant rename, verify the call
      site still reads correctly) — additionally give it an
      `STATE_NEGATIVE`-stroked outline box, not just colored text, so the
      destructive action reads as such from its chrome, not only its
      label color.
- [ ] **Confirmation dialog's Confirm/Cancel get state-colored chrome**:
      today (`ui.rs:1214,1217`) both are plain `ui.button(...)` with no
      color distinction regardless of `ConfirmationKind`
      (`ReseedWorld`/`AbandonRun`, both currently identical). Give Confirm
      an outline box in `STATE_NEGATIVE` (both existing kinds are
      destructive/irreversible — reseeding discards world state, abandon
      discards a run) and Cancel a plain `OUTLINE_STROKE` box — establishes
      the pattern for any future `ConfirmationKind` too, not just today's
      two.
- [ ] Live visual check (`cargo run`, screenshot or interactive): open the
      pause menu (Esc) and a confirmation dialog (reseed or abandon),
      confirm dark chrome, outline buttons, and the Confirm action reading
      visually distinct (red-toned) from Cancel.

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
