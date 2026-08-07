# Task 057 — Species/reproduction-threshold legibility

> **ID**: `057`
> **Category**: UI
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-07 (player-raised UX gap, same session as task 056)

---

## 🎯 Objective

The player can't tell how close an organism is to reproducing, and the notebook's
per-species genome readout is technically complete but not intuitive. Two fixes,
both purely presentational:

1. Show the reproduction energy threshold in the HUD's Population panel, not just
   in the debug-only F2 overlay.
2. Make the notebook's species catalog line (`temp 0.62±0.15`) readable without
   requiring the player to interpret a raw `[0,1]` float.

Raised directly by the user: species info isn't clear in the HUD, the reproduction
threshold is invisible outside `cargo run`'s debug build, and the notebook's raw
floats aren't intuitive.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors and `cargo clippy -- -D warnings` is clean.
- [ ] The Population panel's per-species line shows the reproduction threshold
      alongside average energy (e.g. `avg energy 6.20 / 10.0`), reading
      `config.energy.repro_threshold` rather than hardcoding it.
- [ ] The notebook's species catalog line (`catalog_panel` / `species_catalog_line`)
      gets a human-readable annotation for `temp_optimum`/`temp_tolerance` (e.g. a
      short descriptive label like "cold/temperate/hot", or a color swatch) shown
      *alongside* the existing raw numbers — the raw values must stay, since
      `Splice`'s `ShiftTempOptimum` math needs the precise figure.
- [ ] No new `SimConfig` fields: `repro_threshold` and any thresholds used for the
      readable annotation are either already in config or derived from it, per
      CLAUDE.md's "no magic numbers" rule.
- [ ] Existing tests pass; add unit tests for any new pure formatting/labeling
      function (following the pattern of `text.rs`'s other `*_line` functions).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `species_stats` (population/avg-energy computation, ~line 682) and the Population panel rendering (~line 296-311). |
| `src/text.rs` | `population_line` (~line 177) and `species_catalog_line` (~line 330) — the formatting functions to change. |
| `src/notebook.rs` | `catalog_panel` (~line 558) — renders the per-species catalog line. |
| `src/config.rs` | `EnergyConfig::repro_threshold` (~line 146) — the value to surface. |

---

## 🧩 Technical Context

- **Current behavior**: `text::population_line` renders `"  {label}: {population} · avg energy {avg_energy:.2}"` with no threshold context. `text::species_catalog_line` renders `"{label}: {metabolism:?} · temp {temp_optimum:.2}±{temp_tolerance:.2}"` — raw floats in `[0,1]`, no unit, no link to the map's actual hot/cold rendering.
- **Desired behavior**: the Population panel line reads something like
  `"  {label}: {population} · avg energy {avg_energy:.2} / {repro_threshold:.1}"`.
  The catalog line keeps its raw numbers but adds a short readable label or color
  cue for the thermal optimum — pick whichever is cheaper given task 057's sibling
  discussion (a color-based encoding for temperature is being designed separately
  for the grid itself; if that lands first, reuse the same color scale here for
  consistency rather than inventing a second one).
- `repro_threshold` is a single global `f32` in `SimConfig`, identical for every
  species (not per-species), so it's cheap to thread through: `hud_panel` already
  has `config: Res<SimConfig>` in scope for the Population panel section.

---

## 🔨 Suggested Implementation

1. Update `text::population_line` to accept `repro_threshold: f32` and format it
   into the line; update its one call site in `ui.rs`'s Population panel loop to
   pass `config.energy.repro_threshold`.
2. Decide the catalog annotation format (readable tercile label vs. color swatch)
   — prefer whichever is simplest to keep consistent if/when the map gets a
   temperature color encoding (separate, not-yet-scoped work). A simple tercile
   label (e.g. "cold"/"temperate"/"hot" from fixed cutoffs already implicit in the
   GDD's gradient description, GDD §5.2) is the lowest-risk starting point.
3. Update `text::species_catalog_line` and `notebook.rs`'s `catalog_panel` call
   site accordingly.
4. Add/update unit tests in `text.rs` or `ui.rs`'s test modules for the new
   formatting.
5. Run `cargo test` and `cargo clippy -- -D warnings`.

---

## ⚠️ Constraints and Caveats

- **Style**: Follow the conventions in `TECH_DESIGN.md`; keep all player-facing
  strings behind `src/text.rs` per task 034's centralization.
- **No magic numbers**: any cutoffs used for a readable temperature label must
  live in `SimConfig`, not be hardcoded in `text.rs`/`notebook.rs`.
- **Determinism**: this is presentation-only — no changes to `sim`/`world`/`config`
  simulation logic.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: none (may want to stay consistent with a future map-level
  temperature color encoding, currently under discussion, not yet a task)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/057-species-reproduction-threshold-legibility.md)"$'\n\nExecute this task in the current project.'
```
