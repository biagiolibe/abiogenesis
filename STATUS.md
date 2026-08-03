# Implementation Status — Phase 0, Phase 1, and Phase 2 (complete)

What exists today in Abiogenesis, feature by feature, with the technical solution behind each one. Written as a snapshot after task 025 — Phase 2 is fully done, both tracks; see `tasks/done/` for the task-by-task history and `TECH_DESIGN.md` for the architectural decisions this document assumes.

---

## How to play, today

**Goal**: seed life on an alien world governed by a hidden `tag × tag` matrix — every species secretly carries 1–3 tags, and adjacency between tagged organisms silently helps or hurts their energy. Nothing tells you the matrix directly; you deduce it from what you observe, then spend a limited action budget each era to actively probe it.

**Controls**:
- **Action selector** (HUD, right side): pick `Seed`, `Stress`, `Cull`, or `Splice` — this decides what a left-click does
- **Left click** an empty cell in `Seed` mode → places an organism of the currently-selected species, costs 1 action point
- **Left click** any cell in `Stress` mode → shifts that cell's temperature by a fixed step, costs 1 action point
- **Left click** an occupied cell in `Cull` mode → removes the organism there (no residue left behind), costs 1 action point
- **`Splice` mode** → opens a small editor panel (pick a source species, then swap one of its tags or shift its thermal optimum); "Apply splice" creates a **new** species, costs 2 action points
- **Space** → starts an era: the simulation auto-advances a batch of ticks with animation, then the action budget refills to 3
- **S** → advances exactly one tick manually, for fine-grained observation
- **R** → reseeds the world (new hidden matrix, new terrain, new starting palette, fresh action budget) — allowed even while an era is advancing
- **Tab** → opens/closes the Notebook window
- **F1** (dev builds only) → cycles a debug heatmap overlay (temperature / toxicity / light), stripped entirely from release builds
- **Esc** → quits

**The loop**:
1. Seed organisms of different species near each other on the grid, spending action points from a 3-per-era budget (GDD §6).
2. Advance ticks (`Space` or `S`). Each organism has a metabolism (photolithic, predator, decomposer) and gains/loses energy from light, temperature, crowding, and — critically — the tags of its occupied Moore neighbours, summed through the hidden matrix.
3. Deaths and species extinctions are the *salient* events: they land in the Notebook's **Observation log** section, each tagged with the era they happened in. (Note: `Cull`-ed organisms are removed by player fiat, not by the tick algorithm, so they don't generate a log entry — a deliberate GDD §5.6 reading, but it means a species can be silently erased this way.)
4. Every tick, every occupied-neighbour pair with tags that actually interact (non-zero matrix entry) contributes weighted evidence for that `(exerter_tag, receiver_tag)` hypothesis — an isolated observation counts fully, one crowded with other confounding tags counts for less (`weight = 1 / (1 + confounders)`).
5. Once a tag pair's accumulated evidence crosses the confirmation threshold, that cell of the **Hypothesis grid** (Notebook, second section) flips from `?` to `+!`/`-!` — the sign only, never the magnitude, so the exact strength still has to be inferred from behavior.
6. The Notebook's **Catalog** section lists every active tag (as a colored dot — tags are always glyphs/colors, never raw numbers) and every species' readable genome: metabolism, temperature range, and which tag dots it carries.
7. Beyond just watching, `Stress` and `Splice` let you actively perturb the world to generate more (and more targeted) evidence — e.g. stress a cell to isolate a temperature effect from a matrix effect, or splice a new species variant to test a specific tag combination.

Phase 2 is now complete — the deduction game (notebook + confirmation) and the action economy (budget + all four actions) both exist. Phase 3 (procedural worldgen, real objectives/win conditions, main menu) is backlog, not yet started.

---

## 1. Application shell

**What**: a Bevy app with a real window, deterministic simulation, 2D rendering, an egui HUD, and keyboard/mouse input — structured as one `Plugin` per concern.

**How**: `main.rs` wires seven plugins onto `App`: `ConfigPlugin`, `WorldPlugin`, `SimPlugin`, `GridRenderPlugin`, `UiPlugin`, `NotebookPlugin`, `InputPlugin`. Two crate halves:
- `abiogenesis` (lib): `config`, `world`, `sim`, `state` — pure simulation, no `bevy::render` or `bevy_egui` dependency, runnable headless.
- `abiogenesis` (bin, `main.rs` + `render`/`ui`/`input` modules): the presentation layer that turns the lib into a playable window.

This split is what lets determinism/balance tests run without a GPU or window (`cargo test` spins up `SimWorld` + `sim::step` directly, no `App`).

---

## 2. Configuration (`src/config.rs`)

