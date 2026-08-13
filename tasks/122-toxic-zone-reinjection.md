# Task 122 — Swamp toxicity reinjection (toxicity erodes with no source to counter it)

> **ID**: `122`
> **Category**: Bugfix / Balance
> **Priority**: 🟢 P3
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-12 (found while balance-testing task 108's
> chemolithotroph metabolism); **rescoped 2026-08-13** after a dependency
> review with tasks 113/125 — see the note below before implementing.

---

## ⚠️ Rescoped 2026-08-13 — read before starting

This task originally targeted `world.toxic_zone: ToxicZoneBounds`. Task 113
("Palude replaces `toxic_zone`") removes that struct entirely, and task 125
(score-based biome classification) moves the only generation-time
`toxicity` source from `place_toxic_zone`'s rectangle to a post-
classification step that imposes `toxicity` on a sub-region of `Biome::
Swamp` cells. **This task must therefore land after both 113 and 125**,
and targets `Cell.biome == Biome::Swamp` (with whatever toxic-flavor marker
125 introduces — check its shipped implementation, it may be a plain
`toxicity > 0.0` check rather than a separate boolean) instead of a
rectangle. The underlying problem this task fixes — nonzero `toxicity` set
once at generation, then only ever eroded by `diffuse_environment`, with
nothing pulling it back — is unchanged by 113/125; only the geometry it
reinjects into changes. Every reference to `ToxicZoneBounds`/`toxic_zone`
below describes the *original* (now superseded) scope and needs to be
re-read as "the Swamp cells 125 marked toxic" once 113/125 have landed —
rewrite this file's Acceptance Criteria/Relevant Files against the actual
shipped API at that point rather than following the stale references
literally.

---

## 🎯 Objective

