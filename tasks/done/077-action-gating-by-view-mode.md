# Task 077 — Gate Stress/Cull to Detail mode; Overview placement indicator for Seed/Splice

> **ID**: `077`
> **Category**: Feature / Input
> **Priority**: 🟡 P2
> **Estimate**: ~1-1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-09

---

## 🎯 Objective

Closes out the two-tier view work (`redesign/abiogenesis-two-tier-view.md`,
tasks 075/076): `Stress` and `Cull` need per-organism precision that doesn't
survive Overview's cluster-heatmap aggregation, so they should only be usable
in `Detail` mode. `Seed` and `Splice` are creation actions, not
precision-targeting ones, and stay available in both modes — but since
placement always resolves to one exact cell even from Overview, a brief
transient on-screen indicator marks exactly where it landed (same spirit as
task 054's first-confirmed-hypothesis celebration), so the player isn't left
guessing which cell of a cluster their action actually hit.

---

## 📋 Acceptance Criteria

- [x] **Decision**: disabled button, not inert click. (`ui.rs`'s
      `action_icon_row` wraps Stress/Cull in `add_enabled_ui`, mirroring
      `splice_panel`'s tag-cap gating; verified live via `cargo run` — icons
      visibly dim in Overview and brighten in Detail.) `stress_on_click`
      (`src/input.rs:302`) and `cull_on_click` (`src/input.rs:347`) also gain
      a `MapViewMode` guard as defense-in-depth (stale `SelectedAction` from
      before a zoom-out shouldn't act), but the HUD button itself is greyed
      in Overview so the player never gets to click-and-nothing-happens —
      same legibility bar as the existing budget-gated buttons in `ui.rs`'s
      Moves panel.
- [x] `attempt_seed`/`seed_organism_on_click` (`src/input.rs:203-233`) and
      `apply_splice` (`src/input.rs:394`) remain functional in both
      `Overview` and `Detail`.
- [x] **Correction**: the placement indicator applies to `Seed` only, not
      `Splice`. Per the design doc (`redesign/abiogenesis-two-tier-view.md`,
      "Placement always resolves to one exact cell") the indicator is about
      *placement* — `apply_splice` never calls `clicked_cell` and has no
      target cell at all (it appends a new entry to `world.species`, driven
      by an egui dropdown, not a grid click), so there is no cell to point
      an indicator at. A Seed resolved while in `Overview` mode triggers a
      brief, non-blocking visual indicator at the exact cell that was
      affected (flash, ring, or similar). Task 054 turned out not to
      establish a reusable spatial-indicator mechanism (it ended up as an
      observation-log line + HUD badge, nothing drawn on the grid) — the
      closer existing precedent is `render.rs`'s `terrain_overlay` module
      (egui `Painter` + `camera.world_to_viewport` projection, same
      technique `notebook.rs`'s `draw_dashed_ring` uses for a ring), reused
      here for a new transient ring drawn for a short real-time duration.
- [x] The indicator is purely cosmetic/transient — it doesn't block input,
      doesn't persist once task 076's cluster blob naturally makes the
      population visible, and has no effect on `sim`/`world` state. (Real-time
      `Res<Time>` countdown in `render.rs`'s `placement_indicator` module,
      read-only against `SimWorld`.)
- [x] HUD (`Moves` panel, `src/ui.rs`) reflects the gating — e.g. Stress/Cull
      buttons show as unavailable in Overview, consistent with how other
      budget/precondition-gated actions are already presented.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [x] Verified live via `cargo run`: confirmed Stress/Cull icons visibly dim
      in Overview and brighten in Detail (screenshot diff of the Moves icon
      row across a zoom-in); confirmed Seed placement in Overview draws a
      yellow ring exactly on the newly seeded cell.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `attempt_seed`/`seed_organism_on_click`, `stress_on_click`, `cull_on_click`, `apply_splice` — the four action handlers this task gates or leaves untouched. |
| `src/ui.rs` | HUD `Moves` panel — action button availability/greying, needs to reflect the new gating. |
| `redesign/abiogenesis-two-tier-view.md` | Design record for the gating rule and the placement-indicator rationale. |
| Task 054's file (`tasks/done/054-celebrate-first-confirmed-hypothesis.md`) | Reference for the existing transient-indicator pattern to reuse rather than reinvent. |

---

## 🧩 Technical Context

- **Current behavior**: all four actions (`Seed`, `Stress`, `Cull`,
  `Splice`) are available identically regardless of camera state — there is
  no view-mode concept yet (introduced by task 075).
- **Desired behavior**: `Stress`/`Cull` require `MapViewMode::Detail`;
  `Seed`/`Splice` work in both, with Overview-mode placements getting a
  transient visual confirmation at the exact affected cell.

---

## 🔨 Suggested Implementation

1. Confirm tasks 075 (`MapViewMode` resource) and ideally 076 (cluster
   rendering, so the indicator's handoff to the cluster blob is visible)
   have landed.
2. Add the `MapViewMode` precondition check to `stress_on_click` and
   `cull_on_click`, mirroring how these functions already check
   `selected_action.0` and the action budget before proceeding.
3. Update `ui.rs`'s `Moves` panel to grey out/disable Stress/Cull when not
   in Detail mode.
4. Add the transient placement indicator, triggered from
   `seed_organism_on_click`/`apply_splice` specifically when the action
   resolves while `MapViewMode::Overview` is active — reuse task 054's
   existing celebration/indicator mechanism if its structure allows
   parameterizing the target cell and message.

---

## ⚠️ Constraints and Caveats

- **Don't touch `sim`/`world`/`config`** — this is an `input.rs`/`ui.rs`
  concern layered on top of the existing action handlers, same boundary
  every prior action task (022-025) already respected.
- Keep the indicator cosmetic-only; it must not gate or delay the action's
  actual effect.

---

## 🔗 Dependencies

- **Depends on**: 075 (`MapViewMode` resource). Benefits from 076 landing
  first (so the indicator's handoff to a visible cluster blob can be
  verified end-to-end) but doesn't strictly require it — Overview would just
  keep rendering individual dots until 076 lands, whichever order they're
  done in.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/077-action-gating-by-view-mode.md)"$'\n\nExecute this task in the current project.'
```
