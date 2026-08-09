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

### Raised from playtesting (2026-08-08)

- `[?]` **Always-on discrete temperature/light background tint** — surfaced while reviewing `abiogenesis-ui-redesign.md`; needs its own discussion before scoping: it would make the map always show banded temp/light tint by default, which cuts against task 058's explicit choice to keep temperature/light legibility an opt-in `T`/`L` toggle (kept deliberately distinct from the hidden-matrix mystery). Adopting it means reversing or narrowing that opt-in design, not just adding a layer.
- `[?]` **Extend the procedural background layer (task 062) into the map's interior** — task 062 shipped exactly the mechanism its task file specified (a dim sprite strictly behind the grid), verified by a manual playtest with a temporary brightness/saturation bump: the layer is real and correctly regenerates per-seed, but the grid's opaque, exactly-tiled cell sprites fully occlude it everywhere except the thin `AutoMin` letterbox margin a non-3:2 window shows past the grid's edge (near-zero on a window cropped to the grid's own 3:2 aspect). It never reaches the interior "empty black background" that originally motivated the task. Reaching the interior means giving empty cells partial alpha so the layer shows through — task 062 explicitly walled that off ("don't let it creep into organism/cell rendering itself," `cell_color`), so it needs its own scoping discussion: how much alpha, whether it stays legible against `cell.light`'s existing shading (`dark_rows_stay_uninhabited_across_seeds` depends on that gradient reading correctly), and whether the tradeoff against pillar 3 is still worth it once it touches actual cell rendering, not just a layer behind it.

> The other three ideas from this session — decomposer sustainability, a notebook presentation-refinement bundle, and a procedural background layer — were scoped into tasks 060/061/062 (§2).

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
- `[x]` 045 — World-cleared/defeat screens + world transition: shared `start_world` reset function (replaces the ad-hoc `r`-key reset in `input.rs`) → [045](tasks/done/045-world-transition-defeat-screens.md)
- `[x]` 046 — Minimal meta-progression: in-session unlocks (no disk persistence), GDD §10 → [046](tasks/done/046-minimal-meta-progression.md)

> **💡 Design idea (2026-08-03 playtest, not yet scoped into a task):** a mechanism that progressively "reveals" some tag semantics over the course of a run — surfaced during discussion of task 029's naming, but this is a bigger design question than a display fix. Overlaps partly with what the Hypothesis grid already does (confirming a matrix cell *is* a form of progressive reveal, just of behavior, not meaning) — needs more definition before it becomes a task: what would actually be revealed, when, and does it risk collapsing the deduction pillar the same way named tags would (GDD §11). Revisit once Phase 3's difficulty curve is being designed.

### 🐛 Post-Phase-3 playtest fixes (2026-08-06)

Two bugs and two balance/design changes surfaced by playing a full run end to end, all scoped into task files:

- `[x]` 047 — Fix `SurviveIn`'s toxic-zone membership check: `cell_in_zone` checks the live (diffused) `toxicity` scalar instead of the zone's original geometry, so the objective becomes satisfiable by an organism that was never in the zone once diffusion has spread trace toxicity across the grid → [047](tasks/done/047-fix-toxic-zone-membership-check.md)
- `[x]` 048 — Contain runaway population/energy growth from some generated matrices: root cause was `draw_species_tags` only rejecting net-*negative* self-interaction, not net-*positive* — fixed to require exactly `0`. A residual ~10% grid-saturation rate from *cross*-species reinforcement remains (crowding-penalty tuning was tried and rejected: strong enough to matter, it also crushed normal populations) — budgeted for by `tests/balance.rs`'s new `population_never_saturates_the_grid_across_seeds`, the same statistical tolerance model `MAX_EXTINCTION_RATE` already uses → [048](tasks/done/048-contain-runaway-matrix-growth.md)
- `[x]` 049 — Retune sustained objectives (`Coexistence`/`SurviveIn`) to era scale and show eras (not raw ticks) in the HUD: default config values clear in 2 era-presses or less, and the player-facing unit shouldn't be ticks at all (GDD §11) → [049](tasks/done/049-objectives-in-eras-not-ticks.md)
- `[x]` 050 — Remove auto-placed starting organisms: the player seeds the first world via `Seed` instead of the game placing organisms automatically — closer to the game's own premise, also sidesteps `Coexistence` requiring more species than were ever placed. Fixed `is_total_extinction`'s "species exist, nothing placed yet" false positive with a new `SimWorld::ever_populated` flag → [050](tasks/done/050-no-auto-placed-starting-organisms.md)
- `[x]` 051 — Total extinction retries the world, not the whole run: task 050 exposed a design cliff — a single early-seeded organism dying could trip `TotalExtinction` and end the *entire run* over one bad click. New `GameState::WorldFailed` interstitial + `run_flow::retry_world` (rebuilds the same `world_index`/seed) handle `TotalExtinction` without touching `run_progress` or `MetaProgress`; `EraBudgetExhausted` still ends the run via `Defeat` as before → [051](tasks/done/051-total-extinction-retries-world-not-run.md)

