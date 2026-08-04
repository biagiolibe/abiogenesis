# Task 035 — Run/world state foundation

> **ID**: `035`
> **Category**: Architecture
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Phase 3 planning session

---

## 🎯 Objective

Phase 3 ("The run", GDD §8-§10) introduces the concept of a multi-world run: success → next world, failure → end of run. This task lays the minimal foundation without which no other Phase 3 task can proceed:

1. New `GameState::{WorldCleared, Defeat}` variants (interstitial, not a real "Victory" — the GDD never describes a run victory, only "success → next world" ad libitum: the run is endless-until-failure).
2. `RunProgress` resource that tracks where we are in the run (world index, seed lineage, unlocks).
3. `EraCompleted` event, already documented in `TECH_DESIGN.md` §4 ("Emitted by `sim`, consumed by `ui`, run flow (Phase 3)") but never implemented — it's the hook through which later tasks (failure conditions, world transition) will observe the end of an era without duplicating logic inside `advance_tick`.

This is deliberately a "silent" task: it introduces types and an event, without wiring them to input/UI yet. `main.rs` must NOT be touched — the `Loading → Playing` bypass stays as it is until a main menu UI exists (future task 044); changing it now would boot the game into a state with no UI, breaking manual verification for all the intermediate Phase 3 tasks.

---

## 📋 Acceptance Criteria

- [x] `GameState` (`src/state.rs`) has two new variants `WorldCleared` and `Defeat`, alongside `Loading`, `MainMenu`, `Playing`.
- [x] New `RunProgress` resource with at least the fields: `run_seed: u64`, `world_index: u32`, `world_seed: u64`, `worlds_cleared: u32`, `unlocks: Unlocks` (where `Unlocks` is a minimal struct/enum, empty or nearly so — actual population is task 046, here we only need the field so the resource's shape doesn't have to be redone later). Implemented in a new `src/run.rs` module (exported from `lib.rs`), not in `world.rs`.
- [x] `RunProgress` is inserted as a resource at startup, via `RunPlugin::build` (`app.init_resource::<RunProgress>()`), registered in `main.rs`.
- [x] New `EraCompleted` event/`Message` (follows the existing `OrganismDied`/`SpeciesExtinct` pattern in `src/sim.rs`), emitted by `advance_tick` at the same point where it currently increments `world.era` and calls `next_state.set(EraState::Observing)`.
- [x] `EraCompleted` is registered as a `Message`/event in `SimPlugin` (`app.add_message::<EraCompleted>()`), as well as in the manual test that builds an `App` by hand in `sim.rs`.
- [x] `main.rs` introduces no new state transition: the `Loading → Playing` bypass remains unchanged — the only modification is registering `RunPlugin` in the plugin tuple (needed to insert the `RunProgress` resource at startup, as required by the task's suggested implementation).
- [x] No existing system yet reads `WorldCleared`/`Defeat`/`RunProgress`/`EraCompleted` to drive UI or input — they are only reachable/emitted, not consumed.
- [x] `cargo clippy -- -D warnings` clean.
- [x] `cargo test` green (68 tests total, no regressions).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/state.rs` | Addition of the `GameState::{WorldCleared, Defeat}` variants. |
| `src/sim.rs` | `advance_tick` (lines ~427-450): emission of `EraCompleted` at the same point where `world.era` is currently incremented; event type definition alongside `OrganismDied`/`SpeciesExtinct` (lines ~20-25); registration in `SimPlugin`. |
| `src/run.rs` (new, to be assessed) | Definition of `RunProgress` and `Unlocks`, if it's preferable not to overload `world.rs`. |
| `src/main.rs` | **Read-only** — verify that the `OnEnter(GameState::Loading) → enter_playing` bypass remains unchanged; do not add the registration of new plugins/resources here unless strictly necessary (it's fine to do it from a new `RunPlugin`, but without touching state logic). |

---

## 🧩 Technical Context

**Current state (`src/state.rs`, 32 lines, read in full):**
```rust
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
}

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Playing)]
pub enum EraState {
    Planning,
    #[default]
    Observing,
    Advancing,
}
```
`GameState::MainMenu` and `EraState::Planning` are currently unreachable (`#![allow(dead_code)]` at the top of the file) — explicitly commented as "become reachable in later phases". The new `WorldCleared`/`Defeat` variants will be in the same state: declared but not yet reached by any transition, until tasks 041/045 wire them up.

**Current `advance_tick` (`src/sim.rs`, lines ~427-450):**
```rust
fn advance_tick(
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    mut budget: ResMut<ActionBudget>,
    mut died: MessageWriter<OrganismDied>,
    mut extinct: MessageWriter<SpeciesExtinct>,
    mut adjacencies: MessageWriter<AdjacencyObserved>,
) {
    if progress.remaining() == 0 { return; }
    let events = step(&mut world, &config);
    died.write_batch(events.deaths);
    extinct.write_batch(events.extinctions);
    adjacencies.write_batch(events.adjacencies);
    progress.remaining -= 1;
    if progress.remaining() == 0 {
        world.era += 1;
        budget.refill(config.time.point_budget_per_era);
        next_state.set(EraState::Observing);
    }
}
```
`EraCompleted` should be written in the same `if progress.remaining() == 0` branch (after `world.era += 1`), as a new `mut era_completed: MessageWriter<EraCompleted>` parameter. Minimal event content: `struct EraCompleted { pub era: u32 }` (the era just concluded), sufficient for future consumers (failure conditions will read `world.era`/`RunProgress` directly from the resource, no other payload is needed here).

