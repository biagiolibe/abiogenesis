# Task 003 — Domain types and `SimWorld` resource

> **ID**: `003`
> **Category**: Architecture
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Define the types that describe the simulated world — grid, cells, organisms, species, metabolisms — and the **`SimWorld`** resource that holds them, **including the seeded RNG**.

This is the foundation all subsequent tasks build on. The structural decision (`TECH_DESIGN.md` §3.1) is that **the grid is a `Resource` with dense arrays, not ECS entities**: Bevy entities will only serve rendering (task 006).

---

## 📋 Acceptance Criteria

- [ ] `SimWorld::new(seed, &SimConfig)` builds a `48×32` world.
- [ ] **Determinism**: two `SimWorld::new(42, &cfg)` produce identical state; different seeds produce different state. Covered by a test.
- [ ] The RNG is **kept inside `SimWorld`**, not created on the fly.
- [ ] A reusable **Moore neighborhood (8)** helper exists that handles borders correctly; unit test verifying 3 neighbors in a corner, 5 on an edge, 8 in the center.
- [ ] Double buffering set up: `SimWorld` can produce a snapshot and write to a next buffer (task 005 will use it).
- [ ] **No `use bevy::render` and no `bevy_egui`** in `src/world.rs`.
- [ ] `SimWorld` is constructible and usable **without a Bevy `App`** (verified by the fact that the tests don't create one).
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | Domain types, `SimWorld`, `WorldPlugin` |
| `src/config.rs` | `SimConfig`, already available (task 002) |

---

## 🧩 Technical Context

- **Current behavior**: `src/world.rs` is an empty stub; `SimConfig` exists.
- **Desired behavior**: `SimWorld` available as a resource, with an allocated grid and a seeded RNG.

### The species genome (GDD §5.3)

Each species is defined by:
- **Metabolism** — how it derives energy;
- **Preferred environmental range** — `temp_optimum` + `temp_tolerance`, with Gaussian fitness around the optimum;
- **Reproduction threshold**;
- **1 to 3 biochemical tags** — *the only thing that matters for interactions between species* (GDD §5.5).

Metabolisms and environmental ranges are **readable** by the player; tags are **opaque**.

### Metabolisms (GDD §5.4)

- `Photolithic` — derives energy from local `light`. Primary producer.
- `Predator` — derives energy from neighboring organisms.
- `Decomposer` — derives energy from residue.

All three must be defined **as a type** even though only the photolithic one is active in Phase 0: avoids a refactor in Phase 1.

### Determinism (GDD §5.7)

> The simulation is **deterministic** given the same seed: seeded RNG kept in the world state. Essential for debugging emergence, reproducing bugs, and sharing interesting seeds.

---

## 🔨 Suggested Implementation

1. **Base types**

   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum Metabolism {
       Photolithic,
       Predator,
       Decomposer,
   }

   /// Index into SimWorld::species. Kept small: species are few and never removed.
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub struct SpeciesId(pub u8);

   #[derive(Debug, Clone)]
   pub struct Species {
       pub metabolism: Metabolism,
       pub temp_optimum: f32,
       pub temp_tolerance: f32,
       pub repro_threshold: f32,
       pub tags: Vec<TagId>,   // 1..=3 (GDD 5.3)
   }

   #[derive(Debug, Clone, Copy)]
   pub struct Organism {
       pub species: SpeciesId,
       pub energy: f32,
   }

   #[derive(Debug, Clone, Copy, Default)]
   pub struct Cell {
       pub temperature: f32,
       pub light: f32,
       pub toxicity: f32,
       pub organism: Option<Organism>,
       /// Dead matter left behind, feeds decomposers (GDD 5.6 step 6).
       pub residue: f32,
   }
   ```

2. **`SimWorld`** — dense grid indexed by `y * width + x`:

   ```rust
   #[derive(Resource)]
   pub struct SimWorld {
       pub width: usize,
       pub height: usize,
       pub cells: Vec<Cell>,
       pub species: Vec<Species>,
       pub tick: u64,
       pub era: u32,
       pub seed: u64,
       rng: StdRng,          // seeded, lives in world state (GDD 5.7)
       scratch: Vec<Cell>,   // double buffer for the tick (TECH_DESIGN 6)
   }
   ```

   Use `rand`'s `StdRng::seed_from_u64`: it's reproducible across runs, unlike `SmallRng` on different platforms.

3. **Accessors** — `get(x, y)`, `get_mut(x, y)`, `index(x, y)`, and the RNG exposed only through `&mut self` so nobody can clone it.

4. **Moore neighborhood** — reusable helper, the easiest spot to get borders wrong:

   ```rust
   /// Moore neighbourhood (8 cells), clipped at the grid borders (GDD 5.1).
   pub fn moore_neighbours(&self, x: usize, y: usize) -> impl Iterator<Item = usize> + '_
   ```

   No wrap-around: the grid has real borders (a corner cell has 3 neighbors). The GDD doesn't call for toroidal topology.

5. **`WorldPlugin`** inserts `SimWorld` in `Startup`, reading `SimConfig`. The default seed can come from a fixed value in Phase 0 (interactive reseeding is task 007).

6. **Tests** in `src/world.rs`:
   - two worlds with the same seed are identical; with different seeds, they aren't;
   - Moore neighbor count: `(0,0)` → 3, edge → 5, center → 8.

---

## ⚠️ Constraints and Caveats

- **Invariant 1 (`TECH_DESIGN.md` §5)**: the RNG lives in `SimWorld`. No `rand::rng()` / `thread_rng` anywhere.
- **Invariant 2**: `src/world.rs` doesn't depend on rendering. The only Bevy import allowed is the one for `derive(Resource)`.
- **No `HashMap` for the grid**: dense arrays. A map's iteration order is one of the most common ways to lose determinism.
- Verify the `rand` API resolved by task 001: from `rand` 0.9 some names changed (`thread_rng` → `rng`, `gen` → `random`).
- **Single occupancy per cell** (GDD §5.1): `Option<Organism>`, never a collection.
- In Phase 0 available species reduce to just one, photolithic. The `species` registry is still a `Vec` from the start regardless.

---

## 🔗 Dependencies

- **Depends on**: 002
- **Blocks**: 004, 006

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/003-domain-simworld.md)"$'\n\nExecute this task in the current project.'
```