**What**: every numeric coefficient in the game lives in one `SimConfig` resource, grouped by domain (`grid`, `environment`, `time`, `energy`, `tags`, `notebook`), each with GDD §5.9-sourced defaults.

**How**: plain structs with `Default` impls, no magic numbers elsewhere in the codebase (project convention, enforced by review rather than tooling). `ConfigPlugin` just calls `init_resource::<SimConfig>()`. Not yet hot-reloadable (RON migration is backlog, Phase-tuning item) — read-only at runtime today.

---

## 3. World state (`src/world.rs`)

**What**: `SimWorld` is the single source of truth for the simulation — a dense grid of `Cell`s, the species registry, the hidden tag matrix, and the seeded RNG. Not ECS: Bevy entities only exist for rendering (TECH_DESIGN.md §3.1), so the tick algorithm is a plain Rust sweep over `Vec<Cell>`, independently testable and fully deterministic.

**How**:
- `Cell { temperature, light, toxicity, organism: Option<Organism>, residue }` — single occupancy, no stacking (GDD §5.1).
- `Organism { species: SpeciesId, energy }`, `Species { metabolism, temp_optimum, temp_tolerance, repro_threshold, tags }`.
- Double-buffered: `SimWorld` holds both `cells` (read-side snapshot) and a `pub(crate) scratch` (write-side), swapped at the end of every tick (`sim::step`) — this is *the* mechanism that keeps the whole tick order-independent (TECH_DESIGN.md invariant 1): every system this tick reads only from `cells`, writes only to `scratch`, and no cell's outcome depends on which order cells were visited in.
- `rng: StdRng`, seeded once at `SimWorld::new(seed, config)`, exposed only via `rng_mut()` so nothing can clone it out and desync determinism. `next_seed()` draws the next reseed value from this same stream — reseeding (`r` key) never touches the system clock.
- `moore_neighbours(x, y)` — the 8-cell neighbourhood, clipped at grid borders (a corner cell has 3, an edge cell 5), used by every spatial mechanic below.

---

## 4. Environment (§5.2)

**What**: two static gradients (light falls top→bottom, temperature rises left→right, deliberately crossed axes so they carve 2D niches) plus a toxic corner zone, all seeded once at world construction — and, since task 016, all three scalars now diffuse toward their neighbours' mean every tick.

**How**:
- `apply_gradients` (called once in `SimWorld::new`) linearly interpolates `light`/`temperature` across the grid and stamps a fixed-size toxic rectangle in the bottom-right corner. Exact at `t=0`/`t=1` via a custom `lerp`, not `f32::lerp`, so grid-extreme cells match the GDD's baseline values exactly (tested).
- `diffuse_environment(config)` (task 016): for every cell, blends `temperature`/`light`/`toxicity` toward the mean of its Moore neighbours at `diffusion_rate` (0.05/tick) — reads `self.cells` (snapshot), writes `self.scratch`, same double-buffering discipline as everything else. `sim::step` calls it every tick, right after the scratch copy. A uniform field is a fixed point (tested); a perturbed cell smooths out over ticks without overshoot, staying in `[0,1]`.
- `residue` is explicitly *not* diffused — it's a separate mechanic (decay + decomposer extraction, §7 below), not an "environmental scalar" in the GDD §5.2 sense.

---

## 5. The tick algorithm (`src/sim.rs::step`, GDD §5.6)

**What**: the heart of the simulation — one `step(world, config)` call advances every organism by one tick: environmental diffusion, metabolic gain, matrix-driven interaction, upkeep costs, death, and reproduction. Pure function, no Bevy `App` required, so it's exhaustively unit-tested headless.

**How**, in the fixed order `step` actually executes:

