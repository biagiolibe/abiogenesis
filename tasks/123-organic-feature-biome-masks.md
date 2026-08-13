# Task 123 — Organic masks for placed feature biomes

> **ID**: `123`
> **Category**: Feature (worldgen)
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-13 (Phase 1 of the worldgen pipeline reassessment
> scoped from `redesign/procedural_biome_generation_spec_v2.md` — see that
> doc's §13.2 for the organic-mask technique and §1.7 for the diagnosis
> this task addresses).

---

## 🎯 Objective

Cratere profondo, Distesa di cristalli, and Lago (task 111) are placed as
axis-aligned rectangles via `SimWorld::place_feature_rect`. On a 128×80 grid
these read as visibly artificial — the most obviously non-organic shapes
currently on the map. Replace the rectangle footprint with a deformed-disk
mask (a circle whose radius varies by angle via a small deterministic noise
sum), the technique `procedural_biome_generation_spec_v2.md` §13.2
describes. Keep every other part of the placement pipeline (bounded-retry
search, zero-overlap-with-`reserved` requirement, best-seen fallback,
target scalar imposition) unchanged — only the footprint shape changes.

**Explicitly out of scope: `toxic_zone`.** `ZoneKind::Toxic` in
`objectives.rs:376` checks `world.toxic_zone.contains(x, y)` — a rectangle
containment test that `SurviveIn` depends on for exact, stable geometry
(`ToxicZoneBounds`'s own doc comment explains why: the diffusing
`Cell.toxicity` scalar isn't a reliable proxy for "in the zone" over a
long-running world, so objectives read the fixed rectangle instead). Giving
the toxic zone an organic shape would need `ToxicZoneBounds` (or its
`contains` check) to represent that shape too, not just its footprint —
a separate, riskier change than this task's scope. Track it as a follow-up
if wanted, not folded in here.

---

## 📋 Acceptance Criteria

- [ ] New shared helper (e.g. `organic_disk_contains(dx, dy, base_radius,
      angular_waves) -> bool`, or equivalent) computing whether a cell
      offset from a feature's center falls inside a deformed disk: radius
      at a given angle is `base_radius * (1.0 + angular_distortion(angle))`,
      where `angular_distortion` sums a handful of sine terms over angle
      (same dependency-free technique `terrain_waves`/`wave_band_sum`
      already use for elevation and forest/swamp patches — reuse that
      pattern in angle-space rather than introducing a noise crate).
- [ ] `place_feature_rect` → `place_feature_organic` (or equivalent): same
      bounded-retry loop over candidate **center** positions, but
      `placeable_fraction_in`/`reserved_overlap_in`'s rectangle scan is
      replaced by a scan over the mask's bounding box, testing membership
      per cell instead of assuming every cell in the box is included.
      `set_feature_biome` only writes cells that pass the mask test.
- [ ] `FeaturePlacement` drops `width`/`height` in favor of a `radius`
      (plus whatever distortion-amplitude/frequency fields are needed — see
      below on whether those are per-feature or shared).
- [ ] `BiomeConfig`: `crater_width`/`crater_height` →
      `crater_radius`; same for `crystal_field_*` and `lake_*`. Add
      distortion amplitude/frequency fields — shared across the three
      features (e.g. `feature_mask_distortion`, `feature_mask_frequency`)
      unless a per-feature knob is clearly needed; don't add three copies
      of the same tuning constant without a reason. Mirror every renamed/
      added field in `assets/config/sim_config.ron`.
- [ ] Each feature's own RNG stream (`CRATER_SEED_OFFSET`,
      `CRYSTAL_FIELD_SEED_OFFSET`, `LAKE_SEED_OFFSET`) now also draws the
      per-placement angular-wave parameters (direction/phase per term),
      the same way `generate_terrain`'s stream draws its wave sets — no new
      seed offsets needed, the existing three streams just do more work.
- [ ] `feature_biomes_never_overlap_each_other` and
      `feature_biome_placement_is_deterministic_for_a_given_seed`
      (`world.rs:2117`, `world.rs:2195`) updated to the new shape
      (membership test instead of rectangle bounds) and still pass.
