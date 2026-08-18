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

**Two-tier map view** (2026-08-09, design discussion held right after task
074's visual check surfaced an organism-legibility gap at 128×80 — full
decision record in `redesign/abiogenesis-two-tier-view.md`): a
continuous-zoom camera with a hard-threshold switch between the current
per-cell rendering (Detail) and an aggregated per-species cluster heatmap
(Overview), plus gating Stress/Cull to Detail while Seed/Splice stay
available in both. 075 and 076 are done (archive); 078 is a same-day
playtest correction to 076 (blobs currently trace the real occupied-cell
footprint 1:1, including gaps — should render smaller and uniformly filled).
**078 is on hold as of 2026-08-10 (⏸ do not pick up until unheld)** — no
blocking dependency, just a deliberate pause; still `[ ]` since it's not
cancelled and not blocked by another task.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[ ]` ⏸ | 078 | ON HOLD — Overview heatmap blob shape correction (playtest correction to 076: blobs must render smaller/abstracted and uniformly filled, not a 1:1 trace of the real occupied-cell footprint with its gaps) | 076 | [078](078-overview-heatmap-blob-shape-correction.md) |

**Biomi** (2026-08-11, scoped from `redesign/abiogenesis-biomes.md` after a
design-discussion pass that reconciled the doc with the current codebase — full
decision record in the doc itself). Replaces the flat `TerrainKind` bands with 16
discrete biomes. Dependency order: 110 (data layer, areal biomes) unblocks 111
(explicit feature placement); 112 (rendering) depends on both 110 and 111. 114
(Geyser) is scoped for reference but blocked — no small/pulsing heat-source category
exists yet to back it. **113's dependency on 125 resolved 2026-08-13** (dependency
review, see the "credibility follow-ups" section below and 113/122/125's own files):
`place_toxic_zone` is currently the only generation-time source of nonzero
`Cell.toxicity`; removing it (113) before 125 ships a replacement (Swamp-cell
toxicity) would silently break task 108's chemolithotroph and make 122 moot for the
wrong reason. Resolved order: **125 → 113 → 122**.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 110 | Biome enum + two-stage classification (areal biomes) | — | [110](done/110-biome-classification-two-stage.md) |
| `[x]` | 111 | Explicit placement for feature biomes (Cratere, Distesa di cristalli, Lago, Bocca vulcanica) | 110 | [111](done/111-biome-feature-placement.md) |
| `[x]` | 112 | Biome rendering (flat color, dithering, borders, tree overlay) | 110, 111 | [112](done/112-biome-rendering.md) |
| `[ ]` | 113 | Palude replaces `toxic_zone` | 110, **125 (hard blocker)** | [113](113-swamp-replaces-toxic-zone.md) |
| `[ ]` ⏸ | 114 | BLOCKED — Geyser biome (needs a small/pulsing heat-source category, not yet scoped) | 110, 111, unscoped source-model extension | [114](114-geyser-pulsing-source-blocked.md) |

**HUD & Notebook redesign follow-up** (2026-08-12, scoped from
`redesign/abiogenesis-hud-notebook.md` after a discrepancy-check pass against
the already-scoped notebook tasks 100-103/097 — see `PROJECT_PLAN.md` for
the full list of discrepancies found and how each was resolved). 103 was
extended in place (population + origin era added to its existing scope,
not split out) rather than becoming a new task. No dependencies between
116-120; 117/118 touch the same HUD readout line for different reasons
(math vs. wording) and may need a small rebase if landed out of order, but
neither blocks the other.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 116 | Notebook: left-docked panel with dimmed map behind it, not a floating window | — | [116](done/116-notebook-docked-panel-dimmed-map.md) |
| `[x]` | 117 | Time readout: show progress within the current era, not the run-wide tick counter | — | [117](done/117-time-readout-era-relative-pulse-progress.md) |
| `[x]` | 118 | Rename player-facing "tick" to "pulse" | — | [118](done/118-rename-tick-to-pulse.md) |
| `[ ]` | 119 | Moves icons: monochrome glyphs that actually render (fixes a pre-existing tofu-box bug) | — | [119](119-moves-icon-restyle-monochrome.md) |
| `[ ]` | 120 | Biosphere: numeric population delta alongside the trend arrow | — | [120](120-biosphere-population-delta.md) |

Deliberately **not** scoped from the same doc, per this session's decision:
auto-advance (play/pause continuous ticking) — deferred until a separate
real-time pacing mechanic lands; the doc's mutation-tier badge — descoped
to icon-restyle-only (119) since no tiered-unlock mechanic exists to back a
badge yet.

**UI bugfixes** (2026-08-11, reported live during the 098/099 manual playtest
pass).

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 115 | Grid input (clicks and scroll-zoom) leaks through the HUD panel | — | [115](done/115-egui-panel-click-through-when-zoomed.md) |
| `[ ]` | 121 | Conditional-tag catalog badge never renders in a live playtest | 096, 097 | [121](121-terrain-badge-missing-in-catalog.md) |

**Balance/persistence** (found while balance-testing task 108). **Rescoped
2026-08-13**: originally targeted `ToxicZoneBounds`; now targets
`Biome::Swamp` membership instead, since task 113 removes `ToxicZoneBounds`
and task 125 moves the toxicity source onto Swamp cells. See 122's own file
for the full rescope note.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[ ]` | 122 | Swamp toxicity reinjection (toxicity erodes with no source to counter it) | 085, 108, **113, 125 (hard blockers)** | [122](122-toxic-zone-reinjection.md) |

