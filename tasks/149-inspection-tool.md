# Task 149 — Inspection tool: hover tooltip + per-neighbour energy breakdown card

> **ID**: `149`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~3h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

With the per-cell population model (task 137), "a count went down" no longer
even says *which* individual was affected — there are no individuals left to
observe, only an aggregate. The player has no way to see *why* a cell is the
way it is; the only direct reading today is brightness/fill = energy. Add a
two-tier inspection tool, pure observation, free and outside the action
budget:

- **Hover (always on, no click)**: a minimal label next to the cursor —
  biome name for any cell (populated or empty), plus species/population/trend
  for a populated one.
- **Click, when no action is armed**: a full card. Populated cell: species,
  origin, population, per-capita energy, a tick-style indicator toward the
  reproduction threshold, active tags, and **the last pulse's energy balance
  broken down line by line** — gain, **every neighbouring tag listed
  individually with its own signed contribution** (not one aggregated
  "matrix effect" — this is the detail that makes inspection useful), upkeep,
  crowding, net. Empty cell: biome characteristics instead (temperature/
  light/toxicity as qualitative bands, not GDD's raw numbers; a habitability
  flag if the biome can't support life).

Design source: `redesign/processed/culture-shock-inspect-tool.md` (full doc);
mockup `redesign/processed/inspect-tool.svg` shows content structure only,
not exact layout (explicitly out of scope per the doc itself).

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] Hover tooltip, no click, no budget cost, active regardless of which
      `ActionMode` is selected: shows biome name on any cell under the
      cursor; adds species name + population count + trend arrow when the
      cell is populated. Reuses whatever trend glyph the Biosphere HUD row
      already computes for population delta (`text::population_delta_label`,
      task 120) rather than inventing a second trend convention.
- [ ] `Biome` gets a player-facing display label (it has none today — biomes
      are currently only distinguishable by color/texture on the map, which
      is itself a real gap the doc calls out against the "colour is never
      the only channel" rule in `abiogenesis-cross-cutting.md`).
- [ ] Click-to-inspect card only opens when **no action is armed** — when one
      is, that click performs the action instead, unchanged from today. The
      full conflict-resolution UX (Esc cascade, an explicit "nothing armed"
      state reachable by the player) is task 150's job
      (`culture-shock-controls.md`); this task only needs *some* minimal way
      to reach "no action armed" so the click-inspect path has a real entry
      point — e.g. clicking the already-selected action's button again
      deselects it. Don't build 150's full scheme here; leave a clear seam
      for it to take over (see Dependencies).
- [ ] Populated-cell card: species + origin (seeded / indigenous /
      synthesised — origin values from task 147's catalog work; use whatever
      `Species`/`Organism` already exposes today if 147 hasn't landed yet,
      and note the gap rather than blocking on it), population count,
      per-capita energy, a notch/tick indicator toward the reproduction
      threshold (discrete steps, not a continuous bar — same idiom the HUD
      already uses elsewhere, not a new visual language), active tags.
- [ ] Balance breakdown, **last-pulse values**, one line per component:
      resource gain, **one line per distinct neighbouring tag actually
      contributing this tick** with its own signed value, upkeep, crowding
      penalty, net. Source this from a *pure, on-demand recomputation* over
      the current tick's neighbour tags (same formula `sim::step` already
      applies around `sim.rs:1045-1088` — tag-gate check, `world.matrix.get`,
      `energy.interaction_scale`), not from `AdjacencyObserved` (that stream
      fires only on adjacency **onset**, task 136b — a persisting neighbour
      contributes nothing there after its first tick, so it can't drive a
      "current balance" card) and not from new per-tick persistent storage
      (would mean tracking a full per-neighbour ledger for every cell every
      tick just to serve an occasionally-open card). A pure helper function
      in `sim.rs` (no `bevy` deps, callable from `ui.rs`/`render.rs`) that
      takes `(&SimWorld, &SimConfig, cell_index)` and returns the breakdown
      is the shape to aim for — matches TECH_DESIGN.md §5's separation and
      keeps this genuinely "no new calculation, only new exposure" per the
      design doc's own framing.
- [ ] Saturated-with-no-outlet warning line, reusing task 137's flag
      (`Population::blocked`) and task 141's existing render concept —
      exposed here textually as well, not just the on-map marker.
- [ ] Empty-cell card: biome name, temperature/light/toxicity as qualitative
      bands (reuse-and-extend `notebook::temperature_label`'s idiom — today
      private, temperature-only, thresholds derived from
      `EnvironmentConfig` bounds; needs an equivalent band function for
      light and toxicity, each config-bound-derived, no hardcoded cutoffs),
      plus a habitability flag reusing `EnergyConfig::is_habitable`
      (`sim.rs:932`).
- [ ] **Hard constraint, verify explicitly**: the empty-cell card never
      reveals whether a terrain-conditional tag gate exists on that biome —
      no icon, color, or wording that hints "something special happens
      here." This protects the same opacity the hidden matrix relies on
      (GDD §5.5); the design doc calls this out as an easy accidental leak.
