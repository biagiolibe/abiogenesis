# Task 056 — Player guide (manual + in-game "How to play" panel)

> **ID**: `056`
> **Category**: Docs / UI
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-07 (post-055 follow-up, requested directly by the user)

---

## 🎯 Objective

The game (MVP + onboarding, tasks 001-055) is functionally complete, but there is no player-facing explanation of what it is or how to play beyond in-game hints that only fire at specific moments (intro screen, viewport hints). Add a proper manual: a versioned Markdown document in the repo, and the same content (condensed) surfaced directly on the game's startup screen (the main menu) so a player sees it before starting a run.

---

## 📋 Acceptance Criteria

- [ ] `player_guide.md` exists at the repo root, in English, covering: what the game is (the "double mystery" pitch), controls, the core loop, actions & costs, the notebook/deduction system, objectives & difficulty progression, tips for a first run, and a short "active development / tuning" note.
- [ ] The guide reflects the *current* tuned behavior, not the original GDD baseline where they've since diverged: empty starting grid (task 050), eras — not ticks — shown in the HUD (task 049), total extinction retries the world not the run (task 051).
- [ ] `CLAUDE.md`'s Documents table has a row for `player_guide.md`.
- [ ] The main menu (`src/menu.rs::main_menu_ui`) has a "How to play" toggle button revealing a scrollable condensed version of the guide, without leaving the main menu (no new `GameState`).
- [ ] All new in-game copy lives in `src/text.rs`.
- [ ] The toggle doesn't obstruct or interfere with the seed field / "New run" button.
- [ ] `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt -- --check`, `cargo test` all clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `player_guide.md` | New. Full standalone manual. |
| `CLAUDE.md` | Add a row to the Documents table. |
| `src/text.rs` | New section: button label + section headings/body constants for the condensed in-menu guide. |
| `src/menu.rs` | `main_menu_ui`: add the toggle button + conditional `egui::ScrollArea`. |

---

## 🧩 Technical Context

- **Current behavior**: the main menu (`src/menu.rs`) is a centered panel with title, seed field, "New run" button, and an unlocks summary line. Nothing explains the game itself before the player commits to a run; `GameState::Intro` (task 052) gives a one-time, short framing paragraph, but it's a one-shot interstitial, not a reference manual.
- **Desired behavior**: same main-menu screen gains a "How to play" toggle; when open, a scrollable panel shows a condensed version of `player_guide.md`'s content, always accessible (not gated by any "seen" flag), never blocking "New run."
- The engine has no markdown renderer and no dependency on one — the project's established convention (task 034) is that all in-game copy lives in `text.rs` as Rust constants. The in-game panel therefore gets its own (shorter) copy of the guide's content, not a runtime render of the `.md` file.
- `notebook.rs`'s observation log already uses `egui::ScrollArea` inside a window — the same widget, reused here inside the main menu's `CentralPanel` instead.
- Toggle state: a plain `Local<bool>` inside `main_menu_ui` is enough — nothing else in the codebase needs to read whether the guide panel is open, unlike `NotebookWindowOpen` (which `ui.rs`'s badge/hint systems also read).

---

## 🔨 Suggested Implementation

1. Write `player_guide.md` at the repo root, English, in the same documentation register as `abiogenesis-gdd.md`/`TECH_DESIGN.md`.
2. Add a row for it to `CLAUDE.md`'s Documents table.
3. In `src/text.rs`, add a new section (e.g. `--- Main menu — How-to-play panel ---`) with a button label constant and heading/body constants for each section of the condensed guide.
4. In `src/menu.rs::main_menu_ui`, add a `mut show_guide: Local<bool>` parameter, a toggle button, and — when true — an `egui::ScrollArea` rendering the new `text.rs` constants, placed so it doesn't cover the seed field or "New run" button (e.g. below them, or in a separate collapsible section).
5. `cargo build && cargo clippy -- -D warnings && cargo fmt && cargo test`.
6. Playtest via `cargo run`: launch to the main menu, toggle the panel open/closed, confirm layout and scrolling.

---

## ⚠️ Constraints and Caveats

- **Style**: no magic numbers beyond what egui requires; all in-game copy through `text.rs`; `player_guide.md` and `CLAUDE.md` in English per the project's document convention regardless of chat language.
- **No new `GameState`**: this is informational, always-available content on the existing main menu, not a new screen in the state machine.
- **Two artifacts, not a literal copy-paste**: `player_guide.md` (full manual) and the in-game panel's `text.rs` copy (condensed) cover the same ground but aren't required to be verbatim identical — same relationship the GDD already has to `text.rs`'s existing runtime strings.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/056-player-guide.md)"$'\n\nExecute this task in the current project.'
```
