# Task 009 — Determinism tests and carrying-capacity validation

> **ID**: `009`
> **Category**: Test
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Prove with automated tests that the simulation **is deterministic** and **doesn't degenerate**: a photolithic bloom grows and then stabilizes, instead of exploding or going extinct.

Together with task 007, **this task is Phase 0's milestone** (GDD §13): *"watch a photolithic species bloom and stabilize thanks to carrying capacity."* Here that milestone stops being a visual impression and becomes a verified property.

These tests are also the **tuning safety net**: GDD §13 and §14 flag emergence balancing as the project's most delicate work, and without an automated way to know if a tweak broke something, that work proceeds blind.

---

## 📋 Acceptance Criteria

- [ ] Tests live in `tests/` and run **headless, without building a Bevy `App`**.
- [ ] **Determinism**: two 200-tick runs with the same seed produce identical final state (grid cell by cell, not just population).
- [ ] **Seed sensitivity**: different seeds produce different final state (rules out the trivial case where the RNG is never used).
- [ ] **Carrying capacity**: from a single seed in a bright zone, the population grows, then stabilizes within a band and stays there.
- [ ] **Light niche**: after N ticks, no organism survives in rows with `light < 0.25`.
- [ ] **Non-extinction**: population never reaches zero in the nominal scenario.
- [ ] `cargo test` passes; `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `tests/determinism.rs` | Reproducibility tests |
| `tests/balance.rs` | Carrying capacity, niches, non-extinction |
| `src/lib.rs` | **May need to be created** — see notes |

---

## 🧩 Technical Context

- **Current behavior**: the simulation works and can be watched running, but nothing verifies its properties automatically.
- **Desired behavior**: `cargo test` confirms determinism and stability.

### GDD §5.7 — Determinism

> The simulation is **deterministic** given the same seed: seeded RNG kept in the world state. Essential for debugging emergence, reproducing bugs, and (down the line) sharing interesting seeds.

### GDD §5.8 — Anti-degeneration

> The number-one risk of emergence is collapsing into two boring outcomes: **"everything dies"** or **"one species dominates."**

In Phase 0 there's only one species, so of the three GDD §5.8 levers only two are active and testable: **carrying capacity** (crowding penalty) and **environmental heterogeneity** (niches). The matrix's cyclicity constraint is Phase 1.

### Reference numbers (GDD §5.9)

| Scenario | Expected |
|---|---|
| Isolated photolithic, `light ≈ 0.7` | net `≈ +0.9`/tick → grows |
| With 7 occupied neighbors | net `≈ −0.15`/tick → stalls |
| At `light = 0.2` | `gain 0.4 < upkeep 0.5` → doesn't survive |

The survival threshold sits around `light = 0.25`: `light · 2.0 · env_fit = 0.5`.

---

## 🔨 Suggested Implementation

1. **Make the domain reachable from tests.** Integration tests in `tests/` import the crate as an external consumer, so a `src/lib.rs` is needed that exports `config`, `world`, and `sim`, with `main.rs` becoming a consumer of it. This is the right moment to do it: it confirms in practice that the simulation **is independent of rendering** (invariant 2).

   Alternatively, `#[cfg(test)]` unit tests inside `src/sim.rs`. The `lib.rs` approach is preferable: the boundary becomes explicit and compiler-verified.

2. **Determinism**

   ```rust
   #[test]
   fn same_seed_yields_identical_history() {
       let cfg = SimConfig::default();
       let mut a = SimWorld::new(42, &cfg);
       let mut b = SimWorld::new(42, &cfg);
       for _ in 0..200 {
           step(&mut a, &cfg);
           step(&mut b, &cfg);
       }
       assert_eq!(snapshot(&a), snapshot(&b));
   }
   ```

   `snapshot` should be compared **cell by cell** — species and energy, not just total population: two worlds can have the same population and different configurations. For energy, compare bits (`f32::to_bits`) or with a tight tolerance: the same sequence of operations on the same platform produces the same bits, so exact equality is legitimate here and catches drift that a loose tolerance would hide.

3. **Seed sensitivity** — same pattern with seeds `42` and `43`, `assert_ne!`. Without this test, a `step` that never uses the RNG would pass the determinism test with flying colors.

4. **Carrying capacity**

   ```rust
   #[test]
   fn bloom_stabilises_instead_of_exploding() {
       // Seed one photolithic organism in the lit band, run long enough to saturate.
       // Population must grow, then settle: sampled over the last 50 ticks it should
       // stay within a narrow band and never hit zero or fill the lit area entirely.
   }
   ```

   Sample the population every N ticks and verify that the **relative amplitude** of the last measurements stays under a threshold (e.g., 10%), instead of fixing an absolute number: the exact band depends on the coefficients, and those are declared tunable (GDD §14). A test that fixes an absolute value needs redoing on every tuning tweak; one that checks the *shape* of the curve survives.

   Allow enough ticks (300–500) for saturation to actually occur.

5. **Light niche** — after stabilization, verify that rows with `light < 0.25` contain no living organism. This is proof that environmental heterogeneity is creating real niches.

6. **Non-extinction** — in the nominal scenario, population never touches zero.

---

## ⚠️ Constraints and Caveats

- **No Bevy `App` in the tests.** If testing the simulation required building one, invariant 2 has been violated and *that* needs fixing, not the test.
- **No test dependent on the clock or on parallelism**: `cargo test` runs tests in concurrent threads, and any leftover global state would show up as flakiness.
- **Don't fix absolute population values** except where the GDD sets them. Coefficients are declared tunable: tests must verify *properties* (grows, stabilizes, doesn't die, respects niches), not numbers.
- If a balance test fails, **the first hypothesis is that the coefficients are wrong, not the test**: note the outcome in `PROJECT_PLAN.md` under tuning questions instead of adjusting thresholds until it goes green.
- With 200–500 ticks on a 48×32 grid, tests stay in the range of seconds in release mode; if they're slow in debug, task 001's profile (`opt-level = 3` on dependencies) helps, but for tests `cargo test --release` is what matters.

---

## 🔗 Dependencies

- **Depends on**: 005
- **Blocks**: none — but it's **Phase 0's exit gate**: don't move to Phase 1 with these tests red.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/009-determinism-balance-tests.md)"$'\n\nExecute this task in the current project.'
```
