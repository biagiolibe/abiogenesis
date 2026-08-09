# Task 074 — Final grid size (empirical tuning)

> **ID**: `074`
> **Category**: Balance / tuning
> **Priority**: 🟢 P3
> **Estimate**: ~1-2h (playtesting-heavy, not implementation-heavy)
> **Assigned to**: unassigned
> **Session**: 2026-08-09

---

## 🎯 Objective

GDD §5.1 leaves the grid size explicitly open, flagged "remains empirical."
The current default (`GridConfig { width: 48, height: 32 }`, `src/config.rs`)
has been carried since Phase 0 and never revisited against the game's later
additions — terrain (066-072), objectives, the toxic zone, and the sidebar
console redesign (063-065) all landed on top of it without a dedicated pass
asking whether 48×32 is still the right size now that the map carries much
more visual/simulation information than an empty grid of colored dots.

Pick a final grid size (or confirm 48×32 is already right) by playtesting a
small number of candidate sizes and comparing them against concrete
criteria, not by guessing analytically.

---

## 📋 Acceptance Criteria

- [ ] At least 2-3 candidate grid sizes (including the current 48×32)
      are actually played, not just reasoned about — each for enough ticks/
      eras to judge population dynamics, not just the opening state.
- [ ] The choice is written down with its rationale in `PROJECT_PLAN.md`'s
      Final tuning entry (or a short note in this task file if more
      detail is useful), referencing what was compared and why the winner
      won — not just a bare number change.
- [ ] Whatever size is chosen becomes `GridConfig::width`/`height`'s value
      (via the RON asset from task 073, if that's done first — editing the
      value should not require touching `config.rs`'s Rust source at all
      once 073 is in place).
- [ ] Existing tests that assume specific grid dimensions (search for
      `48`/`32`/`world.width`/`world.height` literals in test code, e.g.
      `balance.rs`, `world.rs`'s terrain tests, any test hardcoding a cell
      index or toxic-zone position) are checked and updated if the new
      size breaks an assumption baked into a test rather than into the
      simulation itself.
- [ ] `min_placeable_fraction`, toxic zone sizing
      (`WorldParams::toxic_zone_width`/`height`), and any other
      size-relative `TerrainConfig`/`DifficultyConfig` values are
      re-checked at the new grid size — a fixed toxic-zone footprint or
      placeable-fraction floor tuned for 48×32 may behave differently at a
      different total cell count.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean at the final
      chosen size.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/config.rs` | `GridConfig` (or the RON asset from 073, if landed first). |
| `redesign/abiogenesis-terrain-map.md`, `redesign/abiogenesis-sidebar-redesign.md` | Prior design docs whose mockups/decisions assumed *some* grid size — worth a skim to confirm a size change doesn't silently break an assumption they made. |
| `tests/balance.rs`, `src/world.rs` (terrain tests) | Tests most likely to hardcode grid-size-relative expectations. |

---

## 🧩 Technical Context

- **Current behavior**: 48×32 = 1536 cells, unchanged since Phase 0's
  walking skeleton, before terrain (066-072) or the current sidebar layout
  existed.
- **What "final" should weigh** (concrete, playtestable criteria — not
  exhaustive, refine during the task):
  - **Readability at the current window/camera setup**: `render.rs`'s
    `spawn_camera` fits the whole grid via `ScalingMode::AutoMin`; a much
    larger grid means smaller cells on the same window, which may hurt
    glyph/organism-shape legibility (task 032's shape-based species
    distinction, task 068's peak glyphs).
  - **Population dynamics at scale**: `tests/balance.rs`'s existing
    invariants (`population_never_saturates_the_grid_across_seeds`,
    `bloom_usually_grows_then_stabilises_across_seeds`, etc.) encode
    balance assumptions tuned at 48×32 — a size change could shift these
    dynamics enough to need re-tuning, which is exactly what this task
    should surface before locking in a size.
  - **Terrain read** (066-072): the "compact organic landmass in a larger
    sea" read task 072 just tuned assumes a specific cell count; a
    different grid size changes how many cells the continent-scale noise
    bands (`continent_freq_min`/`max`) span, which could need its own
    re-check (out of scope to *fix* here if it breaks — flag it as a
    follow-up task if so, don't silently re-tune terrain noise as a side
    effect of a grid-size task).
  - **Performance**: tick cost scales with cell count (Moore-neighbourhood
    passes, diffusion, etc.) — confirm a larger candidate doesn't introduce
    visible per-tick stutter, especially since the sim aims to stay
    headless-testable and responsive.

---

## 🔨 Suggested Implementation

1. If task 073 (RON hot-reload) has landed, use it to flip between
   candidate sizes live via `cargo run` without recompiling. If not, edit
   `GridConfig::default()` directly between playtests.
2. Playtest the current 48×32 baseline for a few eras, taking note of
   readability, pacing, and any terrain/objective oddities.
3. Playtest at least one smaller and/or larger candidate (e.g. something
   like 36×24 and 64×40 as starting points — adjust based on what the
   baseline session reveals is worth testing).
4. Compare against the criteria above; pick a winner (or confirm the
   baseline).
5. Set the final size, re-run `cargo test`, fix any grid-size-relative
   test literals that broke.
6. Spot-check terrain (task 072's sea/land balance), toxic zone placement,
   and objective pacing at the new size — file a follow-up task if any of
   these need their own re-tuning pass rather than fixing them inline here.
7. Write the decision + rationale into `PROJECT_PLAN.md`.

---

## ⚠️ Constraints and Caveats

- **This is a playtesting task, not an implementation task** — resist the
  urge to pick a size analytically and move on; the acceptance criteria
  require actually playing candidates.
- **Don't silently re-tune unrelated systems** (terrain noise frequencies,
  toxic zone size, objective thresholds) as a side effect — if the new
  grid size exposes a real problem in one of those, note it and open a
  follow-up task rather than scope-creeping this one.
- **No magic numbers**: the chosen size is still a `GridConfig` field (or
  RON value), not a hardcoded literal anywhere else.

---

## 🔗 Dependencies

- **Depends on**: 073 (soft dependency — not technically blocking, but
  doing 073 first makes this task much faster to execute).
- **Blocks**: none directly, though a grid-size change is likely to
  surface follow-up tuning tasks (terrain, toxic zone, balance tests).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/074-final-grid-size-tuning.md)"$'\n\nExecute this task in the current project.'
```
