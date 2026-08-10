# Abiogenesis — Vision & long-term roadmap

**This is exploratory design thinking, not a committed spec.** Unlike
[`abiogenesis-gdd.md`](abiogenesis-gdd.md) (the source of truth for what's
actually built) or [`PROJECT_PLAN.md`](PROJECT_PLAN.md) (the operational
backlog of approved/in-progress work), nothing here is scheduled. When an idea
from this document gets prioritized, it graduates the normal way: a
`PROJECT_PLAN.md` entry, a Meridian task file starting at `[?]` proposal, and
— if it changes a core mechanic — an update to the GDD itself.

The throughline across every phase below: the MVP proved the core loop works
(seed → observe → hypothesize → confirm), but it plays out too fast and too
compressed to feel like a *living* world. The next phases are about giving the
simulation room — spatially and temporally — to feel discovered rather than
consumed.

---

## Phase A — Ecosystem depth (space)

**Why**: the grid is currently `48 × 32` (1536 cells) with two linear
environmental gradients (light top→bottom, temperature left→right, GDD §5.2).
Any bloom visibly saturates a whole neighbourhood within a handful of ticks —
there's no real distance for a species to quietly stabilize in a niche before
the player notices. A much larger grid with patch-based, possibly randomized
environmental zones (already flagged as a future direction during the
temperature/light-overlay design discussion — task 058's rationale
specifically anticipated worldgen moving away from fixed-axis gradients)
would let real, spatially separated ecosystems exist side by side, each
discoverable at its own pace.

**Risk/tension**: a bigger grid multiplies per-tick cost (every cell's Moore
neighbourhood gets scanned every tick, `sim.rs`'s `step`) — needs a
performance check before committing to a size, not just a config bump.
Patch-based zones also complicate the "deduce heat/light from the map"
legibility problem solved for gradients (tasks 057/058) — a patchier
environment needs its own legibility pass, not a reuse of the gradient-era
solution unchanged.

---

## Phase B — Pacing & credibility (time)

**Why**: at default config, an isolated `Photolithic` organism nets about
`+1.3` energy/tick (`light 0.9 × gain 2.0 × env_fit 1.0 − upkeep 0.5`) and
reproduces roughly every 4 ticks once past the `10.0` threshold (`repro_cost
5.0`) — over one 25-tick era, an unchallenged lineage can reproduce 5-6 times.
There's no reproduction cooldown; the only throttle is re-accumulating
`repro_cost` worth of net energy. This is a known, actively-managed tension —
tasks 047/048 already fought the *opposite* failure mode (runaway
self-reinforcing growth saturating the grid) by tightening self-interaction
rules, never by slowing the base rate.

**Recommended sequencing** (not a decision made here, just the order that
avoids redoing work):
1. Land task 059 (sequential objectives) first — it already buys longer,
   more substantial worlds independently of the reproduction curve.
2. Do Phase A (grid size / real niches) before touching reproduction
   multipliers — a bigger arena changes the felt pacing on its own, and
   tuning multipliers first risks re-tuning again once the grid changes.
3. If reproduction still feels too eager after 1-2, prefer a **minimum-ticks-
   since-last-reproduction cooldown** over raising `repro_threshold` again —
   a raised threshold fights against `crowd_factor`/upkeep already doing that
   job and risks tipping back toward the "nothing survives" failure mode
   tasks 047/048 guarded against from the other direction. A cooldown is an
   orthogonal, more surgical lever.

**Risk/tension**: any reproduction change must be re-validated against
`tests/balance.rs`'s existing invariants (no total-extinction runaway, no
grid-saturation runaway) — those tests encode hard-won balance from 047/048
and are the right check before/after any retune, not a paper calculation.

---

## Phase C — Evolution

**Why**: the user wants spontaneous evolution — a child's genome drifting
slightly from its parent's (`temp_optimum`, tags) on reproduction, without
requiring the player's `Splice` action. This is a genuinely different feel
from today's fixed species roster: an ecosystem that visibly adapts over an
era, not just grows or shrinks.

**Risk/tension**: this is the phase most likely to conflict with an existing
pillar, not just add to it. GDD §7/§11's deduction loop assumes the player is
inferring a *fixed* hidden matrix and *fixed* species genomes — if species
drift unprompted, the player's hypotheses target a moving system. That could
enrich the "aha" loop (a confirmed relationship degrading as a lineage drifts
away from the tags that earned it) or could undermine it (evidence going
stale faster than it can be gathered), depending entirely on drift rate and
pacing. This needs its own dedicated design pass — including whether drift
should be visible/telegraphed at all — before it's scoped as a task, not an
assumption that it layers cleanly on top of the current notebook mechanic.

**Resolved direction (2026-08-11)**: `redesign/abiogenesis-evolution-xenotypes.md`
answers this risk directly — evolution never mutates an existing,
already-tested species in place; it always produces a new descendant
species (speciation), reusing `Splice`'s genome-editing plumbing and
triggered by an accumulated "selection pressure" threshold (mirroring the
notebook's own evidence-then-confirm shape). Since nothing the player has
already tested ever changes, the staleness risk above doesn't apply — this
supersedes the "drift" framing this phase originally assumed. See the doc
for the full model and open questions before scoping.

---

## Phase D — Biochemistry flavor

**Why**: tags are currently opaque glyphs with no flavor (`tag_glyph`,
`notebook.rs`) — functional but not evocative. The user wants tags to read
more like real biochemical traits (named, described, closer to plausible
biology) without becoming legible shortcuts to their mechanical effect.

**Risk/tension**: flavor and naming must not become a stealth difficulty
reduction. GDD §11's core promise is "you infer it, you're never told" — a
trait named and described evocatively enough to hint at its actual matrix
relationships would quietly undercut the deduction pillar the whole game is
built around. Any naming/flavor pass needs a explicit check: would a
first-time player be able to guess this trait's mechanical effect from its
name alone? If yes, the name is too on-the-nose.

**Direction agreed, execution deferred (2026-08-11)**:
`redesign/abiogenesis-evolution-xenotypes.md` sets the principle to apply
whenever this is picked up — describe what a trait *is* (its biochemical
nature/mechanism), never how it interacts with others; real microbiology
already keeps those two facts separate (knowing a microbe's metabolism
class doesn't tell you its net effect on a specific neighbor without
testing it). The user explicitly wants this held until after the game
stabilizes — nearer-term work adds more *metabolisms* (GDD §5.4's
chemolithotroph) without redesigning the tag/archetype pool itself.

---

*This document has no "last updated" convention like `PROJECT_PLAN.md`/
`tasks/QUEUE.md` — it's revised whenever the vision itself changes, not on
every session.*
