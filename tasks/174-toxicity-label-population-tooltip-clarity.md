# Task 174 — Align toxicity label thresholds to visual tint; clarify population tooltip scope

> **ID**: `174`
> **Category**: Bug fix / UX clarity
> **Priority**: 🟡 P2
> **Estimate**: ~45min
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

Two related legibility bugs from the playtest (`playtest_outcome.md`, issue
I.7 and gameplay note #11), both confirmed by reading the code:

1. **Toxicity label vs. visual tint mismatch.** `toxicity_tint`
   (`src/render.rs:1867-1873`) blends any `toxicity > 0` toward
   magenta/purple, clearly visible on Swamp's green base color well before
   the inspector's label changes. `text::band_label`
   (`src/text.rs:377-386`), called from `empty_cell_card`
   (`src/ui.rs:1087-1095`), divides `[0, swamp_toxicity_value=0.7]` into
   equal thirds for "low"/"moderate"/"high" — so a cell drifted to
   toxicity ~0.1-0.2 already reads as visibly tinted but is still labeled
   "low". The player also expected the dashed purple border to be a
   general high-toxicity indicator; it's actually
   `draw_survive_in_target`'s (`src/render.rs:629-698`) objective-target
   overlay for the active `SurviveIn` objective specifically, unrelated to
   a cell's own toxicity value — Crater (toxicity 0.60, correctly labeled
   "high") never gets it. That overlay-vs-toxicity conflation needs a UI
   fix (tooltip/legend clarifying what the border means), not a mechanics
   change.
2. **Population tooltip "N+M" is not per-cell.** `hover_tooltip`
   (`src/ui.rs:968-973`) prints `population.count` (this cell's
   individuals) next to `population_delta_for` (`src/ui.rs:1883-1896`),
   which is the species' **total population delta across the whole map**
   since the last era boundary (same number shown in the Biosphere sidebar
   row) — not this cell's own growth. Reads as one figure describing the
   cell; it describes two different scopes.

Design source: `playtest_outcome.md` issue I.7 and gameplay note #11.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] `band_label`'s toxicity thresholds (or `toxicity_tint`'s visible
      floor) are reconciled so "low" no longer covers a range that's
      already visibly tinted on the map — pick one direction (lower the
      "low" ceiling to match tint's visible floor, or gate the tint so it
      only starts fading in past the same threshold `band_label` uses) and
      apply consistently; no magic numbers, thresholds stay in
      `SimConfig`/existing constants.
- [ ] Inspector text or a legend clarifies that the dashed purple border is
      the active objective's target zone, not a general toxicity
      indicator — smallest change that removes the ambiguity (e.g. a short
      label near the border, or wording in the objective HUD panel).
- [ ] Population line in the hover tooltip / click card is re-scoped or
      re-labeled: either compute a per-cell delta (this cell's count vs.
      its own value last era, if tracked at that granularity) or relabel
      the existing world-wide delta clearly (e.g. "here: N · world Δ: +M")
      so the two numbers are never presented as describing the same scope.
- [ ] Manual check: hover a drifted-but-still-"low" swamp cell before/after,
      confirm label/tint agreement; hover a populated cell and confirm the
      tooltip's two numbers are unambiguous.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `toxicity_tint` (1867-1873), `draw_survive_in_target` (629-698). |
| `src/text.rs` | `band_label` (377-386). |
| `src/ui.rs` | `empty_cell_card` (1087-1095), `hover_tooltip` (962-974), `population_delta_for`/`update_population_trends` (1883-1935). |
| `src/config.rs` | `swamp_toxicity_value` (132, 161) — threshold source. |

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/174-toxicity-label-population-tooltip-clarity.md)"$'\n\nExecute this task in the current project.'
```
