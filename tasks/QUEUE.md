# Task Execution Queue

This is the operational execution queue. Tasks are ordered by priority.

## How to use this queue

- **Execution**: Take the first available `[ ]` task.
- **Update**: Change `[ ]` to `[/]` when starting, and to `[x]` when finishing.
- **Archiving**: Once completed, move the file to `tasks/done/`.

## Priority

| Code | Meaning |
|--------|-------------|
| 🔴 P1  | Blocking / Critical |
| 🟡 P2  | Important feature |
| 🟢 P3  | Optimization / Polish |

---

## 🤖 How to delegate a task to Claude CLI

```bash
claude "$(cat tasks/NNN-name.md)"$'\n\nExecute this task in the current project.'
```

---

## 🏃 Active Queue

**Phase 2 — Deduction.** Milestone: the *deduction game* is born, not just the simulation (GDD §13).

| Status | ID | Title | Priority | Depends on | Agent | Task File |
|-------|----|--------|----------|------------|--------|-----------|
| `[ ]` | 022 | Action budget economy (`ActionBudget`, `Seed` becomes budget-gated) | 🔴 P1 | — | unassigned | [022](022-action-budget-economy.md) |
| `[ ]` | 023 | Stress action | 🟡 P2 | 022 | unassigned | [023](023-stress-action.md) |
| `[ ]` | 024 | Cull action | 🟡 P2 | 022, 023 | unassigned | [024](024-cull-action.md) |
| `[ ]` | 025 | Splice action | 🟡 P2 | 022 | unassigned | [025](025-splice-action.md) |

Phase 0 (001-009) and Phase 1 (010-017) are complete, archived below. Phase 2's breakdown above comes from the 2026-08-03 planning session (see `PROJECT_PLAN.md`'s Phase 2 section for the same list with GDD references). Two independent tracks: 018 → {019, 020} → 021 (notebook/deduction), and 022 → {023, 024, 025} (actions). Take the first available `[ ]` task and work it per Meridian's "one task at a time" rule: `[ ]` → `[/]` when starting, `[x]` and archived to `done/` when finished.

Later phases (3 and Final tuning) live as backlog in [`PROJECT_PLAN.md`](../PROJECT_PLAN.md) and expand into task files when we get there.

---

## 🧪 Quick Tasks (No File)

Tasks that take < 15 min and don't need a detailed briefing.

| Status | Description | Priority |
|-------|-------------|----------|
| `[ ]` | *(none)* | — |

---

## ✅ Archived (Completed)

| Status | ID | Title | Agent | File |
|-------|----|--------|--------|------|
| `[x]` | 001 | Toolchain, Cargo scaffold, and plugin-based Bevy app | Claude | [001](done/001-scaffold-bevy.md) |
| `[x]` | 002 | `SimConfig`: centralized coefficients | Claude | [002](done/002-sim-config.md) |
| `[x]` | 003 | Domain types and `SimWorld` resource | Claude | [003](done/003-domain-simworld.md) |
| `[x]` | 004 | Environment: static gradients | Claude | [004](done/004-environment-gradients.md) |
| `[x]` | 005 | Tick algorithm (Phase 0), pure and headless | Claude | [005](done/005-tick-algorithm.md) |
| `[x]` | 006 | Grid rendering with sprites + 2D camera | Claude | [006](done/006-grid-rendering.md) |
| `[x]` | 007 | `GameState`/`EraState`, input, animated era | Claude | [007](done/007-states-input-era.md) |
| `[x]` | 008 | `bevy_egui` HUD | Claude | [008](done/008-hud-egui.md) |
| `[x]` | 009 | Determinism tests and carrying-capacity validation | Claude | [009](done/009-determinism-balance-tests.md) |
| `[x]` | 010 | Tag pool and per-species tag assignment | Claude | [010](done/010-tag-pool-species-tags.md) |
| `[x]` | 011 | Hidden matrix generation with cyclicity constraint | Claude | [011](done/011-hidden-matrix-generation.md) |
| `[x]` | 012 | Adjacency (matrix) effect in the tick | Claude | [012](done/012-matrix-adjacency-tick-effect.md) |
| `[x]` | 013 | Starting species palette, multiple species per world | Claude | [013](done/013-starting-species-palette.md) |
| `[x]` | 014 | Predator metabolism | Claude | [014](done/014-predator-metabolism.md) |
| `[x]` | 015 | Decomposer metabolism and residue cycle | Claude | [015](done/015-decomposer-metabolism.md) |
| `[x]` | 016 | Environmental diffusion | Claude | [016](done/016-environmental-diffusion.md) |
| `[x]` | 017 | Seed action with mouse cell selection | Claude | [017](done/017-seed-action-mouse-selection.md) |
| `[x]` | 018 | Simulation event foundation (`OrganismDied`, `SpeciesExtinct`, adjacency observations) | Claude | [018](done/018-simulation-event-foundation.md) |
| `[x]` | 019 | Observation log (notebook window, `tab` toggle) | Claude | [019](done/019-observation-log-notebook-window.md) |
| `[x]` | 020 | Hypothesis confirmation engine (`MatrixKnowledge`, weighted evidence) | Claude | [020](done/020-hypothesis-confirmation-engine.md) |
| `[x]` | 021 | Hypothesis grid UI + tag/species catalog | Claude | [021](done/021-hypothesis-grid-ui-catalog.md) |

---

*Last updated: 2026-08-03*
