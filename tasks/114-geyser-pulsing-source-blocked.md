# Task 114 — Geyser biome (BLOCKED)

> **ID**: `114`
> **Category**: Feature (worldgen + rendering)
> **Priority**: 🟢 P3
> **Estimate**: ~2h (once unblocked)
> **Assigned to**: unassigned — **do not start**, see Dependencies
> **Session**: 2026-08-11 (scoped from `redesign/processed/abiogenesis-biomes.md`)

---

> **Status (governed-sdd)**: QUEUED (blocked) &nbsp;·&nbsp; **Review**: REQUIRED &nbsp;·&nbsp; **Reasoning**: medium
> **Authority**: `redesign/processed/abiogenesis-biomes.md` (Design source) + `TECH_DESIGN.md` §5
> **Expected code surface / Out of scope / Validation**: see 📁 Relevant Files, 🔗 Dependencies (this task must not start before its prerequisite lands), and Acceptance Criteria below.

---

## 🎯 Objective

> **This task is scoped for reference but blocked from starting.** Geyser is
> defined in the design doc as a small (1-2 cell), pulsing point source — mechanically
> distinct from Bocca vulcanica, which is a strong, constant source. Task 085
> (`tasks/done/085-source-driven-temperature-and-light.md`) already implemented point
> heat sources (`SimWorld.heat_sources`, `place_heat_sources`, `world.rs:559-604`,
> `reinject_environment_sources`, `world.rs:703`), but **every source it produces is
> mechanically identical** — same strength, same constant reinjection, no notion of a
> smaller or time-varying source. There is no code today that could make Geyser read
> as anything other than "Bocca vulcanica with a different name" — which would violate
> this document's own rendering constraint ("ogni colore deve rappresentare un dato
> reale"). Do not pick this up until a second, smaller/pulsing heat-source category
> exists (a separate, not-yet-scoped task — likely an extension of task 085's source
> model, possibly connected to the "ambient diffusion visible on empty grid" thread,
> `tasks/081-ambient-diffusion-visible-on-empty-grid.md`, currently on hold).

Once that prerequisite lands, this task assigns the Geyser biome to the small/pulsing
source category's cells (same pattern as task 111's Bocca vulcanica hookup to
`heat_sources`), and adds the pulsing render behavior — the design doc explicitly
calls Geyser out as the first candidate for making the environment "breathe" visually
even on an otherwise-empty grid.

---

## 📋 Acceptance Criteria (once unblocked)

- [ ] Geyser cells identified from the new small/pulsing source category (not from
      `heat_sources` — that's Bocca vulcanica, task 111), footprint 1-2 cells per the
      design doc.
- [ ] `Biome::Geyser` variant added (task 110's `Biome` enum), target
      `temperature`/`light`/`toxicity` values from the table in
      `redesign/processed/abiogenesis-biomes.md`.
- [ ] Rendering: pulsing/animated treatment, distinct from every other (static)
      biome — this is the one place in the biome system where a color legitimately
      changes over time, and it must still read as "a real data pulse," not
      decoration.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test` passes.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:200, 559-604, 703` | `heat_sources`, `place_heat_sources`, `reinject_environment_sources` — the existing (constant-only) source model this task's prerequisite must extend. |
| `tasks/111-biome-feature-placement.md` | Bocca vulcanica's hookup to `heat_sources` — the pattern to mirror once a small/pulsing source category exists. |
| `tasks/081-ambient-diffusion-visible-on-empty-grid.md` | Related "world breathes" thread, currently on hold — possible connection point. |

---

## 🔗 Dependencies

- **Depends on**: a not-yet-scoped extension of task 085's source model (small,
  time-varying heat source category). Also depends on 110/111 (`Biome` enum and the
  feature-placement pattern must exist).
- **Blocks**: none.
