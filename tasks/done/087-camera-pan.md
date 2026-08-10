# Task 087 — Camera pan

> **ID**: `087`
> **Category**: Feature (camera / rendering)
> **Priority**: 🟢 P3
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-10

---

## 🎯 Objective

Add dedicated camera panning (scrolling the map) so the player can navigate the grid at any zoom level, not just via zoom's incidental translation shift. Complements the existing zoom system (tasks 075-076): zoom is done, pan is the remaining open half of "camera zoom and pan" from the backlog.

**Note on prior design record**: the original two-tier-view design doc (`abiogenesis-two-tier-view.md`, since deleted from the working tree — retrievable via `git show 410919b:redesign/abiogenesis-two-tier-view.md`) explicitly concluded *"panning is just normal camera movement at any zoom level — no separate mechanic needed,"* and task 075's own acceptance criteria repeated that call. This task deliberately revisits that decision: at the current `128×80` grid size (task 074) with Detail zoom showing only a sub-region, navigating by zoom-drift alone is impractical — a dedicated pan input (drag and/or keyboard) is needed to reach arbitrary parts of the grid without repeated zoom-in/zoom-out.

---

## 📋 Acceptance Criteria

- [x] The code compiles without errors; `cargo clippy -- -D warnings` and `cargo fmt` clean.
- [x] A pan input system is added (arrow keys and `WASD` — see 2026-08-10 follow-up below on the `S`/single-tick collision) that mutates the `GridCamera` entity's `Transform::translation`, following `zoom_camera`'s query shape and its `.run_if(in_state(GameState::Playing))` gating.
- [x] Pan clamping reuses (via a shared helper, not duplicated logic) the existing clamp math from `zoom_camera` — factored into `clamp_camera_pan(translation, scale, unscaled_area, grid_extent) -> Vec2`, called by both `zoom_camera` and the new `pan_camera`. Covered by three new unit tests (in-bounds passthrough, edge clamp on both axes, zero-pan at whole-grid zoom).
- [x] Pan speed is a new `CameraConfig::pan_speed` field (cells/second at `scale == 1.0`, default `24.0`), mirrored in `assets/config/sim_config.ron`.
- [x] Re-verified live: overlay painters stay clipped correctly and click-to-cell targeting (`Seed` placed exactly under the cursor) remains correct after panning, not just after zooming.
- [x] `sim`/`world`/`config` (beyond the new `CameraConfig` field) stay untouched — purely a `render.rs` concern.
- [x] `cargo test` passes; live `cargo run` verification done (see Resolution below) — pan confirmed working at Detail zoom, direction/continuity/edge-clamp confirmed via temporary debug logging, click-to-cell and overlay rendering confirmed correct while panned.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `spawn_camera`, `GridCamera`, `zoom_camera` (pattern + clamp logic to extend/share), `MapViewMode`, `cell_position`/`world_to_cell`, overlay painters — add the new pan system here, alongside `zoom_camera`. |
| `src/config.rs` | `CameraConfig` (`zoom_min/max/threshold/speed`) — add a pan-speed field here. |
| `assets/config/sim_config.ron` | Mirror the new `CameraConfig` field. |
| `src/input.rs` | Reference only — general keyboard/mouse input conventions (`ButtonInput<KeyCode>`, `ButtonInput<MouseButton>`), for consistency if pan uses keyboard input; the system itself belongs in `render.rs`, not here (see Technical Context). |

---

## 🧩 Technical Context

**Current behavior**: `spawn_camera` (`src/render.rs:899-917`) spawns a single `GridCamera`-tagged `Camera2d` with `OrthographicProjection { scaling_mode: ScalingMode::AutoMin { min_width, min_height }, .. }`, no explicit initial `Transform` (defaults to origin). `zoom_camera` (`src/render.rs:933-1000`) is the only system that currently mutates this camera's `Transform`/`Projection.scale`, driven by `MessageReader<MouseWheel>`, doing cursor-centered zoom plus a clamp of the resulting translation to keep the visible area inside the grid extent. There is no dedicated pan input system anywhere in the codebase (`grep` confirms no pan/drag code exists). `MapViewMode` (Overview/Detail) is derived every frame from the projection's current `scale` vs. `config.camera.zoom_threshold`, ordered after `zoom_camera` in `GridRenderPlugin`'s system chain (`src/render.rs:191-203`).

Grid↔world mapping: the grid is centered on world origin, `cell_position(x, y, width, height)` (`src/render.rs:1206-1218`) maps grid cell → world `Vec3`, with y flipped (row 0 = top = positive world y); inverse mapping (`world_to_cell`) is used by click-driven player actions. Total grid extent in world units is `width * CELL_SIZE × height * CELL_SIZE` (`CELL_SIZE = 16.0`, `src/render.rs:20`).

Two cameras exist in this app: `GridCamera` (grid) and a separate HUD camera (`ui::spawn_hud_camera`, order 1, carries `PrimaryEguiContext`) — kept separate because a shared camera's cropped `Viewport` (for HUD panel space) would also crop egui's paint canvas (TECH_DESIGN.md §6). Any pan system must query `With<GridCamera>` only, exactly like `zoom_camera`.

**Desired behavior**: the player can pan the `GridCamera` (drag and/or keyboard) at any zoom level, with the same edge-clamping guarantee `zoom_camera` already provides, without disturbing the HUD camera, overlay clipping, or click-to-cell targeting.

