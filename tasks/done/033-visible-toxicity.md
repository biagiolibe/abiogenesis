# Task 033 — Render the toxic zone visibly during normal play

> **ID**: `033`
> **Category**: Bugfix / UX
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-03 playtest

---

## 🎯 Objective

`Cell::toxicity` is seeded at world generation (a fixed toxic zone, `world.rs`'s `apply_gradients`), diffuses every tick, and — since task 023 — can be shifted directly by the `Stress` action. But `render.rs::cell_color` never reads it: the toxic zone is **completely invisible** during normal play. The only way to see it at all is the `F1` debug overlay (a dev-only build), which was never meant to be the player's way of perceiving a real, functional hazard in the world.

This is a readability bug more than a feature request: a real gameplay-relevant scalar (organisms in a toxic cell should presumably fare worse — check whether that's actually wired into `sim::step` yet, since task 023's investigation found `toxicity` currently has **no** in-tick effect at all, separate from this task's scope) is silently absent from the player's view of the board.

---

## 📋 Acceptance Criteria

- [x] `cell_color` (or an addition alongside it) factors `cell.toxicity` into what's rendered — e.g. a tint/overlay whose intensity scales with `toxicity`, composited with the existing organism/residue/light rendering rather than replacing it (a toxic cell holding an organism should still show the organism's species color, just visibly "tainted").
- [x] The toxic zone (and any cell whose toxicity changes via `Stress`) is visually identifiable without the `F1` debug overlay.
- [x] Existing render tests (`render::tests::occupied_cells_are_saturated_and_residue_desaturated`) still pass, or are updated if the new toxicity tint changes their saturation/lightness assumptions — don't let this silently break coverage that currently locks down the empty/residue/occupied color relationships.
- [x] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `cell_color` — the only place this needs to change |

---

## 🧩 Technical Context

`cell_color`'s existing precedence is organism > residue > empty (light-shaded). Toxicity needs to compose with *all three*, not slot in as a fourth mutually-exclusive branch — a toxic cell can simultaneously hold an organism. The simplest approach: compute the base color exactly as today, then blend/darken/redden it by a `toxicity`-scaled amount as a final step, rather than restructuring the branch logic.

Related, out of scope for this task: task 023 found `toxicity` has no gameplay effect in `sim::step` at all (only `temperature` affects `env_fit`). This task makes the scalar *visible*; whether it should also start *doing* something to organisms is a separate, larger balance/design decision — don't couple the two in this task's scope, but the code comment near `cell_color`'s toxicity handling should probably note that the visual and the (currently absent) mechanical effect aren't yet the same thing, so a future reader doesn't assume otherwise.

---

## 🔨 Suggested Implementation

1. In `cell_color`, after computing the existing organism/residue/empty color, blend in a toxicity tint (e.g. shift hue toward a warning color like magenta/red, or reduce lightness) scaled by `cell.toxicity`, clamped so it stays a visible-but-not-overwhelming effect even at `toxicity = 1.0`.
2. Check `occupied_cells_are_saturated_and_residue_desaturated` and any other `render::tests` that assert exact color properties — update assertions if the toxicity blend changes them (most existing tests build cells with `toxicity: 0.0` implicitly via `Cell::default()`/`..world.cells[idx]`, so they likely need no change, but verify).
3. Add a new test: two cells identical except for `toxicity`, confirm their rendered colors differ.
4. Manual verification via the `run` skill (not just `F1` — the whole point is this should be visible without it): confirm the toxic corner is visually distinct in the normal view, confirm a `Stress`-ed cell's toxicity change (if `Stress` is ever extended to toxicity — it currently targets temperature only, task 023) or a reseed's toxic zone is visible.

---

## ⚠️ Constraints and Caveats

- Keep rendering strictly read-only against `SimWorld` (TECH_DESIGN.md §3.1/§3.3) — presentation-only.
- Don't wire toxicity into `sim::step`'s energy/death arithmetic as part of this task — that's a separate balance decision with its own regression risk (re-tuning coefficients), out of scope here.
- Don't remove or gut the `F1` debug overlay (`render.rs`'s `debug_view` module) — it stays useful for isolating one scalar at a time during future development, even once toxicity has a normal-play visual.

---

## 🔗 Dependencies

- **Depends on**: 004 (environment gradients, the toxic zone's origin), 010 (rendering)
- **Blocks**: none (independent of 032)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/033-visible-toxicity.md)"$'\n\nExecute this task in the current project.'
```
