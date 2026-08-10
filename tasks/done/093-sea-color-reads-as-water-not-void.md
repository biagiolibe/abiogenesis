# Task 093 — Sea should read as water, not "end of the world"

> **ID**: `093`
> **Category**: Rendering / Presentation
> **Priority**: 🟢 P3
> **Estimate**: ~30min
> **Assigned to**: unassigned
> **Session**: 2026-08-10, user-reported live: `Sea`'s near-black color
> reads as an edge/void rather than water, even though (since task 085)
> `Sea` is real, meaningful terrain — a passive coastal coolant, not an
> absence of data.

---

## 🎯 Objective

Reverse a deliberate task-068 design decision that's now stale.
`terrain_color`'s own doc comment (`render.rs:1361-1367`) states the
original intent plainly: "`Sea` stays near-black, close to the grid's own
background, so it still reads as 'void' the way the pre-terrain empty cell
did." That made sense in task 068, when Sea was inert (just a placement
exclusion, no gameplay data of its own). It stopped making sense once task
085 gave Sea a real mechanical role (`SimWorld::reinject_environment_
sources`' coastal cooling) and task 086 made the T/L overlay render its
real scalar there too — reading it as "void"/"edge of the map" actively
undersells that it's real terrain a player should reason about, the same
gap 086 already closed for the overlay layer.

---

## 📋 Acceptance Criteria

- [x] `terrain_color`'s `TerrainKind::Sea` branch changes from
      `Color::hsl(0.0, 0.0, 0.02)` (near-black) to a color that reads as
      water — a dark, desaturated blue is the obvious first pick, but tune
      visually via `cargo run` rather than picking a value on paper; must
      stay visually distinct from `Plain`/`Hill`/`Mountain`'s existing
      palette (task 068's "one flat color per band" rule) and from the
      toxic zone's tint (`toxicity_tint`).

      Set to `Color::hsl(210.0, 0.35, 0.10)` — a dark desaturated navy,
      staying in the same low-lightness family as the other three bands
      (0.09-0.19) while its hue (blue, vs. the others' green/brown) makes it
      unambiguously distinct. First-pass value per the task's own guidance;
      exact shade is still open to further visual tuning by the user.
- [x] `terrain_color`'s doc comment is rewritten to drop the "reads as void"
      framing and state the new rationale.
- [x] `sea_stays_near_black` (`render.rs:1510-1516`) is rewritten for the
      new intent — asserting Sea reads as a plausible water color (e.g. a
      blue-dominant hue in a sensible lightness range, distinct from the
      other terrain bands) rather than asserting near-zero lightness.
      Rename the test to match what it now actually checks.

      Renamed to `sea_reads_as_a_dark_blue_not_a_void`: asserts hue in
      `180..=260` (blue range), saturation `> 0.1` (reads as a color, not
      gray/black), lightness in `0.05..0.3` (same low-lightness family as
      the other bands, not near-black or bright).
- [x] No change to `TerrainKind::Sea`'s gameplay meaning (`is_placeable_kind`,
      `reinject_environment_sources`, `apply_environment_overlay`) — this is
      a color-only change.

      Confirmed: only `terrain_color`'s `Sea` match arm changed.
      `is_placeable_kind`, `reinject_environment_sources`,
      `apply_environment_overlay` untouched.
- [x] Check `draw_terrain_overlay`'s Sea↔land coastline/boundary drawing
      (task 068) still reads correctly against the new color — a boundary
      line whose contrast was tuned against a near-black Sea might need a
      matching adjustment if it's now hard to see (or too strong).

      `boundary_coastline_color()` (light beige, `rgba(220, 220, 208, 210)`)
      keys off `TerrainKind::Sea` as an enum match, not the fill color, and
      stays a light line against a still-dark (if now blue) fill — no
      adjustment needed; left as the user's own live check to confirm the
      contrast still reads well.
- [x] `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test` clean.
- [x] Verified live via `cargo run` across a couple of seeds: Sea now reads
      as a body of water, not a hole in the map.

      Confirmed by the user (2026-08-10): "funziona."

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/render.rs` | `terrain_color` (`~1368-1375`, the `Sea` branch), `sea_stays_near_black` test (`~1510-1516`), `draw_terrain_overlay`'s boundary-line drawing (task 068) — check contrast against the new color. |

---

## 🧩 Technical Context

`cell_color` (`render.rs:1305+`) calls `terrain_color` for the empty-cell
branch, then `toxicity_tint` on top when `cell.toxicity > 0`. The other
three bands: `Plain: hsl(130, 0.20, 0.09)`, `Hill: hsl(125, 0.22, 0.14)`,
`Mountain: hsl(30, 0.10, 0.19)` — all desaturated, low-lightness, in the
same "console/lab" palette family. A credible Sea color should sit in that
same family (low saturation, low-but-not-near-zero lightness) with a blue
hue, not introduce a bright/saturated "aquarium blue" that breaks the flat-
muted-palette rule task 068 established for every other band.

---

## ⚠️ Constraints and Caveats

- **Style**: presentation-only change, stays in `render.rs`, no
  `sim`/`world`/`config` involvement (`TerrainKind`'s gameplay meaning is
  unaffected).
- Keep it a *flat* color per task 068's rule — no gradient, no animation;
  that's explicitly out of scope here (task 081 already covers "should the
  world visually breathe," this task is a one-time palette correction).
- Don't touch `toxicity_tint`, `heat_color`, or any other `render.rs`
  color function — scope is the one `Sea` branch of `terrain_color`.

---

## 🔗 Dependencies

- **Depends on**: 068 (original `terrain_color`, being partially reversed
  here), 085 (gave Sea real mechanical meaning, the reason this task
  exists).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/093-sea-color-reads-as-water-not-void.md)"$'\n\nExecute this task in the current project.'
```
