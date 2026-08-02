# Task 012 — Adjacency (matrix) effect in the tick

> **ID**: `012`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Wire the hidden matrix (task 011) into `sim::step`'s step 3, replacing the Phase 0 placeholder `interaction_delta = 0.0`. This is the mechanic that turns isolated species dynamics into real emergence (GDD §5.6 step 3, §5.8).

---

## 📋 Acceptance Criteria

- [ ] `interaction_delta` for an organism is the sum, over every occupied Moore neighbour, over every (neighbour tag × own tag) pair, of `world.matrix.get(neighbour_tag, own_tag)`.
- [ ] Read **only from the snapshot** (`world.cells`), matching the existing pattern for `occupied_neighbours` — the tick stays order-independent (no new determinism risk).
- [ ] The effect is **additive and linear** (`TECH_DESIGN.md` §5 invariant 4) — never multiplicative, never capped in a way that isn't itself a named coefficient.
- [ ] Energy update becomes `energy += gain + interaction_delta - upkeep - crowding_penalty`, i.e. the `interaction_delta` slot already present in `sim.rs` is filled in, not restructured.
- [ ] Unit tests with a hand-built 2-tag matrix and two synthetic species (mirroring the existing style in `sim.rs`'s `#[cfg(test)]` module) verify the exact expected delta for: one negative neighbour, one positive neighbour, multiple neighbours summing correctly, and a neighbour with no tag overlap effect (`0`).
- [ ] Existing Phase 0 tests (`isolated_photolithic_grows`, `crowded_photolithic_stalls_at_carrying_capacity`, etc.) still pass unchanged — they use single-species worlds with no matrix entries, so `interaction_delta` must still come out `0` for them.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `step()` — step 3 of the algorithm |
| `src/world.rs` | `TagMatrix::get`, `Species.tags` (tasks 010-011) |

---

## 🧩 Technical Context

- **Current behavior**: `sim.rs` has a comment "Kept as a sum, not folded away, so Phase 1 only has to fill it in" — this task is exactly that.
- **Desired behavior**: two adjacent organisms with matching tags in the matrix now actually gain/lose energy from proximity, on top of their own metabolism.

### Directionality (GDD §5.5 table convention)

Row = exerting tag, column = receiving tag. For an organism `i` with neighbour `j`: the delta `i` receives from `j` is `sum_{t_j in j.tags} sum_{t_i in i.tags} matrix.get(t_j, t_i)`. This matches the worked example in GDD §16.1 (`◆→▲ = -2` means a `◆`-tagged neighbour harms a `▲`-tagged organism, not the other way around).

---

## 🔨 Suggested Implementation

```rust
let mut interaction_delta = 0.0;
for neighbour_idx in world.moore_neighbours(x, y) {
    let Some(neighbour) = world.cells[neighbour_idx].organism else { continue };
    let neighbour_species = &world.species[neighbour.species.0 as usize];
    for &their_tag in &neighbour_species.tags {
        for &my_tag in &species.tags {
            interaction_delta += world.matrix.get(their_tag, my_tag) as f32;
        }
    }
}
```

Place this where the existing `// 3. Hidden matrix effect` comment already sits, replacing the `let interaction_delta = 0.0;` line. `occupied_neighbours` (step 4) already computes a similar Moore scan — keep them as two separate loops for clarity rather than merging, since they read different things and premature fusion would hurt readability for a 8-neighbour scan that's already cheap at this grid size.

---

## ⚠️ Constraints and Caveats

- **Invariant 4** (`TECH_DESIGN.md` §5): additive/linear, no exceptions — this is a design choice for deducibility, not a tuning knob.
- Don't change the reproduction/death steps in this task; only step 3 is in scope.
- `species.tags` can be empty for Phase-0-style single-species setups (task 010 guarantees 1-3 for newly seeded species, but existing tests construct species by hand) — the double loop over an empty `tags` vec naturally contributes `0`, no special-casing needed.

---

## 🔗 Dependencies

- **Depends on**: 011
- **Blocks**: none (013 doesn't need this to proceed, but their combination is what makes emergence observable)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/012-matrix-adjacency-tick-effect.md)"$'\n\nExecute this task in the current project.'
```
