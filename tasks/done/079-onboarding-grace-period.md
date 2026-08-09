# Task 079 — Onboarding: adaptive grace period + softened first-world objective

> **ID**: `079`
> **Category**: Feature / Balance / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (design discussion held right after task 077 closed)

---

## 🎯 Objective

Playtesting surfaced that a run gives the player no room to acclimate:
pressure (era-budget countdown, objective failure, total-extinction risk) is
live from tick 0 of world 0. `player_guide.md`'s own "A note on balance"
section already names the risk this causes ("the whole ecosystem dies before
you can learn anything") as exactly the kind of feedback expected during
tuning. A live playtest during this session also showed a fresh world 0
opening with *"Sable (species 2) survives in the toxic zone"* as its first
objective — a demanding, specific goal on a totally undeciphered biochemistry.

Two changes, decided together in a design session (full reasoning in
`/Users/biagioliberto/.claude/plans/rosy-snuggling-lighthouse.md`):

- **Part A — Adaptive grace period.** A fixed-length grace window alone isn't
  enough: if the player's first seed dies in era 1, a timer-only window still
  ends with an empty grid and nothing observed, then fails outright the
  instant it expires (a "cliff"). Grace must be *adaptive*: total-extinction
  failure is suppressed while `world.era < grace_eras` **or** the player has
  never yet kept a population alive for a full era since this world began.
  Once that "foothold" is reached once, it's sticky — a later extinction
  fails normally, no second grace. The era-budget-exhaustion failure path is
  intentionally left untouched (a `grace_eras` of a handful is always far
  smaller than `era_budget`'s 45-60, so gating that check would be dead code)
  — a player who never seeds anything still eventually fails via the existing
  budget-exhaustion path, no extra safety cap needed.
- **Part B — Softened first-world objective.** World 0's opening objective is
  forced to `Coexistence` with `min_species` hardcoded to `2`, instead of the
  usual severity-scaled, species-pool-clamped value (which can reach 3 and
  force coexistence with every generated species, including a Decomposer —
  one of the harder metabolisms to keep alive on a first try).

Both changes are scoped to the very start of the loop (grace: every world's
opening eras; opening objective: world 0 only) — neither touches the
difficulty curve, the matrix generator, or tick coefficients themselves.

---

## 📋 Acceptance Criteria

- [x] `TimeConfig` (`src/config.rs`) gains `grace_eras: u32` (default `3`),
      mirrored by hand into `assets/config/sim_config.ron`'s `time: (...)`
      block (no automated sync check exists — documented manual convention,
      `config.rs:562`'s comment).
- [x] `src/objectives.rs` gains a `GraceProgress` resource
      (`consecutive_alive_ticks`, `foothold_reached`) and two pure functions:
      `update_grace_progress` (advances/resets the streak, no-ops once
      `foothold_reached`) and `is_grace_active(world_era, grace_eras,
      &GraceProgress) -> bool`.
- [x] `evaluate_world` gains a `grace_active: bool` parameter; only the
      total-extinction branch is gated by it — the era-budget-exhaustion
      check is untouched.
- [x] `ObjectiveOutcomeParams`/`apply_tick_outcome` wire `GraceProgress`
      through: update it every tick, compute `grace_active`, pass it into
      `evaluate_world`. `ObjectivesPlugin` registers the new resource.
- [x] `run_flow.rs`'s `start_world` (both `retry_world` and
      `advance_to_next_world` paths) resets `GraceProgress::default()`, same
      pattern as the existing `ObjectiveProgress` reset.
- [x] HUD (`src/ui.rs`) shows a line while `is_grace_active(...)` is true
      (new `text::GRACE_PERIOD_LINE` constant, no countdown number since the
      window is adaptive, not fixed-length).
- [x] `src/worldgen.rs`'s `generate_objectives` gains a `world_index: u32`
      parameter; when `world_index == 0` and the world has ≥2 species, slot 0
      is forced to `Objective::Coexistence { min_species: 2, ticks:
      scale_severity(coexistence_ticks_base, severity) }` instead of the
      normal random draw. `build_world` threads its existing `world_index`
      through. The forced kind still feeds `previous_kind`, so slot 1's
      existing no-repeat-kind rule is unaffected.
- [x] `abiogenesis-gdd.md` updated: a bullet under §8's failure conditions
      for the adaptive grace period, a `grace_eras` row in §5.9, and a note
      under §9 about world 0's forced opening objective.
- [x] `player_guide.md`'s "Objectives, victory, and failure" section and
      `src/text.rs`'s `HOW_TO_PLAY_SECTIONS` "Objectives and failure" entry
      both get a short sentence documenting the grace period.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean. New unit tests
      per the plan (grace suppresses extinction failure when active; cliff
      extension when the grid is still empty past `grace_eras`; stickiness
      of `foothold_reached`; world 0's first objective is deterministically
      `Coexistence{min_species: 2, ..}` across a spread of seeds; existing
      `evaluate_world`/`generate_objectives` call sites updated with the new
      parameter, preserving prior semantics via `false`/matching
      `world_index`).
- [x] Verified live via `cargo run`: fresh run's world 0 shows a
      `Coexistence` objective 1; a seeded organism dying does not fail the
      world and the grace HUD line stays visible; after keeping a population
      alive a full era the grace line disappears and a later full wipeout
      fails the world normally. (Confirmed working directly by the user on
      their own `cargo run`, not just headlessly.)

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` | `GraceProgress`, `update_grace_progress`, `is_grace_active`, `evaluate_world`'s new gate, `ObjectiveOutcomeParams`/`apply_tick_outcome` wiring, `ObjectivesPlugin` registration. |
| `src/worldgen.rs` | `generate_objectives`'s new `world_index` param and the world-0 forced-opening-objective branch; `build_world` threading. |
| `src/config.rs` + `assets/config/sim_config.ron` | New `TimeConfig::grace_eras` field, kept in sync by hand. |
| `src/run_flow.rs` | `WorldResetParams`/`start_world` resetting `GraceProgress` on every world (re)start. |
| `src/ui.rs` + `src/text.rs` | HUD grace-period line. |
| `abiogenesis-gdd.md`, `player_guide.md` | Documentation of the new mechanic (GDD is source of truth per `CLAUDE.md`). |

---

## 🧩 Technical Context

- **Current behavior**: `evaluate_world` (`objectives.rs:199-222`) checks
  total-extinction unconditionally every tick, `apply_tick_outcome`
  (`objectives.rs:357-419`) turns a `Failed(TotalExtinction)` outcome
  straight into `GameState::WorldFailed`. `generate_objectives`
  (`worldgen.rs:250-263`) picks every slot's kind randomly from whatever
  candidates are coherent for that world.
- **Desired behavior**: see Objective above — total-extinction failure is
  suppressed adaptively at the start of every world; world 0's first
  objective is always the gentlest possible `Coexistence`.
- Full design reasoning, rejected alternatives, and the exact code excerpts
  this was scoped against live in the approved plan at
  `/Users/biagioliberto/.claude/plans/rosy-snuggling-lighthouse.md` — read it
  before starting, it supersedes this file's summary if anything is unclear.

---

## 🔨 Suggested Implementation

Follow the approved plan's Part A then Part B, in order (Part A's
`GraceProgress` plumbing has no dependency on Part B, but doing them in this
order keeps `objectives.rs`'s test-suite churn — the 9 `evaluate_world` call
sites — separate from `worldgen.rs`'s test-suite churn — the 6
`generate_objectives` call sites):

1. `config.rs` + `sim_config.ron`: add `grace_eras: u32 = 3`.
2. `objectives.rs`: `GraceProgress`, `update_grace_progress`,
   `is_grace_active`, `evaluate_world`'s new parameter/gate, wiring through
   `ObjectiveOutcomeParams`/`apply_tick_outcome`, plugin registration, update
   existing tests (+`false`), add new tests.
3. `run_flow.rs`: reset `GraceProgress` on world (re)start; extend existing
   reset-assertion tests.
4. `ui.rs` + `text.rs`: HUD line.
5. `worldgen.rs`: `world_index` param, forced-opening-objective branch,
   `build_world` threading, update existing tests (index argument per the
   plan's per-test breakdown), add the new determinism test.
6. `abiogenesis-gdd.md`, `player_guide.md`, `text.rs`'s
   `HOW_TO_PLAY_SECTIONS`: documentation.
7. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
8. Live verification via `cargo run` (the `run` skill / `cliclick` +
   `screencapture`, as used for task 077).

---

## ⚠️ Constraints and Caveats

- Keep the era-budget-exhaustion failure path untouched — this was
  deliberately scoped out of grace (dead-code risk at these magnitudes), not
  an oversight.
- `update_grace_progress`'s foothold threshold reuses `config.time.era_ticks`
  directly rather than introducing a second, redundant constant — "one full
  era" is already a named quantity.
- Task 078 (heatmap blob shape correction) stays untouched at low priority —
  do not pick it up as part of this task.

---

## 🔗 Dependencies

- **Depends on**: 040/059 (the objective system and its sequencing this task
  extends).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/079-onboarding-grace-period.md)"$'\n\nExecute this task in the current project.'
```