1. **Scratch reset**: `world.scratch.copy_from_slice(&world.cells)` — write-side starts as a copy of the previous tick.
2. **Diffusion** (§4 above) — writes into `scratch`.
3. **Residue decay pre-pass**: every cell's `residue` drops by `residue_decay` per tick (floored at 0), independent of whether it currently holds an organism.
4. **Predation pre-pass** (§6 below) and **decomposition pre-pass** (§7 below) — both shared-resource accumulator passes computed from the snapshot, *before* the main loop.
5. **Main per-cell loop**, for every occupied cell, reading only `world.cells` (never `scratch`) for neighbour lookups:
   - **Environmental fitness**: Gaussian `env_fit(temperature, optimum, tolerance) = exp(-(Δt)² / (2·tolerance²))` — how close the cell's temperature is to the species' optimum.
   - **Metabolic gain**: `Photolithic → light × photolithic_metabolism_gain × fit`; `Predator`/`Decomposer` pull their gain from the accumulator arrays computed in step 4.
   - **Matrix adjacency effect** (§8 below): summed additive delta from every occupied Moore neighbour's tags.
   - **Costs**: a metabolism-specific base upkeep, plus a carrying-capacity penalty (`crowd_factor × occupied_neighbour_count`) — this crowding term is what turns unbounded photolithic growth into an S-curve that stabilizes (Phase 0's whole milestone).
   - **Energy update**: `new_energy = energy + gain + interaction_delta − upkeep − crowding − predation_loss − (implicit via decomposer path)`.
   - **Death**: `new_energy ≤ 0` clears the organism and stamps `residue = residue_on_death`, overwriting whatever residue/decay was there — a death this tick always wins.
   - **Reproduction**: if `new_energy ≥ repro_threshold`, picks a random empty Moore neighbour (via `world.rng_mut()`, so it's deterministic given the seed) and splits off a child at `repro_cost` energy; re-checks `scratch` at write time so two parents can't double-claim the same birth cell.
6. **Swap**: `mem::swap(&mut world.cells, &mut world.scratch)`, `world.tick += 1`.

**Why this shape**: TECH_DESIGN.md's "Tick Processing Order" decision is double buffering over a shuffled-iteration-with-guard alternative — no guard to maintain, no dependency on visitation order, newborns can't act the same tick they're born (by construction, since they only exist in `scratch`, never scanned by the loop that's iterating `cells`).

---

## 6. Predator metabolism (task 014, GDD §5.4)

**What**: predators drain energy from occupied Moore neighbours instead of photosynthesizing — the first mechanic where one cell's outcome depends on a *different* cell's state.

**How**: a **pre-pass accumulator**, not a direct write into a neighbour's `scratch` entry (that would make the outcome depend on scan order). Before the main loop: for every predator cell, `drawn = min(predator_drain_cap × fit, sum of occupied neighbours' energy)`, split evenly across those neighbours; `predation_gain[idx] = drawn` (the predator's own gain), `predation_loss[n] += share` for each prey neighbour `n`. The main loop later folds `predation_gain[idx]` into `gain` and subtracts `predation_loss[idx]` from the energy update. Two predators sharing one prey each compute independently from the same snapshot — deterministic regardless of which one is processed first, though (as documented) their combined draw isn't capped against each other, only against `drain_cap` individually.

**Balance** (GDD §5.9 quick-check, verified by test): an isolated predator with no prey collapses in exactly `⌈seed_energy / predator_upkeep⌉ = ⌈5.0/0.7⌉ = 8` ticks.

---

## 7. Decomposer metabolism and the residue cycle (task 015, GDD §5.4)

**What**: decomposers extract energy from residue in their own cell *and* Moore neighbours (unlike predation, decomposition includes the acting cell itself — a decomposer can sit directly on dead matter). Closes the loop *death → residue → decomposer bloom → (via the matrix) fertilizes photolithics → new biomass → new deaths* (GDD §16.3).

**How**: extends task 014's accumulator pattern with two differences, documented in `TECH_DESIGN.md` §6:
- **Source scope**: `sources = {own cell} ∪ moore_neighbours`, filtered to residue `> 0`.
- **Order**: the decomposer pre-pass reads `world.scratch`'s residue — i.e. *after* the decay pre-pass already ran — so decay and extraction compose (decay first) instead of one clobbering the other. Extraction is distributed proportionally to each source's residue share, then a final `.max(0.0)` clamp on the whole accumulator application guards against multiple decomposers overdrawing the same pool (each sizes its draw against the same undrained snapshot, so their combined total can exceed what's there — the clamp is the correctness backstop, not the primary mechanism).

---

## 8. Tags and the hidden matrix (tasks 010–012, GDD §5.5)

**What**: the game's central mystery. Every species carries 1–3 opaque tags from a global pool; a hidden `tag × tag` matrix defines how adjacency between tagged organisms affects energy — this is what the player is meant to reverse-engineer in Phase 2's notebook.

**How**:
- **Tag pool**: `TagId(u8)`, indexed directly (assumes the active set is the contiguous range `0..active_tags_early` — noted as a Phase 3 risk if world generation ever picks a non-contiguous subset).
- **Species tag assignment** (`draw_species_tags`): 1–3 tags sampled without replacement from `world.active_tags`, using the world's own RNG.
- **Matrix generation** (`generate_matrix`, task 011): each off-diagonal `(exerter, receiver)` cell independently becomes non-zero with probability `matrix_density` (~40%), diagonal always 0 (a tag never affects itself). Then a **negative 3-cycle** is forced among 3 random active tags — `A→B`, `B→C`, `C→A` all negative — overwriting whatever the random pass produced there. This guarantees at least one rock-paper-scissors-style coexistence relationship exists (GDD §5.8's cyclicity anti-degeneration lever), because there's no efficient closed-form way to *sample* a sparse asymmetric matrix guaranteed to contain a negative cycle — forcing it after the fact always terminates, rejection sampling might not.
- **Adjacency effect in the tick** (task 012): for every occupied Moore neighbour, for every `(their_tag, my_tag)` pair, sum `matrix.get(their_tag, my_tag)` into `interaction_delta`, added straight into the energy update — additive and linear (TECH_DESIGN.md invariant 4), read only from the snapshot.

---

## 9. Starting palette (task 013)

**What**: a placeholder for Phase 3's real procedural world generation — two photolithic species seeded at opposite temperature extremes (cold/left, hot/right) on the top (brightest) row, so the matrix adjacency effect has more than one species to act on out of the box.

**How**: `seed_starting_palette`, called once from `WorldPlugin`'s `Startup` system and again on every `r` (reseed). Explicitly marked "do not extend — replace its call sites" once Phase 3 world generation exists.

---

## 10. Rendering (`src/render.rs`)

**What**: one sprite per grid cell, colored by simulation state, plus the camera that projects the grid into the window — kept strictly read-only against `SimWorld` (TECH_DESIGN §3.1: entities are a view, never a data source).

**How**:
- `spawn_camera` (`GridRenderPlugin`, `Startup`): an orthographic `Camera2d` with `ScalingMode::AutoMin` pinned to the grid's pixel size, so a bigger window just letterboxes rather than clipping. Tagged `GridCamera` (see §12) to distinguish it from the dedicated HUD camera.
- `spawn_grid`: one `Sprite` entity per cell, positioned once via `cell_position(x, y, w, h)` (grid centered on the origin, row 0 at the top — Bevy's Y grows upward while row index grows downward, so the mapping flips Y).
- `sync_grid_colors` (`Update`): every frame, recomputes each sprite's `Sprite::color` from `cell_color`, the single place that decides how a cell looks: occupied cells get a per-species hue (`golden-angle` hue step so consecutive `SpeciesId`s are visually distinct with no manual palette) and lightness scaled by energy fraction toward `repro_threshold`; residue-only cells get a desaturated amber scaled by remaining residue; empty cells get a faint grey scaled by `light`.
- `world_to_cell(world_pos, width, height)` (task 017): the exact algebraic inverse of `cell_position`, rounding to the nearest cell and rejecting anything outside `[0,width) × [0,height)` — this is what turns a mouse click into a grid coordinate (§13).

---

## 11. Game/era state machine (`src/state.rs`, task 007)

**What**: two nested Bevy `States` mirroring GDD §16.4's Plan → Advance → Observe loop.

**How**: `GameState` (`Loading → MainMenu → Playing`, `MainMenu` unreachable until Phase 3's real menu) and a `SubStates` `EraState` (`Planning`, `Observing` default, `Advancing`) that only exists while `GameState::Playing`. `Planning` is a stub today — becomes real once Phase 2's action budget exists.

**Era animation** (`SimPlugin`, task 007/009): `EraProgress` resource counts down remaining ticks; `advance_tick` runs in `FixedUpdate` at `era_tick_hz` (20 Hz, presentation-only — never changes simulation outcomes), gated to `EraState::Advancing`, calling `sim::step` once per fixed tick and transitioning back to `Observing` when the countdown hits zero. A single `Update`-scheduled guard prevents a stray extra `FixedUpdate` execution from running one tick too many right at the state boundary.

---

## 12. HUD (`src/ui.rs`, tasks 008 & the mid-session fix)

**What**: an egui side panel showing era/tick/seed/state, live per-species population and average energy, and (task 017) the species selector for seeding.

**How, and why it needed a dedicated camera**: `bevy_egui` derives its own paint canvas (`RawInput::screen_rect`) from whichever camera carries the primary egui context, specifically that camera's `physical_viewport_rect()` — not the window. Early on this was the same camera rendering the grid, and cropping *that* camera's `Viewport` to reserve room for the HUD (so the panel wouldn't draw over the grid) also cropped away the area egui itself could paint into, making the panel invisible in some configurations. Fixed by giving egui its own camera:
- `spawn_hud_camera`: a second `Camera2d`, `order: 1` (composites after the grid camera), `ClearColorConfig::None` (doesn't erase the grid's output), `RenderLayers` no grid entity is ever assigned to (renders nothing of the scene, only the egui overlay), carrying `PrimaryEguiContext`. `EguiGlobalSettings::auto_create_primary_context` is disabled so `bevy_egui` doesn't instead auto-attach to the grid camera (the first one spawned).
- `reserve_hud_viewport` still crops the *grid* camera's `Viewport` by `HUD_WIDTH` (260px) every frame (tracks window resizes) — but now that only affects what the grid camera draws, not egui's canvas.
- `hud_panel` (`EguiPrimaryContextPass`) builds a full-window background-layer `Ui` and shows an `egui::Panel::right` inside it, `exact_size(HUD_WIDTH)`, `resizable(false)`.
- `SelectedSpecies(SpeciesId)` resource (task 017): a UI intent, not simulation state, written by the HUD's radio buttons and read by `input.rs`'s click handler — same ownership rationale as `EraProgress` living in `sim.rs` but being written from `input.rs`.

---

## 13. Input (`src/input.rs`, tasks 007 & 017)

**What**: keyboard shortcuts for the core loop, plus the first real player action — seeding an organism by clicking a grid cell.

**How**:
- `space` — starts an era (`EraProgress::start` + transition to `Advancing`), ignored while already `Advancing`.
- `s` — advances exactly one tick directly (`sim::step`), no state transition; useful for fine-grained observation. Also ignored mid-`Advancing`.
- `r` — reseeds: draws a new seed from the *current* world's RNG (never the system clock), rebuilds `SimWorld` from scratch, re-runs `seed_starting_palette`, cancels any in-flight era, and (task 022 onward) resets `ActionBudget`, `MatrixKnowledge`, the observation log, `SelectedSpecies`, and `SpliceDraft` — everything that would otherwise refer to a world/matrix/species registry that no longer exists. Allowed even mid-`Advancing` — a full reset legitimately invalidates whatever was playing.
- `Esc` — quits (`AppExit`).
- **`clicked_cell` helper** (task 023): the window-cursor → *grid* camera (`Query<..., With<GridCamera>>`, see §12) → world-position → grid-coordinate pipeline task 017 built for `Seed`, factored out so every click action reuses it instead of re-deriving the edge cases (off-grid clicks, no cursor, etc.).
- **Seed action** (task 017, extended by 022): on left-click while `ActionMode::Seed`, `Observing`, and affordable, places `Organism { species: selected.0, energy: seed_energy }` in an empty clicked cell and spends `action_costs.seed` (1) point. An unaffordable click does nothing at all, not even the empty-cell check.

This is one of several write paths from the UI/input layer into `SimWorld`, all gated the same way (`Observing`-only, budget-check-then-decrement) — `TECH_DESIGN.md` §3.3 scopes the `Ui`/input layer as read-only except for turning player intent into exactly this kind of mutation.

---

## 14. Action budget economy (`src/sim.rs`, task 022, GDD §6)

**What**: a per-era pool of action points (3 by default) that `Seed` — and, from tasks 023-025, every other player action — spends from. No new `EraState`: `Observing` already doubles as "observe last era, plan the next" (its existing doc comment), so the confirmed design keeps budget-spending there rather than adding a `Planning` state.

**How**: `ActionBudget { points_remaining: u32 }` with `refill(points)`/`try_spend(cost) -> bool`. Initialized to a full budget at `SimPlugin::build` time (reading `config.time.point_budget_per_era` off `SimConfig`, already inserted synchronously by `ConfigPlugin` — same pattern `NotebookPlugin` uses for `MatrixKnowledge`'s sizing), so the very first `Observing` window isn't stuck at the `Default`-derived `0`. Refilled inside `advance_tick`'s existing "era just ended" branch — once per era transition, not once per frame. Every action system (§13, §15-17 below) checks affordability *before* committing its effect and decrements only on actual success — an unaffordable or otherwise-invalid click spends nothing.

---

## 15. Stress action (`src/input.rs`, task 023, GDD §6)

**What**: the second player action — click a cell to shift its temperature by a fixed step, costing 1 action point. Works on empty and occupied cells alike (it targets the environment, not an organism), unlike `Seed`.

**How**: `stress_on_click` mirrors `seed_organism_on_click`'s budget-then-effect shape but with occupancy no longer a precondition: `cell.temperature = (cell.temperature + config.environment.stress_delta).clamp(0.0, 1.0)`. **Temperature, not toxicity**, despite GDD §6 offering both as examples — `sim::step`'s `env_fit` (§5) reads temperature every tick, while toxicity is currently written (world generation, diffusion) but read by nothing in the tick, so stressing it would be an inert action; this was only discoverable by grepping the tick code, not by playing, which is what motivated the F1 debug overlay (§18). A headless integration test (`tests/action_effects.rs`) verifies the effect is real and dramatic: an organism whose cell gets the full 3-point stress budget applied to its temperature (pushing it far from its thermal optimum) dies outright within one era, where an untouched baseline survives comfortably (~7.6 avg energy) — needed because temperature isn't rendered and the effect is easy to miss via imprecise clicking. Diffusion (§4) gradually smears a stressed cell's temperature back toward its neighbours over subsequent ticks, so the effect is a temporary perturbation, not a permanent terrain edit.

Also introduces the `ActionMode` (`Seed`/`Stress`/`Cull`/`Splice`) and `SelectedAction` resources (`ui.rs`) and the HUD's mode-selector radio buttons — scaffolding tasks 024/025 reuse without touching the selector again.

---

## 16. Cull action (`src/input.rs`, task 024, GDD §6)

**What**: the third player action — click an occupied cell to remove the organism there, costing 1 action point. The simplest of the four: mostly wiring onto task 023's scaffolding.

**How**: `cull_on_click` inverts `Seed`'s empty/occupied precondition (needs an *occupied* cell) and, notably, inverts the check order too: occupancy is checked *before* spending the budget, so clicking an empty cell in `Cull` mode costs nothing and does nothing (`Seed`'s unaffordable-click-skips-everything ordering doesn't apply the same way here, since the "wrong precondition" case must never cost a point). Deliberately deposits **no residue** — GDD §5.6 step 6 ties residue to *death* by the tick algorithm (energy `<= 0`), not to an organism's removal by any means, and a player-culled organism is removed by fiat rather than starving or being predated. One known gap, not fixed by this task: a culled organism doesn't emit `OrganismDied`/`SpeciesExtinct` (§15's events), since `cull_on_click` bypasses `sim::step` entirely — a species can be silently wiped from the world this way with nothing showing up in the Notebook's observation log.

---

## 17. Splice action (`src/ui.rs` + `src/input.rs`, task 025, GDD §6)

**What**: the fourth and most expensive player action (2 points) — GDD §6's "most powerful and most expensive experimental tool": edit a species' genome (swap one tag, or shift its thermal optimum) to create a deliberate variant. Unlike the other three, its target is a *species definition*, not a grid cell, so it drives a small editor panel instead of a click.

**How**: `SpliceDraft` (`ui.rs`) is a UI-intent resource — source species, the in-progress edit (`SpliceEditChoice::SwapTag { old, new }` or `ShiftTempOptimum { warmer }`), and an `apply_requested` flag — staged entirely by `splice_panel`'s egui widgets (shown in the HUD only while `ActionMode::Splice` is selected) with zero direct `SimWorld` writes, honoring `Ui` writes-only-intents (TECH_DESIGN.md §3.3). `input.rs::apply_splice` reads `apply_requested` as the actual trigger — the one action whose "click" is an egui button, not a grid click — and, once budget-affordable and the draft is complete, **clones the source `Species` into a new one** with the edit applied and appends it to `world.species`, rather than mutating the source in place: mutating in place would retroactively change every already-alive organism of that species, but "modify a species' genome" (GDD §6) reads as introducing a variant, with species identity otherwise stable for a run (GDD §5.6 step 7 only floats child-genome mutation as a *future* idea). The new species is automatically seedable — `hud_panel`'s existing `Seed` selector already iterates `0..world.species.len()`, so nothing there needed to change.

**A real bug this surfaced and fixed**: `Splice` is the first mechanism that grows `world.species` past its starting count (previously always exactly 2, from `seed_starting_palette`). A `SelectedSpecies` left pointing at a spliced-in species, followed by `r` (which rebuilds a fresh 2-species world), would index out of bounds the next time anything read `world.species` by that id — `ui.rs::species_stats` gets there first, without even needing a tick to run. `reseed_world` now resets `SelectedSpecies` to `SpeciesId(0)` and clears `SpliceDraft` alongside everything else it already resets, with a regression test reproducing the exact splice → select → reseed → click sequence.

`SpeciesId` is a `u8`, and `world.rs` documents species as "few and never removed" — `Splice` is the first thing that grows the registry at player will, so that assumption now has a 256-species ceiling behind it. Unreachable in practice at 2 points per splice from a 3-point-per-era budget, but worth knowing if that budget or cost ever gets retuned.

---

## 18. Dev-only debug view (`src/render.rs`, Quick Task alongside 023)

**What**: `F1` cycles a full-grid heatmap overlay — Normal → Temperature → Toxicity → Light → Normal — that reads a raw environment scalar straight off `SimWorld` instead of the normal species/residue/light-shaded rendering (§10). Surfaced directly by task 023's discovery that `toxicity` has no in-tick effect and no other visibility: that gap was only findable by grepping the code, not by playing.

**How**: entirely inside a `#[cfg(debug_assertions)] mod debug_view` in `render.rs` — `DebugView` resource, `toggle_debug_view` (the `F1` key), and `apply_debug_view` (runs `.after(sync_grid_colors)`, overwrites the normal-path sprite colors when a non-`Normal` view is active, leaving `cell_color`/`sync_grid_colors` themselves untouched). Compiles out of `--release` builds entirely (verified both profiles build clean) — deliberate, since the game's whole deduction pillar (GDD §7, §11) depends on the player *not* having a direct instrument readout of hidden/backing scalars, so this must never leak past development.

