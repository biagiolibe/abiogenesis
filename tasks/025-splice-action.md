# Task 025 — Splice action

> **ID**: `025`
> **Category**: Feature / UI
> **Priority**: 🟡 P2
> **Estimate**: ~2h (split into two tasks if it runs over — see Constraints)
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Add the fourth and last player action (GDD §6): modify a species' genome — add/change a tag, or shift its thermal optimum. Explicitly "the most powerful and most expensive experimental tool" (GDD §6), costing 2 action points. This is the most complex of the four actions: unlike Seed/Stress/Cull, its target is a *species definition*, not a single cell, so it needs a small editor UI rather than a plain click.

---

## 📋 Acceptance Criteria

- [ ] `ActionMode::Splice` (already an enum variant from task 023) opens a small egui popup/panel (not a full window necessarily — a collapsible section in the HUD, or a modal, whichever fits the existing HUD's layout better) letting the player pick: a target species (from `world.species`), and one edit — either swap/add one tag from `world.active_tags`, or shift `temp_optimum` by a fixed step.
- [ ] Applying a splice creates a **new species** (append to `world.species`, don't mutate an existing `Species` in place) with the edited field(s) copied from the source species — mutating in place would retroactively change every already-alive organism of that species, which isn't what "modify a species' genome" implies at the individual level; a splice should read as "a variant is introduced," consistent with how reproduction/genome changes are framed elsewhere in the GDD (§5.6 step 7 mentions "possible mutation of the child's genome" as a *future* idea, implying species-level identity is otherwise stable). Document this choice in a comment since it's a real design call, not an obvious mechanical detail.
- [ ] After a splice, the new species becomes selectable from the existing `Seed` species selector (`ui.rs`, task 017) — otherwise there'd be no way to actually place the spliced species on the grid.
- [ ] Costs `config.time.action_costs.splice` (already `2`), same budget pattern as the other three actions — spent when the splice is confirmed/applied, not while the editor is merely open.
- [ ] The tag-pool and thermal-shift step sizes are named `SimConfig` constants (no magic numbers) — e.g. `EnergyConfig::splice_temp_shift` or similar, at implementer's discretion.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | Splice editor UI, extends the existing species selector to include newly spliced species |
| `src/input.rs` or `src/ui.rs` | Where the "apply splice" action fires — likely a button in the editor panel rather than a grid click, unlike the other three actions |
| `src/world.rs` | `Species`, `SpeciesId`, `TagId` — read for building the editor's options, `world.species.push(...)` to apply |
| `src/config.rs` | New splice-step constants |

---

## 🧩 Technical Context

Splice doesn't fit the "click a grid cell" interaction model the other three actions share (`clicked_cell` helper from 023) — it's closer to a form: pick a species, pick an edit, confirm. This is the one action in Track B that's mostly UI-state-machine work (which species is being edited, what edit is staged) rather than a one-shot click handler, which is why it's flagged as the task most likely to need splitting.

---

## 🔨 Suggested Implementation

1. `ui.rs`: a small resource tracking in-progress splice state, e.g.:
   ```rust
   #[derive(Resource, Default)]
   pub struct SpliceDraft {
       pub source: Option<SpeciesId>,
       pub edit: Option<SpliceEdit>,
   }

   pub enum SpliceEdit {
       SwapTag { old: TagId, new: TagId },
       ShiftTempOptimum { delta: f32 },
   }
   ```
2. When `ActionMode::Splice` is selected, render a panel: dropdown/radio for `source` species, radio for edit kind, then an "Apply" button.
3. "Apply" button handler: checks budget, builds the new `Species` from `world.species[source.0].clone()` with the edit applied, `world.species.push(new_species)`, spends the budget, clears `SpliceDraft`.
4. Extend the existing seed species radio list (`hud_panel`) to include the newly pushed species automatically (it already iterates `0..world.species.len()`, so this likely needs no change — verify).
5. Manual verification: splice a new species with a shifted `temp_optimum`, confirm it appears in the seed selector, seed one, confirm (via HUD population stats or notebook catalog from 021) that it behaves per its new optimum over a few eras.

---

## ⚠️ Constraints and Caveats

- **Split this task if it runs past ~2h**: a clean split is "021a — Splice UI (draft state, panel, species/edit selection, no application)" and "021b — Splice application logic (budget spend, new-species creation, wiring into the seed selector)". Note the split in `tasks/QUEUE.md` and `PROJECT_PLAN.md` if taken.
- New species from splicing don't need tags drawn via `draw_species_tags` (`world.rs`) — that function is for *procedural* generation at world-seed time; a splice is a deliberate, player-directed edit of one specific field, not a re-roll.
- Don't let splice edit `world.matrix` — the hidden matrix stays fixed for the run; splice only changes what tags a species *carries*, not what the matrix says those tags do. This is what keeps the deduction game coherent: the rules don't move, only which rules apply to which organisms.

---

## 🔗 Dependencies

- **Depends on**: 022
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/025-splice-action.md)"$'\n\nExecute this task in the current project.'
```
