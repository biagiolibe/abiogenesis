# Task 134 — Two-bot harness: does experimenting actually pay off?

> **ID**: `134`
> **Category**: Test harness / Balance instrumentation
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-27 redesign adoption planning

---

## 🎯 Objective

Build a headless, multi-seed harness that runs two automatic strategies against
the same world and reports which one reaches the world's objectives faster.

- **Exploiter** — acts only on already-confirmed matrix relations, experiments
  the bare minimum.
- **Explorer** — deliberately probes unknown tag pairs before committing.

**Why now, and why before task 136.** Task 136 changes the energy coefficients
specifically to make the hidden matrix necessary — i.e. to make experimenting
pay. Running this harness only *after* that change produces one number with
nothing to compare it against. This task establishes the **pre-change baseline**;
136's acceptance criteria depend on re-running it and comparing.

Design source: `redesign/processed/culture-shock-experiment-incentive.md` §1-§2.

---

## 📋 Acceptance Criteria

- [ ] A headless harness (integration test or `examples/` binary — see note
      below) runs both strategies over N seeds with no window, no egui, no
      rendering.
- [ ] Both strategies run against the **same** seed/world, exercising the
      existing determinism guarantees (GDD §5.7) — the only difference between
      the two runs is the action policy.
- [ ] Primary metric reported: **eras taken to complete the world's objectives**,
      per strategy, aggregated across seeds (median and distribution, not just a
      mean — a strategy that wins narrowly on average but loses catastrophically
      on some seeds is not the same as one that wins consistently).
- [ ] Secondary metric (`§2` of the doc): each executed action is logged as
      taking place in a **known** context (every matrix relation it involves is
      already confirmed for the player) or an **unknown** one, and
      objective-progress-per-point-spent is reported separately for the two
      categories.
- [ ] Result is written somewhere durable (a committed baseline file under
      `tasks/` or a printed report the task's Resolution section records
      verbatim) — 136 must be able to compare against it later.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `tests/balance.rs` | Existing multi-seed headless survey — closest prior art for the harness shape. |
| `tests/determinism.rs`, `tests/run_reproducibility.rs` | The determinism guarantees this test leans on. |
| `src/sim.rs` | `sim::step` — the tick the bots drive. |
| `src/objectives.rs` | Objective state/completion the metric reads. |
| `src/notebook.rs` | Confirmation state — what counts as a "confirmed relation" for both the Exploiter policy and the known/unknown action labelling. |
| `src/input.rs` | The four action handlers. The bots must **not** go through Bevy input; factor the action effects if needed, or replicate their world-level effect directly. |

---

## 🧩 Technical Context

- **Current behavior**: no automated play exists. `tests/balance.rs` runs the
  simulation forward with fixed starting placements and asserts population
  properties; nobody spends action points.
- **Desired behavior**: a policy-driven driver that, each season/era, decides
  which actions to spend the point budget on and executes them at world level.

The confirmation data the Exploiter policy needs is the same evidence store the
notebook already accumulates (`notebook.rs`, `confirmation_threshold`). Read it,
don't recompute it.

Note on placement: `tests/balance.rs::place_starting_organisms` already had to
be corrected once (temperature-spread fix, see `PROJECT_PLAN.md`) because a
naive placement created a tautological `fit = 1.0` coupling with the thing under
test. Do not reintroduce that: the bots' seeding policy must not be secretly
optimal in a way that hides the effect being measured.

---

## 🔨 Suggested Implementation

1. Extract a minimal action-application layer usable without Bevy input (the
   world-level effect of Seed/Stress/Cull/Splice), if `input.rs` doesn't already
   expose one. Keep it in `sim`/`world`, not in the binary — `input.rs` is
   `bevy`-coupled and the harness must stay headless.
2. Define a `BotPolicy` trait (or plain enum + match) with the two strategies.
3. Driver loop: advance a season, let the policy spend its budget, repeat until
   objectives complete or the era budget is exhausted.
4. Multi-seed sweep with a seed count matched to signal, not to ambition — start
   around 50-100 seeds like task 074's survey and raise only if the distributions
   overlap too much to read.
5. Report and record the baseline.

**Harness shape**: prefer `examples/` (or a `#[ignore]`d test) over a normal
`cargo test` case if the sweep takes more than a few seconds — this is a
measurement tool, not a regression guard, and it must not slow the test suite.

---

## ⚠️ Constraints and Caveats

- **Headless and deterministic.** No `bevy::render`, no `bevy_egui`, no
  `rand::rng()`, no `HashMap` iteration. Same rules as `sim`/`world`/`config`
  (`CLAUDE.md`, `TECH_DESIGN.md` §5).
- **The Explorer does not have to win.** The failure criterion is the Exploiter
  winning *systematically*. Two close strategies, with the Explorer ahead on
  harder worlds, is the healthy outcome. Do not tune anything in this task —
  it only measures.
- This task changes **no** game balance. If a coefficient looks wrong while
  writing it, record the observation, don't fix it here.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: 136 (needs the baseline)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/134-two-bot-experiment-incentive-harness.md)"$'\n\nExecute this task in the current project.'
```
