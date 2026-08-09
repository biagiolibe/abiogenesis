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

**Phase 3 — The run** is fully **complete**: all 12 task files (035–046), from the 2026-08-04 planning session (see `PROJECT_PLAN.md`'s Phase 3 section for the full dependency graph). The game now has a real main menu, procedurally generated worlds with objectives, world-cleared/defeat transitions, and light meta-progression.

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
| `[x]` | 045 | World-cleared/defeat screens + world transition | 035, 038, 039, 040, 041, 042, 044 | [045](done/045-world-transition-defeat-screens.md) |
| `[x]` | 046 | Minimal meta-progression | 039, 045 | [046](done/046-minimal-meta-progression.md) |

**Post-Phase-3 playtest fixes** (2026-08-06 session): two bugs and two balance/design changes surfaced by playing a full run.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 047 | Fix `SurviveIn`'s toxic-zone membership check (diffusion leaks the zone check to the whole grid) | none | [047](done/047-fix-toxic-zone-membership-check.md) |
| `[x]` | 048 | Contain runaway population/energy growth from some generated matrices | none | [048](done/048-contain-runaway-matrix-growth.md) |
| `[x]` | 049 | Retune sustained objectives to era scale, show eras not ticks in the HUD | none | [049](done/049-objectives-in-eras-not-ticks.md) |
| `[x]` | 050 | Remove auto-placed starting organisms; the player seeds the first world | none | [050](done/050-no-auto-placed-starting-organisms.md) |
| `[x]` | 051 | Total extinction retries the world, not the whole run | 050 | [051](done/051-total-extinction-retries-world-not-run.md) |

**First-minutes engagement** (2026-08-07 design session): the MVP is complete but the opening minutes leave a fresh player facing a silent HUD and an empty grid. Three independent onboarding interventions, plus a fourth (055) added in a same-day follow-up on pacing/guided-evidence design.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 052 | Intro screen for the first run | none | [052](done/052-intro-screen-first-run.md) |
| `[x]` | 053 | In-viewport contextual hints for the first actions | none | [053](done/053-in-viewport-contextual-hints.md) |
| `[x]` | 054 | Celebrate the first confirmed hypothesis-grid cell | none | [054](done/054-celebrate-first-confirmed-hypothesis.md) |
| `[x]` | 055 | Guided first-isolation hint | 053 | [055](done/055-guided-first-isolation-hint.md) |

**Player-facing documentation** (2026-08-07, requested directly by the user, independent of the design sessions above):

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 056 | Player guide (manual + in-game "How to play" panel) | none | [056](done/056-player-guide.md) |

**Species/environment legibility** (2026-08-07, playtest-driven UX gap raised directly by the user: species info unclear, reproduction threshold invisible outside debug overlays, temperature/light hard to read on the map):

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 057 | Species/reproduction-threshold legibility (Population panel + notebook catalog) | none | [057](done/057-species-reproduction-threshold-legibility.md) |
| `[x]` | 058 | Player-facing temperature/light overlay toggles (independent `T`/`L` keys, not F1 cycling) | none | [058](done/058-temperature-light-overlay-toggles.md) |

**Second playtest round** (2026-08-07, same-day follow-up after 057/058 landed): two real bugs fixed immediately (notebook silent on Splice-created species; Decomposer structurally unreachable in a single run), one design question opened as a proposal then resolved into an approved task after a follow-up design discussion (objective pacing), one non-issue confirmed by design (no per-species light preference exists, explained to the user directly, no artifact).

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 059 | Sequential per-world objectives (2 → 3 across the difficulty curve), era budget retuned to compensate | none | [059](done/059-objective-pacing-design.md) |

**From today's design session** (2026-08-08): a decomposer-sustainability balance concern, a bundle of zero-risk UI refinements surfaced while reviewing `abiogenesis-ui-redesign.md`, and an atmospheric background layer (explicit exception to GDD pillar 3) to address the map's "empty black background" feel. The always-on temperature/light background tint idea raised in the same review was deliberately not scoped into a task (needs its own discussion first, see `PROJECT_PLAN.md` §1).

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 060 | Ambient residue trickle so an isolated Decomposer doesn't collapse outright | none | [060](done/060-ambient-residue-trickle.md) |
| `[x]` | 061 | Notebook presentation refinements (evidence-quality log, graph polish, catalog color) | none | [061](done/061-notebook-presentation-refinements.md) |
| `[x]` | 062 | Procedural alien-world background layer | none | [062](done/062-procedural-background-layer.md) |

**Sidebar console redesign** (2026-08-08, from `redesign/abiogenesis-sidebar-redesign.md`, a self-contained design doc with two SVG mockups): a full HUD sidebar reskin — one continuous hairline-divided monospace panel instead of four bordered boxes, diegetic English labels (Moves/Biosphere/Species/"This world wants" — revised from a first, too-formal English pass), discrete tick indicators instead of progress bars, scrollable Biosphere/Species lists for N species, and a narrative-styled objective line. Split into a data-correctness prerequisite (063) and the visual/structural rewrite that consumes it (064).

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 063 | Population trend indicator, repro-threshold relocation, per-era birth log | none | [063](done/063-population-trend-and-repro-threshold-relocation.md) |
| `[x]` | 064 | Sidebar console redesign | 063 | [064](done/064-sidebar-console-redesign.md) |
| `[x]` | 065 | Species list vertical, metabolism glyph, seed relocated (playtest correction to 064) | 064 | [065](done/065-species-list-vertical-metabolism-seed-relocation.md) |

**Terrain map** (2026-08-09, from `redesign/abiogenesis-terrain-map.md`): elevation becomes real per-cell simulation data (plains/hills/mountains/sea, procedurally generated per world), not a decorative visual seed — a possible future factor in evolution. Sea is deliberately not hardcoded as permanently unplaceable (a future aquatic species is planned); placement gating goes through a single centralized check. The toxic zone becomes variable position/size, guaranteed to overlap placeable land so `SurviveIn` stays satisfiable. Split into a data/worldgen task, a placement-gating task, and a rendering task, mirroring the 063→064 pattern.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 066 | Terrain field + procedural elevation generation | none | [066](done/066-terrain-field-procedural-elevation-generation.md) |
| `[x]` | 067 | Placement gating on terrain | 066 | [067](done/067-placement-gating-on-terrain.md) |
| `[x]` | 068 | Terrain rendering: elevation bands, boundaries, peak glyphs, toxic-zone overlay | 066, 067 | [068](done/068-terrain-rendering-bands-boundaries-glyphs.md) |
| `[x]` | 069 | Multi-octave terrain noise (macro-continents + small islands) | 066, 067, 068 | [069](done/069-multi-octave-terrain-noise.md) |
| `[x]` | 070 | Remove task 062's decorative background layer (superseded by terrain colors, leaked through organism shape masks) | 062, 066, 067, 068 | [070](done/070-remove-decorative-background-layer.md) |
| `[x]` | 071 | Ambient residue trickle hid terrain colors grid-wide after the first era advance | 060, 068, 070 | [071](done/071-ambient-residue-trickle-hides-terrain-color.md) |
| `[x]` | 072 | Terrain sea/land balance correction (playtest correction to 069, matching `terrain-map-elevation.svg`) | 069 | [072](done/072-terrain-sea-balance-correction.md) |

**Final tuning kickoff** (2026-08-09, from `PROJECT_PLAN.md`'s Final tuning backlog): the user chose to tackle grid size and the RON config migration next. RON migration goes first since it's what makes iterating on grid size (and every other tuning task after it) fast without recompiling.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 073 | Migrate `SimConfig` to a hot-reloadable RON asset | none | [073](done/073-ron-config-hot-reload.md) |
| `[ ]` | 074 | Final grid size (empirical tuning) | 073 (soft) | [074](074-final-grid-size-tuning.md) |

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
| `[x]` | Bugfix: `Splice`-created species left zero trace in the notebook — `apply_splice` now pushes a `LogEntry` (`text::species_created_message`) the same way extinction/death events already do — raised directly by the player as "quando creo una specie, questa non viene riportata sul notebook" | 🟡 P2 |
| `[x]` | Bugfix: `Decomposer` was structurally unreachable in a single run — `add_bonus_species`'s `i % 2 == 0` parity rule always restarted `i` at 0 on every independent call site, so the shipped default (`extra_available_species_count = 1`) always landed on `Predator`; replaced with a per-slot random draw from the world's seeded RNG — raised directly by the player after 4 cleared worlds with no Decomposer seen | 🟡 P2 |

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
| `[x]` | 045 | World-cleared/defeat screens + world transition | Claude | [045](done/045-world-transition-defeat-screens.md) |
| `[x]` | 046 | Minimal meta-progression | Claude | [046](done/046-minimal-meta-progression.md) |

---

*Last updated: 2026-08-09 (task 073 complete: `SimConfig` and every nested config struct now derive `serde::{Serialize, Deserialize}`, loaded from `assets/config/sim_config.ron` via `bevy_common_assets`' `RonAssetPlugin` with `bevy`'s `file_watcher` feature enabled; a sync system keeps the live `SimConfig` resource current on every reload — verified hot-reload live via `cargo run`, no restart needed. 074 — final grid size — is next, now unblocked for fast iteration)*
