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

**Phase 0 — Walking skeleton.** Milestone: a photolithic species blooms and stabilizes thanks to carrying capacity (GDD §13).

| Status | ID | Title | Priority | Depends on | Agent | Task File |
|-------|----|--------|----------|------------|--------|-----------|
| `[ ]` | 007 | `GameState`/`EraState`, input, animated era | 🟡 P2 | 005, 006 | — | [007](007-states-input-era.md) |
| `[ ]` | 008 | `bevy_egui` HUD | 🟡 P2 | 007 | — | [008](008-hud-egui.md) |
| `[ ]` | 009 | Determinism tests and carrying-capacity validation | 🟡 P2 | 005 | — | [009](009-determinism-balance-tests.md) |

**Phase 0 exit gate:** do not move to Phase 1 while task 009's tests are red.

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

---

*Last updated: 2026-08-02*
