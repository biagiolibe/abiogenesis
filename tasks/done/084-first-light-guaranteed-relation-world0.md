# Task 084 — Guaranteed "first light" relation in world 0's matrix

> **ID**: `084`
> **Category**: Feature / Worldgen / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (scoped from `redesign/processed/abiogenesis-engagement-design.md`, proposal 1.B); unblocked and implemented 2026-08-29 (see notes below)

---

## 🎯 Objective

> **Unblocked 2026-08-29.** Originally blocked on real meta-progression
> persistence — without it, "world 0" means "the first world of *every*
> run," not a true first-timer's, so this guarantee would fire on every
> single playthrough rather than just a player's actual first one.
> **Decision (2026-08-29, user call, following up on the first human
> playtest — `playtest_outcome.md` — and the GDD's own "guaranteed early
> strong interaction" onboarding requirement that was never built):**
> ship it anyway, applied to every `world_index == 0`, accepting that
> trade-off. The playtest showed the opposite failure mode already
> happening in practice — a full run where "even un-evolved Glim keeps
> growing without any interaction from the matrix" — so engagement in the
> first minutes is the more urgent problem today; the rest of a run
> (full matrix, biomes, objectives, xenotraits) stays unpredictable for
> returning players regardless. Revisit if/when real persistence lands
> and a true "first ever world" distinction becomes possible.

World 0 currently generates its hidden matrix with no awareness of being a
new player's first exposure to the system — a first-time player might not
see any strong, legible relation for several eras. Guarantee that at least
one strong (`±2`) matrix relation is reachable and visible early, without
auto-placing any species (stays banned per task 050).

Key simplification found during scoping: the source design doc frames this
as "near the player's first seed," but the game has **no spatial concept of
seed location relevant to matrix effects** — the hidden matrix is entirely
tag-based and position-independent (`world.matrix.get(their_tag, my_tag)`,
`src/sim.rs:288`). "Near the first seed" in practice just means: guarantee
the relation exists **between two tags available in the starting species
palette** (task 013) — whichever two the player picks and seeds adjacent to
each other, they'll see it. No spatial worldgen constraint is needed.

---

## 📋 Acceptance Criteria

- [x] Implemented as `worldgen::ensure_first_light_relation`, called from
      `build_world` right after `generate_starting_palette` for
      `world_index == 0` — **not** inside `generate_matrix`/
      `new_for_world` as originally suggested. Those run *before* any
      species (and so any palette tags) exist, so the check literally
      can't be done there; `build_world`'s doc comment already flagged
      this flexibility as acceptable. `SimWorld::matrix` gained a small
      `pub(crate) fn set` (bypasses the density/cyclicity path) since
      `TagMatrix` previously had no mutator.
- [x] **Deviation found during implementation, resolved 2026-08-29:**
      forcing *any* pair between two palette-carried tags risked breaking
      `draw_species_tags`'s net-zero self-interaction invariant (task
      048/088) whenever both tags happened to already sit inside one
      species' own tag set — empirically ~3% of seeds (e.g. seed 4 with
      default config: one species monopolized every distinct tag the
      whole palette carried). Fixed with a two-tier candidate search:
      tier 1 prefers a pair where both tags are palette-carried *and*
      never co-occur within any single species (fully safe, fully
      visible from the first two organisms seeded); tier 2 falls back to
      pairing a palette-carried tag with a currently-*unused* active tag
      (always safe — no species carries it, so it can never be anyone's
      self-pair) whenever tier 1 has no candidate. Verified failure-free
      across seeds 0-1999 with a throwaway test before removing it.
- [x] The existing cyclicity constraint (a guaranteed negative 3-cycle) is
      untouched — this is a separate, additive step run after
      `generate_matrix` returns, not a change to it.
- [x] No species are auto-placed — only `world.matrix` is touched.
- [x] Unit tests added in `worldgen.rs`: `ensure_first_light_relation_guarantees_a_strong_palette_relation_across_seeds`
      (30 seeds, world 0, always holds) and
      `build_world_does_not_apply_the_first_light_guarantee_past_world_zero`
      (30 seeds, world_index 1, at least one seed lacks it — proves the
      guarantee isn't just ambient random density).
- [x] `cargo build`, `cargo test` (219/219 lib tests, full `cargo test`
      suite including `tests/balance.rs`), `cargo clippy -- -D warnings`,
      `cargo fmt` all clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `generate_matrix` (line 675), cyclicity constraint (696-707), `new_for_world` (194-213). |
| `src/worldgen.rs` | `world_params` (line 53) — precedent for `world_index`-aware generation parameters. |
| `tasks/done/013-starting-species-palette.md` | Reference for which tags are actually available to a fresh player's first seed choices. |

---

## 🧩 Technical Context

- **Current behavior**: `generate_matrix` is called with `slot_count`,
  `matrix_density`, `config`, `rng` — it is **agnostic of `world_index`**;
  no `world_index == 0` branch exists inside matrix generation today (unlike
  `generate_objectives`/`opening_world_objective` in `worldgen.rs`, which are
  already `world_index`-aware per task 079).
- **Desired behavior**: world 0 additionally guarantees a strong, palette-
  reachable relation; every other world's generation is untouched.

---

## 🔨 Suggested Implementation

1. Thread `world_index` into `generate_matrix` (or perform the check/fixup
   in `new_for_world` after calling it, whichever keeps `generate_matrix`
   itself simpler to test in isolation).
2. Add the starting-palette-aware `±2` guarantee, deterministic per seed —
   applied on every `world_index == 0` (see 2026-08-29 decision above, not
   gated on a true first-ever-world check).
3. Unit tests across a seed spread.
4. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- No auto-placed species — stays banned (task 050).
- Keep the existing cyclicity guarantee (696-707) fully intact; this is an
  addition, not a replacement.

---

## 🔗 Dependencies

- **Depends on**: 011 (hidden matrix generation this extends), 013
  (starting species palette this reads). No longer depends on
  meta-progression persistence — see 2026-08-29 decision above.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/084-first-light-guaranteed-relation-world0.md)"$'\n\nExecute this task in the current project.'
```
