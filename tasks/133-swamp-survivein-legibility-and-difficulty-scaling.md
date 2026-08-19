# Task 133 — [DECISION] Two open gaps left by task 113's `toxic_zone` removal

> **ID**: `133`
> **Category**: Decision / correction (worldgen, UI, difficulty)
> **Priority**: 🟡 P2
> **Estimate**: ~30min to decide each, 0-3h to act depending on the choice
> **Assigned to**: unassigned
> **Session**: 2026-08-19 (found during advisor review after task 113 shipped)

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

### Options (not decided)

1. **Add a biome legend/tooltip.** Hover or click a cell to see its biome
   name (`Biome::Swamp` → "Swamp") — general-purpose legibility
   infrastructure, useful beyond this one objective, but the largest scope.
2. **Highlight the active `SurviveIn` target region specifically**, only
   while that objective is active — closer to the old dashed-outline
   behavior/scope, but a one-off special case rather than reusable biome
   legibility.
3. **Rename the objective/zone label to name the biome explicitly**
   (`"toxic zone"` → `"Swamp"`), and lean on `render.rs`'s existing biome
   color alone once the player learns to associate that color with
   "Swamp" from repeated play — cheapest, but doesn't solve first-time
   legibility, only a hint for returning players.
4. **Do nothing for now.** Track as a known gap; `SurviveIn` still
   functions correctly (task 113's own acceptance criteria are met), this
   is a legibility regression, not a correctness one.

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

### Options (not decided)

1. **Scale a `BiomeConfig` swamp-fitness parameter with `world_index`**
   (e.g. `swamp_slope_max`/`swamp_water_distance_max` widen at the late
   endpoint, the same lerp idiom `WorldParams` already uses elsewhere) —
   keeps the "more hostile ground later" intent, implemented through the
   score-based system rather than against it.
2. **Scale `swamp_toxicity_min`** (the noise-threshold selecting which
   fraction of Swamp reads as toxic) instead — a narrower, more targeted
   axis: same Swamp footprint size, but more of it is toxic at higher
   severity.
3. **Drop the axis.** Update GDD §9 to remove "larger toxic zones" from
   the documented curve rather than re-implementing it — decide the
   feature just isn't worth carrying forward now that toxicity is
   biome-derived rather than a standalone knob.

---

## 📋 Acceptance Criteria (once each gap is decided)

- [ ] Gap 1: decision recorded here, implemented if not "do nothing",
      `abiogenesis-gdd.md`/`player_guide.md` updated to match.
- [ ] Gap 2: decision recorded here; if scaled, new `WorldParams`/config
      field(s) added (mirrored in `assets/config/sim_config.ron`), a test
      analogous to `worldgen::tests::the_curve_saturates_at_the_late_endpoints_past_ramp_worlds`
      confirming the new axis actually ramps; if dropped, GDD §9 updated
      to stop claiming it.
- [ ] `cargo clippy -- -D warnings` / `cargo fmt` / `cargo test` clean if
      code changes.

---

## 🔗 Dependencies

- **Depends on**: 113 (done — this task documents gaps found after it shipped).
- **Blocks**: nothing directly.
