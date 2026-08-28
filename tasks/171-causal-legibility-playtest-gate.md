# Task 171 — Causal-legibility playtest gate before Phase 3

> **ID**: `171`
> **Category**: Validation / tooling
> **Priority**: 🟡 P2
> **Estimate**: ~3h (harness) + external playtest sessions (not estimated here)
> **Assigned to**: unassigned
> **Session**: 2026-08-28

---

## 🎯 Objective

Formalize a **gate** that must pass before Phase 3 ("content and variety",
tasks 155+) starts, verifying that the core deductive loop is actually legible
and that the hidden matrix is actually necessary — not just balanced on paper
per §5.9.

Two parts:

**(a) Bot-vs-bot harness extension.** Extend the existing two-bot harness
(tasks 134/134b) to compare an "environment-only" bot (chooses only cells with
favorable visible light/temperature, ignores the matrix) against the existing
"controlled-experimenter" bot, across a fixed seed set. Measure: survival,
growth, objective completion, time-to-victory, number of relevant confirmations.
The experimenter bot must come out ahead on at least one meaningful combination
of these metrics — if it doesn't, the matrix isn't necessary and §5.9's balance
claim is wrong in practice, not just in theory.

**(b) Human playtest protocol.** A short written protocol (not code) for
running sessions with testers who have not read the GDD, capturing:
- time-to-first-useful-observation (target: within 5 min)
- time-to-first-hypothesis (target: within 10 min)
- time-to-first-confirmation (target: within 15-20 min)
- whether the tester can correctly explain at least one positive and one
  negative relationship within ~20 minutes, unprompted

This task is a **gate**, not new game content: no Phase 3 task (155+) should
start until (a) passes on the current build and (b) has been run at least once
with real testers.

Reference: an external design-review document (`culture-shock-risk-decision-review.md`,
not in-tree, reviewed 2026-08-28) proposed this protocol. The GDD/QUEUE structure
already independently follows a similar phased core-before-content approach
(Phase 1 core loop → Phase 2 legibility → Phase 3+ content/payoff); this task
formalizes *verifying* that the phase boundary is actually earned, rather than
just structurally implied by task ordering.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors; `cargo clippy -- -D warnings` clean.
- [ ] Bot harness extended with an "environment-only" bot policy, run against
      the existing experimenter policy across a fixed, documented seed set.
- [ ] A report (test output or committed doc) showing per-seed and aggregate
      results for both bots on: survival, growth, objective completion,
      time-to-victory, confirmations.
- [ ] A short human-playtest protocol document exists (e.g.
      `tasks/171-playtest-protocol.md` or similar) with the metrics/thresholds
      above and a minimal post-session question list.
- [ ] `tasks/QUEUE.md` records this as a blocking gate for Phase 3 (155+) —
      not literally blocking merges, but flagged so the phase isn't started
      casually before this is run.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| existing task 134/134b harness code | Base to extend with new bot policy |
| `tasks/QUEUE.md` | Phase 2/Phase 3 boundary, where the gate note belongs |

---

## 🧩 Technical Context

- **Current behavior**: the two-bot harness (134/134b) exists as a pre-change
  baseline tool (exploiter vs explorer), not currently run as a standing gate
  with pass/fail thresholds.
- **Desired behavior**: the harness becomes a repeatable check with explicit
  success criteria, and a human-testing protocol exists alongside it so the
  "is the loop legible" question gets answered empirically before Phase 3
  content work begins.

---

## 🔨 Suggested Implementation

1. Review the 134/134b harness code to see what bot policies already exist and
   how easily a third "environment-only" policy can be added.
2. Implement the environment-only policy and wire it into the existing
   comparison/report output.
3. Run across a documented fixed seed set; capture results.
4. Write the human playtest protocol as a short markdown doc.
5. Add a short note in `tasks/QUEUE.md` at the Phase 2/Phase 3 boundary
   referencing this gate.

---

## ⚠️ Constraints and Caveats

- **Style**: harness stays deterministic/headless per `TECH_DESIGN.md`.
- This task does not require running actual human playtest sessions to be
  considered "coded complete" — but the protocol doc must exist and be usable,
  and the bot-vs-bot part (a) should actually be run with results captured.

---

## 🔗 Dependencies

- **Depends on**: 134, 134b (two-bot harness, done)
- **Blocks**: informally gates the start of Phase 3 (tasks 155+)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/171-causal-legibility-playtest-gate.md)"$'\n\nExecute this task in the current project.'
```
