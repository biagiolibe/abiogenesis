# Task Queue — Closed Phases Archive

Everything below was fully closed (`[x]`) as of 2026-08-09 and has been moved
out of `tasks/QUEUE.md` to keep that file's per-session read cost down. This
is a reference, not something to re-read at the start of every task — check
it only when you need the history/rationale behind a specific past phase.
`tasks/QUEUE.md` still lists any phase with open (`[ ]`/`[/]`) work.

---

**Mondo vivo, notebook, death legibility, evolution & progression**
(2026-08-11, full scoping pass over five redesign docs from this
session's design discussion — see `PROJECT_PLAN.md` SECTION 1 for the
reasoning and SECTION 2 for the grouped backlog entry; closed 2026-08-12
once 109 unblocked and landed). Dependency order: 096 unblocks 099 and 097
(097 also depends on 103); 098, 100-106, 108 are independently startable;
107 depends on 106; 109 depended on 096-099/106-107 shipping and was
blocked until then.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 096 | Conditional tags: terrain-gated matrix participation | — | [096](done/096-mondo-vivo-conditional-tags-core.md) |
| `[x]` | 097 | Conditional tag catalog badge + (tag, terrain) evidence track | 096, 103 | [097](done/097-mondo-vivo-conditional-tag-catalog-badge.md) |
| `[x]` | 098 | Wild, pre-existing species at world generation | — | [098](done/098-mondo-vivo-wild-species.md) |
| `[x]` | 099 | Reveal-on-first-zone-entry for conditional tags | 096 | [099](done/099-mondo-vivo-zone-entry-reveal.md) |
| `[x]` | 100 | Strip raw per-tick noise from the observation log | — | [100](done/100-notebook-log-rework.md) |
| `[x]` | 101 | Hypothesis grid: reveal-on-first-observation, layout over visible subset only | — | [101](done/101-notebook-grid-visibility-layout.md) |
| `[x]` | 102 | Hypothesis grid: edge grammar rewrite | 101 (soft) | [102](done/102-notebook-grid-edge-grammar.md) |
| `[x]` | 103 | Catalog: one-time metabolism legend, trimmed species rows, population + origin era | — | [103](done/103-notebook-catalog-cleanup.md) |
| `[x]` | 104 | Plain-language death message for player-placed organisms | — | [104](done/104-death-message-plain-language.md) |
| `[x]` | 105 | Cause label on the Biosphere panel for a species taking deaths | 104 | [105](done/105-death-biosphere-trend-diagnosis.md) |
| `[x]` | 106 | Selection pressure accumulation + threshold-crossing trigger | — | [106](done/106-evolution-selection-pressure-trigger.md) |
| `[x]` | 107 | Evolution by speciation: a new descendant species | 106 | [107](done/107-evolution-speciation.md) |
| `[x]` | 108 | Fourth metabolism: chemolithotroph, gain from toxicity | — | [108](done/108-chemolithotroph-metabolism.md) |
| `[x]` | 109 | Long-term objective tier + within-run energy economy | 096-099/106-107 (shipped) | [109](done/109-progression-long-term-objective-energy.md) |

---

