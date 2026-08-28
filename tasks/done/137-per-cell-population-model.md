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

- [x] `Cell.organism: Option<Organism>` becomes an optional **population**
      (species, count, aggregate energy). A cell is **dedicated to one species**
      — never shared.
- [x] Gain and upkeep stay **per capita**, computed with task 136's coefficients
      and applied to the aggregate. No coefficient changes here.
- [x] Reproduction becomes continuous growth: when average per-capita energy
      crosses the threshold, the cell's count grows and the aggregate energy
      drops by the corresponding cost.
- [x] **Carrying capacity** per cell exists as a `SimConfig` value, consistent
      with the decision task 136 recorded about capacity vs `crowd_factor`.
- [x] **Breakout**: when population exceeds capacity, the excess migrates to an
      adjacent cell that is **empty or already the same species** — never into a
      cell held by a different species. Tie-breaking between equally valid
      directions must be deterministic (seeded, no `HashMap` iteration).
- [x] **Saturated with no outlet** — cell at capacity, every neighbour held by a
      different species — is detected and: (a) feeds the excess energy into
      local selection pressure under the existing "environmental mismatch"
      stimulus (§5.11), and (b) is exposed as a **flag readable by the rendering
      layer**, not only as an input to the pressure calculation. Task 141
      depends on (b).
- [x] Matrix interaction is **by presence, not by quantity**: a neighbouring cell
      carrying a tag contributes once regardless of how many individuals it
      holds. Task 136's `interaction_scale` and retune stay valid unchanged.
- [x] Clean-observation weighting (`1/(1+confounders)`, GDD §7) keeps its
      formula; only what counts as a neighbour changes (population-cells instead
      of organisms).
- [x] `Cull` zeroes the whole local population of the targeted cell.
- [x] `tests/balance.rs` updated — it currently assumes one organism per cell —
      and green.
- [x] `assets/sim_config.ron` updated in the same commit.
- [x] `cargo test`, `tests/determinism.rs`, `tests/run_reproducibility.rs` and
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

---

## ✅ Resolution

`Organism` is now `Population { species, count: u32 (>=1), energy: f32 (aggregate),
born_season, blocked: bool }`; `Cell::organism` is `Cell::population`. `Copy`
preserved — the tick's snapshot/scratch double-buffer needed no changes beyond
the field rename.

**Per-capita vs aggregate.** Photolithic/Chemolithotroph gain is genuinely
per-capita (`light`/`toxicity` × coefficient × fit) and is multiplied by
`count` to get a total. Predator/Decomposer gain is already an aggregate
shared-resource draw from the existing pre-pass — left untouched, which means
crowding now self-limits those two metabolisms for free (same fixed pool,
split more ways as `count` grows). Interaction is presence-based per cell
(unchanged scan) and multiplied by `count` for a total, same treatment as
gain. Upkeep and crowding scale by `count` the same way.

**Growth** is a `while (energy / count) >= repro_threshold { count += 1;
energy -= repro_cost }` loop, gated by `born_season < world.season` exactly
like the old reproduction check. Each increment fires `OrganismBorn` — the
event is "an individual crossed the threshold," which is still true whether
the individual stays local or (below) breaks out.

**Breakout**, once `count` exceeds `cell_carrying_capacity`: the excess and
its proportional energy share look for a neighbour that is either empty
(and placeable) or already the same species under its own capacity —
candidates collected from the snapshot in index order, one `rng_mut()` draw,
`world.scratch` re-checked before committing (same contention pattern the old
reproduction code used for its target cell). No valid candidate sets
`blocked: true` on the origin population and routes the excess energy into
`accumulate_selection_pressure`'s `terrain_mismatch` bucket for the cell's
current terrain — the function gained a 10th parameter
(`extra_terrain_mismatch`) rather than a parallel accumulator, so the
existing crossed-guard/threshold/dominant-terrain logic didn't need
duplicating.

**Death stays all-or-nothing per cell**, not a partial die-off: once the
aggregate can no longer sustain any of its members, the whole local
population collapses, same shape the one-organism model always had. This was
a real decision, not forced by the acceptance criteria — a per-individual
starvation trickle was considered and rejected for now: it would need its own
call on how `residue_on_death` scales with deaths-per-tick, and the design
doc gives no signal either way. `Cull` already relied on "empty the cell is
decisive," so keeping starvation symmetric with it seemed the safer default
until a future task motivates something finer-grained.

**`AdjacencyExposure` staleness key changed** from `Organism::born_season`
(task 136b) to `owner_species: Option<SpeciesId>` — an aggregate population
has no single birth season, but "a different species now holds this cell" is
exactly the same staleness signal task 136b needed born_season for.

**Carrying capacity = 6, picked, not swept**: a new knob, not a retune of
136's coefficients (which are untouched). `repro_threshold / repro_cost = 2`
growth events per `repro_threshold` worth of aggregate energy, so 6 lets a
population visibly grow through a few crossings before breakout pressure
starts, without one cell silently absorbing a large share of the grid.

**Population-counting readouts.** `objectives::population_of` and
`notebook.rs`'s species catalog panel now sum `count` across occupied cells
(real individuals) instead of counting cells — the objective/HUD-facing
numbers that describe "how many individuals" needed the fix. `ui.rs`'s
average-energy panel now weights by `count` for the same reason.
`cluster.rs` (task 139's concern) and `tests/balance.rs`'s `population()`
were deliberately left counting **occupied cells**, not individuals: both
measure grid-level occupancy/saturation/extinction, where "is this cell
occupied" is the right unit regardless of how many individuals are in it —
verified by running the full `tests/balance.rs` suite unchanged and green,
not just by reasoning about it.

**`examples/two_bot_survey.rs`** compiles and runs against the new model
(mechanical field/type rename only) but its `peak_population` metric still
counts occupied cells, same as `tests/balance.rs`'s. Once bots' seeded
populations start growing past `count: 1`, this undercounts true individuals
— not fixed here, since the harness isn't in this task's acceptance criteria
and 136's recorded baseline numbers were measured against the old counting.
Flagging so a future harness change doesn't silently compare against a
baseline that no longer means the same thing.

Full suite green: `cargo test` (200 lib + 101 doc-adjacent unit tests across
modules + integration suites, including `tests/balance.rs`,
`tests/determinism.rs`, `tests/run_reproducibility.rs`,
`tests/config_ron_sync.rs`), `cargo clippy --all-targets -- -D warnings`
clean, `cargo fmt` applied.
