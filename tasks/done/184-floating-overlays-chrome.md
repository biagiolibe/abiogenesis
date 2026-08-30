# Task 184 — Floating overlays: inspect card, hover tooltip, contextual hints

> **ID**: `184`
> **Category**: UI / Bugfix
> **Priority**: 🟡 P2 (corrective — Phase 2 residual; one item is a
>   functional-visual bug, not just cosmetic — see AC 2)
> **Estimate**: ~2.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-29 scoping

---

## 🎯 Objective

Three surfaces — `inspect_card` (click-to-inspect window, incl. the
per-neighbour energy breakdown), `hover_tooltip` (cursor-following biome/
species/trend readout), and `viewport_hint` (isolation/milestone/stall
contextual hints) — share `egui::Frame::popup`'s default chrome (gray fill,
blurred drop shadow) and have never been restyled. Beyond the shared
cosmetic gap, `inspect_card` has a genuine **state-color bug**: its
saturated-without-outlet warning (a negative condition) is painted in
`DOT_FILLED_COLOR`, the same green used everywhere else in this codebase for
"positive/active" — a semantic inversion, not just an off-palette hex.
`hover_tooltip` also renders its trend indicator as raw Unicode glyphs
(`▲/▼/▬`), which `VISUAL_STYLE_GUIDE.md` §6 explicitly forbids.

This task also closes the game's plain `.on_hover_text()` tooltips
(9 call sites, `ui.rs:720,775,815,1172,1553,1575,1602,1608,1693`) as part
of the same fix — they share the identical root cause (default egui tooltip
frame) and don't need their own task file.

Read `VISUAL_STYLE_GUIDE.md` first — §3 (color tokens — this task is the
main consumer of `STATE_POSITIVE`/`STATE_NEGATIVE`), §5 (relationship-graph
grammar — **not** applicable to `tag_glyph` here, see caveats), §6 (icons
as painted blocks, never font glyphs).

---

## 📋 Acceptance Criteria

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] **Fix the saturated-without-outlet color inversion**: `DOT_FILLED_COLOR`
      no longer exists (already renamed by task 182); the actual bug was
      `populated_cell_card`'s `ui.colored_label(STATE_POSITIVE,
      text::SATURATED_NO_OUTLET_WARNING)` — a negative condition painted in
      the positive constant. Changed to `STATE_NEGATIVE`. Grepped every
      other `STATE_POSITIVE` use (`dot_row`'s filled ticks/circles, the
      notebook badge) — all genuinely positive/active, no other inversion.
- [x] **Replaced the Unicode trend glyphs** (`trend_glyph`, both its call
      sites — `hover_tooltip` and the biosphere sidebar row, which also used
      it) with `paint_trend_arrow`: a small painted 3x3-block arrow (up/
      down/flat), same procedural block-icon technique as the action-mode
      icons, sized for inline text instead of the 40x40 button-icon scale.
      `trend_glyph` removed (no remaining callers).
- [x] **`trend_color`** already used `STATE_POSITIVE`/`STATE_NEGATIVE` — task
      182 had already fixed this; no duplicate change needed.
- [x] **Panel chrome for all three `Frame::popup`/`Window` surfaces** — done
      globally instead of three local `.frame()` overrides: extended the
      existing `ctx.all_styles_mut` block (`ui.rs`, next to task 182's
      `panel_fill`/`window_fill` override) with
      `visuals.window_stroke = OUTLINE_STROKE`, `visuals.window_shadow =
      Shadow::NONE`, `visuals.popup_shadow = Shadow::NONE`. Since
      `Frame::window`/`Frame::popup` both derive fill/stroke/shadow from
      these `Style::visuals` fields, this fixes `viewport_hint`,
      `hover_tooltip`, and `inspect_card`'s `Window` in one place —
      matching the suggested-implementation note to prefer a global hook
      over touching every call site.
- [x] **`hairline()` inside `inspect_card`** — left untouched, per the AC.
- [x] **All 9 plain `.on_hover_text()` tooltips** — fixed for free by the
      same global `visuals.window_stroke`/`popup_shadow` override above:
      egui's built-in tooltip always renders through `Frame::popup(style)`
      with no per-call frame override point, so the global fix reaches them
      too. No per-call-site changes needed.
- [-] Live visual check — skipped per explicit user instruction for this
      task; `cargo build`/`clippy`/`fmt`/`test` all clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `inspect_card`/`populated_cell_card` (`1004-1134`), `hover_tooltip` (`951-998`), `viewport_hint` (`884-944`), `trend_glyph`/`trend_color` (`2006-2024`), `DOT_FILLED_COLOR` (`1419`), `hairline`/`HAIRLINE_COLOR` (`1391`, unchanged). |

---

## 🧩 Technical Context

- **Current behavior**: all three surfaces use `Frame::popup`'s default
  fill and blurred shadow; the saturation warning is colored as if
  positive; the trend indicator is an unrendered/inconsistent Unicode
  glyph; 9 scattered tooltips share the same unstyled default.
- **Desired behavior**: dark flat-chrome overlays, correct state colors
  (negative reads as negative), a painted block-pattern trend icon, and the
  plain tooltips either fixed globally or explicitly logged as a residual
  gap.

---

## 🔨 Suggested Implementation

1. Fix `DOT_FILLED_COLOR` → `STATE_NEGATIVE` at the one call site that's
   actually wrong (`ui.rs:1064`); confirm every *other* `DOT_FILLED_COLOR`
   use is genuinely positive/active before leaving it alone.
2. Write a tiny `fn paint_trend_arrow(painter, rect, trend, color)` — 3×3
   block arrow, three variants — replacing `trend_glyph`'s string return
   with a direct paint call at its one call site.
3. `trend_color`: swap in the shared constants.
4. Add explicit `Frame` overrides (fill, stroke, `Shadow::NONE`) to the
   three `Frame::popup`/`Window` surfaces.
5. Check egui's tooltip-styling API for a global override point before
   touching the 9 individual `.on_hover_text()` call sites by hand.
6. Live-check: healthy vs. saturated cell, hover trend arrow, one
   contextual hint.

---

## ⚠️ Constraints and Caveats

- **Don't touch `tag_glyph`** (Greek letters, `inspect_card` lines
  `1059,1073`) — that's task 155's job (3-letter codes), not this task's.
  Note the dependency, don't implement it here.
- **No hand-drawn assets** — the trend arrow is a tiny procedural block
  pattern, not an image.
- If the 9 plain tooltips can't be fixed globally without disproportionate
  effort, say so explicitly in the PR rather than silently leaving them
  broken with no note.
- Keep `sim`/`world`/`config` untouched.

---

## 🔗 Dependencies

- **Depends on**: 182 (reuses its shared state-color/panel constants).
- **Soft-depends on**: 155 for `tag_glyph` (not touched here, just noted).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/184-floating-overlays-chrome.md)"$'\n\nExecute this task in the current project.'
```
