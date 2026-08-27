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

---

## ✅ Resolution (2026-08-27)

`examples/two_bot_survey.rs`, plus the two library extractions it needed.

### The baseline (this is the number task 136 must beat)

`cargo run --release --example two_bot_survey -- 40`, on the build at this
commit (`era_ticks 25`, `point_budget_per_era 3`, `photolithic_metabolism_gain
2.0`, no `interaction_scale`):

```text
two-bot survey — world 0, seeds 0..40, era budget 60, era ticks 25

## exploiter
  outcomes            cleared 13, extinct 7, era budget exhausted 20 (of 40)
  short-term eras     reached 13/40 — median 9, p25 7, p75 16, min 6, max 27
  full-sequence eras  reached 13/40 — median 9, p25 7, p75 16, min 6, max 28
  objectives cleared  47 total, 1.17 per world
  points spent        4226 total — isolated 3877, known 0, unknown 349
  objectives / point  0.0111
  pairs confirmed     0.82 per world

## explorer
  outcomes            cleared 14, extinct 5, era budget exhausted 21 (of 40)
  short-term eras     reached 14/40 — median 10, p25 7, p75 15, min 6, max 29
  full-sequence eras  reached 14/40 — median 10, p25 7, p75 16, min 6, max 29
  objectives cleared  54 total, 1.35 per world
  points spent        4458 total — isolated 1442, known 0, unknown 3016
  objectives / point  0.0121
  pairs confirmed     1.45 per world

## head to head (short-term objectives, same seed)
  exploiter faster on 9/17, explorer faster on 4/17, tied 4
```

### What it says

**The exploiter is ahead, and the margin is not the interesting part.** It wins
9 of the 17 head-to-head seeds against the explorer's 4, with 4 ties, and
reaches the short-term objectives a median era sooner. At this sample that is
suggestive rather than conclusive, and the two strategies clear a comparable
number of worlds (13 vs 14) — so the doc's "systematically" bar is arguably not
met.

**The finding that matters is the `pairs confirmed` line, which the survey was
not designed to look for.** With 5 active tags there are 20 off-diagonal pairs
per world. The exploiter ends a world having confirmed **0.82** of them; the
explorer, after spending 3016 of its 4458 points deliberately probing unknown
adjacencies, ends with **1.45**. Both are close to nothing.

So the problem in world 0 is worse than "the matrix is optional". It is that
**the matrix is nearly unobtainable**: 3 clean observations are needed to
confirm one pair (`confirmation_threshold 1.0` against a weight of
`1/(1+confounders)`), and organisms rarely stay adjacent to a stable, unconfounded
neighbour long enough to produce them. A player who wants to decode the matrix
has no efficient way to try. Neither bot ever reached a **known** context even
once across 40 worlds — the `known 0` column is not a policy artefact, it is
the absence of anything to exploit.

That reframes task 136's job. Making environmental adaptation break-even makes
the matrix *necessary*; it does not by itself make it *learnable*. The
`Isolate` action the experiment-incentive document names as its first
corrective lever (and which task 138 is now scoped to leave an attachment point
for) looks better-motivated after this run than it did on paper.

**`short_term_eras` and `full_eras` are near-identical**, which was not
expected. `Objective::Speciation` reads `world.has_speciated`, a sticky flag —
by the time the short-term objectives are done a speciation has usually already
happened, so the final objective clears in the same era or the next. The two
metrics were kept separate anyway: task 135 lengthens the era and retunes
`selection_pressure_threshold`, which should pull them apart.

### Definitions chosen, and why

**Information context of a placement** — decided before the placement, from
what the bot can see. The *pair set* is every `(neighbour tag, seeded species
tag)` combination `sim::step` would evaluate at that cell: the tags of occupied
Moore neighbours crossed with the seeded species' own tags.

- **Isolated** — empty pair set (no occupied neighbour). Its own bucket, not a
  hedge: such a placement is neither known nor unknown, yet it is the cleanest
  observation the game offers (GDD §7's weight-1.0 case) once something grows
  beside it. Folding it into either other bucket would misreport both.
- **Known** — every pair already confirmed in `MatrixKnowledge`.
- **Unknown** — at least one pair unconfirmed.

The pair set is deliberately *not* filtered through `tag_gate_satisfied`: the
bot models a player, and a player cannot know which terrain-conditional gates
(task 096) are active before observing them.

**Objective progress per point** is reported as a spend split plus one overall
`objectives / point`, not as a per-bucket efficiency. Dividing total objectives
by a single bucket's points is not an efficiency — it is the same numerator
over an arbitrary slice of the denominator, and it flatters whichever bucket a
policy barely touches. The comparison that carries information is between the
two policies, which realise the two spend mixes by construction.

### Extractions this needed

`tests/` and `examples/` see only the library crate, never `main.rs`'s modules,
so both moves were forced by the crate structure rather than chosen:

- **`src/knowledge.rs`** (new) — `MatrixKnowledge` moved down from
  `notebook.rs`, plus a new pure `accumulate_adjacency_evidence` that owns the
  GDD §7 weight formula `numerator / (1 + n_confounders)`. `notebook.rs`'s
  system now calls it and keeps only the presentation half (log line, unseen
  badge). The formula existing in exactly one place is the point:
  a second copy in the harness would be precisely the notebook/narration drift
  `tick-pipeline` argues against. `TerrainKnowledge` deliberately stayed in
  `notebook.rs` — the bots don't need it.
- **`src/actions.rs`** (new) — `attempt_seed` moved down from `input.rs`, now
  returning `Option<usize>` (the placed cell index) instead of `bool`, so
  `input.rs` can still do its `PlayerPlacedCells` bookkeeping, which is
  notebook state rather than simulation state and stayed up there. Stress and
  Cull were **not** moved: they should move with tasks 145/146, which rework
  them, not ahead of that work.
- `sim::env_fit` and `objectives::update_grace_progress` made `pub`. The first
  so the harness judges viability with the real §5.9 curve instead of a copy;
  the second because grace suppresses total-extinction failure, and a
  grace-free harness would fail worlds earlier than a real player does.

### Limitations, recorded rather than fixed

- **The bots use only `Seed`.** `Cull` produces no observation in the current
  build (that is task 146) and `Stress` only shifts temperature, so neither
  carries information — including them would add noise to a measurement about
  the value of information, not signal. `Splice` is excluded for a stronger
  reason: its design-intended constraint (assignable traits restricted to
  confirmed ones) is task 147 and does not exist yet, so modelling it now would
  measure a mechanic that is scheduled to change.
- **The two arms share a seed but diverge in RNG draw count** as soon as their
  actions differ. That is inherent to comparing strategies on one world, not a
  determinism defect. The bot's own decisions are drawn from a separate
  `StdRng` seeded identically for both policies, never from `world.rng`, so the
  decision rule is the only deliberate difference.
- **Both policies share one viability filter** (`MIN_VIABLE_FIT`), so neither
  can win by simply placing organisms where they survive better. The only axis
  they differ on is what they do about information.
- **40 seeds.** Distributions are printed in full (median, quartiles, min, max)
  so it is visible when the sample is too small; the seed count is the first
  CLI argument.

Nothing was tuned. `cargo test` (309 tests), `cargo clippy --all-targets -- -D
warnings` and `cargo fmt` all clean.
