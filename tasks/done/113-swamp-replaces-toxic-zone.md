# Task 113 — Palude replaces `toxic_zone`

> **ID**: `113`
> **Category**: Refactor (worldgen + objectives)
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: done
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-biomes.md`);
> implemented 2026-08-19.

---

## 🎯 Objective

`redesign/abiogenesis-biomes.md` proposes the Palude biome as a generalization of the
existing isolated `toxic_zone` rectangle — diffuse toxicity over a whole biome instead
of one ad-hoc rectangle. This task removes `toxic_zone` and re-points everything that
depended on it (most importantly the `SurviveIn`/`ZoneKind::Toxic` objective) at the
Palude biome from task 110.

**This is more than a cosmetic swap.** `ZoneKind::Toxic` (`objectives.rs:320-336`) is
a real game objective, checked as `world.toxic_zone.contains(x, y)` — a fixed
rectangle with bounds. Palude can be multi-patch and irregularly shaped (task 110's
Stage B uses patch-gated noise, not a rectangle), so the check becomes cell-membership
in a biome, not containment in a rectangle.

**⚠️ Hard prerequisite, found 2026-08-13 during a dependency review with task
122: `place_toxic_zone` is currently the *only* generation-time source of
nonzero `Cell.toxicity` anywhere in the game** (`swamp_toxicity_min`'s own
doc comment in `config.rs` says so explicitly). Removing it without a
replacement would leave every cell's `toxicity` at `0.0` forever, which
would silently:
- break task 108's `Chemolithotroph` metabolism (reads `toxicity` directly
  for its energy gain — nothing left to read);
- make task 122 (toxic-zone reinjection) irrelevant for the wrong reason
  (nothing to reinject, not because the erosion problem got solved).

**Task 125 (score-based biome classification) is the task that provides the
replacement** — it moves Swamp's classification off `toxicity` entirely
(fixing the causality complaint that motivated this task in the first
place) and, as part of its own scope, imposes `toxicity` on a sub-region of
the classified Swamp cells as a post-classification modifier. **This task
must not land before 125.** Confirm 125 has shipped, and that some `Swamp`
cells get a real generation-time `toxicity` value, before removing
`place_toxic_zone` here.

---

## 📋 Acceptance Criteria

- [x] Before removing anything: confirm task 125 has landed and that its
      post-classification toxicity step actually gives some `Swamp` cells
      `toxicity > 0.0` at generation time (its own test covers this — check
      it passes on `main` first). If 125 hasn't landed, stop and pick that
      up first; this task is blocked on it, not merely related to it.
- [x] `toxic_zone: ToxicZoneBounds` field, `ToxicZoneBounds` struct, and
      `place_toxic_zone` (`world.rs:345-403`) removed from `SimWorld`.
      `EnvironmentConfig`'s `toxic_zone_*` fields and `TerrainConfig`'s
      `min_toxic_zone_placeable_fraction`/`max_toxic_zone_placement_attempts`
      (`config.rs:141-146, 572-580`) removed or repurposed for Palude's own placement
      config from task 110, not left dead.
- [x] After removal, re-run task 125's "some Swamp cells have `toxicity >
      0.0`" test (and task 108's chemolithotroph balance test in
      `tests/balance.rs`) — both must still pass. If they don't, 125's
      toxicity-imposition step wasn't actually independent of
      `place_toxic_zone` and needs fixing before this task can proceed.
- [x] `cell_in_zone` (`objectives.rs:333-335`) rewritten: `ZoneKind::Toxic =>` checks
      `world.get(x, y).biome == Biome::Swamp` (or equivalent), not a rectangle
      `.contains`.
- [x] The comment on `species_present_in_zone` (`objectives.rs:318-320`) — explaining
      *why* the check uses fixed geometry instead of live `Cell.toxicity` (diffusion
      erodes the meaning of "is in the zone" over time) — updated to explain the new
      invariant instead: `Cell.biome` is a stable per-cell classification set at
      generation time (task 110), not recomputed from `toxicity` each tick, so it
      doesn't have the same erosion problem. Don't lose this reasoning, it's the
      reason the check is correct.
- [x] Every test that constructs `ToxicZoneBounds` by hand
      (`objectives.rs:482, 591-593, 623, 627, 676-683, 710` and any others found by
      `grep -n ToxicZoneBounds src/*.rs`) rewritten to set `Cell.biome = Biome::Swamp`
      on the relevant cells instead.
- [x] `draw_toxic_zone` (`render.rs:582+`) removed — coordinate with task 112 so it's
      deleted exactly once (whichever of 112/113 lands second does the deletion; the
      other leaves a one-line note instead of duplicating the change).
- [x] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test` passes,
      including the `SurviveIn`/toxic-zone objective tests under their new Palude-based
      setup.

---

## ✅ Implementation notes (2026-08-19)

- **Removed**: `ToxicZoneBounds` struct, `SimWorld::toxic_zone` field,
  `place_toxic_zone`/`placeable_fraction_in`/`set_toxic_zone` methods,
  `TOXIC_ZONE_SEED_OFFSET`, `render.rs::draw_toxic_zone`/`draw_dashed_line`
  and their color/dash constants, `EnvironmentConfig::toxic_zone_width/
  height`, `DifficultyConfig::toxic_zone_width_late/height_late`,
  `TerrainConfig::min_toxic_zone_placeable_fraction/
  max_toxic_zone_placement_attempts`, and `WorldParams::toxic_zone_width/
  height`. None of these were repurposed — Swamp's placement is score-based
  (task 125), not a bounded-retry rectangle search, so there was no
  equivalent config to move the values into.
- **Renamed**: `EnvironmentConfig::toxic_zone_value` →
  `swamp_toxicity_value` (same 0.7 default) — it was already Swamp's own
  imposed toxicity value in practice since task 125, just under the old
  name.
- **`cell_in_zone`** (`objectives.rs`): `ZoneKind::Toxic` now checks
  `world.get(x, y).biome == Biome::Swamp` directly.
- **`generate_one_objective`** (`worldgen.rs`): `has_toxic_zone` (from
  `params.toxic_zone_width/height`) replaced with `has_swamp`, computed by
  scanning `world.cells` for any `Biome::Swamp` — this can no longer be a
  pure function of `WorldParams` alone, since Swamp's existence isn't
  guaranteed by construction the way the old rectangle's nonzero size was.
  A 50-seed sample confirmed `SurviveIn` is still offered ~34% of the time
  at `world_index == 0` (new test:
  `survive_in_toxic_zone_is_offered_across_a_real_fraction_of_seeds`) — not
  a regression, just no longer a guarantee.
- **Bug found and fixed in the same pass**: `place_feature_biomes`'s
  VolcanicVent override set `Cell::biome` but never touched `Cell::toxicity`
  — a cell `classify_biomes` had already marked toxic Swamp could fall
  inside a heat source's vent radius and keep its stale 0.7 toxicity under
  a biome that claims "not toxic." Pre-existing (the old `place_toxic_zone`
  ran before `place_feature_biomes` too, so the same gap existed with the
  rectangle), only surfaced now because closing this task meant broadening
  the toxicity-consistency test from one seed to ten. Fixed by resetting
  `toxicity = 0.0` alongside the biome override.
- **Balance test fixed**: `tests/balance.rs`'s chemolithotroph survival
  test used to seed directly into the guaranteed-large 21x15 rectangle; its
  replacement toxic-cell search initially picked cells by "most toxic
  neighbours" alone, which — non-obviously — almost always selected a
  `Lake` cell (`lake_toxicity` 0.05, a flavor value) over a genuinely toxic
  Swamp/Crater cell, since Lake is often the single largest *solid* toxic
  blob on the grid even though its toxicity value is negligible. Collapse
  rate spiked to 60-70% (well past the 30% budget) until the search was
  fixed to sort on toxicity value first, embeddedness only as a tie-break.
  Diagnosed empirically (`cargo run --example`, discarded scratch binary)
  rather than guessed.
- **Not done, filed as task 133 instead**: two gaps the original task scope
  didn't anticipate — `SurviveIn`'s target region has no visual/textual
  affordance now that the dashed-outline rectangle is gone (no biome
  legend/tooltip exists either), and GDD §9's "larger toxic zones"
  difficulty axis lost its implementation with nothing replacing it. Both
  are real open decisions, not correctness bugs, so left for that task
  rather than resolved unilaterally here.
- Docs synced: `abiogenesis-gdd.md` §5.2/§5.9/§5.10/§8/§9,
  `player_guide.md`'s objective list.
- `cargo clippy --all-targets -- -D warnings` / `cargo fmt` / `cargo test`
  all clean; `cargo run` launched cleanly against the updated
  `sim_config.ron`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:345-403` | `place_toxic_zone`, `ToxicZoneBounds` — removed. |
| `src/objectives.rs:318-336` | `species_present_in_zone`, `cell_in_zone`, `ZoneKind::Toxic` — rewritten to check biome membership. |
| `src/objectives.rs:482, 591-593, 623, 627, 676-683, 710` | Tests constructing `ToxicZoneBounds` by hand — rewritten. |
| `src/config.rs:141-146, 572-580` | `toxic_zone_*` config fields — removed/repurposed. |
| `src/render.rs:582+` | `draw_toxic_zone` — removed (coordinate with 112). |

---

## 🔗 Dependencies

- **Depends on**: 110 (Palude must exist as a `Biome` variant, stably assigned on
  `Cell.biome`); **125, hard blocker** (found 2026-08-13) — 125 is what
  gives Swamp a `toxicity` source independent of `place_toxic_zone`. Do not
  start this task before 125 has shipped and its toxicity-imposition test
  passes on `main`.
- **Blocks**: 122 (toxic-zone reinjection) — 122 currently targets
  `world.toxic_zone`, which this task removes; 122 must be rewritten to
  target `Biome::Swamp` membership instead, which only makes sense once
  this task has landed. See 122's own file for its updated scope.
