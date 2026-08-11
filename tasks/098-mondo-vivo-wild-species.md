# Task 098 — Wild, pre-existing species at world generation

> **ID**: `098`
> **Category**: Feature / Worldgen / Simulation
> **Priority**: 🟡 P2
> **Estimate**: ~4h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-living-world.md`, §2a "Wild, pre-existing species")

---

## 🎯 Objective

At world generation, in addition to the species the player seeds, place a
small number of "wild" populations — same `Species`/tag/matrix machinery,
no parallel system — in zones not immediately visible or reachable from
the player's likely starting area. First contact (a player-seeded
species' population reaching interaction range of a wild population) is the
discovery trigger, reusing the existing interaction-spark visual feedback
(task 080) and the unconfirmed→confirmed notebook log pattern (task 054).

**This is a deliberate, narrowly-scoped exception to task 050's "no
auto-placed organisms" rule.** Task 050 banned auto-placement of the
*starting palette* specifically because it undercut the game's premise —
"the player seeds life into a sterile world." Wild species are a different
thing: a hidden, undiscovered population the player finds, not something
handed to them at start. The exception must stay narrow — this task does
not reopen 050 for the starting palette, and the wild populations must not
read, on the HUD or catalog, as "species available to seed" (they aren't;
they're already alive, elsewhere).

---

## 📋 Acceptance Criteria

- [ ] `WorldgenConfig` (`src/config.rs`, alongside `starting_species_count`/
      `extra_available_species_count`) gains `wild_species_count: u32` (no
      magic numbers), first-pass guess `1`-`2`.
- [ ] A new worldgen step (e.g. `worldgen::place_wild_species`, called from
      `build_world`, `src/worldgen.rs:252-259`, after `generate_starting_palette`)
      generates `wild_species_count` species via the same machinery
      `generate_starting_palette`/`add_bonus_species` already use
      (`draw_species_tags`, `draw_species_name`, temperature spread) and
      places **one organism each** directly onto `world.cells` — the one
      place in the codebase allowed to write `cell.organism = Some(..)` at
      worldgen time outside `input.rs`'s player-driven `Seed` path.
- [ ] Wild species are **not** added to `world.species`'s player-facing
      "available to seed" pool distinction if one exists in this codebase's
      UI layer (`input.rs`'s `Seed` picker) — check how `Seed`
      enumerates choices (likely all of `world.species`) and decide: either
      wild species get their own `SimWorld` list separate from
      player-seedable ones, or they stay in `world.species` but are excluded
      from the `Seed` picker by a flag on `Species`/`SpeciesId`. Document the
      choice; either is acceptable, but wild species must not appear as a
      pickable option.
- [ ] Placement respects `is_placeable_index` (`world.rs:756-758` — no `Sea`
      or mountain-peak placement) and a first-pass spatial rule for "not
      immediately reachable/visible from the player's likely starting area."
      There is no existing "player start position" concept in the codebase
      (`Seed` places anywhere the player clicks, no fixed camera-start
      cell) — define reachability as distance from the grid's center (or a
      far quadrant/corner), tunable via a new `SimConfig` field (e.g.
      `wild_species_min_distance_from_center`), first-pass guess, tune
      visually per 081's precedent, not derived from any existing spatial
      constant.
- [ ] **`is_total_extinction` decision, made and documented**
      (`src/objectives.rs:275-277`, `world.ever_populated && all cells empty`):
      a wild population staying alive after every player-seeded organism
      dies must not silently prevent extinction failure — decide whether
      `is_total_extinction` should evaluate over player-lineage organisms
      only (needs a way to distinguish wild vs. player-descended `SpeciesId`s,
      e.g. a bool on `Species` or a tracked `SpeciesId` range) or globally
      (accept that a surviving wild population blocks extinction — likely
      wrong, since the player's own run would then never fail on
      extinction). Pick the option that keeps extinction meaningful for the
      player's own lineages; document the choice and add a regression test.
- [ ] **`ever_populated` decision, made and documented**: does placing a
      wild organism at worldgen set `world.ever_populated = true`
      (`world.rs:216`, flipped today only by `sim::step` once a real
      organism count is seen, `sim.rs:124-130`)? If yes, the extinction
      guard (`objectives.rs:276`) becomes permanently satisfied by the wild
      population alone, defeating its "hasn't been seeded yet" purpose —
      almost certainly wild placement should **not** set `ever_populated`;
      confirm this explicitly and add a test asserting a fresh world with
      only wild species placed still has `ever_populated == false`.
- [ ] **`Coexistence` objective decision, made and documented**
      (`worldgen::generate_objective`'s `min_species`/species-count clamp):
      do wild species count toward the pool `Coexistence` draws its
      `min_species` target from? If yes, the objective could become
      trivially satisfiable by wild populations the player never interacted
      with — decide and document whether wild species are excluded from
      that count.
- [ ] Discovery trigger: when a player-seeded species' organism first
      occupies a cell within interaction range (Moore-neighbor adjacency, the
      same range `interaction_delta` reads) of a wild population's organism,
      fire the same spark (task 080's `spawn_spark_on_first_observation`
      pattern, `src/render.rs:869` and the `SparkIndicators` machinery,
      `render.rs:744-830`) and the same unconfirmed→confirmed notebook log
      entry (task 054's pattern, `notebook.rs`'s confirmation log path,
      `notebook.rs:254-259`) that a normal tag-pair confirmation already
      uses — first contact with a wild species is functionally a first
      observation of its tags, so it should ride the exact same
      `AdjacencyObserved`/`MatrixKnowledge` pipeline, not a new one.
- [ ] Unit test: `wild_species_count` wild organisms exist on the grid
      immediately after `build_world`, before any `Seed` action.
- [ ] Unit test: wild placement never lands on non-placeable terrain.
- [ ] Determinism: same seed → same wild species (tags, position,
      temperature optimum).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: a fresh world contains at least one
      wild population not visible near the player's initial view; seeding a
      player species and letting it spread into contact range with the wild
      population triggers the spark and a notebook log entry.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/worldgen.rs` | `generate_starting_palette` (156-181), `add_bonus_species` (191-222), `build_world` (252-259) — precedent and insertion point for the new wild-placement step. |
