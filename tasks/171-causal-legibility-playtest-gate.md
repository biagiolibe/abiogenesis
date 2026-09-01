# Task 171 — Causal-legibility playtest gate

> **ID**: `171`
> **Category**: Process / Verification
> **Priority**: 🔴 P1 (Phase 2 — legibility, gates Phase 3)
> **Estimate**: ~3h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

> **Status (governed-sdd)**: IN_PROGRESS &nbsp;·&nbsp; **Review**: REQUIRED &nbsp;·&nbsp; **Reasoning**: medium
> **Authority**: `abiogenesis-gdd.md` §5.8/§5.9 + `tasks/QUEUE.md` Phase 2/3 gate notes
> **Expected code surface / Out of scope / Validation**: see 📁 Relevant Files, ⚠️ Constraints and Caveats ("No balance changes in this task" is the out-of-scope boundary), and Acceptance Criteria below.

---

## 🎯 Objective

Phase 2 exists to make the hidden matrix's causal chain *learnable* — Cull
tracked as an observation (146), Splice constrained to confirmed traits
(147), the dominant-stimulus/genome-diff reveal (170), HUD/notebook
legibility work (151-153). This task is the **checkpoint that verifies it
worked**, before Phase 3 spends a whole phase adding content on top of a
loop that might still be opaque. Two checks, not one:

1. **Bot-vs-bot necessity check** — extends the task 134/134b two-bot
   harness (`examples/two_bot_survey.rs`) to measure *legibility*, not
   *energetic necessity* (136 already measured that). A bot only ever reads
   what the game actually surfaces to a player (`knowledge::MatrixKnowledge`,
   the dominant-stimulus/genome-diff data 170 exposes, Cull's new tracked
   observation) — never hidden ground truth (the real matrix, a species'
   real tag-driven modifiers). If a policy built only from surfaced data
   converges on the matrix about as fast as an oracle policy that's allowed
   to peek at ground truth, the surfaced data is sufficient — the chain is
   legible to a mechanical reader, which is a necessary (not sufficient)
   condition for it being legible to a human.
2. **Human playtest protocol** — a written script for an actual playtester,
   since Phase 1's own playtest checkpoint was **explicitly skipped by
   direct user instruction** (`tasks/QUEUE.md`, Phase 1 note) and is still
   unpaid down. This task inherits that debt for the accumulated Phase 1 +
   1b + 2 changes together, because a mechanical check can confirm data is
   *exposed* but not that a person actually *notices and uses* it.

Design source: GDD §5.8 (`Anti-degeneration`) and §5.9 (`Starting
constants`) — read in full this session. Neither section is really about
legibility measurement; §5.8's three anti-degeneration levers and §5.9's
baseline numbers are what this task's bot-vs-bot run must **not**
inadvertently break while it's poking at the harness (e.g. don't retune
`selection_pressure_threshold` or metabolic gains here — that's 136/135's
job, already done; this task only adds read-only instrumentation and new
bot capabilities, no balance changes). `tasks/QUEUE.md`'s own framing (this
session): *"task 171 formalizes a bot-vs-bot necessity check and a human
playtest protocol... don't start Phase 3 content work casually before
running it."*

**This is a gate, not a feature.** Its deliverable is a verified pass/fail
result plus the protocol document, not a shipped game mechanic.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] `examples/two_bot_survey.rs` (or a new sibling example, if extending
      the existing one in place gets unwieldy) gains bot policies that use
      **Cull** and **Splice**, now that both are legible actions (task 146
      gives Cull a tracked observation; task 147 constrains Splice to
      confirmed traits) — the current bots use only `Seed`
      (`abiogenesis::actions::attempt_seed`) because neither action carried
      usable signal when 134 was written. Extending the harness to use them
      is this task's first real dependency on 146/147 landing.
- [ ] A new **legibility-gap metric**: run two policy variants side by side
      — (a) *surfaced-only*, deciding purely from `MatrixKnowledge` +
      whatever 170 exposes (dominant stimulus, genome diff) + Cull's new
      observation; (b) *oracle*, allowed to read the real generated matrix
      directly (bypassing confirmation) for the same decisions. Report the
      gap in eras-to-clear / pairs-confirmed between them across the same
      seed set 134 used. A small, stable gap across seeds is a pass; a
      large or seed-dependent one names exactly which signal is still
      effectively hidden.
