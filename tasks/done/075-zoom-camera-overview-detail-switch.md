# Task 075 — Zoom camera and Overview/Detail render-mode switch

> **ID**: `075`
> **Category**: Feature / Rendering
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-09

---

## 🎯 Objective

Task 074 raised the grid to 128×80 and surfaced a real legibility gap: at that
size, individual organism dots are small and some species colors are hard to
tell apart. A design discussion (2026-08-09, full record in
`redesign/abiogenesis-two-tier-view.md`) settled on a two-tier view — an
aggregated **Overview** and the current per-cell **Detail** rendering — driven
by a single continuous-zoom camera rather than two separate camera/window
systems.

This task builds the foundational piece both other tasks in this batch depend
on: mouse-wheel zoom centered on the cursor, and a hard-threshold switch
between two render modes (`Overview` / `Detail`) as a resource other systems
can read. It does **not** implement the Overview cluster-heatmap itself (task
076) or the action gating (task 077) — those consume the mode resource this
task introduces, but ship their own rendering/gating logic separately.

---

## 📋 Acceptance Criteria

- [x] `spawn_camera`'s fixed `ScalingMode::AutoMin` (`src/render.rs:577`) is
      replaced with a projection that supports zooming — scrolling the mouse
      wheel changes zoom level, centered on the cursor's world position (the
      point under the cursor stays under the cursor as you zoom, standard
      map-zoom behavior).
- [x] A new resource (e.g. `MapViewMode { Overview, Detail }`) tracks the
      current mode, derived from the camera's current zoom level against a
      configured threshold (`SimConfig`/RON — no magic number). Crossing the
      threshold flips the mode; there is no interpolated/blended state.
- [x] The mode is observable by other systems (`Res<MapViewMode>` or
      equivalent) — this task doesn't need to consume it itself beyond maybe
      a debug indicator, but 076/077 will.
- [x] Zoom range is bounded sensibly: can't zoom out past seeing the whole
      grid (the current `AutoMin` behavior becomes the zoomed-out floor, not
      something you can zoom past into empty space), and can't zoom in
      absurdly far past single-cell resolution.
