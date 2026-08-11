# Task 106 — Selection pressure accumulation + threshold-crossing trigger

> **ID**: `106`
> **Category**: Feature / Simulation
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-evolution-xenotypes.md`)

---

## 🎯 Objective

Give the hidden matrix's "aha" a mechanical payoff (the doc's opening
problem: nothing changes once a relationship is confirmed except the
player's own strategy). This task scopes **only** the first half of the
evolution proposal: a per-organism-or-lineage "selection pressure" tally
that accumulates from stimuli the tick already computes, and fires a
discrete event once it crosses a threshold — mirroring `MatrixKnowledge`'s
evidence-accumulate-then-confirm shape (`notebook.rs:137-182`) rather than
continuous per-offspring drift.

**What this task does NOT do**: decide what happens when the threshold
crosses. That's task 107 (speciation) — this task's job ends at emitting a
discrete signal event, the same way `OrganismDied` signals a death without
itself deciding what `notebook.rs` does with it.

The three stimuli named in the source doc:

1. Sign/magnitude of `interaction_delta` experienced this tick (already
   computed per-organism at `src/sim.rs:271-311`, step 3 of the tick).
2. Ticks spent occupying a given `TerrainKind` (`Cell.terrain`,
   `src/world.rs:160`).
3. Exposure to `toxicity` (`Cell.toxicity`, `src/world.rs:153`).

**Exact stimulus-to-outcome mapping is explicitly out of scope and
undesigned** — the source doc's own open question ("does a specific
stimulus bias toward a specific capability, or is the outcome drawn more
loosely?") is left for playtest. This task's accumulator must stay generic
enough to support whichever mapping task 107 eventually picks: track
*how much* pressure has built up and *from what stimuli*, not *which
outcome* it implies.

---

## 📋 Acceptance Criteria

- [ ] A new accumulator (new struct/resource, e.g. `SelectionPressure`) tracks
      per-organism-or-lineage tallies from the three stimuli above. Explicitly
      a **new, separate accumulator** — not a reuse or extension of
      `MatrixKnowledge` itself, which tracks tag-pair evidence, not
      organism/lineage stimulus exposure. Lives in an existing module
      (`sim.rs` is the natural home, since it consumes `step()`'s per-tick
      data directly) — do not introduce a new `Plugin`/module for this
      alone; per `CLAUDE.md`'s "one module = one Bevy `Plugin`" convention,
      a new `evolution.rs` would need its own `Plugin` registered in
      `main.rs`, more structure than a single accumulator warrants yet.
- [ ] Reads `interaction_delta`'s already-computed per-tick value
      (`src/sim.rs:271-311`) rather than recomputing it — no duplicate
      matrix-neighbour scan.
- [ ] Terrain-occupancy tracking: **see Dependencies/Constraints below**
      before building a new terrain-history structure — task 099
      (`abiogenesis-living-world.md`) needs the same data and the two must
      share one mechanism.
- [ ] Config-driven threshold (`SimConfig`, no magic numbers per
      `TECH_DESIGN.md` §5) — a first-pass, tunable value, documented as such
      in a doc comment.
- [ ] A new discrete Bevy `Message` type (e.g. `SelectionThresholdCrossed`),
      mirroring `OrganismDied`'s shape (`src/sim.rs:16-31`): carries enough
      identifying/diagnostic data (organism/species/cell, and which
      stimulus(es) contributed) for a future consumer (task 107) to act on
      without re-deriving it.
- [ ] The event fires at most once per organism/lineage per crossing (mirrors
      `MatrixKnowledge::record`'s `was_confirmed` guard, `notebook.rs:156-161`)
      — no repeat-firing every tick once already above threshold.
- [ ] Unit tests (inline in the accumulator's module, following
      `src/sim.rs`'s `#[cfg(test)]` pattern, e.g. `world_with_one_predator`-style
      fixture helpers) covering: pressure accumulates from each of the three
      stimuli independently; the event fires exactly once at the crossing
      tick; repeat exposure past threshold does not re-fire.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: trigger a crossing (e.g. seed a species
      into the toxic zone or a strongly negative matrix pairing) and confirm
      the event fires (a log line or debug print is sufficient at this
      stage — presentation is task 107's concern, not this one's).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `interaction_delta` computation (271-311) to read from; `OrganismDied` (16-31) as the event-shape precedent; `TickEvents` (94-100) as the drain pattern. |
| `src/notebook.rs` | `MatrixKnowledge` (137-182) and `accumulate_evidence` (233-) — the accumulate-then-confirm shape this task mirrors. |
| `src/world.rs` | `Cell.terrain: TerrainKind` (160), `Cell.toxicity: f32` (153) — the other two stimuli. |
| `src/config.rs` | `EnergyConfig` (255-) as the pattern for adding a new config section/threshold field — add near here or a new sub-config. |
| `redesign/abiogenesis-living-world.md` | Task 099's terrain-occupancy tracking need — read before building a terrain-history structure here. |

