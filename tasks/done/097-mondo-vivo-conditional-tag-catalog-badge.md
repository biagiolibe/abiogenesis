# Task 097 — Conditional tag catalog badge + (tag, terrain) evidence track

> **ID**: `097`
> **Category**: Feature / UI / Notebook
> **Priority**: 🟡 P2
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-living-world.md`, §1 "Where it surfaces in the UI")

---

## 🎯 Objective

Per the design doc's decision: once confirmed, a conditional tag's terrain
requirement is a **catalog** fact (an "anchor," in GDD §7's own language for
metabolism/temperature), not something drawn into the hypothesis grid's
edge/node grammar. Add a small terrain-glyph badge next to a conditional
tag in the catalog panel (`catalog_panel`, `src/notebook.rs:809`),
distinguishing `Inducible` ("turns on here") from `Repressible` ("turns off
here") with a visually distinct treatment.

The badge's reveal timing needs its own evidence track: the doc explicitly
says this reuses `MatrixKnowledge`'s exact evidence-weighting shape
(`accumulate_evidence`, `weight = observation_weight_numerator / (1 + n_confounders)`)
but keyed on `(tag, terrain)` instead of `(tagA, tagB)` — a small, parallel
resource, not a rework of `MatrixKnowledge` itself. This task owns building
that track, since the badge's reveal timing depends on it.

---

## 📋 Acceptance Criteria

- [ ] A new resource, e.g. `TerrainKnowledge`, mirroring `MatrixKnowledge`'s
      shape (`notebook.rs:137-182`): `size = active_tags.len() * 4`
      (`TerrainKind`'s 4 variants), `record(tag: TagSlot, terrain: TerrainKind, weight: f32) -> bool`,
      `is_confirmed`, `evidence`. Constructed in `NotebookPlugin::build`
      (`notebook.rs:197-201`) from `SimConfig` alone, same trick
      `MatrixKnowledge::new` uses — no dependency on `SimWorld` existing yet.
- [ ] A new event, parallel to `AdjacencyObserved` (`src/sim.rs`'s
      `TickEvents`, emitted around `sim.rs:301-307`), emitted from task 096's
      `interaction_delta` gate check: whenever a conditional tag's gate is
      *evaluated* for an organism this tick (regardless of pass/fail — a
      failed gate is itself evidence about the terrain's exclusion, per
      Repressible's "turns off here" framing), push an event carrying the
      tag, the cell's terrain, and whether the gate passed.
- [ ] A new system (parallel to `accumulate_evidence`, `notebook.rs:233-259`)
      draining that event into `TerrainKnowledge`, confirming a `(tag,
      terrain, mode)` fact once its evidence crosses
      `config.notebook.confirmation_threshold` (reuse the existing
      threshold field, no new config value needed unless a case for a
      separate threshold emerges during implementation).
- [ ] New `text.rs` message for the reveal's log entry (mirroring
      `confirmation_message`'s shape) — only needed if task 099's zone-entry
      reveal doesn't already cover this task's log-entry needs; check task
      099's implementation before adding a duplicate.
- [ ] `catalog_panel` (`notebook.rs:809-843`): for each species' tag row
      (the `for &slot in &species.tags` loop, `notebook.rs:837-840`), if
      that tag is conditional (task 096's data) **and** `TerrainKnowledge`
      reports it confirmed, render a small badge next to the tag glyph:
      a terrain glyph (reuse or add a terrain-glyph lookup, similar to
      `tag_glyph`/`metabolism_glyph`, `render.rs:61-67`) plus a directional
      marker distinguishing inducible from repressible (first-pass
      iconography choice for this task — e.g. an up-caret for "turns on
      here" vs. a down-caret or crossed-out glyph for "turns off here";
      exact glyphs are this task's call, not prescribed further by the
      source doc).
- [ ] An unconfirmed conditional tag shows no badge (no leak of the
      terrain/mode before the evidence threshold is crossed) — this is the
      load-bearing legibility rule, matching how `MatrixKnowledge` never
      reveals a pair's value before confirmation.
- [ ] Unit test: `TerrainKnowledge::record` crosses the threshold and
      reports newly-confirmed exactly once, same shape as
      `accumulate_evidence_applies_the_confounder_weight`
      (`notebook.rs:1094`).
- [ ] Unit test: an unconditional tag never accumulates `TerrainKnowledge`
      evidence (no event emitted for it at all — verify at the emission
      site in task 096's gate, not just the accumulation system).
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: seed a species with a conditional tag,
      let it accumulate enough terrain exposure to cross the confirmation
      threshold, confirm the catalog panel's badge appears next to that
      tag and correctly shows inducible vs. repressible.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `MatrixKnowledge` (137-182, shape to mirror), `NotebookPlugin::build` (184-226, construction site), `accumulate_evidence` (233-259, system to mirror), `catalog_panel` (809-843, badge render site). |
| `src/sim.rs` | `TickEvents`/`AdjacencyObserved` emission (301-307) — parallel event to add from task 096's gate check. |
| `src/render.rs` | `metabolism_glyph` (61-67) — precedent for a small per-row icon lookup function; `tag_glyph`/`tag_color` (used throughout `catalog_panel`) — pattern to extend for a terrain-glyph lookup. |
| `src/config.rs` | `NotebookConfig` (362-373) — reuse `confirmation_threshold`/`observation_weight_numerator`, no new fields expected. |
| `tasks/096-mondo-vivo-conditional-tags-core.md` | Supplies the `(TagId, TerrainKind, Mode)` per-world data this task reads — must land first. |

---

## 🧩 Technical Context

- **Current behavior**: `catalog_panel` renders each species' tags as a flat
  glyph row (`notebook.rs:837-840`), no per-tag metadata beyond glyph/color.
  `MatrixKnowledge` is the only evidence-accumulation resource; it's keyed
  on `(exerter TagSlot, receiver TagSlot)` pairs, sized `active_tags^2`.
- **Desired behavior**: a parallel, much smaller evidence track keyed on
  `(TagSlot, TerrainKind)` — sized `active_tags * 4` — confirms per-tag
  terrain facts independently of the tag-pair matrix, surfaced as a catalog
  badge once confirmed.
- **Why a separate track, not a `MatrixKnowledge` rework**: the source doc
  is explicit that conditionality is a *per-tag* fact ("does conditional
  glyph G require/exclude which terrain"), much smaller than the *per-pair*
  matrix hypothesis space — reusing `MatrixKnowledge`'s exact shape but with
  a different key keeps the two hypothesis spaces additive and independently
  sized, rather than inflating the existing per-pair space with a terrain
  axis it was never designed to carry (the doc explicitly rejects a
  "separate matrix per biome" for exactly this reason).

---

## 🔨 Suggested Implementation

1. Land task 096 and task 103 (notebook catalog cleanup, see Dependencies)
   first.
2. `src/notebook.rs`: add `TerrainKnowledge` mirroring `MatrixKnowledge`,
   constructed in `NotebookPlugin::build` alongside the existing
   `MatrixKnowledge::new` call.
3. `src/sim.rs`: emit a new event type from task 096's `interaction_delta`
   gate check (wherever the gate is evaluated, both pass and fail cases).
4. `src/notebook.rs`: add a system draining that event into
   `TerrainKnowledge`, registered in `NotebookPlugin::build`'s system tuple
   alongside `accumulate_evidence`.
5. `src/render.rs` or `src/notebook.rs`: add a terrain-glyph lookup and
   inducible/repressible badge markers.
6. `catalog_panel`: extend the tag-row loop to check conditionality +
   confirmation, render the badge.
7. Unit tests per Acceptance Criteria.
8. `cargo run` live verification.
9. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.

---

## ⚠️ Constraints and Caveats

- **No leak before confirmation**: the badge must never appear (or hint at
  inducible vs. repressible) before `TerrainKnowledge` reports the fact
  confirmed — this is the same discovery discipline `MatrixKnowledge`
  already enforces for tag pairs (GDD §11).
- **Not drawn into the hypothesis grid**: this is explicitly a catalog-only
  surface per the doc's decision — do not add terrain nodes/edges to the
  existing grid grammar (`notebook.rs`'s hypothesis-grid rendering, ~lines
  519-729).
- **Reuse, don't rework `MatrixKnowledge`**: `TerrainKnowledge` is a new,
  small, parallel resource — do not change `MatrixKnowledge`'s key shape or
  size to accommodate terrain.
- **Iconography is this task's first-pass call**: exact glyphs for
  inducible vs. repressible aren't prescribed by the source doc; pick
  something legible, document the choice in a comment, expect it to be
  revisited during the notebook UX redesign (task 103 and beyond).

---

## 🔗 Dependencies

- **Depends on**: 096 (conditional-tag data model), **103** (notebook
  catalog cleanup, scoped from `redesign/abiogenesis-notebook-redesign.md`
  in a parallel scoping pass this session — not yet read in full here, cited
  by number/title only). Both tasks touch `catalog_panel`'s species/tag
  rows; 103 should land first so this task builds on the cleaned-up catalog
  structure rather than adding a badge to the current cluttered rows that
  103 is expected to simplify.
- **Blocks**: none.
- **Related**: 054 (confirmation log pattern, if a new log message is
  needed beyond what 099 already emits), 065 (per-row icon badge precedent
  via `metabolism_glyph`).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/097-mondo-vivo-conditional-tag-catalog-badge.md)"$'\n\nExecute this task in the current project.'
```
