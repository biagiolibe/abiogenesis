# Abiogenesis — notebook UX redesign

Standalone design doc, raised directly by the user (2026-08-10) while
discussing "Mondo vivo" (`abiogenesis-living-world.md`): that proposal's
conditional-tags mechanic needs *some* way to surface "this relationship
depends on terrain" in the notebook, which surfaced a pre-existing problem —
the notebook's current UX doesn't hold up on its own, independent of any new
mechanic. This doc is a **separate, independent track**: it should be scoped
into its own task(s), not bundled with "Mondo vivo." The two intersect only
at one point — the conditional-tags catalog badge (see that doc's §1) — and
should stay two docs, not merge.

## Current state (code + live screenshot review)

Code: `src/notebook.rs`. Three stacked sections in one `egui::Window`
(`notebook_window`, :473): **Observation log** (:488) → **Hypothesis grid**
(:519/:574, a circular node-graph, not force-directed — replaced a
spreadsheet table in task 031) → **Catalog** (:809, tag pool + species
list).

Current edge/node grammar: confirmed relation = solid arrow, red/green by
sign (`EDGE_POSITIVE_COLOR`/`EDGE_NEGATIVE_COLOR`, :542-543), magnitude-2
edges get a thicker stroke **plus** a `±N` text label (:659-685); partial
evidence (evidence > 0, below the 3.0 confirmation threshold) = a small dim
gray dot at 65% along the edge, deliberately not line-shaped
(`draw_partial_marker`, :729); zero-evidence tags get a dashed ring around
an otherwise-empty node (`draw_dashed_ring`, :712). `MatrixKnowledge`
(:137-182) accumulates evidence per `(exerter, receiver)` tag pair; the
confidence float itself is never shown as a number, only collapsed to
binary states (confirmed / partial / none) today.

A death-cause breakdown already exists for player-placed organisms
(`text::player_organism_death_message`, `text.rs:389-408`, "answers 'why'
instead of only 'what'" per its own doc comment) — logs raw energy terms
(`gain`, `interaction_delta`, `upkeep`, `crowding`, `predation`) as signed
numbers. Relevant context for the companion "death/failure legibility"
discussion (see bottom of this doc), not this doc's main subject.

**Live screenshot review** (fresh `cargo run`, real playthrough), confirmed
concretely, not just in code:
- **Log**: the same tag pair logged as a separate line **per tick**
  ("Era 3: ζ→ε observed" repeated 6+ times back to back) — raw telemetry,
  not the "curated feed of what mattered" `player_guide.md` promises.
- **Grid**: 3 of 5 active tags sat inert with dashed rings, never
  interacted, pure clutter; the one real relationship got no layout
  benefit from being the only meaningful pair — static circular placement,
  arrows overlapping, `-2`/`+2` labels cramped next to each other, hard to
  read direction at a glance.
- **Catalog**: near-identical descriptive sentences repeated per species
  sharing a metabolism ("A cold-adapted species that draws its energy from
  light...") — text noise, no new information per row.
- General: flat, high-contrast black background, no spacing/typography
  hierarchy between the three sections, large areas of unused space.

## Decided in discussion

- **Log**: strip raw per-tick "observed" noise entirely. The log keeps only
  narrative events — species births/deaths/extinctions, confirmation `★`
  moments (task 054's pattern). Evidence accumulation moves to the grid
  itself (below) instead of spamming text — no information lost, just
  relocated to where it's contextual.
- **Grid — visibility**: zero-evidence tags don't render at all (no ghost
  dashed ring) until the first tick that actually involves them. Shrinks
  the visible node set to only what's actually been touched.
- **Grid — layout**: kept simple/static, **not** a true force-directed
  physics simulation — recompute an even circular/clustered placement over
  just the *currently visible* subset each time a new tag is revealed
  (fewer nodes = automatically less crowded), instead of simulating
  attraction/repulsion every frame. This was a deliberate complexity/ROI
  call, not a legibility objection to physics-based layout in principle —
  worth revisiting later if the simple version still feels static.
- **Grid — edge grammar, rethought**: no more numeric `±N` labels — encode
  magnitude as **line thickness** only. No more dot markers for partial
  evidence — encode "evidence exists, nature still unknown" as a **dashed
  line** instead (semantically apt too: partial evidence never reveals
  real sign/magnitude pre-confirmation, so a neutral dashed line fits
  better than a marker that implies a value). Confirmed relationships stay
  solid lines, colored by sign, thickness by magnitude.
- **Bidirectional pairs**: today's straight overlapping double-arrow (both
  A→B and B→A present, as in the ζ/ε screenshot) needs to become two
  offset/curved arcs bowing apart instead of overlapping — direct fix for
  the "-2/+2 crammed together" readability problem observed live.
- **Overall visual polish**: explicitly wanted, not just the structural/data
  fixes above — palette, background treatment, spacing, and typographic
  hierarchy across all three sections need their own pass. Not yet
  specified in detail (see open questions).
- **Catalog**: raised as an uncontroversial companion fix, not yet formally
  confirmed by the user — deduplicate the per-metabolism boilerplate
  sentence into a one-time legend (icon + one line, shown once); trim each
  species row to just its specific parameters (name, metabolism icon, temp
  range, repro threshold, tags).

## Proposed direction

1. **Log** — event-only feed: births/deaths/extinctions/confirmations.
   Raw per-observation evidence moves entirely off the log.
2. **Grid — reveal-on-first-observation** — a tag is invisible until its
   first tick of real involvement; consider a brief pop-in/fade-in
   animation on reveal, reusing the existing "interaction spark" feedback
   family (tasks 054/080) instead of inventing a new discovery-feedback
   mechanism.
3. **Grid — edge redesign**:
   - Confirmed edge: solid line, color = sign, **thickness = magnitude**,
     no text label.
   - Partial edge: dashed line, neutral color (sign/magnitude genuinely not
     known yet), uniform thickness (nothing to encode).
   - Bidirectional pairs: offset/curved arcs instead of overlapping
     straight lines — a geometry change to the current edge-drawing code,
     not a new concept.
4. **Visual polish pass** — palette, background, spacing, typography across
   all three panels. Needs a concrete mockup/pass once the structural
   changes above are agreed; not specified further here.
5. **Catalog** — metabolism legend shown once; per-species rows trimmed to
   parameters only. Pending explicit user confirmation before treating as
   settled.

## Open questions for task scoping

- Exact visual language for the conditional-tags catalog badge (inducible
  vs. repressible, per `abiogenesis-living-world.md` §1) — needs to read
  clearly at a glance without competing with the metabolism icon already
  in that row.
- Concrete palette/spacing direction for the visual polish pass (item 4) —
  needs its own mini design pass, possibly with reference mockups, before
  it's scoped-ready.
- Whether the simple non-physics layout (decided above) ages well once
  grids regularly have 4-6 visible tags with several edges each, or if a
  future revisit toward true force-directed layout becomes worth its cost
  — flagged for later, not blocking this round.
- Companion topic spun out into its own doc: broader "why did that
  organism die" legibility — see `redesign/abiogenesis-death-legibility.md`
  (2026-08-11). Touches `sim.rs`/`text.rs`/the HUD's Biosphere panel, not
  this doc's notebook-window surfaces, so it stays separate rather than
  folding in here.

## Scope note

Not scoped into task files yet. The structural changes (log, grid
visibility/layout/edges) are one likely task; the visual polish pass
(palette/spacing/typography) is likely a separate follow-up task once its
own direction is nailed down, mirroring how other multi-part redesigns in
this repo split data/mechanics from presentation (e.g. terrain redesign
066→072, engagement design 080→086).
