# Task 117 — Time readout: show progress within the current era, not the run-wide tick counter

> **ID**: `117`
> **Category**: UI / HUD
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-12 (scoped from `redesign/abiogenesis-hud-notebook.md` §2, after a
> discrepancy-check pass against tasks 100-103/097)

---

## 🎯 Objective

`redesign/abiogenesis-hud-notebook.md` §2 asks for the HUD's time readout to
show both the current era **and** progress in ticks *within* that era, e.g.
`era 5 · pulse 14/25`. Today's `text::era_tick_line(world.era, world.tick)`
(`src/text.rs:131-133`, shown at `src/ui.rs:379`) shows `world.tick` — the
**run-wide, ever-increasing** tick counter (`SimWorld::tick`, incremented
once per `sim::step` call and never reset) — not ticks-within-the-current-
era. After even a couple of eras this number stops meaning anything readable
at a glance ("tick 143" doesn't tell the player where they are in the
current era's ~25-tick span).

This task changes the readout's *math*, not its final label wording — task
118 (rename "tick" → "pulse") is a separate, purely cosmetic follow-up; land
this one first or independently, whichever order, since they touch the same
line but for different reasons.

---

## 📋 Acceptance Criteria

- [ ] The HUD's time readout shows `era {N} · tick {current}/{total}` (exact
      wording aside — task 118 may rename "tick" to "pulse" on top of this),
      where `current` is ticks elapsed in the era **currently being played**
      and `total` is that era's full length.
- [ ] `current`/`total` are derived from `sim::EraProgress` (`remaining`,
      counts down to `0`) and `worldgen::era_ticks_for(world_index, era,
      config)` (the era's total length, which varies for world 0's
      shortened onboarding eras, task 082) — `total = era_ticks_for(..)`,
      `current = total - progress.remaining()`. `hud_panel` doesn't
      currently take `EraProgress`/`RunProgress` as parameters; add them.
- [ ] Correct at era boundaries: right when an era completes and the next
      one's `EraProgress` is freshly started (`progress.start(..)`), the
      readout shows `0/{new_total}`, not a stale value from the just-ended
      era or a divide-by-zero/underflow.
- [ ] Correct for world 0's shortened onboarding eras (task 082) — `total`
      must reflect `time.onboarding_era_ticks`, not the standard
      `time.era_ticks`, for those eras specifically (this is exactly what
      `era_ticks_for` already encodes — just make sure the readout actually
      calls it with the right `world_index`/`era`, not a hardcoded
      standard-length assumption).
- [ ] Unit test: readout math is correct at tick 0 of an era, mid-era, and
      the last tick before completion (off-by-one check both ends).
- [ ] Unit test: readout math uses the onboarding era length for world 0's
      early eras and the standard length otherwise (mirrors
      `era_ticks_for`'s own existing test coverage, just asserting the
      readout function composes it correctly).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: watch the readout across a full era
      (manual `n` ticks or auto `Space`) — confirm it counts up within the
      era and resets to `0/{total}` when the next era starts, matching the
      era-progress dots elsewhere in the HUD.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/text.rs` | `era_tick_line` (line 131-133) — the function to change, from taking a raw `tick: u64` to taking era-relative `current`/`total` values (or computing them from `EraProgress`/`era_ticks_for` results passed in). |
| `src/ui.rs` | `hud_panel` (line ~334-379) — call site (`text::era_tick_line(world.era, world.tick)`, line 379); needs `EraProgress` and `RunProgress` (for `world_index`) added as system parameters. |
| `src/sim.rs` | `EraProgress` (line ~527-544) — `remaining()`/`start()`, the source of "ticks left in this era." |
| `src/worldgen.rs` | `era_ticks_for` (line 103-110) — the source of "this era's total length," already handles world 0's onboarding shortening (task 082). |

---

## 🧩 Technical Context

- **Current behavior**: `era_tick_line(era, tick)` formats `"Era {era}  ·
  tick {tick}"` where `tick` is `world.tick: u64`, incremented every
  `sim::step` call for the whole run's lifetime, never reset between eras or
  worlds (well — reset per-world via `SimWorld::new_for_world`, but not
  per-era).
- **Desired behavior**: `tick`/`pulse` progress is relative to the *current
  era*, resetting to `0` each time a new era starts, out of that era's own
  length (which varies for world 0's onboarding eras).
- `EraProgress::remaining` counts *down* from the era's starting length to
  `0` (`sim.rs`'s `tick_and_complete_era`/`single_tick` decrement it each
  tick) — `current = total - remaining` recovers the count-up value the
  readout needs.
- `worldgen::era_ticks_for(world_index, era, config)` is already exactly
  "this era's total tick length," reused as-is — no new config or duplicate
  logic needed, just call it from the HUD with the right arguments
  (`RunProgress::world_index`, `world.era`).

---

## 🔨 Suggested Implementation

1. Change `text::era_tick_line`'s signature (or add a new function/keep the
   old one for compatibility if still used elsewhere — grep first) to take
   `era: u32, current: u32, total: u32` instead of `tick: u64`.
2. In `hud_panel`, add `EraProgress`/`RunProgress` as system parameters,
   compute `total = era_ticks_for(run_progress.world_index, world.era,
   &config)` and `current = total - progress.remaining()`, pass both to the
   updated `era_tick_line`.
3. Handle the era-boundary edge case explicitly (see Acceptance Criteria) —
   trace through what `EraProgress` actually holds the instant a new era
   starts, don't assume without checking.
4. Unit tests per Acceptance Criteria (pure function of `EraProgress`/
   `era_ticks_for`'s outputs, testable without a running `App`).
5. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
6. `cargo run`: verify per the acceptance criteria's live-verification line.

---

## ⚠️ Constraints and Caveats

- **Don't rename "tick" to "pulse" here** — that's task 118's scope,
  purely cosmetic; keep this task's diff focused on the *math*, not the
  label wording, so the two tasks don't collide on the same lines for
  unrelated reasons.
- **Determinism**: this is a pure display computation over already-tracked
  state (`EraProgress`, `era_ticks_for`) — no new RNG, no simulation
  behavior change, purely a HUD read.

---

## 🔗 Dependencies

- **Depends on**: none (082's `era_ticks_for` onboarding-shortening already
  shipped and is reused as-is).
- **Blocks**: none.
- **Related, not a dependency**: task 118 (tick → pulse rename) touches the
  same `era_tick_line`/readout line for a different reason — no strict
  ordering requirement, but expect a small merge/rebase if both land in the
  same session without coordinating.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/117-time-readout-era-relative-pulse-progress.md)"$'\n\nExecute this task in the current project.'
```