---

## 🧩 Technical Context

- **Current behavior**: nothing tracks cumulative stimulus exposure per
  organism or lineage. `interaction_delta` is computed fresh every tick and
  discarded once applied to energy (`src/sim.rs:328`); terrain and toxicity
  exposure are read per-tick for other purposes (env fitness, zone checks)
  but never accumulated over an organism's lifetime.
- **Desired behavior**: a running tally per organism-or-lineage (design
  choice: per-`Organism` instance resets on death, so it likely needs to
  live keyed by cell index or be carried in `Organism` itself if it should
  survive reproduction as a lineage-level stat — decide and document which,
  since the doc talks about both "organism" and "lineage" loosely).
- **Design choice to make and document**: does pressure reset when an
  organism dies (individual-scoped) or persist/inherit across reproduction
  (lineage-scoped, closer to the doc's "lineage" framing and to how
  speciation — task 107 — would actually read as a lineage adapting over
  several generations rather than one individual's luck)? The doc leans
  toward lineage-level ("a lineage repeatedly exposed to X"), which likely
  means the tally needs to be carried on `Organism` (or a per-`SpeciesId`
  aggregate) and passed to children on reproduction, not reset per-instance.
  Pick lineage-scoped unless there's a concrete blocker — flag the decision
  in a doc comment either way.
- **Representation constraint this decision runs into**: `Cell` and
  `Organism` both derive `Copy` (`src/world.rs:99-107`, `149-166`), and
  `step()` relies on that — `let cell = world.cells[idx];`
  (`src/sim.rs:247`) copies the whole cell out of the snapshot every
  iteration. Any lineage-scoped tally stored on `Organism` must therefore
  stay `Copy` itself (fixed-size fields only, e.g. a small `[f32; N]`
  per-stimulus tally or a few plain scalars — `TerrainKind` has exactly 4
  variants, so a fixed-size array indexed by variant fits) — no `Vec`/`HashMap`
  field, which would break `Organism`'s `Copy` derive and the snapshot-and-swap
  tick pattern. If per-organism `Copy` data isn't enough, use a side table
  (e.g. `Vec<f32>` indexed by cell, rebuilt/carried across ticks like
  `world.scratch`) instead of touching `Organism`'s shape.

---

## 🔨 Suggested Implementation

1. Decide and document the individual-vs-lineage scoping question above.
2. Add the new accumulator type, threshold field(s) in `SimConfig`, and the
   new `Message` type, following `OrganismDied`'s shape.
3. Wire tally accumulation into `step()` (`src/sim.rs`) alongside the
   existing per-organism loop (step 1-6), reading `interaction_delta`,
   `cell.terrain`, `cell.toxicity` from data already in scope — no new scan.
4. Before implementing terrain-occupancy tracking, check whether task 099 has
   already landed a shared structure; if not, design it with 099's stated
   need in mind (comment the shared-mechanism intent either way) rather than
   building a 106-only tracker.
5. Emit the new `Message` via `TickEvents` (mirroring `deaths`/`births`) and
   drain it the same way `advance_tick` drains `OrganismDied` (`src/sim.rs`
   around line 528).
6. Unit tests per the acceptance criteria.
7. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- Do not implement task 107's speciation logic here — this task ends at the
  signal. A consumer system for the new message is not required by this
  task; a test-only reader plus a log line for live verification is enough.
- **Shared terrain-tracking note**: whichever of {099, 106} is built first
  must design its terrain-occupancy-history structure with the other in
  mind — do not let this task build an isolated terrain-history tracker
  that 099 later has to duplicate or replace. If 099 lands first, depend on
  its structure directly instead of adding a second one.
- No magic numbers: threshold(s) and any per-stimulus weight constants
  belong in `SimConfig`.
- Keep the tick deterministic — no new RNG draws, no `HashMap` iteration
  (`TECH_DESIGN.md` §5).

---

## 🔗 Dependencies

- **Depends on**: 018 (`AdjacencyObserved`/`interaction_delta` origin), 066
  (`TerrainKind`), 072 (toxic zone / `toxicity`).
- **Related, share tracking with**: task 099 (`abiogenesis-living-world.md`,
  zone-entry reveal) — both need organism/lineage terrain-occupancy history;
  design once, use twice. See Constraints above.
- **Blocks**: 107 (speciation) — consumes this task's threshold-crossing
  event.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/106-evolution-selection-pressure-trigger.md)"$'\n\nExecute this task in the current project.'
```
