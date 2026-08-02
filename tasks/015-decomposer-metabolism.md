# Task 015 — Decomposer metabolism and residue cycle

> **ID**: `015`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Implement the `Decomposer` metabolism (GDD §5.4): it feeds on `residue` (dead matter left behind by death, already tracked per-cell since task 005) instead of light or prey. Closes the death → residue → new growth loop from GDD §16.3's "decomposer ring".

---

## 📋 Acceptance Criteria

- [ ] `Metabolism::Decomposer` organisms gain `min(decomposer_extract_rate, residue available in own cell + Moore neighbours) * env_fit` per tick.
- [ ] Extracted residue is subtracted from the source cells' `residue` field this same tick, via the **same accumulation-pass pattern established in task 014** (documented in `TECH_DESIGN.md`) — never a direct cross-cell write inside the main loop.
- [ ] Residue can't go negative: total extraction from a cell is capped at that cell's available residue.
- [ ] A decomposer with no residue in range behaves like a photolithic organism in the dark: `gain = 0`, loses `decomposer_upkeep` per tick, eventually dies.
- [ ] A decomposer adjacent to/on top of sufficient residue nets positive energy and can reproduce.
- [ ] Residue decay (the existing per-tick `residue_decay` in `sim::step`) still applies independently — decomposer extraction and passive decay both reduce residue, decay first or extraction first must be picked and documented (recommended: decay first, since it already runs as its own pre-pass; extraction then draws from the post-decay value).
- [ ] Determinism preserved; existing tests unaffected.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `step()` — extends the drain-pass from task 014, `Metabolism::Decomposer` gain arm |

---

## 🧩 Technical Context

- **Current behavior**: `residue` accumulates on death and decays by `residue_decay` per tick (task 005), but nothing ever consumes it — it just fades away.
- **Desired behavior**: a decomposer near residue extracts energy from it, closing GDD §16.3's cycle: *death → residue → decomposer bloom → (via the matrix, task 012) fertilizes photolithics → new biomass → new deaths*.

### Reuse task 014's pattern

Task 014 establishes and documents (in `TECH_DESIGN.md`) the "compute a same-sized accumulator array from the snapshot before the main loop" pattern for predation. Decomposition is structurally identical: a decomposer draws from a shared resource (`residue`, spread across its own cell and neighbours) that must not be double-spent if two decomposers compete for the same cell's residue. Reference that section instead of re-explaining it; extend it only if residue's "own cell + neighbours" scope needs a note the predation case didn't (predation only used neighbours, not the predator's own cell).

---

## 🔨 Suggested Implementation

1. Extend (don't duplicate) the pre-pass from task 014 with a second accumulator, `decomposition_gain` / `residue_loss`:

   ```rust
   for idx in 0..world.cells.len() {
       let Some(organism) = world.cells[idx].organism else { continue };
       let species = &world.species[organism.species.0 as usize];
       if species.metabolism != Metabolism::Decomposer { continue; }
       let (x, y) = (idx % world.width, idx / world.width);
       let sources: Vec<usize> = std::iter::once(idx)
           .chain(world.moore_neighbours(x, y))
           .filter(|&n| world.cells[n].residue > 0.0)
           .collect();
       if sources.is_empty() { continue; }
       let fit = env_fit(world.cells[idx].temperature, species.temp_optimum, species.temp_tolerance);
       let available: f32 = sources.iter().map(|&n| world.cells[n].residue).sum();
       let drawn = (energy.decomposer_extract_rate * fit).min(available);
       decomposition_gain[idx] = drawn;
       // distribute `drawn` across `sources` proportionally to each source's residue,
       // capping each source's loss at its own residue value.
   }
   ```

2. Wire `decomposition_gain[idx]` into the `gain` match arm (replacing the `0.0` left by task 014), and apply `residue_loss[idx]` when writing `scratch[idx].residue` — after the existing decay subtraction, so decay and extraction compose instead of one overwriting the other.

3. Tests in `src/sim.rs`: decomposer alone with no residue dies on the same schedule as a photolithic in the dark (reuse that test's shape); decomposer next to a residue-bearing cell gains and the residue shrinks by the matching amount; residue never goes negative even with a decomposer drawing more than is available.

---

## ⚠️ Constraints and Caveats

- **Invariant 1**: pre-pass reads only `world.cells`; no `rand::rng()`.
- **Invariant 3**: `decomposer_extract_rate` / `decomposer_upkeep` already in `SimConfig` (task 002).
- Keep the decay-then-extract ordering **explicit and commented** — it's a resolved implementation choice, not obvious from the GDD, so a future reader shouldn't have to reverse-engineer it from the code.

---

## 🔗 Dependencies

- **Depends on**: 014
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/015-decomposer-metabolism.md)"$'\n\nExecute this task in the current project.'
```