### 🌱 First-minutes engagement (2026-08-07)

With the MVP complete, a fresh install still hands the player a silent HUD and an empty grid with no framing: `Menu → Playing` is instant (`menu.rs::start_run`), task 050 removed auto-placed organisms so the grid starts empty with no explanation of `Seed`, and the notebook's confirmation "aha" (GDD §7, the game's core discovery beat) produces zero feedback outside the notebook window itself. Three independent onboarding interventions, none touching `sim`/`world`/`config`:

- `[x]` 052 — Intro screen for the first run: one-time interstitial (new `GameState::Intro`, reuses `screens.rs::interstitial()`) framing the double mystery (emergent ecosystem + hidden matrix) before the first `Playing` state ever, gated by a new `MetaProgress.seen_intro` flag → [052](tasks/done/052-intro-screen-first-run.md)
- `[x]` 053 — In-viewport contextual hints: self-dismissing hints drawn over the grid (not buried in HUD tooltips) guiding the player to place their first organism, then to open the notebook, driven by a new `EverSeeded` flag (not `PlayerPlacedCells`, which empties back out on death) plus a "notebook ever opened" flag → [053](tasks/done/053-in-viewport-contextual-hints.md)
- `[x]` 054 — Celebrate the first confirmed hypothesis-grid cell: `MatrixKnowledge::record` now reports the unconfirmed→confirmed transition, driving a `★` observation-log entry and a HUD badge on the notebook affordance (cleared per-world, on notebook open) → [054](tasks/done/054-celebrate-first-confirmed-hypothesis.md)
- `[x]` 055 — Guided first-isolation hint: on the player's first-ever placement of their first-ever run (`MetaProgress.seen_isolation_hint`), checks isolation via `SimWorld::moore_neighbours` and shows a self-dismissing (30-tick) hint pointing at the confounder-weight formula's reward for isolated experiments — informational only, never a placement gate → [055](tasks/done/055-guided-first-isolation-hint.md)

### 📖 Player-facing documentation (2026-08-07)

- `[x]` 056 — Player guide: a versioned `player_guide.md` manual (what the game is, controls, core loop, actions & costs, notebook/deduction, objectives & difficulty, tips, active-development note), condensed into `text::HOW_TO_PLAY_SECTIONS` and shown automatically on the one-time intro screen before "Begin" (replacing the old, now-redundant `INTRO_BODY` paragraph) plus a "How to play" toggle on the main menu for any later visit, since the intro itself never shows twice → [056](tasks/done/056-player-guide.md)

### 🔬 Species/environment legibility (2026-08-07)

Playtest-driven UX gap raised directly by the user: species info isn't clear in the HUD, the reproduction threshold is invisible outside the debug F2 overlay, the notebook's raw genome floats aren't intuitive, and temperature/light are hard to read on the grid itself. Two independent fixes; the second (temperature/light map encoding) is still under design discussion, not yet a task file.

