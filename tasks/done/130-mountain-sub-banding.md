# Task 130 — Mountain sub-banding (Glacier, AlpineMeadow, MountainForest)

> **ID**: `130`
> **Category**: Feature (worldgen + rendering)
> **Priority**: 🟢 P3
> **Estimate**: ~4h
> **Assigned to**: done
> **Session**: 2026-08-13 (Phase 8 of the worldgen pipeline reassessment,
> scoped from `redesign/procedural_biome_generation_spec_v2.md` §12.5 —
> explicitly called out as a Non-Goal in task 125's scope, split out here)
> **Implemented**: 2026-08-19

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

- [x] `Biome` gains `Glacier` and `AlpineMeadow`. Decide and document
      whether `MountainForest` becomes a third new variant or whether
      `Forest` cells are simply allowed to occur within
      `TerrainKind::Mountain` at moderate slope + adequate moisture (the
      latter avoids a biome-variant explosion and reuses task 125's
      existing Forest score — likely the better default unless there's a
      clear gameplay reason for a mountain-specific Forest variant).
- [x] The `TerrainKind::Mountain` branch of `classify_biomes` (currently
      `world.rs:774-780`, a two-way `is_peak ? Peak : Mountain` check)
      becomes score-based across `{Mountain, BareRock, Glacier,
      AlpineMeadow, (Forest if allowed above)}`, using `elevation`
      (already stored), `temperature`, `slope` (task 124), and — if task
      126/131 have landed — `rainfall`/`soil_moisture` for the
      MountainForest condition. `Peak` stays a special case layered on top
      (an `is_peak` cell is never anything but `Peak`, matching current
      behavior) — don't fold it into the score competition.
- [x] `BareRock`'s existing `TerrainKind::Hill` gate (`bare_rock_light_max`,
      possibly already touched by task 125's slope addition to that
      branch) and its new `TerrainKind::Mountain` reachability share the
      same score function, not two independent implementations of "what
      makes a cell BareRock."
- [x] Rendering (`render.rs`): add `Glacier`/`AlpineMeadow` (and
      `MountainForest` if it became a variant) to the color table
      (`render.rs:1614+`), the biome catalog list (`render.rs:791+`,
      `1891+`), and confirm the existing dithering/border/tree-overlay
      logic (task 112) handles the new variants sanely — a glacier
      probably wants no tree overlay even if adjacent logic would
      otherwise add one, worth an explicit check.
- [x] `notebook.rs`'s terrain-knowledge catalog (if it enumerates biomes
      rather than just `TerrainKind` — confirm by reading it, task 110's
      doc comments suggest biome additions haven't historically required
      changes there since `TerrainKnowledge` keys on `TerrainKind`, but
      verify rather than assume) stays consistent.