- [x] Panning: at any zoom level, the visible region is just whatever the
      camera frustum currently shows — no separate "detail window" pan
      mechanic. Confirm existing click-to-cell mapping
      (`world_to_cell`, `src/render.rs:741`) still resolves correctly once
      the camera can be zoomed and (if in scope) panned — it currently
      assumes a camera at a fixed transform.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [x] Verified live via `cargo run`: zooming in with the mouse wheel crosses
      the threshold and the mode resource flips (a debug print or temporary
      on-screen label is fine for verification, doesn't need to ship).

---

## ✅ Resolution (2026-08-09)

Implemented as designed: `CameraConfig` (`zoom_min`/`zoom_max`/`zoom_threshold`/
`zoom_speed`) added to `SimConfig`, mouse-wheel zoom centered on the cursor
via the standard "keep the point under the cursor fixed" translation formula
derived from `OrthographicProjection::area`'s dependence on `scale`, and a
`MapViewMode` resource synced from the current scale against the threshold.

Two real bugs surfaced during live playtesting on this machine, both fixed:

1. **Pan drift at the zoomed-out floor.** The cursor-centered zoom formula
   alone doesn't guarantee the camera returns to translation `(0, 0)` when
   scale returns to `zoom_max` — any residual pan from an earlier zoom-in
   persisted, so at "fully zoomed out" the grid was no longer centered and
   part of it fell outside the viewport (reported as "the map shifts and
   scrolls under the sidebar"). Fixed with a general pan clamp: on any axis
   where the visible world-space span at the current scale is `>=` the
   grid's own extent, the only valid translation is `0`; otherwise it's
   bounded to keep the viewport fully inside the grid. Derived from
   `ortho.area`'s existing pre-mutation value (`area.size() / old_scale`
   recovers `AutoMin`'s unscaled projection dimensions) rather than
   duplicating `AutoMin`'s own width/height math.
2. **Terrain overlay bleeding into the sidebar.** `terrain_overlay::
   draw_terrain_overlay` and the debug-only `energy_overlay` project world
   positions via `Camera::world_to_viewport`, which doesn't clip to the
   camera's actual (HUD-cropped) viewport bounds — invisible at the old
   fixed `scale = 1.0` (the whole grid always projected inside the cropped
   viewport by construction), it became visible once zoom let players view
   a sub-region: an off-frame cell's projected pixel position could land
   inside the sidebar's screen area, drawing boundary/toxic-zone lines
   there. Fixed by clipping both painters to `Camera::logical_viewport_rect()`.

Both fixes verified live via `cargo run` on the user's machine after each
fix (zoom-in/zoom-out round trips, Seed-click precision while zoomed).
`cargo test`/`clippy -- -D warnings` clean; `cargo fmt` applied.

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `spawn_camera` (camera/projection setup), `GridCamera` marker component, `world_to_cell`/`cell_position` (screen↔grid coordinate mapping, currently assumes the fixed `AutoMin` transform). |
| `src/config.rs` | Where the zoom threshold (and zoom bounds) belong as `SimConfig` fields — likely a new small config struct or additions to an existing rendering-adjacent one. |
| `assets/config/sim_config.ron` | Keep in sync with whatever `src/config.rs` default is added (task 073's hot-reload convention). |
| `redesign/abiogenesis-two-tier-view.md` | Full design record — read this first, it has the reasoning behind every decision referenced above. |

---

## 🧩 Technical Context

- **Current behavior**: `spawn_camera` (`src/render.rs:577`) sets a fixed
  `ScalingMode::AutoMin { min_width, min_height }` sized to exactly fit the
  whole grid — there is no zoom, and the camera transform never changes
  after spawn. `world_to_cell`/`cell_position` (`src/render.rs:732-743`)
  convert between world-space and grid coordinates assuming this fixed
  scale.
- **Desired behavior**: the camera supports continuous zoom via mouse wheel,
  centered on the cursor. A `MapViewMode` resource (or similarly-named)
  reflects whether the current zoom is above or below a configured
  threshold. `AutoMin`'s current fit-the-whole-grid framing becomes the
  zoomed-out floor rather than the only state.
- Rendering itself (what actually gets drawn in each mode) is explicitly
  **out of scope here** — this task only needs the camera and the mode
  resource to exist and be correct. Task 076 will read `MapViewMode` to
  decide whether to draw heatmap blobs or the existing per-cell sprites;
  task 077 will read it to gate Stress/Cull.

---

## 🔨 Suggested Implementation

1. Read `redesign/abiogenesis-two-tier-view.md` in full first.
2. Add zoom-threshold (and zoom min/max bound) fields to `SimConfig`,
   defaulted in both `src/config.rs` and `assets/config/sim_config.ron`.
3. Add mouse-wheel input handling to adjust the camera's
   `OrthographicProjection` scale, keeping the point under the cursor fixed
   (standard "zoom toward cursor" transform math).
4. Introduce the `MapViewMode` resource and a system that updates it from
   the current camera scale vs. the configured threshold.
5. Re-verify `world_to_cell`/`cell_position` against a zoomed, possibly
   panned camera — these likely need to read the camera's actual transform
   instead of assuming the original fixed `AutoMin` framing.
6. Bound zoom range so it can't go past the whole-grid view outward or past
   single-cell resolution inward.

---

## ⚠️ Constraints and Caveats

- **This task ships no visible behavior change to the organism layer** —
  Overview's heatmap rendering is task 076's job. It's fine (and expected)
  for the grid to keep rendering exactly as it does today at every zoom
  level after this task, gated only by the new (otherwise-unused) mode
  resource.
- **No magic numbers**: zoom threshold and bounds are `SimConfig` fields.
- Keep `sim`/`world`/`config` untouched — this is purely a `render.rs`
  concern (TECH_DESIGN.md §5, rendering must not depend back on anything
  the headless sim needs).

---

## 🔗 Dependencies

- **Depends on**: 074 (the 128×80 grid size that motivated this).
- **Blocks**: 076 (Overview cluster-heatmap rendering), 077 (action gating
  by view mode) — both need `MapViewMode` to exist first.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/075-zoom-camera-overview-detail-switch.md)"$'\n\nExecute this task in the current project.'
```