- [ ] `toxic_zone_matches_its_own_bounds` (`world.rs:1999`) still passes
      unmodified in its toxic-zone assertions — it also asserts Crater/
      CrystalField/Lake's imposed `toxicity` values, which must still hold
      for whichever cells the new mask actually covers.
- [ ] Visual check: run the game, confirm Crater/CrystalField/Lake read as
      rounded, irregular blobs rather than rectangles, at a sample of
      several seeds (a single seed could accidentally look round by luck of
      the distortion draw).
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:837-994` | `place_feature_biomes`, `place_feature_rect`, `reserved_overlap_in`, `set_feature_biome`, `FeaturePlacement` — the rectangle machinery to replace. |
| `src/world.rs:1508-1530` | `terrain_waves`/`wave_band_sum` — the existing dependency-free noise-sum pattern to adapt to angle-space. |
| `src/config.rs:793-876` | `crater_*`/`crystal_field_*`/`lake_*` fields in `BiomeConfig` — rename/extend here. |
| `assets/config/sim_config.ron` | Mirror every config change here (task 111 did the same). |
| `src/world.rs:2117, 2152, 2195` | Existing feature-biome tests to update. |
| `src/objectives.rs:376` | `ZoneKind::Toxic` — confirms why `toxic_zone` stays out of scope (do not touch this file in this task). |

---

## 🧩 Technical Context

- **Current behavior**: `place_feature_rect` searches random top-left
  corners for a `width × height` axis-aligned rectangle, accepting the
  first candidate with zero overlap against `reserved` and a placeable
  fraction above a floor; `set_feature_biome` then writes every cell in
  that rectangle.
- **Desired behavior**: same search loop, but over candidate **centers**,
  and the accept/write logic tests disk-with-angular-distortion membership
  per cell instead of "is this cell inside the rectangle." The `reserved`
  zero-overlap and placeable-fraction-floor semantics stay conceptually
  identical, just measured over the new footprint shape.

---

## 🔨 Suggested Implementation

1. Add an angle-space noise helper alongside `terrain_waves`/
   `wave_band_sum` — same struct-of-random-terms-summed-and-averaged shape,
   parameterized on angle instead of `(nx, ny)`.
2. Write `organic_disk_contains` (or fold the check inline) using that
   helper.
3. Change `FeaturePlacement` and the three call sites in
   `place_feature_biomes` to pass a radius instead of width/height.
4. Rewrite `place_feature_rect` → `place_feature_organic`: candidate center
   search unchanged in structure; `placeable_fraction_in`/
   `reserved_overlap_in` calls replaced by mask-aware equivalents scanning
   the disk's bounding box (`center ± radius*(1+max_distortion)`, clamped
   to grid bounds).
5. Update `set_feature_biome` to only write cells passing the mask test,
   and to mark only those cells `reserved`.
6. Update `BiomeConfig`/`sim_config.ron`, then the two affected tests.
7. Run the game at a few seeds to eyeball the shapes before considering
   this done — this is a visual-quality task, a passing test suite alone
   doesn't confirm the shapes read as organic.

---

## ⚠️ Constraints and Caveats

- **Determinism**: every new random draw must come from the feature's own
  existing seed offset stream, never `self.rng` or another stage's stream
  — same discipline as `generate_terrain`/`classify_biomes`.
- **No magic numbers**: distortion amplitude/frequency live in
  `BiomeConfig`, not inline constants.
- Don't touch `toxic_zone`/`place_toxic_zone`/`ToxicZoneBounds` — see
  Objective section for why.
- This is a shape-only change: target `temperature`/`light`/`toxicity`
  imposition, the `reserved`-overlap guarantee, and the bounded-retry
  fallback behavior must all still hold, just measured against the new
  footprint.

---

## 🔗 Dependencies

- **Depends on**: 111 (feature biome placement — this task replaces its
  rectangle mechanism, doesn't add a new one from scratch).
- **Blocks**: none. A possible future follow-up (not scoped here) is
  extending the organic-shape treatment to `toxic_zone` itself, which would
  additionally need `objectives.rs`'s `SurviveIn`/`ZoneKind::Toxic` check
  reworked to use real membership instead of rectangle containment.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/123-organic-feature-biome-masks.md)"$'\n\nExecute this task in the current project.'
```
