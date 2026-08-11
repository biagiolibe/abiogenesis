# Task 107 — Evolution by speciation: a new descendant species from selection pressure

> **ID**: `107`
> **Category**: Feature / Simulation
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-evolution-xenotypes.md`)

---

## 🎯 Objective

On receiving task 106's threshold-crossing signal, create a new descendant
`Species` — reusing `Splice`'s existing genome-editing plumbing
(`apply_splice`, `src/input.rs:505-578`) and `draw_species_name`
(`src/world.rs:1114`, task 095) — triggered by the simulation instead of a
player action.

**Load-bearing decision from the source doc, do not deviate**: evolution
**never mutates an existing species in place** — it always produces a new
descendant species (speciation). This is what keeps a player's already-tested
knowledge from going stale (`VISION.md` §C's flagged risk): the parent
species and every species already tested never change; a descendant is a
new, separate thing to learn about, exactly like a player-made `Splice`
output.

**Tag guardrail, also load-bearing**: a descendant may gain an additional
tag, but only ever drawn from `world.active_tags` (`src/world.rs:187`) —
tags already active in that world. Never introduce a tag that wasn't
already part of the world's fixed active set; doing so would add a genuinely
new undiscovered axis, which is explicitly out of scope (the doc's own
guardrail).

---

## 📋 Acceptance Criteria

- [ ] A new system reads task 106's `SelectionThresholdCrossed` message (or
      equivalent) and, for the qualifying organism/lineage, creates a new
      `Species` appended to `world.species` — following `apply_splice`'s
      exact shape (`src/input.rs:526-578`): clone the source species, draw
      an independent name via `draw_species_name`, apply an edit, push to
      `world.species`, allocate its `SpeciesId` as
      `SpeciesId(world.species.len() as u8 - 1)`.
- [ ] The source species referenced by the triggering organism is never
      mutated — only a new entry is appended.
- [ ] The descendant's edit is drawn from the doc's wishlist, reusing
      existing mechanics where they already exist as `Splice` edit choices:
      - shifted `temp_optimum`/tolerance (existing `ShiftTempOptimum` shape),
      - metabolism change,
      - terrain/toxicity tolerance for that lineage specifically — bypassing
        `is_placeable` gating (`src/world.rs:748-758`) for organisms of this
        species only. Needs a small new flag (e.g. on `Species` or
        `Organism`) since `is_placeable`/`is_placeable_kind` today has no
        per-species awareness at all.
      - better dispersal,
      - an additional tag drawn only from `world.active_tags` — never a tag
        outside that world's already-active set (see Objective's guardrail).
      Which specific edit(s) a given crossing applies is a first-pass
      choice (task 106 explicitly leaves the stimulus-to-outcome mapping
      undesigned) — document whatever selection rule is implemented as a
      first pass, tunable later.
- [ ] `net_self_interaction` is checked the same way `apply_splice` does
      (`src/input.rs:539/552`) before committing a tag-changing edit — a
      speciation event must not produce a self-reinforcing/self-draining
      tag set any more than a player `Splice` can.
- [ ] **Hard cap on descendant creation, not just a caveat**:
      `SpeciesId` wraps a `u8` (`src/world.rs:44`, doc comment: "Kept small:
      species are few and never removed") and `apply_splice` allocates it as
      `SpeciesId(world.species.len() as u8 - 1)` (`src/input.rs:571`) — a
      simulation-driven creator has no `ActionBudget` gate the way player
      `Splice` does, so nothing else stops `world.species.len()` from
      exceeding `256` over a long run, at which point the `as u8` cast
      silently wraps/aliases (a new species could allocate `SpeciesId(0)`,
      colliding with an existing one — every `species.0 as usize` index and
      the `population = vec![0u32; world.species.len()]` bookkeeping in
      `step()` would then be wrong). Make `world.species.len() >
      u8::MAX as usize` unreachable by construction: a config-driven cap
      (e.g. `max_evolved_species` in `SimConfig`) that this task's creation
      path checks and no-ops past, not merely documents as a future balance
      concern.
- [ ] The event is logged (`ObservationLog`, mirroring
      `text::species_created_message`, `src/input.rs:572-577`) — a plain log
      line or reuse of the existing birth-summary line is an acceptable
      placeholder. **Explicitly out of scope**: any new celebratory
      presentation/UI for a fresh descendant — the source doc leaves "how a
      fresh descendant is presented" as an open, undesigned question; do not
      build new UI for it here.
- [ ] Unit tests (inline, calling the pure `speciate`-style function directly
      rather than driving a Bevy `App`, following existing `#[cfg(test)]`
      fixture patterns) covering: a
      qualifying crossing produces exactly one new species; the parent
      species is unchanged; the descendant's tags are a subset of
      `world.active_tags`; a resulting `net_self_interaction != 0` edit is
      rejected/no-ops rather than partially applied (mirrors
      `apply_splice`'s existing behavior).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: trigger a qualifying crossing and
      confirm a new species appears in `world.species` with a distinct name
      and a logged entry, without the parent species changing.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `apply_splice` (505-578) — the exact plumbing to mirror: clone, edit, name, push, `SpeciesId` allocation, `net_self_interaction` guard. `SpliceEditChoice` (531-565). |
| `src/world.rs` | `draw_species_name` (1114), `active_tags: Vec<TagId>` (187), `is_placeable`/`is_placeable_kind` (748-758, 942), `Species` (81-96), `Organism` (100-107). |
| `src/sim.rs` | Consumes task 106's threshold-crossing message; `Metabolism` enum (19-23) if metabolism-change is the chosen edit. |
| `src/text.rs` | `species_created_message` (380) — precedent for the log entry text. |

---

## 🧩 Technical Context

- **Current behavior**: the only way a new `Species` is created mid-run is
  the player-driven `Splice` action (`apply_splice`). Nothing in the
  simulation itself creates species.
- **Desired behavior**: task 106's threshold-crossing event additionally
  triggers the same kind of creation, programmatically, on the simulation's
  own schedule.
- **`is_placeable` gating today has no species awareness** (`src/world.rs:748`,
  `is_placeable_kind`, `942`): it's a pure terrain/peak check, independent of
  which species is asking. Granting one lineage toxic/Sea tolerance needs a
  new per-species (or per-organism) flag threaded into that check — a real,
  if small, new piece of state, not just a `Splice`-style edit like the
  others.

---

## 🔨 Suggested Implementation

1. Implement the descendant-creation path as a pure function (no Bevy `App`
   dependency), mirroring `apply_splice`'s clone / edit / name / push /
   `SpeciesId` sequence, including the hard `u8`-cap check.
2. Decide and implement the first-pass edit-selection rule (which wishlist
   capability a given crossing grants) — document it as tunable.
3. Add the new placement-gating flag for toxic/Sea tolerance, threaded
   through `is_placeable`/`is_placeable_kind` for organisms of the
   descendant species only.
4. Add the thin Bevy system that reads task 106's drained
   `SelectionThresholdCrossed` message and calls the pure function, wiring
   the log entry.
5. Unit tests per acceptance criteria, calling the pure function directly.
6. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- Never mutate the triggering organism's source species in place — append
  only. This is the doc's load-bearing decision; do not "simplify" by
  editing in place.
- Descendant tags must come only from `world.active_tags` — never introduce
  a new tag mid-world.
- Do not build new UI/presentation for the "new species" moment — a plain
  log line (or reuse of the existing birth-summary line) is the acceptance
  bar. The doc's "how a fresh descendant is presented" open question is
  deliberately not designed yet and out of scope for this task.
- No auto-placement of the descendant onto the grid beyond what's needed to
  make it exist as a species — where/how it enters the grid (the triggering
  organism "becomes" the founder, vs. spawning nearby) is also an open
  question in the source doc; pick the simpler of the two (the triggering
  organism's cell is reassigned to the new species — no new placement logic
  needed) and document the choice rather than leaving it ambiguous.
- Balance: beyond the hard `u8` cap above (a correctness requirement, not
  optional tuning), the source doc separately flags uncapped speciation
  *frequency* as a species-count-bloat risk (same class of problem tasks
  047/048 fought for reproduction) — a cooldown or rate limit beyond the raw
  count cap is a reasonable first-pass addition but not required to close
  this task if the count cap and task 106's threshold together already keep
  growth slow enough to verify live.
- **Keep speciation headless-callable, mirroring `step()`/`advance_tick`**:
  implement the decision-and-construction logic as a plain function (e.g.
  `fn speciate(world: &mut SimWorld, config: &SimConfig, event: &SelectionThresholdCrossed) -> Option<SpeciesId>`)
  that doesn't require a Bevy `App`, with the consuming Bevy system as a thin
  wrapper that reads the drained message and calls it — the same split
  `TECH_DESIGN.md` invariant 2 requires of `step()` (`src/sim.rs:90-93`
  comment: kept as a plain struct "so `step()` stays callable without a Bevy
  `App`"). `tests/determinism.rs` and `tests/run_reproducibility.rs` drive
  the simulation headlessly; if speciation only exists inside a Bevy system,
  it's invisible to those suites and to this task's own unit tests, which
  should call the pure function directly rather than spinning up an `App`.

---

## 🔗 Dependencies

- **Depends on**: 106 (selection-pressure threshold-crossing signal this
  consumes), 038/088/089 (`net_self_interaction`, tag-set validity), 095
  (`draw_species_name`), the `Splice` mechanic (`apply_splice`,
  `input.rs:505-578`).
- **Blocks**: none. (The "how a fresh descendant is presented" celebration
  UI, if picked up later, would build on top of this task's log-entry
  placeholder — not scoped here.)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/107-evolution-speciation.md)"$'\n\nExecute this task in the current project.'
```
