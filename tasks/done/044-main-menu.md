# Task 044 — Main menu

> **ID**: `044`
> **Category**: UI / Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Phase 3 planning session

---

## 🎯 Objective

`GameState::MainMenu` already exists (task 007) but is unreachable: `main.rs` does `Loading → Playing` directly, with an explicit comment *"Phase 0: `Loading` transitions straight to `Playing`; `MainMenu` becomes real in Phase 3."* `TECH_DESIGN.md` §2 already describes `MainMenu` as "seed selection and run start — (Stub in Phase 0, real in Phase 3.)"

This task makes the main menu real: new `src/menu.rs` module, `main.rs` updated to `Loading → MainMenu`, and the menu UI is where `RunProgress.run_seed` is born — the **only** legitimate point outside the simulation where variety can be introduced (it's not tick logic, it doesn't need to honor the "no system clock in the sim" invariant because it isn't part of the sim; from that point on, everything derives from the run's RNG, never a second clock read, the same pattern already used by `next_seed()` for the `r` key).

---

## 📋 Acceptance Criteria

- [ ] New `src/menu.rs` module with the main menu UI (egui, consistent with the existing stack).
- [ ] `main.rs`: `OnEnter(GameState::Loading)` transitions to `GameState::MainMenu` instead of directly to `Playing`.
- [ ] The main menu offers at least a "new run" action (with the option to specify an explicit seed, or generate one — consistent with GDD §14 "Real main menu with seed selection and sharing" cited as an approved idea in `PROJECT_PLAN.md`).
- [ ] "New run": generates/accepts `run_seed`, initializes `RunProgress` (task 035) with `world_index=0`, derives `world_seed` from the run's RNG (not directly from the raw `run_seed`, if that introduces ambiguity — decide a clear scheme and document it in the code), transitions to `GameState::Playing`.
- [ ] New section in `src/text.rs` for the main menu's strings (title, buttons, possible seed field) — consistent with task 034.
- [ ] **Non-regression criterion**: a "new run" with a fixed seed reproduces the same observable game behavior the game has today with `spawn_world(42, ...)` (when that seed is used).
- [ ] `cargo clippy -- -D warnings` clean, `cargo test` green.
- [ ] Manual verification: `cargo run` boots the game into the main menu, not into a world; pressing "new run" leads to a playable world.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/menu.rs` (new) | Main menu UI, "new run" input handling. |
| `src/main.rs` | Change of the `OnEnter(GameState::Loading)` transition from `Playing` to `MainMenu`; registration of the new module/plugin. |
| `src/text.rs` | New main menu string section. |
| `src/world.rs` | `spawn_world`/`WorldPlugin::build` (lines ~249-255) — currently called from `Startup`, needs to be moved/made conditional on "new run started from the menu" instead of always at app startup. |

---

## 🧩 Technical Context

**Current `main.rs`**:
```rust
App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin { title: "Abiogenesis".into(), .. }))
    .add_plugins(EguiPlugin::default())
    .add_plugins((
        ConfigPlugin, WorldPlugin, SimPlugin, GridRenderPlugin,
        UiPlugin, NotebookPlugin, InputPlugin,
    ))
    .init_state::<GameState>()
    .add_sub_state::<EraState>()
    .add_systems(OnEnter(GameState::Loading), enter_playing)
    .run();
```
`enter_playing` is the only state-transition system in `main.rs` — this task replaces it/adds a transition toward `MainMenu`.

**`WorldPlugin::build`** (`src/world.rs`, lines ~249-255):
```rust
fn spawn_world(mut commands: Commands, config: Res<SimConfig>) {
    let mut world = SimWorld::new(42, &config);
    seed_starting_palette(&mut world, &config);
    commands.insert_resource(world);
}
```
Today it runs in `Startup`, unconditionally. With a real main menu, world creation must happen when the player presses "new run", not at process startup — this task moves this logic behind the menu's action (probably reusing/extending the same function task 045 will use for `start_world`, but here limited to creating the *first* world of a run).

**`reseed_world`** (`src/input.rs`, lines ~107-147, `r` key): reference pattern for "how to generate a new seed without breaking determinism" — uses `world.next_seed()`, never the system clock.

- **Current behavior**: the game boots directly into a world with fixed seed `42`, no startup UI.
- **Desired behavior**: the game boots into a main menu; the player explicitly starts a run, which from then on deterministically generates its own sequence of worlds from the seed chosen/generated at that moment.

---

## 🔨 Suggested Implementation

1. Create `src/menu.rs` with a `Plugin` that adds the egui UI for `OnEnter(GameState::MainMenu)`/during `GameState::MainMenu`, and a system that handles the "new run" input.
2. In `main.rs`, change `OnEnter(GameState::Loading)`'s destination to `GameState::MainMenu`.
3. Move `spawn_world`'s logic from unconditional `Startup` to a system triggered by `OnEnter(GameState::Playing)` (or by the menu's explicit action before the transition), which initializes both `SimWorld` and `RunProgress`.
4. Decide the seed scheme: simplest option — the `run_seed` chosen/generated at the menu directly becomes the first `world_seed` (`world_index=0`); subsequent worlds (task 045) derive their seeds from the run's internal RNG, not from new user input.
5. Add the strings to `text.rs`, implement the UI.
6. Manual verification.

---

## ⚠️ Constraints and Caveats

- **Determinism**: `run_seed` is born at the menu (presentation layer, outside the sim) — from that point on, no second point of the code must read the system clock or generate a "fresh" seed independently; everything derives from there.
- **`sim`/`world`/`config` stay headless**: `menu.rs` may depend on `bevy_egui`, but must not introduce dependencies from `world.rs`/`sim.rs` toward `menu.rs`/rendering.
- **Don't implement the world-cleared/defeat screens yet**: those are task 045 — this task only covers entering the run (main menu → first world).
- **Don't implement meta-progression yet**: no unlocks to show here (task 046).

---

## 🔗 Dependencies

- **Depends on**: 035 (`RunProgress`, `GameState`).
- **Blocks**: 045 (world transition reuses the world-creation scheme introduced here).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/044-main-menu.md)"$'\n\nExecute this task in the current project.'
```
