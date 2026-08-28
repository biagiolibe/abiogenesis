# Task 134b — Make the bot policies competent, and re-record the baseline

> **ID**: `134b`
> **Category**: Test harness / Balance instrumentation
> **Priority**: 🔴 P1
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-28

---

## 🎯 Objective

Task 134's two policies are too passive to be a fair baseline: each spends three
points per era, mostly on isolated placements, and never reaches the population
scale at which anything interesting happens. Task 136 is supposed to be verified
by re-running that survey and comparing — but comparing pre- and post-change
numbers produced by a strategy nobody would actually play tells us very little.

Make both bots grow populations the way a real player does, then re-record the
baseline that task 136 will be measured against.

---

## 🧩 Why this matters — the diagnostic that prompted it

A direct diagnostic run (2026-08-28, 12 world-0 seeds, 30 eras, seeding
greedily) produced numbers the two-bot survey never approached:

```text
AdjacencyObserved events:     12,197,641
confirmed pairs:              48   (~4/world, 43% of the confirmable ones)
peak population:              up to 3307 of 10240 cells
occupied Moore adjacencies:   same species 16,154,913 — cross species 302,800
```

Against the survey's `pairs confirmed: 0.82` (exploiter) and `1.45` (explorer).
The gap is entirely the placement rate: greedy seeding reaches thousands of
organisms, the bots reach dozens. Everything the matrix does — energetically and
epistemically — only starts happening at the former scale.

---

## 📋 Acceptance Criteria

- [ ] Both policies pursue population growth as a baseline behaviour, not only
      information: they should reach a population of the same order as the
      diagnostic's (hundreds to low thousands on a healthy seed), so that the
      strategic difference between them is measured on top of a working
      ecosystem rather than instead of one.
- [ ] The exploiter/explorer distinction is **preserved and still sharp** — they
      must differ in *where* and *what* they place given the same growth
      pressure, not in how much they play. A shared growth policy plus a
      divergent placement rule is the shape to aim for.
- [ ] The survey reports population reached (median/peak) alongside its existing
      metrics, so a future run can tell at a glance whether the bots were
      actually playing.
- [ ] `pairs confirmed` is reported against the number of **confirmable**
      (nonzero) pairs in that world, not against all 20 — roughly half are zero
      by construction (`matrix_density` 0.4 plus the forced negative 3-cycle) and
      can never be confirmed, so the raw count understates performance by ~2x.
- [ ] New baseline re-recorded in this file's Resolution, in the same format
      task 134 used. It supersedes task 134's numbers for 136's comparison.
- [ ] `cargo clippy --all-targets -- -D warnings` and `cargo fmt` clean.
- [ ] Nothing tuned. This task changes the harness, never the game.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `examples/two_bot_survey.rs` | The harness. `choose_placement` holds both policies; `play` holds the driver loop and the per-era spend. |
| `tasks/done/134-two-bot-experiment-incentive-harness.md` | The superseded baseline and the definitions (pair set, context buckets) this task keeps. |

---

## ⚠️ Constraints and Caveats

- **Don't let the growth policy smuggle in a strategy difference.** Both bots
  must share the same viability filter and the same appetite for placement; only
  the information dimension may differ. If the exploiter ends up growing faster
  because its rule happens to pick better cells, the comparison measures cell
  quality, not experimentation.
- The action budget (3 points per era) is the real constraint on placement rate
  and must not be worked around — a bot that seeds more than the budget allows
  is not modelling a player. If reaching a realistic population within budget
  turns out to be impossible, that is itself a finding worth recording rather
  than engineering around.
- Keep the bots on `Seed` only, for the reasons task 134 recorded.

---

## 🔗 Dependencies

- **Depends on**: 134
- **Blocks**: 136 (its verification compares against this baseline)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/134b-competent-bot-policies-and-rebaseline.md)"$'\n\nExecute this task in the current project.'
```
