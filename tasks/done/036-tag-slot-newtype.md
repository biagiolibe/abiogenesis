# Task 036 — `TagSlot` newtype: compiler-driven matrix indexing

> **ID**: `036`
> **Category**: Refactor / Architecture
> **Priority**: 🔴 P1
> **Estimate**: ~3-4h (heavier than the ~2h standard — confirmed with the user during planning: cannot be split without leaving the build broken halfway through)
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Phase 3 planning session

---

## 🎯 Objective

`TagMatrix` and `MatrixKnowledge` are currently indexed directly by `TagId.0 as usize`, assuming that a world's active tags are the contiguous range `TagId(0..n)` — a behavior from task 010, never questioned so far because it has always held. An explicit comment in `world.rs:34-38` flags the risk: *"If Phase 3 world generation ever picks a non-contiguous subset of the global pool, this indexing needs to go through a tag→matrix-index lookup instead."*

For worldgen (GDD §9) to be able to pick a genuinely varied subset from the global pool of 10 tags (not just the first N), this assumption needs to be broken. The chosen solution is to introduce `TagSlot(u8)` — the position of a tag within the *current world's* active subset — as a type distinct from `TagId(u8)` — the tag's identity in the *global* pool. `TagMatrix` and `MatrixKnowledge` are indexed by `TagSlot`; `SimWorld.active_tags: Vec<TagId>` remains the only slot→global-identity map, consulted only by `text.rs`/`ui.rs` to resolve names/colors/glyphs.

This is a pure refactor: no gameplay behavior changes. The worldgen that actually exploits `TagSlot` to select non-contiguous subsets arrives in task 038.

---

## 📋 Acceptance Criteria

- [x] New `TagSlot(pub u8)` type in `src/world.rs`, alongside `TagId`, with consistent derives (`Debug, Clone, Copy, PartialEq, Eq`).
- [x] `TagMatrix::get` takes `TagSlot` instead of `TagId`: `fn get(&self, exerter: TagSlot, receiver: TagSlot) -> i8`.
- [x] `Species.tags: Vec<TagSlot>` (was `Vec<TagId>`).
- [x] `MatrixKnowledge` in `src/notebook.rs` (record/evidence/is_confirmed/revealed_value and its internal indexing) migrated to the same `TagSlot` keys.
- [x] `SimWorld.active_tags: Vec<TagId>` remains the sole slot→global-identity map (the index in the `Vec` *is* the slot); no other part of the code maintains a parallel mapping.
- [x] Every site that currently converts a tag into a matrix/evidence index with `tag.0 as usize` has been updated to use `TagSlot` directly — none remaining (verified via grep: the only occurrences of `TagId` outside `world.rs` are `tag_color`/`tag_glyph`/`node_tooltip_text` in `notebook.rs`, global identity for the UI, and `active_tags` in `input.rs` tests).
- [x] `text.rs`/`ui.rs`: the points that must show a tag's name/color/glyph resolve `TagSlot → TagId` via `world.active_tags[slot.0 as usize]` before calling the `text.rs`/`tag_glyph`/`tag_color` functions.
- [x] `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` all green at end of task.
- [x] No regressions: 68 tests pass, unchanged in expected values (only the fixture types changed from `TagId` to `TagSlot` where they operate on matrix/evidence).

## Implementation summary

- `src/world.rs`: `TagSlot(pub u8)` introduced alongside `TagId`; `TagMatrix::get` and `Species.tags` migrated to `TagSlot`; `generate_matrix` no longer requires `active_tags: &[TagId]`, only `slot_count: usize` (the negative-cycle selection samples slots `0..n`, not identities); `draw_species_tags` returns `Vec<TagSlot>`.
- `src/sim.rs`: `AdjacencyObserved::{exerter_tag, receiver_tag}` and `neighbour_tags` migrated to `TagSlot` (the whole file already operated in "slot space" via `species.tags`, so the change was a direct `TagId` → `TagSlot` substitution).
- `src/notebook.rs`: `MatrixKnowledge` migrated to `TagSlot` keys; `hypothesis_grid`/`node_tooltip_text` now enumerate `world.active_tags` to get `(TagSlot, TagId)` pairs — slot for `MatrixKnowledge` queries, `TagId` for glyph/color; `catalog_panel` resolves `TagSlot → TagId` via `world.active_tags[slot.0 as usize]` before coloring/glyphing.
- `src/ui.rs`: `SpliceEditChoice::{SwapTag, AddTag}` migrated to `TagSlot`; `splice_panel` enumerates `world.active_tags` to build selectable slots, resolving identity only for the visual label.
- `src/input.rs`: `apply_splice` unchanged in its body (it already operated on `species.tags` without naming the type); only the tests updated to `TagSlot` where they construct species tags/`SpliceEditChoice`, keeping `TagId` for `world.active_tags`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | Definition of `TagSlot`, `TagMatrix::get`, `Species.tags`, `SimWorld.active_tags` as the sole slot→identity source. |
| `src/sim.rs` | `AdjacencyObserved` and every call to `matrix.get(...)` in the tick logic; hand-built tests that construct matrices/species. |
| `src/notebook.rs` | `MatrixKnowledge` (record/evidence/is_confirmed/revealed_value) and the grid/graph rendering that iterates `world.active_tags`. |
| `src/ui.rs` | Every site that iterates tags for name/color/glyph — must resolve `TagSlot → TagId` before calling `text.rs`. |
| `src/input.rs` | `MatrixKnowledge::new` reset call (the `r` key) — update to the new signature if it changes. |

