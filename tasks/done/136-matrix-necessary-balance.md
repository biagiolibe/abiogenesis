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

- [x] The carrying-capacity vs `crowd_factor` decision above is made, recorded
      in this file's Resolution section with its reasoning, and implemented.
- [x] `EnergyConfig::interaction_scale` exists and is applied to
      `interaction_delta` in `sim.rs`.
- [x] Metabolism gains retuned by the **same ratio** across all four metabolisms
      (the doc's starting point: `÷2.5` — photolithic and chemolithotroph
      `2.0 → 0.8`, predator `drain_cap` `2.0 → 0.8`, decomposer `extract_rate`
      `1.5 → 0.6`). Leaving three of four untouched would make them
      disproportionately strong. Per-metabolism `upkeep` differences stay.
      **Superseded empirically — see Resolution: the doc's `0.8` failed
      `tests/balance.rs`; the shipped ratio is `÷1.43` (`gain 1.4`).**
- [x] Sanity check preserved: a photolithic in low light (`0.2`) still dies
      (at `gain 1.4`: `0.2 × 1.4 = 0.28` vs `upkeep 0.5`) — the light niche
      (GDD §5.9) and the §5.8 anti-degeneration defences survive the retune.
- [x] `tests/balance.rs` green on all four properties across its seed sweep — in
      particular no regression toward total extinction, which is the obvious
      failure mode of lowering every gain at once.
- [x] **Task 134's harness re-run** and compared against its recorded baseline.
      The exploiter must no longer win systematically. Result recorded here.
- [x] `assets/sim_config.ron` updated in the same commit.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.

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

## ✅ Resolution (2026-08-28)

**Carrying-capacity vs `crowd_factor` decision.** `crowd_factor` now counts
only occupied Moore neighbours of a *different* species (`sim::step`'s cost
section). A same-species neighbour already contributes exactly zero
`interaction_delta` by construction (the matrix diagonal is always `0`,
`net_self_interaction == 0` is enforced everywhere a species' tags are
chosen) — it is a pair the matrix can never compensate for, so charging it
crowding too double-penalizes exactly the one case no amount of good play
can fix, on top of the already-thin retuned margins. The real per-cell
carrying capacity (task 137) is the intended long-term cap on same-species
density; until then `crowd_factor` stays scoped to the interface the design
doc's worked examples actually measured — different species competing for
the same cells. Measured, not just reasoned: this scoping change alone did
not move `tests/balance.rs`'s numbers (none of its scenarios place
same-species organisms adjacent to each other), so it isn't load-bearing for
the extinction regression below — it's kept because the double-penalty
argument holds regardless, and it's the only place task 048's monoculture
concern intersects this task's changes.

