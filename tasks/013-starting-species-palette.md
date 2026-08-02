# Task 013 — Starting species palette, multiple species per world

> **ID**: `013`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Replace the Phase 0 placeholder (`seed_phase0_organism`: one hardcoded photolithic organism) with a small **starting palette** of multiple photolithic species, distinct enough in tags and thermal optimum that the matrix effect from task 012 becomes visible on screen — mirroring the P/Q setup in GDD §16.1-16.2.

---

## 📋 Acceptance Criteria

- [ ] At world creation/reseed, **2-3 photolithic species** are registered in `world.species`, each with its own `temp_optimum` (spread across the temperature gradient, GDD §16.1) and its own tag set (task 010's helper).
- [ ] Each species is seeded with at least one organism, in a spot consistent with its thermal niche (e.g. one in the cooler band, one in the hotter band), so it actually survives long enough to interact.
- [ ] Reproduction, death, and the matrix effect (task 012) all apply normally to every species — no special-casing per species in `sim::step`.
- [ ] Determinism preserved: same seed ⇒ identical palette and placement.
- [ ] Existing tests that build single-species worlds by hand are unaffected (they don't call the new palette function).
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | Replaces `seed_phase0_organism` with a multi-species seeding function |
| `src/config.rs` | `EnergyConfig`/`EnvironmentConfig` — read-only, no new fields expected |

---

## 🧩 Technical Context

- **Current behavior**: `seed_phase0_organism` pushes exactly one `Species` and one `Organism`, at `(width/2, 0)`, with `temp_optimum` read from that single cell.
- **Desired behavior**: a handful of species, each viable in a different part of the temperature gradient (`temperature_gradient_left = 0.2` → `temperature_gradient_right = 0.8`), each carrying tags from task 010, all photolithic (predator/decomposer arrive in tasks 014-015 and are **not** part of this palette).

### Why still hardcoded placement

Full procedural world generation (species count, placement, and objectives) is Phase 3 (`PROJECT_PLAN.md`). This task only needs *enough* species diversity for the matrix to produce observable emergence — not a generator. Mouse-driven placement by the player arrives in task 017; this task's placement stays a fixed part of world setup, same spirit as Phase 0's `seed_phase0_organism`.

---

## 🔨 Suggested Implementation

1. Rename/replace `seed_phase0_organism` with something like `seed_starting_palette(world: &mut SimWorld, config: &SimConfig)`, called from `spawn_world` in `world.rs` and from `input::reseed_world`.

2. For each of e.g. 2 species, pick a `temp_optimum` and a grid position along the temperature axis (`x`) at high light (`y = 0`, same rationale as Phase 0's placement):

   ```rust
   let temp_optima = [config.environment.temperature_gradient_left, config.environment.temperature_gradient_right];
   for &temp_optimum in &temp_optima {
       let tags = draw_species_tags(world, config); // task 010
       let species_id = SpeciesId(world.species.len() as u8);
       world.species.push(Species { metabolism: Metabolism::Photolithic, temp_optimum, temp_tolerance: config.energy.default_temp_tolerance, repro_threshold: config.energy.repro_threshold, tags });
       // place the seed organism at the x matching this temp_optimum, y = 0
   }
   ```

3. Keep the function's doc comment explicit that it's a Phase 1 placeholder, same spirit as the one it replaces, so a future Phase 3 worldgen task knows to replace this call site instead of extending it.

---

## ⚠️ Constraints and Caveats

- **Invariant 1**: tag/placement randomness only via `world.rng_mut()`.
- Don't introduce predator/decomposer species here — this task is strictly photolithic-only, to keep it decoupled from tasks 014-015.
- Don't add a species-selection UI here — that's task 017.

---

## 🔗 Dependencies

- **Depends on**: 010
- **Blocks**: 017

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/013-starting-species-palette.md)"$'\n\nExecute this task in the current project.'
```
