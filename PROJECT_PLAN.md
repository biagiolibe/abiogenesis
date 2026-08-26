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
- `[?]` **Additional metabolisms** beyond the three base ones, e.g. a chemolithotroph tied to `toxicity`, as unlockable content (GDD §5.4). **Linked to "Evolution & xenotypes" below** (2026-08-11): the chemolithotroph is the first candidate for both an authored starting metabolism and an evolution outcome (a lineage evolving into one under repeated toxicity exposure) — the two tracks can share design work.
- `[?]` **Final title** — "Abiogenesis" is a placeholder (GDD §14).

### Camera, pacing & menu features

**Camera pan:** scoped into task 087 (2026-08-10), see SECTION 2's "Camera pan" entry.

- `[?]` **Cap camera zoom-out independent of grid size** — today `zoom_max` (`CameraConfig`, `src/config.rs`) auto-scales via `ScalingMode::AutoMin` (`spawn_camera`, `src/render.rs:899-917`) to always show 100% of `config.grid` at max zoom-out, however large the grid becomes — a deliberate property task 075/076 built Overview mode around ("don't lose the whole-world read," `abiogenesis-two-tier-view.md`). If the grid grows substantially beyond `128×80` in the future, revisit whether that "always see everything" guarantee should still hold, or whether max zoom-out should instead be capped (e.g. by a minimum legible cell-pixel-size floor) so a much larger world requires panning even in Overview. Task 087's pan system already works correctly under either model (its clamp math reads `config.grid` live, not a cached whole-grid assumption), so this doesn't block or get blocked by 087 — purely about `spawn_camera`/`CameraConfig`'s zoom-out semantics. Revisits 075/076's stated design rationale, not just a mechanical camera tweak — needs its own discussion when picked up.
- `[?]` **Real-time mode** as an option — GDD §4 noted it "costs little to add later"; with Bevy it's nearly free (just don't stop at the end of an era).
- `[?]` **Real main menu** with seed selection and sharing — determinism (GDD §5.7) makes sharing interesting seeds worthwhile.

### Onboarding & engagement (2026-08-09, from `redesign/abiogenesis-engagement-design.md`)

Diagnosis: turn zero is a blank world with no signal there's anything to discover; reproduction (start energy `5.0`, threshold `10.0`, ~`+0.9`/tick photolithic gain) outruns a `25`-tick era before the player sees anything happen; matrix `interaction_delta` applies invisibly, read only after evidence accumulates. Full diagnosis and rationale in the linked doc.

**Onboarding foundations:** scoped into tasks 080-084, see SECTION 2's "Onboarding & engagement rollout (2026-08-09)". While scoping 1.E specifically, the proposal itself shrank to a small task (081), but raised a larger, separate idea — see next entry.

**Environment as sources:** scoped into tasks 085-086 (2026-08-10), see SECTION 2's "Environment as sources" entry. Full mechanics, open questions, and blast-radius analysis in `redesign/abiogenesis-environment-sources.md`.

**"Epic" mechanics — subito, da approfondire:**

- `[?]` **2.2 — The Precursor**: a single fixed anomalous cell per world (not an auto-placed species, stays banned) with a constant chemistry that affects the matrix regardless of which tag the player brings near it — a narrative attractor discoverable purely by observation. **Decided to implement by reusing the existing tag/matrix system** (a phantom organism/tag that participates in `interaction_delta` like any other), not as a separate hard-coded effect — consistent with the project's no-parallel-systems convention. **Linked to the "Mondo vivo" proposal below** (2026-08-10): its transient-residue mechanic is the same "phantom tag entity" pattern, procedural/recurring instead of fixed/singular — the two should share an implementation shape when either is scoped.
- `[?]` **2.4 — Epochal events**: rare procedural global shocks built on existing environmental scalars (`temperature`, `light`, `toxicity`) — a toxicity spike, a light flare, a thermal collapse — giving each run a natural growth → shock → adapt/collapse arc with no new data system. **Frequency criterion decided**: no events before an onboarding-window era threshold, then a per-era probability that increases over time, parameterized in `SimConfig` — not a fixed-point guaranteed event, not a constant probability from the start. Sequence last among the "subito" set: an epochal shock during the onboarding window would undermine the stability 1.B/1.C/1.D are meant to build.
- `[?]` **2.6 — The revelation moment**: on closing a world, algorithmically synthesize one sentence from the confirmed-relationship graph (e.g. "in this biochemistry, decomposition generates vitality") — zero art, zero hand-written text, derived entirely from data the game already has. **Empty-graph fallback decided**: when `MatrixKnowledge` has no confirmed relations at world close, pick randomly among several varied neutral phrases (not one fixed line, not silence) — keeps a sense of dynamism/mystery even when nothing was confirmed.

**"Epic" mechanics — futuro:**

- `[?]` **2.1 — The stratigraphic record**: per-cell persistent death log (who died where, when, how concentrated), "core-sampleable" by the player — more data-modeling work (per-cell vs. global log) than the rest of this set. **Retention decided**: capped per cell, oldest entries compress into an aggregate summary (e.g. per-species counts) rather than growing unbounded — the real cost here is the retention policy, not the logging itself; "no new simulation" only holds once this is bounded.
- `[?]` **2.3 — Prior-expedition data**: some worlds start with a few matrix cells pre-filled as "unverified prior readings," some correct, some wrong — stays consistent with "unlock capabilities, not answers" (GDD §10), but needs careful design of credible-yet-sometimes-false testimony that doesn't break trust in the notebook as a tool. **Data model decided**: lives as a separate "unverified" layer, visually distinct, never merged into `MatrixKnowledge`'s real-evidence weighting (task 020, `1/(1+n_confounders)`) — the player promotes a testimony to a real hypothesis only by testing it themselves. Falsification needs a procedural (not hand-authored) rule, seeded deterministically per world.
- `[?]` **2.5 — The hidden grammar between worlds**: a deeper structural regularity under each world's shuffled matrix, perceptible only across many runs (e.g. certain glyph patterns statistically skew catalyst vs. poison) — a meta second layer of mystery for veterans; the most ambitious and most delicate to balance without introducing exploitable real patterns. **Note**: unlike 2.1/2.3, this touches the shared matrix generator itself (task 011, cyclicity constraint), not a downstream system — the exploitability risk is cross-run (a veteran who cracks it knows it on every future world, not just within one session), higher stakes than 1.B's within-session repetition risk. Stays purely conceptual until defined.

