# Task 137 — Per-cell population model

> **ID**: `137`
> **Category**: Architecture (simulation model)
> **Priority**: 🔴 P1
> **Estimate**: large — split if it exceeds ~2h of focused work
> **Assigned to**: unassigned
> **Session**: 2026-08-27 redesign adoption planning

---

## 🎯 Objective

Replace "one organism per cell" with "**one population per cell**": each cell
holds a count of individuals of a single species plus that local population's
aggregate energy, up to a per-cell carrying capacity.

Design source: `redesign/processed/culture-shock-population-model-aesthetic.md` (the
simulation half — the pixel-grain visual register from the same document is
task 151, deliberately in a later phase so the HUD isn't restyled and then
rebuilt).

**Why it is a model change and not a drawing change**: with one organism per
cell, the overview's density view is a pictorial illusion on top of a model that
does not support it. This task makes density a real quantity.

---

## 📋 Acceptance Criteria

- [ ] `Cell.organism: Option<Organism>` becomes an optional **population**
      (species, count, aggregate energy). A cell is **dedicated to one species**
      — never shared.
- [ ] Gain and upkeep stay **per capita**, computed with task 136's coefficients
      and applied to the aggregate. No coefficient changes here.
- [ ] Reproduction becomes continuous growth: when average per-capita energy
      crosses the threshold, the cell's count grows and the aggregate energy
      drops by the corresponding cost.
- [ ] **Carrying capacity** per cell exists as a `SimConfig` value, consistent
      with the decision task 136 recorded about capacity vs `crowd_factor`.
- [ ] **Breakout**: when population exceeds capacity, the excess migrates to an
      adjacent cell that is **empty or already the same species** — never into a
      cell held by a different species. Tie-breaking between equally valid
      directions must be deterministic (seeded, no `HashMap` iteration).
- [ ] **Saturated with no outlet** — cell at capacity, every neighbour held by a
      different species — is detected and: (a) feeds the excess energy into
      local selection pressure under the existing "environmental mismatch"
      stimulus (§5.11), and (b) is exposed as a **flag readable by the rendering
      layer**, not only as an input to the pressure calculation. Task 141
      depends on (b).
- [ ] Matrix interaction is **by presence, not by quantity**: a neighbouring cell
      carrying a tag contributes once regardless of how many individuals it
      holds. Task 136's `interaction_scale` and retune stay valid unchanged.
- [ ] Clean-observation weighting (`1/(1+confounders)`, GDD §7) keeps its
      formula; only what counts as a neighbour changes (population-cells instead
      of organisms).
- [ ] `Cull` zeroes the whole local population of the targeted cell.
- [ ] `tests/balance.rs` updated — it currently assumes one organism per cell —
      and green.
- [ ] `assets/sim_config.ron` updated in the same commit.
- [ ] `cargo test`, `tests/determinism.rs`, `tests/run_reproducibility.rs` and
      `cargo clippy -- -D warnings` all clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `Cell`, `Organism`, `SimWorld`, `moore_neighbours`, scratch/double-buffer. The core type change. |
| `src/sim.rs` | `sim::step` — gain, interaction, costs, energy, death, reproduction; `accumulate_selection_pressure`. |
| `src/cluster.rs` | Cluster computation; task 139 removes the pictorial density machinery that this model makes unnecessary. |
| `src/render.rs` | Reads `Cell::organism` directly in several places. |
| `src/objectives.rs` | Population-counting objectives. |
| `src/notebook.rs` | Observation/adjacency accounting. |
| `src/ui.rs` | Biosphere panel — gets *simpler*: it reads aggregates instead of reconstructing them. |
| `tests/balance.rs` | Assumes the old model throughout. |

---

## 🧩 Technical Context

What does **not** change: the tag matrix and its relations, `Splice` (still
seeds a small initial population), biomes, the speciation mechanism itself (it
already acts at species level, not per individual), and the interaction
coefficient.

Emergent property to preserve, not to special-case: a species entirely
surrounded by different species has its growth blocked, because it has nowhere
to break out to. That is real spatial competition falling out of the model — do
not write it as a separate rule.

Performance note from the doc, worth confirming rather than assuming:
aggregating per (cell, species) is likely *cheaper* at runtime than iterating
thousands of individual organisms at high density.

---

## ⚠️ Constraints and Caveats

- **Determinism is non-negotiable** (`TECH_DESIGN.md` §5): the breakout search
  is a new source of ordering sensitivity. It must read from the snapshot like
  everything else in `sim::step`, and its neighbour preference order must be
  fixed, not incidental.
- Do **not** pull the pixel-grain rendering work in here. Detail-view shapes
  with count badges and the fill/outline energy state are task 151; the overview
  density rewrite is task 139.
- Do **not** re-tune coefficients. If the retune from 136 looks wrong under the
  new model, record it — a follow-up task, not a silent adjustment.

---

## 🔗 Dependencies

- **Depends on**: 136
- **Blocks**: 138, 139, 141, 149

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/137-per-cell-population-model.md)"$'\n\nExecute this task in the current project.'
```
