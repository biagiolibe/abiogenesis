# Abiogenesis

Emergent-simulation roguelike in Rust + Bevy: seed life on alien worlds and reverse-engineer a hidden biochemical matrix.

## Commands

```bash
cargo run                      # runs the game
cargo test                     # unit tests + determinism/balance tests
cargo clippy -- -D warnings    # must be clean before closing a task
cargo fmt
```

## Documents

| File | Contents |
|---|---|
| [`abiogenesis-gdd.md`](abiogenesis-gdd.md) | **Design — source of truth** (v0.7). Mechanics, tick formulas (§5.6), numeric baseline (§5.9). Marks `[DECIDED]` vs `[PROPOSED]` explicitly — respect the distinction. Large (~106KB): read the section you need, not the whole file. |
| [`TECH_DESIGN.md`](TECH_DESIGN.md) | Architecture: plugins, states, `SystemSets`, invariants. |
| [`tasks/QUEUE.md`](tasks/QUEUE.md) | **What to do now.** |
| [`player_guide.md`](player_guide.md) | **Player-facing manual.** What the game is, controls, loop, mechanics — also surfaced in-game via the main menu's "How to play" panel. |
| [`VISION.md`](VISION.md) | **Long-term roadmap — aspirational, not committed.** Ecosystem depth, pacing, evolution, biochemistry flavor. Ideas here graduate into `PROJECT_PLAN.md` + a task file when prioritized. |
| [`VISUAL_STYLE_GUIDE.md`](VISUAL_STYLE_GUIDE.md) | **Visual spec — palette, iconography, chrome, typography.** Written 2026-08-29 after task 151 shipped a narrower interpretation of its own reference mockups than intended. Check here before writing any rendering/UI code — it's the transcribed, citable version of what used to live only in `redesign/processed/*.svg` coordinates. |

### `redesign/processed/` — do not read

**Never read anything under `redesign/processed/` as part of normal work**, and
never re-analyse that corpus. Those documents were consumed on 2026-08-27: their
proposals became the 134-169 backlog in `tasks/QUEUE.md`, and every decision that
survived is in the GDD. Reading them costs a large fraction of a session's
context and adds nothing the canonical documents don't already say.

The one exception: a task file that names a specific document as its
`Design source`. Open that document, read that section, stop. Don't read around
it, don't pull in its neighbours, don't reopen decisions the backlog records.

`redesign/abiogenesis-INDEX.md` stays outside `processed/` and is the map of the
corpus — it carries the Phase 0 code findings and the corrections applied to the
original plan. It is short; read it when you need the corpus's shape.

## Conventions

- **Talk to the user in Italian in chat.** Everything written to disk — code, comments, documents, commit messages — stays in English regardless of the chat language.
- **Code, comments, and documents in English.**
- **One module = one Bevy `Plugin`.**
- **The grid is a `Resource`, not ECS entities.** Bevy entities exist only for rendering.
- **The simulation is deterministic and runs headless**: RNG in world state, no `rand::rng()`, no iteration over `HashMap`, no parallel queries in tick logic. `sim`/`world`/`config` don't depend on `bevy::render` or `bevy_egui`.
- **No magic numbers**: all coefficients live in `SimConfig` (`src/config.rs`).

The rationale for these rules is in `TECH_DESIGN.md` §5. Do not work around them: if a task seems to require it, the task is wrong.

## Workflow

Read `PROJECT_WORKFLOW.md` before acting — it defines the task lifecycle, review policy, roles, and document precedence. Follow `docs/CONTEXT_BUDGET_POLICY.md` for context loading and reasoning selection.

Legacy tasks (this project's original single-operator flow, still governing everything already `ACCEPTED`/`[x]`): verify the acceptance criteria in the task file, move it from `tasks/` to `tasks/done/`, update the status to `[x]` in `tasks/QUEUE.md` and `PROJECT_PLAN.md`. New or materially revised tasks use `tasks/TASK_BLUEPRINT.md`'s `QUEUED`/`IN_PROGRESS`/`READY_FOR_REVIEW`/`ACCEPTED` lifecycle instead.

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
5. Run the task's `Validation` plus the project baseline checks from `## Commands` above, unless a command is inapplicable because the task has not yet established the required project artifact. Report any inapplicable command and why.
6. When every required validation passes, record completion according to the task's review policy: `READY_FOR_REVIEW` for `Review: REQUIRED`, `ACCEPTED` for `Review: NOT_REQUIRED`. Make no status change if any validation failed, a required manual check is incomplete, or acceptance criteria are not met.
7. Review the diff to confirm it contains only the assigned task and its required status updates. Create one atomic commit using Conventional Commit style and the task ID. Pushing the task branch is optional per `docs/BRANCHING_POLICY.md` — never required for the workflow to proceed.
8. Report the branch name, commit hash, changed files, acceptance-criteria evidence, validation results, and assumptions. If validation fails or scope is ambiguous, do not commit a partial implementation; report the blocker.

Never run two writing agents concurrently in the same worktree.

### Owner-acceptance workflow

When the developer says `Accept <TASK-ID>` after personally reviewing a `Review: REQUIRED` task, treat it as explicit authorization to skip the agent review and perform only the acceptance-state handoff. Confirm that the task and its canonical queue row are both `READY_FOR_REVIEW`; do not re-review the implementation, rerun validation, change source code, or merge the branch.

Update exactly the task `Status` and its canonical queue row to `ACCEPTED`, and commit only those two edits as `docs: accept <TASK-ID>`, authored under the reviewer-integrator identity defined in `PROJECT_WORKFLOW.md` (`git commit --author="Abiogenesis Reviewer-Integrator <reviewer-integrator@abiogenesis.local>"`) — this project applies the same author override to owner-triggered acceptance as to a reviewer's `APPROVE`. Report the commit. If the required state records are missing or inconsistent, stop and report `BLOCKED`.

## Approach
See `docs/CONTEXT_BUDGET_POLICY.md` for context loading, reasoning profile, and planning/communication rules.
- No sycophantic openers or closing fluff.