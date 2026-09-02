# Abiogenesis — Agent Rules

Read `PROJECT_WORKFLOW.md` before acting. It defines the task lifecycle, review policy, roles, and document precedence.

Work only on the explicitly assigned task. Before implementation, read the assigned task, its cited `Authority`, its `Expected code surface`, and `git status --short`; state a short plan and stop if unrelated changes or an unresolved higher-precedence conflict exist.

Run the task's `Validation` plus the project baseline checks recorded in `CLAUDE.md`. Never claim completion when required validation fails.

## Execution policies

Apply `docs/CONTEXT_BUDGET_POLICY.md` for task-first context loading, progressive expansion, and reasoning selection. Use `tasks/TASK_BLUEPRINT.md` for new or materially revised tasks and `docs/COMPLETION_REPORT_TEMPLATE.md` for the completion handoff. For `Review: REQUIRED` tasks, use `docs/CODE_REVIEW_PROMPT.md`. For task-branch, commit, push, and merge mechanics, use `docs/BRANCHING_POLICY.md`.

These documents define operating detail; `CLAUDE.md` remains the source for stable agent-wide rules and project invariants.

## Code organization

Follow `docs/CODE_ORGANIZATION.md` for every production-source change: one owning module per responsibility, preserved dependency direction between layers, narrowest working visibility, and structural refactors kept out of behavior-change tasks. If a task needs a new ownership boundary or cannot fit the documented module structure without coupling responsibilities, stop and report the missing architectural decision instead of creating an opportunistic abstraction.

## Command triggers

Treat these developer phrases as the complete authorization for the named workflow. Do not select a different task or act on an unassigned one.

- `Proceed with <TASK-ID>` — run the implementation workflow below for exactly that task.
- `Review <TASK-ID>` — act as an independent reviewer-integrator using `docs/CODE_REVIEW_PROMPT.md`, ideally in a fresh chat or Task-tool subagent.
- `Accept <TASK-ID>` — run the owner-acceptance workflow below.

### Implementation workflow

1. Read the assigned task, its cited `Authority` (or GDD `[DECIDED]` section), and `git status --short`.
2. Before code changes, ensure the worktree contains no unrelated uncommitted changes. If it does, do not stage, modify, discard, or commit those changes; report the exact conflict and stop unless the developer explicitly directs how to proceed.
3. Create and switch to a dedicated task branch per `docs/BRANCHING_POLICY.md` (`task/<NNN>-<slug>`, from `main`). Only one task may write in this checkout at a time. If the branch already exists, inspect it and stop for direction rather than overwriting or rebasing it. Do not create or switch branches in a dirty checkout.
4. State a short plan, then implement only the assigned task and its `Expected code surface`. Preserve architectural boundaries and all SDD scope limits.
5. Run the task's `Validation` plus the project baseline checks recorded in `CLAUDE.md`, unless a command is inapplicable because the task has not yet established the required project artifact. Report any inapplicable command and why.
6. When every required validation passes, record completion according to the task's review policy: `READY_FOR_REVIEW` for `Review: REQUIRED`, `ACCEPTED` for `Review: NOT_REQUIRED`. Make no status change if any validation failed, a required manual check is incomplete, or acceptance criteria are not met.
7. Review the diff to confirm it contains only the assigned task and its required status updates. Create one atomic commit using Conventional Commit style and the task ID. Pushing the task branch is optional per `docs/BRANCHING_POLICY.md` — never required for the workflow to proceed.
8. Report the branch name, commit hash, changed files, acceptance-criteria evidence, validation results, and assumptions. If validation fails or scope is ambiguous, do not commit a partial implementation; report the blocker.

Never run two writing agents concurrently in the same worktree.

### Owner-acceptance workflow

When the developer says `Accept <TASK-ID>` after personally reviewing a `Review: REQUIRED` task, treat it as explicit authorization to skip the agent review and perform only the acceptance-state handoff. Confirm that the task and its canonical queue row are both `READY_FOR_REVIEW`; do not re-review the implementation, rerun validation, change source code, or merge the branch.

Update exactly the task `Status` and its canonical queue row to `ACCEPTED`, and commit only those two edits as `docs: accept <TASK-ID>`, authored under the reviewer-integrator identity defined in `PROJECT_WORKFLOW.md` (`git commit --author="Abiogenesis Reviewer-Integrator <reviewer-integrator@abiogenesis.local>"`) — this project applies the same author override to owner-triggered acceptance as to a reviewer's `APPROVE`. Report the commit. If the required state records are missing or inconsistent, stop and report `BLOCKED`.
