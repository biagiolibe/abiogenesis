# Technical Design Document — Abiogenesis

This document describes the technical architecture and implementation choices of the project.
**Game design** lives in [`abiogenesis-gdd.md`](abiogenesis-gdd.md) (v0.4) and is the source of truth: it is not duplicated here, only referenced.

---

## 1. Tech Stack

- **Language**: Rust (2021 Edition)
- **Toolchain**: **1.97.1**, pinned in `rust-toolchain.toml`
  *Constraint:* Bevy 0.19 requires Rust ≥ 1.95.0.
- **Engine**: **Bevy 0.19.0**
- **UI**: **`bevy_egui` 0.41.1** (egui 0.35.0)
- **RNG**: **`rand` 0.10.2**, with an explicit seed kept in the world state
- **Physics**: none (grid-based simulation, not continuous)

> Versions resolved by `cargo add` in task 001 (see `Cargo.lock`).
> Note: the `rand` 0.10 API differs from 0.8 (`thread_rng` → `rng`, `gen` → `random`, renamed traits) — keep this in mind for task 003 when initializing the RNG in `SimWorld`.

---

## 2. Game States (`GameState`)

```
Loading  →  MainMenu  →  Playing
                            │
                            └── EraState (sub-state)
                                Planning → Advancing → Observing ─┐
                                    ▲                             │
                                    └─────────────────────────────┘
```

- `Loading`: resource initialization and world generation.
- `MainMenu`: seed selection and run start. *(Stub in Phase 0, real in Phase 3.)*
- `Playing`: main game loop.

The **`EraState`** sub-state maps 1:1 onto the cycle described in GDD §16.4 — and it's this correspondence that makes it the backbone of scheduling:

| State | GDD §16.4 | What happens |
|---|---|---|
| `Planning` | PLAN | The player queues actions within budget. Simulation is paused. |
| `Advancing` | ADVANCE ERA | `ERA_TICKS` ticks advance **animated one by one**. Game input is ignored. |
| `Observing` | OBSERVE & RECORD | The player reads the result and the notebook. Simulation is paused. |

In Phase 0 only `Advancing` and `Observing` exist; `Planning` becomes meaningful in Phase 2, with actions.

---

## 3. ECS Architecture & Modules

### 3.1 The structural decision: the grid as a `Resource`

**The simulation is not modeled in ECS. It lives in a `SimWorld` `Resource` with dense, double-buffered arrays. Bevy entities exist only for rendering** — one sprite per cell, synced read-only.

The reason is the **determinism** required by GDD §5.7, which here is not a luxury but a functional requirement: it's needed to debug emergence, to reproduce bugs, and to make tuning in §5.8 repeatable. Parallel ECS query iteration is the fastest way to lose it. A dense grid iterated in index order guarantees it by construction, and on top of that:

- the tick algorithm (GDD §5.6) is a **sweep over a lattice with a Moore neighborhood**: accessed by index, not by entity;
- the logic stays **pure Rust**, runnable **without the Bevy `App`** → determinism and balance tests run headless and fast;
- no overhead from 1536 entities mutated every tick.

Bevy provides what it's good at: scheduling, states, plugins, input, window, rendering, UI.

### 3.2 Plugin Structure

Each module has its own `Plugin` to encapsulate its systems.

| Plugin | Module | Responsibility |
|---|---|---|
| `ConfigPlugin` | `config` | `SimConfig` resource: all GDD §5.9 coefficients in one place |
| `WorldPlugin` | `world` | `SimWorld` resource: grid, species registry, seeded RNG, tick/era counters; world generation |
| `SimPlugin` | `sim` | Tick advancement; invokes the pure `sim::step` logic |
| `GridRenderPlugin` | `render` | Grid sprites, 2D camera, state → color synchronization |
| `UiPlugin` | `ui` | `bevy_egui` panels: HUD and (Phase 2) notebook |
| `InputPlugin` | `input` | Keyboard/mouse → game intents |

Modules planned for later phases: `notebook` (Phase 2), `actions` (Phase 2), `worldgen` (Phase 3).

### 3.3 System Ordering (`SystemSets`)

The guaranteed execution order is:

`SimSet::Advance` → `SimSet::Sync` → `SimSet::Ui`

- **`Advance`** — advances the simulation. Runs in `FixedUpdate`, gated by a run condition on `EraState::Advancing`.
- **`Sync`** — reads `SimWorld` and updates sprite colors. **Read-only on simulation state.**
- **`Ui`** — egui panels. Reads everything, writes only intents.

### 3.4 Time Model

GDD §4 requires the era to be a block of `ERA_TICKS = 25` ticks **animated one by one**, with deliberately step-wise control.

Implementation: the advancement system runs in **`FixedUpdate`**, with a configurable timestep (= animation speed, not logic speed), active only under `EraState::Advancing`. An `EraProgress` resource counts remaining ticks; at zero, transition to `Observing`.

The single-tick key invokes `sim::step` exactly once, without going through `Advancing`.

---

## 4. Development Conventions

### Event Handling

Use Bevy events to decouple modules. Defined from Phase 0 as an integration point, even if the consumer arrives later:

| Event | Emitted by | Consumed by |
|---|---|---|
| `TickCompleted` | `sim` | `ui` (HUD), `notebook` (Phase 2) |
| `EraCompleted` | `sim` | `ui`, run flow (Phase 3) |
| `OrganismDied` | `sim` | `notebook` (Phase 2) |
| `SpeciesExtinct` | `sim` | `notebook`, failure conditions (Phase 3) |

These are the foundation of the **observation log** from GDD §7: the notebook is built by consuming events, not by inspecting the grid.

### FixedUpdate vs Update

- **`FixedUpdate`**: simulation advancement (`SimSet::Advance`).
- **`Update`**: rendering, UI, input.

### Asset Pipeline

No art assets: cells are colored square sprites (GDD, pillar 3). No need for a centralized `GameAssets` in Phase 0.

**Hot-reloadable config:** GDD §5.6 asks for coefficients *"ideally hot-reloadable"* — with `bevy_asset` this is nearly free. In Phase 0, `SimConfig` is built from constants in `config.rs`; during the tuning phase it migrates to a RON asset with hot-reload. The structure should be set up now (a single `Resource`, read and never duplicated), the implementation not.

### Style

- **Code and comments in English** (GDD §12); Meridian and GDD documents in English.
- `cargo fmt` and `cargo clippy -- -D warnings` clean before closing a task.
- Unit tests next to the code; determinism and balance tests in `tests/`.

---

## 5. Architectural Invariants

Rules no task may violate. If a task seems to require it, the task is wrong.

1. **Determinism** (GDD §5.7) — same seed ⇒ same story, always.
   - The RNG lives in `SimWorld`. No `rand::rng()` / `thread_rng` in the logic.
   - No iteration over `HashMap`/`HashSet` at points that affect the simulation.
   - No system clock, no Bevy `Time` inside the tick logic.
   - **No parallel queries inside the simulation.**
2. **`sim` doesn't depend on rendering** — no `use bevy::render` / `bevy_egui` in the `sim`, `world`, `config` modules. The simulation must run headless: it's the prerequisite for tests and tuning.
3. **Centralized coefficients** (GDD §5.6) — no magic numbers scattered through the code. Everything in `SimConfig`.
4. **Additive and linear matrix effect** (GDD §5.6) — this is a *design* choice, not a tuning one: additivity is what makes the matrix **deducible** by the player. It must not be "upgraded" to multiplicative.

---

## 6. Core Mechanics (Details)

The bulk is already specified in the GDD and **must not be duplicated here**:

| Mechanic | Reference |
|---|---|
| Tick algorithm (7 steps) | GDD §5.6 |
| Baseline numeric constants | GDD §5.9 |
| Anti-degeneration (cyclicity, niches, carrying capacity) | GDD §5.8 |
| Tags and hidden matrix | GDD §5.5 |
| Notebook confirmation model | GDD §7 |

Only what the GDD leaves open to implementation lives here.

### Tick Processing Order

GDD §5.6 marks this point as `[PROPOSED]`, leaving the choice to implementation between shuffled iteration with a "born/acted this tick" guard, and double buffering.

**Decided: double buffering** (snapshot → next). Read from the previous tick's immutable snapshot and write into the next buffer; the two swap at the end of the tick.

Reason: it's the simplest path to determinism (invariant 1). No guard to maintain, no dependency on visitation order, newborns cannot act in the same tick by construction. Costs a second grid buffer — irrelevant at 48×32.

Note the **reproduction** edge case: two parents may want to occupy the same empty cell in the same tick. Resolution must happen in deterministic index order (first arrival in scan order), never left to iteration order.

### Shared resource drain (predation, decomposition)

Predator and decomposer metabolisms (Phase 1, tasks 014-015) both draw from a resource that lives in a *different* cell than the organism consuming it — occupied neighbours' energy for predation, residue in the cell and its neighbours for decomposition. Writing directly into another cell's `scratch` entry while iterating would make the outcome depend on scan order (invariant 1), the same hazard double buffering already solves for the organism's own energy.

**Decided:** both mechanics compute a same-sized accumulator array (e.g. `predation_loss: Vec<f32>`) from the immutable snapshot (`world.cells`) in a **pre-pass**, before the main per-cell loop — the same shape as the existing residue-decay pre-pass. The main loop then applies the accumulated gain/loss as an extra term in the energy update, exactly like `interaction_delta`. No cell is ever written to from outside its own iteration; contention over a shared resource (two predators sharing a prey, two decomposers sharing a residue pool) is resolved by the pre-pass itself, deterministically, independent of the main loop's order.

Two differences for decomposition (task 015), noted because predation didn't need them:

- **Source scope**: a decomposer draws from its *own* cell plus its Moore neighbours (predation only ever draws from neighbours — a predator can't eat itself).
- **Decay/extraction order**: residue decay already runs as its own pre-pass before this one. The decomposer pre-pass reads `world.scratch`'s residue (post-decay), so the two compose — decay first, extraction second — rather than the extraction pre-pass overwriting the decay pre-pass's work. The extraction accumulator is applied with a final `.max(0.0)` clamp, since two decomposers competing for the same residue each size their own draw against the same undrained snapshot and can together overdraw it; residue must never go negative regardless.

---

*Last revised: 2026-08-03*
