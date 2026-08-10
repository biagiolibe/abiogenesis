# Task 086 — Environment overlay legibility check

> **ID**: `086`
> **Category**: UI / Playtest verification
> **Priority**: 🟢 P3
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-10, scoped from `redesign/abiogenesis-environment-sources.md`

---

## 🎯 Objective

Under the old fixed-axis model a player learns "left is hot, top is bright" once and it transfers to every world. Under task 085's per-world random heat sources and sun direction, they must re-derive the local layout for each world, which makes the existing T/L overlay toggles (`apply_environment_overlay`, `render.rs:130-146`) load-bearing rather than an optional nicety. Verify in-game whether the existing overlays read clearly under the new source-driven fields: source hot-spots and the sun-direction gradient both need to be legible through the heatmap toggle.

This is scoped as a verification pass, not a pre-committed rendering change: if the overlays already suffice, close with no/minimal changes; if not, scope the actual visual fix as acceptance criteria discovered during the task itself (e.g. marking source cell locations, a wind/sun direction indicator).

---

## 📋 Acceptance Criteria

- [ ] `cargo run`, generate several worlds (varied seeds), toggle the temperature and light overlays, and judge legibility: can you tell where the heat sources are and which direction is "sunward" just from the overlay?
- [ ] If overlays are legible as-is: document that finding here (or in the PR/commit) and close with no rendering changes.
- [ ] If overlays are not legible: scope and implement a concrete fix (e.g. a marker at each source cell, a directional gizmo/arrow for wind or sun), touching `apply_environment_overlay`/`cell_color` (`src/render.rs`) as needed, and add the new acceptance criteria for that fix before closing the task.
- [ ] `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test` clean if any code changed.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `apply_environment_overlay` (`~130-146`), `cell_color` — overlay rendering to inspect/extend. |

---

## 🧩 Technical Context

- **Current behavior**: `apply_environment_overlay` renders temperature/light as a heatmap tint over the grid; designed against the old fixed-gradient model where "hot" and "bright" always point the same screen direction across every world.
- **Desired behavior**: overlays remain legible when temperature/light layouts are randomized per world (source positions, wind direction, sun direction) rather than fixed to screen axes.

---

## 🔨 Suggested Implementation

1. Run the game with task 085's source-driven model live, across a handful of seeds.
2. Toggle each overlay and assess: can hot/cold and bright/dark regions be read at a glance, or does the player need extra cues?
3. If insufficient, add the minimal visual cue that fixes it — prefer reusing existing overlay/painter patterns (e.g. how `draw_energy_overlay` or the toxic-zone dashed outline are drawn) over inventing a new rendering mechanism.

---

## ⚠️ Constraints and Caveats

- **Style**: Follow `TECH_DESIGN.md` — rendering code may depend on `bevy::render`, but must not leak visual-only concerns back into `sim`/`world`/`config`.
- Keep the fix minimal — this task verifies legibility, it does not redesign the overlay system.

---

## 🔗 Dependencies

- **Depends on**: 085
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/086-environment-overlay-legibility-check.md)"$'\n\nExecute this task in the current project.'
```
