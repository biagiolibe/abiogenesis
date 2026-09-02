# Governed SDD Workflow — Abiogenesis

## Document precedence

When documents conflict, the first applicable document wins:

1. `CLAUDE.md` — operating rules for the active agent.
2. `abiogenesis-gdd.md` (marked `[DECIDED]` sections) and `TECH_DESIGN.md` §5 — accepted design and architecture decisions. This project records decisions inline in the GDD instead of a separate ADR log; do not create one.
3. `tasks/QUEUE.md` and atomic task files — execution scope, dependencies, review policy, and validation.
4. `VISION.md`, `VISUAL_STYLE_GUIDE.md`, `player_guide.md`, and `abiogenesis-gdd.md` `[PROPOSED]` sections — background and aspirational material.

Implementation never resolves a conflict silently: update the lower-precedence document, or flip a GDD paragraph from `[PROPOSED]` to `[DECIDED]` once the decision is made.

## Task lifecycle

```text
Review: REQUIRED
QUEUED -> IN_PROGRESS -> READY_FOR_REVIEW -> ACCEPTED

Review: NOT_REQUIRED
QUEUED -> IN_PROGRESS -> ACCEPTED
```

Only `ACCEPTED` tasks satisfy dependencies. This lifecycle applies to task files written or revised after 2026-09-02; task files already `ACCEPTED` under the legacy `[x]`/`[/]`/`[ ]` convention (see `tasks/QUEUE.md` and `tasks/done/`) are not retrofitted and keep their legacy markers.

## Execution assets

- `docs/CONTEXT_BUDGET_POLICY.md` defines task-first context loading, reasoning profiles, and concise communication.
- `tasks/TASK_BLUEPRINT.md` defines the canonical atomic-task shape for new tasks.
- `docs/COMPLETION_REPORT_TEMPLATE.md` defines the implementation and review handoff.
- `docs/CODE_REVIEW_PROMPT.md` defines how a `Review: REQUIRED` task is reviewed.

For a task or review, start with `CLAUDE.md`, then read only the assigned task and the sources it cites. These assets are operational guidance and do not supersede the precedence order above.

## Roles

- Tech designer: defines GDD/`TECH_DESIGN.md` decisions, task scope, dependencies, and review policy. Does not implement feature code unless explicitly assigned.
- Implementer: works on exactly one task, validates it, commits it, and updates the task's and `tasks/QUEUE.md`'s status according to its review policy.
- Reviewer-integrator: independently reviews `READY_FOR_REVIEW` tasks per `docs/CODE_REVIEW_PROMPT.md`. After `APPROVE`, records `ACCEPTED` in the task file and `tasks/QUEUE.md` in a separate commit from the implementation.

## Review policy

Every task written under this workflow declares `Review: REQUIRED` or `Review: NOT_REQUIRED`. `NOT_REQUIRED` is restricted to low-risk documentation, mechanical configuration, simple scaffolding, or focused tests that add no production behavior. It is prohibited for anything touching the simulation's determinism, tick formulas, `SimConfig` balance values, state transitions, persistence, or an unresolved design question.

This project has no branch-per-task or forge/PR mechanism today — commits land directly on `main`, unchanged by this workflow. A `Review: REQUIRED` task's `READY_FOR_REVIEW` → `ACCEPTED` transition is a distinct commit by a reviewer-integrator identity different from the implementer, not a merge.

### Reviewer-integrator identity on a single-operator project

This project has one human operator, so "distinct identity" cannot mean a distinct human. It is satisfied procedurally, by both of the following together:

- **Procedural independence**: the review runs in a fresh agent session that did not write the implementation being reviewed, re-derives the acceptance evidence from the diff and cited sources rather than trusting the implementer's completion report, and never self-approves within the same session that authored the code. The `ACCEPTED` commit message names itself a "Reviewer-integrator pass" and states what was independently re-verified (not just restates the task's claims).
- **A distinct git author on the acceptance commit**: the `READY_FOR_REVIEW` → `ACCEPTED` commit is authored as `Abiogenesis Reviewer-Integrator <reviewer-integrator@abiogenesis.local>` via `git commit --author`, leaving the committer (the operator's own git identity) unchanged. This is a per-commit metadata override, not a change to git config — implementation commits keep the operator's normal author identity. It makes the two roles distinguishable in `git log --format='%an <%ae>'` even though both trace back to the same operator, and it is the mechanical trace that a distinct-identity step actually happened.

Neither substitutes for the other: the git author alone would be a rubber stamp without procedural independence; procedural independence alone leaves no verifiable trace in history.

## Execution discipline

Apply `docs/CONTEXT_BUDGET_POLICY.md`. The review-policy restrictions above remain authoritative; use parallel agents only when their scopes are independent.
