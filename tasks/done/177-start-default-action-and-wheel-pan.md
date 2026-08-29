# Task 177 — No armed action at world start; pan with middle-mouse/wheel-drag

> **ID**: `177`
> **Category**: UX / Feature (small bundle)
> **Priority**: 🟢 P3
> **Estimate**: ~45min
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

Two small, unrelated-but-cheap-to-bundle UX items from the playtest:

1. **Seed action armed by default at world start** (`playtest_outcome.md`
   issue I.2): a fresh world starts with the Seed action pre-selected, so a
   player's first instinct to inspect the environment instead accidentally
   places a species. Start a new world with no action armed (or with
   Inspect as the non-destructive default) instead of Seed.
2. **Pan with wheel held down** (`playtest_outcome.md`, feature F.1): add
   the ability to pan the map by holding the mouse wheel/middle button and
   dragging, alongside whatever pan input already exists.

Design source: `playtest_outcome.md` issue I.2 and feature request F.1.

Not yet investigated in code — locate the armed-action state (likely a
resource in `src/input.rs`, set on `WorldResetParams`/`start_world` in
`src/run_flow.rs`) and the existing pan-input handling before implementing;
this task file only specifies the two behaviors sourced from the playtest,
not the underlying mechanism.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] New/reset world starts with no destructive action armed (Seed no
      longer pre-selected); clicking a cell before choosing an action must
      not place anything.
- [ ] Map can be panned by holding the mouse wheel (middle button) and
      dragging, in addition to any existing pan control (e.g. edge-scroll
      or a keybind) — check for conflicts with existing middle-click/wheel
      bindings before wiring it up.
- [ ] Manual check: start a fresh world, confirm inspecting a cell before
      picking an action doesn't seed anything; confirm wheel-drag pans the
      map smoothly.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | Armed-action state, existing pan handling — locate during implementation. |
| `src/run_flow.rs` | `WorldResetParams`/`start_world` — where the default armed action is likely set on (re)start. |

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/177-start-default-action-and-wheel-pan.md)"$'\n\nExecute this task in the current project.'
```