Heat sources have `reinject_environment_sources` (`src/world.rs:913-936`)
continuously pulling their cells' temperature back toward
`EnvironmentConfig::source_temperature`, countering `diffuse_environment`'s
erosion every tick — documented explicitly as necessary, since diffusion
alone would flatten a heat source into the ambient gradient over time
(`src/world.rs:337` doc comment: "counters `diffuse_environment`'s
erosion"). The toxic zone has no equivalent: `toxicity` is set once at
world generation (originally `set_toxic_zone`, `src/world.rs:639` — after
113/125, the post-classification Swamp toxicity step) and then only ever
diffuses — nothing pulls it back toward the intended value the way
`reinject_environment_sources` does for heat.

**How this was found**: task 108 (chemolithotroph metabolism, gain from
`Cell.toxicity`) added a balance test seeding a well-matched chemolithotroph
directly into a world's toxic zone and running it for `RUN_TICKS` (500
ticks, the same horizon every other test in `tests/balance.rs` uses). It
failed: 36% of seeds lost the organism entirely, well above the file's
`MAX_EXTINCTION_RATE` (30%) budget. The organism was correctly placed
(`temp_optimum` matched to the cell's actual temperature, so `env_fit` was
not the confounder) and started with a comfortable positive net gain
(`0.7 * chemolithotroph_metabolism_gain(2.0) - chemolithotroph_upkeep(0.5) =
+0.9`/tick at the toxic zone's nominal `0.7` value) — the failure only shows
up over a long horizon, consistent with the local toxicity value eroding
toward the grid's ambient average as ticks pass, not an immediate placement
or gain-formula problem. Task 108's own test was narrowed to a 100-tick
window to avoid being confounded by this separate issue (see
`tests/balance.rs`'s `CHEMOLITHOTROPH_SURVIVAL_TICKS` and its doc comment)
— this task is the follow-up to actually fix the underlying erosion, not
just work around it in one test.

---

## 📋 Acceptance Criteria

- [ ] `reinject_environment_sources` (or a parallel function called
      alongside it, whichever reads more naturally next to the existing
      heat-source/sea-coolant reinjection it's modeled on) pulls every cell
      within `world.toxic_zone` back toward
      `EnvironmentConfig::toxic_zone_value` each tick, mirroring the heat
      source's exact math: `toxicity += reinjection_strength * (
      toxic_zone_value - current)`. Reuse `SourceConfig::reinjection_strength`
      only if it's actually the right knob for this — toxicity and
      temperature are different scalars diffusing at the same
      `EnvironmentConfig::diffusion_rate`, so the same strength value is a
      reasonable first guess, but add a **separate** config field (e.g.
      `SourceConfig::toxic_reinjection_strength` or a field on
      `EnvironmentConfig` next to `toxic_zone_value`) rather than coupling
      toxic-zone tuning to heat-source tuning by accident — they should be
      independently tunable even if they start equal.
- [ ] The same invariant heat sources already assert holds for the new
      reinjection too: reinjection strength must exceed `diffusion_rate`,
      or diffusion erodes the zone faster than reinjection restores it
      (mirror the `debug_assert!` at `src/world.rs:915-919`).
- [ ] No magic numbers — the new strength constant lives in `SimConfig`
      (`config.rs` + `assets/config/sim_config.ron`), with a `Default`
      chosen so `tests/balance.rs`'s existing `RUN_TICKS`-horizon tests
      (500 ticks) would keep a well-placed chemolithotroph alive at a rate
      consistent with the file's other `MAX_EXTINCTION_RATE`-style budgets
      — this task's own acceptance bar, not a specific number to reverse-
      engineer up front.
- [ ] Unit test (mirroring `world.rs`'s existing
      `reinjection_strength_stays_compatible_with_diffusion_rate` and the
      heat-source fixed-point tests) confirming a toxic-zone cell's
      toxicity stays near `toxic_zone_value` over many ticks instead of
      eroding toward ambient.
- [ ] `tests/balance.rs`'s `chemolithotroph_survives_reasonably_in_its_toxic_zone_across_seeds`
      (task 108) is restored to the file's normal `RUN_TICKS` horizon
      (500 ticks, matching every other test in the file) now that the
      erosion this task fixes is no longer confounding it — remove the
      `CHEMOLITHOTROPH_SURVIVAL_TICKS` short-horizon workaround and its doc
      comment, and confirm the test passes at the normal horizon before
      calling this done. If it still doesn't pass at 500 ticks after this
      fix, the reinjection strength chosen isn't sufficient — tune it, don't
      leave the test shortened.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: seed a chemolithotroph into the toxic
      zone, let several eras pass, and confirm the zone's toxicity (visible
      via the map's toxic-zone tint) doesn't visibly fade over a long
      session the way it currently does.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `reinject_environment_sources` (913-936) — the pattern to mirror; `toxic_zone: ToxicZoneBounds` (335), `set_toxic_zone` (639) — where the zone's bounds/initial value come from; `diffuse_environment` — what's currently eroding toxicity unopposed. |
| `src/config.rs` | `SourceConfig` (701-, `reinjection_strength`/`sea_coolant_strength` as the pattern) and/or `EnvironmentConfig` (near `toxic_zone_value`) — add the new strength field here. |
| `assets/config/sim_config.ron` | Mirror the new field's default (must match `SimConfig::default()` by hand, per this file's own header comment). |
| `tests/balance.rs` | `chemolithotroph_survives_reasonably_in_its_toxic_zone_across_seeds` and `CHEMOLITHOTROPH_SURVIVAL_TICKS` (task 108) — restore to the full horizon once this lands. |

---

## 🧩 Technical Context

- **Current behavior**: `toxicity` is set once per cell at world generation
  (`set_toxic_zone`) and only ever diffuses afterward
  (`diffuse_environment`) — nothing counters that erosion, unlike
  temperature (heat sources) or the coastal-cooling gradient (sea
  coolant), both of which have `reinject_environment_sources` pulling them
  back every tick.
- **Desired behavior**: the toxic zone should hold roughly steady at
  `toxic_zone_value` over a long run, the same "declared value stays true
  over time" guarantee heat sources already have — a chemolithotroph
  seeded there today, or a `SurviveIn`-the-toxic-zone objective (GDD §8),
  shouldn't have its footing erode out from under it purely from being
  left alone for a few hundred ticks.
- **Why this wasn't caught before task 108**: nothing read `toxicity`
  mechanically before task 108 (GDD §5.2 explicitly called it a
  "declared-but-currently-inert scalar" until then) — so a slowly eroding
  value had no observable gameplay effect until a metabolism actually
  depended on it.

---

## 🔨 Suggested Implementation

1. Read `reinject_environment_sources` and `set_toxic_zone` in full.
2. Add the new strength config field (`config.rs` + `sim_config.ron`),
   with the same diffusion-rate compatibility assertion heat sources have.
3. Extend `reinject_environment_sources` (or add a sibling function called
   from the same place) to pull every cell within `world.toxic_zone`'s
   bounds back toward `toxic_zone_value`.
4. Unit test confirming toxicity holds steady over many ticks.
5. Restore `tests/balance.rs`'s chemolithotroph test to the full `RUN_TICKS`
   horizon; tune the new strength if it doesn't pass yet.
6. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
7. Live-verify per the acceptance criteria.

---

## ⚠️ Constraints and Caveats

- **Don't couple this to heat-source tuning by accident** — a separate
  config field, even if its default starts equal to
  `SourceConfig::reinjection_strength`, keeps the two independently
  tunable (toxicity and temperature diffuse at the same `diffusion_rate`
  today, but there's no reason their reinjection strengths need to move
  together going forward).
- **Determinism**: pure per-cell arithmetic over already-known bounds, no
  RNG — should be safe to add without touching determinism tests, but run
  `tests/determinism.rs`/`tests/run_reproducibility.rs` anyway since they're
  cheap and this does touch the tick's environment step.
- This is a balance/persistence fix, not a rebalance of the chemolithotroph
  metabolism itself (task 108) — don't retune
  `chemolithotroph_metabolism_gain`/`chemolithotroph_upkeep` here unless the
  reinjection fix alone genuinely isn't enough to pass the restored
  500-tick balance test.

---

## 🔗 Dependencies

- **Depends on**: 085 (the `reinject_environment_sources` pattern this
  mirrors), 108 (the chemolithotroph metabolism whose balance test
  surfaced this), **113 and 125, hard blockers as of the 2026-08-13
  rescope** — 125 defines the new toxicity source (Swamp cells), 113
  removes the old geometry (`ToxicZoneBounds`) this task originally
  targeted. Do not start this task before both have shipped.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/122-toxic-zone-reinjection.md)"$'\n\nExecute this task in the current project.'
```