| `src/world.rs` | `is_placeable_index` (756-758), `SimWorld::ever_populated` (216), `Species`/`SpeciesId`. |
| `src/objectives.rs` | `is_total_extinction` (275-277), `Coexistence` species-count clamp. |
| `src/sim.rs` | `ever_populated` flip site (124-130) — reference for whether wild placement should mirror or bypass it. |
| `src/input.rs` | `Seed` action / species picker (231, 261) — where wild species must be excluded from pickable choices. |
| `src/render.rs` | `spawn_spark_on_first_observation` (869), `SparkIndicators` (744-830) — reused discovery feedback. |
| `src/notebook.rs` | `accumulate_evidence` (233-259) — reused confirmation log pattern. |
| `tasks/done/050-no-auto-placed-starting-organisms.md` | The rule this task carves a narrow, documented exception into — read in full before implementing. |
| `tasks/done/080-interaction-spark-visual-feedback.md`, `tasks/done/054-celebrate-first-confirmed-hypothesis.md` | Discovery-feedback precedents this task reuses. |

---

## 🧩 Technical Context

- **Current behavior**: `build_world` (`worldgen.rs:252-259`) calls
  `SimWorld::new_for_world`, `generate_starting_palette`, then
  `generate_objectives` — every generated species goes into the available
  pool only, no organism is ever placed on the grid until the player's first
  `Seed` (task 050). `is_total_extinction` and `ever_populated` both encode
  "the player hasn't acted yet" as a first-class, tested state
  (`objectives.rs:888-897`).
- **Desired behavior**: a small number of wild organisms exist on the grid
  from world start, outside the player's normal picker, without breaking
  the "hasn't been seeded yet" semantics task 050 built for the player's own
  lineages.
- **Open question flagged in the source doc, treated here as a first-pass
  tunable, not a blocker**: how many wild populations per world, and whether
  placement guarantees reachability within a normal run's spread radius, or
  "may never be found" is an acceptable (even desirable, per the doc)
  outcome for some worlds. This task picks a first-pass distance rule and a
  small `wild_species_count`; both are tunable in `SimConfig`, validated by
  playtest, not derived analytically here.

---

## 🔨 Suggested Implementation

1. Read `tasks/done/050-no-auto-placed-starting-organisms.md` in full.
2. `src/config.rs`: add `WorldgenConfig::wild_species_count` and a
   spatial-distance config field for placement.
3. `src/worldgen.rs`: add `place_wild_species(world, config)`, generating
   species via the existing `draw_species_tags`/`draw_species_name` pattern
   and writing directly to `world.cells[idx].organism`, respecting
   `is_placeable_index` and the distance-from-center rule.
4. Decide and implement the `ever_populated`/`is_total_extinction`/
   `Coexistence` interactions per the Acceptance Criteria — likely a bool
   flag on `Species` (`is_wild: bool`) or a separate tracked ID range, used
   to filter extinction/coexistence checks and the `Seed` picker.
5. Wire the discovery trigger through the existing `AdjacencyObserved`/spark
   pipeline — confirm first contact with a wild species' tags naturally
   flows through `interaction_delta`'s existing neighbor scan with no new
   event type needed.
6. Unit tests per Acceptance Criteria.
7. `cargo run` live verification.
8. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- **Narrow exception to task 050, not a reopening of it.** The starting
  palette stays exactly as 050 left it — nothing pre-placed for the player
  to find immediately. Wild species are placed away from the likely
  starting area and excluded from the seed picker.
- **Must not weaken extinction or coexistence checks** — the three
  Acceptance Criteria decisions above (`ever_populated`, `is_total_extinction`,
  `Coexistence` count) are must-decide, not optional; get them wrong and a
  wild population silently makes the player's own run unfailable or an
  objective trivial.
- **No `HashMap` iteration in sim logic**; wild-species bookkeeping (if any
  flag/set is needed) should follow the same `Vec`/bool-on-struct pattern
  used elsewhere in this codebase.
- **Determinism**: wild placement draws from `world`'s own seeded RNG only.
- Reachability/count numbers are first-pass, tune-by-playtest guesses —
  say so explicitly rather than presenting them as derived.

---

## 🔗 Dependencies

- **Depends on**: 050 (the rule this carves an exception into — must be
  read and respected), 010/036/038 (tag drawing/active pool machinery), 067
  (placement gating), 080 (interaction spark), 054 (confirmation log
  pattern).
- **Blocks**: none.
- **Related, not a dependency**: 096 (conditional tags) — a wild species can
  carry a conditional tag like any other; no coordination required, but a
  wild population discovered on its trigger terrain is a natural, doubly
  satisfying "aha" once both land.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/098-mondo-vivo-wild-species.md)"$'\n\nExecute this task in the current project.'
```
