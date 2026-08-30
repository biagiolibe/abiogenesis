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

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] **Gated `toxicity_tint`'s visible floor** to `band_label`'s own "low"
      ceiling (`swamp_toxicity_value / 3`, re-derived from the existing
      config value, no new magic number): `toxicity_tint` now takes an
      explicit `visible_floor` and re-normalizes onto `[floor, 1.0] ->
      [0.0, 1.0]` instead of blending from zero, so nothing still labeled
      "low" tints at all.
- [x] **Objective panel clarifies the dashed border**: added
      `text::SURVIVE_IN_TOXIC_BORDER_HINT`, shown under the narrative line
      only for `Objective::SurviveIn { zone: ZoneKind::Toxic, .. }` — the
      one case `draw_survive_in_target` actually draws for.
- [x] **Population tooltip re-labeled** rather than computing a per-cell
      delta (not tracked at that granularity): `text::hover_population_line`
      renders "Here: N" or "Here: N · World Δ: ±M", replacing the old
      unlabeled "Population N ±M" line.
- [-] Manual check — skipped per explicit user instruction for this task;
      `cargo build`/`clippy`/`fmt`/`test` all clean.

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
