# Task 089 — Reject Splice edits that create a nonzero same-species self-interaction

> **ID**: `089`
> **Category**: Bugfix / Balance
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-10 (diagnosed as a secondary, separate gap from 088 while investigating a user-reported runaway-growth bug — the observed bug itself was a starting/worldgen species, not a spliced one, but this gap is real and independent)

---

## 🎯 Objective

Every tick, two adjacent organisms of the same species apply every ordered
pair of their shared tag set to each other's energy
(`src/sim.rs:261-307`, `interaction_delta`). If a species' tag set has a
nonzero `net_self_interaction` (`src/world.rs:779-789`), same-species
clusters either self-reinforce (positive) or self-drain (negative) —
`crowd_factor` alone cannot contain it.

`draw_species_tags` (`src/world.rs:752-774`, hardened by task 048 and 088)
guarantees `net_self_interaction == 0` for every procedurally-generated
species. **`apply_splice` (`src/input.rs:428-494`) — the only other place a
new species is created, via the player's `SwapTag`/`AddTag` splice edits —
has no equivalent check at all.** It just pushes whatever tag combination
the chosen edit produces.

Because the matrix is asymmetric (`matrix.get(a,b)` need not equal
`matrix.get(b,a)`) and only partially revealed to the player (confirmation-gated
notebook mechanic, GDD §7), a player can create a spliced species whose net
self-interaction is strongly positive (or negative) with no way to see it
coming — the one direction they've confirmed might read as harmless or even
negative while the hidden reverse direction dominates the net sum.

---

## 📋 Acceptance Criteria

