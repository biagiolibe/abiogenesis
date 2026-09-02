# Task [NNN] — [Title]

Priority: [🔴 P1 / 🟡 P2 / 🟢 P3]
Status: QUEUED
Review: REQUIRED | NOT_REQUIRED
Dependencies: [none / task ID, ...]
Reasoning: medium
Reasoning justification: [required for high/xhigh; omit for medium]

## Authority

- [Path to the GDD section (`[DECIDED]`) / `TECH_DESIGN.md` §/ `redesign/processed/` document named as this task's Design source, per `CLAUDE.md`'s exception.]

## Goal

[Concrete outcome and why it's needed.]

## Expected code surface

- Add or change: [exact module, file, or bounded component.]
- Preserve: [interfaces, invariants, and adjacent areas that must not change.]
- Evidence needed: [tests, checks, manual inspection, or handoff evidence.]

## Out of scope

[Adjacent work that must not be implemented as part of this task.]

## Acceptance criteria

- [Measurable observable outcome.]
- [Measurable invariant or test condition.]

## Validation

- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## Completion

Implement on a dedicated task branch and integrate per `docs/BRANCHING_POLICY.md`.

- For `Review: REQUIRED`, set this task's and `tasks/QUEUE.md`'s status to `READY_FOR_REVIEW` only after validation passes; a reviewer-integrator (a different identity) then applies `docs/CODE_REVIEW_PROMPT.md`, records `ACCEPTED`, and integrates the branch into `main`.
- For `Review: NOT_REQUIRED`, set this task's and `tasks/QUEUE.md`'s status to `ACCEPTED` only after validation passes, then integrate the branch into `main` per `docs/BRANCHING_POLICY.md`.

## Delegating this task

```bash
claude "$(cat tasks/[NNN]-name.md)"$'\n\nExecute this task in the current project.'
```
