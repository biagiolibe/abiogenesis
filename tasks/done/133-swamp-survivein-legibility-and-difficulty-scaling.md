# Task 133 — [DECISION] Two open gaps left by task 113's `toxic_zone` removal

> **ID**: `133`
> **Category**: Decision / correction (worldgen, UI, difficulty)
> **Priority**: 🟡 P2
> **Estimate**: ~30min to decide each, 0-3h to act depending on the choice
> **Assigned to**: done
> **Session**: 2026-08-19 (found during advisor review after task 113
> shipped); resolved same session.

---

## 🎯 What happened

Task 113 removed the standalone `ToxicZoneBounds`/`place_toxic_zone`
rectangle and re-pointed `SurviveIn`'s zone check at `Biome::Swamp`. Two
things that rectangle used to provide for free don't have a replacement
yet, and neither was in 113's own acceptance criteria (they weren't
anticipated when that task was scoped) — flagging both here rather than
deciding unilaterally while closing 113, per this session's own "put it in
a decision task" convention (see task 132 for the same pattern).

---

## 🔀 Gap 1 — `SurviveIn`'s target region has no visual/textual affordance

Before task 113, `render.rs::draw_toxic_zone` painted a dashed purple
outline around the toxic zone's exact rectangle — a player pursuing
`SurviveIn` could always see precisely where to go. Task 113 deleted that
function along with the rectangle it outlined (task 112 had already made
Swamp cells render as a flat color, but never added a name/legend/tooltip
for any biome — there is currently **no way for a player to identify a
Swamp cell by looking at the map**, and no textual cross-reference either:
`text.rs::zone_label` still returns the flavor string `"toxic zone"`,
which no longer corresponds to any labeled or outlined thing in the game.

A player given the `SurviveIn` objective today has to guess which color on
the map is "toxic" with no in-game confirmation.

### Decision: Option 2 (2026-08-19)

Highlighted the active `SurviveIn` target region specifically, closest in
scope and behavior to the old dashed-outline rectangle it replaces.
Option 1 (general biome legend/tooltip) is real future work but strictly
bigger scope than this task's finding warranted — filing it isn't
necessary since option 2 fully closes the gap that motivated this task.
Option 3 alone wouldn't have solved first-time legibility (the actual
problem), and option 4 leaves a real regression unaddressed.

**Implementation** (`src/render.rs`, `terrain_overlay` module):

- `draw_terrain_overlay` gained a `Res<CurrentObjective>` param (already
  `init_resource`d at app startup, so always present — no `Option` needed).
- New `draw_survive_in_target`: no-ops unless `CurrentObjective::current()`
  matches `Objective::SurviveIn { zone: ZoneKind::Toxic, .. }`, then traces
  every Swamp/non-Swamp cell-edge boundary with a dashed purple line —
  same "check right and below neighbours only" trick `draw_boundaries`
  already uses, so every internal edge is visited exactly once regardless
  of Swamp's actual (possibly very irregular) shape.
- Revived `draw_dashed_line` (generalized from the deleted
  `draw_toxic_zone`'s single-rectangle version to take arbitrary
  endpoints) and the `#7F77DD` purple color/width/dash constants, renamed
  `SURVIVE_IN_TARGET_*` — same visual language as the old outline, so a
  player who remembers the old toxic-zone rectangle recognizes the new
  highlight as "the same kind of thing."
- **Not independently visually verified in a live playtest** (reaching an
  active `SurviveIn` objective requires navigating the menu/seeding flow;
  skipped given the surrounding chain of tasks this session) — the
  geometry directly mirrors `draw_boundaries`, which *is* exercised every
  frame in normal play, and `cargo run` launches without panicking. Flag
  this if a future playtest pass doesn't confirm it renders correctly.

---

## 🔀 Gap 2 — the "larger toxic zones" difficulty axis lost its implementation

