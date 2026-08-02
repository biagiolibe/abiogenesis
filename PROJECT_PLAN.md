# Project Plan — Abiogenesis

This document tracks the project's evolution from ideas to implementation.

**Vision.** You are a xenobiologist seeding life on alien worlds with **hidden biochemical rules that differ every run**. The game is reverse-engineering them: you seed, you watch an ecosystem live its own life, you form hypotheses, you test them with targeted experiments. The pleasure is the double mystery — what will happen, and what the rules are. Full design in [`abiogenesis-gdd.md`](abiogenesis-gdd.md); architecture in [`TECH_DESIGN.md`](TECH_DESIGN.md).

## Task Lifecycle

```
PROPOSALS  →  (review)  →  BACKLOG  →  (development)  →  DONE
```

| Symbol | Meaning |
|---------|-------------|
| `[ ]`   | Task approved in the backlog |
| `[/]`   | Task in progress |
| `[x]`   | Task completed |
| `[-]`   | Task cancelled / dropped |
| `[?]`   | Proposal (pending review) |

---

## 🗂️ SECTION 1 — PROPOSALS

> Ideas to discuss before moving into the operational backlog.

### Open questions from the GDD (§14)

- `[?]` **Bonus objectives** granting meta-progression currency — planned in principle, but **after** the clean "primary objective → advance" core (GDD §8).
- `[?]` **Meta-progression persistence** (profile/save of unlocks) — deliberately deferred: decided only after verifying the loop is fun (GDD §10).
- `[?]` **Additional metabolisms** beyond the three base ones, e.g. a chemolithotroph tied to `toxicity`, as unlockable content (GDD §5.4).
- `[?]` **Final title** — "Abiogenesis" is a placeholder (GDD §14).

### Born from the move to Bevy (v0.4)

- `[?]` **Config in RON with hot-reload** via `bevy_asset` — GDD §5.6 asks for coefficients "ideally hot-reloadable"; with Bevy this costs little. To be done during the tuning phase, when actually needed.
- `[?]` **Camera zoom and pan** — useful for grids larger than 48×32.
- `[?]` **Real-time mode** as an option — GDD §4 noted it "costs little to add later"; with Bevy it's nearly free (just don't stop at the end of an era).
- `[?]` **Real main menu** with seed selection and sharing — determinism (GDD §5.7) makes sharing interesting seeds worthwhile.

---

## 🔵 SECTION 2 — BACKLOG (Operational)

> Approved tasks. Phase 0 is already expanded into task files; later phases expand when we get there.

### 🏗️ Phase 0 — Walking skeleton

**Milestone:** watch a photolithic species bloom and stabilize thanks to carrying capacity (GDD §13).

- `[x]` 001 — Toolchain, Cargo scaffold, and plugin-based Bevy app → [001](tasks/done/001-scaffold-bevy.md)
- `[x]` 002 — `SimConfig`: centralized coefficients → [002](tasks/done/002-sim-config.md)
- `[x]` 003 — Domain types and `SimWorld` resource → [003](tasks/done/003-domain-simworld.md)
- `[x]` 004 — Environment: static gradients → [004](tasks/done/004-environment-gradients.md)
- `[x]` 005 — Tick algorithm (Phase 0), pure and headless → [005](tasks/done/005-tick-algorithm.md)
- `[x]` 006 — Grid rendering with sprites + 2D camera → [006](tasks/done/006-grid-rendering.md)
- `[x]` 007 — `GameState`/`EraState`, input, animated era → [007](tasks/done/007-states-input-era.md)
- `[x]` 008 — `bevy_egui` HUD → [008](tasks/done/008-hud-egui.md)
- `[x]` 009 — Determinism tests and carrying-capacity validation → [009](tasks/done/009-determinism-balance-tests.md)

### ⚙️ Phase 1 — Emergence

**Milestone:** true emergence appears; multiple species interact via the matrix (GDD §13).

