# Task 103 — Catalog: one-time metabolism legend, trimmed species rows, population + origin era

> **ID**: `103`
> **Category**: UX / Notebook
> **Priority**: 🟡 P2
> **Estimate**: ~2h (extended 2026-08-12, see below)
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-notebook-redesign.md`); extended
> 2026-08-12 after reading `redesign/abiogenesis-hud-notebook.md`, which
> asks for a richer catalog card than this task originally scoped.

---

## 🎯 Objective

Live screenshot review (redesign doc) found the notebook's Catalog panel
repeating near-identical descriptive sentences per species sharing a
metabolism — e.g. every Photolithic species' row prints "A cold-adapted
species that draws its energy from light, reproducing once its energy
reaches 10.0," differing only in the temperature-band word and the
threshold number. This is text noise: no new information per row once
you've read it once for that metabolism.

Replace the per-species repeated prose (`text::species_description`, called
per row in `catalog_panel`, `src/notebook.rs:831-835`) with a **one-time
metabolism legend** shown once at the top of the catalog panel — one icon +
one line per metabolism kind present in the world, using the existing
`metabolism_glyph` precedent (`src/render.rs:61-67`, task 065). Each
species' own row is trimmed to just its specific parameters: name,
metabolism icon, temperature range, reproduction threshold, tags — no
repeated descriptive sentence.

Note: this item is flagged in the redesign doc as "raised as uncontroversial
but not yet formally confirmed by the user" — distinct from the log/grid
changes (tasks 100-102), which were explicitly decided in discussion. Still
fine to scope and build; flag this provenance for whoever picks it up.

**Extension (2026-08-12)**: `redesign/abiogenesis-hud-notebook.md` §8 asks
for two more pieces of per-species data on the catalog card beyond what this
task originally scoped: **current population** and **origin era** (the era
a species was first seeded/created in). Folded into this same task rather
than split out, since both only matter for this card and the work is small.
Origin era needs new state — nothing in the codebase tracks "the era a
species first came into being" today (`Organism::born_era` exists per
*individual*, not per `Species`) — see the extra Acceptance Criteria and
Technical Context below.

---

## 📋 Acceptance Criteria

- [ ] `catalog_panel` (`src/notebook.rs:809-843`) renders a legend section
      before the species list: one line per **metabolism kind actually
      present** among `world.species` (not all possible `Metabolism`
      variants unconditionally — if a world has no Decomposer, don't show a
      Decomposer legend line), each showing `metabolism_glyph` plus a short
      description of what that metabolism does (adapt
      `text::species_description`'s existing diet clause, e.g. "draws its
      energy from light," rather than inventing new copy).
- [ ] Each species row no longer calls `text::species_description` (or its
      per-row `ui.weak(...)` line is removed) — the row shows
      `species_catalog_line`'s stat line (name, temp range, repro threshold)
      plus its tag swatches, **and** a `metabolism_glyph` icon, per the
      brief's explicit ask ("name, metabolism icon, temp range, repro
      threshold, tags"). `species_catalog_line` currently prints
      `{metabolism:?}` as a Debug-formatted word (`src/text.rs:452-463`,
      e.g. "Photolithic") with no icon anywhere on the row today — add the
      `metabolism_glyph(species.metabolism)` icon to the row (e.g. rendered
      via `ui.colored_label`/`ui.label` alongside the stat line, the same
      way the sidebar Species list already pairs glyph + text per task 065)
      rather than assuming the row is otherwise untouched.
- [ ] The per-species reproduction threshold shown on the row must be
      `species.repro_threshold` (the actual per-species value, which
      `Splice` can diverge from the config default), not
      `config.energy.repro_threshold`. Today `species_catalog_line`'s call
      site (`src/notebook.rs:829`) passes `config.energy.repro_threshold`
      while the (now-removed) `species_description` call
      (`src/notebook.rs:834`) was the only place actually passing
      `species.repro_threshold` — deleting `species_description` without
      fixing the `species_catalog_line` call site would silently drop the
      only display of the correct per-species value. Fix the
      `species_catalog_line` call to pass `species.repro_threshold` instead.
- [ ] The temperature-band word (cold/temperate/hot) and reproduction
      threshold stay visible per-species (they're specific, not boilerplate)
      — only the repeated diet sentence structure moves to the legend.
- [ ] New legend copy lives in `src/text.rs`, following the existing
      `HEADING_CATALOG`/`ACTIVE_TAGS_LABEL` constant pattern, not inlined as
      a literal string in `notebook.rs`.
- [ ] If `text::species_description` becomes unused after this change (grep
      to confirm no other caller), remove it and its test
      (`species_description_mentions_the_right_diet_per_metabolism`,
      `src/text.rs`'s test module) or repurpose the test to cover the new
      legend-line function instead of deleting coverage outright.
- [ ] **(Extension)** Each species row shows its **current population**
      (count of `world.cells` whose `organism.species` matches this
      `SpeciesId`) — computed on demand in `catalog_panel` per row, no new
      state needed (this is a cheap per-render scan over the grid, not a
      per-tick tracked value).
- [ ] **(Extension)** Each species row shows its **origin era** (the era it
      was first seeded/created in). This needs new state — `Organism::
      born_era` exists per *individual*, not per `Species`, so there is
      currently nothing to read here. Recommended shape, following task
      098/099's precedent (`SimWorld::wild_species`/`terrain_occupancy`,
      small `Vec`s indexed by `SpeciesId`, kept separate from `Species`
      itself specifically to avoid touching every one of the ~20
      `Species { .. }` construction sites across the codebase): add
      `SimWorld::species_origin_era: Vec<u32>` and a small
      `SimWorld::push_species(species: Species) -> SpeciesId` helper that
      both pushes onto `world.species` and records `world.era` into
      `species_origin_era` in the same place, then migrate every
      `world.species.push(Species { .. })` call site (worldgen.rs's three
      generators, `input.rs::apply_splice`, plus every test call site) to go
      through it. Unlike `wild_species`/`terrain_occupancy`, this **does**
      require touching existing call sites (there's no way to reconstruct
      "the era this already-pushed species was created in" after the fact
      without recording it at push time) — budget time for that, and prefer
      the single-helper approach over duplicating `world.era` capture at
      every site by hand (error-prone, easy to miss one).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: open the notebook with several species
      of the same metabolism present — the descriptive sentence appears
      once in a legend, not once per matching species row; each row still
      shows its own specific numbers, including current population and
      origin era; seed a species mid-run (not world start) and confirm its
      row shows the era it was actually seeded in, not `0`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `catalog_panel` (line 809-843) — add the legend section, trim the per-row description call. |
| `src/text.rs` | `species_description` (line 472-485), `species_catalog_line` (452-463), `HEADING_CATALOG`/`ACTIVE_TAGS_LABEL`/`SPECIES_HEADING` (448-450) — new legend-line constant/function belongs alongside these. |
| `src/render.rs` | `metabolism_glyph` (line 61-67) — existing per-metabolism icon, reused for the legend exactly as the sidebar Species list already uses it (task 065). |
| `src/world.rs` | **(Extension)** `SimWorld` — add `species_origin_era: Vec<u32>` and a `push_species` helper (see Acceptance Criteria); `Organism::born_era` is the existing per-individual precedent for the field's naming/shape. |
| `src/worldgen.rs`, `src/input.rs` | **(Extension)** `generate_starting_palette`/`add_bonus_species`/`place_wild_species` (worldgen.rs) and `apply_splice` (input.rs) — every non-test `world.species.push(Species { .. })` call site, to migrate to `push_species`. |

---

## 🧩 Technical Context

- **Current behavior**: `catalog_panel` loops over `world.species` and, per
  species, renders `species_catalog_line` (precise stats) followed by
  `ui.weak(text::species_description(...))` — a full sentence repeated
  verbatim in structure for every species sharing a metabolism, differing
  only in the interpolated temp-band word and threshold number.
- **Desired behavior**: the diet/behavior sentence is said once per
  metabolism kind present, in a legend row at the top of the panel (icon +
  sentence); each species row keeps only its own specific numbers.
- `metabolism_glyph` (`src/render.rs:61-67`) already exists and is used by
  the sidebar's Species list (task 065) — this task's legend should use the
  same glyphs, not invent new icons, so the visual language stays consistent
  between the HUD sidebar and the notebook catalog.
- Determining which metabolism kinds are "present" needs a single pass over
  `world.species` collecting distinct `Metabolism` values before rendering
  the legend — `Metabolism` likely already derives `PartialEq`/`Eq` (check
  `src/world.rs`); a small `Vec`/fixed-size dedup (not a `HashSet`, per
  `CLAUDE.md`'s no-`HashMap`/`HashSet`-iteration rule — a 3-variant enum has
  at most 3 distinct values, a linear `contains` check on a `Vec` is simpler
  and cheap enough) is enough.
- **(Extension)** Current population needs no new state: a per-row scan of
  `world.cells.iter().filter(|c| c.organism.is_some_and(|o| o.species ==
  id)).count()` at render time is cheap (grid is small, catalog only
  renders while the notebook window is open, not every simulation tick).
  Origin era, by contrast, cannot be computed after the fact — it must be
  captured at the moment a species is created, hence the `push_species`
  helper above.

---

## 🔨 Suggested Implementation

1. Add a `text.rs` function, e.g. `metabolism_legend_line(metabolism:
   Metabolism) -> String`, extracting the diet clause currently embedded in
   `species_description` (light/prey/residue) into its own short sentence
   not tied to a specific species' temp band or threshold.
2. In `catalog_panel`, before the species loop, collect the distinct
   `Metabolism` values present in `world.species` (in first-seen order is
   fine — determinism only matters for simulation state, not UI row order,
   per `CLAUDE.md`'s scope for that rule) and render one legend line per
   value: `metabolism_glyph(m)` + `metabolism_legend_line(m)`.
3. Remove the per-row `ui.weak(text::species_description(...))` call. Add a
   `metabolism_glyph(species.metabolism)` label to the row (near the
   existing species-color swatch, `src/notebook.rs:821`). Fix the
   `species_catalog_line` call (`src/notebook.rs:823-830`) to pass
   `species.repro_threshold` instead of `config.energy.repro_threshold`, so
   the row keeps showing each species' actual current threshold (which
   `Splice` can have shifted away from the config default) now that
   `species_description`'s call — the only other place passing the correct
   per-species value — is gone.
4. Decide `species_description`'s fate (remove vs. repurpose its test) per
   the acceptance criteria.
5. **(Extension)** Add `SimWorld::species_origin_era`/`push_species` (see
   Acceptance Criteria), migrate every non-test `world.species.push(..)`
   call site to it, and add the current-population scan + origin-era read to
   the catalog row.
6. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
7. `cargo run`: verify the legend renders once, rows are trimmed, and
   population/origin era are correct (including for a species seeded
   mid-run, not just world start).

---

## ⚠️ Constraints and Caveats

- This item's provenance: the redesign doc marks catalog cleanup as "raised
  as uncontroversial but not yet formally confirmed by the user" — unlike
  the log/grid changes (tasks 100-102), which came out of explicit
  discussion. Still safe to build as scoped here, but worth a quick
  double-check with the user if anything about the split (what counts as
  "legend" vs. "per-row specific") feels ambiguous during implementation.
- No `HashMap`/`HashSet` iteration for the dedup step — see Technical
  Context; a `Vec`-based linear check is sufficient given at most 3
  `Metabolism` variants.
- Don't touch the tag-swatch rendering at the end of each species row, or
  the `Active tags` legend above the species list (`ACTIVE_TAGS_LABEL`) —
  out of scope, unrelated to the metabolism-description duplication this
  task fixes.

---

## 🔗 Dependencies

- **Depends on**: 065 (`metabolism_glyph` origin), 095 (`species_description`
  origin, the text this task deduplicates).
- **Related, not a dependency**: this is independent of tasks 100-102 (log
  and grid changes) — no shared code path, safe to land in any order
  relative to them.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/103-notebook-catalog-cleanup.md)"$'\n\nExecute this task in the current project.'
```
