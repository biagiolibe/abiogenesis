# Task 026 — Log salient organism deaths, not just extinctions

> **ID**: `026`
> **Category**: Feature / UX
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-03 playtest

---

## 🎯 Objective

Task 019 deliberately logs only `SpeciesExtinct`, not individual `OrganismDied` events, to avoid flooding the Notebook's observation log during normal ecosystem churn (a bloom can produce dozens of deaths per era). A 2026-08-03 playtest surfaced the cost of that choice: a player spends an action point to `Seed` an organism, it dies within the same era (usually to an environmental mismatch — temperature or light, not the hidden matrix), and the Notebook shows **nothing** — no record of what died, when, or why. The player has to stare at the grid in real time to notice, which defeats the point of a log.

This task adds a middle ground: log deaths that are *salient from the player's perspective* — at minimum, an organism the player placed via `Seed` that dies — without reintroducing the "log every death" flood for organisms born naturally through reproduction.

---

## 📋 Acceptance Criteria

- [ ] A `Seed`-placed organism that later dies (`OrganismDied`, task 018) produces one Notebook log entry, era-tagged, distinguishable from a plain extinction entry (e.g. `"Era 4: your species 0 organism at (3,1) died"`).
- [ ] Organisms born through reproduction (`sim::step`'s existing repro logic) do **not** individually log on death — only player-placed ones do. The flood concern from task 019 must not resurface.
- [ ] Works across multiple eras: an organism seeded in era 2 that survives until era 5 and dies then still logs correctly (don't rely on same-era-only tracking).
- [ ] Doesn't touch `sim::step`'s energy/death/reproduction arithmetic — this is additive tracking + a notebook-side consumption change, not a tick-algorithm change (regression risk: re-run the full existing suite, not just new tests).
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `seed_organism_on_click` — the only place an organism becomes "player-placed"; needs to record that fact somewhere |
| `src/notebook.rs` | `record_events` — currently only consumes `SpeciesExtinct`; needs to also consume `OrganismDied` filtered by the player-placed marker |
| `src/sim.rs` | `OrganismDied { cell, species }` (task 018) — read-only reference, no changes expected here unless the marker needs to travel through this event itself (see Technical Context) |

---

## 🧩 Technical Context

**The open design question this task has to resolve**: how does a death event (`OrganismDied { cell, species }`, keyed only by grid index and species) get matched back to "was this the organism the player placed"? A raw cell index is ambiguous over time — the same cell can be seeded, die, and get reseeded (by the player or by reproduction) many times across a run, so "cell 137 died" doesn't by itself mean "the organism I placed at cell 137 died."

Two reasonable approaches, in increasing order of invasiveness — pick one and document the choice:

1. **Track placement, expire on any occupancy change.** `input.rs` (or a new small resource, e.g. `PlayerPlacedCells: HashSet<usize>` living in `ui.rs` or `notebook.rs`) records the cell index whenever `seed_organism_on_click` succeeds. `notebook.rs::record_events` checks `OrganismDied.cell` against this set: if present, log the salient entry and remove the entry from the set either way (the cell's "player-placed-ness" is consumed whether it died or something else happened to it, e.g. `Cull`). This is a UI/presentation-side bookkeeping structure, not simulation state, so the project's "no `HashMap` iteration in sim/world/config" determinism rule (`TECH_DESIGN.md` §5) doesn't apply here — only membership checks are needed, no iteration.
2. **Tag the organism itself.** Add a field to `Organism` (`world.rs`) marking provenance (player-placed vs. born). More invasive — touches a core domain type read by `sim::step` every tick — and probably overkill for what's fundamentally a UI concern. Only go this route if approach 1 turns out to have a correctness gap approach 2 doesn't (e.g. if two placements can land on the same cell across the same tick in a way that confuses the `HashSet` bookkeeping — check this doesn't happen given `Seed`'s existing empty-cell precondition).

Whichever is chosen, remember: reproduction creates a *new* organism at a *different* cell (an empty Moore neighbour) — it should never be marked as player-placed, so approach 1's set should only ever gain entries from `seed_organism_on_click`, never from `sim::step`'s reproduction path.

---

## 🔨 Suggested Implementation

1. Pick and document the tracking approach (see Technical Context) — approach 1 is the recommended default unless it doesn't hold up.
2. `input.rs::seed_organism_on_click`: on a successful seed, record the cell index into the tracking structure.
3. `notebook.rs::record_events`: add a `MessageReader<OrganismDied>` parameter, check each death's cell against the tracking structure, and push a distinct `LogEntry` for matches (clear the entry from tracking regardless of match, so stale entries don't accumulate forever).
4. Handle the reseed (`r` key) case: the tracking structure must be cleared alongside `MatrixKnowledge`/`ObservationLog`/`ActionBudget`/`SelectedSpecies`/`SpliceDraft` in `input.rs::reseed_world` (task 025 already established this "everything referring to a world that no longer exists gets reset here" pattern — this is one more thing in that list).
5. New unit tests: a player-placed organism dying logs a distinct entry; a reproduced organism dying does not; an organism placed in one era and dying several eras later still logs correctly; reseeding clears tracked placements.
6. Manual verification: repeat the 2026-08-03 playtest scenario (seed organisms that die from environmental mismatch) and confirm the Notebook now shows something.

---

## ⚠️ Constraints and Caveats

- Do not relax task 019's "no per-tick flood" guarantee for naturally-reproduced organisms — this task is additive (one more salient-event type), not a reversal of that decision.
- Keep the log entry text clearly distinguishable from an extinction entry (different phrasing, not just a different era number) — task 021's UI (and any future notebook UI) may want to visually distinguish entry types eventually, so don't conflate the two in a way that makes that harder later.
- If `Cull` (task 024) removes a player-placed organism before it would have died naturally, decide whether that should also log (arguably yes — it's the same "what happened to the thing I placed" question) or is explicitly out of scope; if cut, note it rather than silently ignoring it, same as task 024 did for `Cull`'s own missing event.

---

## 🔗 Dependencies

- **Depends on**: 018 (event foundation), 019 (observation log), 022 (action budget — `Seed` is now budget-gated, making each placement more deliberate and worth tracking)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/026-salient-death-logging.md)"$'\n\nExecute this task in the current project.'
```
