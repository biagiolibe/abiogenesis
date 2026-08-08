# Task 062 — Procedural alien-world background layer

> **ID**: `062`
> **Category**: UI / Feature
> **Priority**: 🟢 P3
> **Estimate**: ~3-4h
> **Assigned to**: unassigned
> **Session**: 2026-08-08 (raised directly by the user: the grid reads as an empty black background outside occupied/tinted cells)

---

## 🎯 Objective

Empty cells already scale their brightness by `light` (`cell_color`,
`src/render.rs:525-538`: `Color::hsl(0.0, 0.0, 0.03 + cell.light * 0.12)`),
but the range is compressed enough (lightness `0.03`→`0.15`) that the map
reads as a flat black background. The user wants a low-cost way to make it
feel like a credible alien world without competing with the grid's
readability.

**Decided direction** (discussed and explicitly scoped as an exception to
GDD pillar 3 — "the fun is in the system, not the graphics" — because the
goal here is atmosphere, not information):

- A **procedurally generated background layer**, rendered *behind* the grid
  sprites — no hand-painted art, no art pipeline. Same spirit as the
  existing procedural shape-mask system for organisms
  (`shape_mask_image`, `src/render.rs:395-417`), generating an `Image` from
  code rather than loading an asset file.
- It must stay **dim/low-contrast** relative to the grid — the grid sprites
  are the primary readability layer and must not be visually challenged.
- It must **never carry gameplay signal** a player could mistake for real
  data (temperature/light readouts, toxicity, matrix hints) — this is
  cosmetic noise only, uncorrelated with anything the player needs to
  reason about.
- **Interesting option to explore, not a hard requirement**: derive the
  background's variant (hue/intensity/pattern) from that world's own
  `WorldParams` (`src/worldgen.rs:21`) so each world reads as visually
  distinct, reusing data the worldgen already produces instead of inventing
  new parameters. If this turns out to add complexity disproportionate to
  the payoff, a single fixed procedural look for every world is an
  acceptable first pass — note the decision either way.
- **Explicitly out of scope for this task**: parallax/camera-driven motion
  (pairs naturally with the still-open camera zoom/pan proposal, but is not
  a prerequisite — ship a static version first) and hand-illustrated biome
  art (real production cost, revisit only if the procedural version proves
  the atmosphere is worth it).

---

## 📋 Acceptance Criteria

- [x] A new background `Image` is generated procedurally (noise/gradient
      field over normalized coordinates, same technique as
      `shape_mask_image`'s coverage-predicate pattern but producing colored,
      not just alpha-masked, pixels) and spawned as a single large `Sprite`
      covering at least the camera's visible area at the grid's `AutoMin`
      scaling (`spawn_camera`, `src/render.rs:343-358`).
- [x] The background sprite is guaranteed to render behind every grid cell
      sprite — e.g. `Transform::from_xyz(0.0, 0.0, -1.0)` (grid cells spawn
      at whatever z `cell_position` uses today, `src/render.rs:498`; confirm
      it's `0.0` and give the background a strictly lower z) rather than
      relying on spawn order.
- [x] Regeneration on world change: `SimWorld::seed` (`src/world.rs:130`)
      changes every time `start_world` (`src/run_flow.rs:61`) builds a new
      world, but `start_world` mutates `SimWorld` in place rather than
      re-inserting the resource, so hook a system that tracks the
      last-generated seed in a `Local<Option<u64>>` and regenerates the
      background texture when `world.seed` differs from it — same reactive
      style `sync_grid_colors` already uses against `SimWorld`, no changes
      needed to `run_flow.rs`'s call sites.
- [x] If deriving the variant from `WorldParams`: pick 1-2 concrete params to
      map to the background (e.g. temperature-gradient spread → hue,
      toxic-zone size → noise density) and document the mapping in a doc
      comment; if skipping this for a fixed look instead, say so explicitly
      in the PR/commit description.
- [x] Visual check (manual, `cargo run` + screenshots, verified with a
      temporary constants bump to make the effect unambiguous, then
      reverted): the sprite renders correctly, strictly behind the grid, and
      `sync_background` re-generates it live on `r`-reseed. **Caveat**: at
      the grid's own 3:2 aspect ratio the sprite is fully occluded by the
      grid's opaque, exactly-tiled cell sprites — it's only visible in the
      `AutoMin` letterbox margin a non-3:2 window shows past the grid's
      edge (thin at a maximized widescreen window, near-zero at a
      3:2-cropped one). It does not reach the map's interior "empty black
      background" the task's motivation described; doing that would mean
      partial alpha on empty cells, which is cell rendering, explicitly out
      of this task's contained scope. Organism colors/shapes/toxicity tint
      are untouched either way.
- [x] `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt -- --check`
      clean; add a unit test for the pure generation function (e.g. "given a
      seed, produces a deterministic image") if the generation logic is
      non-trivial enough to warrant one, following `shape_mask_image`'s
      precedent of not needing one (it's tested implicitly via the shapes it
      produces being visually stable, not unit-tested directly) — use
      judgment.
- [x] `PROJECT_PLAN.md`'s "Atmospheric background layer" proposal entry
      updated/removed once this lands.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `GridRenderPlugin::build`, `spawn_camera`, `spawn_grid`, `cell_position`, `shape_mask_image` (pattern to follow), `MetabolismShapes` (pattern for a `Resource` holding generated image handles) — everything lands here. |
| `src/world.rs` | `SimWorld::seed` — the change-detection key for regeneration. |
| `src/worldgen.rs` | `WorldParams` — optional source for background variant mapping. |
| `src/run_flow.rs` | `start_world` — confirms the in-place mutation pattern that rules out `OnEnter`/resource-insertion hooks. |
| `PROJECT_PLAN.md` | §1 "Atmospheric background layer" — the proposal this task implements. |

---

## 🧩 Technical Context

- **Current behavior**: no background layer exists; empty cells' own sprite
  color is the only thing giving any sense of environment, and its range is
  too compressed to read clearly.
- **Desired behavior**: a dim, procedurally generated, seed-varying visual
  layer sits behind the grid at all times, purely atmospheric.
- Bevy 2D z-ordering: sprites at the same z tie-break by spawn order in
  practice, but an explicit lower z is the reliable way to guarantee
  back-to-front ordering — don't rely on spawn order alone.

---

## ⚠️ Constraints and Caveats

- **Pillar 3 exception, scoped narrowly**: this is the one place in the
  codebase deliberately allowed to add a non-"colored squares" visual
  element — keep it contained to this background layer, don't let it creep
  into organism/cell rendering itself.
- **No magic numbers for anything that affects gameplay** — this doesn't
  apply here since nothing here touches `SimConfig`/tick logic, but any
  tunable visual constant (dimness, noise scale) should still be named
  constants, not scattered literals, for the next person tuning it.
- **Determinism**: if deriving the background from `world.seed`, use it (or
  a value derived from it) as the RNG source for the generation, not
  wall-clock time — keeps the same world always looking the same, matching
  GDD §5.7.
- **Don't** make the background sprite intercept mouse input — `input.rs`'s
  cell-click handling (`Seed`/`Stress`/`Cull`/`Splice`) must keep working
  exactly as before; a full-screen sprite behind everything should be inert
  for `Sense`/picking purposes (it doesn't use `egui`, so this is likely a
  non-issue, but verify).

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: none (pairs well with the still-open camera zoom/pan proposal
  in `PROJECT_PLAN.md` §1, but doesn't require it)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/062-procedural-background-layer.md)"$'\n\nExecute this task in the current project.'
```