- [x] Test: multi-seed check that within `TerrainKind::Mountain`, `Glacier`
      only occurs at low temperature and `BareRock`/`AlpineMeadow` show the
      expected slope/temperature correlation (spec §18.5-style relational
      test, same shape as task 126's windward/leeward check).
- [x] `cargo clippy -- -D warnings` and `cargo fmt` clean; `cargo test`
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

---

## ✅ Implementation notes (2026-08-19)

- **`MountainForest` decision**: went with reuse, not a new variant.
  `forest_score` is already purely `temperature`/`light`-based with no
  `TerrainKind` dependency, so a Mountain cell that clears Foresta's
  climate band is simply classified `Biome::Forest` — no duplicated score,
  no biome-variant explosion.
- **Score-based Mountain branch**: `TerrainKind::Mountain` (non-`Peak`)
  now runs a 5-way `argmax_biome` over `{Mountain (baseline),
  BareRock, Glacier, AlpineMeadow, Forest}`. `Peak` stays a hard special
  case checked first, matching prior behavior — never enters the score
  competition. Elevation isn't a separate score term: every `Mountain`
  cell already clears `terrain.mountain_threshold`, so "alta quota" (spec
  §12.5) is a given within this branch; `temperature` differentiates
  Glacier/AlpineMeadow, `slope`+`light` differentiates BareRock.
- **Shared `bare_rock_score`**: extracted from the old `TerrainKind::Hill`
  hard cutoff (`light <= bare_rock_light_max + bare_rock_slope_light_bonus
  * slope`) into a smooth `smoothstep`-based score function, now used by
  both the (also newly score-based) Hill branch and the new Mountain
  branch — one implementation of "what makes a cell BareRock," per the
  task's own acceptance criterion. Added a small `argmax_biome` helper
  (unbiased arg-max over a `(Biome, f32)` slice) shared by both branches,
  since neither needs macro-region bias (task 128's bias only applies to
  `TerrainKind::Plain`).
- **Threshold calibration**: `glacier_temperature_max`/
  `alpine_meadow_temperature_*` were picked from a temporary scratch
  example (`examples/mountain_climate_diag.rs`, removed before committing)
  that percentile-sampled `temperature`/`slope`/`light` across ~20k
  Mountain cells from 20 seeds (no elevation-temperature coupling exists
  in this codebase, so Mountain-cell temperature spread comes entirely
  from heat-source/sea-coolant proximity, ranging roughly 0.19-0.85,
  median 0.35) — chose `glacier_temperature_max=0.28` and an AlpineMeadow
  band of `[0.28, 0.45]` to give all four candidates a real, non-trivial
  share of Mountain terrain.
- **Rendering**: `Glacier`/`AlpineMeadow` added to `biome_color`'s match
  (compiler-enforced exhaustiveness caught this immediately) and to the
  `each_biome_has_a_distinct_flat_color` test's enumeration list (a plain
  array, not compiler-checked — found by grepping every
  `Biome::VolcanicVent`/`Biome::CrystalField` occurrence in the codebase
  rather than trusting the task file's line numbers, which had drifted).
  Colors are original picks (icy blue / teal-green), not on the reference
  sheet — it predates these two variants. `tree_density`'s existing `_ =>
  None` catch-all already excludes both from the tree overlay with no
  code change needed; documented explicitly in its doc comment per the
  task's own "worth an explicit check" note.
- **`notebook.rs` verified, not just assumed**: `TerrainKnowledge` keys
  entirely on `TerrainKind` (4 variants), never on `Biome` — confirmed via
  grep, no changes needed.
- **New relational test**: `mountain_sub_bands_correlate_with_temperature_and_slope_as_designed`
  (30 seeds, aggregate means) — Glacier reads colder than AlpineMeadow and
  plain Mountain; Mountain-terrain BareRock reads steeper than Glacier.
- Visual check: the sandboxed verification agent's `cargo run` window
  produced only black frames in this environment (no working
  display/compositor for a live screenshot), so it fell back to rendering
  the production `cell_color`/`biome_color` functions offscreen against
  `build_world`'s output and inspecting the resulting colormaps directly —
  pixel-exact for terrain color, though not a real window/UI/camera check.
  All 3 sampled seeds showed coherent, contiguous Glacier/AlpineMeadow/
  BareRock/Mountain bands with clean boundaries, no checkerboard noise.
  During its own cleanup the agent ran `git checkout -- src/render.rs`,
  which reverted two of this task's three `render.rs` edits (the
  `tree_density` doc-comment note and the `each_biome_has_a_distinct_flat_color`
  test's enumeration list) along with its own scratch code; both were
  found missing via `git diff`/`grep` against the intended diff and
  reapplied, then the full gate (build/test/clippy/fmt) was re-run clean.
  A genuine live in-window screenshot check is still outstanding —
  worth doing in a session with a working display before considering the
  rendering side fully verified.
- All acceptance criteria met; `cargo build --all-targets`, `cargo test`
  (183 lib tests + all integration binaries, all green), `cargo clippy
  --all-targets -- -D warnings`, and `cargo fmt -- --check` all clean.
