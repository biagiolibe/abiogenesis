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
- `[x]` 011 — Hidden matrix generation with cyclicity constraint (GDD §5.5, §5.8) → [011](tasks/done/011-hidden-matrix-generation.md)
- `[x]` 012 — Adjacency (matrix) effect in the tick (GDD §5.6, step 3) → [012](tasks/done/012-matrix-adjacency-tick-effect.md)
- `[x]` 013 — Starting species palette, multiple species per world → [013](tasks/done/013-starting-species-palette.md)
- `[x]` 014 — Predator metabolism (GDD §5.4) → [014](tasks/done/014-predator-metabolism.md)
- `[x]` 015 — Decomposer metabolism and residue cycle (GDD §5.4) → [015](tasks/done/015-decomposer-metabolism.md)
- `[x]` 016 — Environmental diffusion (GDD §5.2, Phase 1+) → [016](tasks/done/016-environmental-diffusion.md)
- `[x]` 017 — Seed action with mouse cell selection (GDD §6) → [017](tasks/done/017-seed-action-mouse-selection.md)

### 🎨 Phase 2 — Deduction

**Milestone:** the *deduction game* is born, not just the simulation (GDD §13).

**Track A — notebook and deduction** (018 unlocks 019 and 020; 020 unlocks 021):

- `[x]` 018 — Simulation event foundation: `OrganismDied`, `SpeciesExtinct`, raw adjacency-observation records emitted from `sim::step`/`advance_tick` (`TECH_DESIGN.md` §4)
- `[x]` 019 — Observation log: `notebook` module/plugin, egui window toggled with `tab`, log built by consuming the events from 018 (GDD §7, §11)
- `[x]` 020 — Hypothesis confirmation engine: `MatrixKnowledge` resource, weighted evidence `1/(1+n_confounders)`, threshold `3.0` (GDD §7, §5.9)
- `[x]` 021 — Hypothesis grid UI + tag/species catalog, reading `MatrixKnowledge` from 020 (GDD §7, §11)

**Track B — action budget and new actions** (022 unlocks 023–025):

- `[x]` 022 — Action budget economy: `ActionBudget` resource (3 pts/era baseline), `Seed` becomes budget-gated instead of free; no new `EraState` — `Observing` doubles as observe+plan (GDD §6, §5.9)
- `[x]` 023 — **Stress** action: alter an environmental scalar in an area, cost 1 (GDD §6)
- `[x]` 024 — **Cull** action: remove an organism/species in an area, cost 1 (GDD §6)
- `[x]` 025 — **Splice** action: modify a species' genome (tag or thermal optimum), cost 2 (GDD §6)

**Playtest follow-up** (raised 2026-08-03, after both tracks above shipped):

- `[x]` 026 — Log salient organism deaths, not just extinctions: a player-`Seed`-ed organism dying leaves zero trace in the Notebook today, which a first playtest found disorienting (GDD §7, §11)
- `[x]` 027 — Splice: add a real "Add tag" option, not just "Swap" — a species with room under GDD §5.3's 1-3 tag cap should be able to gain a tag without sacrificing an existing one
- `[x]` 028 (🟢 P3, low priority — revisit later) — Distinguish "no evidence" from "unconfirmed evidence" in the hypothesis grid: a `?` cell today can mean either a truly zero matrix interaction or a real one with too little evidence yet, indistinguishable to the player
- `[x]` 029 — Stable tag identifiers (opaque, e.g. Greek letters — GDD §11 still bars descriptive names) and readable species display names, replacing bare "species N"
- `[x]` 030 — HUD reorganization: visual grouping, icon buttons for actions, a progress bar for the action budget, tooltips — presentation-only restructuring of `ui.rs`, no new information or mechanics
- `[x]` 031 — Hypothesis grid as a graph (tag nodes in a circle, confirmed relationships as colored directed edges) instead of the current `?`/`+!`/`-!` spreadsheet table — same `MatrixKnowledge` data, different rendering
- `[x]` 032 — Distinguish organisms by shape (metabolism), not just color — occupied cells are flat colored squares today, indistinguishable by metabolism without checking the HUD
- `[x]` 033 (bugfix-flavored) — Render the toxic zone visibly during normal play — `cell_color` never reads `toxicity` today; the only way to see it is the dev-only `F1` overlay
- `[x]` 034 — Centralize player-facing text (HUD, notebook, tooltips, event log) behind a single `src/text.rs` module — prep for eventual localization, no real i18n/loader yet

### 🏁 Phase 3 — The run

**Milestone:** a complete game cycle, world after world (GDD §13). Broken into 12 task files (035–046) from the 2026-08-04 planning session — see the approved plan for the full rationale (endless-until-failure model, `TagSlot` refactor, difficulty-curve function). Dependency graph:

