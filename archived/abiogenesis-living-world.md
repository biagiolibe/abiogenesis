# Abiogenesis — a world that feels alive: conditional traits, hidden species, transient sources

Standalone design doc, raised directly by the user (2026-08-10) as a
follow-up to the onboarding/engagement work (`abiogenesis-engagement-design.md`,
tasks 080-086, all landed): the "wow" of the opening turns is still incomplete.
Placing a species should feel like discovering *where it can live and evolve*,
not just picking tags and watching numbers move. Three mechanics, discussed
and scoped down to a single underlying pattern in that session; real-time
mode was raised in the same discussion and explicitly deferred (stays a `[?]`
in `PROJECT_PLAN.md` §1 — revisit after these land and are playtested, to
avoid stacking two sources of reduced legibility at once).

## Current model (for reference)

- `interaction_delta` (`src/sim.rs:265-311`) reads **only** species tags:
  for every occupied Moore neighbor, for every `(their_tag, my_tag)` pair, it
  sums `world.matrix.get(their_tag, my_tag)`. No environment or terrain input
  anywhere in the formula. GDD §5.6 explains this is a *deliberate*
  simplification: "multiplicative would be more realistic but couples the
  effects and makes deduction nearly impossible; additive is readable."
- `TagMatrix` (`src/world.rs:55`) is rolled once per world — a secret,
  asymmetric `tag × tag` table, ~40% density, with a cyclicity constraint
  (GDD §5.8) — and never varies within a run.
- `TerrainKind` (`src/world.rs:140`, `Sea`/`Plain`/`Hill`/`Mountain`) today
  only gates placement (`is_placeable`, `world.rs:748-758`) and feeds the
  temperature/light scalars (source-driven model, task 085/086, already
  landed — `world.rs:413-429` and around) that `env_fit`/`Photolithic` gain
  read. It never touches `TagMatrix` or `interaction_delta`.
- The closest existing precedent for what this doc proposes is already
  decided for the **Precursor** proposal (`PROJECT_PLAN.md` §1, "2.2"): *"a
  phantom organism/tag that participates in `interaction_delta` like any
  other, not a separate hard-coded effect — consistent with the project's
  no-parallel-systems convention."* Everything below follows the same
  principle: reuse the tag/matrix machinery, don't invent a parallel one.

## Why change it

