# Task 023 — Stress action

> **ID**: `023`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Add the second player action (GDD §6): click a cell to alter one of its environmental scalars (toxicity or temperature) by a fixed delta, costing 1 action point. This task also introduces the "action mode" selector in the HUD that 024 (Cull) and 025 (Splice) will reuse, since up to now there's only ever been one thing a click could do (`Seed`).

---

## 📋 Acceptance Criteria

- [ ] An `ActionMode` enum (`Seed`, `Stress`, `Cull`, `Splice` — the last two are stubs this task doesn't implement yet, but the enum should include them now so 024/025 don't need to touch the selector again) and a `SelectedAction(pub ActionMode)` resource, likely alongside `SelectedSpecies` in `ui.rs`.
- [ ] HUD gains a mode selector (radio buttons, same widget style as the existing species selector) — only shown/enabled options matter for now; `Cull`/`Splice` can appear disabled or simply do nothing yet, whichever is less code, but must not panic.
- [ ] Left-click while `ActionMode::Stress` is selected (and `EraState::Observing`, budget allows it) modifies the clicked cell's toxicity **or** temperature (pick one as the Phase 2 baseline — GDD §6 gives both as examples without mandating which; toxicity is the more legible "hostile zone" lever given the existing toxic-zone mechanic, so prefer it unless there's a reason to do both) by a fixed delta, clamped to `[0, 1]` (the existing scalar range invariant, `TECH_DESIGN.md`/`world.rs`).
- [ ] The stress delta is a new named constant in `SimConfig` (no magic numbers, project convention) — likely `EnvironmentConfig::stress_delta` or similar.
- [ ] Costs `config.time.action_costs.stress` (already `1` in `SimConfig::default()`), same budget-check-then-decrement pattern task 022 established for `Seed`.
- [ ] Clicking an empty vs. occupied cell both work for Stress (it targets the environment, not the organism) — unlike `Seed`, occupancy isn't a precondition.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | New `ActionMode`/`SelectedAction` resource + HUD radio selector |
| `src/input.rs` | New `stress_on_click` system, or `seed_organism_on_click` generalized to dispatch on `SelectedAction` — pick whichever keeps each action's logic legible; a shared "resolve clicked cell" helper factored out either way |
| `src/config.rs` | New `stress_delta` constant (name/placement at implementer's discretion, consistent with existing `EnvironmentConfig` fields) |

---

## 🧩 Technical Context

`input.rs::seed_organism_on_click` already has the full click→cell resolution pipeline (window cursor → camera → world position → grid cell, with all the bounds/viewport edge cases task 017 worked out). This task's click handling should reuse that resolution logic rather than re-deriving it — either by factoring a small `resolve_clicked_cell(...) -> Option<(usize, usize)>` helper out of the existing system, or by keeping per-action systems that each call a shared helper function.

---

## 🔨 Suggested Implementation

1. `ui.rs`:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum ActionMode { Seed, Stress, Cull, Splice }

   #[derive(Resource)]
   pub struct SelectedAction(pub ActionMode);
   ```
   Default to `ActionMode::Seed` (preserves today's behavior for players who never touch the new selector). Add radio buttons in `hud_panel` next to the species selector.
2. Factor `input.rs`'s cursor→cell resolution out of `seed_organism_on_click` into a helper, e.g. `fn clicked_cell(windows, cameras, buttons) -> Option<(usize, usize)>`, reusable by the new `stress_on_click` system.
3. New system `stress_on_click`, gated the same way as `seed_organism_on_click` (`Observing` only, budget check), applying the delta and clamping.
4. `config.rs`: add the stress delta constant with a doc comment citing GDD §6.
5. Manual verification: switch to Stress mode, click inside vs. outside the existing toxic zone, confirm the scalar visibly shifts (toxic zone rendering, if any, or just re-open the notebook/HUD stats after an era to see downstream effects on organisms placed nearby); confirm budget decrements and blocks at 0 same as Seed.

---

## ⚠️ Constraints and Caveats

- Don't implement Cull or Splice logic in this task — only the shared `ActionMode` enum/selector scaffolding they'll plug into.
- Keep the stress delta a single fixed value per click (no drag-to-paint-an-area mechanic) — GDD §6 says "alter an environmental scalar in an area" but doesn't mandate a multi-cell brush; a single-cell click is the simpler Phase 2 baseline and can grow into an area tool later if playtesting wants it (that's a Final Tuning concern, not this task).

---

## 🔗 Dependencies

- **Depends on**: 022
- **Blocks**: 024 (shares the mode selector)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/023-stress-action.md)"$'\n\nExecute this task in the current project.'
```
