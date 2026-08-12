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

- [ ] The notebook renders as a docked panel anchored to the **left** edge
      of the screen (`egui::Panel::left(...)` or equivalent — mirror
      `hud_panel`'s `egui::Panel::right("hud").exact_size(HUD_WIDTH)`
      pattern for consistency), not a floating `egui::Window`.
- [ ] While the notebook is open, a semi-transparent dark overlay is drawn
      over the visible map area (excluding the notebook panel itself and the
      right HUD sidebar, which both stay at full visibility/opacity) —
      communicating "the world is still there, dimmed" rather than hidden or
      paused-looking.
- [ ] The right-hand HUD panel (`ui.rs::hud_panel`) remains fully visible and
      interactive while the notebook is open — no regression to its current
      always-on-top docked behavior.
- [ ] Closing: `Tab` (existing binding, `notebook.rs::toggle_notebook`)
      still works: **and** a click on the dimmed map area (outside both the
      notebook panel and the HUD sidebar) also closes the notebook. Clicks
      inside the notebook panel or the HUD sidebar must not close it.
- [ ] **Decision required, explicitly**: does simulation time keep advancing
      while the notebook is open? The source doc deliberately leaves this
      open ("verificare con la logica di gioco se il tempo scorre o meno").
      Pick one, document the choice and its reasoning in this task's outcome
      notes (commit message or a note in this file), and implement
      accordingly — don't leave it as an accidental side effect of whatever
      the refactor happens to produce.
- [ ] Existing notebook content (Observation log, Relationships graph,
      Species catalog — tasks 100-103's concern) renders unchanged inside
      the new panel chrome; this task doesn't touch what's drawn *inside*,
      only the window/panel/overlay chrome around it.
- [ ] The Observation log's scroll works: the player can manually scroll up
      to read older entries, and it doesn't fight back to the bottom while
      they're doing so (see the known-bug note in Constraints and Caveats —
      confirm live whether this is still broken once the panel itself has
      changed, since the fix might fall out of the chrome rewrite).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: open the notebook (`Tab` or the HUD
      button), confirm it docks left, the map dims visibly behind it, the
      right HUD sidebar stays fully interactive (e.g. still able to select a
      species or advance a tick while the notebook is open, if that's the
      time-behavior decision made above), and both `Tab` and a click on the
      dimmed map close it again.

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
