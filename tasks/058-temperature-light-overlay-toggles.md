# Task 058 — Player-facing temperature/light overlay toggles

> **ID**: `058`
> **Category**: UI
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-07 (design discussion, same session as task 057)

---

## 🎯 Objective

Temperature has no visible representation anywhere in normal play (only in the
dev-only `F1` debug cycle, `render.rs`'s `debug_view` module), and light is only
a very faint always-on background shading (`cell_color`, `Color::hsl(0.0, 0.0,
0.03 + cell.light * 0.12)`) with no legend. The player currently has no reliable
way to read either scalar while placing/experimenting.

Rejected alternative: always-on background tints for both scalars (like
`toxicity_tint`, task 033). Discarded because (a) stacking a temperature hue tint
on top of light's lightness and toxicity's magenta tint on the same empty-cell
background gets visually noisy, and (b) the game's environment design intends to
move from static linear gradients (GDD §5.2 Phase 0: light top→bottom, temperature
left→right) toward possibly-randomized zones/patches in future worldgen — a fixed
spatial legend (e.g. edge arrows) would mislead the player once zones aren't
simple axis gradients anymore.

Decided approach: promote the existing per-cell heatmap logic (`debug_view`'s
`heat_color`) to two independent, player-facing **toggle keys** — one for
temperature, one for light — each shown as a heatmap overlay covering the whole
grid on demand, replacing the current F1 dev-only cycling behavior for these two
scalars specifically. This works for any spatial pattern (gradient or patch)
since it's computed per-cell from the live value, and stays out of the way when
off.

Note: this is *not* the same category as the hidden interaction matrix (GDD §7,
§11's deduction pillar) — `temperature` and `light` are known/declared Phase-0
environmental scalars, not hidden game state. Task 033 already established the
precedent of making an environmental scalar (`toxicity`) visible during normal
play; this task extends the same principle to the other two.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors and `cargo clippy -- -D warnings` is clean.
- [ ] Two independent key bindings during `GameState::Playing`, each toggling
      (press to show, press again to hide) a full-grid heatmap overlay for one
      scalar: temperature and light. Suggested keys: `T` for temperature, `L`
      for light — confirm neither collides with existing bindings (`Space`,
      `KeyS`/`KeyR` in `input.rs`, `Tab` in `notebook.rs`, `F1`/`F2` dev-only).
- [ ] Activating one overlay deactivates the other (mutually exclusive) — showing
      both scalars as blended tints on the same cells at once is not legible;
      only one heatmap is shown at a time.
- [ ] The overlay reuses (or extracts to a shared, non-`#[cfg(debug_assertions)]`
      location) the existing `heat_color` blue→red mapping from `debug_view`, so
      the visual language matches what a developer already sees in `F1` — no new
      color scale invented.
- [ ] `F1`'s dev-only cycle keeps `Temperature`/`Light` (still convenient for
      cross-checking against `Toxicity`, which stays debug-only since it's
      already visible via `toxicity_tint` in normal play) OR is trimmed to just
      `Toxicity`/`Normal` if the two views become fully redundant with the new
      toggles — pick whichever avoids duplicated logic; note the decision in a
      comment.
- [ ] A small always-visible legend/hint (e.g. in the HUD's keyboard-hint area,
      `text::KEYBOARD_HINT`) tells the player the toggle keys exist — this is a
      new player-facing affordance, not a hidden debug feature, so it must be
      discoverable without reading source.
- [ ] The overlay is gated on `GameState::Playing` like the rest of the grid
      rendering, and does not run in menus/transition screens.
- [ ] Existing tests pass; if `heat_color` is extracted to a shared module, add a
      unit test pinning its endpoints (0.0 → blue-ish hue, 1.0 → red-ish hue).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `debug_view` module (`heat_color`, `DebugView`, `F1` cycling) to reuse/extract from; `cell_color`/`toxicity_tint` for the existing always-on tint precedent (task 033). |
| `src/input.rs` | Existing key-binding patterns (`KeyCode::KeyS`, `KeyCode::KeyR`, `Space`, `Escape`) to follow for the new `T`/`L` toggles. |
| `src/ui.rs` | `text::KEYBOARD_HINT` display area (~line 337) — add the new toggle keys to the discoverability hint. |
| `src/text.rs` | Player-facing string(s) for the new hint text. |

---

## 🧩 Technical Context

- **Current behavior**: `debug_view` (⁠`#[cfg(debug_assertions)]`-gated) cycles
  `Normal → Temperature → Toxicity → Light → Normal` on `F1`, overwriting
  `Sprite::color` for every cell with `heat_color(scalar)` when active. This
  never ships in release builds and is undiscoverable to players.
- **Desired behavior**: two new resources/systems, always compiled (not
  `debug_assertions`-gated), each a simple on/off `bool` resource toggled by its
  own key, applied the same way `debug_view::apply_debug_view` does today
  (overwrite `Sprite::color` after `sync_grid_colors`), but scoped to exactly one
  scalar each and mutually exclusive with each other.
- `heat_color` is currently private to the `debug_view` module; it needs to
  become reachable from the new non-debug-only code (either moved to `render.rs`
  top level, or duplicated with a comment explaining why — prefer moving/reuse
  per CLAUDE.md's no-duplication spirit).

---

## 🔨 Suggested Implementation

1. Move `heat_color` out of the `#[cfg(debug_assertions)]` `debug_view` module
   into `render.rs`'s top level (it's a pure, presentation-only function with no
   dependency on debug-only state).
2. Add an `EnvironmentOverlay` resource (e.g. `enum EnvironmentOverlay { Off,
   Temperature, Light }`, `#[derive(Resource, Default)]` with `Off` default).
3. Add a toggle system: `T` sets it to `Temperature` (or back to `Off` if already
   `Temperature`); `L` does the same for `Light` — pressing one while the other
   is active switches directly (mutually exclusive, no double-press needed).
4. Add an apply system mirroring `apply_debug_view`'s shape, gated on
   `GameState::Playing`, run after `sync_grid_colors`.
5. Decide whether `debug_view`'s `F1` cycle keeps `Temperature`/`Light` (harmless
   overlap, useful for a developer who wants both scalars at once via a single
   key while iterating) or drops them to reduce duplication — document the
   choice inline.
6. Add the new keys to `text::KEYBOARD_HINT` (or a new constant appended to the
   same HUD hint area) so the toggle is discoverable in-game.
7. Run `cargo test` and `cargo clippy -- -D warnings`.

---

## ⚠️ Constraints and Caveats

- **Style**: player-facing strings behind `src/text.rs` (task 034); coefficients
  (if any new ones are introduced, e.g. overlay opacity) belong in `SimConfig`,
  not hardcoded — though this overlay likely needs none, since it fully replaces
  the cell color rather than blending it.
- **Determinism**: presentation-only, no changes to `sim`/`world`/`config` tick
  logic.
- **Scope**: `toxicity` is explicitly out of scope here — it already has
  always-on visibility (task 033) and doesn't need a toggle.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/058-temperature-light-overlay-toggles.md)"$'\n\nExecute this task in the current project.'
```