- `[x]` 057 — Species/reproduction-threshold legibility: surface `repro_threshold` in the HUD Population panel's avg-energy line, and add a human-readable annotation for thermal optimum/tolerance alongside the notebook catalog's raw floats → [057](tasks/done/057-species-reproduction-threshold-legibility.md)
- `[x]` 058 — Player-facing temperature/light overlay toggles: two independent, mutually-exclusive `T`/`L` toggle keys (not `F1`'s dev cycling) reusing `debug_view`'s `heat_color` heatmap, chosen over always-on background tints because they stay legible even if future worldgen turns temperature/light into randomized zones rather than fixed linear gradients → [058](tasks/done/058-temperature-light-overlay-toggles.md)

### 🐛 Second playtest round (2026-08-07)

Four observations from a further playtest session, after 057/058 landed. Two were real bugs, fixed immediately; one is a design question (opened as a proposal, task 059); one is not a bug — light has no per-species preference by design (`gain = light * metabolism_gain * env_fit`, only `env_fit` is temperature-personalized, matching GDD §5.6/§5.9 exactly), explained to the user directly with no code/doc artifact.

- `[x]` Bugfix — `Splice`-created species were invisible in the notebook: no `LogEntry` was ever pushed, unlike every other salient event (deaths, extinctions, matrix confirmations). `apply_splice` now logs the creation via `text::species_created_message`.
- `[x]` Bugfix — `Decomposer` was structurally unreachable within a single run at default config: `add_bonus_species`'s `i % 2 == 0` rule restarted `i` at 0 on every independent call site (`generate_starting_palette`'s fixed slot, `build_world`'s separate meta-progression bonus), so the shipped default (`extra_available_species_count = 1`) always resolved to `Predator`. Replaced with a per-slot random draw from the world's own seeded RNG.
- `[x]` 059 — Sequential per-world objectives: worlds pose 2 objectives at the easy end of the difficulty curve, 3 at the hard end (`WorldParams::objective_count`, ramped like every other field), cleared in order via `CurrentObjective { objectives, index }` — clearing a non-final objective advances `index` and resets `ObjectiveProgress` instead of ending the world, logged via a new `ObjectiveAdvanced` message; no two consecutive objectives share a kind; `era_budget` retuned 40/25 → 60/45 to compensate → [059](tasks/done/059-objective-pacing-design.md)

### 🌌 From today's design session (2026-08-08)

Three proposals from §1 scoped into tasks after discussion; the always-on temperature/light background tint idea raised in the same review stays a `[?]` in §1, needing its own discussion first.

- `[x]` 060 — Ambient residue trickle: a small `EnergyConfig::residue_ambient_trickle` constant, added to every cell's residue each tick (below `residue_decay` so it reaches a small equilibrium, not unbounded growth), so an isolated `Decomposer` doesn't collapse uninformatively before the player can read anything from it — deliberately not enough to make it self-sufficient like Photolithic → [060](tasks/done/060-ambient-residue-trickle.md)
- `[x]` 061 — Notebook presentation refinements: every `AdjacencyObserved` event gets its own log line with a clean/confounded evidence-quality dot (not just confirmations, a deliberate reversal of the log's usual curation for this one case); the hypothesis graph gets a dashed marker for never-observed tag nodes, edge thickness by confirmed magnitude, and numeric labels on strong edges; the notebook catalog gets the species-color swatch the map/HUD/log already share → [061](tasks/done/061-notebook-presentation-refinements.md)
- `[x]` 062 — Procedural alien-world background layer: a dim, code-generated (no art assets) background sprite behind the grid, regenerated per world from `SimWorld::seed`, explicitly scoped as an exception to GDD pillar 3 since it's purely atmospheric and must never carry gameplay signal. Variant derivation used `SimWorld::seed` directly (base hue + noise-wave directions) rather than `WorldParams`, since `WorldParams` isn't stored on `SimWorld` past `new_for_world` and seeding from the world's own seed already gives every world a distinct atmosphere with no extra plumbing → [062](tasks/done/062-procedural-background-layer.md)

### 🖥️ Sidebar console redesign (2026-08-08, from `redesign/abiogenesis-sidebar-redesign.md`)

Full HUD sidebar reskin from a self-contained design doc (with two SVG mockups): one continuous hairline-divided monospace panel instead of four bordered boxes, diegetic English labels, discrete tick indicators instead of progress bars, scrollable Biosphere/Species lists for N species, narrative-styled objective line. The doc's diegetic labels were originally Italian; confirmed directly with the user to translate them to English, then revised again after a first pass (Intervene/Census/Gene bank/Directive) read as too formal/managerial — settled on **Moves / Biosphere / Species / "This world wants"**.

- `[x]` 063 — Population trend indicator, repro-threshold relocation, per-era birth log: fixes a real misleading-UI bug the redesign surfaced — the HUD compared a population *average* energy against `repro_threshold`, an individual-level trait, implying a relationship that isn't there. Moves `repro_threshold` to the notebook catalog (a static per-species trait), replaces the HUD figure with a Rising/Falling/Stable trend vs. the previous era (▲/▼/▬, colored), and adds a per-era birth-count log line (`OrganismBorn`, the real "someone reproduced" signal). Trend snapshots are taken once per era boundary (`EraCompleted`), verified with a temporary debug print against a live playtest to rule out a double-update — same seed showed the average genuinely rise then fall across three eras, not a bug → [063](tasks/done/063-population-trend-and-repro-threshold-relocation.md)
- `[x]` 064 — Sidebar console redesign: the structural/visual rewrite of `hud_panel` — one continuous hairline-divided monospace panel, discrete dot/tick indicators replacing progress bars, an italicized narrative-styled objective line (`RichText::italics()` approximation, no new font asset), and scrollable Biosphere/Species lists for N species. Verification surfaced and fixed two real UX bugs (`HUD_WIDTH` too narrow for monospace text; the horizontal chip strip's scrollbar overlapping the clickable row) and one egui gotcha (`ScrollArea` floors its scrolled axis at `min_scrolled_size = 64.0`pt — the Biosphere row cap is now measured from the panel's own text style instead of a hardcoded guess, keeping it and the "scroll for more" hint threshold consistent) → [064](tasks/done/064-sidebar-console-redesign.md)
- `[x]` 065 — Species list vertical, metabolism glyph, seed relocated: playtest correction to 064, raised directly by the user. The mockup's horizontal chip strip for Species turned out less discoverable in practice than Biosphere's vertical-scroll pattern (its hidden scrollbar needed a dedicated `›` cue just to signal overflow) — switched Species to the same vertical `ScrollArea`/`SCROLL_FOR_MORE` pattern, removing the chip-strip machinery entirely. Each Species row now shows its metabolism (☀/⚔/♻, `render::metabolism_glyph`) since it's a readable GDD trait a fresh player otherwise couldn't see before opening the notebook. The seed number moved from the header to the footer, next to the keyboard hints, matching the mockup's header (which never included it) → [065](tasks/done/065-species-list-vertical-metabolism-seed-relocation.md)

### 🏔️ Terrain map: elevation as real simulation data (2026-08-09, from `redesign/abiogenesis-terrain-map.md`)

The redesign doc originally proposed elevation bands as a visual-only overlay with terrain generation itself out of scope. Design discussion superseded that: elevation becomes a real per-cell dimension (plains/hills/mountains/sea, procedurally generated per world), a possible future factor in evolution alongside others TBD — not a decorative value disconnected from the simulation, per the doc's own point 6. Sea is deliberately *not* hardcoded as permanently unplaceable: a future aquatic species is planned, so gating goes through one centralized `SimWorld` check instead of scattered terrain conditionals. The toxic zone (previously a fixed bottom-right rectangle) becomes variable position/size, generated to always overlap enough placeable land to keep `SurviveIn` satisfiable. Split into three dependency-ordered tasks mirroring the 063→064 data/visual split.

- `[x]` 066 — Terrain field + procedural elevation generation: `Cell` gained `TerrainKind` (Sea/Plain/Hill/Mountain, default `Plain` so every existing test stays placeable) and `is_peak`, generated per world from a summed-plane-wave elevation field on its own derived RNG stream (`self.seed ^ TERRAIN_SEED_OFFSET`, never `world.rng`), bounded-resampled against a configurable minimum placeable-land fraction. `ToxicZoneBounds` became a positionable rectangle (`x0, y0, width, height`, was a corner-anchored `{x0, y0}`); `place_toxic_zone` now searches for a position overlapping enough placeable terrain (own derived stream, `TOXIC_ZONE_SEED_OFFSET`), closing the `SurviveIn`-on-an-all-sea-zone risk flagged during design. All 90 existing tests pass unmodified in logic (only the two `objectives.rs` tests hardcoding a corner-anchored zone needed literal updates); `cargo clippy -- -D warnings` clean → [066](tasks/done/066-terrain-field-procedural-elevation-generation.md)
- `[x]` 067 — Placement gating on terrain: `SimWorld::is_placeable`/`is_placeable_index` are the single centralized check (Sea/peak unplaceable for every species today, deliberately not baking that assumption in anywhere else — a future aquatic species only needs to extend this one function). `seed_organism_on_click`'s core logic was extracted into `attempt_seed` so it's unit-testable without a mouse/window/camera harness (none existed for Seed before this task); reproduction's neighbour filter in `sim.rs` gained the same check. This codebase has no rejected-action feedback mechanism at all (occupied-cell/insufficient-budget Seed clicks are already silent no-ops) — an unplaceable-cell click follows the same convention rather than inventing one → [067](tasks/done/067-placement-gating-on-terrain.md)
- `[x]` 068 — Terrain rendering: elevation bands, boundaries, peak glyphs, toxic-zone dashed overlay: `cell_color`'s empty-cell branch now maps `TerrainKind` to a flat desaturated color per band (Sea stays near-black, still reading as "void"); a new always-on egui painter (`terrain_overlay::draw_terrain_overlay`, mirroring `draw_energy_overlay`'s pattern) draws thin/dark internal boundaries and thicker/lighter Sea↔land coastlines between differently-classed neighbours, a `^` glyph per stored peak cell, and a dashed rectangle around the toxic zone's bounds (adapting `draw_dashed_ring`'s dash-segment technique). `apply_environment_overlay` (T/L toggles) now skips unplaceable cells so the terrain read isn't erased under the heatmap. Verified visually via `cargo run` (menu → world → boundary/toxic-zone dashing/overlay-preservation all confirmed on-screen) → [068](tasks/done/068-terrain-rendering-bands-boundaries-glyphs.md)
- `[x]` 069 — Multi-octave terrain noise (macro-continents + small islands): follow-up raised directly by the user after reviewing 068's rendering, not in the original redesign doc. Task 066's elevation field was single-scale (all waves shared one frequency range), so a world never showed both a large landmass and small separate islands at once. `terrain_waves`/`terrain_elevation` now draw and blend two bands from the same derived RNG stream (`TERRAIN_SEED_OFFSET`, unchanged): a low-frequency continent band (3 waves, 0.8–1.6) shaping macro-continent shape, and a higher-frequency island band (6 waves, 12.0–18.0, blended at 0.45 weight before normalizing) layering small separate island blobs on top — both bands' wave counts, frequency ranges, and blend weight are new `TerrainConfig` fields, no magic numbers. A first tuning pass (island weight 0.65, untouched 0.78/0.88 mountain/peak thresholds) shipped a hidden regression: the new field's narrower variance made Mountain/peak nearly unreachable (a 30-seed histogram, added after an advisor review flagged the risk, showed most individual seeds landing on zero Mountain cells). Caught before archiving; `sea_threshold`/`hill_threshold`/`mountain_threshold`/`peak_elevation_threshold` were retuned (0.32→0.36, 0.55→0.53, 0.78→0.7, 0.88→0.8) to restore reachability — the retuned defaults now produce *more* Mountain cells and peaks over 30 seeds than the pre-069 baseline, not fewer. Verified visually via a real `cargo run` window on the user's machine (driven headlessly with `cliclick`/`screencapture`, several reseeds), not just the histogram/ASCII-dump proxies used mid-investigation; all existing terrain/worldgen tests pass unmodified since they exercise `SimWorld::new`, not the wave functions directly → [069](tasks/done/069-multi-octave-terrain-noise.md)
- `[x]` 070 — Remove task 062's decorative background layer: regression caught by the user from a screenshot right after 068 shipped. Every grid cell is one `Sprite`; occupied cells swap to a mostly-transparent metabolism shape mask (task 032), so the transparent gaps leaked task 062's decorative background (parked behind the grid) instead of the cell's own terrain color — invisible before terrain had distinct colors, now a visible dark/off-color square on every occupied or freshly-reproduced-into cell. User's explicit call: remove 062's layer outright rather than composite-fix it, since 068's terrain colors already solve the "empty black void" problem 062 was added for. `spawn_background`/`sync_background`/`BackgroundTexture`/`background_image`/`background_waves`/`background_field`/`BackgroundWave`/`BACKGROUND_*` deleted from `src/render.rs`; `cargo clippy -- -D warnings` and `cargo test` clean; user confirmed the fix visually → [070](tasks/done/070-remove-decorative-background-layer.md)
- `[x]` 071 — Ambient residue trickle hid terrain colors grid-wide: a second, unrelated regression from the same screenshot thread. Task 060's ambient trickle (`sim::step`) settles every cell's residue at exactly `residue_ambient_trickle` (0.05) after the first tick, grid-wide, not just where something died — `cell_color` treated any `residue > 0.0` as a corpse and painted it brown, taking priority over the terrain branch, so the entire map turned uniformly brown the instant the player pressed `Space` once. Fixed by requiring `residue > residue_ambient_trickle` before the residue color applies. Verified visually (pixel-sampled terrain colors identical before/after two era advances) → [071](tasks/done/071-ambient-residue-trickle-hides-terrain-color.md)
- `[x]` 072 — Terrain sea/land balance correction: direct playtest correction to 069, the user compared generated worlds against `redesign/terrain-map-elevation.svg` and found Sea almost never showed up (measured ~8% of cells over 30 seeds), because `generate_terrain`'s bounded-resample loop only ever rejects a draw for having *too little* land, never for having too little sea — the accepted ensemble was systematically land-heavy. A first retuning pass (higher `sea_threshold`, lower `min_placeable_fraction`) exposed a deeper issue: with only 3 low-frequency continent waves, each world's raw elevation amplitude varies a lot by chance, so fixed thresholds against the raw `[0, 1]` field gave wildly inconsistent sea/land ratios seed-to-seed (some worlds nearly all land, others nearly all sea). Fixed properly with a new `normalize_elevations` step in `generate_terrain` that min-max-rescales each world's own elevation field to fill `[0, 1]` before classification, so `TerrainConfig`'s thresholds land in the same relative place regardless of a given seed's raw amplitude; all four classification thresholds plus `min_placeable_fraction` retuned against the normalized field (Sea ≈ 33% of cells on average, 18–42% spread across a 30-seed sample, vs. the prior near-0%-to-70%+ swings). Along the way, fixed an incidental regression in `sim.rs`'s `world_with_one_organism` test helper (backs 12 pure energy-formula unit tests) — it had been unknowingly depending on seed 42's generated terrain for reproduction placement gating (task 067), so a terrain rebalance silently changed one test's outcome; now forces flat `Plain` terrain, matching task 066's original intent that `Cell::terrain` default to `Plain` so unrelated tests stay terrain-agnostic. Verified visually via a real `cargo run` window on the user's machine (`cliclick`/`screencapture`, several reseeds): Sea now reads as a substantial, clearly visible share of the map, much closer to the mockup → [072](tasks/done/072-terrain-sea-balance-correction.md)

