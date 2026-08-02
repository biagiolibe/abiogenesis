# Task 002 — `SimConfig`: centralized coefficients

> **ID**: `002`
> **Category**: Architecture
> **Priority**: 🔴 P1
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Transcribe the entire numeric baseline from GDD §5.9 into a **single `SimConfig` resource**, the single source of truth for every simulation coefficient.

Necessary because GDD §5.6 mandates it as a design decision: *"all coefficients are named constants in a single place, so final tuning is fast."* Balance tuning is declared the most delicate work in the project (GDD §13, §14): if the numbers end up scattered through the code, that work becomes impractical.

**Coefficients for later phases are included too** (matrix, notebook, actions): they're already decided in the GDD, and centralizing them now costs nothing.

---

## 📋 Acceptance Criteria

- [ ] Every numeric value in the GDD §5.9 tables has a corresponding constant or field in `SimConfig`.
- [ ] `SimConfig` is registered as a `Resource` by `ConfigPlugin`.
- [ ] `SimConfig::default()` exists and returns the GDD baseline.
- [ ] No duplicated values: each number appears exactly once.
- [ ] Every field has a comment with its unit of measurement or GDD reference.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/config.rs` | `SimConfig` and `ConfigPlugin` |

---

## 🧩 Technical Context

- **Current behavior**: `src/config.rs` is an empty stub (task 001).
- **Desired behavior**: `SimConfig` available as a resource, with the entire GDD baseline.

### The baseline to transcribe (GDD §5.9)

**Environment** — all scalars are in `[0,1]`

| Constant | Value |
|---|---|
| Environmental diffusion (Phase 1+) | `0.05` / tick |
| Light gradient: high → low | `0.9` → `0.2` |
| Temperature gradient: left → right | `0.2` → `0.8` |
| Toxic zone | `0.7` (elsewhere `0.0`) |

**Time and actions**

| Constant | Value |
|---|---|
| `ERA_TICKS` | `25` |
| Era budget / world | `40` (early) → `25` (late) |
| Point budget / era | `3` |
| Action costs | seed `1`, stress `1`, cull `1`, splice `2` |

**Energy and metabolism** (per organism)

| Constant | Value |
|---|---|
| Energy at seeding | `5.0` |
| Base `upkeep` | `0.5` / tick |
| `crowd_factor` | `0.15` / occupied neighbor |
| `repro_threshold` | `10.0` |
| `repro_cost` (to the child) | `5.0` |
| Photolithic `metabolism_gain` | `2.0` |
| Predator `drain_cap` / `upkeep` | `2.0` / tick · `0.7` |
| Decomposer `extract_rate` / `upkeep` | `1.5` / tick · `0.5` |
| Residue on death / decay | `3.0` · `0.2` / tick |
| Default `temp_tolerance` (σ) | `0.15` |

**Tags and matrix**

| Constant | Value |
|---|---|
| Global tag pool | `10` |
| Active tags / world | `5` (early) → `8` (late) |
| Tags per species | `1–3` |
| Effect intensity / adjacency | integers in `{−2,−1,0,+1,+2}` |
| Matrix density | ~`40%` non-zero pairs |

**Notebook**

| Constant | Value |
|---|---|
| Confirmation threshold / cell | `3.0` cumulative evidence |
| Weight of one observation | `1 / (1 + n_adjacent_confounders)` |

**Grid**

| Constant | Value |
|---|---|
| Size | `48×32` |
| Neighborhood | Moore (8) |

---

## 🔨 Suggested Implementation

1. Group into themed sub-structs instead of a single flat struct with 30 fields — it reads better and mirrors the GDD tables:

   ```rust
   #[derive(Resource, Debug, Clone)]
   pub struct SimConfig {
       pub grid: GridConfig,
       pub environment: EnvironmentConfig,
       pub time: TimeConfig,
       pub energy: EnergyConfig,
       pub tags: TagConfig,
       pub notebook: NotebookConfig,
   }
   ```

2. Each sub-struct implements `Default` with the GDD values:

   ```rust
   #[derive(Debug, Clone)]
   pub struct EnergyConfig {
       /// Energy an organism starts with when seeded.
       pub seed_energy: f32,
       /// Base maintenance cost per tick.
       pub upkeep: f32,
       /// Carrying capacity: penalty per occupied neighbour (GDD 5.9).
       pub crowd_factor: f32,
       // ...
   }
   ```

3. `ConfigPlugin` inserts the resource:

   ```rust
   impl Plugin for ConfigPlugin {
       fn build(&self, app: &mut App) {
           app.init_resource::<SimConfig>();
       }
   }
   ```

4. Use `f32` for continuous scalars and `u32`/`i8` for counts and matrix intensities.

---

## ⚠️ Constraints and Caveats

- **No value should be "rounded" or reinterpreted.** GDD §14 is explicit: the §5.9 numbers *"need to be confirmed or adjusted through playtesting, not reinvented."*
- `src/config.rs` **must not depend on `bevy::render` or `bevy_egui`** (invariant 2, `TECH_DESIGN.md` §5). The only Bevy import allowed is the one needed for `derive(Resource)`.
- Budgets expressed in the GDD as a range (eras per world `40 → 25`, active tags `5 → 8`) should be modeled as a start/end pair, not a single value: the difficulty curve in Phase 3 will need them.
- Do not implement hot-reload: it's planned (`TECH_DESIGN.md` §4) but belongs to the tuning phase. Here it's enough for the config to be **a single resource, read and never duplicated**, so the migration is painless.

---

## 🔗 Dependencies

- **Depends on**: 001
- **Blocks**: 003

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/002-sim-config.md)"$'\n\nExecute this task in the current project.'
```
