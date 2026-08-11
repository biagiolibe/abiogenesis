# Task 102 — Hypothesis grid: edge grammar rewrite (thickness over labels, dashed partial lines, curved bidirectional arcs)

> **ID**: `102`
> **Category**: UX / Notebook
> **Priority**: 🟡 P2
> **Estimate**: ~3h (includes a visual tuning pass)
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-notebook-redesign.md`)

---

## 🎯 Objective

Live screenshot review (redesign doc) found the hypothesis grid's edge
rendering hard to read in exactly the case that matters most — an active,
bidirectional relationship: both A→B and B→A confirmed edges drew as
overlapping straight double-arrows with `±N` text labels cramped next to
each other (the ζ/ε example: "-2"/"+2" crammed together, hard to tell which
number belongs to which direction).

Three changes to `hypothesis_grid`'s edge drawing
(`src/notebook.rs:590-607`, `draw_edge` at 644-686, `draw_partial_marker` at
729-732):

1. **Confirmed edges — thickness replaces the numeric label.** Drop the
   `±N` text (`draw_edge`'s lines 676-685). Magnitude already sets stroke
   width today (`EDGE_STROKE_WEAK`/`EDGE_STROKE_STRONG`, lines 633-634,
   633-664) — keep that, and let thickness alone carry magnitude. Color
   (sign) and thickness (magnitude) fully replace the text label.
2. **Partial evidence — dashed line replaces the dot marker.** Drop
   `draw_partial_marker`'s dot (lines 729-732); draw a dashed line instead,
   neutral color (`PARTIAL_EVIDENCE_COLOR`, unchanged), uniform thickness
   (sign/magnitude genuinely aren't known pre-confirmation, so there's
   nothing to encode in thickness). This is a semantic fit, not just an
   aesthetic swap — a dashed line reads as "a relationship exists here,
   details pending," which the old lineless dot didn't convey as clearly.
3. **Bidirectional pairs — curved/offset arcs instead of overlapping
   straight lines.** When both A→B and B→A are present (confirmed or
   partial, in any combination), draw them as two arcs bowing apart from
   each other rather than two straight lines offset by a small fixed
   perpendicular amount (`EDGE_OFFSET`, currently 4.0 points — visibly not
   enough at the ζ/ε case's density). This is the part most likely to need
   real geometry work; see Suggested Implementation for the tuning
   expectation.

---

## 📋 Acceptance Criteria

- [ ] Confirmed edges no longer render a `±N` text label on the grid itself
      — magnitude there is only readable via stroke width. Because task
      031's own constraint (also stated in `node_tooltip_text`'s doc
      comment, `src/notebook.rs:734-737`) is that the graph must never be
      the *only* way to read this information, `text::confirmed_relation_line`
      (`src/text.rs:437-440`) must gain the magnitude in its output (e.g.
      `ζ → ε (+2)`, not just `ζ → ε (+)`) so the tooltip stays a complete
      textual fallback once the on-grid label is gone — this is a required
      change, not optional. Derive stroke width from
      `config.effect_intensity_max` (not a hardcoded assumption of exactly 2
      levels) so it stays correct if that config value is retuned —
      `draw_edge`'s own doc comment (lines 640-643) already documents
      `value` as `±1`/`±2` today, but the thickness mapping should scale
      with the config, not a literal `2`.
- [ ] `EDGE_STROKE_WEAK`/`EDGE_STROKE_STRONG` (lines 633-634) are re-tuned,
      not left at their current values. They were originally tuned when the
      `±N` text label was doing the real work of conveying magnitude 2;
      with the label removed, the stroke-width delta becomes the *sole*
      carrier of magnitude and the current 1.5pt gap is unlikely to be
      distinguishable at a glance. Confirm via `cargo run` that magnitude 1
      vs. magnitude 2 edges are clearly different in thickness without any
      text present, and widen the gap if not.
- [ ] Partial-evidence pairs render as a dashed line (reusing or adapting
      `draw_dashed_ring`'s segment-based dash approach if task 101 hasn't
      already deleted it — coordinate with that task's outcome, since 101
      may remove `draw_dashed_ring` as dead code if its only use was the
      now-gone node ring) in `PARTIAL_EVIDENCE_COLOR`, not a dot.
      `PARTIAL_MARKER_RADIUS`/`PARTIAL_MARKER_T`/`draw_partial_marker` are
      removed or repurposed accordingly.
- [ ] A bidirectional pair (both `knowledge.revealed_value` or
      `knowledge.evidence(...) > 0.0` true in both directions between the
      same two tags) renders as two visually distinct, non-overlapping arcs
      — each direction's arrowhead and line style (solid/dashed, color,
      thickness) independently correct for its own confirmation state (e.g.
      A→B confirmed positive + B→A still partial renders one solid green
      arc and one dashed gray arc, clearly separated).
- [ ] A unidirectional pair (only one direction has any evidence) is
      unaffected by the curve logic — still a straight line/arrow as today,
      no unnecessary curvature when there's nothing to bow away from.
- [ ] `node_tooltip_text` (`src/notebook.rs:738-780`) is updated to match:
      `text::confirmed_relation_line` gains magnitude (see above);
      `text::partial_relation_line` (`src/text.rs:442-444`) drops any
      language that implied a dot/point marker, since that marker no longer
      exists (a dashed line is what's actually drawn now) — its current
      "(some evidence)" wording already doesn't name the marker shape, so
      confirm it still reads correctly rather than assuming a rewrite is
      needed.
- [ ] A short written note in this file (or the commit message) on what
      curve/offset parameters were landed on and why, since this is
      genuinely tuned by eye — see Suggested Implementation.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: reach a bidirectional tag pair (or
      force one via a short debug seed/matrix if reaching one naturally is
      slow) and confirm both directions are legible at a glance — distinct
      arcs, correct color/thickness/dash per direction, no cramped
      overlapping labels.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `hypothesis_grid` (line 574-629, the edge-drawing double loop at 590-607), `draw_edge` (644-686), `draw_partial_marker` (729-732), `EDGE_OFFSET`/`EDGE_STROKE_WEAK`/`EDGE_STROKE_STRONG`/`PARTIAL_MARKER_*` constants (537, 633-634, 549-555). `node_tooltip_text` (738-780) — text fallback that must stay consistent with the new grammar. |
| `src/text.rs` | `confirmed_relation_line`/`partial_relation_line` (437-444) — tooltip text builders, likely need small wording updates. |
| `tasks/done/031-hypothesis-grid-as-graph.md` | Origin of the current edge-drawing code, useful for the original design intent behind `EDGE_OFFSET`. |

---

## 🧩 Technical Context

- **Current behavior**: `draw_edge` (line 644) offsets a straight line
  perpendicular to its own direction by a fixed `EDGE_OFFSET` (4.0 points)
  so that a confirmed A→B and confirmed B→A don't literally overlap — but at
  typical node spacing this reads as two nearly-parallel lines close enough
  that their `±N` labels (added at line 676-685 when magnitude ≥ 2) end up
  cramped together, per the ζ/ε live example. `draw_partial_marker` (line
  729) draws an unconnected dot at 65% of the way from exerter to receiver,
  with no line at all — a partial-evidence pair currently has *no* line
  connecting its nodes, only this dot.
- **Desired behavior**: confirmed edges use thickness, not text, for
  magnitude. Partial edges use a dashed line, not a dot, for "evidence
  exists, nature unknown." Bidirectional pairs bow apart into two arcs
  instead of two nearly-parallel offset lines.
- egui's `Painter` has no built-in bezier/arc-with-arrowhead primitive at
  the level of convenience `line_segment`/`convex_polygon` offer — expect to
  either (a) approximate an arc as a short polyline of segments along a
  quadratic bezier control point offset perpendicular from the straight
  chord (similar spirit to `draw_dashed_ring`'s segment-based circle
  approximation at line 712-723), or (b) look for `egui::Shape::QuadraticBezier`/
  `CubicBezier` if the pinned `egui` version (check `Cargo.toml` /
  `bevy_egui`'s pulled-in `egui` version) exposes one directly — worth a
  quick check before hand-rolling segment approximation.
- Dashes for a *line between two points that isn't a full circle* need a
  different helper than `draw_dashed_ring` (which dashes a circle's
  circumference) — likely a small `draw_dashed_line(painter, from, to,
  stroke)` that steps along the segment in fixed-length dash/gap
  increments, structurally similar in spirit but not directly reusable.

---

## 🔨 Suggested Implementation

1. Write a `draw_dashed_line` helper (segment-stepping along a straight
   chord, dash/gap lengths tuned by eye) for the partial-evidence case, and
   swap `draw_partial_marker`'s dot for a call to it between the same
   shortened start/end points `draw_edge` already computes (stopping short
   of node radii).
2. Remove the `±N` text block from `draw_edge` (lines 676-685); re-tune the
   thickness branch (`EDGE_STROKE_WEAK`/`EDGE_STROKE_STRONG`) so magnitude 1
   vs. 2 stay clearly distinguishable without the label, deriving from
   `config.effect_intensity_max` rather than a hardcoded `2`. Add magnitude
   to `text::confirmed_relation_line`'s output so the tooltip stays a
   complete fallback.
3. For bidirectional pairs: detect when both directions between a tag pair
   have something to draw (confirmed or partial, either direction), and in
   that case route each direction's draw through an arc helper instead of
   `draw_edge`'s straight-line path — e.g. compute a control point offset
   perpendicular from the chord's midpoint (opposite signs for the two
   directions so they bow away from each other), and either use a native
   bezier `Shape` if available or approximate with a short polyline.
4. Preserve arrowhead orientation at the arc's end (tangent direction at the
   endpoint, not the straight chord direction, once curvature is
   nontrivial) — an arrowhead pointing along the wrong tangent will look
   more broken than the straight-line version it replaces.
5. **Budget real time for visual tuning via `cargo run`** — curve/offset
   magnitude, dash length/gap, and stroke widths are not derivable from a
   formula; get a bidirectional pair on screen (natural playthrough or a
   short debug tweak to force one) and iterate until both directions read
   clearly at a glance. Record the final parameters and reasoning briefly in
   this file or the commit.
6. Update `node_tooltip_text`/`text.rs` wording to match the new grammar
   (no more implying a dot marker for partial evidence).
7. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
8. `cargo run`: verify per the acceptance criteria's live-verification line.

---

## ⚠️ Constraints and Caveats

- This is the task in the redesign batch most likely to need iteration
  beyond a first-pass implementation — don't treat a first compile-clean
  version as done without actually looking at a bidirectional pair on
  screen.
- Don't change *what* triggers an edge to render (confirmed vs. partial vs.
  invisible thresholds) — that's `MatrixKnowledge`'s domain and task 101's
  visibility scope, not this task's. This task only changes how an edge
  that's already decided to render actually looks.
- Keep color semantics (green = positive, red = negative,
  `PARTIAL_EVIDENCE_COLOR` gray = unknown) unchanged — only the marker shape
  (dot → dashed line) and label (text → thickness) change.

---

## 🔗 Dependencies

- **Depends on**: 031 (hypothesis grid as graph), 028 (partial-evidence
  marker origin), 061 (magnitude stroke-width precedent).
- **Related, should land together or right after**: task 101 (grid
  visibility/layout) — both touch `hypothesis_grid`'s rendering code
  directly; not a hard blocker on each other, but doing them out of order in
  the same session risks needing to rebase edge-drawing changes on top of
  visibility changes or vice versa. Recommend landing 101 first if only one
  can go at a time, since it changes *which* edges exist before this task
  changes *how* they're drawn.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/102-notebook-grid-edge-grammar.md)"$'\n\nExecute this task in the current project.'
```
