# Task 156 — Dominant family bias per world (matrix intensity, not trait selection)

> **ID**: `156`
> **Category**: Feature
> **Priority**: 🟢 P3 (Phase 3 — content and variety)
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-29 scoping

---

> **Status (governed-sdd)**: QUEUED &nbsp;·&nbsp; **Review**: REQUIRED &nbsp;·&nbsp; **Reasoning**: medium
> **Authority**: `redesign/processed/abiogenesis-tag-archetypes.md` (Design source, "Bias di famiglia dominante per mondo") + `abiogenesis-gdd.md:143` (`[PROPOSED]`) + `TECH_DESIGN.md` §5 (determinism invariant)
> **Expected code surface / Out of scope / Validation**: see 📁 Relevant Files, ⚠️ Constraints and Caveats (out-of-scope items and open design questions), and Acceptance Criteria below.

---

## 🎯 Objective

Give each world a chemical "identity" by biasing the **intensity
distribution** of the hidden matrix (GDD §5.5, §5.9: intensities are integers
in `{-2,-1,+1,+2}`), never the trait-selection step and never the sign, for
relationships that involve a trait from one pseudo-randomly-chosen "dominant
family". A world with a dominant genetic family should feel more volatile —
its genetic-family relationships skew toward `±2` — without ever declaring
the family to the player and without changing which traits get selected into
the world's active pool.

Design source: `redesign/processed/abiogenesis-tag-archetypes.md`, section
"Bias di famiglia dominante per mondo (proposta, rivista)" — read together
with GDD `abiogenesis-gdd.md:143` (**[PROPOSED]** "Dominant family per
world"), which is the corrected, current version of the mechanism (the
design doc's own text flags an *earlier* revision — biasing trait
*selection* — as superseded; don't implement that earlier version). The GDD
paragraph is short and is the actual contract for this task:

> "each world draws a dominant trait family from its seed, which biases the
> intensity distribution of that family's matrix relationships toward the
> extremes (`±2` likelier than `±1`) rather than biasing which traits get
> selected — selection bias loses its grip once a world activates most of
> the pool, whereas intensity bias reads identically at 5 or 9 active
> traits. The dominant family is never disclosed; the player infers it from
> play. It never biases the *sign* of effects, only how sharp they are."

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] A world, at generation, deterministically draws one dominant family
      out of the 5 families task 155 introduces (`TraitFamily` or
      equivalent — see Dependencies), from the world's own seeded RNG, the
      same call-site neighbourhood as `select_active_tags`/
      `generate_matrix` (`src/world.rs:759-772`, inside
      `SimWorld::new_for_world`) — **not** a decorrelated offset stream:
      `select_active_tags`/`generate_matrix` already read the main `rng`
      directly (unlike terrain/biome/heat-source generation, which use
      `self.seed ^ *_SEED_OFFSET` streams, `src/world.rs:849,960,1099` etc.
      — see the doc comment on `draw_species_name`, `src/world.rs:3165-3172`,
      for why tag/matrix-adjacent state stays on the main stream), so the
      draw must happen inline with that existing sequence, in a fixed order,
      to keep `same_seed_produces_identical_matrix`-style determinism
      tests passing.
- [ ] `generate_matrix` (`src/world.rs:3041-3074`) or a new helper it calls
      draws intensities for cells where the **exerter or receiver tag**
      belongs to the dominant family from a distribution weighted toward
      `±2` over `±1`, and intensities for every other cell from the
      existing uniform-over-nonzero distribution (`nonzero_intensity`,
      `src/world.rs:3079-3086`, unchanged for non-dominant cells). Resolve
      the open question below (which of exerter/receiver/both triggers the
      bias) before implementing — don't leave it implicit in code with no
      comment recording the choice.
- [ ] The forced negative 3-cycle (`src/world.rs:3061-3071`) is unaffected —
      it already hard-codes intensities in `[effect_intensity_min, -1]` for
      guarantee reasons unrelated to family bias; this task must not touch
      that block's sampling.
