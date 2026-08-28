# Task 144 — Temporary trait-code translation in the first N log lines

> **ID**: `144`
> **Category**: Feature
> **Priority**: 🟢 Bassa-media (Phase 1b)
> **Estimate**: ~45min
> **Assigned to**: Claude CLI
> **Session**: 2026-08-28

---

## 🎯 Objective

The Observation log's confirmation lines name bare tag glyphs the player
hasn't learned to associate with the species they seeded yet. For the
first few confirmations of a run, show which species currently carry the
glyph alongside it; drop back to the compact code-only form afterward.

Design source: `redesign/processed/culture-shock-friction-fixes.md`,
Intervento 4.

---

## 📋 Acceptance Criteria

- [x] `cargo build`/`clippy -- -D warnings` clean.
- [x] `notebook::translated_tag_label` appends every species currently
      carrying the tag (comma-joined), only when `translate` is `true`;
      bare glyph otherwise, and bare glyph if no known species carries it.
- [x] Applied in both places a bare `tag_glyph` reached the log
      (`accumulate_evidence`'s matrix confirmations,
      `accumulate_terrain_evidence`'s terrain-gate confirmations), gated on
      `log.entries.len() < FIRST_N_TRANSLATED_OBSERVATIONS` (`5`).
- [x] Unit test covering translated/untranslated/no-carrier cases.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `translated_tag_label`, `FIRST_N_TRANSLATED_OBSERVATIONS`, both `accumulate_*evidence` call sites. |

---

## 🔨 Implementation notes

This codebase's tags already render as single Greek letters (`TAG_LETTERS`),
not the design doc's illustrative three-letter codes (that rework is task
155, unimplemented) — the underlying friction (an opaque per-run code with
no name attached) is the same regardless, so the fix applies to the
existing glyph scheme rather than waiting on 155.

`log.entries.len()` (total entries of every kind, not just
confirmations) is used as the "first N observations" counter — simpler
than adding dedicated state, and close enough to the design's intent since
confirmation lines are the overwhelming majority of early-log entries in
practice.

`accumulate_evidence`'s confirmations are tag-pair-scoped, not tied to any
one organism/adjacency event by the time they reach this function (task
134's earlier extraction moved the accumulation itself to a shared,
species-agnostic pure function) — so "translation" here means "which
species in the current roster carry this tag right now," not "which two
organisms triggered this specific confirmation." A tag carried by more
than one species lists all of them.

---

## 🔗 Dependencies

- **Depends on**: 029 (`tag_glyph`), 054/097 (the two confirmation log paths).
- **Blocks**: none.
