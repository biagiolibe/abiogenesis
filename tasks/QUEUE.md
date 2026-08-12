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

**Mondo vivo, notebook, death legibility, evolution & progression**
(2026-08-11, full scoping pass over five redesign docs from this
session's design discussion — see `PROJECT_PLAN.md` SECTION 1 for the
reasoning and SECTION 2 for the grouped backlog entry). Dependency order:
096 unblocks 099 and 097 (097 also depends on 103); 098, 100-106, 108 are
independently startable; 107 depends on 106; 109 is scoped for reference
but blocked.

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
| `[ ]` | 108 | Fourth metabolism: chemolithotroph, gain from toxicity | — | [108](108-chemolithotroph-metabolism.md) |
| `[ ]` ⏸ | 109 | BLOCKED — Long-term objective tier + within-run energy economy | 096-099/106-107 (shipped, not just scoped) | [109](109-progression-long-term-objective-energy.md) |

**Biomi** (2026-08-11, scoped from `redesign/abiogenesis-biomes.md` after a
design-discussion pass that reconciled the doc with the current codebase — full
decision record in the doc itself). Replaces the flat `TerrainKind` bands with 16
discrete biomes. Dependency order: 110 (data layer, areal biomes) unblocks 111
(explicit feature placement) and 113 (Palude replaces `toxic_zone`); 112 (rendering)
depends on both 110 and 111. 114 (Geyser) is scoped for reference but blocked — no
small/pulsing heat-source category exists yet to back it.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[ ]` | 110 | Biome enum + two-stage classification (areal biomes) | — | [110](110-biome-classification-two-stage.md) |
| `[ ]` | 111 | Explicit placement for feature biomes (Cratere, Distesa di cristalli, Lago, Bocca vulcanica) | 110 | [111](111-biome-feature-placement.md) |
| `[ ]` | 112 | Biome rendering (flat color, dithering, borders, tree overlay) | 110, 111 | [112](112-biome-rendering.md) |
| `[ ]` | 113 | Palude replaces `toxic_zone` | 110 | [113](113-swamp-replaces-toxic-zone.md) |
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
| `[ ]` | 116 | Notebook: left-docked panel with dimmed map behind it, not a floating window | — | [116](116-notebook-docked-panel-dimmed-map.md) |
| `[ ]` | 117 | Time readout: show progress within the current era, not the run-wide tick counter | — | [117](117-time-readout-era-relative-pulse-progress.md) |
| `[ ]` | 118 | Rename player-facing "tick" to "pulse" | — | [118](118-rename-tick-to-pulse.md) |
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
| `[ ]` | 115 | Grid input (clicks and scroll-zoom) leaks through the HUD panel | — | [115](115-egui-panel-click-through-when-zoomed.md) |
| `[ ]` | 121 | Conditional-tag catalog badge never renders in a live playtest | 096, 097 | [121](121-terrain-badge-missing-in-catalog.md) |

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

*Last updated: 2026-08-12 (116-120 added: HUD & Notebook redesign
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
reference but blocked, same pattern as 084/109. 096-109 added: full scoping pass
over the five redesign docs from this session's "mondo vivo" design discussion —
conditional tags, notebook UX, death legibility, evolution & xenotypes,
progression & pacing. 109 is scoped for reference but blocked, same
pattern as 084. 088-089, self-interaction balance bug fix, completed and
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
