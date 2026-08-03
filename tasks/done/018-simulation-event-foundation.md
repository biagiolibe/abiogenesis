# Task 018 — Simulation event foundation

> **ID**: `018`
> **Category**: Architecture
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Phase 2's notebook is built by *consuming events*, not by inspecting the grid (`TECH_DESIGN.md` §4). Today `sim.rs` emits nothing: `step()` mutates `SimWorld` in place and returns `()`. This task adds the event data the rest of Phase 2 depends on — organism deaths, species extinctions, and raw per-tick adjacency observations (the evidence the confirmation engine in task 020 will weigh) — without breaking `step()`'s headless, pure-Rust testability.

This is the foundation task: 019 (observation log) and 020 (confirmation engine) both consume what this task produces.

---

## 📋 Acceptance Criteria

- [ ] `step()` stays pure Rust, callable without a Bevy `App` (invariant 2) — it gains an output parameter (e.g. `&mut TickEvents` or a returned `TickEvents` struct), not a dependency on `bevy::ecs`.
- [ ] A death (energy `<= 0`) records one `OrganismDied { cell: usize, species: SpeciesId }`.
- [ ] A species whose population goes from `> 0` (start of tick) to `0` (end of tick) records one `SpeciesExtinct { species: SpeciesId }`, in addition to the `OrganismDied` events for its last individuals.
- [ ] For every occupied cell, for every occupied Moore neighbour, for every `(exerter_tag, receiver_tag)` pair that has a non-zero matrix entry, one `AdjacencyObserved` record is produced, carrying enough information for task 020 to compute `weight = 1 / (1 + n_confounders)` (GDD §7) — at minimum the two tags and a confounder count. Document the confounder-counting rule chosen (see Technical Context) in a doc comment, since it's the one open design call in this task.
- [ ] `Message` (Bevy 0.19's event trait — see `AppExit`/`MessageWriter` in `input.rs`) types are derived for `OrganismDied` and `SpeciesExtinct` at minimum, registered in `SimPlugin`, and drained from the pure-Rust output into `MessageWriter`s by a system running right after `advance_tick`.
- [ ] Existing `sim.rs` tests (Phase 0/1) keep passing unmodified — this task only adds an output channel, it doesn't change any energy/death/reproduction arithmetic.
- [ ] New unit tests, in the same hand-built-world style as the existing `sim.rs` test module, cover: a death producing exactly one `OrganismDied`; a species' last organism dying producing both `OrganismDied` and `SpeciesExtinct`; an adjacency between two tagged organisms producing the expected `AdjacencyObserved` record(s); and the confounder count matching a hand-checked scenario (e.g. GDD §7's example: one neighbour tag → 0 confounders → weight 1.0, three confounding tags present → weight 0.25).
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `step()` gains the event-producing side channel; `advance_tick` drains it into `MessageWriter`s |
| `src/world.rs` | `TagId`, `SpeciesId`, `TagMatrix::get` — read, not modified |
| `src/config.rs` | No changes expected — `NotebookConfig` (`confirmation_threshold`, `observation_weight_numerator`) already exists for task 020 to consume |

---

## 🧩 Technical Context

### Why a pure-Rust output, not `MessageWriter` inside `step()`

