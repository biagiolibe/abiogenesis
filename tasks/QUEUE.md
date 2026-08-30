# Task Execution Queue

This is the operational execution queue. Tasks are ordered by priority.

Closed phases (everything fully `[x]`) live in
[`QUEUE_ARCHIVE.md`](QUEUE_ARCHIVE.md), not here — this file only tracks
work with something still open, to keep the per-session read cost down.
Check the archive when you need the history/rationale behind a past phase,
not by default.

## How to use this queue

- **Execution**: Take the first available `[ ]` task.
- **Update**: Change `[ ]` to `[/]` when starting, and to `[x]` when finishing.
- **Archiving**: Once completed, move the file to `tasks/done/`. Once an
  entire phase/section below is fully `[x]`, move its rows to
  `QUEUE_ARCHIVE.md`.

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

---

# 🌍 Culture Shock redesign rollout (134-169)

Adoption of the whole `redesign/processed/` corpus (25 design documents + GDD v0.7 +
`abiogenesis-system-hierarchy.md`), sequenced 2026-08-27. Planning record:
`~/.claude/plans/leggi-redesign-abiogenesis-index-md-e-va-composed-rose.md`.
`redesign/abiogenesis-INDEX.md` is the map of the documents; this queue is the
executable version of it, and it **corrects the INDEX on three points** (all
annotated in the INDEX itself):

1. The per-cell population model obsoletes tasks 076/078 — the Overview's
   hole-filling and erosion get removed, not adapted (task 139).
2. The time scale comes **before** the energy retune, reversing the INDEX's
   order — both documents declare a dependency on the other, and the clock is
   the independent variable.
3. `culture-shock-population-model-aesthetic.md` splits into two tasks in two
   phases: the simulation model in Phase 1 (137), the pixel-grain visual
   register in Phase 2 (151), so the HUD isn't restyled and then rebuilt.

**Task files exist only for Phase 1.** Later phases are backlog rows here and in
`PROJECT_PLAN.md`; each becomes a task file when the previous phase closes.

## Phase 0 — verifications, closed 2026-08-27, no task

- **No `interaction_delta` scale coefficient exists.** `sim.rs:715` sums the raw
  `{−2..+2}` entries; `sim.rs:766` adds them straight to energy. A new
  `EnergyConfig::interaction_scale` is needed (task 136).
- **Action scope: per-cell for 3 of 4.** Seed/Stress/Cull are per-cell; **Splice
  is species-scoped**, and `input.rs:605` already does
  `world.push_species(new_species)` — so `abiogenesis-actions.md`'s "Splice
  creates a new species to be seeded separately" is *already implemented*. GDD §6
  prose ("in an area") still needs correcting.
- **Third finding, not asked for and the most important**: the matrix is
  ignorable *by construction*. `generate_matrix` zeroes the diagonal and
  `draw_species_tags` forces `net_self_interaction == 0`, so inside a
  single-species blob `interaction_delta` is exactly zero. The matrix only acts
  at interfaces between different species. Full writeup in task 136.

## Phase 1 — the central loop

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 134 | Two-bot harness (exploiter vs explorer) — pre-change baseline | — | [134](done/134-two-bot-experiment-incentive-harness.md) |
| `[x]` | 134b | Make the bot policies competent, re-record the baseline | 134 | [134b](done/134b-competent-bot-policies-and-rebaseline.md) |
| `[x]` | 135 | Three-level time scale: Pulse → Season → Era | — | [135](done/135-three-level-time-scale.md) |
| `[x]` | 136 | Make the hidden matrix necessary (interaction_scale + retune) | 134b, 135 | [136](done/136-matrix-necessary-balance.md) |
| `[x]` | 136b | Evidence per distinct observation, not per tick | 136 | [136b](done/136b-evidence-per-distinct-observation.md) |
| `[x]` | 137 | Per-cell population model | 136 | [137](done/137-per-cell-population-model.md) |
| `[x]` | 138 | Tick as an explicit phased pipeline | 137 | [138](done/138-tick-pipeline-explicit-phases.md) |
| `[x]` | 139 | Overview: real density, remove the pictorial version | 137 | [139](done/139-overview-real-density.md) |
| `[x]` | 140 | End-of-era reveal beat + evolution applied there | 135 | [140](done/140-end-of-era-reveal-beat.md) |

> **134 landed 2026-08-27; its diagnosis was corrected 2026-08-28 and 136's
> scope changed with it.** Head to head the exploiter is ahead (faster on 9 of
> 17 comparable seeds, explorer on 4, 4 tied). The first reading of the low
> `pairs confirmed` figures — "the matrix is nearly unobtainable" — was wrong,
> and rested on a threshold of `3.0` taken from a unit-test constant; the
> shipped `confirmation_threshold` is **1.0** against a numerator of 1.0, so one
> unconfounded observation confirms a pair outright. A direct diagnostic (12
> seeds, 30 eras, seeding greedily) found the opposite of the original claim:
> **12.2M `AdjacencyObserved` events** and **43% of every confirmable pair
> confirmed** by a bot making no attempt to observe anything. The bots' low
> figures were a property of their own passivity, not of the game.
>
> Two defects survive that correction, and the design corpus names only the
> first:
>
> 1. **The matrix doesn't matter.** Only **1.8%** of occupied Moore adjacencies
>    are cross-species (302,800 against 16,154,913 same-species). With
>    `net_self_interaction == 0` forced, the other 98.2% contribute exactly
>    zero. The hidden layer is switched off almost everywhere on the grid.
> 2. **The matrix isn't earned.** Evidence accrues per organism, per
>    neighbour-tag, *per tick*: a blob sitting still for 200 ticks emits the
>    same observation 200 times. Scientifically that is one data point repeated,
>    not 200 — so confirmation is a function of population, and the way to
>    decode the world is to flood it, not to design an experiment. Exactly
>    backwards from GDD §7, where the isolated observation is the valuable one.
>    Clean observations (`n_confounders == 0`) are **577 of 12.2M**, 0.005%.
>
> Defect 1 is task 136's existing job, now with a measurable target: raise the
> cross-species contact fraction. Defect 2 is task **136b**, deliberately placed
> *before* the playtest checkpoint even though 138 rewrites the same emission
> site — reaching the checkpoint with an evidence economy that rewards flooding
> would falsify exactly the judgement the checkpoint exists to make.
>
> A third consequence: 134's own baseline is not fit to verify 136 against. Its
> bots reach dozens of organisms where the diagnostic reached thousands, so they
> measure a strategy nobody would play. Task **134b** makes them competent and
> re-records the baseline before 136 uses it. Full numbers in 134's task file.

