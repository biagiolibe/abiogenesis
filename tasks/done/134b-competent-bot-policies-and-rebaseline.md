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

- [x] Both policies pursue population growth as a baseline behaviour, not only
      information: they should reach a population of the same order as the
      diagnostic's (hundreds to low thousands on a healthy seed), so that the
      strategic difference between them is measured on top of a working
      ecosystem rather than instead of one.
- [x] The exploiter/explorer distinction is **preserved and still sharp** — they
      must differ in *where* and *what* they place given the same growth
      pressure, not in how much they play. A shared growth policy plus a
      divergent placement rule is the shape to aim for.
- [x] The survey reports population reached (median/peak) alongside its existing
      metrics, so a future run can tell at a glance whether the bots were
      actually playing.
- [x] `pairs confirmed` is reported against the number of **confirmable**
      (nonzero) pairs in that world, not against all 20 — roughly half are zero
      by construction (`matrix_density` 0.4 plus the forced negative 3-cycle) and
      can never be confirmed, so the raw count understates performance by ~2x.
- [x] New baseline re-recorded in this file's Resolution, in the same format
      task 134 used. It supersedes task 134's numbers for 136's comparison.
- [x] `cargo clippy --all-targets -- -D warnings` and `cargo fmt` clean.
- [x] Nothing tuned. This task changes the harness, never the game.

---

## ✅ Resolution (2026-08-28)

**Root cause of the low population.** Both policies ranked candidates
lexicographically by adjacency bucket (`Isolated`/`Known`/`Unknown`), with
constants (1.0/2.0/3.0) that dominated `viability`'s own 0..1 range. A cell's
*bucket* decided the winner almost every time, not its actual quality — so the
Exploiter, whose scoring never let `Unknown` beat `Isolated`, was locked out of
any cell with an occupied neighbour, and the Explorer's ordering was similarly
bucket-first. Neither policy could do what greedy seeding does: chase the best
cell in the whole candidate pool regardless of who's next to it. Adjacency
(matrix-driven `interaction_delta`) is exactly where the runaway growth in the
diagnostic comes from, so bucket-first scoring capped population by
construction.

**Fix.** `viability` is now the dominant term in `choose_placement`'s score;
the information dimension (`INFO_WEIGHT = 0.15`, `KNOWN_SUM_SCALE = 0.05`) is
added on top as a tie-breaker among comparably-good cells, never large enough
to make a mediocre cell in a favoured bucket beat a genuinely better one in a
disfavoured bucket. The Exploiter still never seeks `Unknown` deliberately
(scored `+0.0`) and the Explorer still ranks `Unknown` highest — the identity
each policy's own doc comment describes is unchanged, only the scale of the
information term relative to viability moved.

Also added: `peak_population` (sampled once per era, since an extinct or
budget-exhausted world ends near zero and only the trajectory says whether
growth was ever real) and `confirmable_pairs` (off-diagonal, nonzero matrix
entries only — `confirmed_pairs` was previously also counting the always-zero
diagonal, which the diagnostic's "all 20" language didn't).

**New baseline** (`cargo run --release --example two_bot_survey -- 40`, world 0,
era budget 60, era ticks 25):

```text
## exploiter
  outcomes            cleared 13, extinct 7, era budget exhausted 20 (of 40)
  short-term eras     reached 13/40 — median 9, p25 7, p75 19, min 6, max 27
  full-sequence eras  reached 13/40 — median 9, p25 7, p75 19, min 6, max 28
  peak population     reached 40/40 — median 118, p25 15, p75 561, min 1, max 3233
  objectives cleared  48 total, 1.20 per world
  points spent        4252 total — isolated 3814, known 0, unknown 438
  objectives / point  0.0113
  pairs confirmed     0.82 per world (33/388 confirmable, 8.5%)

## explorer
  outcomes            cleared 10, extinct 6, era budget exhausted 24 (of 40)
  short-term eras     reached 10/40 — median 11, p25 7, p75 18, min 6, max 29
  full-sequence eras  reached 10/40 — median 11, p25 7, p75 18, min 6, max 30
  peak population     reached 40/40 — median 157, p25 9, p75 743, min 1, max 2679
  objectives cleared  39 total, 0.98 per world
  points spent        4856 total — isolated 2252, known 0, unknown 2604
  objectives / point  0.0080
  pairs confirmed     1.25 per world (50/388 confirmable, 12.9%)

## head to head (short-term objectives, same seed)
  exploiter faster on 9/13, explorer faster on 0/13, tied 4
```

**Reading it.** Population now reaches the diagnostic's order of magnitude
(median in the hundreds, peak in the low thousands for both policies), so the
comparison sits on top of a working ecosystem. The distinction is sharp: the
Explorer spends most of its budget on `unknown` placements (2604 of 4856
points) and confirms 1.5x the pairs per world; the Exploiter spends almost
entirely on `isolated` placements (3814 of 4252) and never touches `known`
either, since early-game evidence is too thin to have confirmed anything yet
by the time most points are spent.

The Exploiter wins the short-term-objective race systematically (9/13 vs
0/13, 4 tied) — this **is** the failure criterion task 134 defined, and it is
the expected pre-136/136b reading: it is exactly the diagnosis that motivated
those two tasks (the matrix is too easy to ignore and too cheap to flood for
information). This baseline is what 136's re-run should be compared against.

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
