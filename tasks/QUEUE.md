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

**Phase 1 — Emergence.** Milestone: true emergence appears; multiple species interact via the matrix (GDD §13).

| Status | ID | Title | Priority | Depends on | Agent | Task File |
|-------|----|--------|----------|------------|--------|-----------|
| `[ ]` | 011 | Hidden matrix generation with cyclicity constraint | 🔴 P1 | 010 | — | [011](011-hidden-matrix-generation.md) |
| `[ ]` | 012 | Adjacency (matrix) effect in the tick | 🔴 P1 | 011 | — | [012](012-matrix-adjacency-tick-effect.md) |
| `[ ]` | 013 | Starting species palette, multiple species per world | 🟡 P2 | 010 | — | [013](013-starting-species-palette.md) |
| `[ ]` | 014 | Predator metabolism | 🟡 P2 | 005 | — | [014](014-predator-metabolism.md) |
| `[ ]` | 015 | Decomposer metabolism and residue cycle | 🟡 P2 | 014 | — | [015](015-decomposer-metabolism.md) |
| `[ ]` | 016 | Environmental diffusion | 🟢 P3 | 004 | — | [016](016-environmental-diffusion.md) |
| `[ ]` | 017 | Seed action with mouse cell selection | 🟡 P2 | 013, 008 | — | [017](017-seed-action-mouse-selection.md) |

Phase 0 is complete (tasks 001-009, archived below); its exit gate is cleared.

Later phases live as backlog in [`PROJECT_PLAN.md`](../PROJECT_PLAN.md) and expand into task files when we get there.

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

---

*Last updated: 2026-08-03*
