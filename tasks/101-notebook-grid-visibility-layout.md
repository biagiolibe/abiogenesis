# Task 101 — Hypothesis grid: reveal-on-first-observation, layout over visible subset only

> **ID**: `101`
> **Category**: UX / Notebook
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-notebook-redesign.md`)

---

## 🎯 Objective

Live screenshot review (redesign doc) found 3 of 5 active tags sitting inert
with dashed rings on the hypothesis grid, never interacted with, pure
clutter — the ghost nodes for zero-evidence tags cost visual space without
carrying information, and the one real relationship among the active tags
got no layout benefit from being the only thing that mattered.

Two changes to `hypothesis_grid` (`src/notebook.rs:574-629`):

1. **Visibility**: a tag with zero evidence in every direction
   (`has_no_evidence`, `src/notebook.rs:692-700`) doesn't render at all — no
   dashed ring, no node, nothing — until the first tick that gives it any
   evidence (`knowledge.evidence(...) > 0.0` in either direction). Today
   `has_no_evidence` only decides whether to draw a dashed ring
   (`draw_dashed_ring`) around an otherwise-present node; after this task it
   decides whether the node exists on the grid at all.
2. **Layout**: positions recompute an even circular/clustered placement over
   just the *currently visible* subset each time a new tag becomes visible,
   instead of the current fixed layout that reserves a slot for every
   `world.active_tags` entry regardless of whether it's ever been touched.
   Fewer visible nodes means automatically less crowding — this is **not**
   a force-directed physics simulation; see Technical Context.

Optionally, a brief pop-in/fade-in animation when a tag transitions from
invisible to visible, reusing the existing interaction-spark feedback family
(tasks 054, 080) rather than inventing a new discovery-feedback mechanism.

---

## 📋 Acceptance Criteria

- [ ] A tag with `has_no_evidence(...) == true` (or equivalent renamed
      check) is skipped entirely in `hypothesis_grid`'s node-drawing loop —
      no `draw_dashed_ring`, no filled circle, no glyph, no interactive hit
      area for it.
- [ ] `draw_dashed_ring` (`src/notebook.rs:712-723`) and
      `DASHED_RING_MARGIN`/`DASHED_RING_SEGMENTS`/`DASHED_RING_COLOR`
      (`src/notebook.rs:703-706`) are removed if this was their only call
      site (grep to confirm before deleting) — the "ghost ring for an
      unobserved tag" concept goes away entirely, it is not replaced by a
      different unobserved-tag visual.
- [ ] Node positions (`positions` in `hypothesis_grid`, currently one entry
      per `world.active_tags` index at a fixed angle) are computed only over
      the subset of tags with `has_no_evidence(...) == false`, evenly spaced
      around the circle for that subset's size — so a 2-visible-tag state
      and a 5-visible-tag state both look evenly laid out, not like 2 dots
      lost in a 5-slot ring.
- [ ] Edge-drawing loop (currently iterating all `ei`/`ri` pairs over
      `tags.len()`) only draws edges between visible tags — an edge can only
      exist where evidence is nonzero on at least one side, which already
      implies both endpoints are visible, so this should fall out naturally
      from the visibility filter rather than needing a separate check (worth
      confirming with a quick manual trace or a test).
- [ ] Newly-revealed tags do not jump discontinuously mid-frame in a way that
      reads as a glitch — either a static recompute-on-reveal (acceptable
      per the redesign doc's explicit scope call, see below) or a short
      pop-in/fade-in reusing the spark/confirmation feedback family (tasks
      054, 080) if it's a small addition; either is acceptable, but note
      which was chosen in the outcome notes.
- [ ] Explicitly **not** implemented: continuous force-directed physics
      (attraction/repulsion simulated every frame). If this task's
      implementer is tempted to reach for it because the static layout still
      feels stiff, don't — that's a deliberate, already-discussed
      complexity/ROI trade-off (see Technical Context), not an oversight to
      silently fix here.
- [ ] When zero tags are currently visible (a fresh world, before any
      `AdjacencyObserved` event), the grid area does not render as a blank
      painter with no explanation — show a centered weak-styled
      `text::NO_OBSERVATIONS_YET` (or an equivalent grid-specific message,
      new constant in `text.rs` if the existing one doesn't read naturally
      in this context) inside the grid's allocated rect, so the empty state
      still communicates something rather than looking broken. Today's
      dashed rings at least conveyed "N tags exist, none observed yet" —
      this replaces that signal, it must not silently disappear.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: start a fresh world, open the notebook —
      the grid shows the empty-state message (no dashed-ring ghosts for
      untouched tags, but also not a blank painter); as tags accumulate
      evidence over several eras, each newly-involved tag appears on the
      grid and the layout stays legible (nodes don't overlap, spacing looks
      even for however many are currently visible).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `hypothesis_grid` (line 574-629) — node/edge iteration and position computation. `has_no_evidence` (line 692-700) — repurposed from "draw a ghost ring" to "skip this node." `draw_dashed_ring` (line 712-723) and its constants (line 703-706) — likely deleted. |
| `tasks/done/054-celebrate-first-confirmed-hypothesis.md` | Precedent for "discovery feedback" — the confirmation `★`/badge pattern, reusable for a reveal animation. |
| `tasks/done/080-interaction-spark-visual-feedback.md` | Precedent for a one-shot fade-in visual (`SparkIndicators`, `src/render.rs`) on a first-seen relation — the closest existing analog to a "tag just became visible" animation, if one is added. |

---

## 🧩 Technical Context

- **Current behavior**: `hypothesis_grid` computes `positions` as one entry
  per index of `world.active_tags` (`src/notebook.rs:582-588`), evenly
  spaced around a fixed circle regardless of whether any evidence exists for
  that tag. `has_no_evidence` (line 692) is checked only to decide whether
  to overlay a dashed ring (line 612-614) on an otherwise fully-rendered
  node — the node (filled circle, glyph, hover tooltip) always renders
  either way.
- **Desired behavior**: a tag with zero evidence anywhere doesn't occupy a
  grid slot at all. The set of visible tags — and therefore the layout — can
  grow over the course of a run as more tags get touched by observations.
- **Layout choice, explicit**: the redesign doc's own decision record states
  this was "a deliberate complexity/ROI call, not a legibility objection to
  physics-based layout in principle" — a simple recompute-on-reveal
  (still circular/clustered, just over a shrinking/growing subset) is
  intentionally chosen over simulating attraction/repulsion every frame.
  Worth revisiting later if the simple version still feels static once grids
  regularly have 4-6 visible tags, per the doc's own open question — but
  that revisit needs a new discussion, not a unilateral upgrade inside this
  task.
- Tags becoming visible is monotonic within a run (evidence never goes back
  to zero once nonzero — `MatrixKnowledge::record` only adds), so "recompute
  layout when a new tag is revealed" never needs to handle a tag
  disappearing again.
- `catalog_panel`'s "Active tags" row (`src/notebook.rs:810-815`) still
  lists the full tag pool unconditionally — hiding untouched nodes from the
  grid does not hide the roster itself, the player can always see how many
  tags exist and their glyphs/colors in the Catalog section below. This is
  what makes reveal-on-observation a legibility improvement rather than
  information loss: nothing becomes uninferable that wasn't already exposed
  elsewhere.
- `node_tooltip_text`'s `lines.len() == 1` fallback (`src/notebook.rs:
  776-778`, prints `NO_OBSERVATIONS_YET` for a node with nothing to report)
  becomes unreachable once only nodes with at least one direction of
  evidence ever render — a node that would trigger this fallback no longer
  exists to be hovered. Leaving the dead branch is harmless but worth a
  one-line note in the function's doc comment if not removed.

---

## 🔨 Suggested Implementation

1. In `hypothesis_grid`, compute a `visible: Vec<usize>` of tag indices where
   `!has_no_evidence(...)`, before building `positions`.
2. Build `positions` only for `visible`'s length, evenly spaced around the
   circle exactly as today's formula does, but indexed against `visible`'s
   position within itself rather than the raw tag index — then map back to
   the original tag index when looking up colors/glyphs/edges.
3. Update the edge-drawing double loop to iterate over `visible × visible`
   instead of `0..tags.len() × 0..tags.len()` (or keep the outer loop as-is
   and skip non-visible pairs early — whichever reads cleaner).
4. Update the node-drawing loop similarly: only draw for `visible` entries;
   delete the `draw_dashed_ring` call and, if now dead, the function and its
   constants. If `visible` is empty, draw the centered empty-state message
   in the painter's rect instead of leaving it blank.
5. Optional: reuse `SparkIndicators`-style fade-in (task 080's pattern) for
   a tag's first frame of visibility — this needs a first-pass visual check
   via `cargo run` either way (timing/easing aren't derivable from a
   formula), so budget time for that even if the animation is skipped in
   favor of a static reveal.
6. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
7. `cargo run`: verify per the last acceptance criterion above.

---

## ⚠️ Constraints and Caveats

- No force-directed physics — see Technical Context. This is a hard
  constraint from the redesign discussion, not a suggestion.
- Don't change edge/marker visual grammar (color, thickness, dashed-vs-solid
  for partial evidence) in this task — that's task 102's scope. This task
  only changes *which nodes exist* and *where they sit*, not how edges are
  drawn once both endpoints are visible.
- Keep `node_tooltip_text`'s hover fallback intact for whatever nodes remain
  visible — the acceptance criterion that the graph isn't the *only* way to
  read this information (task 031's original constraint) still applies.

---

## 🔗 Dependencies

- **Depends on**: 031 (hypothesis grid as graph, the base this modifies),
  028 (partial-evidence marker, `has_no_evidence`'s origin).
- **Related, not a hard dependency**: task 100 (log rework) touches the same
  redesign track and pairs naturally with this one landing in the same
  session, but neither blocks the other. Task 102 (edge grammar rewrite)
  should land together with or immediately after this task since both touch
  `hypothesis_grid`'s rendering — not a hard blocker, but doing them out of
  order risks rebasing edge-drawing code twice.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/101-notebook-grid-visibility-layout.md)"$'\n\nExecute this task in the current project.'
```
