# Task 108 — Fourth metabolism: chemolithotroph, gain from toxicity

> **ID**: `108`
> **Category**: Feature / Simulation
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-evolution-xenotypes.md`,
> GDD §5.4's already-open item)

---

## 🎯 Objective

Add a fourth `Metabolism` variant, `Chemolithotroph`, whose metabolic gain
derives from `Cell.toxicity` the way `Photolithic`'s derives from `light` —
GDD §5.4's already-named, deferred item ("*Others can be added — e.g., a
chemolithotroph tied to toxicity — as unlockable content*"), and GDD §5.2's
current-status note that `toxicity` is declared and rendered but **not read
anywhere in the tick loop** today.

Scoped as a **standalone, near-term, authored-content task**: this is a
normal, always-available starting metabolism, added the same way
`Predator`/`Decomposer` already exist alongside `Photolithic` — not an
evolution outcome. The source doc also floats a chemolithotroph as a
*possible evolution outcome* (a lineage repeatedly exposed to toxicity
evolving into one) for later, once tasks 106/107 exist, but that linkage is
explicitly **not** required here — this task must stand on its own,
independent of 106/107, and ship regardless of whether/when evolution is
built.

---

## 📋 Acceptance Criteria

- [ ] `Metabolism` (`src/world.rs:19-23`) gains a `Chemolithotroph` variant.
- [ ] A new gain formula in `step()`'s metabolism match (`src/sim.rs:259-263`)
      following the existing pattern exactly: `Metabolism::Chemolithotroph =>
      cell.toxicity * energy.chemolithotroph_metabolism_gain * fit` — `fit`
      (environmental/temperature fitness, `env_fit`) still applies the same
      way it does for every other metabolism; only the gain source scalar
      changes (`toxicity` instead of `light`).
- [ ] New `EnergyConfig` fields (`src/config.rs`, alongside
      `photolithic_metabolism_gain`/`predator_drain_cap`/etc., 263-272, with
      `Default` values added to the impl at 297-318):
      `chemolithotroph_metabolism_gain` and `chemolithotroph_upkeep`,
      analogous to the existing per-metabolism gain/upkeep pairs, with
      `Default` values chosen to be roughly balance-comparable to the other
      three (first-pass, tunable) — no magic numbers.
- [ ] Upkeep match arm added (`src/sim.rs:320-324`):
      `Metabolism::Chemolithotroph => energy.chemolithotroph_upkeep`.
- [ ] Every exhaustive `match` over `Metabolism` gets a `Chemolithotroph`
      arm: the glyph match (`src/render.rs:63-65`), the second render match
      (`src/render.rs:1153-1155`), and the description text match
      (`src/text.rs:478-480`). `cargo clippy -- -D warnings` will additionally
      catch any other exhaustive match missing an arm.
- [ ] `add_bonus_species` (`src/worldgen.rs:191-`, its current 50/50
      `Predator`/`Decomposer` coin flip at 204-208) becomes a 3-way draw
      including `Chemolithotroph`, so it's reachable via `Seed` in normal
      play — the same way `Predator`/`Decomposer` are today (available in
      the pool, not pre-placed; the player chooses where to seed it, same as
      the existing two, which also aren't guaranteed viable at their chosen
      spot without prey/residue nearby — a chemolithotroph needs
      `toxicity > 0` nearby by the same logic, the player's placement
      responsibility, not worldgen's). `generate_starting_palette`
      (`src/worldgen.rs:156-181`) stays `Photolithic`-only, unchanged — its
      doc comment already explains why (only metabolism self-sustaining with
      nothing pre-existing on the grid).
- [ ] `WorldgenConfig`'s doc comment (`src/config.rs:439-443`) is updated to
      reflect that `add_bonus_species` now draws a third metabolism, since
      its current wording implies only two.
- [ ] Unit tests in `src/sim.rs`'s `#[cfg(test)]` module, mirroring
      `world_with_one_predator`/`world_with_one_decomposer`
      (`src/sim.rs:976-1003`, `1133-1160`) with an equivalent
      `world_with_one_chemolithotroph` fixture: isolated chemolithotroph in
      a zero-toxicity cell behaves like a dark photolithic (no gain, upkeep
      drains it); a chemolithotroph in a high-toxicity cell nets positive
      energy; gain scales with `fit` the same way the other three do.
- [ ] Balance-test coverage in `tests/balance.rs` mirroring the existing
      photolithic-bloom-style coverage (`population_rarely_reaches_total_extinction_across_seeds`
      etc., 131-224) to the extent a chemolithotroph population is included
      in a nominal scenario — at minimum, confirm a world seeded with
      chemolithotrophs in a toxic zone doesn't crash or trivially collapse
      across a seed spread; does not need full parity with every existing
      balance test if the fixture doesn't naturally support it.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: seed a chemolithotroph species into the
      toxic zone and confirm it gains energy there (survives/grows) rather
      than behaving like a starved photolithic in the dark.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `Metabolism` enum (19-23) — add the variant. |