- [ ] Selection state (`SelectedCell`/inspected cell) is distinct from hover
      state; the card follows the selection, not the cursor, and stays open
      until another cell is selected or Esc is pressed (closing the card,
      not the game — full Esc-cascade semantics are task 150's).
- [ ] Costs nothing: no `ActionBudget` interaction anywhere in this path.
- [ ] At least one test for the pure breakdown helper: given a hand-built
      2-3 cell grid with a known matrix, asserts the returned per-neighbour
      lines sum to the same `interaction_delta` `sim::step` computes for
      that cell, and that each line's sign/magnitude matches
      `world.matrix.get(their_tag, my_tag) * interaction_scale`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | New pure breakdown helper, likely near the phase-3 interaction loop (`~1001-1088`); `AdjacencyObserved.contribution` (`~135-140`) already anticipates this task in a doc comment but isn't the right data source (onset-only) — see acceptance criteria. `EnergyConfig::is_habitable` (`~932`). |
| `src/ui.rs`/`src/render.rs` | New hover/selection systems, tooltip + card rendering. No existing hover-cell system to build on — `input.rs::clicked_cell` is click-only. |
| `src/input.rs` | `clicked_cell` (`~87`) — the screen→cell math to reuse/extract for a hover variant; where the "click performs action if one is armed" precedence already effectively lives (each `*_on_click` system early-returns if `selected_action.0` doesn't match). |
| `src/notebook.rs` | `temperature_label` (`~1161`, private) — the qualitative-band idiom to extend to light/toxicity. |
| `src/text.rs` | New label constants/formatters for the tooltip and card. |
| `redesign/processed/culture-shock-inspect-tool.md` | Design source. |
| `redesign/processed/culture-shock-controls.md` | Task 150's doc — owns the full click/action-armed conflict resolution this task only needs a minimal seam for. |

---

## 🧩 Technical Context

- **Current behavior**: no inspection tool exists. The only per-cell
  information visible is the rendered color/fill (energy-coded) and
  whatever the HUD's Biosphere row aggregates across the whole world.
  `AdjacencyObserved` events (task 136b) carry a per-tag-pair `contribution`
  field whose doc comment already names this task, but the event only fires
  on adjacency *onset* — not a live per-tick signal.
- **Desired behavior**: hover is free and instant (biome + population
  summary); click opens a persistent, detailed, per-line energy breakdown
  for the selected cell, computed fresh each time it's asked for rather than
  tracked continuously.
- No `SelectedCell`/hover-tracking resource exists yet — this task
  introduces it.
- `ActionMode` currently has no "none selected" variant (`Seed`/`Stress`/
  `Cull`/`Splice` only) and defaults to `Seed` on start — see the click-armed
  acceptance criterion above for how this task handles that minimally.

---

## 🔨 Suggested Implementation

1. Write the pure breakdown helper in `sim.rs`: given a cell index, redo the
   tag-gate-checked neighbour loop (mirroring `~1045-1088`) but collect
   `(exerter_tag, receiver_tag, contribution)` triples into a `Vec` instead
   of only summing into `interaction_delta`. Cross-check its sum against
   `sim::step`'s own `interaction_delta` in a test.
2. Add `SelectedCell(Option<usize>)` (or `(x, y)`) as a `Resource`, written
   by a new click-handling system that only acts when `SelectedAction`
   reports "none armed" (see the minimal-deselect acceptance criterion).
3. Add a hover system reading `Window::cursor_position()` + the existing
   camera/grid math (extract the reusable core of `clicked_cell` rather than
   duplicating it) into a `HoveredCell(Option<usize>)` resource, updated
   every frame regardless of action mode or budget.
4. Render the tooltip (egui `on_hover_text`-style, or a custom floating
   label near the cursor) from `HoveredCell`; render the card from
   `SelectedCell`, calling the breakdown helper on demand each frame it's
   open (cheap: bounded by 8 neighbours × tag count).
5. Extend `notebook::temperature_label`'s pattern to light/toxicity bands;
   decide whether to keep them private to `notebook.rs` or lift into
   `text.rs`/a shared location now that both the card and the notebook need
   the same idiom.
6. Explicitly test/verify the terrain-conditional-tag non-leak constraint —
   e.g. a test that renders (or computes the data for) an empty cell on a
   biome with a conditional gate and asserts nothing in the returned
   card data structure names the gated tag.

---

## ⚠️ Constraints and Caveats

- **No new persistent per-tick state** for the breakdown — compute on
  demand, exactly the "pure exposure of already-computed data" the design
  doc insists on (it *is* new computation in the sense that the loop isn't
  reused verbatim, but it must derive from the same inputs/formula `sim::step`
  already uses, not diverge from it).
- **Keep `sim`/`world`/`config` free of `bevy::render`/`bevy_egui`** per
  TECH_DESIGN.md §5 — the breakdown helper returns plain data; only
  `ui.rs`/`render.rs` know about egui/rendering.
- **No magic numbers**: light/toxicity band thresholds derive from
  `EnvironmentConfig`, same as `temperature_label` does today.
- Exact card layout/positioning and any inspect-specific keyboard shortcut
  are explicitly out of scope per the design doc.

---

## 🔗 Dependencies

- **Depends on**: 137 (per-cell population + `Population::blocked`), 138
  (tick pipeline the breakdown mirrors), 141 (existing saturated-no-outlet
  concept to surface textually), 136b (why `AdjacencyObserved` is *not* the
  right source — informs the design, not a blocking dependency).
- **Soft dependency, not blocking**: 147 (Splice/genome-bank origin values —
  "synthesised" as a third origin) and 150 (full Esc-cascade / action-armed
  UX) both touch surfaces this task also touches. 149 should ship a minimal,
  correct version of both (origin field falls back gracefully if 147 hasn't
  landed; click-to-inspect uses a minimal deselect instead of 150's full
  scheme) rather than blocking on either.
- **Blocks**: none directly.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/149-inspection-tool.md)"$'\n\nExecute this task in the current project.'
```
