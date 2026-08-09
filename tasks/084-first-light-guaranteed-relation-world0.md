# Task 084 — Guaranteed "first light" relation in world 0's matrix (BLOCKED)

> **ID**: `084`
> **Category**: Feature / Worldgen / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~2h (once unblocked)
> **Assigned to**: unassigned — **do not start**, see Dependencies
> **Session**: 2026-08-09 (scoped from `redesign/abiogenesis-engagement-design.md`, proposal 1.B)

---

## 🎯 Objective

> **This task is scoped for reference but blocked from starting.** Without
> real meta-progression persistence (SECTION 1's "Meta-progression
> persistence... deliberately deferred" proposal), there is no way to
> distinguish a player's true first-ever world from any later restart. If
> implemented today, "world 0" would mean "the first world of *every* run,"
> guaranteeing the strong relation on every single playthrough — not just a
> true first-timer's — which undermines unpredictability for returning
> players. Do not pick this up until the persistence proposal is resolved.

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

## 📋 Acceptance Criteria (once unblocked)

- [ ] `generate_matrix` (`src/world.rs:675`) or its caller
      `SimWorld::new_for_world` (`src/world.rs:194-213`) gains world-0-only
      logic: when `world_index == 0`, after the existing cyclicity
      constraint (lines 696-707) is applied, verify at least one entry
      between two tags present in the starting species palette (task 013)
      reaches magnitude `±2`. If not, force one such entry to
      `config.effect_intensity_max` (or `-effect_intensity_max`), chosen
      deterministically from the world's RNG stream — no new source of
      non-determinism.
- [ ] The existing cyclicity constraint (a guaranteed negative 3-cycle,
      lines 696-707) is preserved — this task adds an additional, separate
      guarantee, it does not replace or weaken the existing one.
- [ ] No species are auto-placed — this only shapes matrix generation, never
      touches `SimWorld`'s occupied cells.
- [ ] Unit test: across a spread of seeds, world 0's generated matrix always
      has at least one `±2` (or stronger) entry between two starting-palette
      tags; a non-zero world index does not get this guarantee (matrix stays
      as randomly generated as before for `world_index != 0`).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.

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

## 🔨 Suggested Implementation (once unblocked)

1. Resolve the "Meta-progression persistence" proposal first — this task
   cannot start meaningfully before that.
2. Thread `world_index` into `generate_matrix` (or perform the check/fixup
   in `new_for_world` after calling it, whichever keeps `generate_matrix`
   itself simpler to test in isolation).
3. Add the starting-palette-aware `±2` guarantee, deterministic per seed.
4. Unit tests across a seed spread.
5. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- **Do not start this task until the persistence blocker is resolved** — see
  Objective. If picked up prematurely, flag it back to the user rather than
  proceeding.
- No auto-placed species — stays banned (task 050).
- Keep the existing cyclicity guarantee (696-707) fully intact; this is an
  addition, not a replacement.

---

## 🔗 Dependencies

- **Depends on**: "Meta-progression persistence" (still a `[?]` proposal in
  `PROJECT_PLAN.md` SECTION 1, not yet a task — this task cannot be
  scheduled until that one exists and ships), 011 (hidden matrix generation
  this extends), 013 (starting species palette this reads).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/084-first-light-guaranteed-relation-world0.md)"$'\n\nExecute this task in the current project.'
```