```
035 (foundation)
 ├─ 036 (TagSlot) ──────┐
 ├─ 037 (WorldParams) ──┤
 │                      ├─ 038 ── 039 ─────────────┐
 ├─ 040 (objectives) ───┼─ 041                      │
 │                  └── 042 (per-world objective) ──┤
 │                  └── 043 (objective HUD)         │
 └─ 044 (main menu) ─────────────────────────────── 045 (transition) ── 046 (meta-progression)
```

**Foundation:**

- `[x]` 035 — Run/world state foundation: `GameState::{WorldCleared, Defeat}`, `RunProgress` resource, `EraCompleted` event → [035](tasks/done/035-run-world-state-foundation.md)

**Track A — worldgen** (036 and 037 in parallel; both feed 038 → 039):

- `[x]` 036 — `TagSlot` newtype: compiler-driven fix for `TagMatrix`'s contiguous-`TagId` indexing assumption, prerequisite for non-contiguous tag-subset selection (GDD §9) → [036](tasks/done/036-tag-slot-newtype.md)
- `[x]` 037 — `WorldParams` and difficulty curve: pure `world_params(world_index, config)` function (GDD §9; literal acceptance criterion from §16: World 2 has 6 active tags) → [037](tasks/done/037-world-params-difficulty-curve.md)
- `[x]` 038 — Worldgen: matrix, tag subset, environmental hostility, replacing the hardcoded `(0..active_tags_early)` selection and static gradients → [038](tasks/done/038-worldgen-matrix-tags-environment.md)
- `[x]` 039 — Worldgen: starting species pool, replacing the explicit `seed_starting_palette` placeholder → [039](tasks/done/039-worldgen-starting-species-pool.md)

**Track B — run rules** (040 starts right after 035, parallel to Track A):

- `[x]` 040 — Objectives: `Objective` type + evaluation engine (GDD §8 examples: coexistence, toxic-zone survival, bloom trigger) → [040](tasks/done/040-objectives-type-evaluation-engine.md)
- `[x]` 041 — Failure conditions: total extinction + era-budget-per-world exhaustion (GDD §8) → [041](tasks/done/041-failure-conditions.md)
- `[x]` 042 — Worldgen: per-world objective generation and severity scaling → [042](tasks/done/042-worldgen-objective-generation.md)
- `[x]` 043 — Objective HUD, filling the `ui.rs:243` placeholder (GDD §11) → [043](tasks/done/043-objective-hud.md)

**Track C — shell and convergence:**

- `[x]` 044 — Main menu: wires `GameState::MainMenu`, generates `run_seed`, the one legitimate point outside the sim where run variety originates → [044](tasks/done/044-main-menu.md)
- `[ ]` 045 — World-cleared/defeat screens + world transition: shared `start_world` reset function (replaces the ad-hoc `r`-key reset in `input.rs`) → [045](tasks/045-world-transition-defeat-screens.md)
- `[ ]` 046 — Minimal meta-progression: in-session unlocks (no disk persistence), GDD §10 → [046](tasks/046-minimal-meta-progression.md)

> **💡 Design idea (2026-08-03 playtest, not yet scoped into a task):** a mechanism that progressively "reveals" some tag semantics over the course of a run — surfaced during discussion of task 029's naming, but this is a bigger design question than a display fix. Overlaps partly with what the Hypothesis grid already does (confirming a matrix cell *is* a form of progressive reveal, just of behavior, not meaning) — needs more definition before it becomes a task: what would actually be revealed, when, and does it risk collapsing the deduction pillar the same way named tags would (GDD §11). Revisit once Phase 3's difficulty curve is being designed.

### 🎚️ Final tuning — *the real art*

**Goal:** *interesting and readable* emergence, avoiding "everything dies" and "one dominates" (GDD §13, §14).

> **🐛 Playtest finding (2026-08-04, seed `1231000211577056359`):** by era 9/tick 225 one species (Kael, species 1) had saturated the entire grid — population 1536 = exactly `48×32`, zero empty cells anywhere — with average energy 1039.53, roughly two orders of magnitude above normal (`seed_energy` 5.0, `repro_threshold` 10.0). Root cause: `sim::step`'s energy update has no upper cap (only a `<= 0.0` death floor) — with a randomly generated matrix where two of Kael's own tags reinforce each other strongly and positively, same-species neighbours (up to 8, once the grid is saturated) push `interaction_delta` well past what `crowd_factor`'s penalty can offset, so growth never stalls at a carrying-capacity plateau the way `sim::tests::crowded_photolithic_stalls_at_carrying_capacity` expects for a *neutral* matrix. This is exactly the GDD §14 "one dominates" failure mode this section already anticipates, not a new mechanic — task 038's move to a fully-random active-tag draw from the whole pool (vs. the old fixed first-5) widens the seed space that can hit a self-reinforcing pair, likely making this easier to encounter than before. Repro: seed `1231000211577056359`, run to era ~9. Relevant when picking at the anti-degeneration levers below — an energy cap or a stronger/nonlinear crowding penalty are candidate levers, not yet decided.

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

*Last updated: 2026-08-04*