---

## 19. Simulation events (`src/sim.rs`, task 018)

**What**: `sim::step` gained an output channel — `TickEvents { deaths, extinctions, adjacencies }` — without losing its "callable with no Bevy `App`" property (invariant 2). This is what the rest of Phase 2 is built on: the notebook consumes events, it never inspects the grid directly (`TECH_DESIGN.md` §4).

**How**: `step` now returns `TickEvents` instead of `()`. `OrganismDied { cell, species }` is recorded at the existing death site; a pre-tick population count per species (scanned once at the top of `step`) lets a `1 -> 0` transition emit `SpeciesExtinct { species }` alongside the last individual's death. `AdjacencyObserved { receiver_species, exerter_tag, receiver_tag, n_confounders }` is emitted from the same adjacency loop that already sums `interaction_delta` (§8) — one record per occupied-neighbour, non-zero-matrix-entry pair. `n_confounders` is the count of *other* distinct tags among the receiver's occupied neighbours, excluding the exerter tag itself — an organism with exactly one neighbour carrying only the exerter tag yields `n_confounders = 0` (GDD §7's "isolated observation", weight 1.0). `advance_tick` (and `input.rs`'s single-tick `s` key — a second call site to `step` that would otherwise silently drop every event from a manual tick) drain `TickEvents` into Bevy `MessageWriter`s, registered via `App::add_message`.

