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
| `[x]` | 128 | Macro-regions before per-cell biome classification | 125 | [128](done/128-macro-region-biomes.md) |
| `[x]` | 129 | Lakes derived from terrain depressions (123's search becomes fallback) | 123, 127 | [129](done/129-lakes-from-depressions.md) |
| `[x]` | 130 | Mountain sub-banding (Glacier, AlpineMeadow, MountainForest) | 124, 125 | [130](done/130-mountain-sub-banding.md) |
| `[ ]` | 131 | Soil moisture (refines Swamp/Forest beyond the slope/water-distance proxy) | 124, 125, 126 | [131](131-soil-moisture.md) |

Task 132 (`[DECISION]` on `Cell.slope`/`Cell.water_distance` ordering,
found in advisor review after 123-126 shipped) and the phase it belongs to
are archived in `tasks/QUEUE_ARCHIVE.md`, both closed 2026-08-19.

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

*Last updated: 2026-08-19 (130, Mountain sub-banding — non-`Peak`
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
