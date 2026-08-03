# Task 021 — Hypothesis grid UI + tag/species catalog

> **ID**: `021`
> **Category**: UI / Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Close out the notebook's three panels (GDD §7, §11): the observation log exists (019), the confirmation engine exists (020) — this task adds the `tag × tag` hypothesis grid and the tag/species catalog, both inside the notebook window. This is the explicit "aha" — the visible progress bar of pillar 2 (GDD §7).

---

## 📋 Acceptance Criteria

- [ ] The notebook window (task 019) gains a second section/tab: a dense `active_tags × active_tags` table, row = exerting tag, column = receiving tag (matching `TagMatrix::get(exerter, receiver)`'s own convention).
- [ ] Each cell shows one of the states from GDD §5.9's "Notebook" table: `?` (no evidence at all yet), `0` (evidence exists but the cell is confirmed to have zero effect — only reachable if 020's engine records zero-weight confirmations for absence, otherwise this state may collapse into `?` until a real "confirmed no-effect" mechanic exists; if 020 didn't implement that, use just `?`/`±!` for this task and note the gap rather than inventing new confirmation semantics here), `±!` confirmed (shows the sign — not the magnitude, per "the player learns empirically", GDD §5.5).
- [ ] Confirmed cells read their sign from `world.matrix.get(exerter, receiver)` (only for cells `MatrixKnowledge::is_confirmed` returns true for — never leak unconfirmed values).
- [ ] Diagonal cells (`exerter == receiver`) are visually distinct or omitted — the matrix's diagonal is always `0` by construction (`world.rs` doc comment) and isn't a real hypothesis.
- [ ] A tag/species catalog panel: lists each `TagId` encountered so far (i.e. carried by at least one species the player has seen — in Phase 2 the whole active pool is visible from the seed selector already, so this may just be "all active tags" for now, with a note that per-encounter discovery is a Phase 3 worldgen concern once tags aren't fully known upfront) and, for each `SpeciesId`, its readable genome fields (metabolism, temp_optimum/tolerance) alongside its (still-opaque) tag glyphs.
- [ ] Tags are rendered as glyphs/colors, not raw numbers (GDD §11 "nameless glyphs/colors, learned empirically") — reuse whatever hue/symbol scheme `render.rs` already uses for species coloring if there's a natural tag-level equivalent, otherwise a simple deterministic color from `TagId` (e.g. golden-angle hue keyed on `tag.0`, same technique `render.rs` uses for species) is enough.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | Extends the window from 019 with the grid + catalog panels, reads `MatrixKnowledge` from 020 |
| `src/world.rs` | `TagMatrix::get`, `SimWorld::active_tags`, `SimWorld::species` — read-only |
| `src/render.rs` | Precedent for deterministic per-id coloring (golden-angle hues) — reuse the technique for tag glyphs if applicable |

---

## 🧩 Technical Context

GDD §11 calls the hypothesis grid out specifically as "the use case where immediate-mode UI (egui) is clearly better suited than a persistent-widget UI" — a dense interactive table redrawn every frame from `MatrixKnowledge` state is exactly egui's strength, no special caching needed. `egui::Grid` or a manual table via `egui::Frame`/nested `ui.horizontal()` rows both work; check what's idiomatic for the egui version pinned in this project (0.35, per `TECH_DESIGN.md` §12) before picking one.

---

## 🔨 Suggested Implementation

1. In `notebook.rs`, add a second section inside (or a tab within) the existing `egui::Window` from 019:
   ```rust
   fn hypothesis_grid(ui: &mut egui::Ui, world: &SimWorld, knowledge: &MatrixKnowledge) {
       egui::Grid::new("hypothesis_grid").show(ui, |ui| {
           ui.label(""); // corner cell
           for &receiver in &world.active_tags {
               ui.label(tag_glyph(receiver));
           }
           ui.end_row();
           for &exerter in &world.active_tags {
               ui.label(tag_glyph(exerter));
               for &receiver in &world.active_tags {
                   if exerter == receiver {
                       ui.weak("·");
                   } else if knowledge.is_confirmed(exerter, receiver, threshold) {
                       let sign = world.matrix.get(exerter, receiver).signum();
                       ui.label(if sign > 0 { "+!" } else { "-!" });
                   } else {
                       ui.weak("?");
                   }
               }
               ui.end_row();
           }
       });
   }
   ```
2. A `catalog_panel` listing `world.species` with their readable fields, and `world.active_tags` with their glyph/color.
3. Manual verification via the `run` skill: seed organisms with different tags near each other, advance several eras until an evidence total crosses 3.0 (use the `s` single-tick key for fine control), open the notebook (`tab`), confirm the corresponding grid cell flips from `?` to `±!` with the correct sign matching what the tick log/energy changes actually showed.

---

## ⚠️ Constraints and Caveats

- Never render the *unconfirmed* value of a matrix cell, even for debugging — that would defeat the entire deduction mechanic. If a debug overlay is useful during development, gate it behind a `#[cfg(debug_assertions)]` or config flag, and strip/hide it before calling the task done.
- Player-authored conjectures (`±?` state, GDD §5.9) are optional for this task per the plan — if cut for time, note it as a follow-up rather than silently dropping it from the spec.
- Keep this read-only with respect to `SimWorld`/`MatrixKnowledge` — no action economy here (that's Track B, 022+).

---

## 🔗 Dependencies

- **Depends on**: 020 (and transitively 019 for the window it renders inside)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/021-hypothesis-grid-ui-catalog.md)"$'\n\nExecute this task in the current project.'
```
