# Task 045 — World-cleared/defeat screens + world transition

> **ID**: `045`
> **Category**: Feature / UI
> **Priority**: 🔴 P1
> **Estimate**: ~3h (convergence task — integrates nearly all previous Phase 3 tasks)
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Phase 3 planning session

---

## 🎯 Objective

This is Phase 3's convergence task: it connects worldgen (038/039/042), objectives/failure (040/041), state/run (035), and the main menu (044) into a complete loop. It introduces the interstitial screens `GameState::WorldCleared` (success → next world, more tags, nastier matrix, more hostile environment — GDD §8) and `GameState::Defeat` (end of run, return to the main menu).

The central technical point is to extract `reseed_world` (`src/input.rs`, lines ~107-147, the code behind the `r` key) into a shared `start_world(&mut World, world_index, seed)` function that resets **everything** the `r` key resets today — `MatrixKnowledge`, `ObservationLog`, `ActionBudget`, `SelectedSpecies`, `SpliceDraft`, `PlayerPlacedCells` — **plus** `ObjectiveProgress` (task 040) and `world.era = 0`, applying the new world's `WorldParams`/objective (038/042). Both the `r` key and the `WorldCleared`/new-run transition call it: **a single source of truth for the reset**, no duplicated logic between the two paths.

---

## 📋 Acceptance Criteria

- [ ] Shared function `start_world(...)` (indicative name, adapt to code style) that performs the full reset listed above, reused both by the `r` key (`input.rs`) and by this task's world transition.
- [ ] `GameState::WorldCleared` interstitial: shown when `evaluate` (task 040) produces `Cleared`; increments `RunProgress.world_index` and `RunProgress.worlds_cleared`; generates the next world via `start_world` with `WorldParams(world_index + 1)`'s parameters (037/038); an explicit action (or a timer/key) leads to `GameState::Playing` on the new world.
- [ ] `GameState::Defeat` interstitial: shown when `evaluate`/the failure conditions (task 041) produce `Failed`; returns to `GameState::MainMenu` (not to `Playing`) — the run is over, a new run requires going through the menu.
- [ ] Both screens have text in `src/text.rs` (new section), consistent with task 034.
- [ ] No duplication: the `r` key and the world transition call the same reset function — verifiable by reading the diff, not two parallel implementations.
- [ ] **End-to-end criterion**: a run started with a pinned `run_seed` (task 044) reproduces the same sequence of worlds (active tags, matrix, environment, objective) over at least 2 consecutive transitions — verifiable with a test that advances the run programmatically for two world-cleared cycles.
- [ ] `cargo clippy -- -D warnings` clean, `cargo test` green.
- [ ] Manual verification: `cargo run`, start a run, clear a world (or force it for manual testing), observe the interstitial, observe the next world with difficulty parameters consistent with `world_index=1`; fail a world, observe the defeat screen, return to the menu.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `reseed_world` (lines ~107-147) — extracted into the shared `start_world` function, the `r` key updated to call it. |
| `src/state.rs` | Actual use of the `WorldCleared`/`Defeat` variants (already declared by task 035). |
| `src/run.rs` / `src/menu.rs` (or a new dedicated module) | World-transition logic, UI of the two screens. |
| `src/text.rs` | New sections for the two screens' text. |

---

## 🧩 Technical Context

**`reseed_world`** (`src/input.rs`, lines ~107-147, `r` key): rebuilds `SimWorld` from a new seed (taken from `world.next_seed()`, never from the system clock), then explicitly resets: `MatrixKnowledge`, `ObservationLog` (notebook), `ActionBudget`, `SelectedSpecies`, `SpliceDraft`, `PlayerPlacedCells`. This is the closest concrete precedent for "what `start_world` must do" — this task's function is a generalization of this code, not a rewrite from scratch.

- **Current behavior**: the only way to get a "new world" is the `r` key, which always restarts from the same difficulty parameters (the concept of "next world in a run" doesn't exist before this task).
- **Desired behavior**: clearing a world's objective automatically leads (behind an explicit player interaction) to the next, harder world; failing returns to the main menu, ending the run. The `r` key keeps working as a "manual reseed" but through the same reset mechanism, not a copy.

⚠️ **Latent bug discovered during task 038, to be resolved here**: `MatrixKnowledge::new(config.tags.active_tags_early as usize, ...)` is called with the fixed constant `active_tags_early` (5) in two places — `notebook.rs::NotebookPlugin::build` and `input.rs::reseed_world`. Since task 038 made `active_tag_count` variable with `world_index` (later worlds reach 8 tags), a `MatrixKnowledge` sized to 5 would go out of bounds (`record`/`evidence` compute `exerter.0 * size + receiver.0` into a `size*size`-element vector) as soon as a world with more than 5 active tags is generated — a guaranteed panic. When this task introduces `start_world`, the `MatrixKnowledge::new` call inside it **must** use `world.active_tags.len()` (the newly generated world's actual size), not the config constant. Also check `NotebookPlugin::build`: if the first world can already have more than 5 tags (depends on how 044 initializes `world_index`), it needs to be fixed there too.

---

## 🔨 Suggested Implementation

1. Read `reseed_world` in `input.rs` in full before extracting it.
2. Define `start_world(world: &mut SimWorld, ..., world_index: u32, seed: u64, config: &SimConfig)` (indicative signature) that: rebuilds `SimWorld` from the seed and `WorldParams(world_index)` (038), resets the listed resources, applies the generated objective (042), resets `world.era = 0`.
3. Update the `r` key in `input.rs` to call `start_world` with the same current `world_index` (reseeding the same world, not advancing) — verify that the `r` key's visible behavior stays as expected (regenerates the current world, doesn't advance the run).
4. Implement the `WorldCleared` transition: a system that observes the `Cleared` outcome (from 040/041), increments `RunProgress`, calls `start_world` with `world_index + 1`, transitions the state.
5. Implement the `Defeat` transition: a system that observes the `Failed` outcome, transitions to `GameState::Defeat`, and from there (on interaction) to `GameState::MainMenu`.
6. Write the end-to-end reproducibility test over 2 transitions.
7. Full manual verification.

---

## ⚠️ Constraints and Caveats

- **No duplicated reset**: this is the central constraint of this task — if the `r` key and the world transition end up with two similar but distinct implementations, the task isn't complete.
- **End-to-end determinism**: the acceptance criterion on reproducibility over 2 transitions is the most important test of all of Phase 3 — if it fails, there's probably a source of non-determinism introduced in one of the earlier tasks (external RNG, `HashMap` iteration, etc.), to investigate before considering this task closed.
- **`Defeat` doesn't return to `Playing`**: it returns to `MainMenu` — the run is concluded, it's not a game over you can "continue".

---

## 🔗 Dependencies

- **Depends on**: 035, 038, 039, 040, 041, 042, 044.
- **Blocks**: 046 (meta-progression hooks into the run transition introduced here).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/045-world-transition-defeat-screens.md)"$'\n\nExecute this task in the current project.'
```