**Self-interaction balance bug** (2026-08-10, user-reported live: starting species growing explosively despite task 083's incubation). Root cause traced to `draw_species_tags`'s task-048 mitigation being combinatorially unable to reach `net_self_interaction == 0` in ~15% of worlds given the default 5-tag active pool at world 0 (not a rare edge case — a guaranteed outcome whenever no zero-net 3-tag combination exists at all). 088 fixed the worldgen path with an exhaustive deterministic search (always terminates at exact zero); 089 closed the same gap on the separate Splice path (`apply_splice` never checked this at all).

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 088 | Exhaustive search in `draw_species_tags` to guarantee zero self-interaction | 048 | [088](done/088-exhaustive-self-neutral-species-tags.md) |
| `[x]` | 089 | Reject Splice edits that create a nonzero same-species self-interaction | 048 | [089](done/089-splice-self-interaction-gate.md) |

---

**Environment as sources** (2026-08-10, from `redesign/abiogenesis-environment-sources.md`): replaced the fixed left-right temperature / top-bottom light gradients with per-world heat sources (+ wind bias, + `Sea` cells as passive coolant, + reinjection to counter diffusion erosion) and a per-world sun direction (+ `Mountain` shading). 085 is the combined temperature+light generation task; 086's live playtest found the T/L overlay still skipped `Sea`/peak cells (a stale task-068 rule from the old fixed-gradient model, where those cells carried no interesting data) — under the new source model `Sea` is a real coolant that shapes the field, so skipping it tore a black gap through an otherwise continuous gradient. Fixed by rendering every cell's real scalar. Follow-ups (e.g. a Sea/Mountain coupling pass, falloff retuning) deliberately not pre-planned — filed individually if playtest surfaces them, mirroring 069-072.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 085 | Source-driven temperature and light | none | [085](done/085-source-driven-temperature-and-light.md) |
| `[x]` | 086 | Environment overlay legibility check | 085 | [086](done/086-environment-overlay-legibility-check.md) |

---

**Terrain island-band retune** (2026-08-10, surfaced during the same live playtest session as 085/086): user-reported `Sea` generated "in quantità eccessiva e in forme non troppo credibili" — several same-sized, perfectly isolated near-circular `Sea` blobs scattered inland ("polka-dot lakes"), diluting task 085's heat-source visibility too (more sea meant `sea_coolant_radius` reached more of the map). A throwaway ASCII-dump + 30-seed histogram diagnostic (same technique task 069 itself used) isolated the cause to the island wave band (`island_blend_weight: 0.0` made the pattern vanish entirely) — few summed waves interfere into a regular periodic pattern rather than organic noise. 090 raised `island_wave_count` (6→16) and `island_blend_weight` (0.45→0.55, compensating for the smaller per-wave amplitude at higher count) and lowered `sea_threshold` (0.42→0.34), re-verified against the same histogram technique task 069 used to avoid silently collapsing Mountain/peak reachability.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 090 | Terrain island-band retune: organic coastlines, less sea | 069, 085 | [090](done/090-terrain-island-band-retune.md) |

---

**Bugfixing & UX follow-ups** (2026-08-10, user-reported live during a debugging/planning discussion held after tasks 085-090): three bugs plus three improvements, investigated and scoped in one session. 091 merges two input bugs sharing one root cause — nothing in the codebase checked `bevy_egui`'s own input-capture state (`EguiWantsInput`) before the map's own zoom/click/`Tab` systems ran, so the notebook window didn't block map interaction underneath it and `Tab` fought egui's own keyboard-navigation focus-cycling. 092 is a task-082 side effect: a fixed 30-tick isolation-hint duration no longer tracked the shortened onboarding eras (8 ticks), so it outlived the era it was shown in. 093 reversed a stale task-068 decision — `Sea` deliberately near-black to read as "void" — now that task 085 gave it a real mechanical role. 094 added on-screen buttons for tick/era/notebook controls, coexisting with their keyboard shortcuts via a shared `HudControlIntents` flag resource (reseed stayed keyboard-only, deliberately — destructive, no confirmation affordance). 095 replaced the fixed id-indexed species name scheme with a per-world RNG draw (`world::draw_species_name`, pool 16→49 names) plus a new narrative description alongside the existing stat line — user follow-up noted the description reads flat/repetitive across species, left for a future task if picked up.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 091 | Gate map input (zoom/click/Tab) behind egui's own input capture | none | [091](done/091-egui-input-capture-gating.md) |
| `[x]` | 092 | Isolation hint duration should scale with era length | 082, 055 | [092](done/092-isolation-hint-duration-scales-with-era.md) |
| `[x]` | 093 | Sea should read as water, not "end of the world" | 068, 085 | [093](done/093-sea-color-reads-as-water-not-void.md) |
| `[x]` | 094 | On-screen buttons for tick/era/notebook controls | 030 | [094](done/094-hud-buttons-for-tick-era-notebook-controls.md) |
| `[x]` | 095 | Procedural per-world species names + readable descriptions | 029 | [095](done/095-procedural-species-names-and-descriptions.md) |

---

**Camera pan** (2026-08-10): zoom was already done (075-076); pan was the remaining open half of the old "camera zoom and pan" backlog item. Revisited an earlier explicit design call ("no separate pan mechanic needed") now that the grid is `128×80` and Detail zoom alone made navigation impractical.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 087 | Camera pan | 075 | [087](done/087-camera-pan.md) |

---

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

**From the 2026-08-08 design session**: a decomposer-sustainability balance concern, a bundle of zero-risk UI refinements surfaced while reviewing `abiogenesis-ui-redesign.md`, and an atmospheric background layer (explicit exception to GDD pillar 3) to address the map's "empty black background" feel. The always-on temperature/light background tint idea raised in the same review was deliberately not scoped into a task (needs its own discussion first, see `PROJECT_PLAN.md` §1).

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
| `[x]` | 074 | Final grid size (empirical tuning) | 073 (soft) | [074](done/074-final-grid-size-tuning.md) |

**Two-tier map view, first two task files** (2026-08-09, design discussion held right after task 074's visual check surfaced an organism-legibility gap at 128×80 — full decision record in `redesign/abiogenesis-two-tier-view.md`):

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 075 | Zoom camera and Overview/Detail render-mode switch | 074 | [075](done/075-zoom-camera-overview-detail-switch.md) |
| `[x]` | 076 | Overview mode: per-species cluster heatmap rendering | 075 | [076](done/076-overview-cluster-heatmap-rendering.md) |
| `[x]` | 077 | Gate Stress/Cull to Detail mode; Overview placement indicator for Seed/Splice | 075 | [077](done/077-action-gating-by-view-mode.md) |

**Onboarding grace period** (2026-08-09, design session held right after task 077 closed — full decision record in `/Users/biagioliberto/.claude/plans/rosy-snuggling-lighthouse.md`): a run gave no room to acclimate before real stakes (extinction, era budget, objective failure) kicked in. Two changes: an adaptive grace period that suspends total-extinction failure until the player has watched a living population for a full era (extending past a fixed window rather than cutting off with a cliff), and forcing World 0's opening objective to a gentle 2-species `Coexistence` instead of whatever the random draw picks. Verified live by the user on their own `cargo run`.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 079 | Onboarding: adaptive grace period + softened first-world objective | 040/059 | [079](done/079-onboarding-grace-period.md) |

Phase 0 (001-009) and Phase 1 (010-017) are complete. Phase 2's breakdown came from the 2026-08-03 planning session (see `PROJECT_PLAN.md`'s Phase 2 section for the same list with GDD references). Two independent tracks: 018 → {019, 020} → 021 (notebook/deduction), and 022 → {023, 024, 025} (actions) — both finished. Task 026 was raised by a 2026-08-03 playtest session (see the task file for the specific scenario that surfaced the gap).

---

## Quick Tasks (No File)

Tasks that took < 15 min and didn't need a detailed briefing.

| Status | Description | Priority |
|-------|-------------|----------|
| `[x]` | Dev-only `F1` heatmap overlay for raw environment scalars (temperature/toxicity/light), `#[cfg(debug_assertions)]`-gated so it never ships in release — surfaced by task 023's discovery that toxicity has no in-tick effect and isn't otherwise visible | 🟢 P3 |
| `[x]` | Observation log legibility: `LogEntry` carries its `SpeciesId` so each line gets a `species_color` swatch (matching the Population/Seed Palette pattern), messages use `species_label` instead of raw `species N`, and the scroll area sticks to the newest entry (`stick_to_bottom`) instead of leaving new events off-screen — raised directly by the player as "poco leggibile" | 🟢 P3 |
| `[x]` | Dev-only `F2` per-cell energy-number overlay, `#[cfg(debug_assertions)]`-gated, mirroring the F1 heatmap's toggle pattern — requested to debug unexpected deaths without a hidden-matrix cause | 🟢 P3 |
| `[x]` | Death log lines for player-placed organisms include the energy-update breakdown (gain/matrix/upkeep/crowding/predation) so a death's cause is legible without re-deriving it from the tick code — same motivation as above | 🟢 P3 |
| `[x]` | Bugfix: `Splice`-created species left zero trace in the notebook — `apply_splice` now pushes a `LogEntry` (`text::species_created_message`) the same way extinction/death events already do — raised directly by the player as "quando creo una specie, questa non viene riportata sul notebook" | 🟡 P2 |
| `[x]` | Bugfix: `Decomposer` was structurally unreachable in a single run — `add_bonus_species`'s `i % 2 == 0` parity rule always restarted `i` at 0 on every independent call site, so the shipped default (`extra_available_species_count = 1`) always landed on `Predator`; replaced with a per-slot random draw from the world's seeded RNG — raised directly by the player after 4 cleared worlds with no Decomposer seen | 🟡 P2 |

---

## Archived (Completed) — early tasks

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

**Worldgen pipeline reassessment** (2026-08-13, scoped from
`redesign/procedural_biome_generation_spec_v2.md` after a design-discussion
pass comparing its diagnosis against the current 128×80 pipeline; GDD §5.1
corrected in the same session — the doc's `48×32` grid was stale, task 074
already raised it to `128×80`). Five phases, ordered lowest- to
highest-risk: 123 (organic feature masks) is size-independent and purely
local; 124 (geomorphology fields) is additive-only; 125 (score-based
classification + drainage-based Palude) is the first to change existing
biome output — and, per the 2026-08-13 dependency review, had to land
before 113 (125 is what gives Swamp a `toxicity` source that doesn't
depend on `place_toxic_zone`, which 113 removes); 126/127 (rainfall, then
flow accumulation/rivers) were the highest-value, highest-complexity
phases — 127 in particular carried a real determinism risk (elevation-sort
tie-breaking), resolved via a strict-total-order sort key, see its task
file. 126/127 add fields only; wiring rainfall/rivers into biome
classification or rendering is left as future follow-up in both files'
Non-Goals. Closed 2026-08-19 once 127 landed (task 132, a decision/
correction task on `Cell.slope`/`Cell.water_distance` ordering found
mid-phase, is filed separately below).

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 123 | Organic masks for placed feature biomes (Cratere, Distesa di cristalli, Lago) — `toxic_zone` explicitly out of scope | 111 | [123](done/123-organic-feature-biome-masks.md) |
| `[x]` | 124 | Derived geomorphology fields (`slope`, `water_distance`) — additive only | 110, 111 | [124](done/124-geomorphology-fields.md) |
| `[x]` | 125 | Score-based biome classification; Palude from drainage instead of toxicity | 110, 111, 124 | [125](done/125-biome-score-classification.md) |
| `[x]` | 126 | Rainfall field (orographic lift, rain shadow) — additive only | 124 | [126](done/126-rainfall-field.md) |
| `[x]` | 127 | Flow accumulation and rivers | 124, 126 | [127](done/127-hydrology-rivers.md) |

**Resolved 2026-08-19** (task 132, found in advisor review after 123-126
shipped): `Cell.slope`/`Cell.water_distance` (task 124) were computed every
world generation and read by nothing — both task 125's Swamp score and
task 126's rainfall needed the same kind of data earlier in the pipeline
than these persisted fields were populated. Fixed by splitting
`compute_geomorphology` into `compute_slope` (moved to run right after
`generate_terrain`, no Lake dependency) and `compute_water_distance`
(unchanged, after `place_feature_biomes`) — `classify_biomes` now reads
`Cell.slope` directly; `Cell.water_distance` still can't move earlier
(genuine `Biome::Lake` dependency) and stays local-proxy-only in
`classify_biomes`/`compute_rainfall`. 127/128/130/131 re-checked: all read
these fields from steps that run after both are populated, no conflict.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 132 | [DECISION] Resolve `Cell.slope`/`Cell.water_distance` ordering before later hydrology/biome tasks read them | 124, 125, 126 | [132](done/132-persisted-slope-water-distance-unused.md) |

---

**Worldgen pipeline reassessment — credibility follow-ups** (2026-08-13,
same session: after scoping 123-127, an explicit pass over what the spec
still covers that those five don't. Ranked by impact on a *single* world's
credibility, not variety across worlds — the lower-priority items from that
ranking (world profiles + biome budget/validation, biome transition/blend,
erosion, debug metrics tooling) are intentionally not scoped as tasks;
noted in `VISION.md` instead. Closed 2026-08-19 once 131 landed, completing
the "chain from 128 onward" — 129 reordered the pipeline so
`fill_depressions`/`compute_hydrology` run before `classify_biomes`/
`place_feature_biomes`; 130 reused that reorder to give `classify_biomes`
its first access to `Cell.slope` directly for Mountain sub-banding; 131
built on both, computing `Cell.soil_moisture` from `rainfall`/`is_river`/
`lake_depressions` (all available thanks to 129's reorder) before
`classify_biomes` needs it.)

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 128 | Macro-regions before per-cell biome classification | 125 | [128](done/128-macro-region-biomes.md) |
| `[x]` | 129 | Lakes derived from terrain depressions (123's search becomes fallback) | 123, 127 | [129](done/129-lakes-from-depressions.md) |
| `[x]` | 130 | Mountain sub-banding (Glacier, AlpineMeadow, MountainForest) | 124, 125 | [130](done/130-mountain-sub-banding.md) |
| `[x]` | 131 | Soil moisture (refines Swamp/Forest beyond the slope/water-distance proxy) | 124, 125, 126 | [131](done/131-soil-moisture.md) |

---

**Task 113 follow-ups** (2026-08-19, found in advisor review after 113
shipped): two open decisions, neither blocking 113's own acceptance
criteria — see 133's own file for the full options list and the decisions
made. Gap 1 (SurviveIn's Swamp target has no visual/textual affordance):
resolved by highlighting the active target region with a dashed outline,
reviving the visual language task 113 removed. Gap 2 (the "larger toxic
zones" difficulty axis lost its implementation): resolved by scaling
`swamp_toxicity_min` per-world instead of the removed rectangle's size.
Closed 2026-08-19, same session as 113.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 133 | [DECISION] `SurviveIn`'s Swamp target has no visual/textual affordance; the "larger toxic zones" difficulty axis lost its implementation | 113 | [133](done/133-swamp-survivein-legibility-and-difficulty-scaling.md) |

---

**Balance/persistence** (found while balance-testing task 108). **Rescoped
2026-08-13**: originally targeted `ToxicZoneBounds`; now targets
`Biome::Swamp` membership instead, since task 113 removed `ToxicZoneBounds`
and task 125 moved the toxicity source onto Swamp cells. See 122's own
`tasks/done/` file for the full rescope note and implementation. Closed
2026-08-19: `SimWorld::toxic_swamp_cells` + `reinject_environment_sources`
extended to hold Swamp's toxic sub-region near `swamp_toxicity_value` the
same way heat sources already held temperature — `tests/balance.rs`'s
chemolithotroph test restored to the file's normal 500-tick horizon.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 122 | Swamp toxicity reinjection (toxicity erodes with no source to counter it) | 085, 108, 113, 125 | [122](done/122-toxic-zone-reinjection.md) |
