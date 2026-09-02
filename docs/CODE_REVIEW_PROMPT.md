# Code Review Prompt

```text
Review <TASK-ID> as an independent reviewer-integrator.

Read CLAUDE.md, PROJECT_WORKFLOW.md, the assigned task, its cited Authority, the concise completion report, and the exact diff produced by the implementation commit(s). Follow `docs/CONTEXT_BUDGET_POLICY.md`: load only evidence needed for acceptance criteria and expand context only with a recorded reason. Confirm `Review: REQUIRED` and `READY_FOR_REVIEW` in both the task file and `tasks/QUEUE.md`.

Review scope, dependencies, out-of-scope boundary, acceptance criteria, validation evidence, project invariants (`CLAUDE.md`, `TECH_DESIGN.md` §5), and unrelated changes. Read only task-cited documents and the exact diff. Report only actionable findings with P0/P1/P2 priority and file/line evidence; omit style-only commentary.

Return APPROVE, CHANGES_REQUESTED, or BLOCKED. End the handoff with the fields from `docs/COMPLETION_REPORT_TEMPLATE.md` plus the verdict. After APPROVE only, update exactly the task file's and `tasks/QUEUE.md`'s status to ACCEPTED and commit `docs: accept <TASK-ID>` — this project has no forge/PR step, so acceptance is a direct, separate commit from the implementation. The reviewer-integrator identity must be distinct from whoever implemented the task; never self-approve. On a single-operator project this is satisfied per `PROJECT_WORKFLOW.md`'s "Reviewer-integrator identity on a single-operator project" section: run the review in a fresh session that re-derives evidence independently rather than trusting the completion report, and author the acceptance commit with `git commit --author="Abiogenesis Reviewer-Integrator <reviewer-integrator@abiogenesis.local>"` (committer stays the operator's own identity — this is a per-commit override, not a git config change). Never modify implementation code as part of a review — file a new task instead.

Keep the final review report within ten lines unless findings require more detail.
```
