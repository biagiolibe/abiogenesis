# Task 170 — Speciation cause readability: dominant stimulus + genome diff

> **ID**: `170`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~1.5h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

The end-of-era reveal already names *why* a speciation happened (task 142:
`sim::DominantStimulus` is `pub`, `EraEvolutionReveal::dominant_stimulus` is
set, `text::era_reveal_evolution_line` appends a one-clause cause sentence at
the three-stimulus category level — "worn down by sustained harm from a
neighbouring species", etc.). What it still doesn't show is **what actually
changed in the genome**: `EraEvolutionReveal` only carries
`parent_tag_count`/`child_tag_count` (a bare count, before/after), even
though `sim::speciate` computes the exact, concrete edit — which tag was
added, by how much `temp_optimum` shifted (and in which direction), or that
`Sea` placement tolerance was granted — and simply discards that detail once
the edit is applied. Surface the **specific edit**, not just a trait-count
delta, on the reveal card.

Design source: GDD §5.11 (`abiogenesis-gdd.md:300-313`), `[DECIDED, tasks
106-109]`. No `redesign/processed/` document is authoritative here — §5.11 is
GDD prose already decided, and 142's own doc comment
(`src/text.rs:155-159`) explicitly reasons that a *more specific
per-neighbour* cause clause can't be built ("`SelectionThresholdCrossed`
accumulates pressure as scalars, not 'which neighbour/terrain contributed
most'... would have to invent detail the sim doesn't actually track"). That
reasoning stands and isn't reopened here — the category-level clause stays.
What *is* tracked, and currently thrown away, is the concrete genome edit
`speciate` itself performs; that's this task's actual scope.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] New type capturing the concrete edit `sim::speciate` applies (mirror
      its three branches, `src/sim.rs:491-518`):
      - `InteractionHarm` → the `TagSlot` that was added.
      - `TerrainMismatch` → old and new `temp_optimum` (or just the signed
        delta and direction — pick whichever text.rs can phrase without
        raw numbers, see below).
      - `Toxicity` → the fact that `Sea` placement tolerance was granted.
      Attach it to `EraEvolutionReveal` (`src/sim.rs:578-591`) alongside the
      existing `dominant_stimulus`, populated at the same call site
      (`speciate`'s caller in `build_era_reveal`) so the reveal can never
      show an edit that disagrees with the one `speciate` actually made —
      same non-duplication guarantee task 142's own comment calls out for
      `dominant_stimulus`.
- [ ] `text::era_reveal_evolution_line` (or a new sibling function, if
      folding the diff into one sentence gets unreadable) renders the
      concrete edit in the same clinical, no-raw-numbers register as the
      rest of the reveal (§ pillar/`text.rs` convention: qualitative
      language, not GDD-internal floats) — e.g. name the added tag via
      `notebook::translated_tag_label` (task 144) rather than a bare glyph,
      describe the temperature shift as a direction ("became better suited
      to warmer ground") rather than printing `temp_optimum` values, name
      the sea-tolerance grant in prose.
- [ ] `screens.rs`'s reveal card (`~225-272`) displays the new line
      alongside the existing swatch + cause-clause block, same
      `ui.add_space` rhythm.
- [ ] Existing `era_reveal_evolution_line_names_a_different_cause_per_dominant_stimulus`
      test (`src/text.rs:843`) and any other text-layer test touching this
      function still pass, extended to cover the new diff content.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `speciate` (`479-533`) — already computes the concrete edit per branch; `EraEvolutionReveal` (`578-591`) — gains the new field. |
| `src/text.rs` | `era_reveal_evolution_line` (`140-153`) and `dominant_stimulus_clause` (`160-...`) — extend or add a sibling for the genome-diff sentence. |
| `src/screens.rs` | Reveal card rendering (`~225-272`) — display the new line. |
| `src/notebook.rs` | `translated_tag_label` (task 144) — reuse for naming an added tag, don't reinvent. |
| `abiogenesis-gdd.md` | §5.11 — no prose change expected (already `[DECIDED]`); confirm nothing there needs updating once the diff is concrete. |

---

## 🧩 Technical Context

- **Current behavior**: the reveal card shows `"{parent} ({N} traits) evolved
  into {child} ({M} traits), {generic cause clause}"` — a trait *count*
  delta plus a category-level reason, never the actual trait or scalar that
  changed.
- **Desired behavior**: the same line (or an adjacent one) also names the
  concrete change — which trait was gained, which direction the thermal
  optimum moved, or that sea tolerance was granted — using data `speciate`
  already computes and currently discards after applying it.
- `speciate`'s three branches are exhaustive and already mutually exclusive
  per `DominantStimulus` — no new decision logic needed, only capturing what
  each branch already decided instead of throwing it away.

---

## 🔨 Suggested Implementation

1. Define a small enum/struct, e.g. `GenomeEdit { TagAdded(TagSlot),
   ThermalShift { warmer: bool }, SeaToleranceGranted }`, mirroring
   `speciate`'s three match arms exactly (one variant per arm, no
   independent modeling).
2. Have `speciate` return the edit alongside the `SpeciesId` (or build it at
   the `build_era_reveal` call site from data `speciate` already exposes —
   whichever avoids duplicating the branch logic; likely easiest to have
   `speciate` itself return `Option<(SpeciesId, GenomeEdit)>`).
3. Add `genome_edit: GenomeEdit` to `EraEvolutionReveal`, populated where the
   struct is currently built (`sim.rs:699-708` area).
4. `text.rs`: a `genome_edit_clause(edit: &GenomeEdit, ...) -> String`
   (needs a tag-name lookup for `TagAdded`, likely the same table
   `translated_tag_label` reads from — check its signature before adding a
   second one).
5. `screens.rs`: append the new clause to the existing evolution block.
6. Extend/add tests in `text.rs`'s test module for the three edit variants.

---

## ⚠️ Constraints and Caveats

- **No raw internal numbers in player-facing text** — `temp_optimum`,
  `TagSlot` indices, etc. stay internal; translate to natural language or a
  translated tag name, same rule task 144 and 142 already followed.
- Don't reopen or extend `dominant_stimulus_clause`'s per-neighbour
  specificity — that's a deliberately closed question (142's own comment).
  This task adds a *different* piece of information (the edit itself), not
  a more specific cause.
- Keep `sim`/`world`/`config` free of `bevy::render`/`bevy_egui` — the new
  `GenomeEdit` type and its data belong in `sim.rs`; only its text rendering
  touches `text.rs`/`screens.rs`.

---

## 🔗 Dependencies

- **Depends on**: 142 (dominant-stimulus surfacing this extends), 144
  (`translated_tag_label`, reused for naming the added tag).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/170-speciation-cause-readability.md)"$'\n\nExecute this task in the current project.'
```
