# Task 173 — Sidebar species/biosphere lists don't fill panel width

> **ID**: `173`
> **Category**: Bug fix
> **Priority**: 🟢 P3
> **Estimate**: ~20min
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

Playtest (`playtest_outcome.md`, issue I.3) reported the Seed Palette
species list capped at roughly half the sidebar's width. Confirmed in
`src/ui.rs:765-768`: the list's `egui::ScrollArea::vertical()` only enables
vertical scrolling (`direction_enabled = [false, true]`) and never calls
`.auto_shrink([false, true])`. egui's `ScrollArea` defaults `auto_shrink` to
`true` on *both* axes, and for a disabled, auto-shrinking horizontal axis it
sizes the area to its content's natural width instead of filling the panel
(`HUD_WIDTH`) — `species_row` (`src/ui.rs:1420-1455`, label + short subtext)
is narrower than the panel, so the list and its selectable-label highlight
visibly stop short.

The Biosphere list (`src/ui.rs:723-726`) has the identical missing call;
less noticeable there because its rows carry more text (name, population,
energy, trend, delta, death cause) and happen to come closer to the panel
width, but the underlying bug is the same.

Design source: `playtest_outcome.md` issue I.3.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] Both `ScrollArea`s (species list `src/ui.rs:765-768`, biosphere list
      `src/ui.rs:723-726`) call `.auto_shrink([false, true])` so they fill
      the sidebar's horizontal width while keeping vertical auto-shrink
      (height still governed by `max_height`/`SPECIES_VISIBLE_ROWS`).
- [ ] Manual check: selectable-label highlight and hover area now span the
      full sidebar width for both lists.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | Species list `ScrollArea` (765-768), Biosphere list `ScrollArea` (723-726). |

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/173-sidebar-list-width-fix.md)"$'\n\nExecute this task in the current project.'
```
