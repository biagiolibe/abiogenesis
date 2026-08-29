# Task 181 — Notebook chrome fidelity: match the reference mockup's graph, catalog, and log style

> **ID**: `181`
> **Category**: UI / Bugfix
> **Priority**: 🟡 P2 (corrective — Phase 2 residual)
> **Estimate**: ~2.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-29 scoping

---

## 🎯 Objective

Companion to task 180 (`tasks/180-pixel-grain-corrections.md`), same
triggering finding: task 151's pixel-grain restyle shipped a narrow subset
of what its own cited reference images actually specify, confirmed by a
direct screenshot-vs-mockup comparison. This task covers everything inside
the **notebook window** (`notebook.rs`) against
`redesign/processed/pixel-notebook.svg`, the task's own reference for that
surface. 180 covers the HUD/sidebar (`ui.rs`) under the same governing
rule — split for atomicity, not because the two differ in principle.

Design source: `redesign/processed/culture-shock-population-model-aesthetic.md`
§88-121, cross-checked against `redesign/processed/pixel-notebook.svg`.

**Read `VISUAL_STYLE_GUIDE.md` first** — §3 (color), §4 (organism icons),
§5 (trait/tag iconography + relationship-graph grammar) are the sections
this task implements; no need to re-derive them from the SVG by hand.

**Same governing rule as 180: color encodes state, never identity.** Species
and tag identity is carried by text (name, 3-letter code) and by shape
(the metabolism block-icon), never by a per-entity hue. The relationship
graph's nodes in `pixel-notebook.svg` are neutral dark-slate boxes with an
amber stroke (`<rect fill="#1c2229" stroke="#e0c99a">`, lines 17-24 of that
file) labeled by a 3-letter code — never tag-colored. The current build
fills them with `tag_color(tag)` instead.

**User-confirmed exclusion (from the paired 180 discussion, applies here
too): nothing about the map's tree-glyph overlay is in scope — this task
doesn't touch the map at all.**

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.

- [ ] **Relationship-graph nodes become neutral squared boxes, not colored
      circles.** `notebook.rs::hypothesis_grid` (`942-1056`) draws each
      node via `painter.circle_filled(pos, NODE_RADIUS, tag_color(tag))`
      (`1042`) — colored per-tag. Replace with a filled rect using a
      neutral dark-slate fill (matching the notebook's own panel material,
      not a new color) and an amber stroke, squared corners — per
      `pixel-notebook.svg:17-24`. `tag_color(tag)` stops being read here;
      confirm it isn't now dead code (it's also used at `notebook.rs:1328`
      for a tag swatch — check whether that call site is itself in scope
      per the governing rule, and if so fix it too rather than leaving one
      inconsistent survivor).
- [ ] **Node border encodes observed-vs-hypothesis, per the mockup.**
      `pixel-notebook.svg` gives three nodes (CHT/PRN/QRM) a solid border
      and one (LIP) a dashed border (`stroke-dasharray="3 3"`) —
      distinguishing a tag with any confirmed evidence from one that's only
      ever appeared as an unconfirmed hypothesis participant.
      `hypothesis_grid` already computes a comparable per-tag evidence
      signal for edge drawing (`has_something`, `~987-990`); reuse it (or
      the tag-level aggregate task 151's own `has_no_evidence`,
      `~955-961`, already computes) to choose solid vs. dashed stroke per
      node.
- [ ] **Node label becomes the 3-letter tag code, not the glyph.**
      `painter.text(..., tag_glyph(tag), ...)` (`1043-1049`) draws a single
      glyph centered on the (previously colored) circle. The mockup labels
      nodes with 3-letter codes (`CHT`, `PRN`, `QRM`, `LIP`). **This
      acceptance criterion is conditional on task 155** (trait archetypes,
      `tasks/155-trait-archetypes.md`) having landed — 155 is what
      introduces the 3-letter code table in the first place; if 181 is
      picked up before 155, keep the current single-glyph label here (it's
      still readable, just narrower than the mockup) and note in the PR
      that this one line is deferred to whenever 155 ships, rather than
      blocking 181 on 155's own unrelated scope.