---

## 🧩 Technical Context

**Current `TagMatrix`** (`src/world.rs`, lines ~30-51):
```rust
pub struct TagId(pub u8);

pub struct TagMatrix {
    pub(crate) size: usize,
    pub(crate) values: Vec<i8>,
}

impl TagMatrix {
    pub fn get(&self, exerter: TagId, receiver: TagId) -> i8 {
        self.values[exerter.0 as usize * self.size + receiver.0 as usize]
    }
}
```
The comment on lines 34-38 documents exactly the risk this task resolves.

`MatrixKnowledge` in `notebook.rs` duplicates the same indexing pattern (`TagId.0 as usize * size + ...`) to track weighted evidence (`1/(1+n_confounders)`, GDD §7) — it needs to be migrated in sync, not in a separate task, because changing `TagMatrix::get`'s signature breaks `notebook.rs`'s build at the same moment.

- **Current behavior**: `TagId` serves both as the global identity in the pool and as a direct index into matrix/evidence, because so far the active tags have always been `TagId(0..active_tags_early)` — a coincidence that worldgen (task 038) will break.
- **Desired behavior**: `TagId` = identity in the global pool of 10 tags (used only for name/color/glyph via `SimWorld.active_tags`); `TagSlot` = position within the current world's active subset (used for every matrix/evidence indexing). The slot→identity conversion always goes through `SimWorld.active_tags[slot.0 as usize]`, never through a duplicated map.

---

## 🔨 Suggested Implementation

1. Add `TagSlot(pub u8)` in `src/world.rs`, near `TagId`.
2. Change `TagMatrix::get` to accept `TagSlot`. Let `cargo build` fail and use the compiler errors as a checklist to propagate the change to `Species.tags`, `sim.rs`, `notebook.rs`, `ui.rs` — this is the most reliable way not to miss a call site (hence the choice to make this a single task).
3. In `sim.rs`, check `AdjacencyObserved` and every point in the tick logic that calls `matrix.get`: it must receive `TagSlot`, not `TagId`. If `AdjacencyObserved` (event consumed by the notebook) currently carries `TagId`, assess whether to convert it to `TagSlot` upstream (in the tick logic, where `SimWorld` is already accessible) or leave it as `TagId` and convert only on consumption — prefer the first option if it simplifies `notebook.rs`, since `MatrixKnowledge` already works in terms of slots.
4. In `notebook.rs`, update `MatrixKnowledge` (internal struct, `record`, `evidence`, `is_confirmed`, `revealed_value`) to the new `TagSlot` key. Update the grid/graph rendering functions that currently iterate `world.active_tags` to build nodes/labels: they will keep iterating `world.active_tags` (which stays `Vec<TagId>`), but the iteration index *is* the `TagSlot` to pass to `MatrixKnowledge`.
5. In `ui.rs`, for every site that shows a tag's name/color/glyph: make sure it resolves `TagSlot → TagId` (`world.active_tags[slot.0 as usize]`) before calling `text.rs`'s functions — `text.rs` itself doesn't change (it doesn't know about `SimWorld`, task 034).
6. In `input.rs`, update the construction of `MatrixKnowledge::new` in the `r`-key reset if the signature changes.
7. Update the hand-built tests in `world.rs`/`sim.rs`/`notebook.rs` that construct `TagMatrix`/`Species` by hand: only the types used to build test data change, not the verified logic.
8. `cargo build && cargo clippy -- -D warnings && cargo test` — iterate until everything is green.

---

## ⚠️ Constraints and Caveats

- **Don't introduce a separate lookup layer** (e.g. a `HashMap<TagId, TagSlot>`): the index in `SimWorld.active_tags`'s `Vec<TagId>` is already the slot→identity map; a parallel map would be duplicated state to keep in sync, against the "no premature abstractions" principle.
- **Determinism (invariant 1)**: no `HashMap`/`HashSet` iterated in the tick logic — if a lookup structure is needed, use an indexed `Vec`.
- **Don't touch the active-tag selection logic**: this task does not yet select non-contiguous subsets (that arrives in task 038) — here `active_tags` remains populated exactly as today (`0..active_tags_early`), only *how* the matrix is indexed changes.
- **No gameplay behavior must change**: this is a behavior-preserving refactor, verifiable with the existing determinism/balance tests (task 009), which must keep passing with no changes to expected values.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: 038 (worldgen cannot select non-contiguous subsets until `TagSlot` exists).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/036-tag-slot-newtype.md)"$'\n\nExecute this task in the current project.'
```
