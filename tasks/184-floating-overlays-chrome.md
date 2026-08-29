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

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] **Fix the saturated-without-outlet color inversion** (highest
      priority item in this task): `inspect_card`'s
      `ui.colored_label(DOT_FILLED_COLOR, text::SATURATED_NO_OUTLET_WARNING)`
      (`ui.rs:1064`) paints a *negative* condition in the *positive* state
      color. Change to `STATE_NEGATIVE` (task 182's constant). Verify no
      other call site reuses `DOT_FILLED_COLOR` under the same
      "positive/active" assumption for something that's actually negative —
      grep every `DOT_FILLED_COLOR` use before renaming it wholesale.
- [ ] **Replace `hover_tooltip`'s Unicode trend glyphs.** `trend_glyph`
      (`ui.rs:2006-2012`) returns `▲`/`▼`/`▬` rendered via
      `ui.colored_label` (`ui.rs:985`) — a font glyph, forbidden by §6.
      Replace with a small painted 3×3-block arrow (up/down/flat), same
      block-pattern technique as 151/180's icons — reuse whatever shared
      icon-painter helper 180 introduces if it's landed, otherwise a
      minimal local one (this doesn't need `MetabolismShapes`' full
      geometry, just three tiny arrow patterns).
- [ ] **`trend_color`** (`ui.rs:2018-2024`) — replace its hardcoded
      `(96,200,120)`/`(220,96,96)` with `STATE_POSITIVE`/`STATE_NEGATIVE`
      (task 182's constants) instead of its own approximation. (If task
      182 already renamed `trend_color`'s constants as part of its own
      scope, this AC is already satisfied — verify, don't duplicate.)
- [ ] **Panel chrome for all three `Frame::popup` surfaces**
      (`hover_tooltip` `ui.rs:966-979`, `inspect_card`'s `Window`
      `ui.rs:1015`, `viewport_hint` `ui.rs:932-941`): explicit `PANEL_BG`
      fill, `OUTLINE_STROKE` border, **no blurred shadow**
      (`egui::Shadow::NONE` or flat-alpha equivalent — §1 rule 4).
      `inspect_card` is a literal `egui::Window` with a title bar/drag
      handle — keep the interaction (it's meant to be draggable/click-
      dismissible) but override its frame the same way 183 does for its
      two windows.
- [ ] **`hairline()` calls inside `inspect_card`** (`ui.rs:1043,1067`)
      already use the correct `#23262e` in-panel token (`HAIRLINE_COLOR`) —
      per `VISUAL_STYLE_GUIDE.md`'s corrected §3.1, this needs **no
      change**; don't "fix" it to `#3a4048` (that's the outline-stroke
      token, a different role).
- [ ] **Fix all 9 plain `.on_hover_text()` tooltips** at the listed line
      numbers: egui's built-in tooltip layer uses the same default
      `Frame::popup`-derived chrome as the surfaces above. If egui exposes
      a way to restyle the built-in tooltip frame globally (check
      `Style::interaction`/`visuals.window_fill` equivalents used for
      tooltips specifically — may already be partially covered by whatever
      global fill override, if any, this task or 182 establishes), do that
      once rather than touching 9 call sites individually. If no such
      global hook exists in this egui version, leave these 9 as a
      documented residual gap in the PR rather than hand-rolling 9 custom
      tooltip widgets — that's disproportionate effort for plain one-line
      hover text.
- [ ] Live visual check (`cargo run`, screenshot or interactive):
      **trigger both the healthy and the saturated-without-outlet states**
      to confirm the color inversion fix (not just that it compiles); hover
      a populated cell to confirm the trend arrow renders as a painted
      block, not a Unicode glyph or a missing-glyph box; trigger at least
      one contextual hint (e.g. the stall hint) to confirm its chrome.

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