- [ ] Bias is on **intensity magnitude only** — never flips or influences
      sign, and never changes `matrix_density` (which off-diagonal cells
      are non-zero) or `select_active_tags`'s trait-selection weights. A
      test asserts the sign distribution (roughly half positive, half
      negative) is statistically unaffected by which family is dominant,
      mirroring the existing `matrix_is_asymmetric_in_general`-style
      sampling test (`src/world.rs:4420-4430`).
- [ ] A new `TagConfig` field (e.g. `dominant_family_extreme_bias: f32`,
      probability that a dominant-family nonzero cell rolls `±2` instead of
      the baseline 50/50) is added, with a default, a doc comment
      explaining what it controls, and a mirrored entry in
      `assets/config/sim_config.ron:71-81` — `tests/config_ron_sync.rs`
      already fails the build if these two drift, per the precedent noted
      at `src/config.rs:516`.
- [ ] A test seeds two worlds with the same tag pool/active count but forces
      (or samples until it observes) different dominant families, and
      asserts the dominant family's relationships skew toward `±2` more
      than a non-dominant family's do in the same matrix — the direct
      behavioural assertion for this task, analogous to
      `matrix_density_is_close_to_configured_target`
      (`src/world.rs:4400-4419`).
- [ ] `same_seed_produces_identical_matrix` (`src/world.rs:4455-...`) still
      passes unmodified in spirit (same seed -> same matrix, including the
      new dominant-family draw) — extend if the new RNG draw needs its own
      determinism assertion, but don't weaken the existing one.
