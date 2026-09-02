# Task 192 — Key the starting-species unlock off cumulative understanding, not worlds cleared

Priority: 🟡 P2
Status: QUEUED
Review: REQUIRED
Dependencies: 158 (needs its cumulative-aggregates tracking as the source of truth)
Reasoning: medium

## Authority

- `abiogenesis-gdd.md` §10 ("Meta-progression"), design note: "Unlocks should
  key off cumulative *understanding* (relationships confirmed, speciations
  induced, biomes and wild species met) rather than objectives cleared, so
  that exploring is rewarded over rushing — and those are exactly the
  figures the end-of-run summary already reports." Plus the 2026-09-02
  `[DECIDED]` clarification added directly below it, recording this gap.

## Goal

Found during a live UI review of task 191 (main-menu presentation): the
main menu's `NO_UNLOCKS_YET`/`unlocks_summary` line ("clear worlds to earn
more starting species") no longer reads as sensible once checked against
the GDD's own decided rule for this mechanic. `MetaProgress::absorb`
(`src/run.rs:119-126`) grants **one bonus starting species per two worlds
cleared** — worlds cleared is a run-level proxy for objectives cleared,
which §10's own design note explicitly names as the wrong signal ("what to
rule out: anything accelerating the notebook... key off cumulative
understanding... rather than objectives cleared, so that exploring is
rewarded over rushing"). The mechanic itself (starting-species unlocks) is
fine and stays `[DECIDED light]`; only the trigger metric is wrong.

The GDD note also points at the fix's natural source: task 158 ("Leaving a
world, entering the next, ending a run") is specified to compute exactly
"relationships confirmed... speciations induced... biomes and wild species
met" for the end-of-run summary — the same figures this unlock should key
off. Building a second, divergent tally here ahead of 158 would either
duplicate that counting logic or drift from it; hence the dependency.

## Expected code surface

- Add or change: `src/run.rs` (`MetaProgress::absorb`'s signature and
  formula — take whatever cumulative-understanding figure(s) task 158
  exposes instead of `worlds_cleared`), the call site in `objectives.rs:792`
  (pass the new figure instead of `run_progress.worlds_cleared`), and
  `src/text.rs`'s `NO_UNLOCKS_YET`/`unlocks_summary` wording if the new
  metric needs different phrasing (e.g. "N relationships confirmed" instead
  of "clear worlds").
- Preserve: `worlds_cleared` itself stays untouched and still drives
  `defeat_body`'s own unrelated summary line (`text.rs:198-199`) and
  whatever else already reads it — only its use as the *unlock* trigger is
  replaced. `Unlocks`/`RunProgress::start`'s snapshot-at-run-start shape
  (GDD §10: unlocks earned mid-run never retroactively change a run already
  in progress) stays as-is.
- Evidence needed: `cargo test` covering the new trigger formula (mirror
  the existing `absorb_grants_one_bonus_species_per_two_worlds_cleared`-
  style unit tests, retargeted at the new metric); `cargo clippy -D
  warnings`, `cargo fmt`; a live check that the main-menu line reads
  correctly for both the zero-progress and some-progress cases.

## Out of scope

- Speciations-induced and biomes/wild-species-met as additional trigger
  inputs — name them as future extensions in a code comment, don't
  implement them in this pass; relationships-confirmed alone is enough to
  fix the "objectives cleared" violation this task exists for.
- Any change to `sim.rs`/`world.rs`/`config.rs` tick logic — this is
  run/meta-progression bookkeeping, not simulation.
- Persistence (still `[DECIDED: deferred]`, unrelated to this task).

## Acceptance criteria

- `MetaProgress::absorb`'s formula no longer reads `worlds_cleared` as its
  input; it reads a cumulative-understanding figure sourced from task 158's
  own tracking.
- Existing tests retargeted (not deleted) to assert the new formula's
  behavior at the same rigor as the ones they replace.
- The main-menu unlocks line's wording matches whatever the new trigger
  actually measures — no copy left describing "clear worlds" if the trigger
  no longer keys off that.
- `cargo test`/`clippy`/`fmt` clean; live check confirms the line reads
  correctly in both the no-progress and some-progress cases.

## Validation

- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## Completion

- `Review: REQUIRED`: set this task's and `tasks/QUEUE.md`'s status to
  `READY_FOR_REVIEW` only after validation passes; a reviewer-integrator (a
  different identity) then applies `docs/CODE_REVIEW_PROMPT.md` and records
  `ACCEPTED`.

## Delegating this task

```bash
claude "$(cat tasks/192-unlock-trigger-cumulative-understanding.md)"$'\n\nExecute this task in the current project.'
```
