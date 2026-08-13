# Task 125 — Score-based biome classification, Palude from drainage

> **ID**: `125`
> **Category**: Refactor (worldgen)
> **Priority**: 🟡 P2
> **Estimate**: ~4h
> **Assigned to**: unassigned
> **Session**: 2026-08-13 (Phase 3 of the worldgen pipeline reassessment
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §1.3/§1.4/
> §1.5/§12.3/§12.4 — see task 123 for Phase 1 and the session's overall
> diagnosis)

---

## 🎯 Objective

`classify_biomes`'s Stage B (`world.rs:788-811`, the `TerrainKind::Plain`
branch) is a rigid priority chain: `if swamp_patch && toxic {Swamp} else if
desert {...} else if tundra {...} else if forest_patch {...} else {Plain}`.
The spec's diagnosis (§1.5) applies directly: this produces hard
discontinuities at thresholds and makes tuning fragile, because each
branch's condition is evaluated in isolation rather than as competing
scores.

Two changes, done together because they touch the same code:

1. **Replace the priority chain with continuous per-biome scores** (§12.3):
   for each `TerrainKind::Plain` cell, compute a `[0, 1]` fitness score for
   each candidate biome (Plain, Forest, Desert, Tundra, Swamp) from smooth
   compatibility curves over temperature/light/slope/water_distance, then
   pick the highest-scoring one. The existing patch-noise masks
   (`forest_waves`/`swamp_waves`) become a small additive term on top of
   the climate score, not the primary gate (§1.4) — `biome_score =
   climate_score + local_noise * small_amplitude`, not `biome =
   local_noise > threshold`.
2. **Palude's condition moves from `toxicity` to drainage** (§1.3/§12.4):
   today `Swamp` requires `cell.toxicity >= swamp_toxicity_min`, which
   conflates two unrelated concepts — a swamp is a drainage/wetness fact,
   toxicity is a separate gameplay modifier applied after. Replace the
   toxicity gate with a score built from `slope` (low) and `water_distance`
   (close) — both added by task 124. Toxicity is no longer part of what
   makes a cell Palude; it's applied afterward as a modifier the same way
   the spec's §12.4 describes (`if biome == Swamp && toxicity >=
   threshold { /* mark toxic sub-variant / feature */ }`), preserving
   today's "a toxic swamp reads differently" flavor without it gating
   identity.

---

## 📋 Acceptance Criteria

- [ ] `biome_score(cell, candidate) -> f32` (or equivalent, one function
      or match arm per candidate) implemented for `Plain`, `Forest`,
      `Desert`, `Tundra`, `Swamp`, each returning a smooth `[0, 1]` value —
      no binary comparisons in the score itself (`smoothstep`/Gaussian-style
      curves, matching the existing `env_fit` Gaussian pattern already used
      for organism-temperature fitness in `sim.rs`, for consistency of
      idiom).
- [ ] Stage B's `TerrainKind::Plain` branch becomes: compute all five
      scores, take the arg-max, break ties deterministically (fixed
      priority order used only for exact float ties, documented as such —
      not the primary mechanism).
- [ ] Swamp's score uses `slope`/`water_distance` (task 124), not
      `toxicity`. `swamp_toxicity_min` config field repurposed as a
      **post-classification** modifier threshold (e.g. controls whether a
      `Swamp` cell also gets marked toxic-flavored) rather than removed —
      confirm interaction with task 113 if it has landed by the time this
      is picked up (113 re-points `SurviveIn`'s `ZoneKind::Toxic` at
      `Biome::Palude` membership; if 113 landed first, toxic-flavored-swamp
      cells are still the same `Biome::Swamp` and remain valid `SurviveIn`
      targets — this task must not accidentally split "toxic swamp" into a
      different `Biome` variant, which would break that objective).
- [ ] Forest/Swamp patch noise (`forest_waves`/`swamp_waves`) becomes a
      small additive term on the corresponding score, not a hard
      `wave_band_sum(...) > threshold` gate. Confirm the visual effect is
      still organic patches, not a uniform wash — the noise still needs to
      matter, just not as the sole cause.
- [ ] `BareRock` (currently gated on `light` alone within
      `TerrainKind::Hill`, `world.rs:781-787`) additionally considers
      `slope` (task 124) — steep + low light reads as BareRock more
      readily than shallow + low light, per spec §12.5. Keep this a small,
      contained change within the existing `Hill` branch; full multi-band
      mountain classification (Glacier/AlpineMeadow/MountainForest, spec
      §12.5) is explicitly **out of scope** here — that's a larger, separate
      pass over the `Mountain` branch, not folded into this task.
- [ ] Existing biome-distribution assumptions in tests re-verified, not
      just left passing by accident: run a multi-seed histogram (reuse the
      pattern from the 2026-08-10 Sea/Plain/Hill/Mountain retune mentioned
      in `TerrainConfig::sea_threshold`'s doc comment) comparing biome
      fractions before/after — flag in the PR/commit if any biome's typical
      coverage shifts drastically, don't just silently accept whatever
      falls out.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes, including `tests/balance.rs` (biome identity doesn't feed
      balance directly today per a repo-wide grep, but re-run it anyway as
      a regression net).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:743-818` | `classify_biomes` — the priority chain to replace with scores. |
| `src/world.rs:781-787` | `TerrainKind::Hill` branch — BareRock gets a `slope` term added. |
| `src/config.rs` (`BiomeConfig`) | New score-curve parameters; `swamp_toxicity_min`'s role changes from gate to post-hoc modifier threshold. |
| `src/objectives.rs:318-336` | `ZoneKind::Toxic`/`Biome::Palude` membership check — must keep working if task 113 has landed; re-read its acceptance criteria before touching swamp semantics. |

---

## 🧩 Technical Context

- **Current behavior**: `Swamp` requires patch-noise gate + `toxicity >=
  swamp_toxicity_min`; the rest of the chain is temperature/light
  thresholds evaluated in a fixed if/else-if order.
- **Desired behavior**: every candidate biome for a `Plain` cell gets a
  continuous score from climate + geomorphology + a small noise term;
  highest score wins. Swamp's causal basis is drainage (slope, water
  proximity), matching how a real wetland forms, not an unrelated toxicity
  scalar.

---

## ⚠️ Constraints and Caveats

- **No magic numbers**: every curve parameter (optimum, tolerance/width,
  weight) lives in `BiomeConfig`.
- **Coordinate with task 113** if both are open at once — they touch
  overlapping territory (Palude's identity and what reads it). Land
  whichever is ready first; the other rebases.
- Keep `TerrainKind`/Stage A untouched — this task only changes Stage B
  (the `Biome` derivation), not the underlying elevation bands that
  `notebook.rs`'s `TerrainKnowledge` and `SelectionPressure::terrain_mismatch`
  key on.

---

## 🔗 Dependencies

- **Depends on**: 110, 111, 124 (`slope`/`water_distance` fields). Should
  land after or alongside 113 (see coordination note above) — not a hard
  blocker either direction, but don't develop both in isolation without
  checking the other's current state first.
- **Blocks**: none directly, but is a prerequisite for any later
  hydrology-driven refinement of Swamp (task 127) — that task should use
  this one's score-based Swamp entry point, not reintroduce a separate gate.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/125-biome-score-classification.md)"$'\n\nExecute this task in the current project.'
```
