# Task 082 — Shorter eras during world 0's opening

> **ID**: `082`
> **Category**: Feature / Balance / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (scoped from `redesign/abiogenesis-engagement-design.md`, proposal 1.D)

---

## 🎯 Objective

World 0's first eras run at the standard `era_ticks` (`25`), the same as
every other era in the game. For a player still learning to read the system,
that's a long stretch between checkpoints. Give world 0's opening eras a
shorter tick count (e.g. `8`) — an onboarding-only exception, same spirit as
task 079's grace period, not a change to the standard pacing.

**Must be tuned jointly with task 083** (newborn incubation): once eras are
short, the era-boundary wait imposed by 083's incubation rule becomes the
dominant pacing constraint during onboarding, not the energy threshold this
task modulates. Playtest both together before finalizing either constant —
see `PROJECT_PLAN.md`'s "Onboarding & engagement" section for the full
reasoning.

---

## 📋 Acceptance Criteria

- [x] `TimeConfig` (`src/config.rs:160-181`) gains two fields mirroring the
      `grace_eras` pattern (`config.rs:180`, doc-commented, defaulted): e.g.
      `onboarding_eras: u32` (default `3`) and `onboarding_era_ticks: u32`
      (default `8`), mirrored by hand into `assets/config/sim_config.ron`'s
      `time: (...)` block (documented manual-sync convention,
      `config.rs:562`).
- [x] A helper function (e.g. in `worldgen.rs`, alongside `world_params`)
      computing the era-tick count for a given `(world_index, world.era)`:
      returns `onboarding_era_ticks` when `world_index == 0 && world.era <
      onboarding_eras`, otherwise `config.time.era_ticks`.
- [x] `start_era` and `single_tick` (`src/input.rs:80-96` and
      `src/input.rs:107-129`) call this helper instead of reading
      `config.time.era_ticks` directly at their `progress.start(...)` call
      sites (`input.rs:92`, `input.rs:128`) — both need `run_progress:
      Res<RunProgress>` and `world: Res<SimWorld>` added as system
      parameters (precedented: `RunProgress`/`SimWorld` are already
      consumed by sibling systems in the same file).
- [x] `cargo test` and `cargo clippy -- -D warnings` clean. New unit test
      confirming the helper returns `onboarding_era_ticks` only for
      `world_index == 0` within the threshold, and the standard value
      otherwise (including `world_index != 0` at any era, and `world_index
      == 0` past the threshold).
- [ ] Verified live via `cargo run`: world 0's first 3 eras visibly
      advance faster (era counter increments sooner) than era 4 onward, and
      than any era in world 1. **Skipped this session per explicit user
      instruction (no run/screenshot) — pending user's own playtest,
      ideally jointly with task 083's pacing.**

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/config.rs` + `assets/config/sim_config.ron` | New `onboarding_eras`/`onboarding_era_ticks` fields. |
| `src/worldgen.rs` | New helper computing era length for a given world/era index, alongside existing `world_params` (line 53). |
| `src/input.rs` | `start_era` (line 80-96) and `single_tick` (line 107-129) — call sites to update. |
| `src/sim.rs` | `EraProgress`/`tick_and_complete_era` (lines 405-422, 492-506) — read-only reference, no changes expected. |

---

## 🧩 Technical Context

- **Current behavior**: `era_ticks` is read fresh from `Res<SimConfig>` at
  every era start (`input.rs:92`/`128`, `progress.start(config.time.era_ticks)`)
  — not snapshotted per-world, so this task only needs to change what value
  is passed at those two call sites, not fight a cached value.
- **Desired behavior**: world 0's eras 0 through `onboarding_eras - 1` use
  `onboarding_era_ticks`; every other era (any era in any other world, or
  world 0 past the threshold) uses the standard `era_ticks`.
- `RunProgress::world_index` (`src/run.rs:17`) is already the established
  way systems detect world 0 (precedent: `input.rs:164`'s reset handler,
  and task 079's `generate_objectives(world_index: u32, ...)`).

---

## 🔨 Suggested Implementation

1. `config.rs` + `sim_config.ron`: add the two new `TimeConfig` fields.
2. `worldgen.rs`: add the era-length helper.
3. `input.rs`: thread `run_progress`/`world` into `start_era` and
   `single_tick`, call the helper at both `progress.start(...)` sites.
4. Unit tests for the helper's branching.
5. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
6. Live verification via `cargo run`, ideally in the same playtest session
   as task 083 so the combined pacing can be felt together, not just each
   constant checked in isolation.

---

## ⚠️ Constraints and Caveats

- This is an onboarding-only exception (world 0, first N eras) — do not
  generalize `era_ticks` variability beyond that scope; the standard game
  keeps a single fixed `era_ticks`.
- Numeric values (`3`, `8`) are indicative per the source design doc, not
  final — validate in playtest, ideally jointly with task 083's incubation
  rule active, before locking them in.

---

## 🔗 Dependencies

- **Depends on**: 079 (`world_index`/onboarding-exception precedent this
  follows).
- **Tune jointly with**: 083 (newborn incubation) — do not finalize either
  task's numeric constants without playtesting both together.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/082-shorter-onboarding-eras.md)"$'\n\nExecute this task in the current project.'
```
