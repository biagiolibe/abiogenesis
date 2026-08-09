# Task 080 — Interaction spark: instant visual feedback on first-seen relations

> **ID**: `080`
> **Category**: Feature / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (scoped from `redesign/abiogenesis-engagement-design.md`, proposal 1.A)

---

## 🎯 Objective

The hidden matrix's `interaction_delta` (GDD §5.6 step 3) applies every tick but
is invisible to the player — it's only inferred indirectly, after enough
`AdjacencyObserved` evidence accumulates in the Notebook (task 018/020). In
the first minutes of a run this reads as noise, not mystery: nothing signals
"something just happened here."

Add a one-shot visual "spark" that fires the **first time ever** a given
`(exerter_tag, receiver_tag)` pair is observed in a world (not on every
non-zero `interaction_delta` — that would flood dense grids with constant
flashing). It turns the hidden matrix into an *observed* event instead of a
purely deduced one, without touching any balance number.

---

## 📋 Acceptance Criteria

- [ ] `AdjacencyObserved` (`src/sim.rs:79-84`) gains a `cell: usize` field
      (the receiver's cell index), populated at the existing push site
      (`src/sim.rs:293-298`).
- [ ] A new Bevy resource tracks which `(exerter_tag, receiver_tag)` pairs
      have already triggered a spark this world (membership-only, never
      iterated — no `HashMap`/`HashSet` iteration per `CLAUDE.md`'s
      determinism rule; this resource is presentation-only and outside
      `sim`/`world`/`config`, so it may live in Bevy state, but the check
      itself must stay a pure lookup). Reset on every world (re)start,
      alongside the pattern task 079 used for `GraceProgress` in
      `src/run_flow.rs`'s `start_world`.
- [ ] A generalized version of `PlacementIndicator` (`src/render.rs`,
      `placement_indicator` module) supporting **multiple concurrent**
      indicators — today it's a single-slot `Option<PlacementIndicatorState>`,
      but several relations can be confirmed in the same tick. Keep the
      existing animation unchanged (fixed radius, alpha-only fade over
      `DURATION_SECS`) — no radius-shrink, explicitly out of scope for now.
- [ ] In `MapViewMode::Detail`, the spark renders on the exact cell
      (`AdjacencyObserved.cell`).
- [ ] In `MapViewMode::Overview`, the spark renders aggregated on the
      cluster/blob containing that cell (reuse task 076's cluster lookup —
      read `tasks/done/076-overview-cluster-heatmap-rendering.md` for the
      existing cluster data structure before implementing this branch).
- [ ] A system draining `AdjacencyObserved` messages each frame: for each
      event, checks the seen-pairs resource; if unseen, marks it seen and
      spawns a new spark indicator at the right position for the active view
      mode.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: seeding two species whose tags have a
      non-zero matrix entry produces exactly one spark on first contact, no
      repeat spark on subsequent ticks of the same pair, and the spark
      appears correctly in both Detail and Overview.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `AdjacencyObserved` struct (add `cell` field), push site at line 293. |
| `src/render.rs` | `placement_indicator` module (generalize to multi-instance), `cell_color`/cluster rendering for the Overview-mode branch. |
| `src/run_flow.rs` | Reset the seen-pairs resource on world (re)start, same pattern as `GraceProgress`. |
| `tasks/done/076-overview-cluster-heatmap-rendering.md` | Reference for the Overview-mode cluster lookup this task must reuse. |

---

## 🧩 Technical Context

- **Current behavior**: `interaction_delta` is computed and applied silently
  every tick (`src/sim.rs:257-302`); the only player-facing trace is the
  Notebook's evidence log, populated well after the fact.
- **Desired behavior**: the instant a tag-pair relation is seen for the first
  time in a world, a transient ring appears at the relevant location(s),
  independent of whether that relation ever gets confirmed in the Notebook.
- Placement indicator precedent: `src/render.rs`, `PlacementIndicator`
  resource, `PlacementIndicatorState { x, y, remaining_secs }`,
  `tick_placement_indicator` (decrements via `Res<Time>`, clears at 0),
  `draw_placement_indicator` (egui `circle_stroke`, alpha scaled by remaining
  fraction). Today it only fires from `seed_organism_on_click`
  (`src/input.rs`) and only in Overview mode.

---

## 🔨 Suggested Implementation

1. `sim.rs`: add `cell: usize` to `AdjacencyObserved`, populate at the push
   site.
2. New seen-pairs resource + reset wiring in `run_flow.rs`.
3. Generalize `PlacementIndicator` → a `Vec`-backed multi-instance resource
   (or keep `PlacementIndicator` for seed-placement as-is and add a sibling
   `SparkIndicators` resource reusing the same draw/tick pattern — pick
   whichever avoids duplicating the fade-animation logic, e.g. a shared
   helper both call).
4. New system draining `AdjacencyObserved`, filtering against the seen-pairs
   resource, spawning spark indicators positioned per the active
   `MapViewMode` (exact cell in Detail, cluster centroid in Overview via
   task 076's lookup).
5. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
6. Live verification via `cargo run`.

---

## ⚠️ Constraints and Caveats

- No radius-shrink animation — reuse the existing alpha-only fade exactly.
  Do not add new animation complexity.
- Do not gate this on `MatrixKnowledge` confirmation threshold (task 020,
  `3.0`) — the spark fires on first *observation*, which is a much lower bar
  than confirmed evidence. These are two independent signals.
- Presentation-only: no `SimConfig` coefficient changes, no `sim`/`world`
  behavior changes beyond the new `AdjacencyObserved.cell` field.

---

## 🔗 Dependencies

- **Depends on**: 018 (`AdjacencyObserved` origin), 075/076 (view-mode
  switch and cluster data the Overview branch reuses).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/080-interaction-spark-visual-feedback.md)"$'\n\nExecute this task in the current project.'
```
