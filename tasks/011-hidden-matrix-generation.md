# Task 011 — Hidden matrix generation with cyclicity constraint

> **ID**: `011`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Generate the secret `tag × tag` **hidden matrix** (GDD §5.5) that the whole game revolves around: an asymmetric table of adjacency effects, generated once per world, deterministically, with a guaranteed anti-degeneration property (GDD §5.8).

---

## 📋 Acceptance Criteria

- [ ] A `TagMatrix` type stores an `active_tags.len() × active_tags.len()` grid of effect values, indexable by `TagId` on both axes (row = exerting tag, column = receiving tag, per GDD §5.5's convention).
- [ ] Values are integers in `{-2, -1, 0, +1, +2}` (`config.tags.effect_intensity_min..=max`).
- [ ] **Diagonal is always `0`** (a tag has no effect on itself — not stated explicitly in the GDD but implied by every example; document the choice inline).
- [ ] Density of non-zero cells (off-diagonal) is close to `config.tags.matrix_density` (~40%).
- [ ] The matrix is **asymmetric in general**: generation must not force `matrix[a][b] == matrix[b][a]`.
- [ ] **Cyclicity constraint**: generation guarantees at least one negative 3-cycle among active tags — `A`, `B`, `C` with `matrix[A][B] < 0`, `matrix[B][C] < 0`, `matrix[C][A] < 0` (GDD §5.8, §16.1's RPS example).
- [ ] Generation is deterministic: same seed ⇒ identical matrix.
- [ ] `SimWorld` stores the generated matrix (e.g. `pub matrix: TagMatrix`), built at construction from `active_tags`.
- [ ] Unit tests cover: value range, diagonal-zero, approximate density, presence of the guaranteed cycle, determinism.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `TagMatrix` type, generation function, wiring into `SimWorld::new` |
| `src/config.rs` | `TagConfig` — already complete, read-only here |

---

## 🧩 Technical Context

GDD §5.5 and §16.1's worked example (5 tags, ~40% density, one bolded RPS cycle) are the reference. `TagConfig` already has everything needed: `effect_intensity_min/max`, `matrix_density`.

### Why the cyclicity constraint needs a retry loop, not a formula

There's no closed-form way to sample a random sparse asymmetric matrix that's guaranteed to contain a negative 3-cycle. The practical approach: generate candidate matrices from the world's RNG until one satisfies the constraint, or — simpler and still deterministic — generate the matrix normally, then **force** a cycle by picking 3 distinct active tags (again via the world's RNG, so it stays reproducible) and overwriting `matrix[A][B]`, `matrix[B][C]`, `matrix[C][A]` with negative values if they aren't already negative. The second approach is simpler to reason about and to test (it always terminates), and is the recommended one.

### Matrix representation

A flat `Vec<i8>` of size `n * n` (n = `active_tags.len()`) indexed as `matrix[exerter.0 as usize * n + receiver.0 as usize]` is simplest and matches the dense-array style already used for the grid (`TECH_DESIGN.md` §3.1). Wrap access behind methods (`get(exerter, receiver)`) rather than exposing the flat layout.

---

## 🔨 Suggested Implementation

1. ```rust
   /// The secret tag x tag adjacency matrix (GDD 5.5). Row = exerting tag,
   /// column = receiving tag: `matrix.get(a, b)` is the energy delta `b`
   /// receives from being adjacent to `a`.
   pub struct TagMatrix {
       size: usize,
       values: Vec<i8>,
   }

   impl TagMatrix {
       pub fn get(&self, exerter: TagId, receiver: TagId) -> i8 { ... }
   }
   ```

2. Generation function, called from `SimWorld::new` after `active_tags` is set (task 010):

   ```rust
   fn generate_matrix(active_tags: &[TagId], config: &TagConfig, rng: &mut StdRng) -> TagMatrix {
       // 1. start with an all-zero n x n grid, diagonal stays 0.
       // 2. for each off-diagonal cell, with probability ~= matrix_density,
       //    assign a non-zero value uniformly from effect_intensity_min..=max
       //    (excluding 0, otherwise density drops silently).
       // 3. pick 3 distinct tags with rng.random_range/shuffle; force those
       //    3 off-diagonal cells negative (e.g. -1 or -2) to guarantee the cycle.
   }
   ```

3. Density: sampling each off-diagonal cell independently with `p = matrix_density` gives an *expected* density near the target — that's enough to satisfy "approximate density" in tests; don't over-engineer an exact-count sampler.

4. Tests in `src/world.rs`, alongside the existing environment/RNG tests: build a `SimWorld`, inspect `world.matrix`, assert the properties above. For the cycle check, a small brute-force search over triples of active tags is fine at this scale (≤8 tags ⇒ ≤56 ordered triples).

---

## ⚠️ Constraints and Caveats

- **Invariant 1**: matrix generation uses only `world.rng_mut()`.
- **Invariant 3**: density, intensity range, and cycle-forcing all read from `SimConfig`, no hardcoded numbers.
- Keep `TagMatrix` in `world.rs` (or a new headless module if it grows) — it must stay free of `bevy::render`/`bevy_egui` like the rest of the domain (invariant 2).
- Don't wire the matrix into `sim::step` yet — that's task 012. This task only generates and stores it.

---

## 🔗 Dependencies

- **Depends on**: 010
- **Blocks**: 012

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/011-hidden-matrix-generation.md)"$'\n\nExecute this task in the current project.'
```
