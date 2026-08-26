# Task 120 — Biosphere: numeric population delta alongside the trend arrow

> **ID**: `120`
> **Category**: UI / HUD
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-12 (scoped from `redesign/abiogenesis-hud-notebook.md` §4, after a
> discrepancy-check pass against tasks 100-103/097)

---

## 🎯 Objective

`redesign/abiogenesis-hud-notebook.md` §4 asks for each Biosphere row to
show a numeric delta from the previous era (`+4`, `-2`, `±0`) alongside the
existing trend arrow (▲/▼/▬) — the arrow alone gives direction, the number
gives magnitude, letting the player catch a population explosion or crash
before it's irreversible.

**Important finding, worth reading before implementing**: the doc's own
framing assumes the trend arrow already reflects *population*. It doesn't.
`ui.rs::update_population_trends` (`src/ui.rs:1092-...`) currently derives
`PopulationTrend` from **average energy per organism**
(`trends.previous_avg_energy`), not population count — `classify_trend`
compares this era's average energy against last era's. So today's row shows
an energy-based trend arrow. The doc's requested delta (`+4`/`-2`) is
explicitly a **population-count** delta ("permette di accorgersi... di
un'esplosione o crollo demografico" — an explosion/crash is a population
event, not an energy one). Adding a population-count delta next to an
energy-based trend arrow means the two indicators on the same row can
legitimately disagree (population rising while average energy falls, or
vice versa) — this task adds the delta as literally requested without
changing what the trend arrow measures (out of scope here, and the doc
itself says the trend arrow is "invariato"/unchanged) — see Acceptance
Criteria for how to handle this without it reading as a bug.

---

## 📋 Acceptance Criteria

- [x] Each Biosphere row gains a population delta since the previous era,
      formatted `+N`/`-N`/`±0` (sign-prefixed, `±0` for exactly zero
      change), placed next to the existing trend arrow per the doc's
      layout.
- [x] Delta is computed from **population count** (not energy) — reuse
      `species_stats`'s existing `population: usize` per species
      (`src/ui.rs:1012-1032`), comparing this era's count against last
      era's, the same "compare against last era, reset on
      `EraCompleted`" shape `update_population_trends` already uses for
      energy (a parallel `previous_population: Vec<Option<usize>>` field,
      likely folded into the existing `PopulationTrends` resource rather
      than a new one — same lifecycle, same growth-on-new-species handling
      already written there).
- [x] A species with no previous-era record (first era it's ever had a
      nonzero population) shows a sensible baseline delta rather than a
      misleading huge jump from an implicit `0` — decide and document the
      choice (e.g. `±0`/no delta shown for a species' first era with
      population, or `+N` from an explicit `0` baseline if that reads fine
      in practice — check via `cargo run` which looks better, this is a
      first-pass display call, not a simulation-affecting one).
- [x] The energy-based trend arrow (`classify_trend`/`PopulationTrend`)
      itself is **unchanged** — this task adds a new, separate,
      population-count-based number; it does not redefine what the arrow
      measures. If, once both are visible in the same row, the mismatch
      between an energy-trend arrow and a population-count delta reads as
      confusing or actively wrong during the live-verification pass below,
      stop and flag it back rather than silently "fixing" the trend arrow's
      definition — that's a design decision beyond this task's scope.
- [x] Unit test: population delta computed correctly across consecutive
      `EraCompleted` events (mirrors `update_population_trends`'s existing
      energy-trend test coverage, if any — check `ui.rs`'s test module).
- [x] Unit test: a species' first era with any population gets the
      documented baseline behavior (see above), not an unexplained huge
      delta.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: watch a Biosphere row across several
      eras as population visibly grows/shrinks — confirm the delta number
      matches what's actually happening, updates once per era (not every
      tick), and reads clearly next to the trend arrow. **Skipped this
      session by explicit user instruction** ("senza fare live
      verification") — outstanding if a visual check is ever wanted.

---

## ✅ Completion notes

Baseline choice for a species' first era with population: **no delta shown**
(empty string, not `+N` from an implicit zero) — `PopulationTrends` grows a
`previous_population: Vec<Option<usize>>` / `current_population_delta: Vec<Option<i64>>`
pair alongside the existing `previous_avg_energy`/`current` fields, same
resize-on-`EraCompleted` lifecycle. Delta computation extracted as a pure
`population_delta(previous: Option<usize>, current: usize) -> Option<i64>`
helper (mirrors `classify_trend`'s shape) so it's unit-testable without a
full `SimWorld`/ECS harness; `text::population_delta_label` formats it
(`+N`/`-N`/`±0`/empty). Wired into the Biosphere row right after the
existing trend glyph in `hud_panel`. Energy-based `classify_trend`/
`PopulationTrend` left untouched, per the task's own constraint — no
mismatch-driven flag-back needed since live verification was skipped this
session (see above).

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `PopulationTrends`/`update_population_trends` (line ~1078-1121) — add the population-delta tracking alongside the existing energy-trend tracking; `species_stats` (line 1012-1032) — already returns population count per species, the data source for the delta; the Biosphere row-rendering loop (`hud_panel`, where `text::population_line`/`trend_glyph` are called) — add the delta's display. |
| `src/text.rs` | Wherever the Biosphere row's line is formatted (`population_line` or similar) — extend or add a delta-formatting function (`+N`/`-N`/`±0`). |

---

## 🧩 Technical Context

- **Current behavior**: `PopulationTrends` tracks `previous_avg_energy: Vec<Option<f32>>`
  per species, updated once per `EraCompleted`, feeding `classify_trend`
  (energy-based ▲/▼/▬). Population *count* itself is already computed
  per-frame by `species_stats` for the row's absolute-count display, but
  never diffed against a previous value.
- **Desired behavior**: a second, parallel piece of per-era state —
  previous-era population count — diffed against the current era's count,
  displayed as a signed number.
- Reuse `PopulationTrends`'s existing growth-on-new-species handling
  (`if trends.previous_avg_energy.len() < species_count { resize... }`) for
  whatever new field holds previous population, rather than duplicating that
  pattern in a second resource.

---

## 🔨 Suggested Implementation

1. Add `previous_population: Vec<Option<usize>>` to `PopulationTrends`,
   grown the same way `previous_avg_energy` already is.
2. In `update_population_trends`, alongside the existing energy-trend
   update, record this era's population count into a queryable form (either
   the raw previous count, or a precomputed signed delta — whichever is
   simpler to expose to the row-rendering code).
3. Decide and implement the "no previous era" baseline behavior (see
   Acceptance Criteria).
4. Add a `text.rs` delta-formatting function, wire it into the Biosphere
   row alongside the existing trend arrow.
5. Unit tests per Acceptance Criteria.
6. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
7. `cargo run`: verify per the acceptance criteria's live-verification line,
   paying attention to whether the energy-trend-arrow vs.
   population-count-delta mismatch (see Objective) reads as confusing in
   practice.

---

## ⚠️ Constraints and Caveats

- **Don't redefine `PopulationTrend`/`classify_trend`** to be population-
  based instead of energy-based — that's a different, out-of-scope change;
  flag it back if the live-verification pass suggests it's actually needed,
  don't make that call unilaterally inside this task.
- Delta updates once per era (`EraCompleted`), matching the existing trend
  arrow's cadence — not a live per-tick counter, which would make the HUD
  number flicker distractingly during an era's animation.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.
- **Related, not a dependency**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/120-biosphere-population-delta.md)"$'\n\nExecute this task in the current project.'
```
