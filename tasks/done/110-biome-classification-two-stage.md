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

- [x] New `enum Biome` in `src/world.rs`, same style as `TerrainKind`
      (`world.rs:140-146`), covering the 11 areal biomes above (feature biomes are
      added by tasks 111/114, not here — the enum can grow, don't block on it).
      English identifiers (`DeepWater`, `ShallowWater`, `Plain`, `Hill`, `Mountain`,
      `Peak`, `Desert`, `Tundra`, `BareRock`, `Forest`, `Swamp`), mirroring how
      `TerrainKind` variants are English with Italian display strings left to
      `text.rs` (not needed until rendering, task 112).
- [x] `Cell` gains a `biome: Biome` field, set once at generation time (mirrors how
      `terrain: TerrainKind` and `is_peak: bool` are set once and never recomputed
      per tick) — **not** derived live from `Cell.toxicity`/`temperature`/`light` at
      query time.
- [x] **Stage A (reuse, do not rebuild):** `TerrainKind` + `is_peak` give the base
      landform. `Cell` also gains a raw `elevation: f32` field (option (a) from the
      task brief — smaller change, mirrors `is_peak`), letting `TerrainKind::Sea`
      split into Acqua profonda/bassa by `BiomeConfig::deep_water_elevation_max`.
- [x] **Stage B:** `SimWorld::classify_biomes` (new generation step, runs last in
      `new_for_world` — after `place_toxic_zone`, so it reads real generation-time
      `toxicity`, not the all-zero pre-placement value) refines Stage A per the
      table. Foresta/Palude each get their own low-frequency patch mask (reusing
      `terrain_waves`/`wave_band_sum`, own `BIOME_SEED_OFFSET` RNG stream) gating
      where they can occur, ANDed with a scalar-range check.
- [x] New `BiomeConfig` in `src/config.rs`, mirroring `TerrainConfig`'s style;
      `assets/config/sim_config.ron` updated to match (verified via `cargo run` —
      no RON deserialization error on load).
- [x] Determinism test: `biome_classification_is_deterministic_for_a_given_seed`.
- [x] Coverage test: `every_areal_biome_is_reachable_across_seeds` (union over 40
      seeds, since some biomes like Peak/Swamp are sparse per individual seed).
- [x] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test` passes (150
      lib tests + full integration suite).

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
