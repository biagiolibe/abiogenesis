# Task 142 — Name the dominant stimulus in the speciation reveal

> **ID**: `142`
> **Category**: Feature
> **Priority**: 🟡 Media (Phase 1b)
> **Estimate**: ~30min
> **Assigned to**: Claude CLI
> **Session**: 2026-08-28

---

## 🎯 Objective

The end-of-era reveal card names *what* happened (a species evolved) but not
*why* — the player can't connect the event to their own prior choices even
though the mechanical link exists. Name the dominant stimulus
(`sim::DominantStimulus`, already computed to decide which edit `speciate`
applies) in the reveal's generated text.

Design source: `redesign/processed/culture-shock-friction-fixes.md`,
Intervento 3.

---

## 📋 Acceptance Criteria

- [x] `cargo build`/`clippy -- -D warnings` clean.
- [x] `sim::DominantStimulus`/`dominant_stimulus` made `pub`; `EraEvolutionReveal`
      gains a `dominant_stimulus` field, computed in `build_era_reveal` from
      the same `SelectionThresholdCrossed` event `speciate` consumes.
- [x] `text::era_reveal_evolution_line` appends a natural-language cause
      clause, one per `DominantStimulus` variant, no raw numbers.
- [x] `screens.rs`'s reveal card passes `entry.dominant_stimulus` through.
- [x] Unit test asserting the three variants produce three different clauses.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `DominantStimulus`/`dominant_stimulus` (now `pub`), `EraEvolutionReveal` (+field), `build_era_reveal` (computes it before `speciate` consumes the event). |
| `src/text.rs` | `era_reveal_evolution_line` (+`stimulus` param), new `dominant_stimulus_clause`. |
| `src/screens.rs` | `era_reveal_screen_ui` passes `entry.dominant_stimulus`. |

---

## 🔨 Implementation

Reused the exact tie-break logic `speciate` already applies to pick its
edit — no new calculation, only new exposure. Clauses are one generic
sentence per category (interaction harm / terrain mismatch / toxicity),
matching GDD §5.11's existing three-stimulus framing; the design doc's own
example (naming a specific offending tag) isn't derivable from the data the
sim actually accumulates (`SelectionThresholdCrossed` is scalar per
category, not per-neighbour), so this stays at the category level rather
than inventing detail.

---

## 🔗 Dependencies

- **Depends on**: 107 (`DominantStimulus`/`speciate`), 140 (`EraReveal`/reveal beat).
- **Blocks**: none.