### 🎚️ Final tuning — *the real art*

**Goal:** *interesting and readable* emergence, avoiding "everything dies" and "one dominates" (GDD §13, §14).

> **🐛 Playtest finding (2026-08-04, seed `1231000211577056359`), fixed by task 048 (2026-08-06):** by era 9/tick 225 one species (Kael, species 1) had saturated the entire grid — population 1536 = exactly `48×32`, zero empty cells anywhere — with average energy 1039.53, roughly two orders of magnitude above normal (`seed_energy` 5.0, `repro_threshold` 10.0). Root cause, confirmed with a second independent repro during task 048: `world::draw_species_tags` rejected a candidate tag set that net-*drained* itself (a species dying the moment it reproduces next to itself) but not one that net-*reinforced* itself, and `sim::step`'s `crowd_factor` penalty (`0.15`/neighbour) is dwarfed by a single matrix entry (up to `±2`) — so any species whose own tags reinforced each other turned same-species clustering into unbounded growth, exactly the GDD §14 "one dominates" failure mode. Fixed by requiring `net_self_interaction == 0` instead of merely `>= 0`. A stronger/nonlinear crowding penalty (the other candidate lever noted below) was tried and rejected during the investigation — strong enough to matter for cross-species reinforcement (a residual, smaller-magnitude version of the same failure mode this fix doesn't reach), it also crushed normal populations toward extinction.

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

*Last updated: 2026-08-09 (task 072 complete: `generate_terrain` now min-max normalizes its own elevation field before classification, fixing both a land-heavy bias in the resample loop and wild seed-to-seed sea/land inconsistency; Sea now reads as a substantial, visible share of the map matching `terrain-map-elevation.svg`, confirmed via a real `cargo run` window)*
