# Task 066 — Terrain field + procedural elevation generation

> **ID**: `066`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~4-6h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (from `redesign/abiogenesis-terrain-map.md`, discussed and expanded in chat: elevation becomes a real per-cell simulation dimension, not a decorative visual seed — a possible future factor in evolution alongside others TBD)

---

## 🎯 Objective

`redesign/abiogenesis-terrain-map.md` proposes discrete elevation bands for
the grid (flat colors, thin boundaries, sparse peak glyphs, toxic-zone
overlay), originally scoped as visual-only with terrain generation itself
explicitly listed as out of scope. That framing is now superseded: the user
wants elevation to be real per-cell simulation data — plains, hills,
mountains and sea, generated procedurally per world, forming organic
continents and islands — not an aesthetic value disconnected from the
simulation.

This task is the data/generation layer only: `Cell`/`SimWorld` gain a
terrain classification, generated deterministically at world construction.
No rendering, no placement gating — those are tasks 067 and 068.

**Sea is not a permanently-blocked void.** A future aquatic species is
planned, so this task must not hardcode "sea is unusable" into the data
model itself (e.g. don't give Sea cells a different `Cell` shape, don't
skip environmental simulation on them). Placement gating (067) is where
"nothing can live there today" is actually enforced, in one centralized
place — this task only classifies terrain and keeps the door open.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors; `cargo clippy -- -D warnings` is clean.
- [ ] `Cell` carries a terrain classification (e.g. `TerrainKind::{Sea, Plain, Hill, Mountain}` plus a peak flag) — implementer's call on exact shape, but peaks must be representable as a special unplaceable case *within* Mountain, not a 5th band.
- [ ] `Cell::default()` (and therefore `vec![Cell::default(); w*h]`) still produces the ordinary placeable band (e.g. `Plain`) — every existing test/call site building cells via `Default` keeps working unmodified.
- [ ] Terrain is generated deterministically per world at construction time (`SimWorld::new_for_world`), forming organic continents/islands rather than a uniform or purely random scatter.
- [ ] Terrain generation uses its own derived RNG stream (`StdRng::seed_from_u64(...)` derived from the world seed), never `world.rng_mut()` — existing draws (tag selection, species pool, in-tick reproduction) must not shift.
- [ ] A minimum placeable-land-fraction guarantee holds (Plain + Hill + non-peak Mountain; Sea does not count as placeable), with bounded resampling on the terrain RNG stream if a draw falls short. Threshold lives in `SimConfig`, no magic numbers.
- [ ] Peaks are decided once at generation time and stored per-cell — not re-derived later by a render-time or gameplay-time local-maximum scan.
- [ ] `ToxicZoneBounds` is extended to a rectangle placeable anywhere on the grid (not just anchored to the bottom-right corner), and the toxic zone's position/size is generated (from the terrain RNG stream or its own clearly-scoped derived stream) so that it always intersects enough placeable land for the `SurviveIn` objective to remain satisfiable — closing the unwinnable-world risk, not leaving it as a known gap (same defensive-generation spirit as tasks 047/048).
- [ ] Every existing call site/test assuming the old fixed bottom-right-anchored zone shape (`objectives.rs`, `worldgen.rs`, `world.rs`) is updated for the new geometry.
- [ ] `diffuse_environment` is untouched — every cell, including Sea, keeps simulating temperature/light/toxicity normally. No coastline masking.
- [ ] New tests: terrain generation is deterministic for a given seed; the placeable-land-fraction floor holds across a sample of seeds; the toxic zone always intersects enough placeable land across a sample of seeds.
- [ ] Existing determinism/balance tests still pass (baseline numbers may need updating since a new generation step and RNG stream are added at construction; their invariants/logic should not need rewriting).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `Cell`, `SimWorld`, `ToxicZoneBounds`, `apply_gradients` — terrain field and zone geometry live here. |
| `src/worldgen.rs` | `WorldParams`/difficulty curve — likely home for terrain generation parameters and the land-fraction threshold plumbing. |
| `src/config.rs` | `SimConfig` — new terrain/land-fraction thresholds go here, no magic numbers. |
| `src/objectives.rs` | `SurviveIn`/`ZoneKind::Toxic` — reads `world.toxic_zone.contains`, must keep working with the new variable-geometry zone. |
| `src/render.rs` | `background_field`/`BackgroundWave` (lines ~520-552) — closest existing precedent for a dependency-free deterministic noise field, though that one is decorative and lives in the render layer; this task's generation must live in `world.rs`/`worldgen.rs` instead. |

---

## 🧩 Technical Context

**Current state** (`src/world.rs`):
- `Cell { temperature, light, toxicity, organism, residue }` — no terrain field.
- `ToxicZoneBounds { x0, y0 }`, `contains(x, y) = x >= x0 && y >= y0` — always a rectangle anchored to the grid's bottom-right corner, sized by `WorldParams::toxic_zone_width/height`.
- `apply_gradients` lays down temperature (left→right)/light (top→bottom)/toxicity (fixed corner zone) once at construction, deterministically, without touching `world.rng`.
- The grid is always a full rectangle — every cell has valid environmental scalars; there is no existing concept of "outside the world."

**Desired state**: cells additionally carry a terrain classification, generated once per world from a derived RNG stream, forming organic land/sea shapes; the toxic zone's position and size vary per world and are chosen to guarantee overlap with placeable land.

---

## 🔨 Suggested Implementation

1. Design the terrain data shape on `Cell` (classification enum + peak flag, or an elevation scalar the classification is derived from — pick one primary representation).
2. Add terrain-generation config (band thresholds, land-fraction floor, resample bound) to `SimConfig`, following existing `SimConfig` module conventions.
3. Implement deterministic noise-based terrain generation in `world.rs` or `worldgen.rs`, using a `StdRng` derived from the world seed (separate from `world.rng`).
4. Wire it into `SimWorld::new_for_world`, after or alongside `apply_gradients`.
5. Add the placeable-land-fraction check with bounded resampling.
6. Extend `ToxicZoneBounds` to a positionable rectangle; update `apply_gradients`'s toxic zone placement to depend on the generated terrain and guarantee land overlap.
7. Update every call site currently assuming the fixed-corner zone shape.
8. Write the new tests listed in Acceptance Criteria; run the full existing suite and update baseline numbers where determinism naturally shifts.

---

## ⚠️ Constraints and Caveats

- **Determinism**: `sim`/`world`/`config` must not depend on `bevy::render`/`bevy_egui` (TECH_DESIGN §5); no `rand::rng()`, no `HashMap` iteration, no parallel queries.
- **No magic numbers**: every new coefficient goes in `SimConfig`.
- **Don't touch diffusion**: this task changes what a cell *is*, not how temperature/light/toxicity propagate.
- **Don't gate placement here**: Sea/peak unplaceability is task 067's job, via a single centralized check — this task only classifies terrain.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: 067 (placement gating), 068 (rendering).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/066-terrain-field-procedural-elevation-generation.md)"$'\n\nExecute this task in the current project.'
```
