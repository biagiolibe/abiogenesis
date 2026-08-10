# Task 088 — Exhaustive search in `draw_species_tags` to guarantee zero self-interaction

> **ID**: `088`
> **Category**: Bugfix / Balance
> **Priority**: 🔴 P1
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-10 (diagnosed live: user reported starting species growing explosively despite task 083's incubation)

---

## 🎯 Objective

The user observed that some **starting species** (generated at world
creation, never touched by Splice) grow explosively — both in individual
energy and population — even with task 083's newborn incubation delay
active.

**Root cause** (confirmed via combinatorial analysis + code reading, not
assumption): every tick, two adjacent organisms of the same species apply
every ordered pair of their shared tag set to each other's energy
(`src/sim.rs:261-307`, `interaction_delta`, added linearly). If a species'
own tag set has a nonzero `net_self_interaction`
(`src/world.rs:779-789`), every same-species cluster either self-reinforces
(positive) or self-drains (negative) — `crowd_factor` (0.15/neighbour) is
far too small to counteract a matrix entry up to ±2.

`draw_species_tags` (`src/world.rs:752-774`, added by task 048) tries to
avoid this by drawing up to `max_self_conflict_draws` (20) **random**
candidate tag sets and returning the first with `net_self_interaction == 0`,
falling back to the "closest to zero, possibly still nonzero" candidate if
none of the 20 random tries hit exactly zero.

With the actual defaults (`active_tags_early = 5` at world 0,
`matrix_density = 0.4`, `tags_per_species_max = 3`), there are only
`C(5,3) = 10` possible 3-tag combinations. Combinatorial analysis shows that
in roughly **15% of worlds, none of those 10 combinations nets exactly
zero** — the 20-retry loop cannot succeed no matter how many times it runs,
because the search space itself has no solution, not because retries ran
out. With only 3 species generated per world (`starting_species_count: 2` +
`extra_available_species_count: 1`), this reads to the player as "often",
not "rare".

Key fact that makes this fully fixable: a **1-tag** species has no pairs at
all, so its `net_self_interaction` is *always* exactly zero by construction.
A fully safe fallback therefore always exists — the fix is to guarantee
exact zero, not just get closer to it.

---

## 📋 Acceptance Criteria

- [x] `draw_species_tags` (`src/world.rs:752-774`) rewritten to use
      **exhaustive, deterministic search at decreasing tag-set size**
      instead of random-sample-with-retry:
      1. Roll `n` in `[tags_per_species_min, tags_per_species_max]` as
         today (RNG intent unchanged).
      2. Enumerate every possible `n`-tag combination from
         `world.active_tags` and filter to those with
         `net_self_interaction == 0`.
      3. If at least one exists, pick one via `world.rng` (never
         `rand::rng()` — determinism invariant, TECH_DESIGN.md §5).
      4. If none exists at this `n`, decrement `n` by 1 and repeat.
      5. `n = 1` always has at least one valid candidate, so the function
         always returns a tag set with `net_self_interaction == 0` exactly
         — no more "closest to zero" fallback anywhere.
- [x] A small in-house `combinations(slots: &[TagSlot], k: usize) ->
      Vec<Vec<TagSlot>>` helper added next to `net_self_interaction` in
      `src/world.rs`. Do **not** add `itertools` as a dependency — it is
      not currently a direct dependency of this project (only pulled in
      transitively via `Cargo.lock`), and the search space here is tiny
      (`C(8,3) = 56` worst case), so a hand-written helper is simpler and
      keeps enumeration order deterministic and explicit.
- [x] `draw_species_tags`'s signature is unchanged
      (`pub fn draw_species_tags(world: &mut SimWorld, config: &SimConfig)
      -> Vec<TagSlot>`) — its two callers (`src/worldgen.rs:155` and
      `:196`) need no changes.
- [x] `config.tags.max_self_conflict_draws` fully removed: `src/config.rs`
      (field, doc comment, `Default` impl), `assets/config/sim_config.ron`
      (manual-sync mirror), `draw_species_tags`'s own doc comment
      (`src/world.rs:733`), and the one test that sets it explicitly (see
      below). `grep -rn max_self_conflict_draws .` must return zero hits
      when done.
- [x] `tags_per_species_min`'s doc comment updated to note it is now a
      floor on the *initial roll* only, not a guarantee on the returned
      tag-set length — when no zero-net combination exists at `min` tags,
      the algorithm keeps descending below `min`, down to 1 if necessary,
      because the zero-self-interaction invariant takes precedence.
      Invisible under the shipped default (`min = 1`).
- [x] Existing `src/world.rs` tests updated:
      - `draw_species_tags_avoids_a_self_destructive_pair_when_a_safe_one_exists`
        and `draw_species_tags_avoids_a_self_reinforcing_pair_when_a_safe_one_exists`
        (`~1163`, `~1202`): tighten from the probabilistic
        `successes >= trials * 9 / 10` (30-seed sweep) to
        `assert_eq!(successes, trials)` — exhaustive search makes the
        outcome deterministic, not just likely.
      - `draw_species_tags_never_panics_when_every_combination_is_self_destructive`
        (`~1237`): its setup (2-tag pool, the only possible pair nets -3)
        is exactly the case the new algorithm resolves by falling back to
        1 tag. Rename to
        `draw_species_tags_falls_back_to_a_smaller_tag_set_when_no_combination_is_neutral`,
        remove its `config.tags.max_self_conflict_draws = 5` line (field
        gone), change assertions from `tags.len() == 2` /
        `net_self_interaction == -3` to `tags.len() == 1` /
        `net_self_interaction == 0`.
      - `drawn_species_tags_stay_within_bounds_and_the_active_pool`: no
        change needed under the default config (`min = 1`).
- [x] New regression test added to `src/world.rs`'s `mod tests`, exercising
      the **real default config** (`active_tags_early = 5`, i.e. the
      world-0-realistic conditions where the bug was observed), confirming
      `net_self_interaction(&world.matrix, &draw_species_tags(...)) == 0`
      holds across ~300 seeds × several draws per world (this is the direct
      property test the existing suite lacked — the only prior coverage was
      the noisy, downstream, conflated `population_never_saturates_the_grid_across_seeds`
      in `tests/balance.rs`).
- [x] `tests/balance.rs`'s `MAX_SATURATION_RATE` (0.3) left **unchanged** in
      this task — its doc comment already attributes residual saturation
      risk to cross-species reinforcement (untouched by this fix); add one
      comment line noting same-species self-interaction is now eliminated
      exhaustively, not best-effort, so that attribution is now exactly
      correct rather than approximately correct. Re-run the full
      `tests/balance.rs` suite once after the change to confirm existing
      thresholds still hold (expected to only improve, never regress) — do
      not retune blindly if something unexpectedly changes; investigate
      first.
- [x] `cargo test` and `cargo clippy --all-targets -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `draw_species_tags`/`net_self_interaction` (~725-790) rewritten; new `combinations` helper; tests updated/added (~1140-1260). |
| `src/config.rs` | `TagConfig` — remove `max_self_conflict_draws` (~304-341). |
| `assets/config/sim_config.ron` | Remove `max_self_conflict_draws: 20` (line 76), manual-sync mirror of `src/config.rs`. |
| `src/worldgen.rs` | `draw_species_tags` callers (lines 155, 196) — read-only reference, no changes expected. |
| `tests/balance.rs` | `MAX_SATURATION_RATE` doc comment update only (~60-70, ~140-170); re-run, not expected to need threshold changes. |

---

## 🧩 Technical Context

- **Current behavior**: `draw_species_tags` draws `n` random tags up to 20
  times, accepting the first exact-zero `net_self_interaction` hit or,
  failing that, the least-bad nonzero candidate seen. With a small active
  tag pool (5 at world 0) and `tags_per_species_max = 3`, only 10 distinct
  3-tag combinations exist; in ~15% of worlds none of them net exactly
  zero, so the fallback is a *guaranteed*, not rare, nonzero outcome for
  every 3-tag species drawn in that world.
- **Desired behavior**: the function always returns a tag set with
  `net_self_interaction == 0` exactly, by exhaustively checking all
  combinations at the rolled size and gracefully degrading to smaller sizes
  (always terminating at 1 tag, which is trivially always safe) rather than
  accepting a "closest but still wrong" result.
- `net_self_interaction` (`src/world.rs:779-789`) is the existing helper
  that sums `matrix.get(a, b)` over every ordered pair `a != b` in a tag
  set — reused unchanged by the new search, no logic change to this
  function itself.
- RNG must stay `world.rng` throughout (never `rand::rng()`) — the
  determinism invariant (TECH_DESIGN.md §5) that the rest of `sim`/`world`
  already follows.

---

## 🔨 Suggested Implementation

1. Add `combinations(slots: &[TagSlot], k: usize) -> Vec<Vec<TagSlot>>` next
   to `net_self_interaction` in `src/world.rs` (simple recursive or
   iterative generator, lexicographic order, no external crate).
2. Rewrite `draw_species_tags` per the acceptance criteria's 5-step
   algorithm, using `world.rng` (`.choose()`, via `rand::seq::IndexedRandom`,
   already imported) to pick among same-size zero-net candidates.
3. Remove `config.tags.max_self_conflict_draws` everywhere (`src/config.rs`,
   `sim_config.ron`, `draw_species_tags`'s doc comment).
4. Update `tags_per_species_min`'s doc comment with the new floor-vs-guarantee
   nuance.
5. Update the three affected `src/world.rs` tests and add the new
   300-seed regression test.
6. Add the one-line doc comment near `tests/balance.rs`'s
   `MAX_SATURATION_RATE`.
7. `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt`.
8. Re-run `tests/balance.rs` specifically and confirm no threshold
   regressions.

---

## ⚠️ Constraints and Caveats

- Do **not** touch the separate Splice self-interaction gap (planned as its
  own task, 089) — that's a different code path (`apply_splice`,
  `src/input.rs`) with a different mechanism (reject vs. retry) since
  Splice edits are deterministic player choices, not RNG-retryable draws.
- Do **not** touch `sim::step`'s tick formula, reproduction, incubation
  (task 083), or `crowding_penalty`.
- Do **not** touch cross-species reinforcement handling — out of scope,
  already an accepted residual risk per task 048's own tests.
- Do **not** add `itertools` or any new crate dependency for this — the
  search space is small enough that a hand-written helper is simpler and
  keeps determinism explicit.
- Do **not** retune `MAX_SATURATION_RATE` speculatively — only investigate
  and adjust if `tests/balance.rs` actually fails after the change.

---

## 🔗 Dependencies

- **Depends on**: 048 (the `net_self_interaction`/`draw_species_tags`
  invariant this task strengthens).
- **Related**: 089 (Splice self-interaction gap) — independent, no
  ordering dependency either way.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/088-exhaustive-self-neutral-species-tags.md)"$'\n\nExecute this task in the current project.'
```
