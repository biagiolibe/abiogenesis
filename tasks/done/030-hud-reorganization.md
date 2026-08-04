# Task 030 — HUD reorganization: grouping, icons, tooltips, bars

> **ID**: `030`
> **Category**: UX
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-03 playtest

---

## 🎯 Objective

A 2026-08-03 playtest found the whole UI reads as "computer/lab-based" rather than "player/game-based" — `hud_panel` (`ui.rs`) today is a flat, undifferentiated stack of labels and radio buttons (world state, population stats, action selector, species selector, Splice editor, action budget all run together with only `ui.separator()` between them). This task restructures the *existing* HUD content — no new information, no new mechanics — into something more legible and less "debug panel": clear visual grouping, unicode-symbol icons for the four actions instead of text radio labels, an `egui::ProgressBar` for the action budget instead of a raw `"Actions: 2 / 3"` string, and tooltips for anything that currently relies on dense inline text.

This is a **presentation-layer restructuring**, not a redesign of what information exists or how the simulation works — scope it tightly to `ui.rs`.

---

## 📋 Acceptance Criteria

- [x] The HUD groups its content into clearly visually-separated zones, each internally coherent: **World state** (era/tick/seed/state), **Action** (mode selector + budget + the active action's editor, e.g. Splice's panel), **Population** (per-species stats), **Seed palette** (species selector) — exact grouping/order at implementer's discretion, but "everything in one flat list" is what must change.
- [x] The four `ActionMode` options render as compact icon buttons (unicode symbols, e.g. a seed/sprout glyph, a lightning/heat glyph, a skull/X glyph, a DNA/splice glyph — pick whichever renders reliably in egui's default font; no new asset/font loading pipeline) instead of a vertical list of `ui.radio_value(..., "Seed")`-style text rows. Keep `ui.radio_value`'s semantics (single selection, immediate-mode) — this is a visual change, not a new selection widget.
- [x] The action budget (`"Actions: N / 3"`) renders via `egui::ProgressBar` (or equivalent visual bar), with the numeric readout kept as the bar's text/tooltip, not removed — a player should still be able to see the exact number, just not have it be the *only* representation.
- [x] Add `egui::Response::on_hover_text` (or equivalent) tooltips at minimum to: each action icon (what it does, its cost), the keyboard-shortcut hint at the bottom (already present as inline text — a tooltip isn't required to replace it, but check it isn't redundant with what tooltips now cover).
- [x] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `hud_panel`, `splice_panel` — the entire scope of this task |

---

## 🧩 Technical Context

Nothing here touches `SimWorld`, `ActionBudget`, or any resource's data shape — `hud_panel` already reads everything it needs (`Res<ActionBudget>`, `Res<SimWorld>`, etc.); this task only changes how that data is laid out and rendered. Check `egui::Ui::horizontal`/`egui::Frame`/`egui::Grid` for grouping primitives already used elsewhere in the codebase (`notebook.rs`'s `hypothesis_grid` uses `egui::Grid`) before introducing a new layout pattern.

---

## 🔨 Suggested Implementation

1. Wrap each logical group (world state, action, population, seed palette) in an `egui::Frame` or at minimum a heading + consistent spacing, replacing the current flat `ui.separator()`-delimited list.
2. Replace the `ActionMode` radio list with a `ui.horizontal(|ui| { ... })` row of icon buttons — `ui.selectable_label` or a custom small button per action, each wrapped in `.on_hover_text(...)` describing the action and its cost.
3. Replace `ui.label(format!("Actions: {} / {}", ...))` with `ui.add(egui::ProgressBar::new(fraction).text(format!("{}/{}", budget.points_remaining, total)))`.
4. Manual verification via the `run` skill: confirm the HUD reads as visually organized (not a wall of text), icons are distinguishable and tooltip-labeled, the budget bar reflects budget changes correctly across seeds/eras/reseeds.

---

## ⚠️ Constraints and Caveats

- No new information, no new player actions, no changes to `SimWorld`/`ActionBudget`/etc. — purely how existing data renders.
- Don't introduce a custom icon font or image assets — stick to unicode glyphs egui's default font already renders, consistent with `notebook.rs`'s existing `TAG_GLYPH` (`"●"`) approach.
- Keep `Ui` read-only against simulation state (TECH_DESIGN.md §3.3) — this task only touches rendering and existing intent-writes (`SelectedAction`, etc.), not new mutations.

---

## 🔗 Dependencies

- **Depends on**: 022 (action budget), 023 (action mode selector), 025 (Splice panel)
- **Blocks**: none (independent of 031)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/030-hud-reorganization.md)"$'\n\nExecute this task in the current project.'
```
