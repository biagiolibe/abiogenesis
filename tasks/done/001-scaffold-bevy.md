# Task 001 — Toolchain, Cargo scaffold, and plugin-based Bevy app

> **ID**: `001`
> **Category**: Architecture
> **Priority**: 🔴 P1
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Take the project from "folder with only documents" to a **Bevy application that starts and opens a window**, with the plugin structure already in place (empty stubs) for all subsequent tasks to build on.

Necessary because the project **isn't scaffolded**: neither `Cargo.toml` nor `src/` exist at the root. Every other task depends on it.

---

## 📋 Acceptance Criteria

- [x] `rustc --version` reports **1.97.1** and `rust-toolchain.toml` pins that version.
- [x] `cargo build` compiles without errors.
- [x] `cargo run` opens a window titled `Abiogenesis` with a uniform background (no content: that's correct).
- [x] `cargo clippy -- -D warnings` is clean.
- [x] The six plugins exist as stubs and are registered in `main.rs`.
- [x] The exact versions resolved by `cargo add` are recorded in `TECH_DESIGN.md` §1, replacing the "To be completed in task 001" note.
- [x] `.gitignore` covers `/target` (already present: verify).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `rust-toolchain.toml` | Pins the toolchain (new) |
| `Cargo.toml` | Dependencies and build profiles (new) |
| `src/main.rs` | Entry point, plugin registration (new) |
| `src/config.rs`, `src/world.rs`, `src/sim.rs`, `src/render.rs`, `src/ui.rs`, `src/input.rs` | Plugin stubs (new) |
| `TECH_DESIGN.md` | §1 to be updated with resolved versions |

---

## 🧩 Technical Context

- **Current behavior**: the root contains only Markdown documents and a git repo initialized with no commits. No Rust code.
- **Desired behavior**: `cargo run` opens a Bevy window.

**Verified version constraint:**

| Component | Version | Note |
|---|---|---|
| Current local toolchain | 1.90.0 | **insufficient** |
| Bevy 0.19 | requires Rust **≥ 1.95.0** | |
| Available stable | **1.97.1** | needs installing |
| `bevy_egui` | **0.41** | pairing with bevy 0.19 is verified (egui 0.35) |

*Fallback if the toolchain upgrade isn't practical:* bevy `0.18.1` + bevy_egui `0.39.x` run on Rust 1.90. In that case update `TECH_DESIGN.md` §1 and this task file accordingly.

---

## 🔨 Suggested Implementation

1. **Toolchain**

   ```bash
   rustup update stable
   rustc --version   # expected: 1.97.1
   ```

   Create `rust-toolchain.toml`:

   ```toml
   [toolchain]
   channel = "1.97.1"
   components = ["rustfmt", "clippy"]
   ```

2. **Scaffold** — **`init`, not `new`**: the directory already has files.

   ```bash
   cargo init --name abiogenesis
   cargo add bevy@0.19
   cargo add bevy_egui@0.41
   cargo add rand
   ```

3. **Build profile.** Without this, the debug-mode simulation is unwatchably slow: dependencies need to be compiled optimized even in dev. In `Cargo.toml`:

   ```toml
   [profile.dev]
   opt-level = 1

   [profile.dev.package."*"]
   opt-level = 3
   ```

4. **Plugin stubs.** One file per module, each with an empty `Plugin`:

   ```rust
   use bevy::prelude::*;

   pub struct ConfigPlugin;

   impl Plugin for ConfigPlugin {
       fn build(&self, _app: &mut App) {}
   }
   ```

5. **`main.rs`** — `DefaultPlugins` with the window configured, plus the project's six plugins:

   ```rust
   fn main() {
       App::new()
           .add_plugins(DefaultPlugins.set(WindowPlugin {
               primary_window: Some(Window {
                   title: "Abiogenesis".into(),
                   ..default()
               }),
               ..default()
           }))
           .add_plugins((
               ConfigPlugin,
               WorldPlugin,
               SimPlugin,
               GridRenderPlugin,
               UiPlugin,
               InputPlugin,
           ))
           .run();
   }
   ```

6. Record the resolved versions (`cargo tree --depth 0` or `Cargo.lock`) in `TECH_DESIGN.md` §1.

---

## ⚠️ Constraints and Caveats

- **`cargo init`, not `cargo new`** — the folder isn't empty.
- **`EguiPlugin` isn't registered yet**: that's task 008. Here `bevy_egui` is just a declared dependency.
- **No game logic in this task.** Stubs stay empty; filling them in is the job of tasks 002+.
- The `rand` ≥ 0.9 API differs from 0.8 (`thread_rng` → `rng`, `gen` → `random`): verify the resolved version before writing code that uses it (task 003).
- Bevy's first build downloads and compiles a lot: expect several minutes.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: 002 (and cascades to everything else)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/001-scaffold-bevy.md)"$'\n\nExecute this task in the current project.'
```
