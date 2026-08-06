# Task 048 — Contain runaway population/energy growth from some generated matrices

> **ID**: `048`
> **Category**: Bugfix / Balance
> **Priority**: 🔴 P1
> **Estimate**: ~2-3h (needs investigation before a fix can be scoped precisely)
> **Assigned to**: unassigned
> **Session**: 2026-08-06 playtest session

---

## 🎯 Objective

Some procedurally-generated worlds produce uncontained, runaway population/energy growth that saturates the entire grid — clearly outside the intended balance envelope (GDD §5.8/§5.9). Confirmed with a throwaway comparison across 5 seeds (`build_world` + 100 ticks of `sim::step`, no player actions):

```
seed=                   1 tags=5 pop= 132 total_energy=   1514.81
seed=                  42 tags=5 pop= 145 total_energy=   1513.48
seed=        999999999999 tags=5 pop=1536 total_energy= 562741.50   <- saturated (1536 = full grid)
seed= 9223372036854775807 tags=5 pop= 128 total_energy=   1441.59
seed=18446744073709551614 tags=5 pop=1472 total_energy= 379964.78   <- saturated
```

It is **not** correlated with the seed's numeric magnitude (`9223372036854775807` is huge and behaves normally; `999999999999` is comparatively small and saturates) — it's specific to which matrix/tag combination that seed's RNG stream happened to draw. `world.rs::generate_matrix` (task 011) only guarantees a single negative 3-tag cycle; nothing stops other tag pairs from having strong net-positive interactions that overwhelm the crowding penalty (GDD §5.6 step 5) and grow unbounded.

---

## 📋 Acceptance Criteria

- [ ] Root-cause the mechanism: confirm (with a script/test, not just re-running the two known bad seeds) whether it's the matrix's *net* positive-interaction strength, the crowding penalty's formula/coefficients, or an interaction between the two, that allows unbounded growth.
- [ ] A fix — either (a) a stronger constraint on matrix generation (e.g. bound the sum of positive outgoing interactions per tag, or guarantee more than one negative cycle at higher `active_tag_count`), or (b) a crowding-penalty/carrying-capacity change that caps growth regardless of matrix — chosen based on the root-cause finding above. Document the choice and why the other option was rejected.
- [ ] A balance test across a broad seed sample (dozens to hundreds, not just the 2 known-bad ones) asserting population/energy stays within some documented bound after N ticks — extending `tests/balance.rs`'s existing style (`population_rarely_reaches_total_extinction_across_seeds` is the opposite-direction precedent).
- [ ] The two known-bad seeds (`999999999999`, `18446744073709551614`) specifically no longer saturate the grid within a reasonable tick budget.
- [ ] `cargo clippy -- -D warnings` clean, `cargo test` green.
- [ ] No magic numbers: any new coefficient goes in `SimConfig` (CLAUDE.md convention).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `generate_matrix` (task 011/012) — the negative-3-cycle guarantee, matrix value range/density config. |
| `src/sim.rs` | The tick's energy-update formula (GDD §5.6 step 5), specifically the crowding-penalty term. |
| `src/config.rs` | `SimConfig`'s matrix/energy coefficients — likely where a new bound gets added. |
| `tests/balance.rs` | Existing seed-sweep balance tests to extend. |

---

## 🧩 Technical Context

<!-- TODO: add relevant code snippets and file paths -->

- **Current behavior**: `generate_matrix` guarantees one negative 3-tag cycle exists somewhere in the matrix (task 011's original safety net, written before task 038 made matrix size/density vary with `world_index`). No constraint bounds the matrix's *overall* net interaction sign or magnitude. The crowding penalty (GDD §5.6) presumably scales with local population density, but evidently not enough to counteract a strongly mutualistic matrix in every case.
- **Desired behavior**: whatever matrix a world procedurally generates, population growth stays bounded (never saturates the grid) under normal play (no player intervention), the same guarantee `009-determinism-balance-tests.md` originally established for the fixed Phase 0 matrix, now needing to hold across *all* procedurally generated matrices (task 038+), not just one hand-picked one.

---

## 🔨 Suggested Implementation

1. Write a diagnostic (can start from the throwaway comparison above) that, for a saturating seed, logs the matrix values and per-tick energy-update terms for a sample organism, to see which term (matrix gain vs. crowding penalty) dominates and by how much.
2. Based on the finding, prototype either the matrix-generation constraint or the crowding-penalty change.
3. Validate against the acceptance criteria's balance test.
4. Re-run the original 5-seed comparison to confirm both previously-saturating seeds now behave within bounds.

---

## ⚠️ Constraints and Caveats

- **Don't just patch the two known seeds**: the fix must generalize — a seed-specific special case would leave the underlying class of bug live for the next unlucky seed.
- **Determinism**: any fix must stay within `sim`/`world`/`config`'s no-external-RNG, no-`HashMap`-iteration invariants (TECH_DESIGN.md §5).
- **Balance is GDD-owned**: if the fix changes numeric baselines in GDD §5.9, update `abiogenesis-gdd.md` accordingly, don't let the doc drift from the code.

---

## 🔗 Dependencies

- **Depends on**: none (bugfix against already-shipped worldgen).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/048-contain-runaway-matrix-growth.md)"$'\n\nExecute this task in the current project.'
```