- [ ] The dominant family is **not surfaced anywhere in player-facing UI or
      text** — confirm `notebook.rs`, `text.rs`, `screens.rs` gain no new
      family-name display. This matches GDD `abiogenesis-gdd.md:363`, which
      already anticipates this constraint for the (currently `[PROPOSED]`,
      not-yet-built) relationship graph view: *"Trait family is deliberately
      not shown (it would leak §5.5's dominant-family bias)"*.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `SimWorld::new_for_world` (`759-772`) — draw the dominant family here, in sequence with `select_active_tags`/`generate_matrix`. `generate_matrix` (`3041-3074`) and `nonzero_intensity` (`3079-3086`) — where the biased draw must plug in. `select_active_tags` (`2985-2989`) — must NOT change. |
| `src/config.rs` | `TagConfig` (`467-497`) — add the new bias-strength field and its `Default` (`499-518`). |
| `assets/config/sim_config.ron` | `tags:` block (`71-81`) — mirror the new field, or `config_ron_sync` fails. |
| `tests/config_ron_sync.rs` | Confirms the ron file and `TagConfig::default()` stay in sync — read before adding the field. |
| `src/world.rs` (tests) | `matrix_values_are_in_configured_range` (`4378-4388`), `matrix_diagonal_is_always_zero` (`4389-4399`), `matrix_density_is_close_to_configured_target` (`4400-4419`), `matrix_is_asymmetric_in_general` (`4420-4430`), `matrix_guarantees_a_negative_three_cycle` (`4431-4454`), `same_seed_produces_identical_matrix` (`4455-...`) — existing coverage this task must not regress, and the pattern new tests should follow. |
| `redesign/processed/abiogenesis-tag-archetypes.md` | Design source — "Bias di famiglia dominante per mondo" section. |
| `abiogenesis-gdd.md:143` | The `[PROPOSED]` GDD paragraph this task implements; `abiogenesis-gdd.md:363` — the relationship-graph note confirming family must stay hidden. |
| `tasks/155-*.md` (once filed) | Introduces the 5-family grouping this task consumes — see Dependencies. |

---

## 🧩 Technical Context

- **Current behavior**: `generate_matrix` draws every nonzero cell's
  intensity from a flat distribution over `{-2,-1,+1,+2}` via
  `nonzero_intensity` (uniform `random_range` over the full signed range,
  retried on `0`) — no per-world variation in the *shape* of that
  distribution, only in which cells are nonzero (`matrix_density`) and what
  the forced 3-cycle overwrites.
- **Desired behavior**: cells touching the world's dominant family should
  land on `±2` more often than `±1`, at some configured bias strength;
  every other cell keeps today's flat distribution. Sign stays 50/50
  regardless.
- There is currently **no `TraitFamily` concept anywhere in the codebase**
  (confirmed by grep across `src/*.rs` during scoping — no `family` /
  `Family` symbol outside unrelated `egui::FontFamily` usage). Tags today
  are bare `TagId(u8)` / `TagSlot(u8)` with no grouping metadata
  (`src/world.rs:36,46`). This task cannot be implemented until task 155
  introduces that grouping — see Dependencies below.
- `TagConfig::effect_intensity_min/max` (`src/config.rs:486-489`) are fixed
  at `-2`/`+2` in the shipped default; `nonzero_intensity` treats the range
  as inclusive-exclusive-of-zero, so in practice only 4 discrete outcomes
  exist. The bias mechanism should be phrased generically against
  `effect_intensity_min/max` (not hard-coded to literal `-2`/`+2`), so it
  keeps working if those bounds are ever retuned.
- Task 136 (`tasks/done/136-matrix-necessary-balance.md`) is useful
  background on how matrix intensities feed into energy
  (`interaction_scale` in `sim.rs`) but is not itself touched by this task
  — 156 only changes how intensities are *sampled*, not how they're
  *applied*.

---

## 🔨 Suggested Implementation

1. Confirm task 155 has landed and inspect what it actually exposes for
   family membership — likely a `TraitFamily` enum (5 variants) plus a way
   to map a `TagId`/`TagSlot` to its family (e.g. a lookup table or a field
   on whatever per-tag metadata struct 155 introduces). Do not guess this
   shape ahead of 155 landing; re-read 155's task file and diff before
   starting 156's implementation.
2. In `SimWorld::new_for_world` (`src/world.rs:759-772`), after
   `select_active_tags` and before/alongside `generate_matrix`, draw a
   `TraitFamily` from the main `rng` (5-way uniform choice) and pass it
   into `generate_matrix` alongside the existing `slot_count`,
   `matrix_density`, `config`, `rng` parameters — plus whatever family
   lookup 155 exposes, so `generate_matrix` can test "is this
   exerter/receiver's family the dominant one" per cell.
3. Extend `nonzero_intensity` (or add a sibling, e.g.
   `biased_nonzero_intensity`) to take a `bool` (or bias-strength `f32`)
   flag and weight the draw toward `effect_intensity_min`/`effect_intensity_max`
   (the `±2` extremes) over the `±1` values when the flag is set, keeping
   sign 50/50 in both branches (e.g. roll sign first, then magnitude with
   the bias only affecting the magnitude roll).
4. Add `TagConfig::dominant_family_extreme_bias: f32` (a probability or
   weight ratio — pick whichever `nonzero_intensity`'s rewrite makes
   simplest to express) with a doc comment and default value; mirror in
   `sim_config.ron`.
5. Decide and document (open question below) whether a cell counts as
   "dominant" when the exerter's family matches, the receiver's, or either
   — implement exactly that rule, with a one-line comment explaining the
   choice, in `generate_matrix`.
6. Add/extend tests per the acceptance criteria; run the full existing
   matrix test block (`src/world.rs:4378-...`) to confirm no regression.

---

## ⚠️ Constraints and Caveats

- **Do not implement the earlier "bias trait selection" version.** The
  design doc explicitly supersedes it with the intensity-only version — the
  QUEUE row's own parenthetical ("on matrix intensity, not on trait
  selection") exists specifically to prevent this task from reopening that
  discarded direction. `select_active_tags` (`src/world.rs:2985-2989`) must
  not gain any family-weighted logic.
- **Never bias sign, only magnitude.** This is stated three times across
  the design doc and the GDD paragraph — treat it as a hard invariant, not
  a preference, and cover it with a test (see Acceptance Criteria).
- **Never disclose the dominant family to the player.** No new UI text,
  notebook entry, or log line should name it. The mechanism is meant to be
  inferred from observed matrix behaviour only (GDD `abiogenesis-gdd.md:363`
  already treats this as settled for the — separately scoped, not-yet-built
  — relationship graph view).
- **Determinism**: same seed must produce the same dominant family and the
  same matrix, same as every other piece of world generation
  (`TECH_DESIGN.md` §5 invariant — no `rand::rng()`, RNG lives in world
  state). Draw the family from the world's seeded `rng` at a fixed point in
  the existing generation sequence; don't introduce a new offset stream
  unless there's a concrete reason `select_active_tags`/`generate_matrix`'s
  existing main-stream convention doesn't fit (see Acceptance Criteria for
  why they don't use an offset stream today).
- **`sim`/`world`/`config` stay free of `bevy::render`/`bevy_egui`** — this
  is pure `world.rs`/`config.rs` work, no UI-layer changes except the
  negative check (family name must NOT appear in `notebook.rs`/`text.rs`/
  `screens.rs`).
- **No magic numbers** — the bias strength is a named `SimConfig` field
  (`TagConfig::dominant_family_extreme_bias`), not a literal in
  `generate_matrix`.

### Open design questions (not resolved by the source doc — flag, don't silently pick)

- **Exerter, receiver, or either triggers the bias?** The design doc says
  "relazioni che coinvolgono un tratto della famiglia dominante" ("relations
  that *involve* a trait of the dominant family") — ambiguous between
  "either side" and "both sides must match" for a directional matrix. The
  GDD paragraph doesn't disambiguate further either. "Either side" is the
  more natural reading and preserves more affected cells (bias reads
  identically at 5 or 8 active tags per the doc's own stated goal), but
  this needs an explicit decision recorded in code comments, not a silent
  implementation choice.
- **Exact bias strength** is explicitly left unvalidated by the design doc
  itself ("il peso esatto della distribuzione verso gli estremi resta da
  validare in playtest", listed under "Fuori scope"/"Out of scope" in the
  source). Pick a reasonable starting default (e.g. 70% chance of `±2` vs
  50% baseline) and note in the `TagConfig` doc comment that it is a
  placeholder pending playtest, same treatment task 136 gave
  `interaction_scale` before it was tuned.
- **Interaction with the forced negative 3-cycle**: if one or more of the
  3 forced-cycle slots happen to belong to the dominant family, should the
  forced-cycle intensities (currently drawn uniformly from
  `[effect_intensity_min, -1]`, i.e. `-2` or `-1`) also skew toward `-2`?
  The design doc doesn't address this interaction at all (it predates the
  3-cycle guarantee's role in this specific corner). Acceptance criteria
  above default to "don't touch it" (simplest, avoids coupling a hard
  correctness guarantee to a flavor mechanism) — confirm this reading holds
  before implementing, don't revisit it mid-task without flagging it.

---

## 🔗 Dependencies

- **Depends on: task 155** (Trait archetypes: 3-letter codes, 5 families,
  15-trait active pool — `tasks/QUEUE.md` Phase 3, same `tag-archetypes`
  design source). 156 is not implementable until 155 lands, because 155 is
  what introduces the family grouping this task's whole mechanism operates
  on — today there is no `TraitFamily` type, no per-tag family metadata,
  and no 5-family structure anywhere in `src/*.rs` (confirmed by grep
  during this scoping pass). Concretely, 156 needs 155 to expose:
  - A `TraitFamily` (or equivalently-named) enum with 5 variants
    (structural, metabolic, signalling, genetic, storage per the design
    doc's family list).
  - A way to look up a given active tag's family — keyed on whatever
    identity 155 settles on (`TagId`, the new 3-letter code, or a
    per-tag metadata struct), reachable from `world.rs` generation code
    without pulling in UI-layer types.
  - Confirm 155 doesn't already partially implement per-world family
    weighting itself before starting 156 — re-read 155's finished task
    file first, don't duplicate work.
- **Blocks**: none currently queued.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/156-dominant-family-bias.md)"$'\n\nExecute this task in the current project.'
```
