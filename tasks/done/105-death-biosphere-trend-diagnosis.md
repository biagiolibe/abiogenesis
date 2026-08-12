# Task 105 — Cause label on the Biosphere panel for a species taking deaths

> **ID**: `105`
> **Category**: UX / Legibility
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-death-legibility.md`)

---

## 🎯 Objective

The HUD's Biosphere panel shows a per-species trend glyph (`▲`/`▼`/`▬`,
`ui.rs::trend_glyph`, `src/ui.rs:1119-1125`) but no explanation for why a
species is taking losses. The design doc's motivating example is a predator
population quietly starving somewhere on the map with no legible signal
why — the single-death message (task 104) only covers organisms the player
personally placed via `Seed`, so it never explains population-wide decline
among organisms the player didn't place, and per-organism log lines for
those are explicitly out of scope (GDD §7's curated-log principle, the same
noise problem task 100 fixed).

Attach a short qualitative cause label next to a species' Biosphere row,
derived from the dominant term across that species' deaths in the current
era — population-wide aggregate, using the same cause taxonomy as task 104
(temperature fit / resource absence / predation / crowding / vague matrix
mention), not a new one. As grounded below, this label is gated on deaths
actually recorded this era, not on the `▼` glyph — the glyph tracks
something else and would miss the doc's own motivating case if used as the
gate.

**Correction to the design doc's grounding**: the doc states "deaths are
already tracked per species for the trend arrow itself." This is not
accurate as of this session — `PopulationTrend` (`src/ui.rs:1036-1041`,
computed by `update_population_trends`, `src/ui.rs:1086-1116`) is derived
from **era-over-era average energy per surviving individual**
(`species_stats`, `src/ui.rs:1006-1027`), not from a death count or death
cause. No per-species death tally exists anywhere in the codebase today
(confirmed by grep — the closest precedent is `notebook.rs`'s
`BirthTally`, `src/notebook.rs:111-118`, which tallies `OrganismBorn`
per-species per-era and is a reasonable structural pattern to copy, but it
tracks births, not death causes). This task must add new per-species,
per-era aggregation state — it is not free reuse of existing tracking.

---

## 📋 Acceptance Criteria

- [ ] A new resource (e.g. `DeathCauseTally`, modeled on `BirthTally`'s
      shape, `src/notebook.rs:111-118`) accumulates, per species, the
      dominant cause (per task 104's classifier — reused, not
      reimplemented, see Dependencies) of every `OrganismDied` event within
      the current era, for **all** deaths of that species (not only
      player-placed ones — this is the aggregate, population-wide case).
- [ ] On `EraCompleted`, for **every species with at least one death
      recorded this era** (not gated on `PopulationTrend::Falling` — see
      Technical Context for why: the trend glyph tracks average energy of
      survivors, not headcount, so a species being culled by deaths can
      read `▲`/`▬` and never trip a `Falling` gate, which is exactly the
      "quietly starving" case this task exists for), the most-tallied
      dominant cause across that era's deaths is exposed for the HUD to
      read; ties are broken by **most recent dominant cause wins**
      (explicit first-pass choice, stated here rather than left open — pick
      something else only if a clearly better rule surfaces during
      implementation, and document the change).
- [ ] The tally resets to empty at the start of each era (same reset
      cadence as `BirthTally`, not cumulative across eras) and on world
      reset (`r` key), matching `run_flow.rs`'s existing reset pattern for
      per-era HUD state (see `src/run_flow.rs:109-111` for the
      `PopulationTrends` precedent).
- [ ] The Biosphere row for a species with at least one death this era
      (`src/ui.rs:423-424`) renders a short cause label next to whichever
      trend glyph is currently showing (e.g. "▼ cold" / "▲ predation" / "▬
      crowded") — the label is driven by deaths, independent of which
      glyph the energy-average trend happens to show. A species with a
      `Falling` glyph but zero deaths this era (energy sagging without
      anyone actually dying yet) shows no label rather than a fabricated
      guess.
- [ ] A species with zero deaths this era shows no label regardless of its
      glyph.
- [ ] Unit test for the aggregation/tie-breaking logic: construct a
      sequence of per-species dominant causes across several synthetic
      deaths in one era, including a case where two causes are tied in
      count, and assert the exposed cause matches the "most recent dominant
      cause wins" rule stated above.
- [ ] Unit test confirming the tally resets between eras (a cause dominant
      in era N does not leak into era N+1's label after only era-N+1
      deaths with a different dominant cause).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: engineer a species taking deaths this
      era (e.g. isolate a Predator with no prey, or place a species off its
      temperature optimum) and confirm the Biosphere panel shows a
      plausible cause label next to that species' row once an era completes
      with deaths, whatever glyph happens to be showing.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `PopulationTrend`/`PopulationTrends` (1036-1116), `trend_glyph`/`trend_color` (1119-1140+), Biosphere row rendering (~347-424) — add the cause label next to the glyph. |
| `src/notebook.rs` | `BirthTally` (111-118) — structural precedent for a new per-species per-era tally resource. |
| `src/run_flow.rs` | World-reset resource clearing (109-111) — add the new tally resource to this reset list. |
| `src/sim.rs` | `OrganismDied` — source event for the aggregation (consumes the `env_fit` field added by task 104, if 104 lands first — see Dependencies). |
| `src/text.rs` | Task 104's dominant-cause classifier — reuse, don't duplicate (see Dependencies). |

---

## 🧩 Technical Context

- **Current behavior**: `PopulationTrend` reflects average-energy direction
  only; there is no cause information attached to any glyph, and no
  per-species death aggregation exists anywhere.
- **Desired behavior**: any Biosphere row with at least one death this era
  shows the dominant cause behind those deaths, using the same qualitative
  taxonomy as the single-death message (task 104) — attached next to
  whichever glyph is showing, not gated on the glyph being `▼`.
- **Why not gate on `▼`**: `species_stats` (`ui.rs:1006-1027`) averages
  energy over *surviving* individuals only. When the weakest individuals of
  a species die, the survivors' average energy can go *up*, so a species
  actively being culled by deaths can show `▲` or `▬` and never trip a
  `PopulationTrend::Falling` gate — exactly the "predators quietly starving
  somewhere on the map" case the design doc named as this feature's reason
  to exist. Gating the label on deaths-this-era (not on the glyph) is the
  fix; `species_stats` already computes population per species too
  (currently discarded via the `_population` binding at `ui.rs:1007`,
  informative if a future pass wants a true population-based trend
  instead, but out of scope here).

---

## 🔨 Suggested Implementation

1. Land task 104 first if possible, and extract its per-`OrganismDied`
   dominant-cause classification into a function callable from both
   `text.rs` (single-death message) and this task's aggregation — a plain
   `fn dominant_cause(&OrganismDied, Metabolism) -> DeathCause` enum is
   enough. If 105 is built before 104 lands, implement the classifier here
   and flag in task 104 (or its PR) that the two should be reconciled to
   share one classifier rather than diverging.
2. Add `DeathCauseTally` resource: per-species `Vec<(DeathCause, era_tick_or_order)>`
   or equivalent, enough to support "most recent wins" tie-breaking.
3. Add a system reading `MessageReader<OrganismDied>` each tick, tallying
   cause per species (mirrors `notebook.rs`'s `BirthTally` accumulation
   pattern). `OrganismDied` is written from two call sites —
   `sim.rs:544` (era-advance ticks) and `input.rs:165` (the manual
   single-tick path) — so schedule this system the same way
   `notebook.rs::record_events` is scheduled (it already reads deaths from
   both paths correctly); do not restrict it to an era-advance-only run
   condition or manual-tick deaths will silently never get tallied.
4. On `EraCompleted`, compute each species' dominant cause for the era
   (respecting the tie-break rule) and reset the tally.
5. Extend `run_flow.rs`'s reset system to clear the new resource on world
   reset, alongside `PopulationTrends`.
6. In `ui.rs`'s Biosphere row rendering, look up the cause for any species
   with a recorded era cause and render it next to the glyph, regardless of
   which glyph is showing.
7. Unit tests for aggregation, tie-breaking, and era reset.
8. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
9. Live-verify per the acceptance criteria.

---

## ⚠️ Constraints and Caveats

- Do not add per-organism log lines for organisms the player didn't
  place — this task is strictly an aggregate label on the existing trend
  glyph, not a new log feed. That noise problem was deliberately avoided by
  `notebook.rs`'s existing filter and task 100's log cleanup; don't
  reintroduce it here.
- Do not show a cause label for any species with zero deaths recorded this
  era, regardless of its glyph — that would be a fabricated cause, not a
  diagnosis.
- Keep the matrix cause's vagueness intact in the aggregate label too — no
  tag identity, no sign, no number, consistent with task 104's rule.
- The tie-breaking rule ("most recent dominant cause wins") is an explicit
  first-pass choice per the design doc's own admission that this was left
  open — state whatever rule is actually implemented, don't leave it
  ambiguous in code or docs.

---

## 🔗 Dependencies

- **Depends on**: 104 (shares the dominant-cause classification logic —
  land 104 first if sequencing allows, then extract/reuse its classifier
  here rather than duplicating it; if 105 is built independently, flag
  during 104's implementation that the two need reconciling onto one
  shared classifier), 063 (`PopulationTrend`/trend glyph this extends), 064
  (Biosphere panel layout).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/105-death-biosphere-trend-diagnosis.md)"$'\n\nExecute this task in the current project.'
```
