# Task 115 — Grid clicks leak through the HUD panel when the camera is zoomed

> **ID**: `115`
> **Category**: Bugfix
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (reported by the user during 098/099's manual playtest pass)

---

## 🎯 Objective

When the grid camera is zoomed in enough (Detail or Overview) that the
rendered grid extends underneath the right-hand HUD panel, clicks landing on
that overlapping screen area — including clicks on the sidebar's own widgets,
e.g. selecting a species in the Seed Palette — are being interpreted as a
click on the grid cell underneath instead of being absorbed by the egui
panel. Concretely: clicking to select a species in the palette places an
organism on whatever grid cell happens to sit under that panel area on
screen.

This suggests `input.rs`'s click handling isn't correctly gating on
`EguiWantsInput`/the panel's actual screen rect at some zoom levels — or the
panel's occupied area isn't excluded from the world-space click→cell mapping
consistently across zoom levels. Not yet investigated in code; the exact
mechanism needs to be found before this can be scoped precisely.

**Important prior art**: task 091 (`tasks/done/091-egui-input-capture-gating.md`)
already gated `input.rs::clicked_cell` (the shared helper `seed_organism_on_click`/
`stress_on_click`/`cull_on_click` all call) behind
`EguiWantsInput::wants_pointer_input()`, specifically to stop clicks on egui
widgets from also registering as grid clicks. If that gating is still in
place and correctly wired, this is either a regression, an edge case 091
didn't cover (e.g. a specific widget/area in the panel that doesn't set
`wants_pointer_input()` true, or a schedule-ordering issue where
`EguiWantsInput` hasn't been updated yet for the frame the click landed in),
or something specific to how zoom changes what's rendered under the panel
(unlikely to matter for pointer-capture, which is about the panel's screen
rect, not what's drawn beneath it — but worth ruling out explicitly). Read
091's diff/reasoning before assuming this needs new gating logic from
scratch.

---

## 📋 Acceptance Criteria

- [ ] Root cause identified and documented: why does the grid-click path fire
      for screen coordinates that are visually under the HUD panel, at some
      zoom levels but (presumably) not others?
- [ ] Clicking anywhere within the HUD panel's actual screen rect — at any
      zoom level, Detail or Overview — never triggers a grid action (`Seed`,
      `Stress`, `Cull`, `Splice`, species selection, etc. all stay
      panel-only).
- [ ] Clicking the same screen position when the panel is *not* covering it
      (e.g. after zooming out, or on a resolution where the grid doesn't
      reach that far) still behaves exactly as before — this is a gating fix,
      not a change to click→cell mapping math itself.
- [ ] Regression test covering the specific failure: a click at a screen
      position under the panel's rect, at a zoom level where the grid extends
      there, must not produce a grid action.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: zoom in until the grid extends under the
      right panel, click a sidebar widget (e.g. a species row) and confirm no
      organism gets placed on the grid underneath.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `clicked_cell`, `seed_organism_on_click`, and the other action click-handlers — check `EguiWantsInput` usage and how screen→world→cell mapping is gated against the panel. |
| `src/ui.rs` | `hud_panel` — the panel's layout/screen area egui actually occupies. |
| `src/render.rs` | Camera zoom/pan (`clamp_camera_pan` and friends) — whether the grid's rendered extent under the panel changes with zoom in a way that interacts with the click gating. |

---

## 🧩 Technical Context

<!-- TODO: add relevant code snippets and file paths -->

- **Current behavior**: at some zoom levels, a click on the HUD panel's
  screen area is not fully absorbed by egui and falls through to the grid
  click handler, which maps it to a cell and performs the currently-selected
  action there.
- **Desired behavior**: the HUD panel's screen rect should always take
  priority over grid clicks, regardless of zoom/pan state.

---

## 🔨 Suggested Implementation

1. Reproduce: zoom in until the grid visibly extends under the right panel,
   click a panel widget, confirm the bug.
2. Trace `input.rs`'s click-handling systems to find where `EguiWantsInput`
   (or an equivalent egui-focus/hover check) is read, and whether it's
   consulted before or after the world-space cell lookup, and whether it
   covers every zoom level.
3. Fix the gating (likely: check the panel's occupied rect / egui's pointer-
   over-area state before doing any grid-click mapping, not just a global
   "does egui want input" flag that might not be set for a non-interactive
   part of the panel, or might be computed before/after camera zoom updates
   in the frame).
4. Add the regression test per Acceptance Criteria.
5. `cargo run` live verification.
6. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- **Don't change click→cell mapping math** for the normal (non-overlapping)
  case — this is scoped to the gating/exclusion logic, not a rewrite of
  `clicked_cell`.
- **Determinism**: this is pure input/UI handling, no RNG/simulation state
  involved — should be safe to fix without touching `sim`/`world`/`config`.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.
- **Related, not a dependency**: 091 (the existing `EguiWantsInput` gating
  this bug apparently slips past — read it first).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/115-egui-panel-click-through-when-zoomed.md)"$'\n\nExecute this task in the current project.'
```
