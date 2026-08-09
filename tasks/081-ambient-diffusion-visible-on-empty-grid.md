# Task 081 — The world breathes: subtle ambient tint on unoccupied cells

> **ID**: `081`
> **Category**: Feature / Presentation / Onboarding
> **Priority**: 🟢 P3
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (scoped from `redesign/abiogenesis-engagement-design.md`, proposal 1.E)

---

## 🎯 Objective

Environmental diffusion (GDD §5.2) already runs unconditionally every tick,
including on a completely empty grid before the player's first `Seed` —
`SimWorld::diffuse_environment` (`src/world.rs:439`) has no gating on
population. But nothing shows it: unoccupied, residue-free cells render as a
flat, static `terrain_color(cell.terrain)` (`src/render.rs:1113`) — task 066
deliberately removed continuous `light`-based shading from the base map. The
data the whole diagnosis in the design doc leans on ("the world has its own
chemistry independent of the player") already exists and is already
computing; it's just never drawn.

Add a very subtle tint to unoccupied cells derived from their diffusing
`temperature`/`light` values, without reverting task 066's decision or
touching occupied-cell rendering at all.

---

## 📋 Acceptance Criteria

- [ ] A new tunable `SimConfig` coefficient (e.g.
      `visual.ambient_tint_strength: f32`, default small, e.g. `0.08`) caps
      how much the tint can shift the base `terrain_color` — no magic
      numbers per `CLAUDE.md`.
- [ ] `cell_color`'s unoccupied/residue-free branch (`src/render.rs:1113`)
      blends `terrain_color(cell.terrain)` with a slight shift derived from
      `cell.temperature`/`cell.light`, scaled by
      `visual.ambient_tint_strength`, instead of returning the flat color
      unmodified.
- [ ] The shift stays clearly below the existing `T`/`L` overlay's
      legibility (`apply_environment_overlay`, `src/render.rs:131`) — this
      is ambience, not a replacement for the opt-in overlay's precision.
      Verify visually side-by-side.
- [ ] Occupied-cell rendering and the residue-tint branch are untouched.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: on a fresh world 0 before any seed,
      the map shows a faint, slowly-shifting tint instead of a fully static
      terrain color; pressing `T`/`L` still shows the existing, more legible
      overlay on top/instead as today.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `cell_color` (line ~1071-1116), specifically the unoccupied-cell fallback at line 1113. |
| `src/world.rs` | `diffuse_environment` (line 439) — read-only reference, no changes expected here. |
| `src/config.rs` + `assets/config/sim_config.ron` | New `ambient_tint_strength` coefficient. |

---

## 🧩 Technical Context

- **Current behavior**: `diffuse_environment` computes and updates
  `temperature`/`light` every tick regardless of population
  (`src/sim.rs:135` calls it unconditionally inside `step`). `cell_color`
  ignores these values for empty cells and returns a static
  `terrain_color(cell.terrain)` (task 066 explicitly chose this over
  continuous shading).
- **Desired behavior**: empty cells still read as visually calm (pillar 3:
  "the fun is in the system, not the graphics" — no new assets, no loud
  effect) but show a faint, moving signal that the environment has its own
  independent state.
- Related but distinct open proposal already in SECTION 1 ("Extend the
  procedural background layer (task 062) into the map's interior") — that
  one is about a background *sprite layer* showing through partially
  transparent empty cells; this task is about *tinting* the existing flat
  color with diffusion data. Don't conflate the two while implementing.

---

## 🔨 Suggested Implementation

1. `config.rs` + `sim_config.ron`: add `ambient_tint_strength` under
   whichever config struct governs presentation coefficients (create one if
   none exists yet for visual-only tuning).
2. `render.rs`: in `cell_color`'s unoccupied branch, compute a small color
   shift from `cell.temperature`/`cell.light` (e.g. a slight hue/brightness
   nudge) scaled by `ambient_tint_strength`, blended into
   `terrain_color(cell.terrain)`.
3. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
4. Live verification via `cargo run` on a fresh world 0, comparing against
   the `T`/`L` overlay.

---

## ⚠️ Constraints and Caveats

- Keep the shift small enough that it doesn't reopen task 066's decision —
  if it starts looking like continuous shading again, the strength constant
  is too high; tune down, don't redesign.
- No new assets, no shader work — a color-math blend in the existing
  `cell_color` function is sufficient (pillar 3).

---

## 🔗 Dependencies

- **Depends on**: 016 (environmental diffusion), 066 (terrain-color
  rendering this modifies).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/081-ambient-diffusion-visible-on-empty-grid.md)"$'\n\nExecute this task in the current project.'
```
