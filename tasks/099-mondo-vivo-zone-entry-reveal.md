# Task 099 — Reveal-on-first-zone-entry for conditional tags

> **ID**: `099`
> **Category**: Feature / Simulation / Notebook
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-living-world.md`, §2b "Reveal-on-first-zone-entry")

---

## 🎯 Objective

Same underlying mechanism as task 096's conditional tags: when a
player-seeded species occupies a `TerrainKind` it has never occupied before
in this run, and it carries a tag conditioned on that terrain, log the
discovery the same way as an unconfirmed→confirmed transition (task 054's
pattern). The zone entry itself is the trigger — no separate confirmation
system, this reuses 096's per-world conditional-tag data and fires a log
entry the first time a species' population actually sets foot on the
matching terrain.

**Shared-tracking requirement**: this task needs a per-species "which
terrain has this species' lineage ever occupied in this run" structure.
`redesign/abiogenesis-evolution-xenotypes.md`'s speciation trigger (task
106) independently needs terrain-exposure stimulus tracking for the same
purpose (deciding when a lineage has been exposed enough to a terrain to
speciate). **Build this once, here, in a shape task 106 can reuse
directly** — do not let this task build an isolated tracking structure that
106 later has to duplicate or rework.

---

## 📋 Acceptance Criteria

- [ ] A new per-`SpeciesId` terrain-occupancy tracker: a bitmask (or
      fixed-size `[bool; 4]`/small bitflags type) over `TerrainKind`'s 4
      variants (`Sea`/`Plain`/`Hill`/`Mountain`, `world.rs:140-146`), one
      entry per species, **not** a `HashMap` (CLAUDE.md's no-`HashMap`-
      iteration rule for sim logic) — a `Vec` indexed by `SpeciesId.0`,
      growing when `Splice` (or this task's own reveal logic) adds a new
      species, mirroring how `world.species: Vec<Species>` itself grows.
      Reset per world the same way `world.ever_populated` is (fresh, empty,
      at `SimWorld::new_for_world`).
- [ ] Updated wherever an organism actually lands on the grid in
      `sim::step` — the same sites `ever_populated` and reproduction already
      touch (`src/sim.rs:359` initial write-back, `379-384` reproduction
      birth cell) — not recomputed from scratch every tick by scanning the
      whole grid.
- [ ] When updating the tracker for a species/terrain pair that (a) is
      newly set (this species has never occupied this terrain before this
      run) and (b) the species carries a tag that task 096's per-world
      conditional-tag roll conditions on that exact terrain: fire a log
      entry using the same unconfirmed→confirmed notebook pattern task 054
      established (`notebook.rs`'s confirmation log path, `notebook.rs:254-259`)
      — reuse `text::confirmation_message` or add a parallel, clearly-named
      text function if the existing one's wording doesn't fit a
      terrain-reveal (check `src/text.rs` before deciding either way).
- [ ] The reveal fires once per (species, conditional tag, terrain) —
      re-entering the same terrain later in the run must not re-log.
- [ ] Unit test: a species entering its tag's trigger terrain for the first
      time fires exactly one reveal; re-entering does not fire again.
- [ ] Unit test: a species entering terrain its tag is *not* conditioned on
      fires no reveal.
- [ ] Unit test: a species with no conditional tags never fires a reveal,
      regardless of terrain visited (regression coverage — this must not
      touch species with only unconditional tags).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: seed a species carrying a conditional
      tag (per 096), let it spread onto its trigger terrain for the first
      time this run, confirm a log entry appears in the notebook the moment
      it happens.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `TerrainKind` (140-146), `SimWorld` (new tracker field, near `ever_populated`, 216), `Cell::terrain` (160). |
| `src/sim.rs` | Organism placement sites (359, 379-384) — where the tracker gets updated; `ever_populated` flip (124-130) as the closest existing precedent for "state that flips once, tracked outside the tick's core loop." |
| `src/notebook.rs` | `accumulate_evidence`'s confirmation log emission (233-259) — pattern to mirror for the reveal's log entry. |
| `src/text.rs` | `confirmation_message` (or wherever it lives) — check before deciding whether to reuse or add a parallel message function. |
| `tasks/done/054-celebrate-first-confirmed-hypothesis.md` | The unconfirmed→confirmed pattern this task's log entry follows. |
| `tasks/096-mondo-vivo-conditional-tags-core.md` | Supplies the per-world conditional-tag data (`(TagId, TerrainKind, Mode)`) this task reads — must land first. |

---

## 🧩 Technical Context

- **Current behavior**: no terrain-occupancy history exists anywhere —
  `Cell::terrain` is read live per-tick (placement gating, environment
  scalars) but nothing records "has species X ever stood on terrain Y this
  run."
- **Desired behavior**: a small, growable per-species terrain-occupancy
  record, updated at organism-placement time, checked against task 096's
  per-world conditional-tag roll to decide when to fire a one-time reveal.
- **Why this must be a shared, reusable shape, not a one-off**: the source
  doc's own open questions section flags that this tracking need recurs in
  `redesign/abiogenesis-evolution-xenotypes.md` (task 106) — that doc's
  speciation trigger needs to know when a lineage has been exposed to a
  terrain "enough" to justify a new descendant species. If this task builds
  a private, task-099-only structure, 106 either duplicates it or has to
  refactor 099's code to expose it later. Building it as a plain,
  general-purpose `Vec<TerrainOccupancy>` (or similarly named, not
  reveal-specific) on `SimWorld` from the start avoids that rework.

---

## 🔨 Suggested Implementation

1. Land task 096 first (this task reads its per-world conditional-tag data).
2. `src/world.rs`: add a small `TerrainOccupancy` type (bitmask over the 4
   `TerrainKind` variants) and `SimWorld::terrain_occupancy: Vec<TerrainOccupancy>`,
   indexed by `SpeciesId`, grown alongside `world.species`.
3. `src/sim.rs`: at each organism-placement site (359, 379-384), mark the
   occupying species' bit for `world.cells[idx].terrain` (or the target
   cell's terrain for the reproduction case) if not already set; if this is
   a newly-set bit, check task 096's conditional-tag roll for a match and
   queue the reveal.
4. `src/notebook.rs` or a new small system: drain the queued reveals into a
   log entry, following `accumulate_evidence`'s confirmation-log shape.
5. Unit tests per Acceptance Criteria.
6. `cargo run` live verification.
7. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- **No `HashMap` iteration in sim logic** — the occupancy tracker must be a
  `Vec`/array-backed structure indexed by `SpeciesId`, not a `HashMap`.
- **Build once, reuse**: name and shape the tracker so task 106
  (`abiogenesis-evolution-xenotypes.md`'s speciation trigger) can read it
  directly rather than rebuilding it — do not make it reveal-specific in
  naming or API (e.g. avoid `RevealTracker`, prefer `TerrainOccupancy` or
  similar, generically named).
- **Determinism**: the tracker's updates happen in `sim::step`'s existing
  deterministic organism-placement order — no new RNG, no new source of
  non-determinism.
- **Splice-created species**: a `Splice` descendant is a new `SpeciesId`
  with no occupancy history — the tracker must grow to cover it, starting
  empty, not inherit the parent's occupancy (a descendant hasn't itself
  stood anywhere yet).

---

## 🔗 Dependencies

- **Depends on**: 096 (conditional-tag data model this reads).
- **Blocks**: none directly, but **task 106**
  (`abiogenesis-evolution-xenotypes.md`'s speciation trigger, not yet
  scoped into a task file as of this session) is expected to reuse this
  task's `TerrainOccupancy` structure — flag this dependency explicitly
  when 106 is scoped, so it builds on top of this instead of duplicating it.
- **Related**: 054 (log-entry pattern reused), 057/058 (species/notebook
  legibility precedent for how this surfaces without overloading the UI).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/099-mondo-vivo-zone-entry-reveal.md)"$'\n\nExecute this task in the current project.'
```