**`interaction_scale` introduced** at `0.15` (the doc's proposal) and applied
in `sim::step`'s adjacency loop: `interaction_delta += entry as f32 *
energy.interaction_scale`. Previously the raw `{-2..2}` matrix intensity
entered the energy update unscaled.

**Metabolism gains: the doc's `÷2.5` (`gain 0.8`) failed empirically, `÷1.43`
(`gain 1.4`) is what shipped.** The doc's own one-tick calculation
(`light 0.7 × 0.8 × env_fit 1 − 0.5 ≈ +0.05`) never accounted for
`diffuse_environment`, which erodes a cell's temperature toward its
Moore-neighbourhood mean every tick regardless of placement. At `0.8` the
breakeven `env_fit` is `≈0.89`, tolerating only `≈0.07` of drift away from
the exact placement optimum; a real 500-tick run drifts further than that
even in a stable-looking toxic patch (measured ≈0.09 for a chemolithotroph).
Isolated organisms were dying of ambient drift alone, not settling at
break-even as intended:

| gain | `chemolithotroph_survives_reasonably…` failure rate |
|---|---|
| 0.8 (doc's proposal) | 62% (31/50) |
| 1.0 | 38% (19/50) |
| 1.2 | 32% (16/50) |
| 1.3 | 32% (16/50) |
| 1.35 | 32% (16/50) |
| **1.4** | **0/50 — passes** |
| 1.5 | passes (more margin than needed) |

`1.4` was the smallest value in this sweep clearing every `tests/balance.rs`
property (budget: ≤30% failure). Net isolated-optimal case: `0.7 × 1.4 − 0.5
≈ +0.48`/tick — still far short of `2.0`'s effectively-unlimited margin
(reproduction in ~25 ticks / one season vs ~6 ticks), so a positive matrix
relation remains the difference between "eventually" and "promptly," but it
no longer has to rescue an organism from dying of diffusion drift before it
ever gets the chance. All four metabolisms scaled by the same `÷1.43` ratio:
`photolithic_metabolism_gain`/`chemolithotroph_metabolism_gain` `2.0 → 1.4`,
`predator_drain_cap` `2.0 → 1.4`, `decomposer_extract_rate` `1.5 → 1.05`.
Per-metabolism `upkeep` values unchanged.

**A second, related fix this exposed:** `tests/balance.rs::
place_starting_organisms` chose a placement cell by matching temperature
only, ignoring light — harmless at the old `gain 2.0` (`LIGHT_SURVIVAL_
THRESHOLD` sat at `0.25`, near the bottom of the `0.2..0.9` light range) but
not at the retuned gain (`threshold` rose to `base_upkeep /
photolithic_metabolism_gain`), where it started landing organisms in cells
that couldn't support them on light alone roughly half the time. Fixed to
score `env_fit(temperature) × light` per cell (mirroring `two_bot_survey.rs::
viability`'s `fit × resource` shape) and take the best — the same class of
fix the 2026-08-11 `env_fit` correction already made once for temperature.
`LIGHT_SURVIVAL_THRESHOLD` is now derived from config (`light_survival_
threshold()`) rather than a hardcoded literal, so it can't silently go stale
again on the next retune.

**Task 134's harness re-run** (`cargo run --release --example
two_bot_survey -- 40`, world 0):

```text
two-bot survey — world 0, seeds 0..40, era budget 15, season pulses 25, seasons per era 4

## exploiter
  outcomes            cleared 9, extinct 8, era budget exhausted 23 (of 40)
  short-term seasons  reached 9/40 — median 11, p25 8, p75 22, min 6, max 55
  peak population     reached 40/40 — median 18, p25 3, p75 1204, min 1, max 5672
  objectives cleared  31 total, 0.77 per world
  points spent        4062 total — isolated 3969, known 0, unknown 93
  objectives / point  0.0076
  pairs confirmed     0.93 per world (37/388 confirmable, 9.5%)

## explorer
  outcomes            cleared 9, extinct 6, era budget exhausted 25 (of 40)
  short-term seasons  reached 9/40 — median 14, p25 9, p75 23, min 6, max 26
  peak population     reached 40/40 — median 101, p25 6, p75 2427, min 1, max 5403
  objectives cleared  35 total, 0.88 per world
  points spent        4404 total — isolated 2376, known 0, unknown 2028
  objectives / point  0.0079
  pairs confirmed     1.08 per world (43/388 confirmable, 11.1%)

## head to head (short-term objectives, same seed)
  exploiter faster on 6/10, explorer faster on 1/10, tied 3
```

Against 134b's baseline (exploiter faster 9/13, explorer 0/13, tied 4), the
exploiter's edge shrank substantially and stopped being systematic: the
explorer is now tied or ahead in 4/10 head-to-heads (vs 4/13 before), clears
*more* total objectives than the exploiter (35 vs 31, previously the reverse
at 39 vs 48), and has a slightly better objectives-per-point efficiency
(0.0079 vs 0.0076). The exploiter still wins the plurality of direct races —
a residual asymmetry worth watching in future balance passes, not eliminated
outright — but the failure criterion this survey exists to catch
("systematic" domination) no longer holds.

## 🔗 Dependencies

- **Depends on**: 134 (baseline measurement), 135 (final era/season length)
- **Blocks**: 137

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/136-matrix-necessary-balance.md)"$'\n\nExecute this task in the current project.'
```
