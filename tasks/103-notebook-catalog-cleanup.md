# Task 103 — Catalog: one-time metabolism legend, trimmed species rows

> **ID**: `103`
> **Category**: UX / Notebook
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-notebook-redesign.md`)

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
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: open the notebook with several species
      of the same metabolism present — the descriptive sentence appears
      once in a legend, not once per matching species row; each row still
      shows its own specific numbers.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `catalog_panel` (line 809-843) — add the legend section, trim the per-row description call. |
| `src/text.rs` | `species_description` (line 472-485), `species_catalog_line` (452-463), `HEADING_CATALOG`/`ACTIVE_TAGS_LABEL`/`SPECIES_HEADING` (448-450) — new legend-line constant/function belongs alongside these. |
| `src/render.rs` | `metabolism_glyph` (line 61-67) — existing per-metabolism icon, reused for the legend exactly as the sidebar Species list already uses it (task 065). |

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
5. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
6. `cargo run`: verify the legend renders once and rows are trimmed.

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