---

## 20. Observation log / Notebook window (`src/notebook.rs`, task 019)

**What**: the first visible piece of the notebook (GDD §7, §11) — a curated, era-tagged log of salient events, shown in an `egui::Window` toggled with `Tab`.

**How**: `NotebookPlugin` owns `ObservationLog` (`Vec<LogEntry { era, text }>`) and `NotebookWindowOpen(bool)` — a plain UI toggle, not `EraState`, so opening the notebook never blocks or interacts with era advancement. `record_events` reads `MessageReader<SpeciesExtinct>` and appends one entry per extinction; `OrganismDied` is deliberately *not* logged per-event (GDD §7 wants salient signals, not an unfiltered per-tick feed) — a `// TODO: bloom detection` marks the cut second signal type rather than half-building it. The window itself shares `ui.rs`'s existing egui context (no second camera needed — `bevy_egui` supports multiple windows per `EguiPrimaryContextPass` frame) and renders entries in a scrollable `egui::ScrollArea`, oldest first. Reseeding (`r`) clears the log, since `world.era` resets to 0 and stale entries would otherwise show era numbers higher than the fresh run's current one.

---

## 21. Hypothesis confirmation engine (`src/notebook.rs`, task 020)

**What**: the "B with a hint of C" model (GDD §7) that progressively reveals the hidden matrix — not a second opacity mechanic, the *same* matrix from §8, revealed cell by cell as evidence accumulates.

