# Task 095 — Procedural per-world species names + readable descriptions

> **ID**: `095`
> **Category**: Feature / Presentation
> **Priority**: 🟢 P3
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-10, user-requested: species names should vary
> randomly, and each species should carry a short narrative description
> (metabolism, temperature, reproduction) beyond the current compact stat
> line.

---

## 🎯 Objective

Two related presentation gaps in how a species is identified/read:

1. **Names never actually vary.** `render::species_label(id: SpeciesId)`
   (`render.rs:26-43`) is a *pure function of `SpeciesId` alone* —
   `SPECIES_NAMES[id.0 as usize % 16]`. Species 0 is "Nyx" in literally
   every world, every seed, every run — the name list rotates through
   `SpeciesId`, it doesn't randomize per world the way tags/matrix/terrain
   already do. "Nomi casuali sempre diversi" means names should draw from
   the world's own seeded RNG at species creation, like every other
   per-world-varying piece of generation.
2. **No narrative description exists.** `text::species_catalog_line`
   (`text.rs:433-444`) is the only per-species text today — a dense stat
   line ("Nyx: Photolithic · temp 0.52±0.15 (temperate) · repro ≥10.0").
   Useful for precise values, but not a readable sentence a new player can
   parse at a glance. Add a short generated description that translates
   the same underlying stats (metabolism, temp_optimum/tolerance,
   repro_threshold) into plain language, alongside — not replacing — the
   existing precise line (`Splice`'s `ShiftTempOptimum` math still needs
   the exact numbers visible somewhere).

---

## 📋 Acceptance Criteria

- [x] Each species' display name is drawn from the world's own seeded RNG
      (`world.rng_mut()`, same discipline every other per-species draw
      already follows — e.g. `draw_species_tags`) at the moment the species
      is created, not derived from `SpeciesId` alone. Two different worlds
      (different seeds) must be able to produce different names for
      "species 0"; the same world reseeded with the same seed must produce
      the same names (determinism, TECH_DESIGN.md invariant 1).

      `world::draw_species_name(world: &mut SimWorld) -> String` samples
      `SPECIES_NAMES` via `world.rng`, mirroring `draw_species_tags`
      exactly. New tests: `draw_species_name_is_deterministic_for_the_same_seed`
      (world.rs) and `starting_species_names_vary_across_seeds` (worldgen.rs).
