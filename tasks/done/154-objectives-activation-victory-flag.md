# Task 154 (154a) — Objectives correctness pass: victory as a flag, Speciation snapshot, immediate re-check

> **ID**: `154`
> **Category**: Feature / Bug fix
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~3-4h originally scoped; ~1.5h actually delivered as 154a (split per advisor review, 2026-08-29)
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## ✅ Split note (2026-08-29)

Split into **154a** (this file — delivered) and **[178](178-objectives-tuning-new-types-durations.md)**
(remaining scope, not started): the correctness-critical half (victory as a
flag, the `Speciation` activation-snapshot bug, and a related bug the first
playtest surfaced — `evaluate_current_objective` never running during
`EraState::Reveal`, so a `Speciation` clear from the reveal card wasn't
observed until the *next* season advance, `playtest_outcome.md` gameplay
#17) landed here. The additive half (5 new objective types, durations
expressed in seasons) and the `Speciation` snapshot's full
"narrows to a specific target species" behavior (implemented here as a
simpler count-based snapshot instead — see Delivered-vs-deferred below) —
plus a genuinely new finding, the `Coexistence` objective's missing
population floor (`playtest_outcome.md` issue I.6, **not** actually covered
by this task's original scope, corrected in `tasks/QUEUE.md`) — moved to
178.

### Delivered vs. deferred on AC item 1 (Speciation snapshot)

Delivered: `Objective::Speciation` now clears only on a speciation that
happens *after* it becomes the current objective (tracked via
`SimWorld::species_parent.len()`, snapshotted into
`ObjectiveProgress::consecutive_ticks` at activation) — the literal
correctness bug (an unrelated pre-activation speciation instantly clearing
it) is fixed.

Deferred to 178: the doc's fuller special case — narrowing to a specific,
named **target species** once a speciation has already happened before
activation, with deterministic seed-derived selection, a minimum-population
floor, exclusion of already-speciated species, re-selection on the target's
extinction, and naming it in objective text — was not implemented. The
simpler "any new speciation after activation clears it" behavior is
correct but less informative than the doc's target-species design.

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

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] **Snapshot correction (simpler variant — see split note above)**:
      `Objective::Speciation` now clears only on a speciation after
      activation, via `SimWorld::species_parent.len()` snapshotted into
      `ObjectiveProgress::consecutive_ticks`. The target-species-narrowing
      special case is **not** implemented — moved to 178.
- [x] **Playtest-surfaced timing bug, added to this task's scope**:
      `evaluate_current_objective` also now runs once on
      `OnEnter(EraState::Reveal)` (after `build_era_reveal`), so a
      `Speciation` clear is observed the same tick the reveal card shows it,
      not one season-advance later (`playtest_outcome.md` gameplay #17).
- [ ] **5 new `Objective` variants** — moved to 178. Each reading real per-world data
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
- [ ] **Durations in seasons** — moved to 178.
- [x] **Victory as a flag**: `objectives.rs`'s
      `WorldOutcome::Cleared if params.objective.is_last()` arm no longer
      force-transitions `GameState` — it sets `WorldVictory` (a dedicated
      marker resource) instead. The world keeps simulating: era-budget
      exhaustion is explicitly re-derived post-victory in
      `apply_tick_outcome` (since `evaluate_world` itself stops re-checking
      it once `Cleared` is sticky), and extinction was already unconditional
      at the top of `evaluate_world` regardless. A UI affordance
      (`screens::victory_banner_ui`, a small non-blocking `egui::Area`, not
      `world_cleared_screen_ui`'s full interstitial) surfaces the flag and
      offers a player-triggered "advance to next world" action reusing
      `run_flow::advance_to_next_world`/`WorldResetParams`.
      `WorldResetParams`/`start_world` reset `WorldVictory` on every
      (re)start.
- [ ] Explicit completion-effect question from the doc — still open, no
      side effect implemented, unchanged from the original scope.
- [x] Tests (for the delivered scope): `Speciation`'s corrected snapshot
      behaviour (activates after an unrelated pre-activation speciation →
      does not clear immediately, then does clear on a post-activation one
      — `objectives.rs`); victory-flag world continues simulating and still
      fails correctly on era-budget exhaustion after victory
      (`era_budget_exhaustion_still_fails_the_world_after_victory`).
      Target-species selection tests and new-objective-type tests moved to
      178 along with the features themselves.
- [ ] GDD §8 sync — moved to 178 (bundle with the new types/durations so
      the doc is updated once, not twice).

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