### Mondo vivo (2026-08-10, from `redesign/abiogenesis-living-world.md`)

Raised directly by the user as a follow-up to the onboarding/engagement work
(all of 080-086 landed): the opening turns should feel like discovering
*where* a species can live and evolve, not just picking tags and watching
numbers move. Full mechanics, rejected alternatives, and open questions in
the linked doc.

- `[?]` **Conditional tags (zone↔matrix)**: a minority of species tags gain an optional expression condition (`TerrainKind`) — the tag only participates in `interaction_delta` while the organism occupies a matching zone. **Decided**: not a per-terrain intensity multiplier, not a separate matrix per biome — the matrix stays single/world-global; only which tags are active changes. Biochemical grounding: conditional gene expression/operon regulation (trait exists in the genome, only expressed under specific environmental conditions). Main open risk: the notebook has no "terrain-conditioned" axis today (§7's confounder weighting) — needs its own legibility pass before scoping.
- `[?]` **Wild, pre-existing species**: small non-player-seeded populations placed in not-immediately-reachable zones at world-gen, reusing the `Species`/tag/matrix system as-is. First contact (a player species reaching interaction range) is the discovery trigger — reuses the interaction-spark trigger (task 080) and the unconfirmed→confirmed notebook event (task 054).
- `[?]` **Reveal-on-first-zone-entry**: same mechanism as conditional tags — a player species occupying a `TerrainKind` for the first time in a run, while carrying a tag conditioned on it, logs a notebook discovery event. No separate system from the conditional-tags mechanic above.
- `[?]` **Transient world features (e.g. low-decay residue)**: a lighter-weight instance of the Precursor's (2.2) "phantom tag entity" pattern — procedural/recurring instead of fixed/singular, decaying over time to give species a limited discovery window.

**Scoped into tasks 096-099** (2026-08-11, SECTION 2's "Mondo vivo, notebook,
death legibility, evolution & progression" entry) — the transient
world-features/residue item above stays unscoped, needing the shared
"phantom tag entity" design pass with the Precursor (2.2) first.

**Real-time mode** (already a `[?]` above, "Camera, pacing & menu features") was raised again in this same discussion and **explicitly deferred**: the user agreed to keep it independent, revisited only after these mechanics are scoped, implemented, and playtested — stacking it now would add a second, simultaneous source of reduced legibility on top of these.

### Notebook UX redesign (2026-08-10, from `redesign/abiogenesis-notebook-redesign.md`)

Spun out of the "Mondo vivo" discussion above (conditional tags need *some*
notebook surface) into its own, independent track: a live screenshot review
confirmed the current notebook has standing legibility problems of its own,
unrelated to any new mechanic. Full findings, decisions, and open questions
in the linked doc; intersects with "Mondo vivo" only at the conditional-tags
catalog badge.

- `[?]` **Observation log**: strip raw per-tick "observed" noise entirely — keep only narrative events (births/deaths/extinctions, `★` confirmations). Evidence accumulation moves to the hypothesis grid instead of spamming text.
- `[?]` **Hypothesis grid — visibility & layout**: zero-evidence tags don't render at all until first touched; layout stays simple/static (not full force-directed physics — a deliberate complexity/ROI call, not a legibility objection to physics in principle).
- `[?]` **Hypothesis grid — edge grammar**: numeric `±N` labels replaced by line thickness for magnitude; partial-evidence dots replaced by dashed lines (semantically apt — sign/magnitude genuinely unknown pre-confirmation); bidirectional pairs drawn as offset/curved arcs instead of overlapping straight lines.
- `[?]` **Visual polish pass**: palette, background, spacing, and typography across all three panels — not yet specified, needs its own mini design pass before scoping.
- `[?]` **Catalog cleanup**: dedupe repeated per-metabolism boilerplate into a one-time legend; trim species rows to parameters only — raised as uncontroversial but not yet formally confirmed.

**Scoped into tasks 100-103** (2026-08-11). The visual polish pass stays
unscoped — needs its own concrete mockup/design pass before it's
task-ready.

### Death/failure legibility (2026-08-11, from `redesign/abiogenesis-death-legibility.md`)

Companion topic spun out of the notebook redesign discussion: bad
temperature fit, no nearby prey/resource, etc. are largely invisible today
beyond a raw-number death breakdown for player-placed organisms only
(`text.rs:389-408`). **Key framing**: GDD §7 already declares metabolisms
and environmental ranges "always readable as anchors" — so being fully
direct about these causes costs zero discovery value; only the matrix
(`interaction_delta`) term must stay deliberately vague, per §11.

- `[?]` **Anchor-cause plain language**: rewrite the existing player-placed death message from a 5-number dump into one qualitative sentence per death (poor temperature fit / no resource / overcrowded / eaten / harmed by a nearby species for the matrix case) — requires a small `OrganismDied` (`sim.rs:16-31`) schema addition to distinguish "poor temperature fit" from "resource genuinely absent," both of which collapse into the same `gain` value today.
- `[?]` **Matrix-cause number removed**: when `interaction_delta` dominates, drop the exact number from the message (consistent with the hypothesis-grid decision above to stop showing raw numbers) — still says *that* a species harmed it, never which tag/sign/magnitude.
- `[?]` **Aggregate Biosphere trend diagnosis**: for population-wide cases (e.g. a predator population quietly starving from lack of prey, not player-placed), attach a short dominant-cause label to the HUD's existing ▼ trend indicator instead of adding per-organism log spam for untracked organisms.

**Scoped into tasks 104-105** (2026-08-11). Scoping corrected two
assumptions in the doc above: the HUD's population trend is driven by
era-over-era average survivor energy, not a death tally (`ui.rs:1006-1116`)
— so 105 gates its cause label on "at least one death this era," not on
the ▼ glyph specifically, which can otherwise show ▲/▬ even while deaths
are happening.

### Evolution & xenotypes (2026-08-11, from `redesign/abiogenesis-evolution-xenotypes.md`)

Raised directly by the user: the matrix "aha" has no real payoff today
beyond knowledge itself. Two proposals, resolved against tensions already
flagged (unresolved) in `VISION.md` Phase C (Evolution) and Phase D
(Biochemistry flavor) — full reasoning, corrected assumptions, and open
questions in the linked doc.

- `[?]` **Evolution via speciation, never in-place mutation**: crossing an accumulated "selection pressure" threshold (stimuli: `interaction_delta` sign/magnitude experienced, terrain occupancy, toxicity exposure — mirrors `MatrixKnowledge`'s evidence-then-confirm shape) spawns a new descendant species, reusing `Splice`'s existing genome-editing plumbing (`SpliceEditChoice`, `input.rs:531-565`) and procedural naming (`draw_species_name`, task 095). Never mutates an existing, already-tested species in place — this is what fully resolves `VISION.md` §C's "evidence going stale" risk, not a mitigation of it.
- `[?]` **Evolution can touch tags, not just the anchor layer** — corrected from an initially more cautious anchor-only stance: since speciation never changes the world's fixed matrix values and descendants only draw from tags already active in that world, a new tag on a descendant exposes an already-fixed relationship rather than adding a new mystery axis.
- `[?]` **Bigger authored starting roster** (not just emergent via evolution) — independent, near-term want.
- `[?]` **More metabolisms, near-term** — first candidate is the chemolithotroph already named in GDD §5.4 (linked above), ahead of any full xenotype/archetype redesign.
- `[?]` **Xenotype/archetype naming redesign: deferred** until after the game stabilizes. Design principle to carry forward: describe what a trait *is* (its biochemical nature), never how it interacts — real microbiology already separates those two facts. `VISION.md` §D's litmus test ("could a first-time player guess the matrix effect from the name alone?") applies whenever this is picked up.

**Species-persistence follow-up (2026-08-11)**: how an evolved species survives a world reset is answered in "Progression & pacing" below — within-run only, via `RunProgress` (mirroring `MetaProgress::absorb`'s existing `worlds_cleared → bonus_available_species` shape), not the deferred cross-run persistence layer.

**Scoped into tasks 106-108** (2026-08-11) — the new-species presentation
moment (open question above) and the full xenotype naming redesign stay
unscoped, both explicitly not designed yet.

### Progression & pacing (2026-08-11, from `redesign/abiogenesis-progression-pacing.md`)

Reconciles "Mondo vivo" and "Evolution & xenotypes" (both above) with the
current core loop: clearing a world's objective sequence today triggers a
full reset (grid/species/matrix/notebook knowledge all wiped,
`run_flow.rs:68-145`) — any long-term ecosystem investment is destroyed
right as it starts to pay off. Full reasoning and open questions in the
linked doc. **Sequenced after** "Mondo vivo" and "Evolution & xenotypes"
get scoped — this doc's long-term objective content depends on their
mechanics existing.

- `[?]` **Long-term objective tier replaces the current sequence as the reset trigger**: a new `Objective` variant tied to mondo-vivo/evolution milestones (e.g. a speciation event, sustained population over many eras, N confirmed matrix relations) becomes what actually clears a world; existing short-term objectives keep today's in-place-advance behavior (`apply_tick_outcome`, `objectives.rs:403-467`) but now grant an energy reward on each clear, which they don't today. The fresh-matrix-per-world pillar itself is untouched — only how long a world lasts before the reset changes.
- `[?]` **New within-run energy resource**: distinct from the per-era-resetting `ActionBudget` (`sim.rs:436-445`), lives at the `RunProgress` level (`run.rs:15-21`, already survives world-to-world resets within a run).
- `[?]` **`Splice` stays available from world 0**: energy unlocks a more powerful tier (more simultaneous edits, reduced action-point cost) instead of gating the action's existence — deliberately avoids touching the already-tuned onboarding (tasks 082/083, grace period).

**Implemented as task 109 (unblocked and shipped 2026-08-12)** once
096-099/106-107 all landed — see the task file for the concrete
`Objective::Speciation` content and energy-economy wiring.

### Biomes (2026-08-11, from `redesign/abiogenesis-biomes.md`)

16 discrete biomes replacing the flat `TerrainKind` bands as the primary
environmental classification, reconciled against the current codebase in a
design-discussion pass: `TerrainKind`/`is_peak` (task 066/068) already cover the
elevation-based biomes structurally, task 085's point heat sources already back
Bocca vulcanica, and task 108 (chemolithotroph) makes the per-biome `toxicity`
values load-bearing rather than cosmetic. Full reasoning in the linked doc.

- `[?]` **Two-stage biome classification**: Stage A reuses `TerrainKind`/`is_peak`
  for the base landform; Stage B refines it into a final biome from
  `temperature`/`light`/`toxicity`, with feature biomes (Cratere profondo, Distesa
  di cristalli, Lago, Bocca vulcanica) placed explicitly (bounded-retry, same
  pattern as `place_toxic_zone`) rather than derived from thresholds.
- `[?]` **Palude replaces `toxic_zone`**: the isolated rectangle is superseded by a
  proper biome; the `SurviveIn`/`ZoneKind::Toxic` objective (`objectives.rs:320-336`)
  moves from rectangle-containment to biome-cell-membership.
- `[?]` **Geyser stays out of reach**: needs a small/pulsing heat-source category
  that task 085 doesn't provide (every source it generates is mechanically
  identical) — a real, currently-unscoped prerequisite, not just a formality.

**Scoped as tasks 110-114 (2026-08-11)**: 110 (classification data layer), 111
(feature placement), 112 (rendering), 113 (Palude/`toxic_zone`) are startable per
the dependency order in the tasks themselves; 114 (Geyser) is scoped for reference
but BLOCKED, same pattern as 084.

---

## 🔵 SECTION 2 — BACKLOG (Operational)

> Approved tasks. Phase 0 is already expanded into task files; later phases expand when we get there.

### 🌍 Mondo vivo, notebook, death legibility, evolution & progression (2026-08-11)

Full scoping pass over the five redesign docs from this session's design
discussion (`redesign/abiogenesis-living-world.md`,
`abiogenesis-notebook-redesign.md`, `abiogenesis-death-legibility.md`,
`abiogenesis-evolution-xenotypes.md`, `abiogenesis-progression-pacing.md`
— SECTION 1 above has the full reasoning). Dependency order: 096
(conditional tags core) unblocks 099 and 097 (097 also depends on 103);
098, 100-106, 108 are independently startable; 107 depends on 106; 109
was blocked until 096-099/106-107 actually shipped, then implemented
2026-08-12.
Intentionally **not** scoped: transient world features/residues (needs a
shared design pass with the Precursor), the notebook's visual polish pass,
evolution's new-species presentation moment, and the full xenotype naming
redesign — all explicitly flagged as not design-ready in their source docs.

- `[x]` 096 — Conditional tags: terrain-gated matrix participation → [096](tasks/done/096-mondo-vivo-conditional-tags-core.md)
- `[x]` 097 — Conditional tag catalog badge + (tag, terrain) evidence track (depends on 096, 103) → [097](tasks/done/097-mondo-vivo-conditional-tag-catalog-badge.md)
- `[x]` 098 — Wild, pre-existing species at world generation → [098](tasks/done/098-mondo-vivo-wild-species.md)
- `[x]` 099 — Reveal-on-first-zone-entry for conditional tags (depends on 096) → [099](tasks/done/099-mondo-vivo-zone-entry-reveal.md)
- `[x]` 100 — Strip raw per-tick noise from the observation log → [100](tasks/done/100-notebook-log-rework.md)
- `[x]` 101 — Hypothesis grid: reveal-on-first-observation, layout over visible subset only → [101](tasks/done/101-notebook-grid-visibility-layout.md)
- `[x]` 102 — Hypothesis grid: edge grammar rewrite (thickness, dashed partial lines, curved bidirectional arcs) → [102](tasks/done/102-notebook-grid-edge-grammar.md)
- `[x]` 103 — Catalog: one-time metabolism legend, trimmed species rows, population + origin era (extended 2026-08-12, see "HUD & Notebook redesign follow-up" below) → [103](tasks/done/103-notebook-catalog-cleanup.md)
- `[x]` 104 — Plain-language death message for player-placed organisms → [104](tasks/done/104-death-message-plain-language.md)
- `[x]` 105 — Cause label on the Biosphere panel for a species taking deaths (depends on 104) → [105](tasks/done/105-death-biosphere-trend-diagnosis.md)
- `[x]` 106 — Selection pressure accumulation + threshold-crossing trigger → [106](tasks/done/106-evolution-selection-pressure-trigger.md)
- `[x]` 107 — Evolution by speciation: a new descendant species from selection pressure (depends on 106) → [107](tasks/done/107-evolution-speciation.md)
- `[x]` 108 — Fourth metabolism: chemolithotroph, gain from toxicity → [108](tasks/done/108-chemolithotroph-metabolism.md)
- `[x]` 109 — Long-term objective tier + within-run energy economy (depended on 096-099/106-107 shipping) → [109](tasks/done/109-progression-long-term-objective-energy.md)

### 🖥️ HUD & Notebook redesign follow-up (2026-08-12, from `redesign/abiogenesis-hud-notebook.md`)

Before starting the notebook UX track's next task, read this new doc against
what tasks 097/100-103 already planned, specifically to avoid double work.
Discrepancy-check findings and resolutions (decided with the user):

- **Already aligned, no new work**: curated observation log (100), grid
  reveal-on-evidence + static layout (101), grid edge grammar (102), shared
  metabolism icon set (065, reused by 103), "Moves rimasti" as tick dots
  (already implemented that way), notebook-button unread badge (exists,
  confirmation-triggered).
- **Folded into task 103** (catalog card gains current population and
  origin era — the latter needs new per-species state, since only
  `Organism::born_era` exists today, not a per-`Species` field; task 103's
  file now documents the `SimWorld::push_species`/`species_origin_era`
  approach, mirroring 098/099's `wild_species`/`terrain_occupancy`
  precedent).
- **Kept as currently scoped, doc's broader wording not adopted**: task
  100's notebook-button badge stays confirmation-only (not "any new
  observation") — the doc's "osservazioni nuove non ancora lette" reading
  is broader than what's built and would need its own decision; not
  revisited now.
- **New tasks scoped**: 116 (notebook docked-left panel + dimmed map,
  replacing today's floating `egui::Window`), 117 (era-relative time
  readout, replacing the current run-wide tick counter), 118 (tick → pulse
  rename, cosmetic only, no internal identifier renames), 119 (Moves icon
  restyle — narrowed from the doc's full ask: while investigating, found
  `ui.rs`'s own doc comment already documents that 3 of the 4 current Move
  icons are almost certainly rendering as tofu boxes today, since egui has
  no color-emoji support — this task fixes that as a side effect of
  switching to monochrome glyphs), 120 (Biosphere population delta — found
  during scoping that the existing trend arrow is energy-based, not
  population-based, so the new delta and the existing arrow can legitimately
  disagree; flagged explicitly in the task rather than silently resolved).
- **Deliberately not scoped**: auto-advance play/pause (deferred until a
  real-time pacing mechanic exists), the mutation-tier badge (no
  progression mechanic exists to back it — 119 does the icon restyle only,
  no badge), and the doc's own claim that Stress is "a new action" (it
  isn't — already shipped; the doc appears to have been written without
  checking current game state on this specific point, noted here as a
  caution for reading the rest of it).

- `[x]` 116 — Notebook: left-docked panel with dimmed map behind it, not a floating window → [116](tasks/done/116-notebook-docked-panel-dimmed-map.md)
- `[x]` 117 — Time readout: show progress within the current era, not the run-wide tick counter → [117](tasks/done/117-time-readout-era-relative-pulse-progress.md)
- `[x]` 118 — Rename player-facing "tick" to "pulse" → [118](tasks/done/118-rename-tick-to-pulse.md)
- `[-]` 119 — Moves icons: monochrome glyphs that actually render (fixes a pre-existing tofu-box bug) — cancelled 2026-08-26, superseded by a planned redesign (see `tasks/QUEUE_ARCHIVE.md`)
- `[x]` 120 — Biosphere: numeric population delta alongside the trend arrow → [120](tasks/done/120-biosphere-population-delta.md)

### 🗻 Biomes (2026-08-11)

Full scoping pass over `redesign/abiogenesis-biomes.md` (SECTION 1 above has the
full reasoning). Dependency order: 110 unblocks 111 and 113; 112 depends on both
110 and 111; 114 is scoped for reference but blocked on an unscoped small/pulsing
heat-source category.

- `[x]` 110 — Biome enum + two-stage classification (areal biomes) → [110](tasks/done/110-biome-classification-two-stage.md)
- `[x]` 111 — Explicit placement for feature biomes (depends on 110) → [111](tasks/done/111-biome-feature-placement.md)
- `[x]` 112 — Biome rendering: flat color, dithering, borders, tree overlay (depends on 110, 111) → [112](tasks/done/112-biome-rendering.md)
- `[x]` 113 — Palude replaces `toxic_zone` (depends on 110) → [113](tasks/done/113-swamp-replaces-toxic-zone.md)
- `[ ]` ⏸ 114 — BLOCKED — Geyser biome (needs a small/pulsing heat-source category, not yet scoped) → [114](tasks/114-geyser-pulsing-source-blocked.md)

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
- `[x]` 090 — Terrain island-band retune: organic coastlines, less sea: user-reported follow-up surfaced during the 085/086 live playtest session (not the original 069-072 sequence) — `Sea` was generated in "quantità eccessiva e forme non troppo credibili," several same-sized isolated near-circular blobs scattered inland ("polka-dot lakes") rather than a connected coastline, which also diluted 085's heat-source visibility (`sea_coolant_radius` reached more of the map with more sea). A throwaway ASCII-dump + 30-seed histogram diagnostic (same technique 069 used) isolated the island wave band as the cause (`island_blend_weight: 0.0` made the pattern vanish): few summed waves (6) interfere into a regular periodic pattern instead of organic noise. `island_wave_count` raised 6→16, `island_blend_weight` raised 0.45→0.55 (compensating the smaller per-wave amplitude at higher count), `sea_threshold` lowered 0.42→0.34 (sea coverage ~33%→~24% over 30 seeds) — re-verified against the same before/after histogram technique to avoid silently collapsing Mountain/peak reachability, task 069's own recorded failure mode → [090](tasks/done/090-terrain-island-band-retune.md)

### 🌱 Onboarding & engagement rollout (2026-08-09, from `redesign/abiogenesis-engagement-design.md`)

The 5 "onboarding foundations" proposals (§1) scoped into task files after a
multi-round discussion covering sequencing, permanence, and reuse of
existing systems — full reasoning kept in each task file's Objective/
Technical Context. 080 ships first (diagnostic value: makes the other
changes' effects visible while playtesting them). 082/083 are numerically
coupled and must be tuned together, not in isolation. 084 is scoped but
explicitly blocked — see its file.

- `[x]` 080 — Interaction spark: instant visual feedback on first-seen relations → [080](tasks/done/080-interaction-spark-visual-feedback.md)
- `[x]` 081 — The world breathes: toxic zone pulse + diffusion drift check: `toxicity_tint` (`render.rs`) now oscillates its blend strength (`0.45 * (0.85 + 0.15 * (elapsed * 0.4).sin())`, `elapsed` from `Res<Time>` threaded through `cell_color`) instead of a fixed `0.45`, only visible where `toxicity > 0`. Diffusion drift-perceptibility check (code inspection): not perceptible on the base map — `Cell::biome` (what the empty-cell branch renders) is assigned once at generation and never re-derived from `temperature`/`light` per tick, so the eroding gradient has no render effect outside the dev-only overlay; making it visible is out of scope here, tracked in `redesign/abiogenesis-environment-sources.md` instead. Live verification skipped this pass at user request → [081](tasks/done/081-ambient-diffusion-visible-on-empty-grid.md)
- `[x]` 082 — Shorter eras during world 0's opening: `TimeConfig` gained `onboarding_eras`/`onboarding_era_ticks` (defaults `3`/`8`, mirrored into `sim_config.ron`); a new `worldgen::era_ticks_for(world_index, era, config)` helper returns the onboarding length only for world 0's first `onboarding_eras`, standard `era_ticks` otherwise. `start_era`/`single_tick` (`input.rs`) now call it instead of reading `config.time.era_ticks` directly, threading in `RunProgress`/`SimWorld`. Fixed an existing test (`repeated_single_ticks_alone_complete_an_era_and_refill_the_budget`) that implicitly assumed world 0/era 0 always uses the standard length — updated to `world_index: 1` since it's testing generic tick bookkeeping, not the onboarding exception → [082](tasks/done/082-shorter-onboarding-eras.md)
- `[x]` 083 — Newborn incubation: reproduction delayed to the following era: `Organism` gained `born_era: u32`; the child spawn site in `sim.rs` sets it to `world.era` explicitly, the parent's post-cost copy inherits it via `..organism`. Reproduction now additionally requires `organism.born_era < world.era`. No new `SimConfig` coefficient — a structural gate on existing state, per the task's explicit constraint. `tests/balance.rs`'s population-dynamics suites re-ran clean with no assertion changes needed. New inline test confirms a same-era-born organism can't reproduce even above `repro_threshold`, and can once `world.era` advances → [083](tasks/done/083-newborn-incubation-reproduction-delay.md)
- `[ ]` 084 — Guaranteed "first light" relation in world 0's matrix (**BLOCKED** on the "Meta-progression persistence" proposal, §1 — do not start) → [084](tasks/084-first-light-guaranteed-relation-world0.md)

> The "epic" mechanics (2.2/2.4/2.6 subito, 2.1/2.3/2.5 futuro) remain `[?]` in §1 — not scoped this round.

### 🌡️ Environment as sources (2026-08-10, from `redesign/abiogenesis-environment-sources.md`)

Replaces the fixed left-right temperature / top-bottom light gradients with per-world heat sources (+ wind bias, + `Sea` cells as passive coolant, + reinjection to counter diffusion erosion) and a per-world sun direction (+ `Mountain` shading). Same class of change as the terrain redesign (066-072): worldgen + downstream balance, not just rendering. 085 is the combined temperature+light generation task (Sea/Mountain coupling folded into its acceptance criteria rather than split out); 086 is a legibility check on the existing T/L overlays once 085 lands. Follow-ups deliberately not pre-planned — filed individually if playtest surfaces them, mirroring 069-072. GDD §5.2's "2D niche via crossed axes" framing is explicitly not preserved and gets rewritten after implementation + playtest, not before.

- `[x]` 085 — Source-driven temperature and light → [085](tasks/done/085-source-driven-temperature-and-light.md)
- `[x]` 086 — Environment overlay legibility check → [086](tasks/done/086-environment-overlay-legibility-check.md)

### 🗺️ Camera pan (2026-08-10)

Zoom was already done (075-076); pan was the remaining open half of the old "camera zoom and pan" backlog item. Deliberately revisited an earlier explicit design call (the now-deleted `abiogenesis-two-tier-view.md` and task 075's own acceptance criteria both concluded "no separate pan mechanic needed") now that the grid is `128×80` (task 074) and Detail-zoom navigation via zoom-drift alone was impractical. Arrow-key pan added; the existing zoom clamp was factored into a shared `clamp_camera_pan` helper both systems now call, covered by 3 new unit tests. Live-verified via `cargo run` driven headlessly with `cliclick`/`screencapture`/`osascript` System Events (`cliclick`'s own synthetic keyboard events didn't reach the app — likely an Accessibility/Input Monitoring gap — `osascript`'s System Events did); pan direction/continuity confirmed via a temporary debug log rather than by eye, since screenshot pixel-comparison was ambiguous. **Same-day follow-up**: playtest feedback found the pan too slow (retuned `24 → 60 → 120` cells/s) and wanted `WASD` too — `input::single_tick` was rebound `KeyS → KeyN` to free up `S`, and `WASD` added alongside the arrow keys. See task file's "Follow-up tuning" addendum.

- `[x]` 087 — Camera pan → [087](tasks/done/087-camera-pan.md)

### 🐛 Self-interaction balance bug (2026-08-10, user-reported live)

User reported starting species growing explosively (energy + population)
despite task 083's incubation. Diagnosed via two rounds of exploration +
plan agents: the incubation gate itself is correct — the actual cause is
`draw_species_tags`'s task-048 mitigation (retry up to 20 random candidates
for `net_self_interaction == 0`, falling back to "closest to zero, possibly
still nonzero") being combinatorially unable to succeed in ~15% of worlds
given the real default 5-tag active pool at world 0 (`C(5,3) = 10` possible
3-tag combinations; roughly 15% of randomly-generated matrices have none of
those 10 net exactly zero — not a rare retry failure, a guaranteed outcome
whenever the search space itself has no solution). With only 3 species
generated per world, this reads as "often", matching the report exactly.
088 fixes this properly: exhaustive deterministic search at decreasing tag-set
size, always terminating at an exact-zero result (1-tag sets are trivially
always safe). A separate, independently-diagnosed gap — `apply_splice`
(player-driven species creation) never applied this check at all — is 089;
it wasn't the cause of what the user saw (their case was a starting/worldgen
species, confirmed explicitly), but is a real, consistent-with-048 fix worth
doing regardless.

- `[x]` 088 — Exhaustive search in `draw_species_tags` to guarantee zero self-interaction: replaced the random-sample-with-retry search with a deterministic enumeration at decreasing tag-set size (3→2→1 tags), guaranteed to always terminate at an exact `net_self_interaction == 0` result (a 1-tag set is trivially always safe). Removed the now-unnecessary `config.tags.max_self_conflict_draws`. New 300-seed property test under the real default config confirms the invariant directly; `tests/balance.rs` re-run clean with no threshold changes needed → [088](tasks/done/088-exhaustive-self-neutral-species-tags.md)
- `[x]` 089 — Reject Splice edits that create a nonzero same-species self-interaction → [089](tasks/done/089-splice-self-interaction-gate.md)

### 🐛 Bugfixing & UX follow-ups (2026-08-10)

Three bugs plus three improvements, user-reported live during a
debugging/planning discussion held right after tasks 085-090 landed.
Investigated and scoped in one session, no implementation yet. 091 merges
two input bugs sharing one root cause: nothing in the codebase checks
`bevy_egui`'s own input-capture state (`EguiWantsInput`, auto-registered by
`EguiPlugin` since `bevy_egui` 0.41) before the map's own zoom/click/`Tab`
systems run — the notebook window doesn't block map interaction underneath
it, and `Tab` fights egui's own keyboard-navigation focus-cycling. 092 is a
task-082 side effect: `ISOLATION_HINT_DURATION_TICKS` is a fixed 30-tick
window that used to read as "about one era" back when every era was 25
ticks, but no longer tracks the shortened onboarding eras (8 ticks) task
082 introduced. 093 reverses a stale task-068 decision (`Sea` deliberately
rendered near-black to read as "void") now that task 085 gave `Sea` a real
mechanical role. 094 (on-screen buttons for tick/era/notebook controls,
coexisting with their keyboard shortcuts) and 095 (species names drawn from
the world's own RNG instead of a fixed id-indexed list, plus a readable
description per species) are real feature additions, not fixes. No
dependency between the five — pick up in any order.

- `[x]` 091 — Gate map input (zoom/click/Tab) behind egui's own input capture → [091](tasks/done/091-egui-input-capture-gating.md)
- `[x]` 092 — Isolation hint duration should scale with era length → [092](tasks/done/092-isolation-hint-duration-scales-with-era.md)
- `[x]` 093 — Sea should read as water, not "end of the world" → [093](tasks/done/093-sea-color-reads-as-water-not-void.md)
- `[x]` 094 — On-screen buttons for tick/era/notebook controls → [094](tasks/done/094-hud-buttons-for-tick-era-notebook-controls.md)
- `[x]` 095 — Procedural per-world species names + readable descriptions (follow-up noted: the description reads flat/repetitive across species, phrasing fixed per metabolism — left for a future task if picked up) → [095](tasks/done/095-procedural-species-names-and-descriptions.md)

### 🐛 Temperature-spread bug + HUD click-through (2026-08-11, user-reported live during 098/099 playtest)

Two issues surfaced during manual playtesting of tasks 098/099. Both
diagnosed same session; the first fixed immediately (small, contained, no
open design questions), the second scoped as its own task since the root
cause needs investigation before it can be fixed correctly.

`temp_optimum` generation (`generate_starting_palette`/`add_bonus_species`/
`place_wild_species`) used to read two fixed grid corners
(`world.get(0, 0)`/`world.get(width - 1, 0)`) as the "cold"/"hot" endpoints
to spread species temperatures across — a leftover from before tasks 085/086
replaced the left-right temperature gradient with a heat-source-distance
model. Those two corners very often both landed near `ambient`, so almost
every generated species' `temp_optimum` fell in `notebook.rs`'s "cold" band
regardless of the world's real range (confirmed via a diagnostic probe
across 8 seeds). Fixed by sampling from the actual distribution of
*placeable* cells' temperatures (`worldgen::placeable_temperature_distribution`),
mapped through an interior 10th-90th percentile band
(`temp_optimum_at_percentile`) rather than the distribution's literal
extremes — the naive whole-grid min/max version regressed `tests/balance.rs`
badly (25/50 → still 21/50 seeds hitting extinction) because
`tests/balance.rs`'s own `place_starting_organisms` placed species 0/1 at
those exact same two corners, a tautological fit=1.0 coupling that the fix
had to break intentionally; the test's placement was corrected to seek the
placeable cell nearest each species' own `temp_optimum` instead, which
restored green (`0/50` across all four balance properties).

