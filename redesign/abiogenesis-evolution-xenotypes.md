# Abiogenesis — evolution as speciation, and the road to real xenotypes

Standalone design doc, raised directly by the user (2026-08-11): the
matrix "aha" currently has no real payoff beyond knowledge itself — nothing
mechanical changes once a relationship is confirmed except the player's own
strategy. Two proposals, discussed together because they reinforce each
other: (1) an evolution mechanic that gives discovered interactions and
environmental exposure a tangible consequence, and (2) turning tags into
biochemically real "xenotypes" without leaking their mechanical effect.
Both connect to tensions already flagged — unresolved — in `VISION.md`
Phase C (Evolution) and Phase D (Biochemistry flavor); this doc is where
those get an actual resolution.

## Current state (grounding)

- **Two layers already exist in a species' genome**: an *anchor* layer
  (metabolism, `temp_optimum`/tolerance, repro threshold — GDD §7:
  "always remain readable") and a *mystery* layer (1-3 tags drawn from the
  world's active pool, driving the secret matrix). This distinction is the
  hinge the whole proposal below turns on.
- **`Splice` already does most of the plumbing this needs** —
  `SpliceEditChoice::{SwapTag, AddTag, ShiftTempOptimum}` (`input.rs:531-565`)
  is the player-driven action that creates a new `Species` by editing an
  existing one's genome (tags and/or temp optimum), naming it via
  `draw_species_name` (`world.rs:1114`, task 095's procedural per-world
  names). Evolution-by-speciation (below) is the same operation triggered
  by the simulation instead of the player.
- **GDD §5.4** already has an open, deferred item: "additional metabolisms
  beyond the three base ones, e.g. a chemolithotroph tied to `toxicity`" —
  directly relevant to both "add more metabolisms" (near-term want) and
  "metabolism change via evolution" (a chemolithotroph could be something a
  lineage *becomes* under toxicity pressure, not only something authored
  from world start).
- **`VISION.md` §C's flagged risk**: unprompted genome drift on a species
  the player has already tested "stales" their knowledge of it — hypotheses
  chase a moving target. **§D's flagged risk**: a trait named/described
  evocatively enough hints at its own matrix effect, undercutting §11's
  "you infer it, you're never told."

## Decided in discussion

- **Evolution never mutates an existing species in place — it always
  produces a new descendant species (speciation).** This is the load-bearing
  decision that resolves §C's risk: since the parent species and every
  species the player has already tested never change, nothing the player
  already knows can go stale. A descendant is a new, separate thing to
  learn about, exactly like a player-made `Splice` output — not a rewrite
  of something already understood.
- **Evolution *can* touch tags, not just the anchor layer** — initially
  scoped more cautiously (anchor-only), corrected in discussion: because
  speciation never touches the world's fixed matrix values, and a
  descendant only ever draws from tags **already active in that world**
  (the 5-8 drawn at world start, GDD §5.5), giving a descendant a new tag
  doesn't add a new axis of mystery — it exposes an *already-fixed*
  relationship that just hadn't been tested by that particular lineage
  yet. Net effect: more of the matrix becomes observable over a run, none
  of it becomes less knowable. The one guardrail: descendants must stay
  within the world's already-active tag set, never introduce a
  never-before-seen tag mid-run (that genuinely would add a new
  undiscovered axis, unlike reusing an active one).
- **Trigger model reuses the notebook's own pattern**: accumulate a
  "selection pressure" tally per organism/lineage from stimuli the sim
  already computes — sign/magnitude of `interaction_delta` experienced,
  ticks spent on a given `TerrainKind`, exposure to `toxicity` — and cross
  a threshold to fire a discrete speciation event, mirroring
  `MatrixKnowledge`'s evidence-accumulate-then-confirm shape rather than
  continuous per-offspring drift. Discrete events are also easier to
  telegraph to the player (a distinct log/visual moment, task 054's
  `★`-style pattern) than silent continuous drift would be.
- **Wishlist capabilities the descendant can gain** — better dispersal, a
  shifted temperature/light tolerance, tolerance for `Sea`/toxic-zone
  occupancy (bypassing today's `is_placeable` gating for that lineage
  specifically), a different metabolism, and/or an additional active tag.
  All either already exist as `Splice` edit choices or are natural
  extensions of the same genome-editing shape.
- **"More species": authored starting roster, not (only) emergent** — the
  user wants a bigger roster available to seed from turn zero,
  independent of evolution; evolution's speciation additionally grows the
  roster organically over a run, but that's additive, not a replacement
  for authoring more starting species.
- **"More metabolisms": near-term, concrete** — scoped ahead of the full
  xenotype/archetype redesign. First candidate: the chemolithotroph
  already named in GDD §5.4, tied to `toxicity` — a natural fit for both
  an authored starting metabolism *and* an evolution outcome (a lineage
  repeatedly exposed to toxicity evolving into one), so the two tracks
  (add metabolisms now, evolution later) can share design work instead of
  being built twice.
- **Full xenotype/archetype redesign (idea 2's naming overhaul):
  deferred**, explicitly "after the game stabilizes," not scoped now. The
  design principle to carry forward when it is picked up: **describe what
  a trait *is* (its biochemical nature), never how it interacts** — real
  microbiology already works this way (knowing a microbe is
  sulfate-reducing tells you its chemistry, not whether it helps or harms
  a specific neighbor; that's an ecological fact you still have to
  observe). `VISION.md` §D's litmus test ("could a first-time player guess
  the matrix effect from the name alone?") is the check to apply to any
  future naming pass. A possible refinement for later: the flavor/name
  reveal itself could be progressive (glyph-only until some discovery
  moment reveals the "true" biochemical identity) — a second, safe "aha"
  that never touches the mechanical mystery.

## Open questions / discussion points for next round

- **Exact stimulus → outcome mapping**: does a specific stimulus
  (repeated `interaction_delta` harm, toxicity exposure, time on `Hill`)
  bias toward a *specific* kind of capability (defensive resistance,
  dispersal, metabolism shift), or is the outcome drawn more loosely from
  whatever's eligible? The doc assumes the former is more satisfying
  (deliberate "expose your species to X to steer evolution toward Y") but
  it isn't designed yet.
- **How a fresh descendant is presented**: a new species appearing
  mid-run is a bigger moment than a normal birth — needs its own
  celebration distinct from task 054's confirmation `★` and the existing
  per-era birth summary, so it doesn't read as just another log line.
- **Resolved (2026-08-11)**: whether an evolved species survives a world
  reset — see `redesign/abiogenesis-progression-pacing.md`. Within-run
  only, via `RunProgress` (mirrors `MetaProgress::absorb`'s existing
  `worlds_cleared → bonus_available_species` shape), not the deferred
  cross-run persistence layer task 084 is blocked on.
- **Where/how the descendant enters the grid**: does the first qualifying
  organism itself "become" the founder of the new species (its cell
  reassigned), or does the descendant spawn adjacent/nearby once the
  threshold crosses, leaving the trigger organism as-is? Affects both
  implementation and how legible the moment is.
- **Balance/growth control**: uncapped speciation risks species-count
  bloat over a long run, the same class of risk `VISION.md` Phase B names
  for reproduction (tasks 047/048 already had to fight runaway growth from
  the *other* direction) — needs an explicit budget or cooldown, not an
  assumption that low base rates are enough on their own.
- **Overlap with "Mondo vivo" terrain tracking**: `abiogenesis-living-world.md`
  already needs to track an organism's terrain-occupancy history (for
  zone-entry discovery and conditional-tag triggers) — evolution's
  terrain-exposure stimulus needs the same kind of data. These should
  share one tracking mechanism, not build two parallel ones — flag this
  explicitly when either is scoped first.
- **Late-game density**: worlds with 8 active tags already sit near "too
  many to decode all" per GDD §5.5 — does speciation frequency/richness
  need to scale *down* as active-tag count goes up, so late worlds don't
  drown the player in new species on top of an already-dense matrix?
- **Sequencing the new-metabolism work**: should the first new metabolism
  (the toxicity-linked chemolithotroph) be designed and shipped
  independently of evolution as pure roster content, or held until
  evolution exists so it can debut as an evolution outcome from day one?
  Affects whether "add more metabolisms" is its own near-term task or
  waits on this doc's speciation mechanic.

## Scope note

Not scoped into task files yet. Likely splits into: (1) the
selection-pressure accumulation + threshold-crossing trigger (mirrors
`MatrixKnowledge`'s shape, touches `sim.rs`), (2) speciation itself
(reusing `Splice`'s genome-editing/naming plumbing, `input.rs`/`world.rs`),
(3) presentation of a new-species event (notebook/log/HUD, blocked on the
"how it's presented" open question above), (4) the chemolithotroph
metabolism as a separate, more contained near-term task that doesn't need
to wait on 1-3.
