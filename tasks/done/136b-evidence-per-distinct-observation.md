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

- [x] An adjacency contributes evidence **on onset**, not on every tick it
      persists: while the same receiver stays adjacent to the same exerter tag,
      no further evidence accrues from that configuration.
- [x] The tracking is deterministic and snapshot-based like the rest of
      `sim::step` — no `HashMap` iteration, no ordering sensitivity
      (`TECH_DESIGN.md` §5).
- [x] `confirmation_threshold` retuned so that the weighting finally bites: a
      clean observation (weight 1.0) should be worth confirming on its own or
      nearly so, while a confounded one (0.33) should need several *distinct*
      episodes. Record the reasoning and the measured before/after.
      **Measured and kept at `1.0` — see Resolution: the value already gives
      exactly this shape once onset-gating stops inflating episode count.**
- [x] Re-run the diagnostic shape above and record the new event count and
      confirmed-pair rate. The target is that confirmation tracks player
      activity, not elapsed time: doubling the run length with no new actions
      should no longer roughly double what gets confirmed.
- [x] `assets/config/sim_config.ron` updated in the same commit
      (`tests/config_ron_sync.rs`). **No value changed, so no edit was
      needed — `confirmation_threshold` stayed at its existing default.**
- [x] `cargo test` and `cargo clippy --all-targets -- -D warnings` clean.

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

## ✅ Resolution (2026-08-28)

**Mechanism.** `SimWorld` gained `adjacency_exposure: Vec<AdjacencyExposure>`,
one entry per cell (sized once at construction, like `cells` — the grid
never resizes). Each entry records `owner_born_season` (whose organism this
history belongs to) and a `u32` bitmask of `TagSlot`s adjacent as of the
last tick that organism was processed. Each tick, `sim::step` computes the
current tick's adjacent-tag mask (already gathered as `neighbour_tags` for
the confounder count), takes `onset_mask = current & !prior`, and only
pushes `AdjacencyObserved` for tags in `onset_mask`. If a different organism
(different `born_season`) now owns the cell, the prior mask is discarded
first — a fresh organism has observed nothing yet. `interaction_delta` (the
*energy* effect of the same adjacency) is untouched and keeps applying every
tick regardless; only the evidence emission is onset-gated.

State lives on `SimWorld`, not `Organism` — consistent with `SelectionPressure`'s
own precedent (`Organism` stays a small `Copy` snapshot) — and is applied via
a one-pass local buffer (`new_exposure: Vec<Option<AdjacencyExposure>>`)
collected during the main per-cell loop and written back after it, rather
than mutating `world.adjacency_exposure` mid-loop against the live
`species`/`world` borrows already in scope. Deterministic and
snapshot-based: no `HashMap`, no ordering sensitivity — a cell's onset only
ever depends on its own prior state and this tick's own neighbour scan.

**`confirmation_threshold`: measured, not changed.** At `threshold = 1.0`,
`observation_weight_numerator = 1.0`, a clean observation (weight `1.0`)
already confirms outright, and a confounded one (e.g. `2` confounders,
weight `0.33`) already needs `3` distinct episodes (`3 × 0.33 = 1.0`) — that
is exactly the shape this task asked for. The bug was never the threshold
number; it was that "episode" and "tick" were conflated, so the same episode
counted itself dozens or hundreds of times. Once onset-gating stops that
inflation, the existing threshold already bites correctly, so it was left
unchanged (`assets/config/sim_config.ron` needed no edit as a result).

**Diagnostic re-run** (same shape as task 134's original: 12 world-0 seeds,
greedy per-season seeding, matched to the original's 750-tick horizon —
30 old-eras × 25 ticks, unaffected by task 135's relabeling):

| | before (task 134's original) | after |
|---|---|---|
| `AdjacencyObserved` events | 12,197,641 | **204** |
| confirmed pairs | 48 / ~112 confirmable (43%) | **5 / 112 (4.5%)** |

A ~60,000x drop in event volume, and confirmation now tracks real
exploration rather than population × time. Directly verified the target
property acceptance criterion #4 names: running each seed for a further 750
idle ticks (no further seeding, population just sitting) after the first
750 added **zero** additional confirmed pairs in every one of the 12 seeds —
previously, doubling the run length with no new actions would have
roughly doubled the confirmed count, since evidence was elapsed-time-bound.
It is now flat under idling, which is the whole point of this task.

**Not done here (by design, per the task's own constraints):** no attempt
to build the task 137 (per-cell population) version of this tracking — the
per-cell `Vec<AdjacencyExposure>` here is the "simplest thing that is
correct now" the task asked for, and is expected to simplify once organisms
become per-cell populations rather than single occupants.

## 🔗 Dependencies

- **Depends on**: 136
- **Blocks**: the Phase 1 playtest checkpoint

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/136b-evidence-per-distinct-observation.md)"$'\n\nExecute this task in the current project.'
```
