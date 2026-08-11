# Task 109 — Long-term objective tier + within-run energy economy (BLOCKED)

> **ID**: `109`
> **Category**: Feature / Progression / Objectives
> **Priority**: 🟡 P2
> **Estimate**: ~3h (once unblocked)
> **Assigned to**: unassigned — **do not start**, see Dependencies
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-progression-pacing.md`)

---

## 🎯 Objective

> **This task is scoped for reference but blocked from starting.** The
> long-term objective this task adds must check real gameplay state from
> "Mondo vivo" (e.g. a confirmed zone-conditional relation, a wild-species
> first contact) and "Evolution & xenotypes" (a completed speciation
> event) — content that doesn't exist as *shipped* code yet, only as
> scoped tasks (096-099, 106-107). Scoping this task doesn't require that
> content to exist, but a meaningful implementation does: an `Objective`
> variant with nothing real to check against is dead code. Do not pick
> this up until at least one of {096-099, 106-107} has landed and
> produced a checkable, observable piece of state.

Reconciles the long-horizon ambitions of "Mondo vivo" and
"Evolution & xenotypes" with the current core loop: today, clearing a
world's short 2-3-item objective sequence triggers a full reset (grid,
species, matrix, notebook knowledge all wiped,
`run_flow.rs:68-145`) — destroying any long-term ecosystem investment
right as it starts to pay off. This task adds a genuine **long-term
objective tier** that becomes the real world-clear trigger, while
existing short-term objectives keep their current in-place-advance
behavior but start granting an energy reward. Full reasoning in
`redesign/abiogenesis-progression-pacing.md`.

---

## 📋 Acceptance Criteria (once unblocked)

- [ ] A new long-term `Objective` variant (or variants) exists, checking
      state from whichever of 096-099/106-107 has landed (e.g. "a
      zone-conditional relation has been confirmed," "a speciation event
      has occurred") — exact variant(s) depend on what's actually
      available at implementation time, not prescribed further here.
- [ ] `CurrentObjective`'s sequence (`objectives.rs:136-166`) is restructured
      so the long-term objective is what triggers `WorldCleared`
      (`apply_tick_outcome`, `objectives.rs:403-467`) — existing short-term
      objectives (`Coexistence`/`SurviveIn`/`TriggerBloom`) keep their
      current in-place-advance behavior (index increments, progress
      resets, no world reset) when cleared, unchanged from today.
- [ ] Clearing **any** objective (short- or long-term tier) grants an
      energy reward — a new field, first-pass tunable amount in
      `SimConfig`, no magic numbers.
- [ ] A new energy resource lives at the `RunProgress` level (`run.rs:15-21`)
      — persists across world resets within a run (like `worlds_cleared`
      already does), resets at run start/end like the rest of
      `RunProgress`.
- [ ] `Splice`'s existing cost gate (`input.rs:567`,
      `config.action_costs.splice`) gains an energy-funded upgraded tier
      (e.g. more simultaneous `SpliceEditChoice` edits per use, or a
      reduced action-point cost) once enough energy is banked — `Splice`
      itself stays available from world 0, unchanged (no gating of the
      action's existence, per the doc's explicit decision to avoid
      touching onboarding).
- [ ] Unit tests: energy accumulates correctly across objective clears and
      survives a world reset within a run (mirrors existing
      `RunProgress`/`worlds_cleared` test coverage); `Splice`'s upgraded
      tier only activates once threshold is met.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: a full run showing a short-term
      objective clear granting energy without a world reset, and (once the
      long-term objective's underlying content exists) the long-term
      objective eventually triggering the reset.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` | `Objective` enum (32-52), `CurrentObjective` (136-166), `apply_tick_outcome` (403-467) — sequence/reset logic to restructure. |
| `src/run.rs` | `RunProgress` (15-21), `MetaProgress::absorb` (90-92) — precedent for "progress becomes a carried value," new energy field lands here. |
| `src/run_flow.rs` | `start_world`/`advance_to_next_world` (68-145) — confirms what resets vs. persists per world transition. |
| `src/input.rs` | `apply_splice` (505-578), cost gate at 567/578 — where the energy-funded upgrade tier hooks in. |
| `src/config.rs` | `ActionCosts` (230-248) — where new energy-reward/threshold constants belong. |
| `redesign/abiogenesis-progression-pacing.md` | Full design rationale and open questions this task must resolve at implementation time. |
| `redesign/abiogenesis-living-world.md`, `redesign/abiogenesis-evolution-xenotypes.md` | Source of the long-term objective's actual content — read whichever has landed before defining the new `Objective` variant. |

---

## 🧩 Technical Context

- **Current behavior**: objectives are a fixed 2-3-item sequence; clearing
  the final one always resets the world; clearing a non-final one grants
  no reward, only advances the index. No currency persists across eras,
  worlds, or runs beyond `RunProgress`'s existing integer counters.
- **Desired behavior**: the reset trigger becomes a long-term objective
  tied to mondo-vivo/evolution content; every objective clear grants
  energy; energy persists within a run and funds a `Splice` upgrade.
- Per the source doc's own open questions (unresolved, to decide at
  implementation time, not now): exact energy amounts per objective tier,
  how strong the `Splice` upgrade should feel, whether the long-term
  objective's difficulty scales with `world_index`, and whether the
  short-term sequence itself should become a repeating/renewable loop
  instead of a fixed short list.

---

## 🔨 Suggested Implementation (once unblocked)

1. Confirm which of 096-099/106-107 has actually shipped and pick a
   concrete, checkable long-term objective condition from it.
2. Add the new `Objective` variant(s) and restructure `CurrentObjective`/
   `apply_tick_outcome` so it's the reset trigger, not the current
   short sequence's last item.
3. Add the energy reward on objective clear (short- and long-term) and the
   new `RunProgress`-level energy field.
4. Wire the `Splice` upgrade tier behind an energy threshold.
5. Unit tests per Acceptance Criteria; `cargo test`, `cargo clippy -- -D
   warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- **Do not start this task until at least one of {096-099, 106-107} has
  shipped** — see Objective. If picked up prematurely, flag it back to the
  user rather than inventing placeholder long-term-objective content.
- The fresh-matrix-per-world pillar is untouched by this task — only how
  long a world lasts before that reset changes, never whether it resets.
- `Splice` must remain available from world 0 exactly as today — this task
  adds an upgrade tier, never a gate on the base action.

---

## 🔗 Dependencies

- **Depends on**: at least one of 096, 097, 098, 099, 106, 107 (see
  Objective) — this task cannot be meaningfully scoped further or started
  until real content exists to check against.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/109-progression-long-term-objective-energy.md)"$'\n\nExecute this task in the current project.'
```
