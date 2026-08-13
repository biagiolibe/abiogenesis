# Task 116 — Notebook: left-docked panel with dimmed map behind it, not a floating window

> **ID**: `116`
> **Category**: UI / Notebook
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-12 (scoped from `redesign/abiogenesis-hud-notebook.md` §9, after a
> discrepancy-check pass against tasks 100-103/097)

---

## 🎯 Objective

`redesign/abiogenesis-hud-notebook.md` §9 describes how the notebook should
open: a panel that slides in from the **left** side of the screen, covering
the left portion of the map. The map itself stays visible behind it,
**dimmed** (a semi-transparent dark overlay), signaling the simulation is
"still there," not hidden. The right-hand HUD sidebar stays visible and
interactive the whole time — the player can read Biosphere/Species/Time
while the notebook is open, without closing one to see the other. Closing:
`Tab` again, or a click on the dimmed map area outside the panel.

Today's implementation doesn't match any of this: `notebook_window`
(`src/notebook.rs:487-...`) is a plain floating `egui::Window::new("Notebook")`
— draggable, no fixed side, no map-dimming overlay, no click-outside-to-close.
By contrast, the HUD sidebar (`ui.rs::hud_panel`) is *already* a properly
docked `egui::Panel::right("hud")` with `exact_size(HUD_WIDTH)` — so the
"sidebar stays right and interactive" half of this requirement is already
true today; only the notebook side needs to change.

This is a **structural layout change**, not a content change — tasks
100-102 (log/grid content inside the notebook) are unaffected by this and
can land independently, in either order.

---

## 📋 Acceptance Criteria

- [x] The notebook renders as a docked panel anchored to the **left** edge
      of the screen (`egui::Panel::left(...)` or equivalent — mirror
      `hud_panel`'s `egui::Panel::right("hud").exact_size(HUD_WIDTH)`
      pattern for consistency), not a floating `egui::Window`.
- [x] While the notebook is open, a semi-transparent dark overlay is drawn
      over the visible map area (excluding the notebook panel itself and the
      right HUD sidebar, which both stay at full visibility/opacity) —
      communicating "the world is still there, dimmed" rather than hidden or
      paused-looking.
- [x] The right-hand HUD panel (`ui.rs::hud_panel`) remains fully visible and
      interactive while the notebook is open — no regression to its current
      always-on-top docked behavior.
- [x] Closing: `Tab` (existing binding, `notebook.rs::toggle_notebook`)
      still works: **and** a click on the dimmed map area (outside both the
      notebook panel and the HUD sidebar) also closes the notebook. Clicks
      inside the notebook panel or the HUD sidebar must not close it.
- [x] **Decision required, explicitly**: does simulation time keep advancing
      while the notebook is open? The source doc deliberately leaves this
      open ("verificare con la logica di gioco se il tempo scorre o meno").
      Pick one, document the choice and its reasoning in this task's outcome
      notes (commit message or a note in this file), and implement
      accordingly — don't leave it as an accidental side effect of whatever
      the refactor happens to produce. **Decision: unaffected, no new
      gating** — see Outcome notes below.
