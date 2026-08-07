# Task 059 — Sequential per-world objectives (2 → 3 across the difficulty curve)

> **ID**: `059`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~4-5h (touches worldgen, objective evaluation, HUD, run-flow, and several existing tests)
> **Assigned to**: unassigned
> **Session**: 2026-08-07 (design follow-up to the same-day playtest that opened this task as a proposal)

---

## 🎯 Objective

A world currently has exactly one objective (`CurrentObjective(Option<Objective>)`,
`src/objectives.rs`); clearing it immediately ends the world (`GameState::
WorldCleared`). A playtester found this ends worlds too fast, before much of
the hidden matrix or the species pool gets exercised — confirmed as a
deliberate MVP scope cut (task 042) rather than a bug, but one the user wants
revisited now.

**Decided direction** (discussed directly with the user, superseding this
task's original open-questions list): worlds get **multiple objectives in
sequence**, not a single one. Clearing objective *i* doesn't end the world —
it advances to objective *i+1* with fresh progress tracking; only clearing the
*last* objective in the sequence triggers `WorldCleared`. The count scales with
difficulty, same ramp mechanism `WorldParams`'s other fields already use:
**2 objectives** at the early endpoint, **3** at the late endpoint
(`DifficultyConfig::ramp_worlds`, currently `3`).

Because late worlds now need to clear more objectives, not just harder ones,
`TimeConfig`'s era budget must grow to compensate (also discussed and decided
directly): `era_budget_early` **40 → 60**, `era_budget_late` **25 → 45**. This
keeps a similar per-objective time allowance to today while the difficulty
curve still tightens it as worlds progress (60/2 = 30 eras/objective early,
45/3 = 15 eras/objective late — still a real squeeze, just proportionally
similar to today's 40/1 vs. 25/1). These are starting values, not sacred —
`tests/balance.rs` already exists to catch anything that ends up too easy/hard
in practice; retune there if a balance test fails or a playthrough disagrees.

---

## 📋 Acceptance Criteria

- [ ] `WorldParams` (`src/worldgen.rs:21-41`) gains an `objective_count: u32`
      field, computed in `world_params` the same way every other field ramps
      (`lerp_u32`/`ramp_fraction`), from a new `DifficultyConfig` pair
      (`objective_count_early: u32 = 2`, `objective_count_late: u32 = 3`).
- [ ] `TimeConfig::era_budget_early`/`era_budget_late` defaults updated to `60`/
      `45` (currently `40`/`25`, `src/config.rs:107-108`).
- [ ] `worldgen::generate_objective` becomes `generate_objectives`, returning
      `Vec<Objective>` of length `params.objective_count`, reusing the existing
      per-slot selection/parametrization logic in a loop. No two **consecutive**
      objectives share the same `ObjectiveKind` (pick, then exclude that kind
      from the next slot's candidates) — otherwise the existing coherence rules
      (species-count/toxic-zone gating) are unchanged.
- [ ] `CurrentObjective` (`src/objectives.rs:88-92`) becomes a small struct
      holding the sequence and an index (e.g. `{ objectives: Vec<Objective>,
      index: usize }`) with a `current(&self) -> Option<&Objective>` accessor,
      replacing the old bare `Option<Objective>` at every call site
      (`build_world`'s return type, `run_flow.rs`, `ui.rs`'s objective panel,
      tests).
- [ ] `apply_tick_outcome` (`src/objectives.rs:294-337`): on the current
      objective's `WorldOutcome::Cleared`, if it's not the last one in the
      sequence, advance the index, reset `ObjectiveProgress` to `default()`,
      and stay `Ongoing` (no `GameState` transition) instead of firing
      `WorldCleared`. Only the *last* objective clearing transitions to
      `WorldCleared`, same as today.
- [ ] Advancing to the next objective emits a Bevy `Message` (e.g.
      `ObjectiveAdvanced { index: usize, objective: Objective }`, mirroring
      `SpeciesExtinct`/`OrganismDied`'s existing pattern) rather than
      `objectives.rs` reaching into `ObservationLog` directly — keeps the
      module boundary `notebook.rs` already relies on (`record_events`
      consumes `sim.rs`'s messages the same way). `notebook.rs::record_events`
      reads it and appends a `LogEntry` announcing the newly-cleared/next
      objective.
- [ ] HUD's objective panel (`src/ui.rs`'s `objective_panel` /
      `text::HEADING_OBJECTIVE`) shows "Objective i/N" alongside the current
      objective's description and progress bar, so the player knows there's
      more than one and how far along they are.
- [ ] `player_guide.md`'s "Objectives and failure" section gets a short update
      noting a world can pose more than one objective in sequence.
- [ ] Existing tests updated for the `Vec<Objective>`/`CurrentObjective` shape
      change (`objective_generation_is_deterministic_for_the_same_seed`,
      `survive_in_toxic_zone_is_never_chosen_without_a_toxic_zone`,
      `objective_species_reference_is_always_within_the_generated_pool`, and
      `objectives.rs`'s own test suite) plus new tests for: `objective_count`
      ramping (2 at `world_index = 0`, 3 at/after `ramp_worlds`), no two
      consecutive same-kind picks, and a full sequence clearing only
      transitions to `WorldCleared` on the last objective (not earlier ones).
- [ ] `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt
      -- --check` all clean, `tests/balance.rs` still green (retune the new
      era-budget/objective-count defaults if it isn't).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` | `Objective`, `CurrentObjective`, `ObjectiveProgress`, `evaluate`/`evaluate_world`/`apply_tick_outcome` — the core structural change (single → sequence). |
| `src/worldgen.rs` | `WorldParams`/`world_params` (new `objective_count` field), `generate_objective` → `generate_objectives`, `build_world`'s return type. |
| `src/config.rs` | `DifficultyConfig` (new `objective_count_early`/`_late`), `TimeConfig` (`era_budget_early`/`_late` retuned). |
| `src/ui.rs` | `objective_panel` — needs the "i/N" progress display. |
| `src/notebook.rs` | `record_events` — new `ObjectiveAdvanced` message consumer, `LogEntry` for the transition. |
| `src/run_flow.rs` | Wherever `build_world`'s objective return value is threaded into `CurrentObjective` — update for the new type. |
| `player_guide.md` | "Objectives and failure" section. |
| `tasks/done/042-worldgen-objective-generation.md` | Prior art: explicit MVP cut on multi-objective worlds — this task deliberately revisits that call. |
| `tasks/done/049-objectives-in-eras-not-ticks.md` | Prior art: the last pacing retune (bases only, not structure). |

---

## 🧩 Technical Context

- **Current behavior**: one `Objective` per world; clearing it transitions
  straight to `GameState::WorldCleared`; `era_budget` ramps 40 → 25 across
  `ramp_worlds` (`config.rs:107-108`).
- **Desired behavior**: 2-3 objectives per world (ramped like every other
  `WorldParams` field), cleared in sequence; `era_budget` ramps 60 → 45 to
  compensate; the HUD and notebook log make the multi-objective structure
  visible to the player, not just to the underlying state machine.
- `evaluate_sustained`/`evaluate`/`population_of`/`count_coexisting_species`/
  `species_present_in_zone` (the per-`Objective`-variant condition checks) are
  unaffected — they still evaluate exactly one `Objective` at a time; only the
  *what happens after it clears* logic changes.

---

## ⚠️ Constraints and Caveats

- **Determinism**: `generate_objectives`' sequence must stay a pure function of
  the world's own seeded RNG stream, same as today's single-objective
  generation (no wall-clock/external randomness).
- **No magic numbers**: `objective_count_early`/`_late` and the retuned era
  budgets belong in `SimConfig` (`DifficultyConfig`/`TimeConfig`), not
  hardcoded in `worldgen.rs`.
- **Style**: player-facing strings (objective panel "i/N", the new log message)
  behind `src/text.rs` (task 034's convention).
- **Don't touch**: `ObjectiveConfig`'s base thresholds
  (`coexistence_ticks_base`, `survive_in_ticks_base`,
  `trigger_bloom_population_threshold_base`) — this task's scope is the
  *count* and *sequencing* of objectives and the budget to match, not
  retuning individual objective difficulty (that's task 049's territory,
  already visited once; a further retune, if still needed after this lands,
  is a separate follow-up).

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/059-objective-pacing-design.md)"$'\n\nExecute this task in the current project.'
```
