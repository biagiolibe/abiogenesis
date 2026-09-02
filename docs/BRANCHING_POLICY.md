# Branching & Integration Policy

Canonical source for branch, commit, push, and merge mechanics in the governed
SDD workflow. `PROJECT_WORKFLOW.md`, `docs/CODE_REVIEW_PROMPT.md`,
`AGENTS.md`, and `tasks/TASK_BLUEPRINT.md` reference this file rather than
restating it.

## Task branches

- Every task (`Review: REQUIRED` or `NOT_REQUIRED`) is implemented on a
  dedicated branch, `task/<NNN>-<slug>`, created from `main`.
- The implementer works on this branch only, runs the task's `Validation`,
  and creates one atomic commit for the implementation.
- Pushing the task branch to `origin` is optional and never required for the
  workflow to proceed; it is never a substitute for integrating into `main`.

## Review (`Review: REQUIRED` tasks)

- The reviewer-integrator operates in a fresh, independent session on the
  **same task branch** — do not create a new branch, rebase, or cherry-pick.
- After `APPROVE`, the reviewer-integrator makes exactly one additional
  commit on the task branch: a status-only commit setting the task file's and
  `tasks/QUEUE.md`'s status to `ACCEPTED`, authored under the
  reviewer-integrator identity defined in `PROJECT_WORKFLOW.md`.
- **Do not push the task branch after this commit.** It is normal and
  expected for `origin/<task-branch>` to not contain it — acceptance
  evidence reaches the remote through the subsequent push of `main`, not
  through pushing the task branch.

## Integration into `main`

Applies once a task branch's tip is `ACCEPTED` — for `Review: REQUIRED` tasks
that is the reviewer-integrator's status-only commit above; for
`Review: NOT_REQUIRED` tasks it is the implementer's own `ACCEPTED` update
per `PROJECT_WORKFLOW.md`'s lifecycle.

1. Check `git merge-base --is-ancestor main <task-branch>` succeeds (the task
   branch has not fallen behind `main`).
2. If it succeeds, integrate without asking for further authorization — this
   sequence is pre-authorized by this policy, not a fresh push/merge decision
   each time:
   - `git switch main`
   - `git merge --ff-only <task-branch>`
   - `git push origin main` (exactly once)
   - delete the local task branch (`git branch -d <task-branch>`)

If the ancestor check fails, stop and resolve the divergence before
integrating — do not force the merge.

## Remote task-branch cleanup (optional, non-blocking)

After `git push origin main` succeeds, optionally attempt
`git push origin --delete <task-branch>`. Never pass `--force`.

- If this fails, or the remote branch does not exist, report a warning only.
  Do not mark the task `BLOCKED`, and do not run `fetch`, `rebase`, or a
  forced delete to work around it.
- A task correctly integrated into `main` is never `BLOCKED` solely because
  its remote task branch lacks the `ACCEPTED` commit, or because remote
  branch deletion failed — both are expected, cosmetic states.

## What this does not change

Reviewer-integrator identity rules (distinct git author on the `ACCEPTED`
commit, procedural independence, never self-approve) are defined in
`PROJECT_WORKFLOW.md` and are unchanged by this policy — this document only
adds the branch/push/merge mechanics around them.
