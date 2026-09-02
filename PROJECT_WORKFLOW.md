# Governed SDD Workflow — Abiogenesis

## Document precedence

When documents conflict, the first applicable document wins:

1. `AGENTS.md` / `CLAUDE.md` — operating rules for the active agent.
2. `abiogenesis-gdd.md` (marked `[DECIDED]` sections) and `TECH_DESIGN.md` §5 — accepted design and architecture decisions. This project records decisions inline in the GDD instead of a separate ADR log; do not create one.
3. `tasks/QUEUE.md` and atomic task files — execution scope, dependencies, review policy, and validation.
4. `docs/CODE_ORGANIZATION.md` — normative source-organization policy; it cannot change task scope, behavior, or public contracts.
5. `VISION.md`, `VISUAL_STYLE_GUIDE.md`, `player_guide.md`, and `abiogenesis-gdd.md` `[PROPOSED]` sections — background and aspirational material.

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
- `docs/BRANCHING_POLICY.md` defines task-branch, commit, push, and merge mechanics — the canonical source for all git integration steps below.
- `docs/CODE_ORGANIZATION.md` defines module ownership, dependency direction, and visibility rules for production code.
- `docs/AUDIT_PROMPT_READ_ONLY.md` defines a read-only conformance audit for this workflow.

For a task or review, start with `AGENTS.md`/`CLAUDE.md`, then read only the assigned task and the sources it cites. These assets are operational guidance and do not supersede the precedence order above.

## Roles

- Tech designer: defines GDD/`TECH_DESIGN.md` decisions, task scope, dependencies, and review policy. Does not implement feature code unless explicitly assigned.
- Implementer: works on exactly one task, on a dedicated task branch (`docs/BRANCHING_POLICY.md`), validates it, commits it, and updates the task's and `tasks/QUEUE.md`'s status according to its review policy.
- Reviewer-integrator: independently reviews `READY_FOR_REVIEW` tasks per `docs/CODE_REVIEW_PROMPT.md`. After `APPROVE`, records `ACCEPTED` in the task file and `tasks/QUEUE.md` in a separate commit from the implementation, then integrates the task branch into `main` per `docs/BRANCHING_POLICY.md`.

The developer drives these roles with three command triggers, defined in `AGENTS.md`/`CLAUDE.md`: `Proceed with <TASK-ID>` starts implementation, `Review <TASK-ID>` starts independent review, and `Accept <TASK-ID>` performs the owner-acceptance status handoff after the developer's own review, skipping the agent review without skipping the status/queue update.

## Review policy

Every task written under this workflow declares `Review: REQUIRED` or `Review: NOT_REQUIRED`. `NOT_REQUIRED` is restricted to low-risk documentation, mechanical configuration, simple scaffolding, or focused tests that add no production behavior. It is prohibited for anything touching the simulation's determinism, tick formulas, `SimConfig` balance values, state transitions, persistence, or an unresolved design question.

Every task is implemented on a dedicated task branch and integrated into `main` by fast-forward merge — see `docs/BRANCHING_POLICY.md` for the exact branch, commit, push, and merge sequence. This project still has no forge/PR mechanism: integration is a local merge the operator's own tooling performs, not a hosted pull request. A `Review: REQUIRED` task's `READY_FOR_REVIEW` → `ACCEPTED` transition is a distinct commit by a reviewer-integrator identity different from the implementer, not a merge; the merge into `main` is a separate, later step.

### Reviewer-integrator identity on a single-operator project

This project has one human operator, so "distinct identity" cannot mean a distinct human. It is satisfied procedurally, by both of the following together:

- **Procedural independence**: the review runs in a fresh agent session that did not write the implementation being reviewed, re-derives the acceptance evidence from the diff and cited sources rather than trusting the implementer's completion report, and never self-approves within the same session that authored the code. The `ACCEPTED` commit message names itself a "Reviewer-integrator pass" and states what was independently re-verified (not just restates the task's claims).
- **A distinct git author on the acceptance commit**: the `READY_FOR_REVIEW` → `ACCEPTED` commit is authored as `Abiogenesis Reviewer-Integrator <reviewer-integrator@abiogenesis.local>` via `git commit --author`, leaving the committer (the operator's own git identity) unchanged. This is a per-commit metadata override, not a change to git config — implementation commits keep the operator's normal author identity. It makes the two roles distinguishable in `git log --format='%an <%ae>'` even though both trace back to the same operator, and it is the mechanical trace that a distinct-identity step actually happened.

Neither substitutes for the other: the git author alone would be a rubber stamp without procedural independence; procedural independence alone leaves no verifiable trace in history.

## Execution discipline

Apply `docs/CONTEXT_BUDGET_POLICY.md`. The review-policy restrictions above remain authoritative; use parallel agents only when their scopes are independent.
