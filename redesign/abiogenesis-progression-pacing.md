# Abiogenesis — progression & pacing: reconciling "mondo vivo" with the per-world reset

Standalone design doc, raised directly by the user (2026-08-11): the
long-horizon ambitions of `abiogenesis-living-world.md` (zone discovery,
wild species) and `abiogenesis-evolution-xenotypes.md` (speciation) all
need sustained time to pay off — but the current core loop resets
everything (grid, species, matrix, notebook knowledge) the moment a
world's objective sequence clears. Any long-term ecosystem investment is
destroyed right as it starts to matter. This doc reconciles the two by
adding a long-term objective tier and a within-run energy economy, without
touching the fresh-matrix-per-world pillar.

## Current state (grounding)

- **Objectives are a sequence of 2-3 per world** (`Objective` enum,
  `objectives.rs:32-52`: `Coexistence`, `SurviveIn`, `TriggerBloom`;
  `CurrentObjective`, `:136-166`). Clearing a **non-final** objective
  already advances in place without resetting the world
  (`apply_tick_outcome`, `:403-467`) — the "short objective that doesn't
  reset the world" mechanism already exists structurally; it just grants
  no reward today, and the sequence is short. Clearing the **final**
  objective triggers `WorldCleared` → a full reset via
  `advance_to_next_world`/`start_world` (`run_flow.rs:68-145`,
  `*world = new_world`) — grid, species, matrix, notebook knowledge, all
  wiped.
- **`RunProgress`** (`run.rs:15-21`: `run_seed`, `world_index`,
  `world_seed`, `worlds_cleared`, `unlocks`) already persists **across
  worlds within a run** — it survives the per-world reset described above.
  **`MetaProgress::absorb`** (`run.rs:90-92`) already converts
  `worlds_cleared` into `bonus_available_species` for a *future run* — an
  existing precedent for "progress becomes an unlocked capability, not
  surviving world state," matching GDD §10's "unlock capabilities, not
  answers."
- **No currency accumulates today.** `ActionBudget` (`sim.rs:436-445`)
  resets to `point_budget_per_era = 3` every era, by design (GDD §6: "an
  era = one deliberate experiment"). A persistent energy currency would be
  genuinely new state, not a repurposing of `ActionBudget`.
- **Task 084** is blocked specifically on **cross-run** persistence (GDD
  §10, deferred — nothing survives a process restart). Irrelevant to a
  within-run-only design, which needs no new architectural dependency and
  doesn't reopen that decision.

## Decided in discussion

- **The world-reset pillar stays** — a fresh matrix per world is core to
  the game and isn't up for revision here. What changes is **how long a
  world lasts before that reset happens**, not whether it happens.
- **Surgical approach: reuse the existing objective machinery, don't build
  a parallel system.** Add a genuine **long-term objective tier** to the
  end of a world's objective sequence, tied to "mondo vivo"/evolution
  milestones (e.g. achieve a speciation event, sustain a population
  through many eras, confirm N matrix relations) — this is what actually
  triggers the world-clear reset now, replacing today's short 2-3-item
  sequence as the reset trigger. Existing short-term objectives keep their
  current in-place-advance behavior (`apply_tick_outcome`'s non-final
  path, unchanged) but now **grant a reward** (energy) on each clear,
  which they don't today.
- **Evolved-species persistence: within-run only.** A species born via
  speciation (`abiogenesis-evolution-xenotypes.md`) that's active when a
  world resets carries into the *next world of the same run* as an
  unlocked seedable option — mirroring `MetaProgress::absorb`'s existing
  shape, but scoped to `RunProgress` (which already survives world-to-world
  transitions) instead of `MetaProgress` (which would require the
  deferred cross-run persistence layer task 084 is blocked on). This was a
  deliberate scoping choice: it delivers the "your evolutionary investment
  survives the reset" goal without reopening that architectural question.
- **Energy: a new accumulating resource**, distinct from the
  per-era-resetting `ActionBudget`, living at the `RunProgress` level —
  survives world resets within a run, resets at run start/end like the
  rest of `RunProgress`. Earned from objective clears, both short- and
  long-term tier.
- **`Splice` stays available from world 0** — no change to existing
  onboarding tuning (tasks 082/083, the grace period). Energy instead
  unlocks a **more powerful tier** of `Splice` (e.g. more simultaneous
  edits per use, or a reduced action-point cost) rather than gating the
  action's existence outright — chosen specifically to avoid touching
  onboarding, which was already carefully tuned in a separate design pass.

## Open questions for task scoping

- **Concrete definition of the new long-term `Objective` variant(s)** —
  this doc's pacing model depends on content from `abiogenesis-living-world.md`
  (terrain discovery, wild species) and `abiogenesis-evolution-xenotypes.md`
  (speciation) that doesn't exist as code yet. **This doc is sequenced
  after those two get scoped, not independent of them.**
- How much energy each objective tier grants, and how strong `Splice`'s
  upgraded tier should feel — pure balance, not decided here.
- Whether the long-term objective's difficulty scales with `world_index`
  like today's existing difficulty ramp, or stays roughly constant while
  only short-term objectives ramp with world index as they do today.
- Exact integration point for carrying an evolved species into the next
  world's seedable roster — likely the same slot `bonus_available_species`
  already occupies in `WorldResetParams`/`start_world`, but needs
  confirming against that code at scoping time, not assumed here.
- Whether the existing short-term objective *sequence* itself needs
  restructuring (e.g. becoming a repeating/renewable loop instead of a
  fixed short list) now that it's a reward loop rather than only a gate to
  the next world — flagged, not resolved.

## Scope note

Not scoped into task files yet, and explicitly **blocked on** two sibling
docs landing first: `abiogenesis-living-world.md` and
`abiogenesis-evolution-xenotypes.md` need their own mechanics scoped before
this doc's long-term objective content can be concretely defined. The
energy-economy and `Splice`-upgrade pieces are more independent and could
plausibly be scoped earlier, but the headline goal (long worlds with room
for mondo-vivo/evolution to develop) can't be verified without that
content existing.
