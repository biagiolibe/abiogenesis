# Task 110 — Biome enum + two-stage classification (areal biomes)

> **ID**: `110`
> **Category**: Feature (worldgen)
> **Priority**: 🟡 P2
> **Estimate**: ~4h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-biomes.md`, after a
> design-discussion pass that reconciled the doc with the current codebase)

---

## 🎯 Objective

`redesign/abiogenesis-biomes.md` proposes 16 biomes giving discrete, readable shape
to the environmental layer, replacing the current flat `TerrainKind` bands as the
primary visual/data classification. This task builds the **data layer** for the
"areal" biomes only — the ones that emerge from elevation + ambient scalars, not the
ones placed as explicit point/patch features (Cratere profondo, Distesa di cristalli,
Lago, Bocca vulcanica — task 111 — or Geyser — task 114, blocked).

Areal biomes in scope: Acqua profonda, Acqua bassa, Pianura, Collina, Montagna, Vetta,
Deserto, Tundra, Roccia nuda, Foresta, Palude.

No rendering here (task 112). No explicit feature placement here (task 111).

---

## 📋 Acceptance Criteria

- [ ] New `enum Biome` in `src/world.rs`, same style as `TerrainKind`
      (`world.rs:140-146`), covering the 11 areal biomes above (feature biomes are
      added by tasks 111/114, not here — the enum can grow, don't block on it).
- [ ] `Cell` gains a `biome: Biome` field, set once at generation time (mirrors how
      `terrain: TerrainKind` and `is_peak: bool` are set once and never recomputed
      per tick) — **not** derived live from `Cell.toxicity`/`temperature`/`light` at
      query time. This matters: `objectives.rs:318-320`'s comment on
      `species_present_in_zone` explicitly notes that `toxicity` diffuses every tick
      via `diffuse_environment` and "given enough ticks, stops meaning 'is in the
      zone' at all" — the same trap applies to biome membership, so `biome` must be
      a stable per-cell classification, not a threshold read at use time.
- [ ] **Stage A (reuse, do not rebuild):** `TerrainKind` (`world.rs:140-146`) +
      `is_peak` (`world.rs:165`) already give the base landform. Acqua
      profonda/bassa, Pianura, Collina, Montagna, Vetta map from these directly.
      **Known gap to resolve in this task:** `classify_elevation`
      (`world.rs:925-935`) discards the raw elevation value once it produces
      `TerrainKind::Sea` — no depth is kept on `Cell` today, so Acqua profonda vs.
      Acqua bassa cannot be split yet. Pick one: (a) keep the raw elevation float on
      `Cell` alongside `terrain`, or (b) derive depth from distance-to-nearest-land
      at generation time. Prefer (a) — it's the smaller change and mirrors how
      `is_peak` already keeps one extra bit of generation-time data around.
- [ ] **Stage B (new):** ambient scalars (`temperature`, `light`, `toxicity`) refine
      the Stage-A landform into a final biome, target values from the table in
      `redesign/abiogenesis-biomes.md`. Concretely:
  - `Plain` + high `toxicity` → Palude.
  - `Plain` + very high `temperature`/`light` → Deserto.
  - `Plain` + very low `temperature` → Tundra.
  - `Hill`/`Plain` at low organic viability (see doc's Roccia nuda rationale) → Roccia
    nuda.
  - Foresta and Palude need a **patch-level** decision, not a per-cell threshold —
    thresholding `temperature`/`light` per cell independently produces checkerboard
    noise, not organic patches. Reuse the low-frequency noise technique already used
    for `terrain_elevation` (`world.rs:842-887`, summed plane waves) to gate where
    Foresta/Palude *can* occur, then still require the underlying scalars to be in
    range — this is a pragmatic baseline, not a production-quality biome generator
    (explicitly out of scope per the design doc).
- [ ] New `BiomeConfig` in `src/config.rs`, same style/doc-comment density as
      `TerrainConfig` (`config.rs:511-603`) — every threshold named, no magic numbers.
      `assets/config/sim_config.ron` updated to match.
- [ ] Determinism test: same seed → same biome map, byte-for-byte (mirrors existing
      terrain determinism tests near `world.rs:997+`).
- [ ] Coverage test: over N fixed seeds, every areal biome in scope is reachable at
      least once (catches a threshold that's accidentally unreachable).
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test` passes.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:140-146` | `TerrainKind` — pattern to follow for the new `Biome` enum. |
| `src/world.rs:165` | `is_peak` — pattern for a second generation-time classification bit on `Cell`. |
| `src/world.rs:925-935` | `classify_elevation` — where the Sea depth gap lives. |
| `src/world.rs:842-887` | `terrain_elevation` / plane-wave noise — reusable technique for Stage B patch gating. |
| `src/config.rs:511-603` | `TerrainConfig` — pattern for the new `BiomeConfig`. |
| `redesign/abiogenesis-biomes.md` | Target values per biome (table), rendering/style constraints (not this task's concern but keep data consistent with it). |

---

## 🔗 Dependencies

- **Depends on**: none (elevation/`TerrainKind`/`is_peak` already exist).
- **Blocks**: 111, 112, 113.