**→ CHECKPOINT before Phase 1b: play it, and re-run 134 against its baseline.**
The INDEX is right that this is the highest information-per-hour moment in the
whole plan — it is where you find out whether the game is fun, before investing
in everything else. Do not skip it.

Phase 1b (141-144, anti-friction interventions) closed 2026-08-28 and is
archived in `tasks/QUEUE_ARCHIVE.md`. **Recommended before Phase 2:** replay
`culture-shock-naive-player-example.md`'s scenario with all four applied,
ideally with a real playtester — not yet done.

## Phase 2 — making the loop legible

| Status | ID | Title | Doc |
|-------|----|--------|-----|
| `[x]` | 145 | Stress on three selectable axes + gradual decay | `actions` — [145](done/145-stress-three-axes-gradual-decay.md) |
| `[x]` | 146 | Cull emits a tracked notebook observation | `actions` — [146](done/146-cull-emits-tracked-observation.md) |
| `[x]` | 147 | Splice restricted to confirmed traits + growing genome bank + "synthesised" origin | `actions`, `hud-notebook` — [147](done/147-splice-confirmed-traits-genome-bank.md) |
| `[x]` | 149 | Inspection tool: hover tooltip + per-neighbour energy breakdown card | `inspect-tool` — [149](done/149-inspection-tool.md) |
| `[x]` | 150 | Full control scheme + `Esc` cascade + pause menu + protected `R` | `controls` — [150](done/150-control-scheme-pause-menu.md) |
| `[x]` | 151 | Pixel-grain visual register across map, HUD and notebook | `population-model-aesthetic`, `ui-redesign` — [151](done/151-pixel-grain-visual-register.md) |
| `[x]` | 152 | HUD: auto-advance toggle, mutation-tier badge, species subtext (title corrected — `sidebar-redesign` already shipped as 064/065, this is `hud-notebook`'s residual gap) | `hud-notebook` — [152](done/152-hud-sidebar-diegetic-redesign.md) |
| `[x]` | 153 | Notebook: Chronicle section + "descends from" (node graph already shipped, see file) | `notebook-cronaca` — [153](done/153-notebook-node-graph-chronicle.md) |
| `[x]` | 154 | Objectives correctness pass (154a): victory as a flag, Speciation activation snapshot, immediate re-check during Reveal | `objectives` — [154](done/154-objectives-activation-victory-flag.md) |
| `[x]` | 178 | Objectives tuning: Coexistence population floor + durations in seasons | `objectives`, `playtest_outcome.md` I.6 — [178](done/178-objectives-tuning-new-types-durations.md) |
| `[x]` | 179 | Objectives: 4 new types (Homeostasis/Tolerance/WildCoexistence/Rootedness), Speciation target-species narrowing (FirstConfirmation deferred, needs `MatrixKnowledge`) | `objectives` — [179](done/179-objectives-new-types-target-species.md) |
| `[x]` | 170 | Speciation cause readability: surface dominant pressure stimulus + genome before/after diff | GDD §5.11 — [170](done/170-speciation-cause-readability.md) |
| `[/]` | 171 | Causal-legibility playtest gate: bot-vs-bot necessity check (done, pass) + human playtest protocol (written, **not yet run**), gates Phase 3 | GDD §5.8/§5.9 — [171](171-causal-legibility-playtest-gate.md), [results](171-results.md) |

**All 10 Phase 2 task files now exist (2026-08-29).** Suggested execution
order, from cross-task dependencies each file's own scoping surfaced (not a
strict requirement, but avoids rework):

1. **145** (Stress axes) — no dependencies, already scoped first.
2. **146, 147, 170** — independent of each other and of everything else
   below; each extends an existing system (Cull's evidence hookup, Splice's
   trait filter, the speciation reveal) without touching UI structure.
3. **150** (control scheme) before **152** (HUD) — 152 explicitly claims the
   `p`/continuous-advance keybind that 150 deliberately punted; run 150
   first so 152 isn't guessing at 150's final key-handling shape. **149**
   (inspection tool) also soft-depends on 150 for the click-to-inspect vs.
   armed-action conflict.
4. **153** (notebook Chronicle) — independent, can run anywhere after 145.
5. **154** (objectives) — independent but the largest (~3-4h); split into
   154 (delivered, `tasks/done/`) and 178 (remainder, not yet started) on
   2026-08-29, per the file itself's own suggestion.
6. **151** (pixel-grain restyle) — deliberately **last**: restyles whatever
   UI surface 149/150/152/153 add, per the aesthetic doc's own phase-split
   rationale.
7. **171** (causal-legibility gate) — deliberately **last of all**: its
   bot-vs-bot half specifically depends on 146/147/170 (what a bot can read
   changes), and it's the Phase-3 gate, so nothing about running it early
   helps.

⚠️ Apply the colour-accessibility rule ("colour is never the only channel",
`cross-cutting` §3) **while building** this phase. Retrofitting costs far more.

⚠️ **Gate before Phase 3**: task 171 formalizes a bot-vs-bot necessity check and
a human playtest protocol. Don't start Phase 3 content work casually before
running it — see task file for rationale.

**171 status (2026-08-29): bot-vs-bot half done, verdict pass** (see
[`171-results.md`](171-results.md) §1) — no evidence the surfaced data is
insufficient. **Human playtest half still owed**: the protocol is written
and handoff-ready (§2 of the results file) but no real playtester was
available this session, mirroring how Phase 1's own skipped checkpoint was
tracked. 171 stays `[/]`, and Phase 3 stays informally gated, until a real
playtest run happens and its findings are appended to the results file.

