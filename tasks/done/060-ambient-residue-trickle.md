# Task 060 — Ambient residue trickle so an isolated Decomposer doesn't collapse outright

> **ID**: `060`
> **Category**: Balance / Feature
> **Priority**: 🟢 P3
> **Estimate**: ~1-2h
> **Assigned to**: unassigned
> **Session**: 2026-08-08 (raised directly by the user while reviewing open proposals)

---

## 🎯 Objective

`Metabolism::Decomposer` only gains energy from `residue`, which today only
exists because *something already died* (`residue_on_death`, `src/sim.rs:310`)
and decays every tick (`residue_decay`, `src/sim.rs:124`). A Decomposer seeded
alone, with nothing else on the grid, has zero residue to draw from and
starves at a flat `-decomposer_upkeep`/tick — confirmed by the existing test
`decomposer_with_no_residue_behaves_like_dark_photolithic`
(`src/sim.rs:1029-1053`), which asserts it dies within ~200 ticks with no
residue anywhere.

This is consistent with the GDD's intent (§16's playthrough has the
Decomposer bloom only *after* a predator collapse leaves a residue field —
it's meant to be a second-order niche, not self-sufficient like Photolithic).
But in practice it means a player who seeds a Decomposer first, before
triggering any death elsewhere, gets an uninformative "it just dies" result
with no way to read anything from it — already confusing enough to need an
ad-hoc player-guide clarification (commit `cc90b26`).

**Decided direction**: add a small ambient background residue trickle,
independent of organism deaths, so an isolated Decomposer survives long
enough to be readable — without becoming self-sufficient the way Photolithic
is. This is deliberately *not* meant to make Decomposer viable as a starting
metabolism on its own merit; it's a floor against total uninformative
collapse.

---

## 📋 Acceptance Criteria

- [x] `EnergyConfig` (`src/config.rs:138-168`) gains a new field
      `residue_ambient_trickle: f32`, documented like its neighbours, default
      `0.05` (starting value, not sacred — see tuning note below).
- [x] In `advance_tick` (`src/sim.rs`, the residue-decay loop at lines
      122-125), every cell's residue gains the trickle *after* decay each
      tick: `cell.residue = (cell.residue - energy.residue_decay).max(0.0) +
      energy.residue_ambient_trickle;`. Applies uniformly to every cell
      (occupied, empty, or already holding organism-death residue) — it's
      ambient background detritus, not tied to any specific cell state.
- [x] **Invariant**: `residue_ambient_trickle` must stay strictly less than
      `residue_decay` (`0.05 < 0.2` today), so residue reaches a small stable
      equilibrium per cell instead of growing unboundedly — add a debug
      assertion or a `tests/balance.rs` check for this relationship, not just
      a comment.
- [x] `decomposer_with_no_residue_behaves_like_dark_photolithic`
      (`src/sim.rs:1029`) needs updating: its premise ("no residue anywhere")
      no longer holds by default. Either construct its `SimConfig` with
      `residue_ambient_trickle: 0.0` to preserve the original assertion
      (probably the right call — it's testing the *no-trickle* baseline
      behavior, which should still exist and still work), or add a sibling
      test for the new trickle-present behavior. Pick one and document why in
      the test's comment.
- [x] New test: an isolated Decomposer (same `world_with_one_decomposer`
      helper) with the *default* config (trickle enabled) survives
      meaningfully longer than the no-trickle case — doesn't need to survive
      indefinitely or grow, just noticeably past the ~7-tick collapse window
      the GDD's predator quick-check uses as its "obviously starving"
      baseline (GDD §5.9). Pick a concrete tick count once you see the actual
      numbers; assert on that, not on vague "longer".
- [x] `tests/balance.rs` still green; if the new trickle changes any existing
      balance assumption, retune `residue_ambient_trickle` rather than
      loosening the test.
- [x] `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt
      -- --check` all clean.
- [x] `abiogenesis-gdd.md` §5.9's residue row gets the new constant added
      (same table that already lists `Residue on death` / decay rate).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `advance_tick`'s residue-decay loop (~line 122-125) — where the trickle is added; decomposer draw logic (~line 170-220) is unaffected, it already reads whatever residue exists. |
| `src/config.rs` | `EnergyConfig` — new `residue_ambient_trickle` field. |
| `src/sim.rs` (tests) | `decomposer_with_no_residue_behaves_like_dark_photolithic` (~1029), `world_with_one_decomposer` helper (~1004). |
| `tests/balance.rs` | Whole-run balance assertions that might shift with any residue baseline change. |
| `abiogenesis-gdd.md` | §5.9 numeric baseline table. |
| `PROJECT_PLAN.md` | §1 "Raised from playtesting (2026-08-08)" — the proposal this task implements. |

---

## 🧩 Technical Context

- **Current behavior**: residue only exists where an organism has died; it
  decays linearly (`residue_decay`, currently `0.2`/tick) toward zero and
  never regenerates on its own.
- **Desired behavior**: every cell also gains a small constant
  (`residue_ambient_trickle`, `< residue_decay`) each tick, so residue
  reaches a small non-zero equilibrium everywhere instead of flooring at
  zero — enough for an isolated Decomposer to not immediately starve out
  uninformatively, not enough to make it self-sufficient.
- **Why the constant must stay small**: the Decomposer's draw is shared
  across its Moore neighbourhood (`src/sim.rs:170-216`, up to 9 cells
  including its own), each contributing roughly the trickle's equilibrium
  value. A trickle too close to `residue_decay` could let an isolated
  Decomposer approach full `decomposer_extract_rate` gain from ambient
  residue alone — that would make it self-sufficient like Photolithic,
  which is explicitly *not* the goal here. Keep it tuned toward "survives,
  doesn't starve nonsensically" not "thrives alone."

---

## 🔨 Suggested Implementation

1. Add `residue_ambient_trickle` to `EnergyConfig` with a `0.05` default.
2. Add the trickle to the residue-decay loop in `advance_tick`.
3. Update the now-invalidated "no residue anywhere" test to set
   `residue_ambient_trickle: 0.0` explicitly (preserves it as the true
   no-trickle baseline).
4. Add the new "isolated Decomposer survives longer with ambient trickle"
   test, running the sim far enough to observe the actual survival window,
   then asserting a concrete number.
5. Run `tests/balance.rs`; retune `residue_ambient_trickle` if anything
   regresses.
6. Update the GDD §5.9 table and `PROJECT_PLAN.md`'s proposal entry.

---

## ⚠️ Constraints and Caveats

- **No magic numbers**: the trickle constant belongs in `SimConfig`
  (`EnergyConfig`), never hardcoded in `sim.rs`.
- **Determinism**: the trickle is a fixed per-tick constant, not
  RNG-derived — no new determinism surface to worry about.
- **Don't** touch `decomposer_extract_rate` or `decomposer_upkeep` — this
  task's scope is the residue *supply floor*, not decomposer's own
  efficiency, which is separate tuning territory.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/060-ambient-residue-trickle.md)"$'\n\nExecute this task in the current project.'
```