**How**: `MatrixKnowledge` holds cumulative evidence per `(exerter_tag, receiver_tag)` pair, laid out exactly like `TagMatrix` (`exerter * size + receiver`) so the two stay trivially parallel. `accumulate_evidence` drains `MessageReader<AdjacencyObserved>` (§19) each frame, adding `config.notebook.observation_weight_numerator / (1 + n_confounders)` per event — 3 isolated observations (weight 1.0 each) or 12 heavily-confounded ones (weight 0.25 each) both reach the default threshold of `3.0`. A pair is `is_confirmed` once its evidence crosses that threshold; evidence only accumulates, never decays or resets within a run. Confirmation is monotonic by construction — the only place it resets is `input.rs`'s `r` key, which rebuilds `MatrixKnowledge` from scratch alongside the new `SimWorld`, since a new seed means a new (unrelated) hidden matrix. Sized from `config.tags.active_tags_early` at `NotebookPlugin::build` time — the tag *count* is config-fixed, only the matrix's *values* are seed-random, so this dodges any `Startup`-ordering dependency on `WorldPlugin`. `revealed_value` reads through to `world.matrix.get(...)` for a confirmed pair rather than storing its own snapshot of the value — there's only ever one `SimWorld` to read from.

---

## 22. Hypothesis grid UI + tag/species catalog (`src/notebook.rs`, task 021)