- [x] The name is stored on `Species` (new field — representation is the
      implementer's call: a `String`, or an index into an expanded name
      pool resolved at display time; either way `Species` stays
      `Clone`/`PartialEq`, already not `Copy` today) rather than recomputed
      from `SpeciesId` at every display call site — a species' name must
      stay stable for its whole lifetime once assigned, including across
      `Splice`-derived children (each gets its *own* independent draw, not
      a copy of its parent's).

      Went with `pub name: String` directly on `Species`. `apply_splice`
      clones `source_species` then explicitly overwrites `new_species.name`
      with a fresh draw — proven by a new test,
      `spliced_species_draws_its_own_name_not_a_copy_of_its_parents`
      (sets the parent's name to a sentinel value, asserts the child's name
      isn't it).
- [x] Every species-construction site draws a name the same way:
      `worldgen::generate_starting_palette`, `worldgen::add_bonus_species`,
      `input.rs::apply_splice`'s new-species push (`input.rs:534`) — no
      site left using the old id-indexed scheme.
- [x] `render::species_label` (or its replacement) reads the stored name
      instead of indexing `SPECIES_NAMES` by `id.0 % 16` — signature
      necessarily changes to take `&Species` or `&SimWorld` alongside
      `SpeciesId` (every current call site already has a `world`/`&SimWorld`
      in scope, confirmed by inspection — this is a mechanical signature
      change, not a new dependency to thread through).

      `species_label(world: &SimWorld, id: SpeciesId) -> String` now reads
      `world.species[id.0].name`. Every call site updated (`input.rs`,
      `notebook.rs`, `ui.rs`) — two functions (`tally_births`,
      `objective_panel`) needed a new `&SimWorld`/`Res<SimWorld>` param
      since they didn't already have one in scope.
- [x] The fixed 16-name `SPECIES_NAMES` pool is expanded meaningfully (exact
      count is implementer's call, but 16 repeating names across a run that
      can exceed 16 species via `Splice` reads as *more* repetitive once
      names are supposedly "always different" — widen the pool so
      repetition is rare in an ordinary run, not guaranteed past species
      #16).

      Expanded 16 → 49 names, moved from `render.rs` to `world.rs`
      alongside `draw_species_name` (it's generation content now, not a
      rendering concern — `render::species_label` is just a thin formatter
      over the stored name).
- [x] A new function generates a short natural-language description from a
      species' existing readable fields (metabolism, temp_optimum/
      tolerance → the existing `temperature_label` band, repro_threshold) —
      e.g. "A photolithic species thriving in temperate light, reproducing
      once well-fed." Exact phrasing/template structure is a design pass,
      not pinned here — keep it a pure function of the species' own fields
      (deterministic, testable, no RNG needed for the phrasing itself
      unless the implementer chooses to vary phrasing too, in which case
      that draw follows the same world-RNG discipline as the name).

      `text::species_description(metabolism, temp_label, repro_threshold) ->
      String`, e.g. "A temperate-adapted species that draws its energy from
      light, reproducing once its energy reaches 10.0." No RNG — phrasing
      stays fixed per metabolism, only the plugged-in values vary. Unit
      test: `species_description_mentions_the_right_diet_per_metabolism`.
- [x] The description is surfaced somewhere in the notebook's species
      catalog (`catalog_panel`/`species_catalog_line` call site,
      `notebook.rs`) alongside the existing precise stat line — additive,
      not a replacement (`Splice`'s exact-value math still needs the
      numbers visible).

      Added as a `ui.weak(...)` line right below `species_catalog_line`'s
      existing stat line, inside a new `ui.vertical(...)` grouping so the
      two lines read as one entry per species.
- [x] `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test` clean —
      expect several existing tests that construct `Species { ... }`
      literals directly to need the new field added; this is expected
      churn, not a design problem.

      19 `Species { ... }` literals across `input.rs`/`objectives.rs`/
      `sim.rs`/`world.rs`/`worldgen.rs` needed the new field; three test
      helper `App`s (`app_for_record_events`, `app_for_tally_births`,
      `world_with_one_taggable_species`) needed dummy `world.species`
      entries added too, since `species_label` now indexes into that list
      instead of computing from the bare id — these didn't previously need
      real species to exist for a `SpeciesId` to be valid.
- [x] Verified live via `cargo run`: reseeding the same world (`R`) with a
      different seed changes species names; the notebook catalog shows a
      readable description per species alongside its stat line.

      Confirmed by the user (2026-08-10): "funziona." Follow-up noted: the
      description reads as flat/repetitive across species (the phrasing is
      fixed per metabolism, only the plugged-in numbers vary) — a real
      finding, deliberately not addressed here; revisit in a future task if
      picked up.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `Species` struct (`~81-89`) — new name field; `draw_species_tags` — the existing per-species-creation RNG-draw pattern to mirror. |
| `src/render.rs` | `SPECIES_NAMES` (`~31-34`), `species_label` (`~40-43`) — replace the id-indexed scheme. |
| `src/worldgen.rs` | `generate_starting_palette` (`~154`), `add_bonus_species` (`~176`) — species construction sites needing a name draw. |
| `src/input.rs` | `apply_splice`'s new-species push (`~534`) — third construction site. |
| `src/text.rs` | `species_catalog_line` (`~433-444`) — existing stat line, kept; new description function lives here or in a new dedicated function. |
| `src/notebook.rs` | `catalog_panel`/species catalog rendering (`~776+`) — where the new description gets displayed. |

---

## 🧩 Technical Context

**Current behavior**: `species_label(id) = format!("{name} (species {})",
id.0)` where `name = SPECIES_NAMES[id.0 as usize % 16]` — a pure function
of the numeric id, identical across every world/seed. `species_catalog_line`
is the only descriptive text per species, a dense stat string.

**Desired behavior**: name is a per-species, per-world-random draw made
once at creation and stored; a second, readable description function turns
the same stats into a sentence, shown alongside the existing precise line.

`draw_species_tags` (`world.rs`, called from every species-construction
site already) is the existing reference for "draw something from
`world.rng_mut()` at species creation" — the name draw should follow the
same call-site pattern (called from the same three construction sites,
using the same RNG stream, not a separate derived stream — names carry no
GDD §11 secrecy constraint, unlike tags/matrix, so there's no reason to
decorrelate them into their own RNG offset the way terrain/toxic-zone/
heat-source generation does).

---

## ⚠️ Constraints and Caveats

- **Style**: no magic numbers — if description templates need tunable
  thresholds/wording beyond what `temperature_label`'s existing bands
  already provide, keep them as plain Rust constants/match arms in
  `text.rs` (this is presentation copy, not a `SimConfig` coefficient —
  compare to how `text.rs`'s other message functions already work, no
  config-file plumbing needed for wording).
- **Determinism**: the name draw must come from `world.rng_mut()`, never
  `rand::rng()` — TECH_DESIGN.md invariant 1, same rule every other
  worldgen draw in this codebase already follows.
- Don't change `SpeciesId`'s role as the stable index into `world.species`
  — only the *display name* stops being derived from it.

---

## 🔗 Dependencies

- **Depends on**: 029 (original `species_label`, being replaced here).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/095-procedural-species-names-and-descriptions.md)"$'\n\nExecute this task in the current project.'
```
