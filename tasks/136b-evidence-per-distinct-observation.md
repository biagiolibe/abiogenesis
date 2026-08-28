# Task 136b — Evidence per distinct observation, not per tick

> **ID**: `136b`
> **Category**: Balance / Simulation
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-28

---

## 🎯 Objective

Make confirming a matrix relation a function of **what the player does**, not of
how long they let the simulation run.

Today `sim::step` emits an `AdjacencyObserved` for every organism, for every
neighbouring tag, **every tick**. A blob sitting still for 200 ticks emits the
same observation 200 times. Scientifically that is one data point repeated, not
200 — but the notebook counts each one as fresh evidence, so confirmation ends up
proportional to population × time.

The consequence is the exact inverse of GDD §7: the efficient way to decode a
world is to **flood it**, and the isolated observation the design calls most
valuable is worth nothing in practice.

---

## 🧩 The measurement this is built on

Direct diagnostic, 2026-08-28, 12 world-0 seeds, 30 eras, greedy seeding:

```text
AdjacencyObserved events:     12,197,641
n_confounders histogram:      [577, 3666168, 8483892, 32584, 14420, 0, ...]
confirmed pairs:              48   (~4/world, 43% of the confirmable ones)
```

Two readings, both load-bearing:

- **43% of every confirmable pair gets confirmed within 30 eras by a bot that
  makes no attempt to observe anything.** `confirmation_threshold` is `1.0`
  against `observation_weight_numerator` `1.0`, so a single unconfounded
  observation confirms outright — and there are twelve million observations.
- **Clean observations are 577 of 12.2M: 0.005%.** The `1/(1+confounders)`
  weighting is a well-designed mechanism that currently never matters, because
  bulk drowns it. Nearly everything arrives at weight 0.5 or 0.33, and at a
  threshold of 1.0 two or three of those are still trivially cheap.

This is also why task 134's explorer *lost* to its exploiter: it spent points
buying information the simulation hands out for free.

---

## 📋 Acceptance Criteria

- [ ] An adjacency contributes evidence **on onset**, not on every tick it
      persists: while the same receiver stays adjacent to the same exerter tag,
      no further evidence accrues from that configuration.
- [ ] The tracking is deterministic and snapshot-based like the rest of
      `sim::step` — no `HashMap` iteration, no ordering sensitivity
      (`TECH_DESIGN.md` §5).
- [ ] `confirmation_threshold` retuned so that the weighting finally bites: a
      clean observation (weight 1.0) should be worth confirming on its own or
      nearly so, while a confounded one (0.33) should need several *distinct*
      episodes. Record the reasoning and the measured before/after.
- [ ] Re-run the diagnostic shape above and record the new event count and
      confirmed-pair rate. The target is that confirmation tracks player
      activity, not elapsed time: doubling the run length with no new actions
      should no longer roughly double what gets confirmed.
- [ ] `assets/config/sim_config.ron` updated in the same commit
      (`tests/config_ron_sync.rs`).
- [ ] `cargo test` and `cargo clippy --all-targets -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | The emission site, ~lines 696-728: the `AdjacencyObserved` push inside the neighbour/tag double loop, and `n_confounders`. |
| `src/knowledge.rs` | `MatrixKnowledge`, and `accumulate_adjacency_evidence` which owns the weight formula. |
| `src/config.rs` | `NotebookConfig::confirmation_threshold`, `observation_weight_numerator`. |
| `assets/config/sim_config.ron` | Sync. |

---

## ⚠️ Constraints and Caveats

- **Where the onset state lives is the main design decision.** Per-organism is
  the obvious reading today, but task 137 replaces organisms with per-cell
  populations, at which point a *cell-to-cell* adjacency onset is a much cleaner
  unit. Implement the simplest thing that is correct now, and expect task 137/138
  to simplify it — that trade was made deliberately (see below), don't try to
  pre-build the 137 version.
- **Deliberate ordering call:** this lands right after task 136 and *before* the
  Phase 1 playtest checkpoint, even though task 138 rewrites pipeline phase 7 and
  will absorb some of this work. The checkpoint is where we find out whether the
  game is fun; reaching it with an evidence economy that rewards flooding would
  falsify exactly that judgement.
- Do **not** raise the threshold as a substitute for the onset fix. The event
  count scales with population², so any fixed threshold is eventually swamped —
  a threshold change alone moves the problem, it doesn't solve it.
- This interacts with task 136 in the right direction: once monocultures stop
  expanding for free, both the event count and the confounder count fall on
  their own. Measure after 136 has landed, not before.

---

## 🔗 Dependencies

- **Depends on**: 136
- **Blocks**: the Phase 1 playtest checkpoint

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/136b-evidence-per-distinct-observation.md)"$'\n\nExecute this task in the current project.'
```