- [ ] **Explicitly do not change balance.** No `SimConfig` retuning as part
      of this task — if the bot-vs-bot run surfaces a balance regression,
      file it as a new task rather than fixing it here (mixing measurement
      and fix would make future checkpoints incomparable, same reasoning
      134b gave for re-baselining before 136).
- [ ] Written **human playtest protocol** (own file or a section of this
      task's own results, `tasks/171-results.md` or similar — decide during
      implementation) covering: what to hand the playtester (fresh build,
      no prior explanation beyond the existing onboarding), what task to
      give them (e.g. "get through world 0, then tell me in your own words
      why the last speciation happened and what one matrix relation you're
      confident about and why"), what to observe (do they open the
      notebook unprompted, do they read the dominant-stimulus reveal line,
      do they correctly attribute a Cull/Splice outcome), and concrete
      pass/fail criteria distinguishing "the game surfaces this and they
      used it" from "the game surfaces this and they never noticed" — the
      exact gap `culture-shock-naive-player-example.md`'s five friction
      points (referenced in GDD §4's changelog and already fixed once by
      141-144) were about, now re-run against the accumulated Phase 1+1b+2
      changes.
- [ ] At least one actual playtest run against the protocol, with a real
      person if available; if not available this session, the protocol
      must be complete and immediately runnable by someone else, and the
      task stays open (not archived) until a run happens — do not mark this
      task `[x]` on protocol-authorship alone, only once at least the
      bot-vs-bot check has run and produced a verdict, and the human
      protocol has either run once or is explicitly handed off with a
      clear "still owed" note in `QUEUE.md` mirroring how Phase 1's
      checkpoint was tracked as owed.
- [ ] Results (bot-vs-bot numbers, human playtest notes if run) recorded
      verbatim in this task file or a linked results file, same pattern
      134's baseline recording — future checkpoints need something to
      compare against.
- [ ] `tasks/QUEUE.md` Phase 3 stays gated (⚠️ note already present) until
      this task's verdict is a clear pass; if it's a fail, the task's
      results should name which specific Phase 2 task under-delivered
      rather than a vague "still not legible."

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `examples/two_bot_survey.rs` | Existing bot harness (670 lines) — exploiter/explorer policies, seed sweep, eras-to-clear + evidence-split reporting. Extended with Cull/Splice-using policies and the surfaced-vs-oracle legibility comparison. |
| `src/knowledge.rs` | `MatrixKnowledge`, `accumulate_adjacency_evidence` — the surfaced-data API a "surfaced-only" bot policy is restricted to. |
| `src/actions.rs` | `attempt_seed` — the existing action helper extracted for bot use in task 134; a sibling `attempt_cull`/`attempt_splice` (or equivalent) likely needs the same extraction treatment if `input.rs` still owns that logic exclusively. |
| `tasks/done/134-two-bot-experiment-incentive-harness.md`, `134b-*.md` | Prior art for this harness's shape, seed set, and reporting format — reuse rather than reinvent. |
| `redesign/processed/culture-shock-naive-player-example.md` | Source of the five original friction points 141-144 fixed; the human protocol's playtest script should walk the same kind of scenario, now re-checked against the fuller Phase 1+1b+2 state. |
| `tasks/QUEUE.md` | Phase 2/3 gate notes — update once this task has a verdict. |

---

## 🧩 Technical Context

