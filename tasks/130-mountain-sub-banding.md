# Task 130 — Mountain sub-banding (Glacier, AlpineMeadow, MountainForest)

> **ID**: `130`
> **Category**: Feature (worldgen + rendering)
> **Priority**: 🟢 P3
> **Estimate**: ~4h
> **Assigned to**: unassigned
> **Session**: 2026-08-13 (Phase 8 of the worldgen pipeline reassessment,
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §12.5 —
> explicitly called out as a Non-Goal in task 125's scope, split out here)

---

## 🎯 Objective

Today every non-peak `TerrainKind::Mountain` cell reads as the single
`Biome::Mountain`, regardless of temperature or slope — a whole mountain
range renders as visually uniform. The spec's §12.5 table distinguishes:

```text
alta quota + temperatura bassa   -> Glacier/Snow
alta pendenza + temperatura media -> BareRock
quota alta + temperatura moderata -> AlpineMeadow
pendenza moderata + umidità       -> MountainForest
```

`BareRock` already exists as a `Biome` variant but today is only reachable
from `TerrainKind::Hill` (`world.rs:781-787`), gated on `light` alone. This
task extends the same idea to `TerrainKind::Mountain`, using the
score-based approach task 125 introduced (reuse `biome_score`'s idiom, not
a new priority chain), and adds the two genuinely new variants: `Glacier`
and `AlpineMeadow`. `MountainForest` is a judgment call — see below.

---

## 📋 Acceptance Criteria

- [ ] `Biome` gains `Glacier` and `AlpineMeadow`. Decide and document
      whether `MountainForest` becomes a third new variant or whether
      `Forest` cells are simply allowed to occur within
      `TerrainKind::Mountain` at moderate slope + adequate moisture (the
      latter avoids a biome-variant explosion and reuses task 125's
      existing Forest score — likely the better default unless there's a
      clear gameplay reason for a mountain-specific Forest variant).
- [ ] The `TerrainKind::Mountain` branch of `classify_biomes` (currently
      `world.rs:774-780`, a two-way `is_peak ? Peak : Mountain` check)
      becomes score-based across `{Mountain, BareRock, Glacier,
      AlpineMeadow, (Forest if allowed above)}`, using `elevation`
      (already stored), `temperature`, `slope` (task 124), and — if task
      126/131 have landed — `rainfall`/`soil_moisture` for the
      MountainForest condition. `Peak` stays a special case layered on top
      (an `is_peak` cell is never anything but `Peak`, matching current
      behavior) — don't fold it into the score competition.
- [ ] `BareRock`'s existing `TerrainKind::Hill` gate (`bare_rock_light_max`,
      possibly already touched by task 125's slope addition to that
      branch) and its new `TerrainKind::Mountain` reachability share the
      same score function, not two independent implementations of "what
      makes a cell BareRock."
- [ ] Rendering (`render.rs`): add `Glacier`/`AlpineMeadow` (and
      `MountainForest` if it became a variant) to the color table
      (`render.rs:1614+`), the biome catalog list (`render.rs:791+`,
      `1891+`), and confirm the existing dithering/border/tree-overlay
      logic (task 112) handles the new variants sanely — a glacier
      probably wants no tree overlay even if adjacent logic would
      otherwise add one, worth an explicit check.
- [ ] `notebook.rs`'s terrain-knowledge catalog (if it enumerates biomes
      rather than just `TerrainKind` — confirm by reading it, task 110's
      doc comments suggest biome additions haven't historically required
      changes there since `TerrainKnowledge` keys on `TerrainKind`, but
      verify rather than assume) stays consistent.
- [ ] Test: multi-seed check that within `TerrainKind::Mountain`, `Glacier`
      only occurs at low temperature and `BareRock`/`AlpineMeadow` show the
      expected slope/temperature correlation (spec §18.5-style relational
      test, same shape as task 126's windward/leeward check).
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
      passes.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs:774-787` | `classify_biomes`'s `TerrainKind::Mountain`/`Hill` branches — extended here. |
| `src/world.rs:221-249` | `Biome` enum — add `Glacier`/`AlpineMeadow` (and possibly `MountainForest`). |
| `src/render.rs:656, 791, 798-799, 1614-1618, 1891-1895` | Color table, catalog list, dithering/tree-overlay rules — every site enumerating biomes needs the new variants added. |

---

## ⚠️ Constraints and Caveats

- **No magic numbers**: new score-curve parameters in `BiomeConfig`.
- Keep `Peak` a special case, not part of the score competition — matches
  existing behavior and avoids a peak "losing" to a Glacier score by a
  small margin, which would read as a regression.
- This task touches rendering (`render.rs`) as well as worldgen — unlike
  123-129, which stayed on the data side. Budget review time for the
  catalog/color additions accordingly (same shape as task 112's original
  scope, applied to three new variants instead of the original set).

---

## 🔗 Dependencies

- **Depends on**: 124 (`slope`), 125 (the score-based idiom to extend).
  126/131 (`rainfall`/`soil_moisture`) improve the MountainForest condition
  if landed first but aren't a hard blocker — a slope-only proxy is an
  acceptable first pass if picked up before those exist.
- **Blocks**: none.
- **Note (task 129, resolved 2026-08-19)**: the worldgen pipeline was
  reordered so `compute_hydrology` (and depression/Lake data) now runs
  *before* `classify_biomes`, instead of after. `Cell.is_river` and
  `flow_accumulation` are therefore available to `classify_biomes` for the
  first time — worth considering as an extra input for MountainForest (a
  mountain cell near a headwater stream reads as plausibly wetter than
  slope/rainfall alone would suggest), though not a requirement if
  rainfall/soil_moisture already cover it adequately.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/130-mountain-sub-banding.md)"$'\n\nExecute this task in the current project.'
```