`step()` is called directly from unit tests with no `App` (see `sim.rs`'s existing test module). If `step()` took `MessageWriter<T>` parameters it would stop being callable that way. Pattern: `step()` takes `&mut TickEvents` (a plain struct with `Vec<OrganismDied>`, `Vec<SpeciesExtinct>`, `Vec<AdjacencyObserved>` fields, or three separate `Vec`s) or returns one, and `advance_tick` — which already has Bevy `ResMut`/`MessageWriter` access — is the only place that touches Bevy event machinery:

```rust
fn advance_tick(
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    mut died: MessageWriter<OrganismDied>,
    mut extinct: MessageWriter<SpeciesExtinct>,
    // AdjacencyObserved likely wants its own writer, or bundling into a
    // single MessageWriter<TickEvents> if that turns out cleaner — decide
    // based on how task 019/020 want to consume them.
) {
    if progress.remaining() == 0 { return; }
    let events = step(&mut world, &config);
    died.write_batch(events.deaths);
    extinct.write_batch(events.extinctions);
    // ...
    progress.remaining -= 1;
    // ...
}
```

### Species extinction detection

`step()` already iterates `world.cells` (snapshot) and writes `world.scratch`. Extinction needs a before/after population count per species. Cheapest approach: accumulate a `Vec<u32>` (len `world.species.len()`) of pre-tick populations by scanning `world.cells` once at the top of `step()` (or reuse `species_stats`-style counting from `ui.rs`, but headless — don't import `ui.rs` into `sim.rs`, invariant 2), then decrement it live as deaths are recorded, and check for `1 -> 0` transitions.

### Adjacency observation and the confounder count — the open design call

The tick loop already computes, for each occupied cell, the set of occupied Moore neighbours and iterates `(their_tag, my_tag)` pairs to sum `interaction_delta` (`sim.rs`, step 3). This task threads a parallel emission through that same loop instead of just accumulating a sum.

**Recommended confounder rule** (derive from GDD §7's own example — "isolated observation = 1.0 weight, one with three confounding tags = 0.25"): for a given receiver organism `O` and a specific candidate hypothesis cell `(exerter_tag X, receiver_tag Y)` observed via neighbour `N` carrying `X`, the confounders are the **other distinct tags** present among `O`'s neighbours this tick, excluding `X` itself — i.e. how many *other* things could plausibly be affecting `O`'s energy this tick besides the `X → Y` relationship being observed. This makes the count per-organism-per-hypothesis rather than per-neighbour-pair; a cell with exactly one neighbour carrying only tag `X` (and `Y`'s own tags don't count as confounders for effects on `Y` from `X`) yields `n_confounders = 0` → `weight = 1.0`, matching the GDD's "isolated observation" example directly.

Write this rule as a doc comment on `AdjacencyObserved` or the function that produces it, since task 020 depends on the exact semantics, not just the struct shape.

---

## 🔨 Suggested Implementation

1. In `sim.rs` (or a new `src/events.rs` if `sim.rs` gets too large — check current line count first), define:
   ```rust
   #[derive(Debug, Clone, Copy, Message)]
   pub struct OrganismDied { pub cell: usize, pub species: SpeciesId }

   #[derive(Debug, Clone, Copy, Message)]
   pub struct SpeciesExtinct { pub species: SpeciesId }

   #[derive(Debug, Clone, Copy, Message)]
   pub struct AdjacencyObserved {
       pub receiver_species: SpeciesId,
       pub exerter_tag: TagId,
       pub receiver_tag: TagId,
       pub n_confounders: u32,
   }

   #[derive(Debug, Default)]
   pub struct TickEvents {
       pub deaths: Vec<OrganismDied>,
       pub extinctions: Vec<SpeciesExtinct>,
       pub adjacencies: Vec<AdjacencyObserved>,
   }
   ```
2. Change `step()`'s signature to `pub fn step(world: &mut SimWorld, config: &SimConfig) -> TickEvents`, threading pushes into a local `TickEvents` at the existing death site (step 6) and adjacency loop (step 3). Compute pre-tick population counts at the top for extinction detection.
3. Register the `Message` types in `SimPlugin::build` (`app.add_message::<OrganismDied>()` or the 0.19-equivalent registration — check what `AppExit` uses, since it's the existing precedent in this codebase).
4. Update `advance_tick` to capture `step()`'s return value and drain it into the appropriate `MessageWriter`s.
5. Port the existing `sim.rs` tests to the new `step()` signature (they'll need to bind/ignore the returned `TickEvents`), then add the new tests listed in Acceptance Criteria.

---

## ⚠️ Constraints and Caveats

- Don't consume these events anywhere yet — no `notebook` module, no UI. This task only produces them; 019/020 read them.
- Don't let `TickEvents` or its production path depend on `bevy::render` or `bevy_egui` (invariant 2) — `sim.rs`/`world.rs` stay headless.
- Keep the adjacency emission **additive to**, not a replacement of, the existing `interaction_delta` accumulation — the energy-tick arithmetic must not change (regression risk: re-run the full existing test suite, not just the new tests).
- If `AdjacencyObserved` volume turns out large (up to 8 neighbours × up to 3×3 tag pairs × every occupied cell, every tick) and profiling matters later, that's a Phase-2-final-tuning concern, not this task's — correctness first.

---

## 🔗 Dependencies

- **Depends on**: 012 (matrix adjacency effect), 015 (residue/death path)
- **Blocks**: 019, 020

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/018-simulation-event-foundation.md)"$'\n\nExecute this task in the current project.'
```