Already done, do not redo: tick→pulse rename (118), Biosphere numeric delta
(120), era-relative time readout (117), Splice-creates-a-new-species (see
Phase 0).

Not planned: **Isola/Quarantena** as a fifth action — only its attachment point
in the pipeline (task 138, phase 3). **Sposta** — the document itself does not
recommend it. Both stay deferred.

## Phase 2 — playtest fixes (opened 2026-08-29)

First human playtest run, logged in `playtest_outcome.md` (kept at repo
root, not moved — source evidence for these tasks). Not part of the
`redesign/processed/` corpus rollout; independent bug/UX fixes the run
surfaced, verified against the current code before filing. **Correction
(post advisor-review, 2026-08-29): the original note here claiming two
playtest findings were already covered by task 154's own scope was wrong**
— 154's AC never touched `Coexistence`'s missing population floor
(issue I.6), and 154's `Speciation`-snapshot fix was a different bug from
the evolution-reveal-timing one (gameplay #17). #17 was folded into 154's
delivered scope (154a, `tasks/done/154-*.md`); I.6 landed in 178
(`tasks/done/178-*.md`, population floor + durations in seasons); the
5-new-types/target-species-narrowing remainder moved on to
[179](179-objectives-new-types-target-species.md).
None of Phase 2's playtest-fix tasks gate task 171's Phase-3 gate, but
172/174/175 are worth doing before running 171's human playtest protocol
since they affect what a fresh player notices/attributes correctly.

| Status | ID | Title | Doc |
|-------|----|--------|-----|
| `[x]` | 172 | Inspection tool UX fixes: stable tooltip sizing, cursor-positioned card, biome info when populated | `playtest_outcome.md` I.1 — [172](done/172-inspection-tool-ux-fixes.md) |
| `[x]` | 173 | Sidebar species/biosphere lists don't fill panel width | `playtest_outcome.md` I.3 — [173](done/173-sidebar-list-width-fix.md) |
| `[x]` | 174 | Align toxicity label thresholds to visual tint; clarify population tooltip scope | `playtest_outcome.md` I.7 — [174](done/174-toxicity-label-population-tooltip-clarity.md) |
| `[x]` | 175 | New species from speciation should place near where pressure actually accrued | `playtest_outcome.md` #10 — [175](done/175-speciation-placement-near-parent.md) |
| `[x]` | 176 | Continuous-advance needs its own, slower cadence | `playtest_outcome.md` I.11 — [176](done/176-continuous-advance-dedicated-cadence.md) |
| `[x]` | 177 | No armed action at world start; pan with wheel held down | `playtest_outcome.md` I.2, F.1 — [177](done/177-start-default-action-and-wheel-pan.md) |
| `[x]` | 180 | HUD chrome fidelity: action-button block icons (replacing broken emoji), box chrome, neutral-icon Biosphere rows, section-label style — governing rule: color = state, never identity. Live-verified 2026-08-30; also found and fixed a real pre-existing bug it surfaced (circle/cross metabolism shapes collapsing to indistinguishable squares at block-snap coarseness — see task file) | `population-model-aesthetic` — [180](done/180-pixel-grain-corrections.md) |
| `[/]` | 181 | Notebook chrome fidelity: neutral relationship-graph nodes (amber stroke, not tag-colored), Catalog block-pattern icons, Observation-log marker semantics. Code lands 2026-08-30, build/clippy/test clean; user screenshot confirmed nodes/Catalog icons render correctly and surfaced a real tiling bug in 180's `asterisk_mask` (fixed, see 180's own file); full systematic live-check (dashed-border case, observation log) still not run; Observation-log marker AC flagged as an open gap (no valence signal in `LogEntry` to key off, not invented) | `population-model-aesthetic` — [181](done/181-notebook-chrome-fidelity.md) |
| `[x]` | 182 | Interstitial screens + main menu chrome, plus shared exact-hex state-color constants (`ALERT_COLOR`/`DOT_FILLED_COLOR`/`trend_color` are 3 independent approximations today). Build/clippy/test clean; not live-verified (skipped per user instruction) | `VISUAL_STYLE_GUIDE.md` — [182](done/182-interstitial-screens-menu-chrome.md) |
| `[x]` | 183 | Pause menu + confirmation dialog: chrome, and Confirm/Cancel state-colored | `VISUAL_STYLE_GUIDE.md` — [183](done/183-pause-menu-confirmation-chrome.md) |
| `[x]` | 184 | Floating overlays (inspect card, hover tooltip, contextual hints): chrome + fixes a state-color inversion bug (saturation warning painted positive-green) + Unicode trend glyph | `VISUAL_STYLE_GUIDE.md` — [184](done/184-floating-overlays-chrome.md) |
| `[x]` | 185 | Era-reveal card: remove `species_color` identity swatch from genome-diff rows (found by code audit, not in original 180/181 scope) | `VISUAL_STYLE_GUIDE.md` — [185](done/185-era-reveal-species-color.md) |

