# Task 172 — Inspection tool UX fixes: stable tooltip sizing, cursor-positioned card, biome info when populated

> **ID**: `172`
> **Category**: Bug fix
> **Priority**: 🟡 P2 (core tool, used every session)
> **Estimate**: ~1h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

First human playtest (`playtest_outcome.md`, issue I.1) surfaced three real
bugs in the inspection tool shipped by task 149, confirmed by reading
`src/ui.rs`:

1. **Hover tooltip wraps to one character per line / width jitters.**
   `hover_tooltip` (`src/ui.rs:940-979`) creates its `egui::Area` with a
   fixed literal id `"hover_tooltip"` (line 955) shared across every hovered
   cell. egui caches an `Area`'s measured size under that id and feeds the
   *previous* frame's rect back in as this frame's `max_rect` before
   re-laying-out (egui `area.rs` `rect()`/`show()` flow). Moving the cursor
   from a cell with short content (biome only) to one with long content
   (species name + population line) renders that transition frame
   constrained to the old, narrower width, and label wrapping breaks the
   long text down — in the worst case to one character per line.
2. **Click card always opens top-left instead of near the cursor.**
   `inspect_card` (`src/ui.rs:985-1004`) never calls `.default_pos()` /
   `.fixed_pos()` / `.current_pos()` on its `egui::Window`, so egui falls
   back to `automatic_area_position`, which places the first-ever window
   near the viewport's top-left corner — with no awareness of the clicked
   cell or cursor position at all.
3. **Populated-cell card never shows biome info.** `populated_cell_card`
   (`src/ui.rs:1012-1058`) and `empty_cell_card` (`src/ui.rs:1066-1102`) are
   mutually exclusive branches of the `match` at `src/ui.rs:999-1001` keyed
   on `cell.population.is_some()`. Only `empty_cell_card` renders
   biome/temperature/light/toxicity (`text::band_label`) — a cell with a
   species loses that context entirely.

Design source: `playtest_outcome.md` issue I.1 (this playtest run).

Not in scope: the card staying open until another cell is clicked or Esc is
pressed is **working as designed** — task 149's own acceptance criteria
(`tasks/done/149-inspection-tool.md:101-104`) state this explicitly. Don't
change dismiss behavior as part of this task.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] Hover tooltip no longer wraps mid-word/mid-character on cell-to-cell
      transitions with differing content length — either key the `Area` id
      on content that forces a fresh layout (e.g. include the hovered
      `CellIndex` or a content hash in the id) or set an explicit
      `.min_width()`/fixed width sized for the longest realistic line
      (species name + population), whichever reads cleaner once tried.
- [ ] Click card opens anchored near the triggering cursor position (or a
      fixed, deliberately chosen screen position — pick whichever the
      restyling direction in `playtest_outcome.md` I.1 suggests: "vicino al
      cursore oppure fixed in un punto"), never defaulting to
      `automatic_area_position`'s top-left fallback.
- [ ] `populated_cell_card` also renders the cell's biome/environment info
      (reuse `empty_cell_card`'s `band_label` calls / a shared helper) above
      or alongside the species detail, so biome context is never lost when
      a species is present.
- [ ] Manual check: hover across cells with short and long content
      back-to-back, click several populated and empty cells near map edges,
      confirm no clipping/wrap regressions.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `hover_tooltip` (940-979), `inspect_card` (985-1004), `populated_cell_card` (1012-1058), `empty_cell_card` (1066-1102). |

---

## ⚠️ Constraints and Caveats

- Don't touch dismiss-on-click-elsewhere behavior — confirmed working as
  designed (149's acceptance criteria).
- Keep this a targeted bug fix, not a restyle — `playtest_outcome.md`
  explicitly suggests deeper restyling ("magari con il restyling grafico")
  as a later pass, not this task's scope.

---

## 🔗 Dependencies

- **Depends on**: 149 (inspection tool, already shipped).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/172-inspection-tool-ux-fixes.md)"$'\n\nExecute this task in the current project.'
```
