# Task 138 — The tick as an explicit phased pipeline

> **ID**: `138`
> **Category**: Refactor + Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-27 redesign adoption planning

---

## 🎯 Objective

Restructure `sim::step` from "one formula producing updated energy" into an
explicit pipeline with **three outputs** — updated energy, accumulated selection
pressure, and emitted observations/events — all fed by the *same* intermediate
values, computed once and passed forward.

The concrete risk this removes: the same quantities being recomputed in three
places and drifting apart, so that notebook, reveal and narration end up telling
inconsistent versions of the same event.

Design source: `redesign/processed/abiogenesis-tick-pipeline.md`.

---

## 🧩 Scope reality check — this is mostly a restructure

**Already exists, do not rebuild**: `interaction_harm`, `SelectionPressure` and
`SelectionThresholdCrossed` (task 106), speciation (task 107),
`AdjacencyObserved` carrying `n_confounders` (`sim.rs:718-726`), and the
dominant-stimulus classification (`sim.rs:368-390`).

**Genuinely new**: the habitat gate (phase 0), biome-modulated `crowd_factor`
and residue decay (phase 4), the protected-cell hook (phase 3), and making the
intermediates explicit instead of locals inside one long loop body.

---

## 📋 Acceptance Criteria

The pipeline, per organism-population, in order:

- [ ] **Phase 0 — habitat gate (new).** A biome can make a cell outright
      uninhabitable, independent of scalars (deep water is not "low light and
      cold", it is a place a land organism cannot be). Failing the gate kills the
      population with cause `habitat` and stops the pipeline for it. Needs a
      biome → habitable lookup in `SimConfig`.
- [ ] **Phase 1 — environmental fit.** Unchanged maths. Produces `env_fit` **and
      `env_mismatch`**, the latter *kept* rather than recomputed later as a
      selection-pressure stimulus.
- [ ] **Phase 2 — metabolic gain.** Unchanged. **The biome does not enter here**
      — a biome already *is* a combination of scalars, so a biome multiplier on
      top would double-count and make the two impossible to tune independently.
- [ ] **Phase 3 — matrix interaction.** Produces `interaction_delta` and,
      separately, `interaction_harm`. **Keeps the per-neighbour breakdown**, not
      just the sum — task 149's inspection card needs the contribution per
      neighbouring tag. Includes a **"protected cell" hook**: if a cell is in
      protected state this phase contributes zero. (The `Isola` action itself is
      *not* being implemented — only the attachment point, which is near-free now
      and expensive to retrofit.)
- [ ] **Phase 4 — costs.** `upkeep` plus crowding. **The biome enters here, not
      in phase 2**: `crowd_factor` becomes a per-biome lookup instead of a global
      constant (a forest sustains more density than a peak), and residue decay
      rate becomes per-biome (a lake retains, a slope disperses) — the latter
      outside the per-organism pipeline. Both are *costs*, so no double-count
      with the scalars.
- [ ] **Phase 5 — energy update**, then starvation death and growth. Emits
      birth/death events **with cause** (starvation / interaction / habitat /
      temperature).
- [ ] **Phase 6 — selection pressure accumulation**, using the values already
      computed above. No recomputation. Energy and pressure keep **sharing
      inputs but not values** — deliberately two separate clocks.
- [ ] **Phase 7 — observation and event emission**: notebook adjacencies with
      their weights, ranked event candidates for the reveal, and the `Cull`
      knockout observation entering through *this* phase rather than via a
      separate path (task 146 wires `Cull` up; the entry point belongs here).
- [ ] Same-species neutrality preserved and **explicitly tested**: see caveat.
- [ ] `assets/sim_config.ron` updated for the new per-biome tables.
- [ ] `cargo test`, `tests/determinism.rs` and `cargo clippy -- -D warnings`
      clean. Determinism guarantees (GDD §5.7) unchanged.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `sim::step` (the whole loop body), `accumulate_selection_pressure`, `AdjacencyObserved`, `tag_gate_satisfied`. |
| `src/config.rs` | New per-biome tables (habitability, `crowd_factor`, residue decay). |
| `src/world.rs` | `Cell::biome`, `moore_neighbours`, `TagMatrix::get`. |
| `assets/sim_config.ron` | Sync (task 128). |

---

## ⚠️ Constraints and Caveats

- **The maths does not change.** `env_fit`, `gain`, `interaction_delta`,
  `upkeep`, crowding, reproduction: identical. Processing order and determinism
  guarantees: identical. What changes is where values come from and that they
  are kept.
- **Same-species neutrality is already approximate.** `tag_gate_satisfied`
  filters `(their_tag, my_tag)` pairs by each cell's terrain, so the
  `net_self_interaction == 0` cancellation is exact only when both cells' gates
  agree — since task 096's conditional tags, that is not guaranteed. Preserve
  the property and add a test that pins the behaviour, rather than discovering
  the discrepancy later during balance work.
- Numeric values for the new per-biome parameters are **out of scope** — define
  them alongside the biome roster and validate in playtest. Ship defaults that
  are neutral (all-habitable, uniform `crowd_factor`) if calibration isn't ready,
  so the structure lands without a balance change riding along.

---

## 🔗 Dependencies

- **Depends on**: 137
- **Blocks**: 141, 142, 146, 149, 157

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/138-tick-pipeline-explicit-phases.md)"$'\n\nExecute this task in the current project.'
```
