# Task 006 — Grid rendering with sprites + 2D camera

> **ID**: `006`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Make the grid **visible**: one square sprite per cell, colored according to simulation state, with a 2D camera framing it all.

Without this task the simulation runs blind. Phase 0's milestone (GDD §13) is literally *"watch a photolithic species bloom and stabilize"*: this task builds the "watch."

---

## 📋 Acceptance Criteria

- [ ] `cargo run` shows a `48×32` grid of colored squares.
- [ ] Sprites are spawned **once**, in `Startup`; subsequent ticks only update color.
- [ ] **Occupied cells**: color = species, brightness = energy.
- [ ] **Empty cells**: faint background proportional to `light`, so environmental gradients are visible at a glance.
- [ ] **Residue** is visually distinguishable from both empty and occupied cells.
- [ ] The grid stays centered and fully visible when resizing the window.
- [ ] **No rendering system writes to `SimWorld`** (only `Res`, never `ResMut`).
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `GridRenderPlugin`, sprite spawning, camera, synchronization |
| `src/world.rs` | `SimWorld` read-only (task 003) |

---

## 🧩 Technical Context

- **Current behavior**: the window opens empty; the world exists in memory but isn't visible.
- **Desired behavior**: the grid is visible and reflects simulation state.

### GDD §11 — Presentation

> - **Rendering:** 2D window. Grid of cells as colored squares.
>   - Occupied cells: color = species/tag; brightness = energy.
>   - Empty cells: faint background reflecting the environment (e.g., brightness = `light`).

### Architecture: entities only for rendering

`TECH_DESIGN.md` §3.1 establishes that the grid is **not** modeled in ECS: state lives in `SimWorld`. The sprite entities created here are a **view**, not the source of truth. They carry a `GridCell { x, y }` component linking them to the corresponding cell.

1536 sprites (48×32) is a trivial load for Bevy.

---

## 🔨 Suggested Implementation

1. **Linking component**

   ```rust
   /// Links a rendered sprite back to its cell in SimWorld. The sprite is a view:
   /// simulation state lives in the resource, never here (TECH_DESIGN 3.1).
   #[derive(Component)]
   struct GridCell {
       x: usize,
       y: usize,
   }
   ```

2. **Spawn in `Startup`** — one sprite per cell, `custom_size` matching the cell's side length, positioned on a grid centered at the origin. Remember that in Bevy `y` grows upward while the row index grows downward: row `0` (high light) must appear **at the top**, so the world's `y` needs to be inverted.

3. **Camera** — `Camera2d`. To make sure the grid always fits in the window, use a fixed-size projection (`ScalingMode::AutoMin` or the 0.19 equivalent) sized to `width * cell_size` × `height * cell_size`. This way resizing never clips the grid.

4. **Sync system** — runs in `SimSet::Sync`, after `SimSet::Advance`:

   ```rust
   fn sync_grid_colors(
       world: Res<SimWorld>,              // read-only: never ResMut here
       mut cells: Query<(&GridCell, &mut Sprite)>,
   ) {
       for (cell, mut sprite) in &mut cells {
           sprite.color = cell_color(&world, cell.x, cell.y);
       }
   }
   ```

5. **Color scheme** — a single rule in a single place:

   | Cell state | Color |
   |---|---|
   | Occupied | species hue, brightness scaled by energy |
   | Has residue, empty | desaturated neutral hue, intensity proportional to residue |
   | Empty | very dark gray, brightness proportional to `light` |

   For energy, normalize against `repro_threshold` (`10.0`) and **clamp**: energy can exceed it in the tick right before reproduction. Working in HSL/HSV makes "same hue, variable brightness" natural.

   Keep the empty-cell background low-key: it should suggest the light gradient without competing with organisms.

6. **Manual test**: run the app, seed an organism into `SimWorld` by hand (or wait for task 007) and check it shows up. The light gradient should be perceptible as a vertical shading on the empty grid.

---

## ⚠️ Constraints and Caveats

- **Rendering is read-only.** If a system in `render.rs` requests `ResMut<SimWorld>`, the architecture has been violated.
- **Do not spawn/despawn sprites every tick**: create them once, update the color. Despawning for an empty cell would cost more than the rendering itself.
- **No art assets** (GDD, pillar 3): white sprites colored via `Sprite::color`, no textures to load.
- Rendering **tags as glyphs** (GDD §11) is Phase 1+: here color alone identifies the species.
- Check the Bevy 0.19 API for `Sprite`/`Camera2d`: 0.19 is recent and names differ from the earlier 0.1x versions found in most online examples.

---

## 🔗 Dependencies

- **Depends on**: 003
- **Blocks**: 007

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/006-grid-rendering.md)"$'\n\nExecute this task in the current project.'
```
