# Task 050 — Remove auto-placed starting organisms; the player seeds the first world

> **ID**: `050`
> **Category**: Feature / Balance
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-06 playtest session

---

## 🎯 Objective

`worldgen::generate_starting_palette` currently places `config.worldgen.starting_species_count` (default `2`) organisms automatically on the grid at fixed, evenly-spaced positions along `y = 0`, at every world's start (including world transitions, task 045). Playtest feedback: this undercuts the game's own premise — "Abiogenesis" is about seeding life into a sterile world, and the `Seed` action (GDD §6) already exists as the player's tool for exactly that. Starting with organisms already alive and growing also interacts awkwardly with the `Coexistence` objective (task 042), which can require more species coexisting than the number actually placed at start (see the 2026-08-06 playtest note in `PROJECT_PLAN.md`).

Decision from the 2026-08-06 planning discussion: **no organisms are placed automatically** — every species generated for a world (`starting_species_count + extra_available_species_count`, or whatever this task's config changes make of that split) exists only in the `available` pool, ready for the player to place via `Seed`. This is a bigger change than "randomize the position" (also discussed and rejected in favor of this) because it changes what a world looks like the instant `Playing` is entered: zero living organisms.

**This has a direct, must-fix technical consequence**: `objectives.rs::is_total_extinction` treats "species exist but nothing is placed" as extinction —

```rust
fn is_total_extinction(world: &SimWorld) -> bool {
    !world.species.is_empty() && world.cells.iter().all(|cell| cell.organism.is_none())
}
```

— today this is guarded only for the synchronous instant between `SimWorld::new_for_world` and `generate_starting_palette` placing organisms (see the function's own doc comment). With nothing ever auto-placed, this condition would be true from the moment `Playing` starts until the player's first successful `Seed`, instantly failing every world before the player can act at all.

---

## 📋 Acceptance Criteria

- [ ] `generate_starting_palette` (or its replacement) no longer places any organism on the grid — every generated species goes into the available pool only.
- [ ] `is_total_extinction` (or the system that calls it, `objectives::evaluate_current_objective`) no longer fails a world just because the player hasn't placed anything yet — decide and document the new rule (e.g. "extinction can't trigger until at least one organism has ever existed this world," or "grace period of N ticks/eras before the check activates," or gate it behind `EraState::Advancing` only starting after the player's first `Seed`). Pick the simplest rule that can't be trivially exploited (e.g. don't let a player dodge extinction forever by never seeding).
- [ ] The `Coexistence` objective's `min_species` clamp (`worldgen::generate_objective`) still makes sense against a world where *nothing* starts placed — re-check the clamp's rationale now that "already placed" isn't a floor at all.
- [ ] Manual verification: `cargo run`, start a run, confirm the grid starts empty, confirm placing organisms via `Seed` works exactly as before, confirm the world doesn't instantly fail on entering `Playing` or on the first few ticks before any `Seed`.
- [ ] `cargo clippy -- -D warnings` clean, `cargo test` green — update tests that assumed `generate_starting_palette`'s old placement behavior (`worldgen.rs`'s own tests, `tests/determinism.rs`, `tests/action_effects.rs`, `tests/balance.rs` all currently call it expecting placed organisms).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/worldgen.rs` | `generate_starting_palette` (the placement loop to remove), `StartingPalette::placed` (becomes always empty, or the type simplifies — your call), `generate_objective`'s `min_species` clamp. |
| `src/objectives.rs` | `is_total_extinction` — needs the new "hasn't started yet" exemption. |
| `tests/determinism.rs`, `tests/action_effects.rs`, `tests/balance.rs`, `src/worldgen.rs`'s own test module, `src/input.rs`'s test module | All currently call `generate_starting_palette` expecting organisms on the grid afterward — will need updating to explicitly `Seed`/place organisms themselves where the test's premise depends on it. |

---

## 🧩 Technical Context

<!-- TODO: add relevant code snippets and file paths -->

**Current behavior**: `generate_starting_palette` places `starting_species_count` organisms deterministically (evenly spaced, `y = 0`) and adds `extra_available_species_count` further species to the pool only. `is_total_extinction` relies on the placement happening synchronously in the same call that creates the species registry, per its own doc comment ("in practice, seeding the starting palette pushes species and places their organisms in the same synchronous call, so no real tick ever observes 'species exist, none are placed yet'") — this task removes the premise that comment depends on.

**Desired behavior**: a world starts with a populated *available* pool (species defined, tagged, positioned nowhere) and an empty grid. The player's first meaningful action is choosing what to seed and where. Total extinction only becomes a real failure condition once there's been life to lose.

---

## 🔨 Suggested Implementation

1. Read `generate_starting_palette` in full, decide how much of the "placed" concept survives (maybe `StartingPalette` collapses to just `available: Vec<Species>` — check every caller of `.placed` first, e.g. `worldgen.rs`'s own tests).
2. Remove the placement loop; keep tag-drawing/temp-optimum-assignment logic for every generated species (still needs *some* stand-in temperature-optimum spread across the world's range, since there's no longer a placement position to read `world.get(x, 0).temperature` from — reuse the "extra species" loop's approach, which already doesn't place).
3. Fix `is_total_extinction`: the simplest option is probably tracking "has any organism of this world's species ever been placed" as a small piece of state (e.g. a bool/counter on `SimWorld`, reset by `start_world`/`build_world`) and only enabling the extinction check once true. Consider edge cases: a player who seeds once, has that organism die immediately, and never seeds again — should that still fail the world? (Almost certainly yes — that's a real total extinction, just delayed by one action.)
4. Re-derive `Coexistence`'s `min_species` clamp rationale — it currently clamps to `species_count` (all generated species, placed or not), which stays valid since it was never actually checking "placed" species count, only the generated pool size.
5. Update every test call site listed above.
6. Manual verification with `cargo run`.

---

## ⚠️ Constraints and Caveats

- **Determinism**: whatever "has anything ever been placed" tracking you add must not introduce non-determinism (invariant 1, TECH_DESIGN.md §5) — a plain field on `SimWorld`, set by ordinary game logic, is fine.
- **Don't let this become unwinnable**: if extinction can trigger before the player has had a real chance to act, that's worse than the bug being fixed — err toward a rule that's generous to the player at the boundary.
- **Re-check `009-determinism-balance-tests.md`/`tests/balance.rs`'s existing assumptions**: several of those tests likely assume a populated grid immediately after world construction; they'll need an explicit `Seed`-equivalent step inserted.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none, but touches the same `is_total_extinction`/`objectives.rs` territory as task 047 — coordinate if both are in flight at once to avoid a merge headache.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/050-no-auto-placed-starting-organisms.md)"$'\n\nExecute this task in the current project.'
```