**What**: the explicit "aha" of pillar 2 (GDD §7) — a live `active_tags × active_tags` table inside the Notebook window showing which hypotheses are confirmed, plus a catalog of every active tag and species' readable genome.

**How**: `hypothesis_grid` (an `egui::Grid`, row = exerting tag, column = receiving tag, matching `TagMatrix::get`'s own convention) renders `?` for unconfirmed pairs, `+!`/`-!` for confirmed ones (sign read from `MatrixKnowledge::revealed_value`, magnitude never shown — GDD §5.5's "learned empirically"), and `·` on the diagonal (always 0 by construction, not a real hypothesis). There's no "confirmed zero effect" state: task 018 only emits `AdjacencyObserved` for non-zero matrix entries, so evidence never accumulates for a genuinely-zero pair — the grid only distinguishes unconfirmed from confirmed-non-zero. `catalog_panel` lists `world.active_tags` and, per species, its metabolism/temperature range alongside its tags — both tags and the catalog's tag dots use a deterministic golden-angle hue keyed on `TagId` (`tag_color`, same technique `render.rs`'s `SPECIES_HUE_STEP` uses for species), rendered as colored glyphs (`●`), never as raw tag numbers. Player-authored conjectures (GDD §5.9's `±?` state) aren't implemented — cut for this task, left as a follow-up.