| `src/sim.rs` | Gain match (259-263), upkeep match (320-324), existing `world_with_one_predator`/`world_with_one_decomposer` test fixtures (976, 1133) to mirror. |
| `src/config.rs` | `EnergyConfig` gain/upkeep field cluster (263-272) + its `Default` impl (297-318) — add `chemolithotroph_metabolism_gain`/`chemolithotroph_upkeep`. `WorldgenConfig`'s doc comment (439-443, "Always generated as `Metabolism::Photolithic`... the only metabolism that's self-sustaining") needs updating once a chemolithotroph is added to `add_bonus_species`'s draw pool — see below. |
| `abiogenesis-gdd.md` | §5.2 (94, toxicity's current "not read anywhere" status — this task changes that), §5.4 (110-116, the deferred item this closes). |
| `tests/balance.rs` | Existing bloom/extinction/saturation tests (131-224) — pattern for any added chemolithotroph-inclusive scenario coverage. |
| `src/worldgen.rs` | `add_bonus_species` (191-, coin-flip between `Metabolism::Predator`/`Metabolism::Decomposer` at line 204-208) — the actual authoring site (task 013's starting-species palette): extend the draw to a 3-way pick including `Chemolithotroph`. `generate_starting_palette` (156-181) stays `Photolithic`-only per its existing doc comment, unchanged. |
| `src/render.rs` | Metabolism glyph match (63-65) and another match (1153-1155) — both need a `Chemolithotroph` arm. |
| `src/text.rs` | `species_description`'s metabolism-text match (478-480) — needs a `Chemolithotroph` arm ("draws its energy from environmental toxicity" or similar). |

---

## 🧩 Technical Context

- **Current behavior**: `Metabolism` has exactly three variants
  (`Photolithic`, `Predator`, `Decomposer`); `toxicity` is a per-cell scalar
  (`Cell.toxicity`, `src/world.rs:153`) that's generated, diffused
  (`diffuse_environment`), and rendered (`toxicity_tint`), but never read by
  `step()`'s gain/cost computation — GDD §5.2 states this explicitly as the
  current baseline.
- **Desired behavior**: a fourth metabolism reads `toxicity` as its gain
  source, following exactly the shape `Photolithic` already establishes for
  `light` — same `fit` multiplier, same upkeep-match-arm pattern, same
  config-driven coefficients.
- Precedent for the pattern to copy: `Metabolism::Photolithic => cell.light *
  energy.photolithic_metabolism_gain * fit` (`src/sim.rs:260`) and its
  `energy.base_upkeep` upkeep arm (`src/sim.rs:321`) — no shared-resource
  pre-pass is needed (unlike `Predator`/`Decomposer`, which draw from
  neighbours/residue via a pre-pass, `src/sim.rs:160-235`); `toxicity` is a
  per-cell scalar exactly like `light`, read directly at the point of use,
  no pre-pass required.

---

## 🔨 Suggested Implementation

1. Add `Chemolithotroph` to `Metabolism` (`src/world.rs:19-23`).
2. Add `chemolithotroph_metabolism_gain`/`chemolithotroph_upkeep` to
   `EnergyConfig` (fields near `src/config.rs:263-272`, `Default` values in
   the impl at `src/config.rs:297-318`).
3. Add the gain match arm (`src/sim.rs:259-263`) and upkeep match arm
   (`src/sim.rs:320-324`), following `Photolithic`'s shape exactly (no
   pre-pass).
4. Add the new match arm to `src/render.rs:63-65`, `src/render.rs:1153-1155`,
   `src/text.rs:478-480`.
5. Extend `add_bonus_species`'s coin flip (`src/worldgen.rs:204-208`) to a
   3-way draw including `Chemolithotroph`; update `WorldgenConfig`'s doc
   comment (`src/config.rs:439-443`) accordingly.
6. Unit tests mirroring `world_with_one_predator`/`world_with_one_decomposer`.
7. Balance-test coverage as scoped in Acceptance Criteria.
8. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- **No dependency on tasks 106/107** — do not gate this behind evolution
  existing, and do not implement it as an evolution-only outcome. It must be
  a normal, always-available starting metabolism from day one of this task
  landing, exactly like the existing three.
- `fit` (temperature fitness) still applies to the new metabolism — do not
  skip `env_fit` for it; the source doc's "toxicity tied" framing is about
  the gain source, not an exemption from temperature fitness.
- No magic numbers — all new coefficients live in `EnergyConfig`.
- Keep the tick deterministic; no pre-pass/shared-resource-drain logic is
  needed for this metabolism (toxicity is per-cell, not shared/depleting the
  way prey energy or residue is), so do not copy `Predator`/`Decomposer`'s
  pre-pass machinery unnecessarily.

---

## 🔗 Dependencies

- **Depends on**: 072 (toxic zone / `toxicity` generation), the base tick
  loop (`sim.rs` step formula, GDD §5.6).
- **Not dependent on**: 106, 107 (evolution) — see Objective; this task must
  stand alone.
- **Blocks**: none. (A later evolution-outcome linkage — a lineage evolving
  into a chemolithotroph via task 107 — could reuse this variant once both
  exist, but is not scoped here.)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/108-chemolithotroph-metabolism.md)"$'\n\nExecute this task in the current project.'
```