- [x] `net_self_interaction` (`src/world.rs:779`) made `pub fn` (no logic
      change) so `input.rs` (a different crate — the binary — from
      `world.rs`'s lib crate) can call it. Doc comment updated to mention
      the new caller.
- [x] `TagMatrix::from_values(size: usize, values: Vec<i8>) -> Self` added
      (`impl TagMatrix`, near line 62), with `assert_eq!(values.len(), size
      * size, ...)`. Needed because `TagMatrix`'s fields are `pub(crate)`
      (documented as being for `sim.rs`'s in-crate tests only) — `input.rs`
      lives in a different crate and cannot construct a `TagMatrix` literal
      directly for its own tests.
- [x] `apply_splice` (`src/input.rs:428-494`) gains a
      `net_self_interaction(&world.matrix, &new_species.tags) != 0`
      rejection, added **only** inside the `SwapTag { old: Some(old), new:
      Some(new) }` arm (after `new_species.tags[pos] = new;`) and the
      `AddTag { tag: Some(tag) }` arm (after `new_species.tags.push(tag);`,
      after the existing 3-tag-cap check), placed before the budget check —
      matching the existing "validity before affordability" order the
      3-tag-cap check already follows. `ShiftTempOptimum` is **not**
      touched — it never mutates `tags`, and a uniform post-match check
      would wrongly reject a `ShiftTempOptimum` splice off a source species
      that already (legitimately, via task 088's own "closest to
      zero"-eliminated-but-still-possible-pre-088-species, or simply an
      older save) carries a nonzero self-interaction for a reason unrelated
      to the player's edit.
- [x] Rejection is **strict `!= 0`**, not just `> 0` — consistent with task
      048's own precedent (`draw_species_tags` moved from `>= 0` to `== 0`
      specifically because net-negative self-interaction is also a real
      failure mode: a lineage that kills itself the instant it reproduces
      next to itself). No new `SimConfig` coefficient — `0` is the matrix's
      existing semantic neutral point, already used identically by
      `draw_species_tags`.
- [x] Rejection is a **silent no-op**, matching this file's established
      convention for rejected actions (e.g. the existing 3-tag-cap
      rejection, `attempt_seed`'s unaffordable/occupied-cell rejections):
      no species appended, no budget spent, no log entry, `draft.apply_requested`
      still reset to `false` (already set unconditionally near the top of
      `apply_splice`, before any check).
- [x] `apply_splice`'s doc comment (`src/input.rs:415-427`) updated with the
      new no-op case, mirroring how it already documents the tag-cap and
      incomplete-draft no-ops.
- [x] `world_with_one_taggable_species` (test helper, `src/input.rs:~515-527`)
      updated to install an explicit all-zero `TagMatrix` via
      `TagMatrix::from_values(2, vec![0, 0, 0, 0])` right after
      `SimWorld::new`, instead of relying on whatever matrix seed 42
      happens to generate — needed so the **existing** splice tests (e.g.
      `add_tag_splice_appends_a_tag_without_removing_any`,
      `src/input.rs:637-669`) stay deterministically passing under the new
      gate for a stated reason (matrix is neutral), not by luck of the
      seed. Doc comment on the helper updated to explain why.
- [x] Five new/updated unit tests added to `src/input.rs`'s existing `mod
      tests`, each overriding `world.matrix` after the helper call:
      **Implementation note**: `SwapTag` only replaces one tag in place, so
      a 1-tag source (as originally sketched here) always yields a
      still-1-tag, trivially-self-neutral result regardless of the matrix —
      not a meaningful test. The 3 `SwapTag` tests below instead use a
      3-tag active pool and a 2-tag source (`[TagSlot(0), TagSlot(2)]`,
      swapping `TagSlot(2)` for `TagSlot(1)`), with a 3×3 matrix isolating
      the `(0,1)` pair; the `AddTag` test keeps the original 1-tag-source
      shape since `AddTag` does grow the tag count.
      1. `swap_tag_splice_is_rejected_when_the_result_self_reinforces` —
         `(0,1)` pair nets +2; assert no species appended, source
         species unchanged, budget unspent, log empty,
         `draft.apply_requested == false`.
      2. `swap_tag_splice_is_rejected_when_the_result_self_drains` —
         `(0,1)` pair nets -2, same assertions — the regression guard
         that distinguishes "reject `!= 0`" from a weaker "reject only if
         positive" design.
      3. `swap_tag_splice_is_applied_when_the_result_is_self_neutral` —
         `(0,1)` pair nets 0 via nonzero individual entries (+1/-1) — splice
         applies normally (species count 2, budget decremented, logged),
         confirming the gate only blocks nonzero *net*, not nonzero
         individual matrix entries.
      4. `shift_temp_splice_is_unaffected_by_a_self_reinforcing_source` —
         source tags `[TagSlot(0), TagSlot(1)]` already net-nonzero under
         the reinforcing matrix from test 1, request `ShiftTempOptimum` —
         must still apply (species count 2, `temp_optimum` shifted) —
         regression guard for "only the tag-mutating arms are gated".
      5. `add_tag_splice_is_rejected_when_the_result_self_reinforces` —
         mirrors test 1 via the `AddTag` arm instead of `SwapTag`.
- [x] `cargo test` and `cargo clippy --all-targets -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `net_self_interaction` made `pub` (~779); new `TagMatrix::from_values` constructor (~62). |
| `src/input.rs` | `apply_splice`'s `SwapTag`/`AddTag` arms gated (~451-469); doc comment updated; `world_with_one_taggable_species` test helper updated (~515-527); 5 new/updated tests in `mod tests`. |

---

## 🧩 Technical Context

- **Current behavior**: `apply_splice` builds `new_species.tags` from the
  player's chosen edit with no check against the hidden matrix at all.
- **Desired behavior**: a `SwapTag`/`AddTag` edit that would leave the
  resulting species with a nonzero `net_self_interaction` is silently
  rejected, exactly like every other invalid-action case this file already
  handles silently (no rejected-action feedback mechanism exists yet in
  this codebase).
- Unlike `draw_species_tags` (task 088), Splice edits are deterministic,
  player-chosen single mutations — there is no "retry with a different
  random candidate" available here, so the mechanism is reject-on-violation,
  not search-for-a-safe-alternative.

---

## 🔨 Suggested Implementation

1. `src/world.rs`: make `net_self_interaction` `pub`; add
   `TagMatrix::from_values`.
2. `src/input.rs`: import `net_self_interaction`; add the `!= 0` guard to
   the `SwapTag` and `AddTag` arms of `apply_splice`, before the budget
   check.
3. Update `apply_splice`'s doc comment.
4. Update `world_with_one_taggable_species` to install an explicit all-zero
   matrix; verify the 4 pre-existing splice tests still pass.
5. Add the 5 new/updated tests listed above.
6. `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- Do **not** touch `draw_species_tags` or task 088's fix — this task only
  closes the equivalent gap on the Splice path.
- Do **not** add a new `SimConfig` tunable (e.g. a splice-specific
  tolerance) — reuse the exact same `0` neutral point `draw_species_tags`
  already targets.
- Do **not** touch `sim::step`, reproduction, or incubation.
- No live `cargo run` verification required unless the user asks for it —
  confirm with the user before running/screenshotting, per this session's
  established preference (082/083 skipped it on request).

---

## 🔗 Dependencies

- **Depends on**: 048 (the `net_self_interaction` invariant this task
  extends to Splice).
- **Related**: 088 (same underlying invariant, worldgen side) — independent,
  no ordering dependency either way.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/089-splice-self-interaction-gate.md)"$'\n\nExecute this task in the current project.'
```
