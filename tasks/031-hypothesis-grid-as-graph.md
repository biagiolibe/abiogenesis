# Task 031 — Hypothesis grid as a graph, not a spreadsheet table

> **ID**: `031`
> **Category**: UX
> **Priority**: 🟡 P2
> **Estimate**: ~2.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-03 playtest

---

## 🎯 Objective

A 2026-08-03 playtest called the current hypothesis grid (task 021, `notebook.rs::hypothesis_grid`) too "lab/spreadsheet-like": an `active_tags × active_tags` `egui::Grid` of `?`/`+!`/`-!` text cells. This task replaces that table with a **network diagram**: tags as nodes arranged in a circle, confirmed relationships as directed edges between them (colored by sign, e.g. green for positive/red for negative), unconfirmed pairs simply having no edge at all. The goal is the same information, read at a glance instead of scanned cell by cell.

**Semantics must not change** — this is a rendering swap for `hypothesis_grid`'s output, reading the exact same `MatrixKnowledge`/`world.active_tags` data it does today. No new confirmation logic, no new evidence semantics.

---

## 📋 Acceptance Criteria

- [ ] `world.active_tags` render as nodes arranged evenly around a circle (radius/center computed from the available `egui::Ui` space), each using the existing `tag_color`/tag-glyph treatment (task 029, if landed first — otherwise the current `TAG_GLYPH` dot) so this stays visually consistent with the catalog panel.
- [ ] For every `(exerter, receiver)` pair `MatrixKnowledge::revealed_value` returns `Some` for, draw a directed edge from the exerter node to the receiver node, colored by sign (e.g. a warm color for positive, a cool color for negative — reuse whatever convention, if any, existing UI already implies; otherwise pick and document one). An arrowhead or other directional marker distinguishes `A → B` from `B → A` when both exist.
- [ ] Unconfirmed pairs draw **no edge** — don't invent a "maybe" edge style; this must not leak information beyond what `is_confirmed` already reveals (same constraint task 021's original grid honored).
- [ ] The diagonal (`exerter == receiver`, always 0 by construction) is never drawn as a self-loop or otherwise implied — there was never a real hypothesis there, same as the old grid's `·`.
- [ ] Hovering a node shows the tag's identity (color/glyph) and, optionally, a list of its confirmed relationships as text (a fallback for players who prefer reading a list — don't remove all textual access to the same information, just make the primary view graphical).
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `hypothesis_grid` — replaced internals; `notebook_window`'s call site should need no signature change |

---

## 🧩 Technical Context

`egui::Painter` (via `ui.painter()` or `ui.allocate_painter(size, sense)`) provides `circle_filled`, `line_segment`, and `arrow` — everything needed for a basic node-and-edge diagram without pulling in a graph-layout crate or new dependency. A circular layout for `n` nodes is simple trigonometry (`angle = i as f32 / n as f32 * TAU`, position via `center + radius * Vec2::angled(angle)`) — no force-directed layout needed given `active_tags_early` is small (5 by default, up to ~8 per Phase 3's difficulty curve plan).

---

## 🔨 Suggested Implementation

1. Allocate a fixed-size painting region in `hypothesis_grid` (`ui.allocate_painter` or `ui.allocate_space` + `ui.painter_at`), sized to fit comfortably in the Notebook window.
2. Compute node positions: evenly spaced around a circle, radius derived from the allocated space.
3. Draw all nodes first (colored circles + glyph/label), then iterate all `(exerter, receiver)` pairs with `exerter != receiver`, drawing an edge only where `revealed_value` is `Some`.
4. Add hover detection per node (`egui::Response::hovered()` on a per-node interactive area, or a manual distance check against pointer position) showing a tooltip with the tag's confirmed relationships as a short text fallback.
5. Manual verification via the `run` skill: confirm nodes/edges render correctly against a world with a mix of confirmed and unconfirmed pairs (may need to play a few eras, or temporarily lower `confirmation_threshold` in a local config tweak to speed up manual testing, reverted before considering the task done), confirm no edge appears where none is confirmed, confirm arrow direction reads correctly for an asymmetric pair.

---

## ⚠️ Constraints and Caveats

- No new crate dependencies — `egui::Painter` covers everything needed at this scale.
- Don't reveal anything the old grid didn't: no edge, weak/dashed or otherwise, for unconfirmed pairs. If a "some evidence exists" indicator (task 028) also lands, keep that as a *visual weight/opacity* on nodes or a separate small marker, not as a partial edge that could be mistaken for a confirmed relationship.
- If `active_tags_early` grows significantly in Phase 3 (GDD §9's difficulty curve, up to ~8), a circular layout still works but gets busier — don't over-engineer a general-purpose graph layout algorithm for that now; note it as a future concern if it comes up.

---

## 🔗 Dependencies

- **Depends on**: 020 (confirmation engine), 021 (existing hypothesis grid, being replaced)
- **Blocks**: none (independent of 030)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/031-hypothesis-grid-as-graph.md)"$'\n\nExecute this task in the current project.'
```