- `[x]` 010 — Tag pool and per-species tag assignment (GDD §5.5) → [010](tasks/done/010-tag-pool-species-tags.md)
- `[ ]` 011 — Hidden matrix generation with cyclicity constraint (GDD §5.5, §5.8) → [011](tasks/011-hidden-matrix-generation.md)
- `[ ]` 012 — Adjacency (matrix) effect in the tick (GDD §5.6, step 3) → [012](tasks/012-matrix-adjacency-tick-effect.md)
- `[ ]` 013 — Starting species palette, multiple species per world → [013](tasks/013-starting-species-palette.md)
- `[ ]` 014 — Predator metabolism (GDD §5.4) → [014](tasks/014-predator-metabolism.md)
- `[ ]` 015 — Decomposer metabolism and residue cycle (GDD §5.4) → [015](tasks/015-decomposer-metabolism.md)
- `[ ]` 016 — Environmental diffusion (GDD §5.2, Phase 1+) → [016](tasks/016-environmental-diffusion.md)
- `[ ]` 017 — Seed action with mouse cell selection (GDD §6) → [017](tasks/017-seed-action-mouse-selection.md)

### 🎨 Phase 2 — Deduction

**Milestone:** the *deduction game* is born, not just the simulation (GDD §13).

- `[ ]` Notebook: egui window with log, `tag × tag` hypothesis grid, catalog (GDD §7)
- `[ ]` Observation log built by consuming simulation events (`TECH_DESIGN.md` §4)
- `[ ]` **Confirmation model** "B with a hint of C": weighted evidence `1/(1+n_confounders)`, threshold `3.0` (GDD §7)
- `[ ]` **Stress / cull / splice** actions (GDD §6)
- `[ ]` Action budget per era: 3 points, differentiated costs; `EraState::Planning` becomes real (GDD §6)

### 🏁 Phase 3 — The run

**Milestone:** a complete game cycle, world after world (GDD §13).

- `[ ]` **Objectives** system per world and satisfaction check (GDD §8)
- `[ ]` **Procedural world generation**: matrix, environment, active tags, species, objectives (GDD §9)
- `[ ]` Failure conditions: total extinction + finite era budget (GDD §8)
- `[ ]` **Difficulty curve**: 5 → ~8 active tags, more hostile environments, shorter budget (GDD §9)
- `[ ]` Run flow: main menu, victory, defeat, transition to the next world
- `[ ]` Minimal meta-progression, without persistence (GDD §10)

### 🎚️ Final tuning — *the real art*

**Goal:** *interesting and readable* emergence, avoiding "everything dies" and "one dominates" (GDD §13, §14).

- `[ ]` Tuning of the three anti-degeneration levers: cyclicity, environmental heterogeneity, carrying capacity (GDD §5.8)
- `[ ]` Tuning of tick coefficients and the notebook confirmation threshold (GDD §5.6, §5.9, §7)
- `[ ]` Final grid size (remains empirical, GDD §5.1)
- `[ ]` Migrate config to RON with hot-reload, to shorten the tuning cycle

---

## 🟡 SECTION 3 — IN PROGRESS

> Tasks currently assigned to agents or in manual development.

- *(none at the moment)*

---

## ✅ SECTION 4 — COMPLETED

### Milestones

- `[x]` Initial concept definition — GDD v0.3, closed design decisions with numeric baseline and playthrough example
- `[x]` Stack choice: Rust + Bevy (ECS), 2D window, egui UI — GDD v0.4
- `[x]` Meridian bootstrap from the GDD: `TECH_DESIGN.md`, backlog, operational queue, Phase 0 task files
- `[x]` Task 001 — Toolchain, Cargo scaffold, and plugin-based Bevy app
- `[x]` Task 002 — `SimConfig`: centralized coefficients
- `[x]` Task 003 — Domain types and `SimWorld` resource
- `[x]` Task 004 — Environment: static gradients
- `[x]` Task 005 — Tick algorithm (Phase 0), pure and headless
- `[x]` Task 006 — Grid rendering with sprites + 2D camera
- `[x]` Task 007 — `GameState`/`EraState`, input, animated era

---

*Last updated: 2026-08-03*
