# Task 179 — Objectives: 5 new types, Speciation target-species narrowing

> **ID**: `179`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~2.5h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29 (filed; not started)
> **Completed**: 2026-08-29 — 4 of 5 new types shipped, `FirstConfirmation`
> deferred (needs `MatrixKnowledge`, a separate `Resource` not reachable
> from `evaluate`'s `&SimWorld`).

---

## 🎯 Objective

Remainder of task 178's scope (itself the remainder of 154's original
scope — see `tasks/done/154-*.md` and `tasks/done/178-*.md` for the full
history). Two independent pieces, both design-heavy rather than
correctness fixes, which is why they were sequenced after 178's items:

1. **5 new `Objective` variants**: Homeostasis, Tolerance, WildCoexistence,
   Rootedness, FirstConfirmation.
2. **`Speciation`'s target-species narrowing**: 154a shipped the literal
   correctness fix (any post-activation speciation clears it); this is the
   doc's fuller design on top of that.

Design source: `redesign/processed/abiogenesis-objectives.md`.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] **4 of the 5 new `Objective` variants**, each reading real per-world
      data to parametrize itself (mirrors `SurviveIn`'s "pick a zone
      that's actually present" and `TriggerBloom`'s "pick a seedable
      species" pattern in `worldgen.rs`) — resolvable from `&SimWorld`
      alone, so `evaluate`'s pure signature stays untouched:
  - `Homeostasis`: target species' mean energy (`population.energy /
    population.count`, aggregated across the species' cells) stays within
    a config-bound band for N seasons.
  - `Tolerance`: target species survives in a high-toxicity zone for N
    seasons — reuses `ZoneKind`/`species_present_in_zone`; decide whether
    `ZoneKind::Toxic` (currently Swamp-only) needs a Crater-adjacent
    variant or whether Tolerance simply reuses the existing zone as a
    second, independently-generated instance of the same mechanic
    (`SurviveIn` and `Tolerance` differ in name/pool-membership, not
    necessarily in mechanism — note the choice).
  - `WildCoexistence`: a wild species (task 098, `world.is_wild`) stays
    alive alongside at least one player-seeded species for N seasons.
  - `Rootedness`: a species stays alive specifically in a biome tied to an
    active terrain-conditioned trait (`world.conditional_tags`, GDD §5.5)
    in that world — needs a `TerrainKind` → `Biome` mapping if one doesn't
    already exist in a reusable form (check `sim.rs`'s per-tick terrain
    classification first, likely already does this per-cell).
  - Objective text states only *what* it measures, never *why* that
    parameter was picked for this world (no family-bias or trait-sign
    leaks) — same rule already applied to trait naming.
- [ ] **`FirstConfirmation` — investigate before implementing.** Confirmed
      during 178/179 scoping: it needs `MatrixKnowledge` (confirmed-relation
      state), which is a separate `Resource`, not reachable from
      `evaluate`'s `&SimWorld`. Two routes: (a) widen `evaluate`'s
      signature (touches ~18 call sites, breaks the "narrow pure function"
      property the other 4 variants preserve), or (b) special-case it in
      `apply_tick_outcome` with a `Res<MatrixKnowledge>` added to
      `ObjectiveOutcomeParams` (mirrors how `WorldVictory`/`GraceProgress`
      already ride along there) — evaluated separately from `evaluate_world`'s
      generic per-objective dispatch. Route (b) is very likely the right
      one; implement it if time allows, otherwise defer this single variant
      with a note, ship the other 4.
- [ ] **Speciation target-species narrowing**: once at least one speciation
      has already happened in the world before `Speciation` activates
      (detectable via the same `species_parent.len()` snapshot 154a
      introduced), the objective narrows to a **specific target species**
      — deterministic, seed-derived selection among currently-eligible
      living species above a minimum-population floor, excluding species
      that have already speciated — with re-selection on the target's
      extinction. Objective text must name the species (existing
      name/icon plumbing — Catalog/HUD/genome bank), never a bare ID.
      **Any new state this needs should be a field on `ObjectiveProgress`,
      not a new `Resource`** — a new resource re-triggers the test-site
      registration trap hit twice already in 154/178 (three
      `app.insert_resource(CurrentWorldOutcome…)` blocks in `input.rs`,
      `objective_outcome_world` in `objectives.rs`, `resource_world` in
      `run_flow.rs` all need updating for every new resource in this
      dependency chain).
- [ ] Tests: each new objective type's evaluate logic against a hand-built
      `SimWorld` (mirrors existing `evaluate`/`evaluate_world` unit tests,
      no Bevy `App` needed); target-species selection determinism and
      extinction-fallback reselection.
- [ ] GDD §8 updated: 5 new objective types marked `[DECIDED]` (whichever
      actually ship — `[PROPOSED]` stays for `FirstConfirmation` if
      deferred), target-species narrowing marked `[DECIDED]`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` | `Objective` enum, `evaluate`, `ZoneKind`, `ObjectiveProgress`. |
| `src/worldgen.rs` | `generate_objectives`/`generate_one_objective` — new types wired in. |
| `src/config.rs` | `ObjectiveConfig` — config for the new types. |
| `src/knowledge.rs` | `MatrixKnowledge` — `FirstConfirmation`'s data source, if implemented. |
| `src/ui.rs` | Objective display match arms (`~1252`, `~1284`). |
| `sim_config.ron` | Mirror `ObjectiveConfig` additions. |
| `abiogenesis-gdd.md` | §8 sync. |

---

## ⚠️ Constraints and Caveats

- No magic numbers — every new threshold into `SimConfig`.
- Never reveal *why* a parameter was picked for a given world in objective
  text.
- Keep `sim`/`world`/`config`/`objectives` free of `bevy::render`/
  `bevy_egui` deps — `evaluate`'s narrow signature is worth protecting;
  route new data needs through `apply_tick_outcome`/`ObjectiveOutcomeParams`
  instead, per `FirstConfirmation`'s note above.
- Prefer a field on `ObjectiveProgress` over a new `Resource` for any new
  per-objective state — see the test-site note under target-species
  narrowing above.

---

## 🔗 Dependencies

- **Depends on**: 154, 178 (landed — victory-as-flag, base Speciation
  snapshot, Coexistence population floor, durations in seasons).
- **Blocks**: none directly; widens the objective pool before Phase 3.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/179-objectives-new-types-target-species.md)"$'\n\nExecute this task in the current project.'
```