- [x] Existing notebook content (Observation log, Relationships graph,
      Species catalog — tasks 100-103's concern) renders unchanged inside
      the new panel chrome; this task doesn't touch what's drawn *inside*,
      only the window/panel/overlay chrome around it.
- [ ] The Observation log's scroll works: the player can manually scroll up
      to read older entries, and it doesn't fight back to the bottom while
      they're doing so (see the known-bug note in Constraints and Caveats —
      confirm live whether this is still broken once the panel itself has
      changed, since the fix might fall out of the chrome rewrite). **Left
      unverified, deliberately** — `stick_to_bottom(true)` was left
      untouched (no reproduction available to this agent). Not checked
      during the 2026-08-13 live playtest either; explicitly out of scope
      for this task's close-out per the user's own call ("va bene per ora,
      in attesa del redesign grafico") — filed as a known follow-up, not
      silently dropped.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [x] Verified live via `cargo run`: open the notebook (`Tab` or the HUD
      button), confirm it docks left, the map dims visibly behind it, the
      right HUD sidebar stays fully interactive (e.g. still able to select a
      species or advance a tick while the notebook is open, if that's the
      time-behavior decision made above), and both `Tab` and a click on the
      dimmed map close it again. Confirmed by the user across several
      iterations (2026-08-13) — see Outcome notes for the two real bugs this
      surfaced and fixed (terrain-overlay bleed-through, catalog text
      overflow) before the user signed off as working.

---

## 🔍 Outcome notes (2026-08-13)

**Chrome rewrite.** `notebook_window` (`notebook.rs`) now builds its own
root `Ui` on `egui::LayerId::background()` (mirroring `hud_panel`'s
`viewport_ui` pattern exactly) and shows `egui::Panel::left("notebook")
.exact_size(NOTEBOOK_WIDTH)` into it, instead of `egui::Window::new
("Notebook")`. `NOTEBOOK_WIDTH = 480.0` (raised from an initial `420.0`
during the live playtest round below).

**Dimming overlay.** Computed as `viewport` minus `NOTEBOOK_WIDTH` (left)
minus `ui::HUD_WIDTH` (right, now `pub(crate)` for this), filled with
`MAP_DIM_COLOR` (semi-transparent black) via the same
`ctx.layer_painter(egui::LayerId::background())` technique
`terrain_overlay` already uses — drawn *before* the panel `.show()` call so
the panel lands on top within the same layer pass.

**Click-outside-to-close.** Detected directly off `ctx.input(|i| i.pointer
.button_clicked(...))` + `interact_pos()` containment in the dimmed-map rect
— not `EguiWantsInput`, which task 115's investigation already found
unreliable over any panel sharing the Background layer (this new notebook
panel included). `Tab` (`toggle_notebook`, unchanged) still works
independently.

**A third leak-through case, found while wiring this up.** The notebook
panel paints on the same Background layer the HUD panel does, so it has
exactly task 115's click/scroll-leak bug too — a click on the dimmed map
(meant only to close the notebook) would otherwise also reach
`input.rs::clicked_cell` and fire whatever `SelectedAction` is armed, and
scrolling the Observation log's own `ScrollArea` would zoom the map
underneath. Fixed the same way as 115: `clicked_cell` now blocks *all* grid
actions outright whenever `NotebookWindowOpen` is true (not just clicks
inside the panel/dim rect — the whole point of that click is "close the
notebook," never a second effect on the grid), and `render.rs::zoom_camera`
gates on a new `notebook::cursor_over_notebook_panel` alongside the existing
HUD check.

**Simulation-time decision: unaffected, no new gating.** `EraState`/ticking
keep running exactly as before while the notebook is open — this is the
task's own suggested default ("world keeps living while you observe it,"
matching the dimmed-not-paused backdrop) and requires no new code, only this
explicit record of the choice.

**Observation log scroll bug: not investigated further.** The task file
flags a possible `stick_to_bottom(true)` interaction bug, to be confirmed
live once the chrome changes. This agent has no way to reproduce scrolling
interactively in this environment, so the `ScrollArea` code was left exactly
as it was (untouched) rather than guessing at a fix for an unreproduced
symptom. Needs the user's live check; if still broken, it's a small
follow-up against the same `ScrollArea` call, not a chrome issue.

**Live playtest round (2026-08-13) — three real bugs found and fixed before
sign-off**, none catchable from `cargo test`/`clippy` alone (pure layout/
compositing issues, screen-only):

1. *Terrain overlay bleeding through the notebook panel.* First live
   screenshot showed `render.rs`'s boundary lines and tree glyphs painting
   straight across the notebook's own black background. Root cause:
   `ui::reserve_hud_viewport` only ever shrunk `GridCamera`'s viewport for
   `HUD_WIDTH`, never for the notebook — so `draw_terrain_overlay`'s clip
   (built from `Camera::logical_viewport_rect()`) never excluded the
   notebook's screen area either. First attempt shrunk the camera viewport
   by `NOTEBOOK_WIDTH` too, mirroring the HUD pattern — this visibly fixed
   the bleed-through but introduced a second regression (next item) and was
   reverted.
2. *The map visibly resized/rescaled every time the notebook opened or
   closed.* Caused by the viewport-shrink fix above: unlike the HUD sidebar
   (permanent, so resizing the map to exactly fill the remaining space is
   correct), the notebook is a toggleable overlay, and `redesign/
   abiogenesis-hud-notebook.md` §9 explicitly wants the map to stay the same
   size, dimmed behind the panel — not resized. Fixed by reverting
   `reserve_hud_viewport` to HUD-only, and instead having
   `draw_terrain_overlay` additionally `.with_clip_rect(...)` (egui rects
   intersect on chained calls) to exclude the notebook's reserved rect when
   open — occlusion via clipping/opaque-panel compositing, not by moving the
   camera. Bevy-rendered sprites need no equivalent fix: the notebook panel
   renders through the same dedicated HUD camera (`ui.rs::spawn_hud_camera`,
   `order: 1`) that already composites on top of `GridCamera`'s output, so
   its opaque background occludes sprites underneath without any camera
   change — the same reason the HUD sidebar's own sprite-occlusion needs its
   viewport shrink but the notebook's occlusion doesn't.
3. *"Observation log" heading rendered under macOS's traffic-light window
   buttons.* Top-left is the one screen corner `main.rs`'s
   `fullsize_content_view`/`titlebar_transparent` setup never had to account
   for before (the grid tolerates content there fine; `hud_panel`'s own
   heading sits top-*right*, away from the buttons). Fixed with a
   `TITLEBAR_CLEARANCE` (28pt) `ui.add_space` before the notebook panel's
   first heading only.
4. *Catalog species rows overflowing/truncating past the panel edge*
   (visible only after fix #2 stopped it from bleeding onto the map instead
   — same underlying cause as #2's original symptom, different visible
   effect once properly clipped). The old floating `egui::Window` auto-sized
   to fit; a fixed-width panel doesn't. Fixed with `.wrap()` on the long
   catalog line label plus moving the per-species tag glyphs onto their own
   `horizontal_wrapped` row (both correctness fixes, not guesses — they hold
   at any panel width/species-name length), and `NOTEBOOK_WIDTH` raised
   `420.0` → `480.0` for comfort on top of that.

**Residual, not investigated**: the Observation log scroll bug (`stick_to_bottom`) — see the acceptance-criteria note above; the user signed off on the rest as sufficient for now, pending a separate graphic redesign pass. Also not audited: `placement_indicator`/`spark_indicator`/`energy_overlay` (debug-only) painting the same way `draw_terrain_overlay` did — narrower windows for the same class of bug (a Seed placement or an `AdjacencyObserved` spark firing while the notebook happens to be open), left as-is since neither was observed live and this task's confirmed scope was the terrain overlay specifically.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `notebook_window` — convert from `egui::Window` to a docked left `egui::Panel`; add the dimming overlay and click-outside-to-close handling. |
| `src/ui.rs` | `hud_panel` — the existing `egui::Panel::right("hud").exact_size(HUD_WIDTH)` pattern to mirror; `HUD_WIDTH` constant (line ~140) as a naming/sizing precedent for a `NOTEBOOK_WIDTH` equivalent. |
| `redesign/abiogenesis-hud-notebook.md` | §9 "Come si apre il notebook" — the full decision record this task implements; §10 "Cosa serve per l'integrazione" explicitly flags the time-behavior decision as unresolved. |

---

## 🧩 Technical Context

- **Current behavior**: `notebook_window` (`src/notebook.rs`) draws via
  `egui::Window::new("Notebook").open(&mut open.0).show(ctx, |ui| { ... })`
  — a free-floating, draggable window with no fixed position, no backdrop
  dimming, closed only via its own `[x]` or `Tab`.
- **Desired behavior**: a fixed left-docked panel, map dimmed behind it,
  closable by `Tab` or clicking the dimmed area, HUD sidebar unaffected.
- `hud_panel` already builds a `viewport_ui` from `ctx.viewport_rect()` and
  docks its own panel via `egui::Panel::right("hud").exact_size(HUD_WIDTH)`
  (`src/ui.rs:352-365`) — the notebook panel should follow the same
  established pattern (`egui::Panel::left`), not a different egui API, to
  keep the two panels' behavior consistent (resizing, spacing, etc.).
- The dimming overlay needs to sit *between* the grid rendering and the
  panels, above the map but below both docked panels' content — likely an
  `egui::Area`/full-screen painter rect drawn in `notebook_window` before
  the panel itself, at a background-ish `egui::LayerId`, semi-transparent
  black fill over the viewport rect minus the two panels' reserved widths.
  Check how `MapViewMode`/camera viewport-shrinking already accounts for
  `HUD_WIDTH` (`src/ui.rs`, "Shrinks the grid camera's viewport by
  `HUD_WIDTH`") for the existing precedent of "reserve panel width, render
  grid in what's left" — the notebook's dimming overlay needs the same kind
  of rect math, now accounting for both panel widths when the notebook is
  open.
- Click-outside-to-close needs to distinguish "click landed on the dimmed
  map area" from "click landed on either docked panel" — probably via
  `EguiWantsInput`/hit-testing the two panels' known rects, similar in
  spirit to task 115's investigation (that task is about clicks
  *leaking through* a panel; this one is about detecting a click that
  correctly lands *outside* both panels, to trigger a close). Not the same
  bug, but adjacent code paths — worth a glance at 115's findings if it
  lands first, though neither blocks the other.

---

## 🔨 Suggested Implementation

1. Read `hud_panel`'s current `egui::Panel::right` usage in full as the
   pattern to mirror.
2. Convert `notebook_window` to build a left-docked `egui::Panel` with the
   same content it renders today (Observation log / Relationships /
   Species catalog sections unchanged).
3. Add the dimming overlay: a semi-transparent rect covering the map's
   remaining visible area (viewport minus both panels), drawn only while
   the notebook is open.
4. Add click-outside-to-close: detect a click landing outside both panels'
   rects while the notebook is open, and close it (set `NotebookWindowOpen`
   to `false`), same as the existing `Tab` toggle path.
5. Decide and implement the simulation-time-while-open question (see
   Acceptance Criteria) — simplest default if no strong reason otherwise:
   leave `EraState`/ticking completely unaffected by `NotebookWindowOpen`
   (i.e. don't add new gating), matching today's actual (undocumented)
   behavior, and just document that choice explicitly rather than treating
   it as unconsidered.
6. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
7. `cargo run`: verify per the acceptance criteria's live-verification line.

---

## ⚠️ Constraints and Caveats

- **Content unchanged**: this task only touches the notebook's window/panel
  chrome (position, backdrop, close behavior) — not what's rendered inside
  it. Tasks 100 (log), 101/102 (graph), 103 (catalog) own the content and
  can land independently, before or after this one.
- **Don't regress the HUD sidebar**: `hud_panel`'s existing docked-right
  behavior must stay exactly as it is — this task adds a second docked
  panel (left), it doesn't touch the first.
- **Determinism**: this is pure UI/rendering, no simulation/RNG state
  involved, except for whatever the time-behavior decision above implies
  (which, if it does gate ticking, must not introduce non-determinism —
  gating an existing deterministic system on/off based on a UI bool is
  fine, same category as `EraState` itself).
- **Known bug to fix while this panel is being touched anyway (reported
  2026-08-12, deliberately deferred to this task rather than a standalone
  fix)**: the Observation log's `ScrollArea` (`notebook.rs` ~577-598,
  `.stick_to_bottom(true)`) no longer lets the player scroll up through
  older entries in a live playtest — it appears to keep snapping back to
  the bottom. Not root-caused yet; `stick_to_bottom(true)`'s interaction
  with manual drag is the prime suspect, but confirm live rather than
  assuming. Since this task already rebuilds the notebook's chrome
  (`egui::Window` → docked `egui::Panel`), verify the scroll behavior
  explicitly as part of this task's own live-verification pass instead of
  filing a separate task against code this task is about to replace
  anyway.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.
- **Related, not a dependency**: 100/101/102/103 (notebook content, safe to
  land in any order relative to this task); 115 (HUD-panel click-through bug
  — adjacent code path, not the same bug, doesn't block either way).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/116-notebook-docked-panel-dimmed-map.md)"$'\n\nExecute this task in the current project.'
```
