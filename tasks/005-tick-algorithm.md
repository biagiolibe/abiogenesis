# Task 005 — Tick algorithm (Phase 0), pure and headless

> **ID**: `005`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Implement the simulation's atomic unit: **one tick**, following the 7 steps of GDD §5.6, limited to what Phase 0 requires (photolithic metabolism only, no interaction matrix).

This is the heart of the project. It must be **pure Rust, callable without the Bevy `App`**: this property is what makes determinism and balance tests (task 009) and final tuning possible.

---

## 📋 Acceptance Criteria

- [ ] `pub fn step(world: &mut SimWorld, config: &SimConfig)` exists, callable **without a Bevy `App`**.
- [ ] Implements the 7 steps of GDD §5.6, with `interaction_delta = 0` (no matrix in Phase 0).
- [ ] Uses **double buffering** (snapshot → next), as decided in `TECH_DESIGN.md` §6.
- [ ] Deterministic: same seed + same sequence of `step` calls ⇒ identical state.
- [ ] Contention over a birth cell is resolved in **deterministic index order**, never by iteration order.
- [ ] `world.tick` increments on every call.
- [ ] **Numeric verification** — tests reproducing the three GDD §5.9 scenarios (details below).
- [ ] `SimPlugin` exposes a system that invokes `step`; per-state activation is task 007.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `step()`, `SimPlugin` |
| `src/world.rs` | `SimWorld`, Moore neighborhood, double buffer (task 003) |
| `src/config.rs` | `SimConfig` (task 002) |

---

## 🧩 Technical Context

- **Current behavior**: world and environment exist, but nothing moves.
- **Desired behavior**: calling `step` makes organisms gain energy, die, and reproduce.

### GDD §5.6 — the 7 steps, for each occupied cell

1. **Environmental fitness:** `env_fit = gaussian(temperature, temp_optimum, temp_tolerance)` ∈ `[0,1]`.
2. **Metabolic gain** — Phase 0 has only photolithic: `gain = light * metabolism_gain * env_fit`.
3. **Hidden matrix effect** → `interaction_delta`. **In Phase 0 this is 0.**
4. **Costs:** `upkeep` + `crowding_penalty = crowd_factor * n_occupied_neighbors`.
5. **Energy update:** `energy += gain + interaction_delta − upkeep − crowding_penalty`.
6. **Death:** if `energy <= 0` → the organism dies, the cell frees up, leaves residue.
7. **Reproduction:** if `energy >= repro_threshold` and an empty neighbor exists → spawn a child in an empty neighbor (seeded random choice) with `repro_cost` energy subtracted from the parent.

The `env_fit` formula (GDD §5.9): `exp(−(temp − temp_opt)² / (2 · temp_tol²))`, with `temp_tol` (σ) defaulting to `0.15`.

### Expected numeric verification (GDD §5.9)

These are the numbers the tests must reproduce — **and they're also the definition of "correct"** for this task:

| Scenario | Expected |
|---|---|
| Isolated photolithic, `light ≈ 0.7`, `env_fit ≈ 1` | `gain ≈ 1.4`, net **`≈ +0.9`/tick** → grows |
| Same, with **7** occupied neighbors | `1.4 − 0.5 − 0.15·7` = net **`≈ −0.15`/tick** → stalls (carrying capacity) |
| Photolithic in a dark zone, `light = 0.2` | `gain = 0.4 < upkeep 0.5` → **doesn't survive** (light niche) |

*(The GDD cites "6–8 neighbors → ≈ −0.15": the exact `−0.15` value is obtained with 7 neighbors. With 8 the net is `−0.3`, with 6 it's `0.0`.)*

### Why double buffering

GDD §5.6 leaves the choice of processing order open. `TECH_DESIGN.md` §6 settles it on **double buffering**: read from the previous tick's immutable snapshot, write into the next buffer. No "born/acted this tick" guard to maintain, no dependency on visitation order, newborns don't act in the same tick **by construction**.

---

## 🔨 Suggested Implementation

1. **Structure**

   ```rust
   /// Advances the simulation by one tick (GDD 5.6).
   /// Pure: no Bevy App required, so determinism and balance can be tested headless.
   pub fn step(world: &mut SimWorld, config: &SimConfig) {
       // 1. snapshot = read side, next = write side
       // 2. for each occupied cell in index order: energy update, death, reproduction
       // 3. decay residues
       // 4. swap buffers, world.tick += 1
   }
   ```

2. **Environmental fitness**

   ```rust
   /// Gaussian environmental fitness around the species' thermal optimum (GDD 5.9).
   fn env_fit(temperature: f32, optimum: f32, tolerance: f32) -> f32 {
       let d = temperature - optimum;
       (-(d * d) / (2.0 * tolerance * tolerance)).exp()
   }
   ```

3. **Occupied neighbors** — read them **from the snapshot**, not from the write buffer: this is what makes the tick order-independent.

4. **Reproduction.** Collect empty neighbors *from the snapshot*, pick one with the world's RNG, then **verify it's still free in the write buffer**. If another parent has already occupied it this tick, the birth fails (the parent keeps its energy). Scanning cells in increasing index order makes the outcome deterministic.

   ```rust
   // Collect empty Moore neighbours from the snapshot, in index order.
   // Order matters: it is what makes the RNG draw reproducible.
   ```

5. **Death.** `energy <= 0.0` → empty the cell and add `residue_on_death` (`3.0`) to the cell's residue. Residue decays by `residue_decay` (`0.2`) per tick and must stay `>= 0`. It feeds no one in Phase 0 (decomposers are Phase 1), but it should already accumulate: task 006 makes it visible.

6. **Scan order.** Iterate `for idx in 0..cells.len()`. GDD §5.6 mentions shuffled iteration as an alternative: **don't use it**, double buffering makes it unnecessary and would reintroduce an order dependency.

7. **`SimPlugin`** — a system that calls `step`:

   ```rust
   fn advance_tick(mut world: ResMut<SimWorld>, config: Res<SimConfig>) {
       step(&mut world, &config);
   }
   ```

   Register it in `FixedUpdate` inside `SimSet::Advance`. **Per-state run conditions are task 007**: here it's fine for it to always run, or to be registered but inactive.

8. **Tests** in `src/sim.rs` — build a world, force `light`/`temperature` on the relevant cells, place an organism, and verify the energy delta after one `step` for the three scenarios in the table above. Use a tolerance (`(a - b).abs() < 1e-4`), not `==`, on `f32` values.

---

## ⚠️ Constraints and Caveats

- **Invariant 1**: no RNG other than `SimWorld`'s. Choosing the birth cell is the only random point in Phase 0 — it's where determinism breaks most easily.
- **Invariant 2**: `src/sim.rs` doesn't import `bevy::render` or `bevy_egui`. The `step` function must not touch any Bevy type.
- **Invariant 3**: no magic numbers. Every coefficient from `SimConfig`.
- **Invariant 4**: the adjacency effect will be **additive and linear** (Phase 1). Even though it's 0 here, leave the integration point as a sum, not a product.
- **Do not implement** predation, decomposition, the matrix, or mutation: those are Phase 1. The `match` on metabolism can have non-photolithic arms with `todo!()` or zero gain — but documented.
- Watch out for **residue with negative energy**: if `energy` drops below zero before death, the residue stays fixed at `residue_on_death`, not `residue + energy`.

---

## 🔗 Dependencies

- **Depends on**: 004
- **Blocks**: 007, 009

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/005-tick-algorithm.md)"$'\n\nExecute this task in the current project.'
```