**Worldgen pipeline reassessment** (2026-08-13, scoped from
`redesign/procedural_biome_generation_spec_v2.md` after a design-discussion
pass comparing its diagnosis against the current 128×80 pipeline; GDD §5.1
corrected in the same session — the doc's `48×32` grid was stale, task 074
already raised it to `128×80`). Five phases, ordered lowest- to
highest-risk: 123 (organic feature masks) is size-independent and purely
local; 124 (geomorphology fields) is additive-only; 125 (score-based
classification + drainage-based Palude) is the first to change existing
biome output — **and, per the 2026-08-13 dependency review, must land
before 113** (see the "Biomi" section above): 125 is what gives Swamp a
`toxicity` source that doesn't depend on `place_toxic_zone`, which 113
removes; 126/127 (rainfall, then flow accumulation/rivers) are the
highest-value, highest-complexity phases —
127 in particular carries a real determinism risk (elevation-sort
tie-breaking) flagged in its own task file. 126/127 add fields only;
wiring rainfall/rivers into biome classification or rendering is
explicitly left as future follow-up in both files' Non-Goals, not
pre-scoped here.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 123 | Organic masks for placed feature biomes (Cratere, Distesa di cristalli, Lago) — `toxic_zone` explicitly out of scope | 111 | [123](done/123-organic-feature-biome-masks.md) |
| `[ ]` | 124 | Derived geomorphology fields (`slope`, `water_distance`) — additive only | 110, 111 | [124](124-geomorphology-fields.md) |
| `[ ]` | 125 | Score-based biome classification; Palude from drainage instead of toxicity | 110, 111, 124 | [125](125-biome-score-classification.md) |
| `[ ]` | 126 | Rainfall field (orographic lift, rain shadow) — additive only | 124 | [126](126-rainfall-field.md) |
| `[ ]` | 127 | Flow accumulation and rivers | 124, 126 | [127](127-hydrology-rivers.md) |

