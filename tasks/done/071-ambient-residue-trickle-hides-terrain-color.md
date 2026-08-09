# Task 071 — Ambient residue trickle hid terrain colors grid-wide

> **ID**: `071`
> **Category**: Bugfix / UI
> **Priority**: 🔴 P1
> **Estimate**: ~30min
> **Assigned to**: Claude
> **Session**: 2026-08-09, raised directly by the user after task 070 shipped

---

## 🎯 Objective

Task 070 removed task 062's decorative background layer, fixing the
black-square halo around organism sprites — the user confirmed that part.
But a second, unrelated regression remained: after the *first* era advance
(`Space`), the entire grid turned a uniform brownish wash and every terrain
band color (task 068) disappeared.

Root cause: task 060's ambient residue trickle (`sim::step`, `src/sim.rs:
141-144`) adds a small amount of residue to **every** cell, every tick, so
an isolated Decomposer still has something to read. Residue decays faster
than the trickle replaces it, so every cell — including ones nothing ever
died on, including `Sea` — settles at exactly `residue_ambient_trickle`
(0.05) after a single tick. `cell_color` (`src/render.rs`) treated any
`residue > 0.0` as "a corpse decayed here" and painted it brown
(`Color::hsl(30.0, 0.2, ...)`), which took priority over the terrain
branch. Once the ambient floor established grid-wide (immediately after the
first `Space` press), literally every empty cell rendered brown instead of
its terrain color. This was invisible before terrain had distinct colors
(068) — the old empty-cell shading and the residue brown were both
near-black and indistinguishable.

---

## 📋 Acceptance Criteria

- [x] `cell_color`'s residue branch condition changed from `cell.residue > 0.0` to `cell.residue > config.energy.residue_ambient_trickle`, so the ambient floor no longer counts as "residue worth showing."
- [x] `cargo clippy -- -D warnings` clean.
- [x] New unit test (`ambient_residue_trickle_does_not_hide_the_terrain_color`) pins a cell's residue at exactly the ambient floor and asserts the terrain color still renders.
- [x] Existing `occupied_cells_are_saturated_and_residue_desaturated` test still passes unmodified (residue set to `residue_on_death`, well above the floor).
- [x] Full `cargo test` clean.
- [x] Verified visually via `cargo run`: pixel-sampled several terrain-band cells before and after two era advances — colors identical, no brown wash.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `cell_color`'s residue-branch threshold; new regression test. |
| `src/sim.rs` | `step`'s ambient trickle (`residue_ambient_trickle`) — read, not modified. |

---

## 🔗 Dependencies

- **Depends on**: 060 (ambient trickle), 068 (terrain colors that exposed this), 070 (same user-reported thread, different root cause).
- **Blocks**: none.
