# Task 115 — Grid input (clicks and scroll-zoom) leaks through the HUD panel

> **ID**: `115`
> **Category**: Bugfix
> **Priority**: 🟡 P2
> **Estimate**: ~2h (was ~2h for clicks alone; budget more now the scope
> covers a second input path — reassess once the root cause is found, since
> a shared root cause would make both fixes cheap together)
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (clicks, reported during 098/099's manual playtest
> pass); **extended 2026-08-12** (scroll-zoom, reported during a live
> playtest of tasks 097/103/105 — see the new Objective paragraph and
> acceptance criterion below)

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

**2026-08-12 addition — a second input path with the same symptom**: a live
playtest also found that scrolling the mouse wheel while the pointer sits
over the HUD sidebar zooms the map camera instead of being absorbed by the
panel (e.g. scrolling over the Biosphere list). This is a *different*
system — `render.rs::zoom_camera` (~948), not `input.rs`'s click handlers —
but it already has the same kind of guard task 091 added for clicks:
`if egui_wants_input.wants_pointer_input() { return; }` (`render.rs` ~967).
That the guard is present and still doesn't stop the zoom is itself
evidence: whatever makes `EguiWantsInput` unreliable over the HUD panel is
likely shared between the click path and the scroll path, so this is folded
into the same investigation rather than filed separately. If the two turn
out to have unrelated root causes after investigation, split the zoom half
back out into its own task at that point — don't assume they're the same
fix going in, just start from one investigation.

---

## 📋 Acceptance Criteria

- [x] Root cause identified and documented: why does the grid-click path fire
      for screen coordinates that are visually under the HUD panel, at some
      zoom levels but (presumably) not others?
- [x] Clicking anywhere within the HUD panel's actual screen rect — at any
      zoom level, Detail or Overview — never triggers a grid action (`Seed`,
      `Stress`, `Cull`, `Splice`, species selection, etc. all stay
      panel-only).
- [x] Clicking the same screen position when the panel is *not* covering it
      (e.g. after zooming out, or on a resolution where the grid doesn't
      reach that far) still behaves exactly as before — this is a gating fix,
      not a change to click→cell mapping math itself.
- [x] Regression test covering the specific failure: a click at a screen
      position under the panel's rect, at a zoom level where the grid extends
      there, must not produce a grid action.
- [x] Scrolling the mouse wheel while the pointer is over the HUD panel
      (any zoom level — the reported case didn't require the grid to extend
      under the panel first, unlike the click bug) never changes camera
      zoom. Root cause documented: does it share the click bug's cause, or
      is `EguiWantsInput` failing differently for scroll events specifically?
- [x] Regression test covering the scroll-zoom failure, parallel to the
      click one above.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [x] Verified live via `cargo run`: zoom in until the grid extends under the
      right panel, click a sidebar widget (e.g. a species row) and confirm no
      organism gets placed on the grid underneath; separately, scroll the
      mouse wheel over the HUD sidebar (e.g. over the Biosphere list) and
      confirm the map does not zoom. Confirmed by the user 2026-08-13 (the
      agent cannot drive mouse input on a native winit window in this
      environment and only confirmed a clean launch beforehand).

---

## 🔍 Outcome notes (2026-08-13)

**Root cause (shared by both bugs).** `hud_panel` (`ui.rs:394`) shows its
`egui::Panel::right("hud")` into a `Ui` it builds itself on
`egui::LayerId::background()` — the same layer/order the grid's own terrain
overlay paints on. `egui::Context::is_pointer_over_egui` (which
`EguiWantsInput::is_pointer_over_area`/`wants_pointer_input` are derived
from) special-cases `Order::Background`: it only reports "over egui" there
if the cursor falls *outside* `root_ui_available_rect`, egui's own bookkeeping
of the space left over after side panels reserve theirs. That rect is only
ever populated by `Context::run_ui`'s own top-level root `Ui`. bevy_egui
0.41's multi-pass driver (`run_egui_context_pass_loop_system`) calls
`ctx.run_ui(input, |_| { world.try_run_schedule(EguiPrimaryContextPass) })`
— the root `Ui` argument is discarded (`|_|`) and never drawn into; the real
UI work happens inside `EguiPrimaryContextPass`'s own systems
(`hud_panel`/`notebook_window`), against the raw `Context`, entirely outside
that closure. So `root_ui_available_rect` stays the *full* viewport forever,
`is_pointer_over_egui`'s Background-order branch never excludes the panel,
and `EguiWantsInput` under-reports for the entire viewport — including
squarely over the HUD panel — whenever the topmost layer under the cursor is
Background-order. This is a single-root-cause bug behind both the click
leak (previously suspected as an `EguiWantsInput`/`wants_pointer_input`
down-frame quirk during `any_down()`, which was investigated and ruled out:
it can't explain the scroll case, since no button is down during a wheel
scroll) and the scroll leak (`zoom_camera` uses the same resource, same way).

**Fix.** Sidesteps `EguiWantsInput` for this specific area with a plain rect
check: `ui::cursor_over_hud_panel(cursor, window_width)` — the HUD's on-screen
strip, computed from `HUD_WIDTH` the same way `reserve_hud_viewport` already
does for the grid camera's viewport. Both `input.rs::clicked_cell` and
`render.rs::zoom_camera` gate on it as an *additional* check alongside the
existing `EguiWantsInput::wants_pointer_input()` (kept, not replaced — it's
still correct for Foreground-order egui surfaces like popups/tooltips this
rect doesn't cover).

**Live verification.** `cargo run` launches cleanly (window opens, no
panic/error in the log). The mouse-driven check itself — zoom in, click a
sidebar widget, confirm no placement; scroll over the sidebar, confirm no
zoom — was performed by the user (2026-08-13) and confirmed working, since
this agent has no way to drive mouse input on a native winit window in this
environment.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `clicked_cell`, `seed_organism_on_click`, and the other action click-handlers — check `EguiWantsInput` usage and how screen→world→cell mapping is gated against the panel. |
| `src/ui.rs` | `hud_panel` — the panel's layout/screen area egui actually occupies. |
| `src/render.rs` | `zoom_camera` (~948) — already gated on `EguiWantsInput::wants_pointer_input()` (~967) yet still lets scroll-zoom through over the HUD; `clamp_camera_pan` and friends for whether the grid's rendered extent under the panel changes with zoom in a way that interacts with the click gating. |

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