GDD §11 (deduction pillar) and the numeric baseline (§5.9, "~20 relationships
decodable at 5 tags in one run") are the binding constraint on every idea
here: the matrix is confirmable in a single run *because* it has exactly one
axis of variation (tag pair) today. Any new source of variation multiplies
the hypothesis space the notebook's confounder-weighted evidence model
(§7, `weight = 1/(1+n_confounders)`) has to disentangle — that model has no
terrain axis at all right now. So the goal isn't "add more variables," it's
"add exactly one new, readable axis, reusing existing systems, at low
density" — the same discipline task 085/086 already applied to environment
sources and 011 applies to the base matrix's cyclicity constraint.

Biochemically, the chosen mechanism (see below) has real grounding:
conditional gene expression / operon regulation (e.g. the *lac* operon: the
gene exists in the genome but is only transcribed under specific
environmental conditions). This fits GDD pillar 3 ("depth via simple rules,
not simulated realism") — it's a scalar gate, not a chemistry simulation —
while still being a genuine biochemical parallel rather than an arbitrary
game mechanic. §5.4 is the GDD's only other gesture toward this kind of
realism (a deferred chemolithotroph metabolism tied to `toxicity`) — this
proposal stays in that same spirit: simple, gated, not deeply simulated.

## Decided in discussion

- **Zone↔matrix mechanism: conditional tags**, not a per-terrain intensity
  multiplier and not a separate matrix per biome. The matrix stays a single,
  world-global table (`TagMatrix` unchanged); what changes is *which tags are
  currently active* for `interaction_delta`'s lookup, based on the organism's
  current cell. Rejected alternatives and why:
  - *Per-terrain intensity multiplier*: simpler, but reads as "the same
    relationship, stronger or weaker" rather than "a new trait revealed" —
    weaker fit for the "discover where it can live" goal.
  - *Separate matrix per biome*: most "depth," but multiplies the notebook's
    hypothesis space per tag pair from one scalar to one-per-terrain-type,
    directly fighting §5.9's baseline and §7's confounder model, which has no
    terrain dimension to weight against. Would need a notebook/evidence
    redesign first — out of scope here.
- **Diffusion discoveries: both**, as two distinct, additive mechanics (2a/2b
  below), not one merged into the other.
- **Real-time mode: deferred**, stays a `[?]` proposal in `PROJECT_PLAN.md`
  §1, not scoped alongside this doc.
- **Which glyphs are conditional: fixed at the pool level**, not re-rolled
  per world. A small, fixed subset of the 10-glyph pool is always
  terrain-linked, in every world — the same kind of structural fact as
  "the pool has 10 glyphs, the matrix is asymmetric" (things the player
  learns from the manual, not from decoding a specific world). **Which
  terrain triggers each conditional glyph is what's re-rolled per world**
  and stays the actual mystery. This keeps the added hypothesis space to
  exactly one axis per conditional tag (not one axis × every world), in
  line with §5.9's baseline concern.
- **Terrain relationship can be positive or negative — modeled as
  inducible vs. repressible**, not a continuous signed modifier (rejected
  the "intensity multiplier" alternative above for the same reason: it
  reads as "same relationship, stronger/weaker," not "a trait revealed").
  Real biochemical parallel: operons can be *inducible* (silent by
  default, switched on by a condition — e.g. the *lac* operon) or
  *repressible* (active by default, switched off by a condition — e.g. the
  *trp* operon). Applying that directly answers "can terrain help or hurt
  a tag": an inducible conditional tag is inactive everywhere except its
  trigger terrain; a repressible one is active everywhere *except* its
  trigger terrain. Both are still a single boolean gate per cell — no new
  runtime complexity over the original proposal, just two default states
  instead of one. Whether a given conditional glyph is inducible or
  repressible is rolled per world, same roll as its trigger terrain (one
  discovery process either way: the player notices the tag's presence is
  inconsistent across zones and experiments to find the boundary and its
  direction).

## Proposed model

### 1. Conditional tags (core mechanic)

- A small, **fixed** subset of the 10-glyph pool (structural, same in every
  world — see "Decided" above) carries a terrain condition:
  `(TerrainKind, Mode)` where `Mode` is `Inducible` (inactive by default,
  active only on the trigger terrain) or `Repressible` (active by default,
  inactive only on the trigger terrain). Which terrain and which mode apply
  to each conditional glyph is rolled fresh per world, same RNG moment as
  today's tag-pool/matrix generation (GDD §5.5, `src/world.rs`).
- In `interaction_delta`, a conditional tag counts toward the lookup only
  if the organism carrying it currently occupies a cell whose `TerrainKind`
  matches its trigger, respecting `Mode` (inducible: match required;
  repressible: match excludes). Unconditional tags (`None`) behave exactly
  as today, active everywhere.
- **Density**: only a small minority of the pool should be conditional (a
  first guess: 1-2 out of 10, tunable in `SimConfig`, no magic numbers per
  the project convention) — the majority stay unconditional so the
  existing §5.9 baseline (relationships decodable per run) isn't diluted.
- **Discovery model, simpler than first assumed**: because conditionality
  is a fixed, small set of *specific tags* (not a property re-derived per
  tag pair), the notebook doesn't need to split its existing per-pair
  `MatrixKnowledge` estimates by terrain at all. What's actually unknown
  per world is much smaller: "does conditional glyph G require/exclude
  which terrain, and in which direction" — a per-*tag* fact, not a
  per-*relationship* fact. This can reuse the exact same evidence-weighting
  machinery (`accumulate_evidence`, `weight = 1/(1+n_confounders)`) keyed
  on `(tag, terrain)` instead of `(tagA, tagB)` — a parallel, much smaller
  evidence track, not a rework of the existing one.
- **Where it surfaces in the UI**: spun out into its own discussion —
  `redesign/abiogenesis-notebook-redesign.md` covers the current
  notebook's legibility problems more broadly. Direction agreed so far:
  once confirmed, a conditional tag's terrain requirement is a **catalog**
  fact (an "anchor," per GDD §7's own language for metabolism/temperature),
  not something drawn into the hypothesis grid's edge/node grammar — shown
  as a small terrain-glyph badge next to the tag, distinguishing inducible
  ("turns on here") from repressible ("turns off here") with a distinct
  badge treatment (exact glyph/iconography TBD in the notebook redesign
  doc, not decided here).

### 2a. Wild, pre-existing species

- At world generation, in addition to the species the player seeds, the
  world can contain a small number of "wild" populations — same `Species`
  representation, own tags, drawn from the same pool/matrix machinery, no
  parallel system — placed in zones not immediately visible or reachable
  from the player's likely starting area.
- **Discovery trigger**: first contact, i.e. the moment a player-seeded
  species' population reaches interaction range of a wild population. Can
  reuse the existing "first-seen relation" spark trigger (task 080,
  interaction spark) for immediate visual feedback, and the
  unconfirmed→confirmed notebook event (task 054) for the log entry.
- Open question: how many wild populations per world, and does placement
  need to guarantee reachability within a normal run's spread radius, or is
  "may never be found" an acceptable outcome for some worlds (arguably
  reinforces "mystery," per GDD's stated design values)?

### 2b. Reveal-on-first-zone-entry

- Same underlying mechanism as (1): when a player-seeded species occupies a
  `TerrainKind` it has never occupied before in that run, and it carries a
  tag conditioned on that terrain, the notebook logs the event the same way
  as an unconfirmed→confirmed transition (task 054's pattern) — the zone
  entry itself is the discovery trigger, no separate system needed.

### 3. Transient world features (e.g. low-decay residue)

- Treated as a lighter-weight instance of the same pattern already decided
  for the **Precursor** (2.2): a "phantom" entity with its own tags that
  participates in `interaction_delta` exactly like a real organism, not a
  hardcoded effect. A residue spawned in a zone is a small, temporary tag
  source; nearby species interact with it through the existing matrix.
- **Decay**: the source's effective tag presence fades over time (e.g. a
  removal probability that increases with age, or a depleting internal
  "charge") — gives species a limited window to discover the relation
  before it's gone, matching the "discover it while it's happening" framing
  the user wants.
- **Explicit link to 2.2**: when either is scoped into a task, the two
  should share their underlying "phantom tag entity" implementation —
  fixed-and-singular for the Precursor, procedural-and-recurring for
  residues — rather than being built as two separate systems.

## Open questions for task scoping

- How the notebook/observation log surfaces terrain-conditioning without
  overloading the UI (task 057/058's species/notebook legibility work is the
  relevant precedent to build on, not replace).
- Exact density/count tuning for conditional tags, wild species count, and
  residue spawn frequency — first-pass numbers belong in `SimConfig`,
  validated by playtest per the project's usual balance-tuning approach, not
  chosen on paper here.
- Whether wild-species placement needs a reachability guarantee within a
  normal run, or "may go undiscovered" is an acceptable, even desirable,
  outcome for some worlds.
- Shared implementation shape for "phantom tag entity" (Precursor + transient
  residues) — needs a short design pass before either is scoped, so the
  second one doesn't get built as a bolt-on to the first.
- Interaction with 084 (guaranteed "first light" relation in world 0, itself
  blocked on meta-progression persistence) — once conditional tags exist,
  084's guarantee may need to specify *which* zone the guaranteed relation is
  reachable in, not just that it exists.

## Scope note

Not scoped into task files yet. Expect at least three dependency-ordered
tasks once approved: (1) conditional-tag data model + `interaction_delta`
change + notebook terrain-conditioning surface, (2) wild species generation
and first-contact discovery, (3) transient residue sources (shared shape with
Precursor, 2.2, scoped together or immediately after). Real-time mode
(`PROJECT_PLAN.md` §1) stays independently deferred and should be revisited
only after these three are playtested, not scoped alongside them.