---

## 23. Testing strategy

**What exists**: unit tests co-located in every module (`#[cfg(test)] mod tests`) plus three headless integration suites, `tests/determinism.rs`, `tests/balance.rs`, and `tests/action_effects.rs`.

**How it's organized**:
- **Determinism**: same seed ⇒ byte-identical `SimWorld` state (environment, matrix, species tags) at construction, and identical multi-tick trajectories through `sim::step` and `diffuse_environment` — different seeds provably diverge.
- **Balance**: carrying-capacity stall (a crowded photolithic organism's net energy matches the hand-computed GDD figure), bloom-then-stabilize population curves, dark-zone non-habitability, population never reaching exactly zero under the tuned coefficients.
- **Per-mechanic unit tests**: every metabolism's boundary behavior (isolated predator collapses on the GDD's exact tick count, decomposer with no residue behaves like a photolithic organism in the dark, residue never goes negative under contention), the matrix's structural guarantees (diagonal zero, cyclicity, density, asymmetry), diffusion's fixed-point/smoothing/range properties, and the render layer's coordinate math (`cell_position`/`world_to_cell` round-trip).
- **Notebook/events (tasks 018–020)**: `sim.rs` tests cover a death producing exactly one `OrganismDied`, a species' last organism dying also producing `SpeciesExtinct`, an adjacency between tagged organisms producing the expected `AdjacencyObserved`, and the confounder count against GDD §7's own worked numbers. `notebook.rs` tests (in `src/main.rs`'s binary target, since the module depends on `bevy_egui` and can't live in the headless lib crate) cover the extinction-to-log-entry path via a minimal `App` + `MessageWriter`, `MatrixKnowledge`'s pure accumulation logic directly (no `App` needed), and `accumulate_evidence`'s confounder-weight wiring end to end.
- **Actions (tasks 022–025)**: `sim.rs` covers `ActionBudget::try_spend`/`refill` directly, plus an `advance_tick` integration test asserting the budget refills exactly on the era-end transition. `tests/action_effects.rs` verifies `Stress`'s effect headlessly (a stressed organism dies within one era where an untouched baseline survives at ~7.6 avg energy) — added specifically because the effect isn't rendered and is easy to miss via imprecise manual clicking. `input.rs` tests cover `apply_splice`'s core logic with a minimal `App` (tag-swap creates a new species and leaves the source untouched, temperature-shift clamps to `[0,1]`, an incomplete or unaffordable draft applies nothing) and the reseed-after-splice regression: a `SelectedSpecies` pointing past a fresh world's species count must be reset, not left dangling (it previously wasn't, and would panic the next frame anything indexed `world.species` by it).
- All of this runs **without a Bevy `App`** where possible (`sim`/`world` tests construct `SimWorld` + `SimConfig` directly), which is only possible because of the ECS-free architecture in §3 — this is the payoff of that decision, not incidental. Where a test genuinely needs Bevy machinery (state resources, message queues), it builds the smallest `App` that exercises the system under test rather than the full `main.rs` wiring.

---

*Snapshot after task 025 (2026-08-03). Phase 2 is complete: Track A (018–021: events, notebook, confirmation engine, hypothesis grid) and Track B (022–025: action budget economy, Stress, Cull, Splice) are both done. Phase 3 (procedural worldgen, real objectives, main menu) and Final Tuning are backlog, not yet started.*
