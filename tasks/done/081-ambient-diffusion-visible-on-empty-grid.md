# Task 081 — The world breathes: toxic zone pulse + diffusion drift check

> **ID**: `081`
> **Category**: Feature / Presentation / Onboarding
> **Priority**: 🟢 P3
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (scoped from `redesign/abiogenesis-engagement-design.md`,
> proposal 1.E — rescoped down after discussion, see below)

---

## 🎯 Objective

Source proposal 1.E's exact wording: *"un gradiente che si muove
impercettibilmente, una zona tossica che pulsa piano"* (a gradient that
moves imperceptibly, a toxic zone that pulses slowly) — two **localized,
animated** cues, not a uniform tint painted over every empty cell. An
earlier scoping pass for this task proposed exactly that uniform tint
(blending `terrain_color` toward `heat_color(temperature)` on every
unoccupied cell); on review that was rejected as reading like "filling
every empty cell with a tint" rather than "a few things here and there,"
which is what the source doc actually asked for. Rescoped to two much
smaller, literal pieces:

1. **Toxic zone pulses** — animate `toxicity_tint`'s blend strength with a
   slow oscillation over time, instead of the current fixed `0.45`. Reads
   directly as "the toxic zone pulses slowly," and stays localized by
   construction (only visible where `toxicity > 0`).
2. **Gradient movement check** — `diffuse_environment` (`src/world.rs:439`)
   already runs unconditionally every tick, eroding the initial
   temperature/light gradients over time (a real, slow spatial change). No
   new rendering needed *if* that drift is actually perceptible within a
   normal run's timeframe — verify it is, on the base map, without opening
   the `T`/`L` overlay. If it isn't perceptible at default `diffusion_rate`
   (`0.05`), this task file's job is to say so, not to invent a new visual
   effect to compensate — the broader idea of making the environment feel
   dynamic-by-design is picked up separately in
   `redesign/abiogenesis-environment-sources.md`, not here.

---

## 📋 Acceptance Criteria

- [x] `toxicity_tint` (`src/render.rs`) takes a time input (`Res<Time>` read
      in `sync_grid_colors`, threaded through `cell_color` as `elapsed:
      f32`) and oscillates its blend strength slowly around the current
      `0.45` (`0.45 * (0.85 + 0.15 * (elapsed * PULSE_FREQUENCY).sin())`,
      `PULSE_FREQUENCY = 0.4` rad/s, a ~16s period) — exact amplitude/
      frequency are a first-pass guess, tune visually.
- [x] The pulse is visible only on cells with `toxicity > 0` — `strength`
      is `toxicity.clamp(0.0, 1.0) * 0.45 * (...)`, so at `toxicity == 0`
      the blend stays exactly `0` regardless of the oscillation term.
- [x] No changes to `terrain_color`, the empty-cell branch, or any other
      part of `cell_color` — only `toxicity_tint`'s call and body changed.
- [x] Diffusion drift-perceptibility check (code inspection, not a live
      run — see note below): **not perceptible, and structurally so, not
      just small in magnitude.** `diffuse_environment` only ever writes
      `Cell::temperature`/`Cell::light`/`Cell::toxicity`; the base map's
      empty-cell color comes from `Cell::biome` via `dithered_biome_color`
      (`cell_color`'s empty branch), and `Cell::biome` is assigned once by
      `classify_biomes` during world generation — nothing re-runs it, or
      otherwise reads `temperature`/`light` into any non-overlay render
      path, per tick. So the eroding gradient has zero effect on what the
      base map shows: this isn't "erosion too slow to notice," it's
      "nothing currently displays it." Confirmed by grepping every
      `cell.temperature`/`cell.light` read in `render.rs`: the only hit is
      the dev-only `EnvironmentOverlay::Temperature` (`F1`-style overlay),
      not the base map. Making this visible would require new rendering
      (the "environment feel alive-by-design" idea already carved out to
      `redesign/abiogenesis-environment-sources.md`), so out of scope here
      per this task's own framing.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Live verification skipped for this pass at the user's explicit
      request — not run via `cargo run`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `toxicity_tint` (line 1150-1153) — add the time-based oscillation. |
| `src/world.rs` | `diffuse_environment` (line 439) — read-only reference for the drift-perceptibility check. |

---

## 🧩 Technical Context

- **Current behavior**: `toxicity_tint(base, toxicity)` blends toward a
  fixed warning hue at a constant `toxicity.clamp(0.0, 1.0) * 0.45` strength
  — static, no time dependency.
- **Desired behavior**: that strength oscillates slowly over real time, so
  the toxic zone visibly "breathes" without changing its average intensity
  much (the oscillation should be small enough that the zone doesn't flicker
  distractingly or misrepresent the underlying `toxicity` value).
- Rejected direction (kept here for record, do not resurrect without a new
  discussion): a uniform ambient tint blended into every unoccupied cell's
  `terrain_color` from `cell.temperature`/`cell.light`. The bigger idea
  behind wanting the environment to feel alive-by-design, not just tinted,
  is now its own design doc,
  `redesign/abiogenesis-environment-sources.md` — read it before proposing
  any further empty-cell tinting work, to avoid duplicating that effort.

---

## 🔨 Suggested Implementation

1. `render.rs`: thread a time value into `toxicity_tint` (or compute the
   oscillating strength at its call site and pass it in instead of the
   fixed `0.45`).
2. Tune amplitude/frequency visually via `cargo run`.
3. Separately, run the game for several real-time minutes on a fresh world
   with no overlay active, and note whether the base terrain colors visibly
   shift due to diffusion. Record the finding in this file's acceptance
   checklist or the commit message.
4. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- Do not touch `terrain_color` or the empty-cell branch of `cell_color` —
  that idea was explicitly rejected for this task; see Technical Context.
- Keep the oscillation subtle — this is ambience, not a new gameplay signal
  (toxicity's mechanical effect on the tick, or lack thereof, is unrelated
  and out of scope here, same caveat `toxicity_tint`'s own doc comment
  already makes).

---

## 🔗 Dependencies

- **Depends on**: 033 (`toxicity_tint`'s origin), 072 (toxic-zone placement
  this pulses).
- **Related, not a dependency**: `redesign/abiogenesis-environment-sources.md`
  — a separate, larger environment-model redesign raised during this task's
  scoping; not required for 081, but read it before extending 081's scope.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/081-ambient-diffusion-visible-on-empty-grid.md)"$'\n\nExecute this task in the current project.'
```