**Worldgen pipeline reassessment — credibility follow-ups** (2026-08-13,
same session: after scoping 123-127, an explicit pass over what the spec
still covers that those five don't. Ranked by impact on a *single* world's
credibility, not variety across worlds — the lower-priority items from that
ranking (world profiles + biome budget/validation, biome transition/blend,
erosion, debug metrics tooling) are intentionally not scoped as tasks;
noted in `VISION.md` instead. 128 is the single highest-value item found in
that pass — per-cell scoring (125) alone still doesn't produce large
coherent regions without a macro-region layer above it.).

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[ ]` | 128 | Macro-regions before per-cell biome classification | 125 | [128](128-macro-region-biomes.md) |
| `[ ]` | 129 | Lakes derived from terrain depressions (123's search becomes fallback) | 123, 127 | [129](129-lakes-from-depressions.md) |
| `[ ]` | 130 | Mountain sub-banding (Glacier, AlpineMeadow, MountainForest) | 124, 125 | [130](130-mountain-sub-banding.md) |
| `[ ]` | 131 | Soil moisture (refines Swamp/Forest beyond the slope/water-distance proxy) | 124, 125, 126 | [131](131-soil-moisture.md) |

**Onboarding & engagement rollout** (2026-08-09, from `redesign/abiogenesis-engagement-design.md`, full rationale in `PROJECT_PLAN.md`'s "Onboarding & engagement rollout"): 5 onboarding-foundation proposals scoped after a multi-round discussion. 080 first (diagnostic value for playtesting the rest); 082/083 are numerically coupled — tuned together (2026-08-10), both now done. Live playtest of the combined pacing still pending (082/083 verification steps skipped this session, see below).

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 080 | Interaction spark: instant visual feedback on first-seen relations | 018, 075, 076 | [080](done/080-interaction-spark-visual-feedback.md) |
| `[ ]` ⏸ | 081 | ON HOLD (2026-08-10) — The world breathes: toxic zone pulse + diffusion drift check (rescoped down after discussion) | 033, 072 | [081](081-ambient-diffusion-visible-on-empty-grid.md) |
| `[x]` | 082 | Shorter eras during world 0's opening | 079 | [082](done/082-shorter-onboarding-eras.md) |
| `[x]` | 083 | Newborn incubation: reproduction delayed to the following era | 009 | [083](done/083-newborn-incubation-reproduction-delay.md) |

084 (guaranteed "first light" relation in world 0's matrix) is scoped
(`tasks/084-first-light-guaranteed-relation-world0.md`) but deliberately
**excluded from this queue** — blocked on the "Meta-progression persistence"
proposal (`PROJECT_PLAN.md` §1), not available to pick up yet.

Final tuning phase still lives as backlog in [`PROJECT_PLAN.md`](../PROJECT_PLAN.md) beyond what's already expanded into task files here.

---

*Last updated: 2026-08-12 (112, biome rendering — flat dithered colors via
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
follow-up, scoped from `redesign/abiogenesis-hud-notebook.md` after a
discrepancy-check pass against tasks 097/100-103 — see `PROJECT_PLAN.md`
for the full discrepancy list and resolutions. Task 103 extended in place
(population + origin era) rather than split into a new task. 096, 098, 099
completed and archived to
`tasks/done/` — 098's manual playtest also surfaced and fixed a temperature-
spread bug in `generate_starting_palette`/`add_bonus_species`/
`place_wild_species` that predated this session, plus the matching
`tests/balance.rs` harness correction. 115 added: HUD-panel click-through
bug at high zoom, reported during that same playtest. 110-114 added: biome system scoped from
`redesign/abiogenesis-biomes.md` after a design-discussion pass reconciling the
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
archived to `tasks/done/`. 084 stays intentionally out of the queue as
blocked. 078 and 081 on hold. 085-086, "Environment as sources," fully
closed and archived to `QUEUE_ARCHIVE.md`. 090, terrain island-band
retune, completed and archived. 091-095, the bugfixing/UX batch, fully
closed and archived to `QUEUE_ARCHIVE.md`. 102 and 106/107 extended in
place (no new task IDs, no status change) after a review of an external
design draft, `abiogenesis-concurrent-idea.md`: 102 gains a partial-evidence
confidence-percentage tooltip; 106/107 gain a concrete first-pass
dominant-stimulus → edit mapping, resolving 107's previously-open
stimulus-to-outcome question.)*