- `[x]` Temperature-spread fix (no task file — small, contained, fixed same
  session as diagnosed; see `worldgen.rs`'s `placeable_temperature_distribution`/
  `temp_optimum_at_percentile` and `tests/balance.rs`'s `place_starting_organisms`)
- `[x]` Task 103 follow-up (no task file — reported live 2026-08-12, fixed
  same session): catalog's "seeded era" label showed for species never
  actually placed on the grid. Added `SimWorld::species_ever_placed`
  (`world.rs`), set in `sim::step`; `species_population_line` now takes
  `Option<u32>` and omits the era when unset (`text.rs`, `notebook.rs`).
- `[x]` Task 105 follow-up (no task file — reported live 2026-08-12, fixed
  same session): the Biosphere row's cause label overflowed past
  `HUD_WIDTH` instead of wrapping. Switched the row's `ui.horizontal` to
  `ui.horizontal_wrapped` (`ui.rs`).
- `[x]` 115 — Grid input (clicks and scroll-zoom) leaks through the HUD panel → [115](tasks/done/115-egui-panel-click-through-when-zoomed.md)
- `[x]` 121 — Conditional-tag catalog badge never renders in a live playtest: root cause was `accumulate_terrain_evidence` (`notebook.rs`) keying confirmation evidence on `event.terrain` (whichever terrain the organism was standing on when the gate happened to be evaluated) instead of `conditional.terrain` (the tag's one fixed trigger terrain, the only one `conditional_tag_badge` ever queries) — evidence fragmented across every visited `TerrainKind` instead of concentrating on the one slot the badge reads. Fixed by keying both the evidence record and the confirmation log message off `conditional.terrain`. New regression test (`accumulate_terrain_evidence_confirms_the_trigger_terrain_even_when_observed_elsewhere`) reproduces the bug without egui. Live verification skipped this pass at user request → [121](tasks/done/121-terrain-badge-missing-in-catalog.md)
- `[x]` 122 — Toxic zone reinjection: toxicity erodes over long runs with no source to counter diffusion, unlike heat sources (found balance-testing task 108's chemolithotroph) → [122](tasks/done/122-toxic-zone-reinjection.md)

### 🎚️ Final tuning — *the real art*

**Goal:** *interesting and readable* emergence, avoiding "everything dies" and "one dominates" (GDD §13, §14).

> **🐛 Playtest finding (2026-08-04, seed `1231000211577056359`), fixed by task 048 (2026-08-06):** by era 9/tick 225 one species (Kael, species 1) had saturated the entire grid — population 1536 = exactly `48×32`, zero empty cells anywhere — with average energy 1039.53, roughly two orders of magnitude above normal (`seed_energy` 5.0, `repro_threshold` 10.0). Root cause, confirmed with a second independent repro during task 048: `world::draw_species_tags` rejected a candidate tag set that net-*drained* itself (a species dying the moment it reproduces next to itself) but not one that net-*reinforced* itself, and `sim::step`'s `crowd_factor` penalty (`0.15`/neighbour) is dwarfed by a single matrix entry (up to `±2`) — so any species whose own tags reinforced each other turned same-species clustering into unbounded growth, exactly the GDD §14 "one dominates" failure mode. Fixed by requiring `net_self_interaction == 0` instead of merely `>= 0`. A stronger/nonlinear crowding penalty (the other candidate lever noted below) was tried and rejected during the investigation — strong enough to matter for cross-species reinforcement (a residual, smaller-magnitude version of the same failure mode this fix doesn't reach), it also crushed normal populations toward extinction.

- `[ ]` Tuning of the three anti-degeneration levers: cyclicity, environmental heterogeneity, carrying capacity (GDD §5.8)
- `[ ]` Tuning of tick coefficients and the notebook confirmation threshold (GDD §5.6, §5.9, §7)
- `[x]` Final grid size (remains empirical, GDD §5.1) — task 074 → [074](tasks/done/074-final-grid-size-tuning.md)
- `[x]` Migrate config to RON with hot-reload, to shorten the tuning cycle — task 073 → [073](tasks/done/073-ron-config-hot-reload.md)

> **📐 Task 074 (2026-08-09): grid size raised from `48×32` (1536 cells) to `128×80` (10240 cells, ~6.7x).** The user wants a substantially larger, richer world for future plans beyond what a decorative-scale grid supports. Compared headlessly (`50`-seed survey, `500`-tick nominal-scenario run) across `48×32`, `64×48`, `80×60`, `96×64`, `128×80`, `160×100`: population dynamics stayed healthy at every size (0% total-extinction and 0% grid-saturation across all candidates, instability rate at 128×80 actually *lower* than the 48×32 baseline — 2/50 vs 4/50), and per-tick cost scales linearly but stays trivial even at the top end (~1.2ms/tick at 128×80, ~1.9ms/tick at 160×100, against a ~125ms tick budget at `era_tick_hz = 8.0`) — performance was never the binding constraint. `128×80` was chosen as the largest candidate confirmed live via `cargo run` to still read well at the current fixed-camera `ScalingMode::AutoMin` setup (peak glyphs, terrain bands, toxic-zone outline all legible); `160×100` was not visually verified and is left as a possible future step if the camera/legibility work below lands first. `min_placeable_fraction` and terrain classification thresholds needed no changes (continent/island noise frequencies are sampled in grid-normalized `[0,1]` coordinates, so the terrain *read* — number of continents, island density — is scale-invariant); `EnvironmentConfig::toxic_zone_width/height` (8×6 → 21×15) and `DifficultyConfig::toxic_zone_width_late/height_late` (16×12 → 43×30) were rescaled to preserve the same relative footprint, since a fixed absolute cell count would have shrunk to a sliver of the new grid. Three tests broke on grid-size-relative assumptions, not on the simulation itself: `sim::photolithic_in_the_dark_eventually_dies` (a forced-dark test cell was recovering to ambient light via diffusion over its 200-tick run, independent of the fix — grid size only changed *when* the marginal reproduction/crowding interplay tipped the outcome — fixed by disabling diffusion for that isolation test, matching its "pure energy-formula test" intent already documented on the shared helper); `objectives::diffused_toxicity_outside_the_zone_does_not_satisfy_survive_in` (a fixed `500`-tick diffusion budget assumed the `48×32` corner-to-corner distance; now runs diffusion until it actually leaks, capped generously relative to `width + height`); `tests/balance.rs::dark_rows_stay_uninhabited_across_seeds` (the light gradient's per-row step shrinks as height grows, so several rows now land within a hair of `LIGHT_SURVIVAL_THRESHOLD` by construction — added a `DARK_ROW_MARGIN` so only rows clearly below the threshold are asserted uninhabited). `cargo test` and `cargo clippy -- -D warnings` clean at `128×80`.
>
> **🔭 Follow-up raised during the visual check, not fixed here:** at `128×80`, individual organism dots read as small and some species colors (dark/muted hues against dark terrain, see the toxic-zone screenshot from this session) are hard to distinguish at a glance — a legibility problem this task's acceptance criteria explicitly scope out ("don't silently re-tune unrelated systems"). The user's own framing points past a simple zoom control toward a two-tier rendering idea: a zoomed-out view showing per-species population-density indicators (e.g. a glyph that scales with local population) rather than individual dots, with a zoom-in revealing actual per-organism granularity — mechanics for isolated/small-population experiments under that scheme still need to be worked out. Worked out into a proposal below.

- `[/]` Two-tier map view (Overview cluster-heatmap + Detail per-cell grid) — design closed 2026-08-09, full decision record in `redesign/abiogenesis-two-tier-view.md`. Split into task files 075-077, plus a same-day playtest correction (078) → [075](tasks/done/075-zoom-camera-overview-detail-switch.md) (done), [076](tasks/done/076-overview-cluster-heatmap-rendering.md) (done), [077](tasks/done/077-action-gating-by-view-mode.md) (done), [078](tasks/078-overview-heatmap-blob-shape-correction.md). Single continuous-zoom camera (mouse wheel, centered on cursor); hard threshold switch (not a blend) between Overview (per-species connected-component clusters rendered as density-heatmap blobs — not a fixed grid of blocks, so an isolated organism is its own visible one-cell cluster) and Detail (today's per-cell organism rendering, whatever's in the camera frustum — no separate windowed system, panning is just normal camera movement). Stress/Cull require Detail (need per-organism precision); Seed/Splice stay available in both, with a brief transient on-screen indicator (task-054 style) marking exactly where a Overview-mode placement landed. Task 076's first pass rendered blobs as a 1:1 recoloring of the real occupied-cell sprites, so a cluster's blob reproduced its exact footprint including internal gaps; task 078 corrects this to a smaller, uniformly-filled abstraction, per direct user feedback right after seeing 076 live.

> **📐 Task 075 (2026-08-09): zoom camera + `MapViewMode` landed**, `CameraConfig` (`zoom_min`/`max`/`threshold`/`speed`) added to `SimConfig`. Two real bugs surfaced during live playtesting on the user's machine, both fixed: pan drift left the grid off-center at the zoomed-out floor (a cursor-centered zoom formula alone doesn't guarantee translation returns to `(0,0)` at `zoom_max` — fixed with a general pan clamp derived from `AutoMin`'s own projection dimensions); and the terrain/toxic-zone overlay bled into the HUD sidebar once zoom let players view a sub-region (`Camera::world_to_viewport` doesn't clip to the camera's actual cropped viewport — fixed by clipping the overlay painters to `Camera::logical_viewport_rect()`). Full writeup in [075](tasks/done/075-zoom-camera-overview-detail-switch.md)'s Resolution section.

- `[x]` Onboarding: adaptive grace period + softened first-world objective — design session 2026-08-09 held right after task 077 closed, full decision record in `/Users/biagioliberto/.claude/plans/rosy-snuggling-lighthouse.md` → [079](tasks/done/079-onboarding-grace-period.md). A run gives no room to acclimate before real stakes kick in from tick 0 (a live playtest this session opened world 0 with "survive in the toxic zone" as its first objective; `player_guide.md`'s own "A note on balance" already names "the whole ecosystem dies before you can learn anything" as an expected tuning risk). Two changes: total-extinction failure is suppressed adaptively (not on a fixed timer alone, to avoid a cliff at expiry) until the player has kept a population alive for a full era at least once per world; and world 0's opening objective is forced to a gentle `Coexistence{min_species: 2}` instead of whatever the random draw picks (which could reach 3 and demand a Decomposer stay alive on a first try). Era-budget-exhaustion failure is deliberately left untouched (grace's magnitude is always far smaller than the budget, so gating it would be dead code). Verified live by the user on their own `cargo run`.

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

*Last updated: 2026-08-10 (088-089, user-reported self-interaction balance bug fix (worldgen + Splice paths), completed. Task 087, camera pan, completed and archived to `tasks/QUEUE_ARCHIVE.md`)*