GDD §9's difficulty curve description lists "large toxic zones" as one of
the axes later worlds ramp up on, backed by (now-removed)
`EnvironmentConfig::toxic_zone_width/height` and
`DifficultyConfig::toxic_zone_width_late/height_late`. Task 113 removed
those fields with nothing replacing them — `Biome::Swamp`'s footprint is
entirely a function of terrain/climate scalars and doesn't scale with
`world_index` at all, so **this difficulty axis is now silently a no-op**:
world 10 and world 0 have the same statistical Swamp footprint, all else
equal.

Nobody noticed this coupling when task 113 was originally scoped (2026-08-11,
before task 125's score-based classification existed) — the acceptance
criteria talk about *removing* the config fields, not about what should
replace their difficulty-curve role.

### Decision: Option 2 (2026-08-19)

Scaled `swamp_toxicity_min` rather than widening Swamp's own
classification footprint (option 1): GDD's "larger toxic zones" literally
describes toxicity-area size, and scaling the toxicity threshold directly
reproduces that without the side effect option 1 would have had on
Desert/Tundra/Forest/Plain's relative shares (widening Swamp's score
window changes who wins the arg-max among *all* Plain-kind candidates,
not just how toxic Swamp itself is). Option 3 (drop the axis) was
rejected — the mechanism to keep it was cheap and direct once identified.

**Implementation:**

- New `DifficultyConfig::swamp_toxicity_min_late: f32 = -0.2` (early end is
  the existing `BiomeConfig::swamp_toxicity_min = 0.3`). Calibrated with a
  20-seed scratch measurement (`cargo run --example`, discarded):
  threshold `0.3` → ~21% of Swamp cells toxic, `0.0` → ~57%, `-0.2` → ~78%,
  `-0.4` → ~92% (rejected — starts to look uniformly toxic, losing the
  sub-region visual variety the noise mask was designed to have). `-0.2`
  gives a real "later worlds are more hostile" step without going
  degenerate.
- New `WorldParams::swamp_toxicity_min: f32`, lerped in `world_params()`
  the same way every other early/late axis is.
- `SimWorld::classify_biomes` now takes `params: &WorldParams` (previously
  `config: &SimConfig` alone) and reads `params.swamp_toxicity_min` for
  the toxicity-imposition threshold instead of `config.biome.
  swamp_toxicity_min` directly. All 6 callsites (1 production, 5 tests)
  updated.
- Mirrored in `assets/config/sim_config.ron`.
- Tests: `world_index_zero_matches_the_early_endpoints_exactly`/
  `the_curve_saturates_at_the_late_endpoints_past_ramp_worlds` extended
  with the new field (the late-endpoint assertion uses an epsilon
  comparison, not `assert_eq!` — `0.3 + (-0.2 - 0.3) * 1.0` doesn't
  round-trip to exactly `-0.2` in `f32`, a genuine floating-point gotcha
  the other fields' round-number defaults happened to avoid); new
  `later_worlds_have_a_larger_toxic_fraction_of_swamp` (world.rs) confirms
  the effect end-to-end, not just that `WorldParams` carries the right
  number.

---

## 📋 Acceptance Criteria (once each gap is decided)

- [x] Gap 1: decision recorded here, implemented if not "do nothing",
      `abiogenesis-gdd.md`/`player_guide.md` updated to match.
- [x] Gap 2: decision recorded here; if scaled, new `WorldParams`/config
      field(s) added (mirrored in `assets/config/sim_config.ron`), a test
      analogous to `worldgen::tests::the_curve_saturates_at_the_late_endpoints_past_ramp_worlds`
      confirming the new axis actually ramps; if dropped, GDD §9 updated
      to stop claiming it.
- [x] `cargo clippy -- -D warnings` / `cargo fmt` / `cargo test` clean if
      code changes.

---

## 🔗 Dependencies

- **Depends on**: 113 (done — this task documents gaps found after it shipped).
- **Blocks**: nothing directly.
