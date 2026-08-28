# Task 154 — Objectives: activation snapshot, 5 new types, durations in seasons, victory as a flag

> **ID**: `154`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~3-4h (largest single task in Phase 2 — four largely independent sub-changes; consider splitting into 154a/b/c if picked up piecemeal)
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

Four related corrections to the objectives system (GDD §8), all from
`redesign/processed/abiogenesis-objectives.md`:

1. **Snapshot at activation.** An objective must measure change *since it
   became active*, never a pre-existing world state — today `Coexistence`/
   `SurviveIn` already do this correctly by construction (`consecutive_ticks`
   starts at 0 when `ObjectiveProgress::default()` is installed on
   activation), but `Speciation` does not: it checks `world.has_speciated`,
   a sticky flag set the moment *any* speciation ever happens in the world's
   life, so if a speciation occurred before `Speciation` became the active
   objective (it's always last in sequence, but a Coexistence/SurviveIn
   objective earlier in the sequence could still run long enough for an
   unrelated speciation to fire first), the objective clears the instant it
   activates, with the player having done nothing to earn it.
2. **5 new objective types**: Homeostasis (energy), Tolerance
   (chemolithotroph/toxicity — consumes task 145's toxicity-axis Stress),
   Wild Coexistence, Rootedness (terrain-conditioned traits), First
   Confirmation (world-0/onboarding). Widens the pool `generate_objectives`
   draws from; does not raise the 2→3 + Speciation sequence count.
3. **Durations expressed in seasons**, not raw ticks — `Coexistence`/
   `SurviveIn`'s `ticks: u32` fields (and worldgen's
   `coexistence_ticks_base`/`survive_in_ticks_base`) predate task 135's
   three-level time split; a `tests/config_ron_sync.rs`-adjacent test
   already asserts both bases are exact multiples of `season_pulses`, so the
   values are season-aligned in practice but not season-*expressed* — the
   field name and the generation-time math still talk in ticks.
4. **Victory as a flag, not a forced world end `[DECIDED]`** — clearing the
   last objective in the sequence today calls
   `params.next_game_state.set(GameState::WorldCleared)` unconditionally
   (`objectives.rs:511-513`), which immediately routes to the next world via
   `screens.rs`'s `world_cleared_screen_ui`. This makes Emersione
   (needs a chain of ≥3 related speciations, cfr. `redesign/processed/
   abiogenesis-emersione.md`, task 169) unreachable by construction, since
   the world always ends at the *first* speciation-driven clear. Change:
   clearing the last objective sets a victory flag and the world keeps
   running (same budget-exhaustion/extinction failure conditions still
   apply); the player advances to the next world by choice, same as the
   existing "run only ends by player choice" philosophy already in GDD §8
   for the run as a whole.

