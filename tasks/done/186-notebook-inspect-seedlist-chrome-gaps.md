# Task 186 — Notebook/inspect-card font gap + two more species-color leaks

> **ID**: `186`
> **Category**: UI / Bugfix
> **Priority**: 🟡 P2 (corrective — Phase 2 residual, found by user screenshot audit)
> **Estimate**: ~45min
> **Assigned to**: Claude CLI
> **Session**: 2026-08-30

---

## 🎯 Objective

User-provided screenshots (2026-08-30, sidebar/notebook/inspect card) showed
the Notebook and Inspect card still not matching the pixel-grain register,
and the Seed Palette's species list still color-coding by species identity.
Source audit confirms three concrete, previously-untracked gaps the
180-185 pass missed:

1. **`ui::species_row`** (Seed Palette list, `ui.rs`) still applies
   `.color(species_color(species))` to the whole row's `RichText` — the
   exact "color = state, never identity" violation (`VISUAL_STYLE_GUIDE.md`
   §1 rule 3) tasks 180/181/185 already fixed on the Biosphere rows,
   Catalog icons, and era-reveal swatches respectively. This one call site
   was missed by that audit.
2. **`notebook::notebook_window`** never calls `apply_monospace` on its
   panel `Ui`, and its four section headings (`Observation log`/
   `Hypothesis grid`/`Catalog`/`Chronicle`) use plain `ui.heading()` and
   `ui.separator()` instead of the HUD's own `section_header`/`hairline`
   treatment — every other restyled surface (HUD, interstitials, menu,
   pause menu, floating overlays) picked up the monospace override
   explicitly at its own top-level `Ui`; the notebook's top-level panel
   never did, so its whole body renders in egui's default proportional
   font with default headings, the single largest source of "doesn't
   match the mockup" in the second screenshot.
3. **`notebook::chronicle_panel`** independently reimplements the same
   species-colored swatch bug task 185 just fixed in `screens.rs`'s
   era-reveal card — its own evolution-entry rows still paint
   `species_color(*parent)`/`species_color(*child)` swatches, since the
   Chronicle is the archived copy of the same reveal data.

Additionally, `ui::inspect_card`/`viewport_hint`/`hover_tooltip` (task 184)
never call `apply_monospace` either — narrowed out of that task's scope at
the time as not explicitly ACed, but inconsistent with every sibling
surface and visibly plain-proportional in the third screenshot.

Read `VISUAL_STYLE_GUIDE.md` §1 rule 3, §2 (monospace panel-wide), §6
(icons as painted blocks) before touching any of this — same sections
180/181/185 already cite.

---

## 📋 Acceptance Criteria

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] `species_row`: dropped `.color(species_color(species))` — the row's
      text now goes to the default (neutral) label color, `species_color`
      import removed from `ui.rs` (no other call site left there). The
      existing `metabolism_glyph` unicode symbol stays, per the task's own
      scope note.
- [x] `notebook_window`: `apply_monospace(ui)` called once at the top of
      its panel closure. Its four section headings switched from
      `ui.heading(...)` to `section_header(ui, ...)`, and its
      `ui.separator()` calls to `hairline(ui)`. Both helpers made
      `pub(crate)` in `ui.rs`.
- [x] `chronicle_panel`: replaced the two `species_color` swatches with
      `paint_metabolism_icon` in `ICON_INK_SELECTED`; gained a `&SimWorld`
      parameter to look up each species' `metabolism`.
- [x] `inspect_card`, `viewport_hint`, `hover_tooltip`: each now calls
      `apply_monospace(ui)` on its own top-level `Ui`.
- [-] Manual check — skipped per this session's standing "no live
      verification" instruction; `cargo build`/`clippy`/`fmt`/`test` all
      clean. **Ask the user to rebuild (`cargo run`) and re-screenshot**
      before treating this as fully confirmed — the source-level diagnosis
      is solid but wasn't checked against a running binary this session.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `species_row` (Seed Palette row), `inspect_card`, `viewport_hint`, `hover_tooltip`, `section_header`/`hairline` (visibility). |
| `src/notebook.rs` | `notebook_window` (top-level panel + headings/separators), `chronicle_panel` (swatch fix). |

---

## 🧩 Technical Context

- **Why this wasn't caught by 180-185**: those tasks each targeted a named
  surface (HUD, notebook's relationship-graph/Catalog specifically,
  interstitials, pause menu, floating overlays, era-reveal) via direct code
  citation. `species_row` (Seed Palette) and `chronicle_panel` (distinct
  from the Catalog/hypothesis-grid sections 181 actually touched) were
  never named in any of those task files' relevant-files tables, and
  `notebook_window`'s own top-level panel setup — as opposed to the
  sections nested inside it — was likewise never called out. Same root
  cause task 185 itself was: a corpus-wide audit finds named surfaces,
  not every call site of a shared helper.
- **Rounded title-bar corners in the inspect-card screenshot**: the global
  `visuals.window_corner_radius = CornerRadius::ZERO` override has been in
  `ui.rs` since task 151 (2026-08-29), well before this session's changes,
  so a rebuilt binary should not show rounded `egui::Window` corners
  anywhere. If they persist after a fresh `cargo run` post-186, that's a
  distinct, new finding — not something this task's diff addresses (it
  never touches `Frame`/corner-radius code, only font/color).

---

## ⚠️ Constraints and Caveats

- Presentation-only — `sim`/`world`/`config` stay untouched (`chronicle_panel`
  gains a `&SimWorld` *parameter*, purely to read `species.metabolism`, no
  new dependency on `bevy::render`/`bevy_egui` in `sim`/`world`/`config`
  themselves).
- Don't touch the Observation-log's species-colored markers — that's task
  181's own documented, deliberate deferral (no valence signal in
  `LogEntry` to key off), not a bug this task should "fix" by inventing one.
- Don't touch `hypothesis_grid`'s node chrome — task 181 already corrected
  it to the neutral-box/amber-stroke grammar.

---

## 🔗 Dependencies

- **Depends on**: 180, 181, 185 (reuses their constants/helpers:
  `ICON_INK_SELECTED`, `paint_metabolism_icon`, `apply_monospace`,
  `section_header`, `hairline`).
- **Blocks**: none.
