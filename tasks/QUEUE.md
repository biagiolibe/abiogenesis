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

**Phase 2 — Deduction** is fully **complete**: core scope (018–025, both tracks), every playtest-driven follow-up (026–028, 030–033), and the localization-prep task (034).

**Phase 3 — The run** is now planned and broken into 12 task files (035–046), from the 2026-08-04 planning session (see `PROJECT_PLAN.md`'s Phase 3 section and the approved plan for the full dependency graph). Task 035 (run/world state foundation) is the only task with no dependencies — start there.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 035 | Run/world state foundation | none | [035](done/035-run-world-state-foundation.md) |
| `[x]` | 036 | `TagSlot` newtype: compiler-driven matrix indexing | none | [036](done/036-tag-slot-newtype.md) |
| `[x]` | 037 | `WorldParams` and difficulty curve | none | [037](done/037-world-params-difficulty-curve.md) |
| `[x]` | 038 | Worldgen: matrix, tag subset, environmental hostility | 036, 037 | [038](done/038-worldgen-matrix-tags-environment.md) |
| `[x]` | 039 | Worldgen: starting species pool | 038 | [039](done/039-worldgen-starting-species-pool.md) |
| `[x]` | 040 | Objectives: type + evaluation engine | none | [040](done/040-objectives-type-evaluation-engine.md) |
| `[x]` | 041 | Failure conditions | 040 | [041](done/041-failure-conditions.md) |
| `[x]` | 042 | Worldgen: per-world objective generation | 038, 040 | [042](done/042-worldgen-objective-generation.md) |
| `[x]` | 043 | Objective HUD | 040 | [043](done/043-objective-hud.md) |
| `[x]` | 044 | Main menu | 035 | [044](done/044-main-menu.md) |
| `[ ]` | 045 | World-cleared/defeat screens + world transition | 035, 038, 039, 040, 041, 042, 044 | [045](045-world-transition-defeat-screens.md) |
| `[ ]` | 046 | Minimal meta-progression | 039, 045 | [046](046-minimal-meta-progression.md) |

Phase 0 (001-009) and Phase 1 (010-017) are complete, archived below. Phase 2's breakdown came from the 2026-08-03 planning session (see `PROJECT_PLAN.md`'s Phase 2 section for the same list with GDD references). Two independent tracks: 018 → {019, 020} → 021 (notebook/deduction), and 022 → {023, 024, 025} (actions) — both finished. Task 026 was raised by a 2026-08-03 playtest session (see the task file for the specific scenario that surfaced the gap).

Final tuning phase still lives as backlog in [`PROJECT_PLAN.md`](../PROJECT_PLAN.md) and expands into task files after Phase 3.

---

## 🧪 Quick Tasks (No File)

Tasks that take < 15 min and don't need a detailed briefing.

| Status | Description | Priority |
|-------|-------------|----------|
| `[x]` | Dev-only `F1` heatmap overlay for raw environment scalars (temperature/toxicity/light), `#[cfg(debug_assertions)]`-gated so it never ships in release — surfaced by task 023's discovery that toxicity has no in-tick effect and isn't otherwise visible | 🟢 P3 |
| `[x]` | Observation log legibility: `LogEntry` carries its `SpeciesId` so each line gets a `species_color` swatch (matching the Population/Seed Palette pattern), messages use `species_label` instead of raw `species N`, and the scroll area sticks to the newest entry (`stick_to_bottom`) instead of leaving new events off-screen — raised directly by the player as "poco leggibile" | 🟢 P3 |
| `[x]` | Dev-only `F2` per-cell energy-number overlay, `#[cfg(debug_assertions)]`-gated, mirroring the F1 heatmap's toggle pattern — requested to debug unexpected deaths without a hidden-matrix cause | 🟢 P3 |
| `[x]` | Death log lines for player-placed organisms include the energy-update breakdown (gain/matrix/upkeep/crowding/predation) so a death's cause is legible without re-deriving it from the tick code — same motivation as above | 🟢 P3 |

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
| `[x]` | 022 | Action budget economy (`ActionBudget`, `Seed` becomes budget-gated) | Claude | [022](done/022-action-budget-economy.md) |
| `[x]` | 023 | Stress action | Claude | [023](done/023-stress-action.md) |
| `[x]` | 024 | Cull action | Claude | [024](done/024-cull-action.md) |
| `[x]` | 025 | Splice action | Claude | [025](done/025-splice-action.md) |
| `[x]` | 026 | Log salient organism deaths, not just extinctions | Claude | [026](done/026-salient-death-logging.md) |
| `[x]` | 027 | Splice: add a real "Add tag" option, not just "Swap" | Claude | [027](done/027-splice-add-tag.md) |
| `[x]` | 029 | Stable tag identifiers and readable species names | Claude | [029](done/029-tag-identifiers-species-names.md) |
| `[x]` | 030 | HUD reorganization: grouping, icons, tooltips, bars | Claude | [030](done/030-hud-reorganization.md) |
| `[x]` | 031 | Hypothesis grid as a graph, not a spreadsheet table | Claude | [031](done/031-hypothesis-grid-as-graph.md) |
| `[x]` | 032 | Distinguish organisms by shape (metabolism), not just color | Claude | [032](done/032-organism-shape-legibility.md) |
| `[x]` | 033 | Render the toxic zone visibly during normal play | Claude | [033](done/033-visible-toxicity.md) |
| `[x]` | 028 | Distinguish "no evidence" from "unconfirmed evidence" in the hypothesis grid | Claude | [028](done/028-partial-evidence-visibility.md) |
| `[x]` | 034 | Centralize player-facing text behind a single `text` module | Claude | [034](done/034-centralize-player-facing-text.md) |
| `[x]` | 035 | Run/world state foundation (`GameState::{WorldCleared,Defeat}`, `RunProgress`, `EraCompleted`) | Claude | [035](done/035-run-world-state-foundation.md) |
| `[x]` | 036 | `TagSlot` newtype: compiler-driven matrix indexing | Claude | [036](done/036-tag-slot-newtype.md) |
| `[x]` | 037 | `WorldParams` and difficulty curve | Claude | [037](done/037-world-params-difficulty-curve.md) |
| `[x]` | 038 | Worldgen: matrix, tag subset, environmental hostility | Claude | [038](done/038-worldgen-matrix-tags-environment.md) |
| `[x]` | 039 | Worldgen: starting species pool | Claude | [039](done/039-worldgen-starting-species-pool.md) |
| `[x]` | 040 | Objectives: type + evaluation engine | Claude | [040](done/040-objectives-type-evaluation-engine.md) |
| `[x]` | 041 | Failure conditions | Claude | [041](done/041-failure-conditions.md) |
| `[x]` | 042 | Worldgen: per-world objective generation | Claude | [042](done/042-worldgen-objective-generation.md) |
| `[x]` | 043 | Objective HUD | Claude | [043](done/043-objective-hud.md) |
| `[x]` | 044 | Main menu | Claude | [044](done/044-main-menu.md) |

---

*Last updated: 2026-08-04 (Phase 3 planning)*