Design source: `redesign/processed/abiogenesis-objectives.md` in full.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] **Snapshot correction**: `Objective::Speciation` records which species
      have already speciated (or simply the count of `has_speciated`-style
      events, or a species-set snapshot) at activation time and only clears
      on a *new* speciation after that point, whose resulting species
      survives at least one full era (per the doc's "more substantial than
      a bare event" note) — not the current unconditional
      `world.has_speciated` read. Add the doc's special case: once at least
      one speciation has already happened in the world before `Speciation`
      activates, the objective narrows to a **specific target species**
      (deterministic, seed-derived selection among currently-eligible
      living species above a minimum-population floor, excluding species
      that have already speciated), with re-selection on the target's
      extinction. Objective text must name the species (existing
      name/icon plumbing — Catalog/HUD/genome bank), never a bare ID.
- [ ] **5 new `Objective` variants** added, each reading real per-world data
      to parametrize itself (mirrors `SurviveIn`'s existing "pick a zone
      that's actually present" and `TriggerBloom`'s "pick a seedable
      species" pattern in `worldgen.rs`) — not generic/fixed parameters:
  - `Homeostasis`: target species' mean energy stays within a band for N
    seasons.
  - `Tolerance`: target species survives in a high-toxicity zone (Swamp or
    Crater-adjacent) for N seasons — reuses `ZoneKind`/`species_present_in_zone`
    machinery, extend `ZoneKind` if Crater-adjacency needs a new variant.
  - `WildCoexistence`: a wild species (task 098) stays alive alongside at
    least one player-seeded species for N seasons.
  - `Rootedness`: a species stays alive specifically in a biome tied to an
    active terrain-conditioned trait (GDD §5.5) in that world.
  - `FirstConfirmation`: at least one matrix relation confirmed within the
    first N eras — reserved primarily for world 0 (mirrors `Coexistence`'s
    existing world-0 softening, GDD §9).
  - Objective text states only *what* it measures, never *why* that
    parameter was picked for this world (no family-bias or trait-sign
    leaks) — same rule already applied to trait naming.
- [ ] **Durations in seasons**: `Coexistence`/`SurviveIn` (and the 5 new
      sustained-condition types) express their duration as a season count
      internally, converted to ticks only where `evaluate_sustained`'s
      per-tick counter needs it (or `evaluate_sustained` itself is reworked
      to count seasons/eras where that's more natural — doc explicitly
      allows era-scale for event-matured types, case by case, not a
      mechanical find-replace). `ObjectiveConfig`'s `*_ticks_base` fields
      renamed/reworked to match; `sim_config.ron` kept in sync
      (`tests/config_ron_sync.rs`).
- [ ] **Victory as a flag**: `objectives.rs:511-513`'s
      `WorldOutcome::Cleared if params.objective.is_last()` arm no longer
      force-transitions `GameState`. New state (e.g. a `victory: bool` on
      `CurrentWorldOutcome` or a dedicated `WorldVictory` marker resource)
      set instead; the world keeps simulating under its existing
      failure conditions (extinction, era-budget exhaustion — both must
      still apply post-victory, per the doc's "same budget as before, the
      player just isn't kicked out"). A UI affordance (HUD badge/banner,
      not a blocking screen) surfaces the flag and offers a
      player-triggered "advance to next world" action — reusing
      `start_world`/`WorldResetParams` (`run_flow.rs`), the existing
      mechanism `screens.rs`'s forced transition already drives.
      `WorldResetParams`/`start_world` reset the new victory state on
      every (re)start.
- [ ] Explicit completion-effect question from the doc ("small narrative
      reinforcement anchored to real data" vs. "no side effect") is **not**
      decided by this task — doc leaves it open; default to no side effect
      unless a follow-up decision is made, and don't block this task on it.
- [ ] Tests: `Speciation`'s corrected snapshot behaviour (activates after an
      unrelated pre-activation speciation → does not clear immediately);
      target-species selection determinism and extinction-fallback
      reselection; each new objective type's evaluate logic against a
      hand-built `SimWorld` (mirrors existing `evaluate`/`evaluate_world`
      unit tests, no Bevy `App` needed); victory-flag world continues
      simulating (doesn't force a state transition) and still fails
      correctly on era-budget exhaustion after victory.
- [ ] GDD §8 updated to match: victory-as-flag noted `[DECIDED]` (already
      marked so in the design doc; sync into the GDD itself), 5 new
      objective types listed, snapshot-at-activation rule stated generally.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` | `Objective` enum, `evaluate`/`evaluate_world`/`evaluate_sustained`, `apply_tick_outcome` (victory-flag change lands at `~511-513`), `ZoneKind`. |
| `src/worldgen.rs` | `generate_objectives` (`~442`) — draws objective types/parameters from real per-world data; needs the 5 new types wired in, plus season-based duration generation. |
| `src/config.rs` | `ObjectiveConfig` (`~716`) — `coexistence_ticks_base`/`survive_in_ticks_base`/`objective_clear_energy_reward`; add config for the 5 new types. |
| `src/run_flow.rs` | `WorldResetParams`/`start_world` — reset victory-flag state on (re)start. |
| `src/screens.rs` | `GameState::WorldCleared` handling — becomes a player-triggered path instead of (or alongside) the automatic one; may need a new UI entry point rather than removing the screen outright. |
| `src/state.rs` | `GameState` enum — decide whether `WorldCleared` stays a distinct state reached only by player action, or a new state/resource represents "victory achieved, still playing." |
| `sim_config.ron` | Mirror `ObjectiveConfig` renames/additions. |
| `abiogenesis-gdd.md` | §8 — sync all four corrections. |

---

## 🧩 Technical Context

- **Current behavior**: `Objective::Speciation` clears on any
  `world.has_speciated == true`, regardless of when that speciation
  happened relative to activation — a real, if narrow, correctness bug
  once other objectives can run long enough before it activates. Duration
  fields are named/typed as raw ticks, generated as tick counts (verified
  season-aligned only via a config-sync test, not expressed as seasons).
  Clearing the sequence's last objective force-sets `GameState::WorldCleared`
  the same tick it clears, ending the world immediately — this is what
  makes Emersione unreachable (task 169 depends on this task landing first,
  or at least on the victory-flag piece of it).
- **Desired behavior**: all four corrections above.

---

## 🔨 Suggested Implementation

1. Victory-as-flag first (self-contained, unblocks the most, cheapest to
   verify in isolation): swap the forced `next_game_state.set(WorldCleared)`
   for a flag + player-triggered transition; keep `evaluate_world`'s
   failure checks running unconditionally afterward.
2. Speciation snapshot correction: add an activation-time snapshot field
   (which species have already speciated) to `CurrentObjective` or a
   sibling resource, populated when `Speciation` becomes current (both on
   initial activation and on `index += 1` advancing into it).
3. Target-species selection: a pure function taking `&SimWorld` + seed,
   returning a deterministically-chosen eligible `SpeciesId`, called once
   at activation and again on the current target's extinction.
4. Durations in seasons: rework `ObjectiveConfig` fields and
   `generate_objectives`'s `scale_severity` calls; keep `evaluate_sustained`
   tick-counting internally if simplest, converting only at the
   config/generation boundary — or convert `evaluate_sustained` itself if
   that reads cleaner; use judgement, note the choice in a comment.
5. 5 new objective types last — each is an incremental addition to the
   `Objective` enum/`evaluate`/`generate_objectives`, following the
   existing `SurviveIn`/`TriggerBloom` pattern precisely.

---

## ⚠️ Constraints and Caveats

- **No magic numbers**: every new threshold (Homeostasis's energy band,
  Tolerance's zone list, target-species minimum-viability floor) into
  config.
- Never reveal *why* a parameter was chosen for a given world in objective
  text (family-bias, trait-sign leaks) — matches the existing trait-naming
  rule.
- Keep `sim`/`world`/`config`/`objectives` free of `bevy::render`/
  `bevy_egui` deps; the victory-flag UI affordance belongs in `ui.rs`.
- Don't implement Emersione itself (task 169) or its lineage-chain
  detection — this task only removes the structural blocker.

---

## 🔗 Dependencies

- **Depends on**: 135 (season/era scale, already shipped), 137 (per-cell
  population for Homeostasis's mean-energy read), 145 (Tolerance's
  toxicity-axis Stress, so the objective has a deliberate tool to pursue
  it with — not a hard code dependency, but the objective is much less
  playable without it).
- **Blocks**: 169 (Emersione) needs the victory-flag correction to be
  reachable at all.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/154-objectives-activation-victory-flag.md)"$'\n\nExecute this task in the current project.'
```