- **Current state**: the two-bot harness exists and works, but only
  exercises `Seed` (134's own scope note: *"the bots use only Seed, since
  Cull emits no observation yet and Splice's confirmed-traits constraint is
  task 147"*). It measures whether experimenting (Explorer) beats exploiting
  known-good placements (Exploiter) — a *behavioural incentive* check, not a
  *data-sufficiency* check. Nothing in the codebase today asks "if a policy
  is restricted to exactly what the player sees, how close does it get to a
  policy with perfect information?" — that comparison is what this task
  introduces.
- **What "surfaced-only" means concretely**: the same restriction the real
  player already has — a policy may only branch on `MatrixKnowledge`'s
  confirmed/hypothesis state, the dominant-stimulus and genome-diff fields
  170 adds to speciation/reveal events, and Cull's new tracked observation
  (146) — never on `world.species`' real tag interactions or the generated
  matrix directly. "Oracle" means the same policy shape, but allowed a
  direct read of the real matrix for its decisions (still spending budget
  normally — this isolates *information*, not *cost*).
- **Why this needs 146/147/170 landed first, not just any three Phase 2
  tasks**: those three specifically change what data a mechanical policy
  *could* use (Cull's observation, Splice's trait gating implying which
  traits are "confirmed enough to build with," the dominant-stimulus/diff
  fields). The HUD/notebook/pixel-grain tasks (151-153) change how a
  *human* perceives the same data, not what a bot can read — they matter to
  the human-protocol half of this task, not the bot-vs-bot half.

---

## 🔨 Suggested Implementation

1. Confirm 146, 147, 170 are landed (this task is naturally last in Phase 2
   for that reason — see Dependencies).
2. Extract whatever Cull/Splice logic still lives only in `input.rs` into
   `src/actions.rs`, mirroring `attempt_seed`'s task-134 extraction, so the
   example binary can call it without depending on Bevy input systems.
3. Add `Policy::Cull`-aware and `Policy::Splice`-aware decision logic to the
   existing Exploiter/Explorer bots, or add new named policies if mixing
   concerns muddies the existing pair — decide based on how the current
   `Policy` enum/trait is shaped once read.
4. Add the surfaced-only vs. oracle policy pair and the seed-sweep
   comparison; report the gap the same tabular way the existing survey
   reports eras-to-clear and evidence splits.
5. Run it, record numbers verbatim in this task file (or a linked results
   file), verdict pass/fail.
6. Write the human playtest protocol as a standalone, handoff-ready
   document (concrete steps, not vague guidance) referencing
   `culture-shock-naive-player-example.md`'s scenario shape.
7. If a real playtester is available this session, run it and record
   findings; otherwise mark that half explicitly outstanding in `QUEUE.md`,
   the same way Phase 1's own skipped checkpoint was tracked, and leave 171
   `[ ]`/`[/]` rather than `[x]` until it happens.

---

## ⚠️ Constraints and Caveats

- **No balance changes in this task.** It measures; it doesn't tune. A
  discovered regression becomes a new task, not an inline fix here — keeps
  this checkpoint's numbers comparable to the next one, same principle
  134b already established (re-baseline before, don't fix-and-compare in
  the same pass).
- **`examples/` stays out of `cargo test`** (existing convention, see the
  file's own doc comment) — this remains a measurement tool run on demand,
  not a regression guard; don't fold its assertions into `tests/`.
- The "surfaced-only" restriction must be a real code boundary (the policy
  genuinely cannot read hidden state), not just a convention respected by
  hand — otherwise the comparison proves nothing.
- Keep `sim`/`world`/`config` free of `bevy::render`/`bevy_egui` deps per
  `TECH_DESIGN.md` §5; this task lives in `examples/`, already outside that
  boundary, but any extraction into `src/actions.rs` must respect it.

---

## 🔗 Dependencies

- **Depends on**: 146 (Cull observation), 147 (Splice confirmed-traits
  constraint), 170 (dominant-stimulus/genome-diff surfacing) — the three
  Phase 2 tasks that change what a mechanical policy can legitimately read.
  Should run **last** in Phase 2, after those three land, for the bot-vs-bot
  half to be meaningful; the human-protocol half could in principle run
  earlier but is more useful covering the *whole* accumulated Phase 2 state
  at once, so keeping both halves together at the end is simpler than
  splitting this into two tasks.
- **Blocks**: Phase 3 content work (155-157) — informally, per `QUEUE.md`'s
  existing ⚠️ gate note; not a hard code dependency, a process one.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/171-causal-legibility-playtest-gate.md)"$'\n\nExecute this task in the current project.'
```
