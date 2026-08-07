# Task 053 — In-viewport contextual hints for the first actions

> **ID**: `053`
> **Category**: UI / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-07 (first-minutes engagement design session)

---

## 🎯 Objective

Since task 050, a new world starts with an empty grid — the player must select a species in the HUD's Seed palette and click an empty cell to place it. Nothing in the game currently says so outside of a hover tooltip (`text::SEED_PALETTE_HOVER`, only visible if the player happens to hover the right label) and `KEYBOARD_HINT`. A fresh player faces a silent grid and an HUD full of unexplained numbers.

Add lightweight, self-dismissing hints drawn **over the simulation viewport** (not buried in HUD tooltips) that guide the player through their first two key actions: placing the first organism, and opening the notebook for the first time.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors (`cargo build`).
- [ ] While `GameState::Playing` and no cell has ever been placed by the player (`PlayerPlacedCells` empty, `src/notebook.rs`), an in-viewport hint is shown telling the player to pick a species in the palette and click an empty grid cell.
- [ ] The hint disappears automatically the moment the player places their first organism — no manual dismiss button needed.
- [ ] After the first placement, if the notebook window has never been opened, a second hint appears telling the player to open the notebook to log hypotheses. It disappears automatically the first time the notebook window is opened.
- [ ] Hints are visually distinct from the always-present `KEYBOARD_HINT` (e.g. a soft panel over the grid, or an attention-grabbing but non-blocking overlay) — they must not block clicks on the grid underneath, or must be positioned so they don't overlap the area the player needs to click.
- [ ] All new copy lives in `src/text.rs`.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` are clean.
- [ ] `cargo test` passes; if a new resource tracks "notebook ever opened", cover any pure logic with a unit test if applicable.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | Add a new system (in `EguiPrimaryContextPass`, gated on `GameState::Playing`) drawing the viewport-anchored hint. |
| `src/notebook.rs` | `PlayerPlacedCells` (already exists, task 050) gives the "no placement yet" condition. Add a small resource/flag for "notebook window ever opened" if one doesn't already exist — check the notebook window's toggle system first (`tab` key, task 019) for a natural place to set it. |
| `src/text.rs` | New hint strings, in a section near the existing `SEED_PALETTE_HOVER`/`KEYBOARD_HINT` HUD strings. |

---

## 🧩 Technical Context

- **Current behavior**: `SEED_PALETTE_HOVER` ("Click an empty cell to place the selected species", `text.rs:124`) only shows on hover of the Seed palette heading in the HUD (`src/ui.rs:289-290`) — easy to miss entirely. No hint exists at all about opening the notebook.
- **Desired behavior**: the moment the player is in a state where the "next obvious action" is unclear (empty grid, unopened notebook), the game says so directly in the space the player is already looking at (the viewport), and stops saying so once the action is taken.
- `PlayerPlacedCells` is already introduced by task 050 to guard `is_total_extinction` against firing before the player has seeded anything (`src/objectives.rs:187-189`) — reuse it as-is, don't duplicate the tracking.
- Check `src/notebook.rs` for how the notebook window's visibility/toggle is currently implemented (likely an `egui::Window` shown/hidden by a boolean resource toggled on the `tab` key, per task 019) — hook into that toggle to flip an "ever opened" flag the first time it goes from closed to open.

---

## 🔨 Suggested Implementation

1. In `src/notebook.rs` (or wherever the notebook window toggle lives), add a resource (e.g. `NotebookEverOpened(bool)`, default `false`) and set it `true` the first time the window is shown.
2. In `src/text.rs`, add two hint strings: one for "place your first organism", one for "open the notebook".
3. In `src/ui.rs`, add a new system that:
   - runs only in `GameState::Playing`;
   - reads `PlayerPlacedCells` and `NotebookEverOpened`;
   - if `PlayerPlacedCells` is empty, draws the first hint over the viewport (e.g. a small `egui::Area`/`Window` anchored top-center or bottom-center of the grid area, not the HUD strip);
   - else if `!NotebookEverOpened`, draws the second hint;
   - else draws nothing.
4. Keep the overlay non-interactive (no buttons) so it never intercepts grid clicks — position it where it can't overlap the grid's clickable area (e.g. anchored above the grid, within the viewport but outside the HUD).
5. Playtest via `cargo run`: verify the first hint shows on a fresh world, disappears after the first Seed click; verify the second hint appears next and disappears after pressing `tab`/opening the notebook.

---

## ⚠️ Constraints and Caveats

- **Style**: no magic numbers for any layout constants beyond what egui itself requires; all copy through `text.rs`; UI-only change, `sim`/`world`/`config` untouched (`TECH_DESIGN.md` §5).
- **Non-blocking**: hints must never intercept mouse input meant for the grid.
- **No dismiss button**: hints are purely state-driven, not user-dismissible — keep it simple, don't add a close affordance.

---

## 🔗 Dependencies

- **Depends on**: none (reads existing `PlayerPlacedCells` from task 050)
- **Blocks**: none (independent of tasks 052, 054 from the same design session)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/053-in-viewport-contextual-hints.md)"$'\n\nExecute this task in the current project.'
```
