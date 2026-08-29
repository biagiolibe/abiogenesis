# Task 178 (delivered: items 1+3) — Coexistence population floor + durations in seasons

> **ID**: `178`
> **Category**: Feature / Bug fix
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~3h originally scoped; ~1.5h for items 1+3 (advisor-recommended split, 2026-08-29)
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## ✅ Split note (2026-08-29)

Items 1 (Coexistence population floor) and 3 (durations in seasons)
delivered here. Items 2 (5 new objective types) and 4 (Speciation
target-species narrowing) moved to
[179](179-objectives-new-types-target-species.md) — an advisor review
recommended sequencing rather than re-splitting the file: land the two
correctness/tuning fixes that touch the same call sites first, defer the
larger additive work.

### What shipped

- **`ObjectiveProgress` collision fixed first**: 154a had repurposed
  `consecutive_ticks` as the `Speciation` activation-snapshot baseline —
  reworking that same counter's unit here (seasons vs. ticks) would have
  silently broken it. Gave the snapshot its own field,
  `speciation_snapshot`, before touching anything else.
- **Coexistence population floor**: took the enum-field route (not
  `evaluate(&SimConfig)` widening) per advisor review — `Objective::
  Coexistence` gained `min_population: u32`, severity-scaled by worldgen
  the same way `min_species`/`ticks` already are, world 0's opening
  objective hardcoded to `min_population: 1` (stays maximally gentle).
  Default `coexistence_min_population_base: 3` — started at `4`, lowered
  to `3` after checking it against `cell_carrying_capacity: 6`: the
  playtest complaint ("2 or 3 individuals cleared it") is closed with
  margin without risking an unclearable objective, which would be worse.
- **Second bug caught and fixed along the way, not part of I.6's filed
  scope**: `count_coexisting_species` was summing *occupied cells* per
  species, not individuals (`occupant.count`) — a task-137 regression
  (per-cell population model landed, this counter was never updated).
  Fixed as part of the same change since the population floor is
  meaningless against the wrong unit.
- **Durations in seasons**: `ObjectiveConfig::coexistence_ticks_base`/
  `survive_in_ticks_base` renamed to `coexistence_seasons_base`/
  `survive_in_seasons_base` (now genuinely season-native values, `4`/`3`),
  converted to the tick count `evaluate_sustained` counts against at
  `worldgen::generate_objectives`'s two call sites
  (`seasons_to_ticks(scale_severity(seasons_base, severity),
  season_pulses)`). `evaluate`'s pure `(objective, world, progress)`
  signature deliberately untouched — the season unit lives at the
  generation layer, not inside the evaluation engine.
- GDD §8 synced for both: victory-as-flag and the activation-snapshot
  correctness rule marked `[DECIDED, task 154]`, durations-in-seasons
  marked `[DECIDED, task 178]`, the Coexistence worked example updated
  with the population floor. The 5-new-types and named-target paragraphs
  stay `[PROPOSED, task 179]` — genuinely not implemented yet.

---

## 🎯 Objective

Remainder of task 154's original scope, split off after an advisor review
found 154's own AC list didn't actually cover two real playtest findings —
see `tasks/done/154-objectives-activation-victory-flag.md`'s split note for
the full history. Four independent pieces:

1. **`Coexistence` population floor** (`playtest_outcome.md` issue I.6, the
   playtest's single loudest complaint — "basta mantenere stabili due
   specie e anche se hanno 2 o 3 individui conta come obiettivo
   raggiunto"). Confirmed in code: `count_coexisting_species`
   (`src/objectives.rs`) only requires **presence** (≥1 living organism) per
   species, no population threshold. **This was never in 154's AC** — 154
   only touched the `Speciation` snapshot and victory-flag bugs; QUEUE.md's
   original note claiming 154 already covered this playtest finding was
   wrong and has been corrected.
2. **5 new `Objective` variants**: Homeostasis, Tolerance, WildCoexistence,
   Rootedness, FirstConfirmation — see 154's archived file for full
   per-variant specs, unchanged.
3. **Durations expressed in seasons**, not raw ticks — unchanged from 154's
   original scope.
4. **`Speciation`'s full target-species narrowing** — 154a shipped a
   simpler "any post-activation speciation clears it" fix (the literal
   correctness bug). The doc's fuller design — once a speciation has
   already happened before activation, narrow to a specific, named target
   species (deterministic selection, minimum-population floor, exclude
   already-speciated species, re-select on the target's extinction, name it
   in objective text) — was not implemented.

Design source: `redesign/processed/abiogenesis-objectives.md` (items 2-4,
same as 154 originally) + `playtest_outcome.md` issue I.6 (item 1, new).

---

## 📋 Acceptance Criteria

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] **Coexistence population floor**: took the enum-field route (`Objective::
      Coexistence` gained `min_population: u32`), not the `evaluate(&SimConfig)`
      widening — see split note above for why. Also fixed a second,
      previously-unreported bug in the same function
      (`count_coexisting_species` was summing occupied cells, not
      individuals).
- [ ] **5 new `Objective` variants** — moved to
      [179](179-objectives-new-types-target-species.md).
- [x] **Durations in seasons**: `ObjectiveConfig::coexistence_seasons_base`/
      `survive_in_seasons_base` (renamed from `*_ticks_base`), converted to
      ticks at generation time via `worldgen::seasons_to_ticks`. `evaluate`'s
      signature untouched.
- [ ] **Speciation target-species narrowing** — moved to
      [179](179-objectives-new-types-target-species.md).
- [x] Tests (for the delivered scope): Coexistence population-floor
      rejection/acceptance (including a case isolating the population-floor
      check from the species-count check); `speciation_snapshot`/
      `consecutive_ticks` collision fixed and covered by the existing
      154a snapshot test, now targeting the correct field.
- [x] GDD §8 updated for the delivered scope: victory-as-flag and
      activation-snapshot correctness rule marked `[DECIDED, task 154]`,
      durations-in-seasons marked `[DECIDED, task 178]`, Coexistence worked
      example updated with the population floor. 5-new-types and
      named-target paragraphs stay `[PROPOSED, task 179]`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` | `count_coexisting_species`, `Objective` enum, `evaluate`, `ZoneKind`. |
| `src/worldgen.rs` | `generate_objectives` — 5 new types wired in, population-floor generation, season-based durations. |
| `src/config.rs` | `ObjectiveConfig` — new population-floor constant, config for the 5 new types, duration fields reworked. |
| `src/ui.rs` | Coexistence/objective display match arms (`~1252`, `~1284`). |
| `sim_config.ron` | Mirror all `ObjectiveConfig` changes. |
| `abiogenesis-gdd.md` | §8 sync. |

---

## ⚠️ Constraints and Caveats

- No magic numbers — every new threshold into `SimConfig`.
- Never reveal *why* a parameter was picked for a given world in objective
  text (family-bias/trait-sign leaks) — same rule as trait naming.
- Keep `sim`/`world`/`config`/`objectives` free of `bevy::render`/
  `bevy_egui` deps.

---

## 🔗 Dependencies

- **Depends on**: 154 (landed — victory-as-flag and the base Speciation
  snapshot fix this task builds on).
- **Blocks**: 169 (Emersione, already blocked-then-unblocked by 154's
  victory-flag piece — this task doesn't reblock it, just adds more
  objective variety before Phase 3).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/178-objectives-tuning-new-types-durations.md)"$'\n\nExecute this task in the current project.'
```
