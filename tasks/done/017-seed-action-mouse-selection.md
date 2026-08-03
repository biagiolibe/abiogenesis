# Task 017 — Seed action with mouse cell selection

> **ID**: `017`
> **Category**: UI / Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Give the player their first real intervention (GDD §6): click a cell to place an organism of a chosen species from the starting palette (task 013). This is Phase 1's player-facing milestone — everything before this task builds the simulation side of emergence; this task makes it interactive.

---

## 📋 Acceptance Criteria

- [ ] Left-click on a grid cell, while `EraState::Observing`, places an organism of the currently-selected species in that cell **if it's empty**; clicking an occupied cell does nothing (no overwrite, no stacking — single occupancy, GDD §5.1).
- [ ] Clicks are ignored during `EraState::Advancing` (consistent with the existing rule that advancement inputs are ignored mid-animation, task 007).
- [ ] Clicks landing outside the grid's rendered area (including on the HUD panel) do nothing — no out-of-bounds panics.
- [ ] The HUD (`ui.rs`) gains a minimal species selector (e.g. radio buttons) listing the palette from task 013, defaulting to the first species.
- [ ] Placing an organism uses `config.energy.seed_energy` as starting energy, matching how `seed_phase0_organism`/task 013's palette already seed organisms.
- [ ] No action budget is charged — GDD §6's point economy is explicitly Phase 2 (`EraState::Planning` stays a stub per `TECH_DESIGN.md` §2 until then).
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | New system: mouse click → grid cell → place organism |
| `src/ui.rs` | Species selector, holds the "currently selected species" as a resource egui reads/writes |
| `src/render.rs` | `cell_position()` — needs an inverse (world position → grid cell) |

---

## 🧩 Technical Context

### Screen → cell conversion

`render.rs::cell_position(x, y, width, height)` maps a grid cell to a world-space `Vec3`; this task needs the inverse. Bevy's `Camera::viewport_to_world_2d` (or equivalent in 0.19) converts a cursor position to a world-space point given the camera and its `GlobalTransform`; from there, invert `cell_position`'s formula to recover `(x, y)` and round to the nearest cell, rejecting points outside `[0, width) x [0, height)`.

Because task 008 already shrinks the camera's `Viewport` by `HUD_WIDTH` so the grid never renders under the HUD panel, `viewport_to_world_2d` naturally accounts for that offset — no extra correction needed as long as the standard Bevy camera APIs are used instead of hand-rolling the projection math.

### Where the "selected species" resource lives

A small `Resource` (e.g. `SelectedSpecies(pub SpeciesId)`), owned by `ui.rs` (it's a UI intent, not simulation state — same rationale as `EraProgress` living in `sim.rs` but being written to by `input.rs`). The egui radio-button system writes it; the mouse-click system in `input.rs` reads it.

---

## 🔨 Suggested Implementation

1. `ui.rs`: add

   ```rust
   #[derive(Resource)]
   pub struct SelectedSpecies(pub SpeciesId);
   ```

   initialized to `SpeciesId(0)`, and a small block in `hud_panel` with `ui.radio_value(&mut selected.0, SpeciesId(i as u8), format!("species {i}"))` for each `i` in `0..world.species.len()`.

2. `input.rs`: a new system, gated the same way as the other era-sensitive systems (`if *era_state.get() == EraState::Advancing { return; }`):

   ```rust
   fn seed_organism_on_click(
       buttons: Res<ButtonInput<MouseButton>>,
       windows: Query<&Window>,
       cameras: Query<(&Camera, &GlobalTransform)>,
       era_state: Res<State<EraState>>,
       selected: Res<SelectedSpecies>,
       mut world: ResMut<SimWorld>,
       config: Res<SimConfig>,
   ) {
       if *era_state.get() == EraState::Advancing { return; }
       if !buttons.just_pressed(MouseButton::Left) { return; }
       // window cursor position -> camera.viewport_to_world_2d -> grid (x, y)
       // bounds check, occupancy check, then:
       // world.get_mut(x, y).organism = Some(Organism { species: selected.0, energy: config.energy.seed_energy });
   }
   ```

3. Register `SelectedSpecies` as a resource in `UiPlugin` (or `InputPlugin`, whichever module ends up owning it) and the new system in `InputPlugin`.

4. Manual verification (this task touches rendering/input, so it needs the `run` skill / a live check per the project's UI-change convention): click an empty cell in the lit band, confirm a colored sprite of the selected species' hue appears; click an occupied cell, confirm nothing changes; switch species in the HUD radio buttons and confirm the next click uses the new selection.

---

## ⚠️ Constraints and Caveats

- **The UI/input layer only reads `SimWorld` except for this one write path** (`TECH_DESIGN.md` §3.3 scopes `Ui` as read-only; this task's write belongs to `input.rs`'s existing precedent of being the layer that turns player intent into `SimWorld` mutation, same as `reseed_world`/`single_tick`).
- Don't add stress/cull/splice in this task — GDD §6 lists them, but only `Seed` is Phase 1's action (`PROJECT_PLAN.md` scopes the rest to Phase 2).
- Don't spend action-budget points — that economy doesn't exist yet (Phase 2).
- Keep the species selector minimal (radio buttons over a flat list is enough); a richer palette browser is a Phase 2/notebook concern.

---

## 🔗 Dependencies

- **Depends on**: 013, 008
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/017-seed-action-mouse-selection.md)"$'\n\nExecute this task in the current project.'
```
