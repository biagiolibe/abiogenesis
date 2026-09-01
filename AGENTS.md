# Abiogenesis — Agent Rules

Read `PROJECT_WORKFLOW.md` before acting. It defines the task lifecycle, review policy, roles, and document precedence.

Work only on the explicitly assigned task. Before implementation, read the assigned task, its cited `Authority`, its `Expected code surface`, and `git status --short`; state a short plan and stop if unrelated changes or an unresolved higher-precedence conflict exist.

Run the task's `Validation` plus the project baseline checks recorded in `CLAUDE.md`. Never claim completion when required validation fails.

## Execution policies

Apply `docs/CONTEXT_BUDGET_POLICY.md` for task-first context loading, progressive expansion, and reasoning selection. Use `tasks/TASK_BLUEPRINT.md` for new or materially revised tasks and `docs/COMPLETION_REPORT_TEMPLATE.md` for the completion handoff. For `Review: REQUIRED` tasks, use `docs/CODE_REVIEW_PROMPT.md`.

These documents define operating detail; `CLAUDE.md` remains the source for stable agent-wide rules and project invariants.