- [ ] **Species Catalog icons reuse the same block-pattern shape library as
      the map/HUD.** `notebook.rs`'s Catalog section (`~1328-1400`,
      `TAG_GLYPH = "●"` at `606`) renders a flat colored bullet per
      species/tag entry. Per the design doc's "un solo set riusato ovunque,
      non uno per contesto" requirement and `pixel-notebook.svg:28-35`'s
      `#e0c99a`-colored cluster icons, render each Catalog entry's icon via
      the shared metabolism block-icon painter (introduce in 180, or here
      if 181 lands first — see Dependencies), amber ink, keyed by that
      species' metabolism. Name text remains the identity carrier.
- [ ] **Observation-log entry markers encode outcome, not species.**
      `notebook.rs:797-808`: a log entry with a species draws
      `ui.colored_label(species_color(species), TAG_GLYPH)` — species-hued.
      `pixel-notebook.svg:7-12` colors log-entry markers green
      (`#7fae6a`) or red-rust (`#c96a5c`) — reading as outcome/valence
      (e.g. "weakened" vs. "stable"), not species identity. Determine what
      state each log entry actually carries today (check
      `ObservationLog`/whatever type backs `entries` in `notebook.rs` —
      likely already has a positive/negative or clean/confounded signal
      from task 061's own green/amber distinction, `tasks/done/
      061-notebook-presentation-refinements.md:29`) and key the marker
      color off that instead of `species_color`. If no such per-entry
      valence signal currently reaches this rendering call, flag it as an
      open gap in the PR rather than inventing one — don't add new
      `SimWorld` state from a UI-styling task.
- [ ] Live visual check (`cargo run`, screenshot or interactive): graph
      nodes are neutral boxes with amber stroke, solid/dashed by evidence
      state; Catalog icons show block patterns, not dots; Observation log
      markers (if fixed) read as outcome-colored, not species-colored.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `hypothesis_grid` (`942-1056`) — node style. Catalog section (`~1328-1400`) — icon swap. Observation log (`~788-810`) — marker semantics. `tag_color` (`601`) — audit remaining call sites. |
| `src/render.rs` | `MetabolismShapes` — reuse pattern (shared with 180, don't duplicate the icon geometry a second time — coordinate with whichever of 180/181 lands first). |
| `abiogenesis-gdd.md` | §11 sync happens once **both** 180 and 181 land, not from this file alone. |

---

## 🧩 Technical Context

- **Current behavior**: graph nodes are tag-colored filled circles labeled
  with a single glyph; Catalog entries use a flat colored bullet; log
  entries with a species attached use a species-hued marker.
- **Desired behavior**: graph nodes are neutral slate boxes with an amber
  stroke (solid/dashed by evidence state) labeled with a 3-letter code (once
  155 lands); Catalog icons show the metabolism block pattern in amber;
  log-entry markers encode outcome/valence, not species.

---

## 🔨 Suggested Implementation

1. Check whether 180 already landed and exposed a shared
   `paint_metabolism_icon` helper; if not, extract it here (same shape,
   coordinate to avoid two divergent implementations — leave a `// TODO`
   pointing at the sibling task if landing first).
2. `hypothesis_grid`: swap `circle_filled` for `painter.rect_filled` (slate
   fill) + `painter.rect_stroke` (amber, solid or dashed by evidence),
   swap the glyph text draw for the 3-letter code once available.
3. Catalog section: replace `TAG_GLYPH` bullet with the shared icon painter.
4. Observation log: trace what data `entries` actually carries per item,
   decide the outcome-color mapping from what's real, not invented.
5. Live-check the notebook's Relationships, Catalog, and Observation Log
   sections with at least one confirmed pair and one hypothesis-only tag
   present, so both node border states are exercised.

---

## ⚠️ Constraints and Caveats

- **No hand-drawn assets** — same constraint as 151/180.
- **Don't invent new `SimWorld`/`sim.rs` state** to satisfy the log-marker
  AC if the valence signal doesn't already reach `notebook.rs` — flag it
  instead, this is a presentation-layer task.
- Keep `sim`/`world`/`config` untouched — `notebook.rs`/`render.rs`
  (shared icon painter) only.
- The 3-letter-code node label AC is explicitly conditional on 155 — don't
  block on it, don't invent a placeholder code scheme either.

---

## 🔗 Dependencies

- **Depends on**: 151 (corrects/extends it). Soft-depends on 155 for the
  node-label AC only (rest of the task is independent).
- **Related**: 180 (HUD-side equivalent, same governing rule and shared
  icon-painter helper — coordinate whichever lands second to reuse the
  first's extraction rather than duplicating it).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/181-notebook-chrome-fidelity.md)"$'\n\nExecute this task in the current project.'
```
