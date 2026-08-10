# Task 091 — Gate map input behind egui's own input capture

> **ID**: `091`
> **Category**: Bugfix / Input
> **Priority**: 🔴 P1
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-10, user-reported live while playtesting tasks 085/086/090

---

## 🎯 Objective

Two related input bugs, same root cause: nothing in this codebase checks
whether `bevy_egui` currently wants the pointer/keyboard before the game's
own map-input systems process an event.

1. **Notebook window doesn't block map interaction.** With the notebook
   open, scrolling over it zooms the map camera underneath, and clicking a
   notebook widget can also register as a `Seed`/`Stress`/`Cull` click on
   whatever cell is under the cursor.
2. **`Tab` also shifts egui's own widget focus.** `notebook.rs::
   toggle_notebook` correctly opens/closes the notebook window on `Tab`, but
   nothing stops egui's default keyboard-navigation behavior from *also*
   treating `Tab` as "focus the next widget" — moving focus onto the
   sidebar's action buttons (`ui.rs`'s `ACTION_GLYPHS` row) or other
   persistent HUD widgets.

`bevy_egui` 0.41 ships exactly the resource this needs:
`bevy_egui::input::EguiWantsInput` (auto-inserted by `EguiPlugin`, updated
every frame in `EguiInputSet::WriteEguiEvents`), with `wants_pointer_input()`
/ `wants_keyboard_input()` / `is_pointer_over_area()`, plus ready-made run
conditions `egui_wants_any_pointer_input` / `egui_wants_any_keyboard_input`
/ `egui_wants_any_input`. This is a wiring gap, not a missing feature.

---

## 📋 Acceptance Criteria

- [x] `render.rs::zoom_camera` (mouse-wheel zoom) and `pan_camera` (if
      pointer-driven) no longer act while `EguiWantsInput::
      wants_pointer_input()` is true (or via a `.run_if(not(
      egui_wants_any_pointer_input))` system-ordering gate — implementer's
      call on exactly where the check lives, as long as it's not duplicated
      ad hoc per system).

      Done for `zoom_camera` (early-returns on `wants_pointer_input()`, right
      after draining `MouseWheel` for the frame). `pan_camera` left
      ungated — it's `WASD`/arrow-key driven, not pointer-driven, and this
      app has no keyboard-editable widget during `GameState::Playing` for it
      to steal input from (menu.rs's seed `TextEdit` is a different screen).
- [x] `input.rs::seed_organism_on_click`, `stress_on_click`, `cull_on_click`
      (and any other pointer-driven map action) are gated the same way, so
      clicking a notebook widget never also triggers a map action on the
      cell underneath.

      Gated in one place: `clicked_cell`, the shared helper all three call,
      now takes `&EguiWantsInput` and returns `None` when
      `wants_pointer_input()` is true — no per-system duplication.
- [x] `notebook.rs::toggle_notebook`'s `Tab` handling coexists cleanly with
      egui's own keyboard navigation: pressing `Tab` opens/closes the
      notebook and does *not* leave focus sitting on an HUD action button
      afterward. Concrete mechanism is an implementation decision (e.g.
      clearing egui's focused widget via `ctx.memory_mut(|mem| mem.
      surrender_focus(...))` after consuming `Tab` for our own toggle, or
      gating our own `Tab` handling behind `!wants_keyboard_input()` so the
      two don't fight over the same keypress) — verify empirically via
      `cargo run`, this is exactly the kind of egui-internals interaction
      that doesn't resolve from reading docs alone.

      Implemented as a new system, `clear_stray_tab_focus`, scheduled in
      `EguiPrimaryContextPass` and ordered `.after(hud_panel)` (`hud_panel`
      made `pub(crate)` for this) — runs *after* the sidebar's action
      buttons have had their chance to claim keyboard focus via egui's own
      `Tab` navigation this frame, then unconditionally surrenders whatever
      ended up focused, whenever `Tab` was just pressed.
      **Not independently live-verified by the agent** — this session's
      `screencapture` is unavailable (macOS Screen Recording permission not
      granted to the shell), so the actual on-screen focus behavior needs
      the user's own `cargo run` pass before this box is fully trusted; the
      reasoning above (schedule/ordering) is sound but egui's internals are
      exactly the kind of thing task 091 itself flagged as unverifiable from
      docs alone.
- [x] System ordering respects `EguiWantsInput`'s own documented update
      point (`EguiInputSet::WriteEguiEvents`, i.e. it reflects the
      *current* frame's UI layout only after egui has laid it out) — reading
      it too early would gate against last frame's UI state instead of this
      frame's.

      `zoom_camera`/`clicked_cell` read `Res<EguiWantsInput>` from plain
      `Update` systems, same schedule position every other input system in
      this codebase already runs in — `EguiWantsInput` itself is written
      during `EguiInputSet::WriteEguiEvents` (a `PreUpdate`-time set, ahead
      of `Update`), so by the time these systems run this frame's value is
      already current; no explicit `.after(...)` needed beyond the default
      `PreUpdate` → `Update` ordering Bevy already guarantees.
- [x] No change to any action's actual effect (`attempt_seed`, `apply_splice`,
      etc.) — this task only gates *whether* the system runs, not what it
      does.
- [x] `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test` clean.
- [ ] Verified live via `cargo run`: with the notebook open, scrolling over
      it zooms the notebook's own content (or does nothing), never the map
      camera; clicking a notebook widget never seeds/stresses/culls a cell;
      pressing `Tab` toggles the notebook without stray focus landing on a
      sidebar action button.

      **Pending — needs the user's own `cargo run` pass**, see the note
      under the `Tab`-focus criterion above.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `zoom_camera` (`~940`), `pan_camera` (`~1030`) — gate against `EguiWantsInput`. |
| `src/input.rs` | `seed_organism_on_click`, `stress_on_click`, `cull_on_click` — same gating. |
| `src/notebook.rs` | `toggle_notebook` (`~269-282`) — reconcile `Tab` with egui's own focus-cycling. |
| `src/ui.rs` | `ACTION_GLYPHS`/action button row (`~732-790`) — where stray focus currently lands, for reproducing/verifying the fix. |

---

## 🧩 Technical Context

`bevy_egui = "0.41"` (`Cargo.toml`). `EguiWantsInput` is registered via
`app.init_resource::<EguiWantsInput>()` in `EguiPlugin::build`
(`bevy_egui-0.41.1/src/input.rs:1007`) — no extra plugin wiring needed, it's
already present as soon as `bevy_egui`'s default plugin runs. Its API
(`bevy_egui-0.41.1/src/input.rs:1274-1390`):

```rust
pub struct EguiWantsInput { /* private fields */ }
impl EguiWantsInput {
    pub fn is_pointer_over_area(&self) -> bool;
    pub fn wants_pointer_input(&self) -> bool;
    pub fn wants_keyboard_input(&self) -> bool;
    // + is_using_pointer, is_popup_open, etc.
}
pub fn egui_wants_any_pointer_input(res: Res<EguiWantsInput>) -> bool;
pub fn egui_wants_any_keyboard_input(res: Res<EguiWantsInput>) -> bool;
pub fn egui_wants_any_input(res: Res<EguiWantsInput>) -> bool;
```

No system in this codebase currently reads `EguiWantsInput` or calls
`EguiContexts::ctx_mut().wants_pointer_input()`/`wants_keyboard_input()`
anywhere — confirmed by a full-repo grep. Every map-input system (`render.rs`
zoom/pan, `input.rs` click handlers) reads raw `ButtonInput`/`MessageReader`
events unconditionally.

---

## 🔨 Suggested Implementation

1. Add `.run_if(not(egui_wants_any_pointer_input))` (or an equivalent inline
   `Res<EguiWantsInput>` check) to `zoom_camera`, `pan_camera`, and the three
   click-action systems in `input.rs`.
2. For `Tab`: try gating `toggle_notebook`'s own handling behind
   `!wants_keyboard_input()` first (simplest); if egui still grabs focus on
   the *same* keypress that opens the notebook (a plausible ordering issue,
   since `wants_keyboard_input()` reflects the *previous* frame until egui
   processes this frame's `Tab`), fall back to explicitly clearing egui's
   focused widget after our own toggle runs.
3. Verify live via `cargo run` for all three: scroll-over-notebook, click-
   over-notebook-while-in-Seed-mode, and repeated `Tab` presses while
   watching which HUD element visually highlights as focused.

---

## ⚠️ Constraints and Caveats

- **Style**: `render.rs`/`input.rs`/`notebook.rs` may depend on
  `bevy::render`/`bevy_egui` (they already do) — this is presentation/input
  wiring, not a `sim`/`world`/`config` change, so TECH_DESIGN.md's headless-
  determinism invariants don't apply here.
- Don't gate the *notebook's own* internal interactions (scrolling its
  species list, clicking its own buttons) — only the *map*'s systems need
  the `EguiWantsInput` check. The notebook's egui widgets already only
  receive input egui itself routes to them.
- Verify the fix doesn't regress the always-on HUD (action bar, budget
  display, etc.) — those are also egui widgets that legitimately want
  pointer input when directly interacted with; only the *map*'s systems
  should be gated, not egui's own widget handling.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/091-egui-input-capture-gating.md)"$'\n\nExecute this task in the current project.'
```