**`SpeciesExtinct`/`OrganismDied` as the reference pattern** (`src/sim.rs`, lines ~20-25):
```rust
#[derive(Message, Debug, Clone, Copy)]
pub struct SpeciesExtinct { pub species: SpeciesId }
```
`EraCompleted` follows the same shape (`#[derive(Message, ...)]`).

**`TECH_DESIGN.md` §4** already documents the event table with `EraCompleted` assigned to "run flow (Phase 3)" — this task makes it real, it does not introduce a concept new to the documented architecture.

- **Current behavior**: there is no concept of a "run" or "current world within a sequence" — the game generates a single `SimWorld` at startup (fixed seed `42`) and it can only be manually reseeded with the `r` key. There's no way to know "how many worlds has the player cleared" or "what is the seed of the current run".
- **Desired behavior after this task**: the types exist (`GameState::{WorldCleared, Defeat}`, `RunProgress`, `EraCompleted`) on which later tasks will build failure conditions, procedural worldgen, and world transition — but the game behaves exactly as it does today from the player's point of view (no UI, no newly reachable transition).

---

## 🔨 Suggested Implementation

1. In `src/state.rs`, add `WorldCleared` and `Defeat` to `GameState`. Update any exhaustive `match`es elsewhere in the code that list `GameState` variants (`cargo build` will flag them as compile errors if they exist — this is the most reliable way to find them all).
2. Decide where `RunProgress` lives: if the project prefers a new dedicated module (recommended, since tasks 044-046 will extend it with transition/meta-progression logic), create `src/run.rs` with:
   ```rust
   #[derive(Resource, Debug, Clone, Default)]
   pub struct RunProgress {
       pub run_seed: u64,
       pub world_index: u32,
       pub world_seed: u64,
       pub worlds_cleared: u32,
       pub unlocks: Unlocks,
   }

   #[derive(Debug, Clone, Default)]
   pub struct Unlocks; // populated by task 046
   ```
   A dedicated `Plugin` isn't needed yet if there's no system logic to register — `app.init_resource::<RunProgress>()` called from `WorldPlugin` or a new minimal `RunPlugin` in `main.rs` can suffice (in that case it's the only line `main.rs` gains, without touching `Loading → Playing` logic).
3. In `src/sim.rs`: add `EraCompleted` alongside `SpeciesExtinct`, add the `MessageWriter<EraCompleted>` parameter to `advance_tick`, emit it right after `world.era += 1`. Register the event in `SimPlugin::build` (`app.add_message::<EraCompleted>()`, same API used for existing events — verify the exact method name by checking how `OrganismDied`/`SpeciesExtinct` are registered in the same file).
4. Run `cargo build`, `cargo clippy -- -D warnings`, `cargo test` and fix any non-exhaustive `match` or unused-variant warning (`#[allow(dead_code)]` already present in `state.rs` covers the new variants as long as they remain unreachable).

```rust
// EraCompleted example, alongside SpeciesExtinct in src/sim.rs
#[derive(Message, Debug, Clone, Copy)]
pub struct EraCompleted {
    pub era: u32,
}
```

---

## ⚠️ Constraints and Caveats

- **Determinism (TECH_DESIGN.md §5, invariant 1)**: `RunProgress::run_seed`/`world_seed` are data, not generated here — this task does not yet introduce a seed generator (it arrives with the main menu, task 044). If a default value for `RunProgress::default()` is needed, use `0`, not a clock read.
- **No magic numbers outside `SimConfig`**: this task introduces no new numeric coefficients (the era budget already lives in `SimConfig::time`), so it doesn't touch `config.rs`.
- **`sim`/`world`/`config` stay headless** (invariant 2): `EraCompleted` and `RunProgress` must not depend on `bevy::render`/`bevy_egui`.
- **Don't anticipate later tasks**: don't implement the total-extinction check here, the consumption of `EraCompleted` for game over, or the main menu UI — those are tasks 040/041/044/045. This task stops at defining the types and emitting the event.
- **Style**: follow `TECH_DESIGN.md` conventions and the existing event/resource pattern in the rest of the codebase.

---

## 🔗 Dependencies

- **Depends on**: none (first task of Phase 3).
- **Blocks**: 040 (objectives — reads `RunProgress`/can react to `EraCompleted`), 041 (failure conditions — consumes `GameState::Defeat`), 044 (main menu — initializes `RunProgress.run_seed`), 045 (world transition — consumes `GameState::WorldCleared`/`Defeat` and updates `RunProgress`).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/035-run-world-state-foundation.md)"$'\n\nExecute this task in the current project.'
```
