# Task 010 — Tag pool and per-species tag assignment

> **ID**: `010`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Give the world an **active tag pool** and give species **actual tags**, replacing the always-empty `tags: Vec::new()` from Phase 0. This is the foundation Phase 1 builds on: the hidden matrix (task 011) is indexed by active tags, and the adjacency effect (task 012) reads species' tags.

---

## 📋 Acceptance Criteria

- [ ] `SimWorld` exposes the world's active tags (e.g. `pub active_tags: Vec<TagId>`), populated at construction.
- [ ] Active tags are `TagId(0)..TagId(config.tags.active_tags_early)` — a **fixed** subset in Phase 1. Procedural per-world tag selection is explicitly out of scope here (`PROJECT_PLAN.md` assigns "active tags" selection to Phase 3 world generation).
- [ ] A helper assigns each species **1 to 3** tags (`config.tags.tags_per_species_min..=tags_per_species_max`), drawn from `active_tags` using `world.rng_mut()` — never a new/thread-local RNG.
- [ ] Same seed ⇒ identical tag assignment (determinism test).
- [ ] Assigned tags are always a subset of `active_tags` (no out-of-range `TagId`).
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `SimWorld.active_tags`, tag-assignment helper, `#[cfg(test)]` tests |
| `src/config.rs` | `TagConfig` — already complete (task 002), read-only here |

---

## 🧩 Technical Context

- **Current behavior**: `TagId(pub u8)` and `Species.tags: Vec<TagId>` exist (task 003) but nothing ever populates `tags`; `seed_phase0_organism` hardcodes `tags: Vec::new()`.
- **Desired behavior**: any species pushed into `world.species` after this task carries a real, in-range tag set.
- **Already available**: `TagConfig` (`src/config.rs`) has `global_tag_pool: 10`, `active_tags_early: 5`, `active_tags_late: 8`, `tags_per_species_min: 1`, `tags_per_species_max: 3` — no new config fields needed.

### Why a fixed active-tag subset for now

GDD §5.5 says active tags are a per-world procedural choice, and `PROJECT_PLAN.md`'s Phase 3 backlog explicitly lists "active tags" as part of "Procedural world generation". Picking `TagId(0..active_tags_early)` deterministically is functionally equivalent for Phase 1's purposes (the tags are nameless indices; only the matrix gives them meaning) and avoids building throwaway worldgen logic that Phase 3 will replace anyway.

---

## 🔨 Suggested Implementation

1. Add the field and populate it in `SimWorld::new`:

   ```rust
   pub active_tags: Vec<TagId>,
   ```

   ```rust
   active_tags: (0..config.tags.active_tags_early as u8).map(TagId).collect(),
   ```

2. A small helper, called wherever a species is pushed (currently just `seed_phase0_organism`, soon also task 013's palette):

   ```rust
   /// Draws 1..=3 tags for a new species from the world's active pool (GDD 5.5).
   fn draw_species_tags(world: &mut SimWorld, config: &SimConfig) -> Vec<TagId> {
       let n = world.rng_mut().random_range(
           config.tags.tags_per_species_min..=config.tags.tags_per_species_max,
       ) as usize;
       // sample without replacement from world.active_tags, using world.rng_mut()
   }
   ```

   Sampling without replacement keeps a species from carrying the same tag twice; a simple approach is a partial Fisher-Yates shuffle of a copy of `active_tags` truncated to `n`.

3. Update `seed_phase0_organism` to call this helper instead of `tags: Vec::new()`.

---

## ⚠️ Constraints and Caveats

- **Invariant 1**: only `world.rng_mut()`, never `rand::rng()`.
- Keep `TagId` semantics as an opaque index — no display names, no glyphs yet (that's presentation, not in scope).
- Don't touch `sim::step` in this task — tags exist but have no effect until task 012.

---

## 🔗 Dependencies

- **Depends on**: 005
- **Blocks**: 011, 013

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/010-tag-pool-species-tags.md)"$'\n\nExecute this task in the current project.'
```
