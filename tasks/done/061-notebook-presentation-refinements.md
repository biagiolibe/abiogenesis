# Task 061 — Notebook presentation refinements (evidence-quality log, graph polish, catalog color)

> **ID**: `061`
> **Category**: UI
> **Priority**: 🟢 P3
> **Estimate**: ~3-4h (three independent presentation-only changes, bundled because each is individually small and zero gameplay risk)
> **Assigned to**: unassigned
> **Session**: 2026-08-08 (from reviewing `abiogenesis-ui-redesign.md` against the current codebase)

---

## 🎯 Objective

Reviewing `abiogenesis-ui-redesign.md` against the current implementation
found that most of it (hypothesis graph, per-metabolism shapes, toxic-zone
tint, shared species color) already shipped in tasks 031/032/033. Three
small, purely presentational gaps remain, each additive to existing code and
carrying zero gameplay/simulation risk:

1. **Observation-log entries don't show evidence quality.** The GDD (§7)
   makes "an isolated observation is worth more" central
   (`weight = observation_weight_numerator / (1 + n_confounders)`,
   `src/notebook.rs:201-202`), but nothing in the log currently reflects it.
   **Decided direction** (confirmed directly with the user, overriding the
   original UI-redesign note's assumption that log lines already exist
   per-observation): every `AdjacencyObserved` event gets its **own** log
   line, not just the ones that push a pair over the confirmation threshold.
   Each line shows a colored dot for that specific observation's quality —
   green (`n_confounders == 0`, weight 1.0) vs. amber (`n_confounders >= 1`,
   reduced weight). This is a deliberate reversal of `record_events`' current
   "curated, not a per-tick feed" filtering philosophy
   (`src/notebook.rs:237-239`) for adjacency observations specifically —
   accept that the log gets noisier in exchange for making the isolation
   principle visible by reinforcement, not just in the player guide's prose.
2. **The hypothesis graph (`hypothesis_grid`, `src/notebook.rs:428`, already
   a circular node graph since task 031) is missing three refinements** the
   redesign notes call out: a visual marker for tags with zero observations
   at all, edge thickness tied to confirmed effect magnitude, and a numeric
   label on strong confirmed edges.
3. **The notebook's catalog panel (`catalog_panel`, `src/notebook.rs:592`)
   is the one screen that doesn't reuse `species_color()`**
   (`src/render.rs:55`), the shared source the map, HUD, and observation log
   already use — species are listed with tag glyphs only, no species-color
   swatch.

---

## 📋 Acceptance Criteria

### 1. Per-observation evidence-quality log line

- [x] `accumulate_evidence` (`src/notebook.rs:192-216`) pushes a `LogEntry`
      for **every** `AdjacencyObserved` event it reads, not only ones that
      trigger a new confirmation. The existing confirmation `LogEntry` (line
      208-212, `species: None`, confirmation glyph) stays as-is for the
      "aha" moment; the new per-observation entry is a separate, additional
      line logged first.
- [x] `LogEntry` (`src/notebook.rs:29-33`) gains a field to carry the
      observation's quality — e.g. `evidence_quality: Option<EvidenceQuality>`
      (`enum EvidenceQuality { Clean, Confounded }`, derived from
      `event.n_confounders == 0`), `None` for every non-adjacency entry kind
      (deaths, extinctions, confirmations, species-created) so they render
      unchanged.
- [x] `notebook_window`'s log loop (`src/notebook.rs:357-369`) renders a
      small colored dot before the line when `evidence_quality.is_some()`:
      green for `Clean`, amber for `Confounded` — reuse `EDGE_POSITIVE_COLOR`-
      style constants or define new ones near `PARTIAL_EVIDENCE_COLOR`
      (`src/notebook.rs:403`) for consistency.
