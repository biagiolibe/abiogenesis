# Task 014 — Predator metabolism

> **ID**: `014`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Implement the `Predator` metabolism (GDD §5.4, §5.6 step 2): it drains energy from neighbouring organisms instead of photosynthesizing. This is the first mechanic that needs one organism's tick to affect a *different* cell's energy — a new architectural pattern that must stay order-independent (`TECH_DESIGN.md` §5 invariant 1).

---

## 📋 Acceptance Criteria

- [ ] `Metabolism::Predator` organisms gain `min(predator_drain_cap, total energy available in occupied Moore neighbours) * env_fit` per tick, instead of the photolithic light-based gain.
- [ ] The drained amount is **subtracted from the prey's own energy update this same tick** — predation is a real transfer, not a free bonus.
- [ ] The transfer is computed via a **separate accumulation pass over the snapshot**, before the main per-cell loop (see Technical Context) — never a direct write from one cell's processing into another cell's `scratch` entry.
- [ ] A predator with no prey neighbours collapses: starting at `seed_energy = 5.0`, upkeep `predator_upkeep = 0.7`, it dies after ⌈5.0 / 0.7⌉ = 8 ticks with **no gain at all** (matches GDD §5.9's "collapses in ~7 ticks" quick-check, rounding to the nearest tick).
- [ ] A predator with abundant adjacent prey nets positive energy and can reproduce.
- [ ] Determinism preserved: two predators sharing a prey neighbour split it the same way regardless of scan order.
- [ ] Existing Phase 0/1 tests unaffected (no photolithic-only test exercises `Metabolism::Predator`).
- [ ] `TECH_DESIGN.md` gains a short note documenting the "shared resource drain pass" pattern (see below) — this is a resolved architectural decision, same category as the existing "Tick Processing Order" section.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `step()` — new pre-pass, `Metabolism::Predator` gain arm |
| `TECH_DESIGN.md` | New subsection under §6, documenting the drain-pass pattern |

---

## 🧩 Technical Context

### Why a pre-pass, not a direct write

`sim::step` already establishes the pattern for cross-cell-safe mutation: residue decay runs as its own loop over `scratch` **before** the main per-organism loop, and the main loop itself only ever reads neighbours from the immutable `world.cells` snapshot, writing exclusively to its own `scratch[idx]`. Predation breaks the "only touch your own cell" rule unless it's modeled the same way: compute all predation transfers from the snapshot first, into a same-sized accumulator array, then have the main loop apply `predation_gain[idx]` / `predation_loss[idx]` as additional terms in the existing energy-update formula. This keeps the guarantee that iteration order never affects the outcome — the alternative (a predator writing directly into a prey's `scratch` entry while iterating) would make the result depend on which cell processes first.

### Suggested transfer rule

Keep it simple and symmetric with the existing carrying-capacity formula: a predator's total draw is `drain = (predator_drain_cap * env_fit).min(sum of prey neighbours' energy)`, split **evenly** across its occupied prey neighbours (any occupied neighbour counts as "prey" in Phase 1 — species-specific predation targeting isn't in the GDD baseline). Document this even-split choice inline; it's a coefficient-adjacent decision, not one the GDD pins down.

---

## 🔨 Suggested Implementation

1. Before the main `for idx in 0..world.cells.len()` loop, add:

   ```rust
   let mut predation_gain = vec![0.0f32; world.cells.len()];
   let mut predation_loss = vec![0.0f32; world.cells.len()];
   for idx in 0..world.cells.len() {
       let Some(organism) = world.cells[idx].organism else { continue };
       let species = &world.species[organism.species.0 as usize];
       if species.metabolism != Metabolism::Predator { continue; }
       let (x, y) = (idx % world.width, idx / world.width);
       let prey: Vec<usize> = world.moore_neighbours(x, y)
           .filter(|&n| world.cells[n].organism.is_some())
           .collect();
       if prey.is_empty() { continue; }
       let fit = env_fit(world.cells[idx].temperature, species.temp_optimum, species.temp_tolerance);
       let available: f32 = prey.iter().map(|&n| world.cells[n].organism.unwrap().energy).sum();
       let drawn = (energy.predator_drain_cap * fit).min(available);
       predation_gain[idx] = drawn;
       let share = drawn / prey.len() as f32;
       for &n in &prey { predation_loss[n] += share; }
   }
   ```

2. In the main loop's gain computation, extend the `match`:

   ```rust
   let gain = match species.metabolism {
       Metabolism::Photolithic => cell.light * energy.photolithic_metabolism_gain * fit,
       Metabolism::Predator => predation_gain[idx],
       Metabolism::Decomposer => 0.0, // task 015
   };
   ```

   and fold `predation_loss[idx]` into the energy update alongside the existing terms:

   ```rust
   let new_energy = organism.energy + gain + interaction_delta - upkeep - crowding_penalty - predation_loss[idx];
   ```

3. Add the `TECH_DESIGN.md` note — a short paragraph under §6, next to "Tick Processing Order", titled something like "Shared resource drain (predation, decomposition)", stating the pre-pass/accumulator pattern so task 015 can cite it instead of re-deriving it.

4. Tests in `src/sim.rs`, alongside the existing photolithic ones: isolated predator (no prey) collapses on schedule; predator with one adjacent high-energy prey gains and the prey loses the matching amount; two predators sharing one prey split it deterministically regardless of which predator has the lower grid index.

---

## ⚠️ Constraints and Caveats

- **Invariant 1**: the pre-pass reads only `world.cells` (the snapshot); it must not read or write `world.scratch`.
- **Invariant 3**: `predator_drain_cap` and `predator_upkeep` come from `SimConfig`, already defined (task 002) — no new constants.
- Don't implement species-specific prey targeting (e.g. "predators only eat photolithics") — not in the GDD baseline; any occupied neighbour is valid prey.
- Don't touch `Metabolism::Decomposer`'s `gain` beyond leaving it at `0.0` — that's task 015.

---

## 🔗 Dependencies

- **Depends on**: 005
- **Blocks**: 015 (reuses the drain-pass pattern documented here)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/014-predator-metabolism.md)"$'\n\nExecute this task in the current project.'
```
