# Task 187 — Seed Palette scroll height + notebook text clipping (post-186 regressions)

> **ID**: `187`
> **Category**: UI / Bugfix
> **Priority**: 🟡 P2 (corrective — surfaced by user screenshot review of task 186)
> **Estimate**: ~30min
> **Assigned to**: Claude CLI
> **Session**: 2026-08-30

---

## 🎯 Objective

User screenshots after task 186 landed showed two more concrete bugs:

1. **Sidebar Species (Seed Palette) list scrolls after ~2-3 species**
   instead of `SPECIES_VISIBLE_ROWS` (5). Root cause: `console_row_height`
   measures a *single* text line, but `species_row` (`ui.rs`) is two lines
   — the selectable name label plus a `ui.weak` metabolism/temperature-fit
   subtext (added by task 152, after `console_row_height`/
   `SPECIES_VISIBLE_ROWS` were written for the Biosphere list's one-line
   rows). The `max_height` budget was never doubled for the Species list's
   own two-line row, so 5 "rows" of budget only fit ~2.5 real rows.
2. **Notebook text silently clipped**, worst in the Catalog's metabolism
   legend and any Observation-log/Chronicle line — not wrapped, not
   horizontally scrollable, just cut off at `NOTEBOOK_WIDTH`. Root cause:
   several `ui.label`/`ui.weak` calls sit inside a `ui.horizontal(...)`
   row, and (per `catalog_panel`'s own task-116 doc comment) a plain label
   inside a horizontal layout does not wrap by default in egui — that fix
   was only ever applied to the Species catalog's own per-species line, not
   the metabolism legend line, the observation log's entry line, or the
   Chronicle's evolution line. Task 186 switching the whole panel to
   monospace (wider glyphs than the previous proportional font) pushed
   several previously-borderline lines over the edge, making the
   pre-existing gap suddenly obvious.

Design source: same `VISUAL_STYLE_GUIDE.md`/task-116 precedent already
cited by `catalog_panel`'s own comments — wrapping, not horizontal
scrolling, is this codebase's established fix for this exact failure mode.

---

## 📋 Acceptance Criteria

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] Seed Palette `ScrollArea`'s `max_height` budget doubled
      (`SPECIES_VISIBLE_ROWS as f32 * console_row_height(ui) * 2.0`) to
      account for `species_row`'s two lines — 5 species now visible before
      scrolling, matching the Biosphere list's actual visible-row count.
- [x] Observation log's entry line, Chronicle's evolution line, and the
      Catalog's metabolism legend line each get explicit `.wrap()`
      (`egui::Label::new(...).wrap()`), the same fix already applied to the
      Species catalog's own per-species line — no more silent clipping
      inside a `ui.horizontal` row.
- [x] Manual check — user rebuilt and confirmed fixed (2026-08-31,
      together with task 188's follow-up fixes).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | Seed Palette `ScrollArea` (`max_height`). |
| `src/notebook.rs` | Observation log entry line, Chronicle evolution line, Catalog metabolism legend line. |

---

## ⚠️ Constraints and Caveats

- Wrap, not horizontal scroll — matches the codebase's existing precedent
  (`catalog_panel`'s task-116 fix) rather than introducing a second fix
  pattern for the same failure mode.
- Didn't touch `NOTEBOOK_WIDTH` itself — wrapping is content-agnostic and
  doesn't need a width retune the way task 116's original fit-first attempt
  did.
- A separate, not-yet-fixed overflow was noticed in passing:
  `viewport_hint`'s contextual-hint box (e.g. "You isolated this species…")
  also appears to run past the visible viewport edge in the user's
  screenshot — not part of this task (not what was reported), flagging for
  a future pass if it recurs.

---

## 🔗 Dependencies

- **Depends on**: 152 (`species_row`'s subtext line), 116 (`catalog_panel`'s
  wrap precedent), 186 (monospace switch that exposed the notebook gap).
- **Blocks**: none.
