# Task 136 — Make the hidden matrix necessary, not optional

> **ID**: `136`
> **Category**: Balance
> **Priority**: 🔴 P1
> **Estimate**: ~2h (plus measurement time)
> **Assigned to**: unassigned
> **Session**: 2026-08-27 redesign adoption planning

---

## 🎯 Objective

Make environmental adaptation alone yield roughly an energy **break-even**
(survive, don't reproduce), so that only a **positive** matrix interaction
provides the margin to cross the reproduction threshold, and a **negative** one
is what pushes from break-even into real decline.

Design source: `redesign/processed/abiogenesis-matrix-necessity-balance.md`.

---

## ⚠️ Two findings from the code that the design document does not have

**Both were verified in-session against the current build. Read them before
writing any number.**

### 1. There is no interaction scale coefficient. The intensities enter raw.

`sim.rs:715` does `interaction_delta += entry as f32;` and `sim.rs:766` sums it
straight into the energy update:

```rust
let new_energy = organism.energy + gain + interaction_delta
    - upkeep - crowding_penalty - predation_loss[idx];
```

So a single `±2` matrix entry is worth ±2.0/pulse today, against `base_upkeep`
of `0.5`. The document's Phase-0 question ("does a scale coefficient already
exist?") is answered: **no**. A new `EnergyConfig::interaction_scale` must be
introduced (doc's starting proposal: `0.15` per unit of intensity).

### 2. The matrix is ignorable *by construction*, not because it is weak.

- `generate_matrix` (`world.rs:2725`) always zeroes the diagonal.
- `draw_species_tags` (`world.rs:2815`) exhaustively searches for a tag set with
  `net_self_interaction == 0`, descending in size until it finds one —
  guaranteed exact zero, never "closest to zero". The same constraint is
  enforced on `apply_splice` (`input.rs:579`, `:592`) and on speciation
  (`sim.rs:453`).

Consequence: **inside a single-species blob, `interaction_delta` is exactly
zero.** The matrix only acts on cells at the interface between *different*
species. That is the real mechanism behind "the matrix is optional" — not small
coefficients. Scaling the coefficients up makes interfaces more violent without
making the matrix more *necessary*; what makes it necessary is that a
monoculture cannot grow on environment alone.

(Caveat to preserve: `tag_gate_satisfied` filters pairs by terrain, so exact
same-species cancellation holds only when both cells' terrain gates agree —
post-task-096 the "same-species neighbour is perfectly neutral" invariant is
already approximate.)

---

## 🚧 Decision this task must resolve FIRST, before writing coefficients

`crowd_factor` and the per-cell **carrying capacity** introduced by task 137 do
the same job — capping local density — and no design document reconciles them.
`draw_species_tags`' own doc comment states that crowding is currently "the only
thing that caps local density."

Under the document's proposed coefficients plus finding 2:

| Case | net/pulse |
|---|---|
| isolated, optimal | `0.7 × 0.8 × 1 − 0.5 = +0.05` |
| 2 same-species neighbours | `+0.05 − 0.30 = −0.25` |
| 8 same-species neighbours | `+0.05 − 1.20 = −1.15` |

That is past the document's stated goal ("a lone expanding species should be an
energetic dead end") and into "a monoculture cannot cluster at all."

**Decide and record**: is the cap the carrying capacity (and therefore
`crowd_factor` should not apply, or should apply reduced, to same-species
neighbours), or is it `crowd_factor` (and capacity is only a rendering cap)?
Everything downstream in this task depends on the answer.

---

## 📋 Acceptance Criteria

- [ ] The carrying-capacity vs `crowd_factor` decision above is made, recorded
      in this file's Resolution section with its reasoning, and implemented.
- [ ] `EnergyConfig::interaction_scale` exists and is applied to
      `interaction_delta` in `sim.rs`.
- [ ] Metabolism gains retuned by the **same ratio** across all four metabolisms
      (the doc's starting point: `÷2.5` — photolithic and chemolithotroph
      `2.0 → 0.8`, predator `drain_cap` `2.0 → 0.8`, decomposer `extract_rate`
      `1.5 → 0.6`). Leaving three of four untouched would make them
      disproportionately strong. Per-metabolism `upkeep` differences stay.
- [ ] Sanity check preserved: a photolithic in low light (`0.2`) still dies
      (`gain 0.16` vs `upkeep 0.5`) — the light niche (GDD §5.9) and the §5.8
      anti-degeneration defences must survive the retune.
- [ ] `tests/balance.rs` green on all four properties across its seed sweep — in
      particular no regression toward total extinction, which is the obvious
      failure mode of lowering every gain at once.
- [ ] **Task 134's harness re-run** and compared against its recorded baseline.
      The exploiter must no longer win systematically. Result recorded here.
- [ ] `assets/sim_config.ron` updated in the same commit.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/config.rs` | `EnergyConfig` (`base_upkeep 0.5`, `crowd_factor 0.15`, `photolithic_metabolism_gain 2.0`, `chemolithotroph_metabolism_gain 2.0`, `predator_upkeep`, `decomposer_upkeep`, `residue_on_death`) — new `interaction_scale` goes here. |
| `src/sim.rs` | `sim::step` lines ~660-790: gain, `interaction_delta` accumulation, costs, energy update. |
| `src/world.rs` | `generate_matrix` (2725), `draw_species_tags` (2815), `net_self_interaction` (2884) — the constraints behind finding 2. |
| `assets/sim_config.ron` | Sync (task 128). |
| `tests/balance.rs` | The regression guard for this whole task. |

---

## ⚠️ Constraints and Caveats

- **The numbers in the design document are a shape and an order of magnitude,
  not final values.** They were also computed against `era_ticks = 25`; task 135
  has since changed the clock. Re-derive against the new season/era length.
- Do **not** weaken the `net_self_interaction == 0` constraint to "fix" the
  monoculture case. It exists because of a real playtest failure (task 048:
  one species saturating the entire grid, `PROJECT_PLAN.md` has the writeup).
- Do not tune `crowd_factor` blind: the same investigation already tried a
  stronger/nonlinear crowding penalty and rejected it — strong enough to matter
  for cross-species reinforcement, it also crushed normal populations toward
  extinction.
- The `Omeostasi` objective and the newborn-incubation pacing proposal both
  acquire meaning from this change but are **not** in scope here.

---

## 🔗 Dependencies

- **Depends on**: 134 (baseline measurement), 135 (final era/season length)
- **Blocks**: 137

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/136-matrix-necessary-balance.md)"$'\n\nExecute this task in the current project.'
```
