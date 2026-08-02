# Task 008 — `bevy_egui` HUD

> **ID**: `008`
> **Category**: UI
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Add a panel showing simulation state as numbers: tick, era, population, average energy, seed, available commands.

Serves two purposes. For the player, to **read** the ecosystem beyond the grid's visual impression — GDD §14 flags readability as the project's second risk. For the developer, to see immediately whether tuning is heading where it should.

---

## 📋 Acceptance Criteria

- [ ] `EguiPlugin` is registered and a panel is visible on startup.
- [ ] The panel shows: **tick**, **era**, **population per species**, **average energy**, **seed**, **current state** (`EraState`).
- [ ] Shows **command hints** (`space`, `s`, `r`, `Esc`).
- [ ] Values match `SimWorld` and update during era animation.
- [ ] The panel **doesn't cover the grid**: the camera should be adjusted or the panel positioned accordingly.
- [ ] Game input keeps working while the pointer is over the panel.
- [ ] **No writes to `SimWorld`** from UI systems.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `UiPlugin`, HUD panel |
| `src/main.rs` | `EguiPlugin` registration |

---

## 🧩 Technical Context

- **Current behavior**: the grid is visible and eras advance, but no numeric data appears on screen. `bevy_egui` is a declared dependency (task 001) but not yet registered.
- **Desired behavior**: an always-visible HUD.

### GDD §11

> **UI panels:** current tick, era number, populations per species, average energy, current objective, action budget, command hints.

Objective and action budget belong to Phase 3: here the space is set up, without inventing their values.

### Why egui

`TECH_DESIGN.md` §1 chooses `bevy_egui` for the UI. The real reason arrives in Phase 2: the notebook (GDD §7) is a **dense, interactive `tag × tag` hypothesis grid**, the use case where immediate-mode UI is clearly better suited than persistent-widget UI. The HUD is the first proving ground for that choice.

---

## 🔨 Suggested Implementation

1. **Register the plugin** in `main.rs`, before `UiPlugin`. Check the signature required by `bevy_egui` 0.41 (in recent versions `EguiPlugin` has configuration fields: `EguiPlugin::default()` is the starting point).

2. **Side panel**, so it doesn't overlap the grid:

   ```rust
   fn hud_panel(
       mut contexts: EguiContexts,
       world: Res<SimWorld>,               // read-only
       state: Res<State<EraState>>,
   ) {
       egui::SidePanel::right("hud").show(contexts.ctx_mut(), |ui| {
           ui.heading("Abiogenesis");
           ui.label(format!("Era {}  ·  tick {}", world.era, world.tick));
           // ...
       });
   }
   ```

3. **Statistics** — computed by reading the grid. At 1536 cells the cost is negligible every frame; if it ever mattered, the computation would move into `SimSet::Advance` and be cached in a resource.

   ```rust
   /// Population and mean energy per species, computed from the grid.
   fn species_stats(world: &SimWorld) -> Vec<(SpeciesId, usize, f32)>
   ```

   **Average energy** should be computed only over living organisms (denominator = population, not number of cells) — otherwise it always looks like it's in free fall.

4. **Space reserved for future phases.** Sections present but inactive, so evolving the UI doesn't require redesigning the panel:

   ```rust
   // Placeholder: objective and action budget arrive in Phase 3 (GDD 8, 6).
   ```

5. **Command hints** at the bottom, in muted text: `space` era · `s` tick · `r` reseed · `Esc` quit.

6. **Input coexistence.** Egui captures keyboard and mouse when one of its widgets has focus. With plain `label`s this doesn't happen, but when text fields arrive in Phase 2, game input will need to check `ctx.wants_keyboard_input()` first. If this task introduces an interactive widget, handle it now.

---

## ⚠️ Constraints and Caveats

- **The UI only reads** (`TECH_DESIGN.md` §3.3): `Res<SimWorld>`, never `ResMut`.
- **Don't duplicate state in a UI resource.** The HUD computes from the grid every frame; a cached copy would be a second source of truth to keep in sync.
- **Don't invent the world's objective**: that's Phase 3 (GDD §8). Just a placeholder here.
- Watch the version pairing: `bevy_egui` 0.41 ↔ bevy 0.19 ↔ egui 0.35. The `EguiContexts` API has changed several times between nearby versions — use the 0.41 documentation, not the first examples you find.
- Keep the panel **compact**: the grid is the star, the HUD is support.

---

## 🔗 Dependencies

- **Depends on**: 007
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/008-hud-egui.md)"$'\n\nExecute this task in the current project.'
```
