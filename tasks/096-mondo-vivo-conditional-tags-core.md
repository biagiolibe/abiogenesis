# Task 096 — Conditional tags: terrain-gated matrix participation

> **ID**: `096`
> **Category**: Feature / Simulation / Worldgen
> **Priority**: 🟡 P2
> **Estimate**: ~4h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-living-world.md`, §1 "Conditional tags (core mechanic)")

---

## 🎯 Objective

`interaction_delta` (`src/sim.rs:265-311`) reads only species tags today — no
terrain input anywhere in the tick formula. The design doc's core mechanic:
a small, **fixed** subset of the 10-glyph pool always carries a terrain
condition, in every world (a structural fact, like "the pool has 10 glyphs"
— learned from the manual, not decoded per world). Which *specific* terrain
triggers each conditional glyph, and whether it's `Inducible` (silent by
default, switched on by the trigger terrain) or `Repressible` (active by
default, switched off by the trigger terrain), is rolled fresh per world,
same RNG moment as today's tag-pool/matrix generation — that roll is the
actual mystery. Biochemical grounding: operon regulation (*lac* = inducible,
*trp* = repressible), fitting GDD pillar 3 ("depth via simple rules, not
simulated realism").

This task adds the data model, the per-world roll, and the
`interaction_delta` gate. It does **not** add any UI surface — the catalog
badge and its evidence-tracking are task 097, deliberately split out so this
task's acceptance criteria stay testable in isolation (matrix/sim internals
only, no live-verification line needed, matching task 084's precedent for
non-visible worldgen/sim changes).

---

## 📋 Acceptance Criteria

- [ ] `TagConfig` (`src/config.rs:321-344`) gains `conditional_tag_count: u32`
      (no magic numbers) — a first-pass guess of `1` or `2` (out of
      `global_tag_pool: 10`), tunable, matching the doc's density guidance
      ("only a small minority of the pool should be conditional").
- [ ] Conditionality is keyed on `TagId` (pool-wide identity), **not**
      `TagSlot` (per-world position) — the first `conditional_tag_count`
      `TagId`s by convention (e.g. `TagId(0)`, `TagId(1)`) are always the
      conditional ones, in every world. This is deliberate: `TagSlot` is
      assigned by `select_active_tags`'s per-world sample
      (`src/world.rs:962-966`), so keying conditionality on slot position
      would make a *different* glyph conditional every world — exactly what
      the doc's "fixed at the pool level" decision rules out.
- [ ] A new per-world roll — a small struct or `Vec` on `SimWorld`, e.g.
      `conditional_tags: Vec<(TagId, TerrainKind, Mode)>` — populated once at
      construction, same RNG moment as `select_active_tags`/`generate_matrix`
      (`SimWorld::new_for_world`, `src/world.rs:238-250`). Only conditional
      `TagId`s that actually landed in this world's `active_tags` need an
      entry (a conditional tag not drawn into a given world's active subset
      simply doesn't exist there this run — no entry, no gating needed).
- [ ] New `Mode` enum: `Inducible` | `Repressible`, rolled per conditional
      tag alongside its trigger `TerrainKind`, using the world's own RNG
      (`StdRng`, never `rand::rng()`).
- [ ] `interaction_delta` (`src/sim.rs:294-296`) gates a conditional tag's
      participation in the matrix lookup: for each `their_tag`/`my_tag` in
      the existing double loop, resolve `world.active_tags[slot.0 as usize]`
      to its `TagId`, look up whether that `TagId` is conditional in this
      world, and if so, only count it if the *organism carrying that tag's*
      current cell (`world.cells[idx]` for `my_tag`, `world.cells[neighbour_idx]`
      for `their_tag`) satisfies the gate: `Inducible` requires
      `cell.terrain == trigger`; `Repressible` requires `cell.terrain != trigger`.
      A tag that fails its gate is excluded from that pair's lookup entirely
      (both the `interaction_delta` sum and the `AdjacencyObserved` event at
      `sim.rs:301-307` — an organism that never satisfies its gate shouldn't
      generate observation evidence for a relationship it isn't actually
      exhibiting).
- [ ] Unconditional tags (no entry in the per-world conditional set) behave
      exactly as today — active on every terrain, no gate applied. Add a
      regression test asserting this explicitly (e.g. build a world with
      `conditional_tag_count = 0` and confirm `interaction_delta` matches the
      pre-096 formula for a hand-crafted matrix/species pair).
- [ ] Unit test: a conditional tag in `Inducible` mode contributes to
      `interaction_delta` only when its carrier occupies the trigger
      terrain, and not otherwise (same organism, same neighbours, terrain
      varied).
- [ ] Unit test: a conditional tag in `Repressible` mode contributes
      everywhere *except* the trigger terrain.
- [ ] Unit test: **a conditional `TagId` that isn't in this world's
      `active_tags` at all** doesn't panic and doesn't affect
      `interaction_delta` — the likeliest silent-panic site is indexing a
      per-slot conditional lookup sized for the wrong axis; keying by
      `TagId` and checking membership (not indexing by slot count) avoids
      this by construction, but test it directly.
- [ ] Determinism: same seed → same conditional-tag rolls (trigger terrain
      and mode), same as the existing `TagMatrix`/`active_tags` determinism
      guarantee. `tests/determinism.rs`'s `same_seed_yields_identical_history`
      test already asserts cell-by-cell equality across two same-seed runs
      (not hardcoded expected values), so it exercises this for free once
      `SimWorld::new_for_world` draws the new roll from `world`'s own RNG —
      confirm it still passes, don't need a new determinism test file.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `interaction_delta` (lines 265-311) — add the gate; `AdjacencyObserved` event emission (301-307) must respect it too. |
| `src/world.rs` | `TagId`/`TagSlot` (25-40), `TagMatrix` (55-76), `Species`/`tags` (81-96), `TerrainKind` (140-146), `Cell`/`terrain` (150-166), `SimWorld` (`active_tags`, 187; new conditional-tags field), `new_for_world` (238-267, insertion point for the new roll), `select_active_tags` (962-966), `generate_matrix` (982-1015). |
| `src/config.rs` | `TagConfig` (321-344) — add `conditional_tag_count`. |
| `tests/determinism.rs`, `tests/balance.rs` | Verify same-seed-equality assertions (not hardcoded values) still pass with the new RNG draw in `new_for_world` — checked this session, both use `assert_eq!(a.cells, b.cells)`-style comparisons, not fixed expected constants, so they're churn-safe by construction. |

---

## 🧩 Technical Context

- **Current behavior**: `interaction_delta` sums `world.matrix.get(their_tag, my_tag)`
  for every occupied Moore neighbor's tag against every tag the organism
  itself carries — pure tag-pair lookup, no terrain read anywhere in the
  formula (`sim.rs:265-311`). `TagMatrix` is rolled once per world
  (`generate_matrix`, `world.rs:982`) and never varies within a run.
  `TerrainKind` (`world.rs:140`) today only gates placement
  (`is_placeable`/`is_placeable_index`, `world.rs:748-758`) and feeds the
  temperature/light source model (tasks 085/086) — it never touches
  `TagMatrix` or `interaction_delta`.
- **Desired behavior**: a fixed, structural subset of `TagId`s (same subset
  in every world) is terrain-conditional; the specific terrain/mode is a
  per-world roll. `interaction_delta` gates those tags' participation in the
  matrix lookup by the carrying organism's current cell terrain.
- **Why `TagId`, not `TagSlot`**: `TagSlot` is this world's per-run position
  within its active subset, assigned by `select_active_tags`'s RNG sample
  (`world.rs:962-966`) — a different draw order every world. `TagId` is the
  stable, pool-wide identity (`world.rs:25-30`). The doc is explicit that
  *which glyphs* are conditional must be a structural, cross-world constant
  ("things the player learns from the manual, not from decoding a specific
  world") — only keying on `TagId` satisfies that; keying on `TagSlot` would
  make a different glyph conditional every world, contradicting the
  decision.
- **RNG stream churn**: adding a draw inside `SimWorld::new_for_world`
  (after `generate_matrix`, before/alongside `generate_terrain` at
  `world.rs:268`) shifts every downstream `StdRng` draw this world makes
  (terrain generation, environment sources, toxic zone placement, every
  `draw_species_tags`/`draw_species_name` call). This is expected and safe:
  `tests/determinism.rs`/`tests/balance.rs` assert same-seed *equality*
  between two independently constructed worlds, not fixed expected values —
  confirmed by reading both files this session — so the churn doesn't break
  any test, it just means a given seed's specific matrix/species/name
  outputs will differ from pre-096 runs. No test file encodes those as
  golden values today.

---

## 🔨 Suggested Implementation

1. `src/config.rs`: add `TagConfig::conditional_tag_count`, default `1` or
   `2`. Update `impl Default for TagConfig` and the RON asset
   (`assets/config/sim_config.ron`) to match.
2. `src/world.rs`: add a `Mode` enum (`Inducible`/`Repressible`) and a small
   struct/field on `SimWorld` holding the per-world roll, e.g.
   `conditional_tags: Vec<ConditionalTag>` where
   `ConditionalTag { tag: TagId, terrain: TerrainKind, mode: Mode }`. Add a
   private `roll_conditional_tags(active_tags: &[TagId], config: &TagConfig, rng: &mut StdRng) -> Vec<ConditionalTag>`
   function, called from `new_for_world` right after `select_active_tags`
   (only tags that landed in `active_tags` and fall within the first
   `conditional_tag_count` `TagId`s get an entry).
3. Add a lookup helper (e.g. `SimWorld::conditional_gate(tag: TagId) -> Option<&ConditionalTag>`
   or a small `HashMap`-free linear scan — the conditional set is tiny,
   ~1-2 entries, so a `Vec` scan is fine and avoids the `HashMap`-iteration
   ban for sim logic).
4. `src/sim.rs`: in `interaction_delta`'s double loop (`sim.rs:294-296`),
   resolve each `their_tag`/`my_tag` `TagSlot` to its `TagId` via
   `world.active_tags[slot.0 as usize]`, check the gate against the
   respective organism's current cell terrain, skip the pair (both the sum
   and the `AdjacencyObserved` push) if the gate fails.
5. Unit tests per Acceptance Criteria: inducible gating, repressible gating,
   unconditional regression, conditional-tag-not-active-this-world edge
   case, determinism (existing test suite exercises this, confirm it still
   passes).
6. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- **No parallel systems**: reuse `TagMatrix`/`interaction_delta` exactly as
  the doc requires — this is a gate on existing lookups, not a second
  matrix or a per-terrain intensity multiplier (both explicitly rejected in
  the doc's "Decided in discussion" section).
- **No `HashMap` iteration in sim logic** (`CLAUDE.md`) — the conditional
  set is small; a `Vec` scan or fixed-size array keyed by position is fine,
  do not reach for a `HashMap` for convenience.
- **Determinism**: all new rolls come from `world`'s own seeded `StdRng`,
  never `rand::rng()` — same discipline as every other worldgen draw.
- **This task does not touch the notebook or catalog UI.** The `(tag, terrain)`
  evidence track and the catalog badge are task 097's job — do not build
  them here even though the doc describes them in the same section; keeping
  096's surface to sim/worldgen internals only is what lets it ship without
  a UI dependency.
- **Guardrail context (not this task's job)**: the doc notes a
  descendant/species may only ever combine tags already active in that
  world — that's `abiogenesis-evolution-xenotypes.md`'s speciation
  guardrail (task 107), not enforced here. This task's conditional-tag roll
  should be readable by 107 later (a descendant inheriting a conditional
  tag inherits its world's existing gate, nothing new to roll), but no
  code coordination is required now.

---

## 🔗 Dependencies

- **Depends on**: 011 (hidden matrix generation), 036 (`TagSlot` vs `TagId`
  split), 038 (`select_active_tags`/per-world active pool), 066/067 (terrain
  generation and placement gating).
- **Blocks**: 097 (catalog badge + evidence track — needs this task's data
  model), 099 (zone-entry reveal — needs the gate to exist to detect).
- **Informs, not blocks**: 107 (evolution/speciation tag guardrail,
  `abiogenesis-evolution-xenotypes.md`) — relevant context for how a
  descendant inherits a conditional tag, not a coordination requirement for
  this task.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/096-mondo-vivo-conditional-tags-core.md)"$'\n\nExecute this task in the current project.'
```
