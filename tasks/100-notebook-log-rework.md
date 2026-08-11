# Task 100 — Strip raw per-tick noise from the observation log

> **ID**: `100`
> **Category**: UX / Notebook
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-notebook-redesign.md`)

---

## 🎯 Objective

`player_guide.md` promises the observation log is "a curated feed of what
mattered." Live screenshot review (documented in the redesign doc) found the
opposite: the same tag pair logs a separate line **per tick** it's observed
("Era 3: ζ→ε observed" repeated 6+ times back to back) — raw telemetry, not
curation. This is `accumulate_evidence`'s per-`AdjacencyObserved` push of
`text::observation_message` (`src/notebook.rs:244-249`), driven by
`text::observation_message` (`src/text.rs:416-418`).

Remove this per-observation log line entirely. The log keeps only narrative
events: species births/deaths/extinctions (`record_events`, `tally_births` —
unchanged), and matrix-confirmation `★` moments (task 054's pattern, also
unchanged — `accumulate_evidence`'s *second* push, `src/notebook.rs:254-263`,
stays exactly as it is). Evidence accumulation remains fully tracked in
`MatrixKnowledge` and rendered on the hypothesis grid (task 101/102's
concern) — no information is lost, it's relocated to where it's already
contextual, per the redesign doc's explicit decision.

---

## 📋 Acceptance Criteria

- [ ] `accumulate_evidence` (`src/notebook.rs:233-265`) no longer pushes a
      `LogEntry` for every `AdjacencyObserved` event — only the
      newly-confirmed branch (`text::confirmation_message`) still logs.
- [ ] `text::observation_message` (`src/text.rs:416-418`) and its doc comment
      are removed if nothing else calls it (grep to confirm no other caller
      before deleting); if kept for another reason, explain why in this
      task's outcome notes.
- [ ] `EvidenceQuality`, `evidence_quality_color`, `EVIDENCE_DOT_GLYPH`
      (`src/notebook.rs:39-58`, `447-452`, `445`) and the `LogEntry::
      evidence_quality` field become dead weight once no per-observation
      line is pushed — either remove them (preferred, since nothing else in
      the codebase reads `evidence_quality` off a `LogEntry` today per this
      session's review) or, if a future task is expected to reuse them,
      leave a one-line comment saying so instead of silently leaving unused
      code for clippy to flag.
- [ ] Existing `record_events`/`tally_births` narrative logging (extinctions,
      player-organism deaths, births-per-era, objective-advanced) is
      untouched — this task removes exactly one log source, nothing else.
- [ ] Task 054's confirmation `★` log line and HUD badge behavior
      (`NotebookHasUnseenConfirmation`) are unchanged.
- [ ] Existing tests referencing `evidence_quality` (e.g.
      `a_clean_observation_logs_an_evidence_quality_clean_entry`,
      `a_confounded_observation_logs_an_evidence_quality_confounded_entry`,
      `accumulate_evidence_applies_the_confounder_weight`,
      `non_adjacency_log_entries_carry_no_evidence_quality`, all in
      `src/notebook.rs`'s `#[cfg(test)]` module) are updated to match the new
      behavior (no log entry for a raw observation) rather than left
      asserting on removed code paths.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: play until several `AdjacencyObserved`
      events fire for the same tag pair across multiple ticks — the log does
      not grow a line per tick; it only gains a line on the tag pair's
      eventual confirmation (or not at all, if it never confirms).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `accumulate_evidence` (line 233-265) — remove the per-observation `log.entries.push`. `LogEntry`/`EvidenceQuality` (lines 32-58) — likely simplify once the log no longer carries evidence-quality dots. |
| `src/text.rs` | `observation_message` (line 416-418) — remove if unused after this change. |
| `tasks/done/054-celebrate-first-confirmed-hypothesis.md` | Reference for the confirmation `★` log line this task must not touch. |

---

## 🧩 Technical Context

- **Current behavior**: `accumulate_evidence` (`src/notebook.rs:233-265`)
  pushes two possible log entries per loop iteration over
  `observed.read()`: (1) unconditionally, a per-observation line via
  `text::observation_message`, colored by `EvidenceQuality::from_confounders`;
  (2) conditionally, on the confirmation transition, a `★` line via
  `text::confirmation_message`. The first is the noise; the second is the
  keeper.
- **Desired behavior**: only the confirmation line remains. Evidence still
  accumulates in `MatrixKnowledge` exactly as before (`knowledge.record(...)`
  is unrelated to logging and stays untouched) — this task is purely about
  what gets written to `ObservationLog`, not about the underlying evidence
  model.
- `notebook_window` (`src/notebook.rs:497-515`) renders `LogEntry` with a
  three-way match on `(species, evidence_quality)` to pick a glyph. Once no
  entry ever carries `Some(evidence_quality)`, that match arm becomes dead —
  simplify it to the two remaining cases (`Some(species)` → species swatch,
  `None` → confirmation glyph) rather than leaving an unreachable arm for
  clippy to complain about.

---

## 🔨 Suggested Implementation

1. In `src/notebook.rs::accumulate_evidence`, delete the first
   `log.entries.push(LogEntry { ... text: text::observation_message(...) ... })`
   block; keep the `weight`/`knowledge.record`/confirmation block as-is.
2. Simplify `LogEntry`/`EvidenceQuality` and the rendering match in
   `notebook_window` accordingly — remove now-dead code rather than leaving
   it for clippy's `dead_code` lint to flag.
3. Remove `text::observation_message` from `src/text.rs` if grep confirms no
   remaining caller.
4. Update/remove the notebook tests that assert on `evidence_quality` values
   for logged entries, per the acceptance criteria above.
5. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
6. `cargo run`: play a few eras, confirm the log stays curated (no per-tick
   repeats) and the confirmation `★` line still appears when a pair crosses
   threshold.

---

## ⚠️ Constraints and Caveats

- Do not touch `record_events` or `tally_births` — this task's scope is
  exactly the `accumulate_evidence` per-observation line, nothing else in
  the log pipeline.
- Do not weaken or remove `MatrixKnowledge`'s evidence accumulation itself —
  only the log line tied to a *raw* observation goes away; the hypothesis
  grid (task 101/102) still needs `knowledge.evidence(...)` to keep working
  exactly as it does today.
- If `EvidenceQuality`/`evidence_quality_color` turn out to have a use
  elsewhere (e.g. a future grid-side rendering need), don't delete them
  blind — grep first, and if kept, note why in this file rather than
  leaving unexplained unused-looking code.

---

## 🔗 Dependencies

- **Depends on**: none — standalone.
- **Related, not a hard dependency**: task 101 (grid visibility/layout) and
  task 102 (grid edge grammar) touch the same notebook redesign track and
  pair naturally with this one landing in the same session, but neither
  blocks nor is blocked by this task.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/100-notebook-log-rework.md)"$'\n\nExecute this task in the current project.'
```
