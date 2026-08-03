# Task 019 — Observation log (notebook window)

> **ID**: `019`
> **Category**: Feature / UI
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Give the player the first piece of the notebook (GDD §7, §11): a log of salient events — blooms, deaths of note, extinctions — recorded era by era, shown in a dedicated egui window toggled with `tab`. This task is read-only: it consumes the events task 018 now emits and renders them. No confirmation math, no hypothesis grid yet (that's 020/021).

---

## 📋 Acceptance Criteria

- [ ] New module `src/notebook.rs`, one `Plugin` (`NotebookPlugin`) per the project's "one module = one Plugin" convention.
- [ ] A `ObservationLog` resource holds an ordered list of log entries, each tagged with the era it happened in (`world.era` at the time) and a human-readable line (e.g. `"Era 3: species 2 went extinct"`).
- [ ] Systems consume `OrganismDied`/`SpeciesExtinct` (`MessageReader`, task 018) and append entries. `SpeciesExtinct` always logs; `OrganismDied` should **not** log every single death (that would flood the log every tick) — only aggregate/salient signals: at minimum, species extinctions. A "bloom" heuristic (population of a species crosses some multiple of its starting count within an era) is a reasonable second entry type if time allows; if cut, leave a `// TODO: bloom detection` note rather than a half-implementation.
- [ ] `tab` opens/closes the notebook window (new system in `notebook.rs` or `input.rs`, following existing key-handling patterns in `input.rs`). The window itself is a `bevy_egui::egui::Window`, not a second full-viewport panel like the HUD — check whether it can share `ui.rs`'s existing `EguiContexts`/`EguiPrimaryContextPass` or needs a small extension; document whichever it turns out to be.
- [ ] Log entries render in the window, newest last (chronological), scrollable once long (`egui::ScrollArea`).
- [ ] The window's open/closed state is a plain `bool` in a resource, not `EraState` — opening the notebook must not block or interact with era advancement.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | New: `NotebookPlugin`, `ObservationLog` resource, log-window system |
| `src/main.rs` / `src/lib.rs` | Register `NotebookPlugin` alongside the other plugins |
| `src/input.rs` | Precedent for key-driven toggles (`start_era`, `reseed_world`) — `tab` follows the same `keys.just_pressed(...)` pattern |
| `src/ui.rs` | Precedent for egui usage in this codebase (`hud_panel`, `EguiContexts`) — read before deciding how the notebook window attaches to the egui context |
| `src/sim.rs` | Source of `OrganismDied`/`SpeciesExtinct` (task 018) |

---

## 🧩 Technical Context

`TECH_DESIGN.md` names `notebook` explicitly as a module planned for Phase 2 — this task is what creates it. `ui.rs`'s `UiPlugin` currently owns the *only* egui context in the app (a dedicated full-viewport camera, see its "HUD camera" doc comment) because `bevy_egui` auto-attaches to the first spawned camera otherwise. A second `egui::Window` drawn from a *different* system in the *same* `EguiPrimaryContextPass` should coexist fine with the HUD panel (egui supports multiple windows/panels per frame from the same context) — this task should not need a second camera. Verify this assumption early; if it turns out `notebook.rs` needs to read `EguiContexts` itself, it'll be consuming the same context `ui.rs` already sets up, not creating a new one.

---

## 🔨 Suggested Implementation

1. `src/notebook.rs`:
   ```rust
   #[derive(Resource, Default)]
   pub struct ObservationLog {
       pub entries: Vec<LogEntry>,
   }

   pub struct LogEntry { pub era: u32, pub text: String }

   #[derive(Resource, Default)]
   pub struct NotebookWindowOpen(pub bool);

   pub struct NotebookPlugin;

   impl Plugin for NotebookPlugin {
       fn build(&self, app: &mut App) {
           app.init_resource::<ObservationLog>()
               .init_resource::<NotebookWindowOpen>()
               .add_systems(Update, (toggle_notebook, record_events))
               .add_systems(EguiPrimaryContextPass, notebook_window);
       }
   }
   ```
2. `record_events` reads `MessageReader<OrganismDied>`/`MessageReader<SpeciesExtinct>` plus `Res<SimWorld>` (for `world.era`), pushes `LogEntry`s into `ObservationLog`.
3. `toggle_notebook` mirrors `input.rs`'s key-handling systems: `if keys.just_pressed(KeyCode::Tab) { open.0 = !open.0; }`.
4. `notebook_window` draws `egui::Window::new("Notebook").open(&mut open.0).show(ctx, |ui| { egui::ScrollArea::vertical().show(ui, |ui| { for entry in &log.entries { ui.label(...) } }) })`, gated on `open.0`.
5. Register `NotebookPlugin` in `main.rs`/`lib.rs` next to `UiPlugin`.
6. Manual verification via the `run` skill: place organisms with conflicting tags near each other (from the existing seed action), advance a few eras, kill off a species (e.g. via the toxic zone), press `tab`, confirm the extinction shows up in the log with the right era number.

---

## ⚠️ Constraints and Caveats

- Read-only with respect to `SimWorld` — the notebook observes, it doesn't mutate simulation state (`TECH_DESIGN.md` §3.3).
- No hypothesis grid, no matrix confirmation UI in this task — that's 020 (logic) and 021 (UI). This task is the log only.
- Don't log every `OrganismDied` unfiltered — GDD §7 explicitly calls out *salient* events, and an unfiltered feed defeats the point of a curated log.

---

## 🔗 Dependencies

- **Depends on**: 018
- **Blocks**: none directly (021 depends on 020, not on this task, though it likely reuses the same window)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/019-observation-log-notebook-window.md)"$'\n\nExecute this task in the current project.'
```
