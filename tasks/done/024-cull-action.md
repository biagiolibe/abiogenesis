# Task 024 — Cull action

> **ID**: `024`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Add the third player action (GDD §6): click an occupied cell to remove the organism there, costing 1 action point. The simplest of the three new actions — mostly wiring, since the `ActionMode` selector and click-resolution helper already exist from task 023.

---

## 📋 Acceptance Criteria

- [ ] Left-click while `ActionMode::Cull` is selected (and `EraState::Observing`, budget allows it) removes the organism at the clicked cell, if any. Clicking an empty cell costs nothing and does nothing (mirrors `Seed`'s existing empty/occupied asymmetry, just inverted).
- [ ] Costs `config.time.action_costs.cull` (already `1`).
- [ ] Removing an organism this way does **not** deposit residue (GDD §5.6 step 6 ties residue to *death* by the tick algorithm, i.e. energy `<= 0`; a player-culled organism is removed by fiat, not by starving/predation — document this distinction in a code comment so it doesn't read as a bug later). If this reading turns out wrong during implementation (e.g. it feels better for decomposers to be able to scavenge a culled organism too), flag it as a design question rather than silently picking one — check with the user before deviating from the GDD-literal reading.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | New `cull_on_click` system, reusing the `clicked_cell` helper factored out in task 023 |
| `src/ui.rs` | `ActionMode::Cull` already exists as an enum variant from 023 — this task just makes it functional |

---

## 🧩 Technical Context

No new patterns beyond what 022/023 established — this task is the simplest confirmation that the `ActionMode` + budget scaffolding generalizes cleanly. If it doesn't (e.g. the click-resolution helper needs a third tweak), that's a signal 023's factoring wasn't quite right and worth a small follow-up rather than duplicating logic a third time.

---

## 🔨 Suggested Implementation

1. `input.rs`:
   ```rust
   fn cull_on_click(
       buttons: Res<ButtonInput<MouseButton>>,
       windows: Query<&Window>,
       cameras: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
       era_state: Res<State<EraState>>,
       selected_action: Res<SelectedAction>,
       mut budget: ResMut<ActionBudget>,
       mut world: ResMut<SimWorld>,
       config: Res<SimConfig>,
   ) {
       if selected_action.0 != ActionMode::Cull { return; }
       if *era_state.get() == EraState::Advancing { return; }
       let Some((x, y)) = clicked_cell(&buttons, &windows, &cameras) else { return; };
       let cell = world.get_mut(x, y);
       if cell.organism.is_none() { return; }
       if !budget.try_spend(config.time.action_costs.cull) { return; }
       cell.organism = None;
   }
   ```
2. Register the system in `InputPlugin`.
3. Manual verification: switch to Cull mode, click an organism, confirm it disappears and no residue appears in its cell; confirm budget decrements; confirm clicking empty cells is a no-op that doesn't spend budget.

---

## ⚠️ Constraints and Caveats

- Keep this to single-cell removal — "eliminate an organism or a species in an area" (GDD §6) leaves an area/species-wide variant open, but a single click matches Stress's Phase 2 baseline (023) and keeps the three actions consistent in interaction model.
- If GDD's "or a species" reading is wanted later (e.g. a modifier key to cull all instances of the clicked cell's species), treat that as a follow-up enhancement, not part of this task's acceptance criteria.

---

## 🔗 Dependencies

- **Depends on**: 022, 023
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/024-cull-action.md)"$'\n\nExecute this task in the current project.'
```