Not filed as a task — a GDD/balance discussion, not a code task yet: whether
an isolated species with no matrix stimulus should trend flat/negative
instead of growing on base metabolism gain alone (`playtest_outcome.md`
gameplay note #16). Current behavior (`src/sim.rs:1241-1266,1924-1942`) is
deliberate — a comment there explicitly notes base gain is kept
neighbour-independent so isolated organisms don't starve from drift alone.
Revisit in GDD §5.6/§5.9 before scoping a task, if the balance question
still stands after Phase 2 lands.

## Phase 3 — content and variety

| Status | ID | Title | Doc |
|-------|----|--------|-----|
| `[x]` | 155 | Trait archetypes: 3-letter codes replacing the Greek glyphs, 5 families, 15-trait active pool | `tag-archetypes` — [155](done/155-trait-archetypes.md) |
| `[ ]` | 156 | Dominant family bias per world (on matrix intensity, not on trait selection) — depends on 155 (needs its `TraitFamily` grouping) | `tag-archetypes` — [156](156-dominant-family-bias.md) |
| `[ ]` | 157 | Narrative generation: event ranking, fragment grammar, clinical register — blocking pre-decision inside the file (cross-cutting §5: structured fragments vs. strings) | `narrative-generation` — [157](157-narrative-generation.md) |

⚠️ **Before 157**, decide whether text fragments are structured data rather than
concatenated strings (`cross-cutting` §5). Deciding later means rewriting the pool.

Deliberately excluded from 155, playtest-gated: the "active traits per world"
revision (4 in world 0, ceiling 9) — the document itself warns it touches the
only difficulty knob already tuned in playtest.

## Phase 4 — session structure

| Status | ID | Title | Doc |
|-------|----|--------|-----|
| `[ ]` | 158 | Leaving/entering a world, end of run, cumulative aggregates | `transitions-metaprogression` Part 1 |
| `[ ]` | 159 | World-summary export (seed sharing) | `culture-shock-distribution` |
| `[ ]` | 160 | Main menu, new-run setup, updated "how to play" summary | `menu-onboarding` |
| `[ ]` | 161 | Snapshot save, single slot consumed on load | `cross-cutting` §1 |

161 is deliberately not earlier: snapshotting a state that changes every week is
work redone continuously.

## Phase 5 — polish and rare systems

| Status | ID | Title | Doc |
|-------|----|--------|-----|
| `[ ]` | 162 | Three-layer audio + spatialisation | `audio`, `cross-cutting` §2 |
| `[ ]` | 163 | Alternative accessible palette | `cross-cutting` §3 |
| `[ ]` | 164 | Dynamic biomes (shared transition mechanism) | `world-events-catastrophes` |
| `[ ]` | 165 | Catastrophic and neutral world-event catalogue | `world-events-catastrophes`, `world-events` |
| `[ ]` | 166 | Biome signatures + cosmic-origin events | `biome-cosmic-events` |
| `[ ]` | 167 | Anomaly pockets, fossil traces, concrete extremophile | `wonder` |
| `[ ]` | 168 | Xenotraits as a rare speciation outcome | `tag-archetypes`, `evolution-xenotypes` |
| `[ ]` | 169 | Emersione: lineage, family coverage, probabilistic trigger, collapse to a single entity | `emersione` |

Emersione is last because it depends on nearly everything else — speciation,
trait families, victory-as-a-flag, reveal tiers — not because it matters least.

Explicitly post-MVP, not in this backlog: declared-and-refuted hypotheses
(`culture-shock-identity.md`), concrete meta-progression and Codex
(`transitions-metaprogression` Part 2), localisation (`cross-cutting`).

---


# 🧱 Pre-redesign work still open

Everything below predates the Culture Shock redesign rollout above. Only one
item is still open here (114, blocked) — the rest is history kept for the
rationale behind past phases.

The "Two-tier map view" phase (075-078) is fully closed and archived in
`tasks/QUEUE_ARCHIVE.md` — 078, the last open item (a same-day playtest
correction to 076's blob rendering, on hold since 2026-08-10), landed
2026-08-19.

**Biomi** (2026-08-11, scoped from `redesign/processed/abiogenesis-biomes.md` after a
design-discussion pass that reconciled the doc with the current codebase — full
decision record in the doc itself). Replaces the flat `TerrainKind` bands with 16
discrete biomes. Dependency order: 110 (data layer, areal biomes) unblocks 111
(explicit feature placement); 112 (rendering) depends on both 110 and 111. 114
(Geyser) is scoped for reference but blocked — no small/pulsing heat-source category
exists yet to back it. **113's dependency on 125 resolved 2026-08-13** (dependency
review, see the "credibility follow-ups" section below and 113/122/125's own files):
`place_toxic_zone` was, at the time, the only generation-time source of nonzero
`Cell.toxicity`; removing it (113) before 125 shipped a replacement (Swamp-cell
toxicity) would have silently broken task 108's chemolithotroph and made 122 moot for
the wrong reason. Resolved order: **125 → 113 → 122**. **113 landed 2026-08-19**;
only 114 remains blocked in this section.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 110 | Biome enum + two-stage classification (areal biomes) | — | [110](done/110-biome-classification-two-stage.md) |
| `[x]` | 111 | Explicit placement for feature biomes (Cratere, Distesa di cristalli, Lago, Bocca vulcanica) | 110 | [111](done/111-biome-feature-placement.md) |
| `[x]` | 112 | Biome rendering (flat color, dithering, borders, tree overlay) | 110, 111 | [112](done/112-biome-rendering.md) |
| `[x]` | 113 | Palude replaces `toxic_zone` | 110, 125 | [113](done/113-swamp-replaces-toxic-zone.md) |
| `[ ]` ⏸ | 114 | BLOCKED — Geyser biome (needs a small/pulsing heat-source category, not yet scoped) | 110, 111, unscoped source-model extension | [114](114-geyser-pulsing-source-blocked.md) |

The "UI bugfixes" phase (115, 121) closed 2026-08-25 once 121 landed and is
archived in `tasks/QUEUE_ARCHIVE.md`.

The "Worldgen pipeline reassessment — credibility follow-ups" phase
(128-131) closed 2026-08-19 once 131 landed and is archived in
`tasks/QUEUE_ARCHIVE.md`, alongside task 132 (`[DECISION]` on
`Cell.slope`/`Cell.water_distance` ordering, found mid-phase).

| `[x]` | 084 | Guaranteed "first light" relation in world 0's matrix — unblocked and implemented 2026-08-29 (was blocked on "Meta-progression persistence", `PROJECT_PLAN.md` §1; applies to every `world_index == 0` instead, per user decision following the first human playtest's engagement findings, `playtest_outcome.md`) | 011, 013 | [084](done/084-first-light-guaranteed-relation-world0.md) |

084 is from the "Onboarding & engagement rollout" phase (080-083, fully
closed and archived to `QUEUE_ARCHIVE.md`) — the last item of that phase to
land, since it wasn't available to pick up until 2026-08-29.

Final tuning phase still lives as backlog in [`PROJECT_PLAN.md`](../PROJECT_PLAN.md) beyond what's already expanded into task files here.

---

*Last updated: 2026-08-29 (Phase 3 scoped — task files written for 155/156/157,
no implementation done, mirroring how Phase 2 was scoped whole before being
built. 155: single choke point found (`notebook::tag_glyph`,
`src/notebook.rs:613-626`, fed by `TagConfig::global_tag_pool`), scope is
raising the 10-tag pool to 15 and replacing `TAG_LETTERS` with a
code/name/family table; flagged as an RNG-stream-affecting change (shifts
seeded outcomes downstream) and four GDD/player_guide locations needing a
docs update. 156: confirmed via grep that no `TraitFamily` concept exists
anywhere in the codebase yet, so it hard-depends on 155 landing first;
scoped the bias as applying only to matrix intensity magnitude
(`generate_matrix`, `src/world.rs:3041-3086`), never to sign or trait
selection, per the design doc. 157: the QUEUE's own blocking pre-decision
(structured fragment data vs. plain strings, `cross-cutting` §5) is
surfaced as a mandatory Step 0 in the task file rather than resolved by the
scoping pass; event scoring reuses the notebook's existing confounder-weight
formula instead of a second causal-attribution system. None of the three
files pick a design decision the source docs left open — open questions are
recorded as caveats for whoever picks the task up. Phase 3 remains
informally gated on 171's human playtest half per the note above; scoping
ahead of the gate follows the same precedent as Phase 2's own scoping
session.)*

*Last updated: 2026-08-29 (147, Splice restricted to confirmed traits +
growing genome bank + "synthesised" origin — implemented and archived to
`tasks/done/`. New `MatrixKnowledge::is_tag_confirmed` (a tag counts as
confirmed if it appears in at least one confirmed pair, as exerter or
receiver — a judgment call the task file flags explicitly, since
confirmation is stored per-pair, not per-tag); `splice_panel`'s SwapTag/AddTag
tag lists now filter through it, sourced from `world.active_tags` (a guard
structurally exclusive of any future xenotrait pool, task 168, as long as
that pool stays out of `active_tags`). New `SimWorld::spliced_species`
(mirrors `wild_species`) + `species_origin` resolving Seeded/Indigenous/
Synthesised, populated by `apply_splice` alongside `push_species`. Origin
surfaced in the Catalog (`text::species_origin_label`) and as a `⚗` marker
in the Seed Palette row. `hud_panel` hit Bevy's 16-parameter ceiling again
adding `MatrixKnowledge` — bundled into a new `SpliceReadouts` SystemParam
with `SpliceDraft`, same pattern as `ObjectiveReadouts` (task 109). Origin
era (doc's other open question) confirmed already shipped, no work needed.
`cargo test`/`clippy -D warnings`/`fmt` clean.)*

*Last updated: 2026-08-29 (146, Cull emits a tracked notebook observation —
implemented and archived to `tasks/done/`. Extracted the tick loop's
per-neighbour-pair adjacency scan into two shared helpers (`sim.rs`):
`distinct_neighbour_tags` (confounder basis) and `adjacency_pair_observations`
(tag-gate + matrix lookup + contribution + confounder count, no onset
gating). `sim::step` now calls these and filters the result by
`onset_mask` itself; the new `cull_knockout_observations` calls them once
per living neighbour with the culled cell as exerter, no filtering —
matching the design doc's "Y behaves differently after removing X" framing
(culled organism as exerter, surviving neighbour as receiver). Wired into
`input.rs::cull_on_click` via a `MessageWriter<AdjacencyObserved>`, called
before the cell is cleared. `TerrainGateObserved` events from the
underlying tag-gate checks are discarded for Cull — out of scope, terrain
evidence stays a tick-loop concern. `sim.rs:222`'s task-146 placeholder
comment resolved. `cargo test`/`clippy -D warnings`/`fmt` clean, including
the full `tests/balance.rs` suite (unaffected — the refactor changes
control flow, not the energy formula).)*

*Last updated: 2026-08-29 (145, Stress on three selectable axes + gradual
decay — implemented and archived to `tasks/done/`. `ActionMode::Stress`
keeps one slot; `SelectedStressAxis` (task 032, `ui.rs`) picks
thermal/light/toxicity, shown as a sub-row only while Stress is active.
`SimWorld::apply_stress` snapshots the pre-stress value per axis into a new
`stress_decay: Vec<StressDecay>` grid, `SimWorld::decay_environment_stress`
relaxes it back at `EnvironmentConfig::stress_decay_rate` every tick
(wired into `sim::step` right after `reinject_environment_sources`, whose
heat-source/toxic-Swamp cells it explicitly skips rather than fighting).
`EnvironmentConfig::stress_delta` kept as one shared value across axes, no
design reason found yet to split it per-axis. Era-spanning permanent
biome-transition accumulation stayed out of scope per the task file, still
blocked on task 164. GDD §6 flipped to `[DECIDED]`. `cargo test`/`clippy -D
warnings`/`fmt` clean.)*

*Last updated: 2026-08-29 (all 10 Phase 2 task files written this session —
145 first, then 146/147/149/150/152/153/154/170/171 scoped in parallel by
sibling agents, 151 last (needed 152/153's findings first). No
implementation done yet, scoping only, per direct user instruction to scope
the whole phase before building any of it. Notable findings from scoping,
not from playtest: **152's QUEUE title was stale** — the `sidebar-redesign`
doc it named already shipped as tasks 064/065 (verified against the live
`ui.rs`/`text.rs`); 152 was re-scoped from `hud-notebook.md`'s residual gap
instead (continuous-advance toggle, mutation-tier badge, species subtext),
row title corrected above. **153's "node graph" is also already shipped**
(`notebook.rs::hypothesis_grid`, tasks 021-102) — 153's real scope is the
Chronicle section and a new `Species::parent` field for "descends from".
**149 surfaced a real gap**: `ActionMode` has no "no action armed" state,
so click-to-inspect can't fire without at least a minimal deselect seam
(149 adds one, defers the full `Esc`-cascade UX to 150). **150 and 152
both reached for the `p`/continuous-advance keybind** — resolved by
cross-linking the two files: 150 explicitly excludes it (no underlying
auto-play loop exists), 152 owns it as one of its three real gaps. **171
recommends running last**: its bot-vs-bot half depends on 146/147/170
(what a bot can legitimately read). **154 is the largest single scope**
(~3-4h, four sub-changes including the `objectives.rs:511-513` fix that
makes Emersione/task-169 structurally unreachable today) — flagged as a
candidate for a future 154a/b/c split if picked up piecemeal, kept as one
file for now since QUEUE.md only reserved one ID. A minor unrelated
discrepancy surfaced by 153's scoping — `notebook_window`'s doc comment
says sim time is deliberately unaffected while the notebook is open, but a
companion design doc says it should pause — is noted in 153's file but not
assigned to any task; revisit if it turns out to matter. Suggested
execution order recorded above the Phase 2 table.)*

*Last updated: 2026-08-28 (Phase 1b, 141-144, all four landed this session —
Phase 1 playtest checkpoint deliberately skipped per direct user instruction,
to be recovered after Phase 1b. 141: `Population::blocked` (already computed
by task 137) rendered as a shape+color marker in Detail view only —
`render::BlockedIndicatorSeen` tracks first-occurrence-per-cell for a pulsing
accent vs. a static one after. 142: `sim::DominantStimulus` made `pub`,
`EraEvolutionReveal` gains a `dominant_stimulus` field, `text::era_reveal_evolution_line`
appends a one-clause-per-stimulus cause sentence — no new calculation, only
new exposure of what `speciate` already computes. 143: new
`SimWorld::stall_ticks` (grid-sized, `sim::step`-maintained) plus
`sim::any_population_stalled`; `ui::StallHint` mirrors `IsolationHint`'s
one-shot `MetaProgress`-gated, era-derived-duration shape, shown below both
task-053 hints and the isolation hint. 144: `notebook::translated_tag_label`
appends the current roster's species names to a bare tag glyph for the
log's first 5 entries — applied to both `accumulate_evidence` (matrix
confirmations) and `accumulate_terrain_evidence` (terrain-gate
confirmations). Two scope corrections against the design doc, both noted in
QUEUE_ARCHIVE.md's Phase 1b entry: 144 applies its principle to this
codebase's actual single-Greek-letter tag glyphs, not the doc's
illustrative three-letter codes (task 155's rework, unshipped); 142's cause
clause stays at the three-stimulus category level, since
`SelectionThresholdCrossed` has no per-neighbour attribution to name more
specifically. Recommended before Phase 2: replay
`culture-shock-naive-player-example.md`'s scenario with all four applied —
not yet done. `cargo test`/`clippy -- -D warnings` clean throughout.)*

*Last updated: 2026-08-27 (134, two-bot experiment-incentive harness —
`examples/two_bot_survey.rs` runs an exploiter and an explorer against the same
40 world-0 seeds and reports eras-to-clear plus the known/unknown/isolated split
of every point spent. Two library extractions were forced by the crate structure
(`tests/`/`examples/` can't see the binary's modules): `MatrixKnowledge` moved
from `notebook.rs` into a new `src/knowledge.rs` alongside a pure
`accumulate_adjacency_evidence` that now owns the GDD §7 weight formula, and
`attempt_seed` moved from `input.rs` into a new `src/actions.rs`, returning
`Option<usize>` so `PlayerPlacedCells` bookkeeping stays in the binary.
`sim::env_fit` and `objectives::update_grace_progress` made `pub`. Stress/Cull
stay in `input.rs` until tasks 145/146 rework them; the bots use only `Seed`,
since Cull emits no observation yet and Splice's confirmed-traits constraint is
task 147. Baseline recorded verbatim in the task file — it is what 136 must be
compared against. Nothing tuned.)*

*Last updated: 2026-08-27 (Culture Shock redesign rollout planned — the whole
`redesign/processed/` corpus turned into the 134-169 backlog above. Phase 0's two INDEX
verifications executed against the build and closed: no `interaction_delta`
scale coefficient exists (raw `{−2..+2}` into the energy sum), and action scope
is per-cell for 3 of 4 with Splice already creating a new species as
`abiogenesis-actions.md` specifies. A third finding, not asked for, reframes the
Phase 1 balance work: the matrix is ignorable **by construction** — zero diagonal
plus a forced `net_self_interaction == 0` make `interaction_delta` exactly zero
inside a single-species blob, so the matrix only ever acts at interfaces between
different species. Task files written for Phase 1 only (134-140); later phases
are backlog rows. Three corrections to `redesign/abiogenesis-INDEX.md` applied
and annotated there: clock before balance, population model split across two
phases, and the 076/078 Overview machinery marked for removal rather than
adaptation.)*

*Last updated: 2026-08-26 (120, Biosphere numeric population delta —
`PopulationTrends` gains a `previous_population`/`current_population_delta`
pair alongside its existing energy-trend fields, same resize-on-
`EraCompleted` lifecycle; delta math extracted as a pure `population_delta`
helper (mirrors `classify_trend`'s shape) so it's unit-testable without a
full ECS harness. First-era-with-population baseline: no delta shown
(empty string), not `+N` from an implicit zero. `text::population_delta_label`
formats `+N`/`-N`/`±0`, wired into the Biosphere row right after the
existing (unchanged, energy-based) trend glyph. Live `cargo run`
verification explicitly skipped this session by direct user instruction —
completed and archived to `tasks/done/`. 119 (Moves icons monochrome
restyle) was cancelled the same day, deferred to a future redesign pass —
the "HUD & Notebook redesign follow-up" phase is now fully closed and
archived to `tasks/QUEUE_ARCHIVE.md`.)*

*Last updated: 2026-08-19 (078, Overview heatmap blob shape correction —
on hold since 2026-08-10, unheld and picked up directly by user request.
`cluster::compute_cluster_render` replaces `compute_cluster_density`:
each cluster's blob now fills interior holes (flood-fill the bounding
box's border-connected exterior; any non-member cell never reached is an
enclosed hole) and then erodes smaller (`ClusterConfig::
blob_erosion_iterations`, skipped below `blob_erosion_min_size` filled
cells, aborted if a pass would erode a blob to nothing) — so a blob reads
as a smaller, solid, abstracted shape instead of a 1:1 trace of the real
occupied-cell footprint with its gaps. Density formula unchanged (task
076's own population-mass reading). `render.rs`'s `cell_color` Overview
branch now keys off blob membership (`ClusterRender::species`) instead of
literal `Cell::organism`, so a filled-hole cell can render without a real
organism and an eroded-away edge cell can fall back to plain terrain —
the intended abstraction, not a regression. Verified via a headless
ASCII-diagram diagnostic against hand-built realistic cluster shapes (a
79-cell circular blob eroded to a clean 45-cell filled circle; a 91-cell
elongated oval eroded to 43 cells while staying clearly elongated) —
live `cargo run` screenshot check explicitly skipped this session by
direct user instruction — completed and archived to `tasks/done/`,
closing the "Two-tier map view" phase (075-078), moved to
`QUEUE_ARCHIVE.md`. 131, Soil moisture — `Cell.soil_moisture`
(`SimWorld::compute_soil_moisture`, run right after `compute_hydrology`
and before `classify_biomes`) replaces task 125's `slope`/`water_distance`
drainage proxy for Palude's fitness score: `rainfall` retained against
`slope` runoff, real proximity bonuses toward `is_river` cells (task 127)
and toward `record_significant_depressions`' future-Lake footprints (task
129, known before `Biome::Lake` is painted — a new `bfs_distance_from_indices`
sibling of the existing predicate-based BFS makes this possible), minus
evaporation (`temperature`) and drainage (`slope` again, a separate
term). `swamp_score` now takes `soil_moisture` directly; task 128's
`compute_macro_regions` region-aggregate call updated to match (flagged as
a forward coupling in 131's own file before 130 landed). Threshold
calibration went through three values before settling: `0.35` (naive
median guess) let Swamp dominate at 41% of Plain cells; `0.65` fixed the
aggregate fraction (8.2%) but broke the existing
`some_swamp_cells_are_toxic_across_seeds` balance test (36/60 seeds,
below its 75% floor) — Swamp got too rare per-seed even though the
aggregate looked right; `0.5` lands both (19% coverage, 47/60 seeds
passing). New relational test asserts soil_moisture's correlation sign
against rainfall/temperature/slope across 20 seeds. No dedicated
biome-distribution histogram test existed to re-run for task 125 (that
acceptance criterion was satisfied via the calibration measurements
instead) — completed and archived to `tasks/done/`, closing the whole
"128 onward" chain; the phase itself (128-131) moved to
`tasks/QUEUE_ARCHIVE.md`. 130, Mountain sub-banding — non-`Peak`
`TerrainKind::Mountain` cells now sub-band into `Glacier`/`AlpineMeadow`/
`BareRock`/reused-`Forest` (no new `MountainForest` variant — `Forest`'s
existing temperature/light score has no `TerrainKind` dependency, so it
already applies) via the same score-based `argmax_biome` idiom task 125
introduced; `Peak` stays a hard special case outside the competition.
`BareRock`'s score is now shared between `Hill` and `Mountain` (a new
`bare_rock_score`, replacing the old Hill-only hard light cutoff) per the
task's own "one implementation, not two" criterion. Thresholds calibrated
from a temporary scratch sample of ~20k Mountain cells across 20 seeds
(temperature has no elevation coupling in this codebase, so its spread
across Mountain terrain comes entirely from heat-source/sea-coolant
proximity). New relational test
`mountain_sub_bands_correlate_with_temperature_and_slope_as_designed`
(30-seed aggregate means). Visual verification was partial: the
background agent's sandbox had no working display for a live `cargo run`
screenshot, so it rendered the production color functions offscreen
instead (pixel-exact, not a real window check) — a genuine live check is
still outstanding. That agent's own `git checkout -- src/render.rs`
cleanup accidentally reverted two of the three `render.rs` edits, caught
via `git diff` and reapplied before closing — completed and archived to
`tasks/done/`. 129, Lakes derived from terrain depressions —
`record_significant_depressions` (Moore-adjacency connected components of
`fill_depressions()`'s basin cells) records qualifying depressions
(`lake_depression_min_size`/`_max_size`/`_min_depth`, calibrated via a
temporary scratch histogram across 25 seeds); `place_feature_biomes`
promotes them directly to `Biome::Lake` using the depression's real
footprint, falling back to task 123's organic-mask search only if fewer
than `lake_min_depression_count` were promoted. Pipeline reordered:
`fill_depressions`/`compute_hydrology` now run before `classify_biomes`/
`place_feature_biomes` (previously after), since Lake placement needs
depression data up front — confirmed via doc-comment audit that neither
step ever depended on biome data. Empirically checked river/Lake-interior
overlap post-reorder (only 0.3% of Lake cells carry `is_river`, not a
problem). GDD §5.10 updated to note Lake is now derived, not placed like
Crater/CrystalField. Notes added to tasks 130/131 flagging forward
couplings (`Cell.is_river`/`flow_accumulation` now available to
`classify_biomes`; `compute_macro_regions`'s second `swamp_score` call
site would need updating if 131 swaps in `soil_moisture`) — completed and
archived to `tasks/done/`. 128, macro-regions before per-cell biome
classification — a coarse 6-point Voronoi partition
(`SimWorld::compute_macro_regions`, its own `MACRO_REGION_SEED_OFFSET`
stream) with one dominant biome per region, applied as a multiplicative
bias on each Plain cell's own score before arg-max; deliberately
multiplicative rather than additive since task 125's own score functions
can plateau at `1.0`, where an additive bias would be a no-op. Confirmed
via live `cargo run` screenshots across 3 seeds: biomes now read as a
handful of large contiguous regions with feature-biome patches as local
texture inside them, not the fine speckled mosaic before this task —
completed and archived to `tasks/done/`. Also added
`tests/config_ron_sync.rs` (deserializes `sim_config.ron` against
`SimConfig::default()`), which immediately caught two more pre-existing
drifts from task 106 (`conditional_tag_count`, `confirmation_threshold`),
now fixed. 122, Swamp toxicity reinjection —
`reinject_environment_sources` extended with a new `toxic_swamp_cells`
list and `SourceConfig::toxic_reinjection_strength`, mirroring the
heat-source pattern task 085 established, so `diffuse_environment` no
longer erodes Swamp's toxic sub-region toward ambient over a long run;
`tests/balance.rs`'s chemolithotroph test restored to the file's normal
500-tick horizon — completed and archived to `tasks/done/`, phase archived
to `QUEUE_ARCHIVE.md`. 133, the two Task 113 follow-ups — SurviveIn's
Swamp target now gets a dashed purple highlight while that objective is
active, reviving the visual language `draw_toxic_zone` used to provide
(`render.rs::draw_survive_in_target`, gated on `CurrentObjective`); the
"larger toxic zones" difficulty axis is revived as a scaled
`swamp_toxicity_min` (new `DifficultyConfig::swamp_toxicity_min_late`,
`WorldParams::swamp_toxicity_min`, threaded into `classify_biomes`) rather
than a resized rectangle — completed and archived to `tasks/done/`, phase
archived to `QUEUE_ARCHIVE.md`. 113, Palude replaces `toxic_zone` — removed the
standalone `ToxicZoneBounds`/`place_toxic_zone` rectangle and
`draw_toxic_zone`'s dashed outline; `SurviveIn`'s zone check now reads
`Cell::biome == Swamp` directly, and Swamp's own post-classification
toxicity modifier (task 125) is the sole remaining generation-time
toxicity source — completed and archived to `tasks/done/`. Found and fixed
in the same pass: `place_feature_biomes`'s VolcanicVent override didn't
reset `toxicity`, silently inheriting stale Swamp values on overlap; the
`chemolithotroph_survives_reasonably_in_its_toxic_zone_across_seeds`
balance test's toxic-cell search picked the largest contiguous toxic blob
regardless of toxicity value, which is usually `Lake` (0.05, a flavor
value) rather than a genuinely hostile cell — fixed to sort on toxicity
value first. Only 114 remains open in the Biomi phase now. GDD/player_guide
synced; two open follow-ups filed as task 133 (SurviveIn's Swamp region
has no visual/textual affordance now that the dashed-outline rectangle is
gone, and the "larger toxic zones" difficulty axis lost its
implementation). 127, flow accumulation and rivers, and its follow-on
decision task 132 (`Cell.slope`/`Cell.water_distance` ordering) — completed
and archived to `tasks/done/`, closing the 123-127 worldgen pipeline
reassessment phase (archived to `QUEUE_ARCHIVE.md`) in a prior pass this
session that this note previously failed to log here.
112, biome rendering — flat dithered colors via
`biome_color`/`dithered_biome_color`, biome-based boundaries/coastline, a
new deterministic tree overlay (`♣` glyph, no RNG) — completed and
archived to `tasks/done/`. `draw_toxic_zone` deliberately left untouched,
task 113's job. Only 113 (and blocked 114) remain open in the Biomi
phase. 111, explicit placement for feature biomes —
Crater/CrystalField/Lake via bounded-retry rectangles with a hard
zero-overlap requirement against each other, Bocca vulcanica hooked
directly into `SimWorld::heat_sources` via a small independent vent
radius — completed and archived to `tasks/done/`. Unblocks 112 (still
open). 110, biome enum + two-stage areal classification,
completed and archived to `tasks/done/` — adds `Biome`/`Cell::biome`/
`Cell::elevation`/`BiomeConfig`, and a `classify_biomes` generation step
run after `place_toxic_zone` so Stage B reads real generation-time
`toxicity`. Palude currently only appears as an organic sub-region of the
toxic zone's footprint, since that's still the only nonzero-`toxicity`
source at generation time — task 113 decouples the two. Unblocks 111 and
113. 116-120 added: HUD & Notebook redesign
follow-up, scoped from `redesign/processed/abiogenesis-hud-notebook.md` after a
discrepancy-check pass against tasks 097/100-103 — see `PROJECT_PLAN.md`
for the full discrepancy list and resolutions. Task 103 extended in place
(population + origin era) rather than split into a new task. 096, 098, 099
completed and archived to
`tasks/done/` — 098's manual playtest also surfaced and fixed a temperature-
spread bug in `generate_starting_palette`/`add_bonus_species`/
`place_wild_species` that predated this session, plus the matching
`tests/balance.rs` harness correction. 115 added: HUD-panel click-through
bug at high zoom, reported during that same playtest. 110-114 added: biome system scoped from
`redesign/processed/abiogenesis-biomes.md` after a design-discussion pass reconciling the
doc with the current codebase — `TerrainKind`/`is_peak` already cover the
elevation-based biomes, task 085's heat sources already back Bocca vulcanica,
task 108 will make biome toxicity values load-bearing. 114 is scoped for
reference but blocked, same pattern as 084. 096-109 added: full scoping pass
over the five redesign docs from this session's "mondo vivo" design discussion —
conditional tags, notebook UX, death legibility, evolution & xenotypes,
progression & pacing. 109, unblocked 2026-08-12 once 096-099/106-107 all
shipped, implemented the same day (`Objective::Speciation` long-term
objective + within-run energy economy) — full phase now closed and
archived to `QUEUE_ARCHIVE.md`. 088-089, self-interaction balance bug fix, completed and
archived to `QUEUE_ARCHIVE.md`. 082 and 083, tuned jointly, completed and
archived to `tasks/done/`. 084 stayed intentionally out of the queue as
blocked until 2026-08-29, when it was unblocked and implemented the same
day (see its row above) — the persistence dependency was traded for
applying the guarantee to every `world_index == 0` instead of only a
player's true first world.
078 done (see recent commits). 081, toxic-zone pulse + diffusion
drift check, completed and archived to `tasks/done/`. 085-086, "Environment as sources," fully
closed and archived to `QUEUE_ARCHIVE.md`. 090, terrain island-band
retune, completed and archived. 091-095, the bugfixing/UX batch, fully
closed and archived to `QUEUE_ARCHIVE.md`. 102 and 106/107 extended in
place (no new task IDs, no status change) after a review of an external
design draft, `abiogenesis-concurrent-idea.md`: 102 gains a partial-evidence
confidence-percentage tooltip; 106/107 gain a concrete first-pass
dominant-stimulus → edit mapping, resolving 107's previously-open
stimulus-to-outcome question.)*
