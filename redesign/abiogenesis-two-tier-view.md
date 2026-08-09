# Abiogenesis — two-tier map view (overview heatmap + detail grid)

Self-contained proposal document capturing a design discussion held right after task 074 (grid size raised to 128×80). Doesn't require reading other redesign docs, aside from the GDD for existing simulation concepts (species, metabolism, toxic zone, action economy).

## Context

Task 074 raised the grid from 48×32 (1536 cells) to 128×80 (10240 cells) to support a richer, larger world. Population dynamics and performance held up fine at that size, but a real legibility problem surfaced during the visual check: individual organism dots read as small, and some species colors (dark/muted hues) are hard to tell apart against dark terrain at this scale. A simple camera zoom control was the first idea raised, but it doesn't solve the underlying problem on its own — zoomed out, the grid is still 10240 individual dots competing for attention; zoomed in, you lose the whole-world read entirely.

This document captures a two-tier alternative worked out through direct discussion, to be split into implementation task files once approved.

## Decision

**Two discrete rendering modes on a single continuous-zoom camera**, not two separate camera/window systems:

1. **Overview** (default) — the whole grid, but individual organisms are not drawn as dots. Instead, occupied cells are grouped into **per-species clusters** (connected components of contiguous same-species cells, not a fixed grid of equal-sized blocks), and each cluster renders as a **density heatmap blob**: hue = the cluster's species color, brightness/intensity = local population density within it, shape/extent = the cluster's real spatial footprint (so a population that has spread out in one direction reads as an elongated blob, not a uniform square). Terrain (elevation bands, boundaries, peak glyphs, toxic-zone outline — tasks 066–072) keeps rendering exactly as today underneath; only the organism layer changes representation. An isolated single organism is simply its own one-cell cluster — it stays visibly distinct rather than getting absorbed into an aggregated tile the way a fixed-grid scheme would risk.
2. **Detail** — the current per-cell rendering (individual organism dots/shapes, exactly as today).

**Input: mouse-wheel zoom, centered on the cursor position** (standard map-style zoom — scrolling in over a point keeps that point under the cursor), a single camera with continuous zoom. The **Detail viewport is whatever's in the camera frustum** at the time — no separate fixed-size "magnifier window" system. The switch between Overview and Detail rendering is a **hard threshold on zoom level**, not a continuous blend: cross the threshold and the renderer swaps from heatmap blobs to individual organism sprites for whatever region is currently in frame. Panning is just normal camera movement at any zoom level — no separate mechanic needed.

### Actions and mode

- **Stress** and **Cull** require **Detail** mode. Both target an existing organism/cell precisely (Stress shifts one cell's temperature; Cull removes a specific organism) — that precision doesn't survive aggregation, so these stay disabled (or simply inert) in Overview.
- **Seed** and **Splice** remain available in **both** modes. They're creation actions, not precision-targeting actions — placing a new organism or modifying a species' tags doesn't inherently need pixel-level cell selection the way culling or stressing an existing one does. Placement always resolves to one exact cell (the sim has no concept of an approximate cell); when used from Overview, that exact cell gets a brief transient on-screen indicator (same spirit as task 054's first-confirmed-hypothesis celebration) so the player sees where it landed without needing to zoom in themselves — the resulting cluster blob (see above) then carries that visibility forward on its own.

## Notes for implementation (not blocking, but worth deciding deliberately rather than by accident)

- **Zoom threshold value.** Needs an actual number, tuned by feel against the 128×80 grid (task 074) — not derived analytically here.
- **Cluster computation cost.** Connected-component clustering across up to ~10000+ cells needs to not run every frame; recomputing only on population-changing events (tick advance, action resolution) rather than per-render-frame is the obvious lever if it shows up as a cost.
- **Overlapping/interleaved species clusters.** Two species can occupy adjacent or interleaved cells in the same area; since clustering is per-species (not per fixed block), this becomes two overlapping/adjacent blobs rather than a single tile needing a color tie-break — simpler than the fixed-grid version of this problem, but the exact blending/z-ordering when blobs visually overlap still needs a rendering pass.

## Out of scope for this document

- The actual aggregation/rendering implementation (block-size math, shader vs. sprite-batch approach, camera state machine) — this is a design decision record, not an implementation spec.
- Any change to the underlying simulation grid, action economy, or species/matrix mechanics — this is a rendering/camera/input concern layered on top of the existing sim, same boundary `render.rs`/`input.rs` already keep from `sim`/`world`/`config` (TECH_DESIGN.md §5).
- The deferred always-on temperature/light background tint idea (raised separately during the 062/066-072 sessions) — unrelated, still parked on its own.