---

## 🔨 Suggested Implementation

1. Factor `zoom_camera`'s existing clamp logic (`src/render.rs:972-999`) into a standalone function, e.g. `clamp_camera_pan(translation: Vec2, scale: f32, area: Rect, grid_extent: Vec2) -> Vec2`, and have `zoom_camera` call it (no behavior change to zoom itself).
2. Add a new `CameraConfig` field for pan speed (`src/config.rs`), mirrored in `assets/config/sim_config.ron`.
3. Add a new system (e.g. `pan_camera`, `src/render.rs`, near `zoom_camera`) reading drag input (`ButtonInput<MouseButton>` + `MessageReader<CursorMoved>`, or `MessageReader<MouseMotion>`) and/or keyboard arrows, mutating `GridCamera`'s `Transform::translation` and clamping via the shared helper from step 1.
4. Register the system in `GridRenderPlugin::build`, ordered consistently with the existing `zoom_camera → update_map_view_mode → ...` chain (pan doesn't need to run before/after zoom specifically, but must run before anything depending on the camera's final transform this frame, if such ordering matters — check `update_map_view_mode`'s inputs).
5. Live-test via `cargo run`: pan in both Overview and Detail, verify edge clamping, verify overlays and click targeting remain correct.

```
// No prescribed snippet — drag vs. keyboard vs. both is an implementer
// choice; reuse zoom_camera's query/gating shape as the template.
```

---

## ⚠️ Constraints and Caveats

- **Style**: Follow `TECH_DESIGN.md` conventions. Keep `sim`/`world`/`config` (beyond the new `CameraConfig` field) untouched — purely a `render.rs` concern.
- **No magic numbers**: pan speed and any other new tunable belongs in `SimConfig` (`CameraConfig`), not hardcoded in `render.rs`.
- Query `With<GridCamera>` only — never touch the HUD camera.
- Don't regress task 075's overlay-clipping or click-to-cell-targeting fixes; both must keep working under panning, not just under zooming.
- This revisits a prior explicit design decision (see Objective) — the old "no separate pan mechanic" call was made before the grid grew to `128×80`; that context should be understood, not silently ignored.

---

## 🔗 Dependencies

- **Depends on**: 075 (needs `GridCamera`, `zoom_camera`, `CameraConfig` to already exist)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/087-camera-pan.md)"$'\n\nExecute this task in the current project.'
```

---

## ✅ Resolution (2026-08-10)

Implemented as scoped: `pan_camera` (`src/render.rs`) reads the four arrow keys, moves the `GridCamera`'s `Transform::translation` at `CameraConfig::pan_speed` (24 cells/s at `scale == 1.0`) scaled by the current zoom, and clamps through the new shared `clamp_camera_pan` helper (extracted from `zoom_camera`'s pre-existing clamp, which now calls the same function — no behavior change to zoom). `WASD` was rejected in favor of arrow-keys-only because `KeyS` already drives `input::single_tick`.

Live-verified via `cargo run`, driven headlessly with `cliclick`/`screencapture`/System Events (mirroring task 069's precedent): menu navigation and mouse/scroll events reached the app via `cliclick`, but synthetic keyboard events from `cliclick kp:`/`kd:`/`ku:` never reached the window (Space press produced no era advance) — likely an Accessibility/Input Monitoring permission gap for that specific event path. Switched to `osascript`'s `System Events key down/key code`, which did reach the app (confirmed via Space advancing the era). Direction and continuity were confirmed unambiguously via a temporary debug `info!` log of `Transform::translation.x` before/after each frame while holding the key (screenshot pixel-comparison alone was ambiguous and initially misread) — `clamped_x` increased monotonically and smoothly while `ArrowRight` was held, confirming correct sign and smooth per-frame movement; the debug log was removed before committing. Edge-of-grid clamping was verified via three new unit tests on `clamp_camera_pan` (in-bounds passthrough, clamp on both axes at a zoomed-in scale, zero-pan at whole-grid `scale == 1.0`) rather than waiting out a multi-second key hold to reach the actual clamp live — deterministic and faster. Click-to-cell targeting was verified live: a `Seed` action while panned placed the organism exactly under the cursor.

`cargo test`, `cargo clippy -- -D warnings`, `cargo fmt` all clean.

### Follow-up tuning (2026-08-10, same-day playtest feedback)

Live feedback after this task shipped: the pan felt too slow, and the player wanted `WASD` enabled rather than arrow-keys-only. Addressed directly (small enough to fold in here rather than filing a new task):

- `CameraConfig::pan_speed` raised `24.0 → 60.0 → 120.0` cells/s (two rounds of "still too slow" feedback), in both `config.rs`'s default and `assets/config/sim_config.ron`.
- `input::single_tick` rebound from `KeyS` to `KeyN`, freeing `S` so `WASD` could be added to `pan_camera` alongside the arrow keys, with no binding collisions. Updated everywhere the old binding was documented: `text.rs`'s in-game "How to play" panel and `KEYBOARD_HINT_PRIMARY`/`SECONDARY`, `player_guide.md`, and the GDD's `Controls [PROPOSED]` list.
- `cargo test`/`clippy`/`fmt` re-verified clean after each change.