- [x] The new per-observation line's text is short and doesn't duplicate the
      confirmation message — e.g. "`{exerter glyph} -> {receiver glyph}
      observed`" via a new `text.rs` helper (task 034's convention), not an
      inline `format!` in `notebook.rs`.
- [x] Verify in a manual playtest (or a test asserting log length) that a
      single tick with several simultaneous adjacencies doesn't make the log
      unreadably noisy — `stick_to_bottom` (already set, line 351) should
      keep the latest entries visible; if volume turns out to be a real
      problem, note it as a follow-up rather than silently reverting the
      per-observation logging decision.
- [x] New/updated tests in `notebook.rs`'s test module covering: a clean
      observation (`n_confounders == 0`) logs `Clean`, a confounded one logs
      `Confounded`, and non-adjacency log entries still have
      `evidence_quality: None`.

### 2. Hypothesis graph refinements

- [x] **Dashed border for zero-evidence nodes**: a tag node renders with a
      dashed circle outline (instead of, or in addition to, the current
      solid `painter.circle_filled`, `src/notebook.rs:466`) when it has zero
      evidence in *every* direction — i.e. `knowledge.evidence(slot, other)
      == 0.0 && knowledge.evidence(other, slot) == 0.0` for every other
      active tag. egui's `Painter` has no built-in dashed-circle primitive;
      approximate with a series of short arcs/line segments, or accept a
      simpler "thin gray outline ring" if a true dash pattern is impractical
      in the time budget — note in the task's completion notes which one was
      chosen and why.
- [x] **Edge thickness by magnitude**: `draw_edge` (`src/notebook.rs:487`)
      takes a `color` parameter today; extend it (or add a variant) to also
      take the confirmed value's magnitude (`1` or `2`, from
      `world.matrix.get(exerter, receiver).abs()`) and scale stroke width
      accordingly — e.g. `1.5` for magnitude 1, `3.0` for magnitude 2 (tune
      by eye, no config needed, this is pure presentation).
- [x] **Numeric label on strong (magnitude 2) confirmed edges only**: draw
      the signed value (`+2`/`-2`) near the edge midpoint for those edges;
      magnitude-1 edges stay unlabeled, matching the redesign note's intent
      to avoid clutter.
- [x] Existing `hypothesis_grid`/`draw_edge`/`draw_partial_marker` tests (if
      any exist beyond the module's evidence-accumulation tests) still pass;
      add a test only if the magnitude-lookup logic is non-trivial enough to
      warrant one in isolation from egui rendering.

### 3. Catalog species-color swatch

- [x] `catalog_panel` (`src/notebook.rs:592-617`)'s per-species
      `ui.horizontal` block (line 603) gets a `species_color(SpeciesId(id as
      u8))`-colored swatch (matching the pattern already used for the
      observation log's species glyph, `src/notebook.rs:361`) before the
      existing `species_catalog_line` label.

### General

- [x] `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt
      -- --check` all clean.
- [x] `PROJECT_PLAN.md`'s "Presentation refinement bundle" proposal entry
      updated/removed once this lands.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `LogEntry`, `accumulate_evidence`, `notebook_window`'s log loop, `hypothesis_grid`, `draw_edge`, `draw_partial_marker`, `catalog_panel` — every change in this task lands here. |
| `src/render.rs` | `species_color`/`species_hue` — reused, not modified, for the catalog swatch. |
| `src/sim.rs` | `AdjacencyObserved` (`n_confounders` field, ~line 69-73) — the data source for evidence quality, read-only. |
| `src/text.rs` | New helper for the per-observation log line's text (task 034's convention: player-facing strings live here). |
| `abiogenesis-ui-redesign.md` | Source proposal document. |
| `PROJECT_PLAN.md` | §1 "From `abiogenesis-ui-redesign.md`" — the proposal this task implements. |

---

## 🧩 Technical Context

- **Current behavior**: the log only ever gets entries for deaths (filtered
  to player-placed organisms or extinctions), species creation, and matrix
  confirmations — never for raw adjacency observations. The hypothesis graph
  already distinguishes confirmed (solid colored line + arrowhead) from
  partial (small gray dot) from no-evidence (nothing drawn), but with no
  distinction for a *node* that's never been touched by any observation, and
  no visual weight for confirmed effect magnitude. The catalog panel lists
  species with tag glyphs but no species-color identity.
- **Desired behavior**: described per acceptance criterion above. None of
  this changes `SimWorld`, `MatrixKnowledge`'s stored data, or any tick
  logic — every change is either a new presentation-only field on `LogEntry`
  (populated from data that already exists in `AdjacencyObserved`) or pure
  `egui`/`Painter` rendering code.

---

## ⚠️ Constraints and Caveats

- **Style**: new player-facing text goes through `text.rs` (task 034).
- **Determinism**: none of this touches simulation state or RNG — purely
  presentation, safe by construction.
- **Scope discipline**: this task is presentation-only per its source
  proposal. If implementing the per-observation log reveals it needs
  further tuning (log volume, a way to collapse repeated observations of the
  same pair, etc.), land the straightforward version first and file a
  follow-up rather than expanding this task's scope.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/061-notebook-presentation-refinements.md)"$'\n\nExecute this task in the current project.'
```
