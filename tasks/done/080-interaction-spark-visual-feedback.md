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

- [x] `AdjacencyObserved` (`src/sim.rs:79-84`) gains a `cell: usize` field
      (the receiver's cell index), populated at the existing push site
      (`src/sim.rs:293-298`).
- [x] A new resource (`SeenRelations`, `src/render.rs`) tracks which
      `(exerter_tag, receiver_tag)` pairs have already triggered a spark this
      world — not a `HashSet`, a flat `Vec<bool>` indexed exactly like
      `TagMatrix`/`MatrixKnowledge` (`exerter * size + receiver`), per
      `CLAUDE.md`'s no-`HashMap`/`HashSet`-iteration rule (pure membership
      lookup, never iterated). Reset on every world (re)start: both
      `run_flow::start_world` (via `WorldResetParams`, alongside
      `GraceProgress`) **and** `menu::start_run` (the very first world of a
      run) — `MatrixKnowledge` needed the same second reset point for the
      identical reason (worldgen doesn't guarantee `active_tags.len()`
      matches the build-time default), found live during this task's own
      verification.
- [x] A generalized sibling of `PlacementIndicator`: `SparkIndicators`
      (`src/render.rs`, `spark_indicator` module), `Vec`-backed instead of
      single-slot, since several relations can be confirmed in the same
      tick. Same animation as `PlacementIndicator`, unchanged (fixed radius,
      alpha-only fade over `DURATION_SECS`) — no radius-shrink.
- [x] ~~In Overview, the spark renders aggregated on the cluster/blob
      containing that cell (reuse task 076's cluster lookup)~~ — **dropped
      during implementation**: `cluster::compute_cluster_density` (task 076)
      has no centroid/blob geometry at all, it only recolors the *same*
      per-cell sprites Detail mode uses (task 078, not yet done, is what
      would add real blob shapes). The spark renders at the exact cell
      (`AdjacencyObserved.cell` → `cell_position`) unconditionally — this is
      already correct in both `MapViewMode::Detail` and `Overview`, with no
      mode branching needed, exactly like `draw_placement_indicator` already
      does for the seed-placement ring.
- [x] A system (`spawn_spark_on_first_observation`) draining
      `AdjacencyObserved` messages each frame: for each event, checks
      `SeenRelations`; if unseen, marks it seen and spawns a new spark
      indicator at `event.cell`.
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [x] Verified live via `cargo run` (seed 42, Nyx seeded adjacent to Sable —
      their tags have several non-zero matrix entries): the very first tick
      produced two overlapping rings (one per direction of the interaction,
      landing on each organism's own cell) exactly once, confirmed via a
      temporary `DURATION_SECS` bump (6.0, reverted before commit) to make
      the screenshot timing reliable — the default 0.6s fade is real-time
      based (`Res<Time>`) and was otherwise too fast to reliably catch
      against synthetic input/screenshot latency. Also added a unit test
      (`spawn_spark_on_first_observation_fires_once_per_pair_not_per_event`,
      `src/render.rs`) exercising the real production system function
      directly: fires once on first observation, not again on a repeat of
      the same pair, and independently for a genuinely new pair.

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
2. `render.rs`: `SeenRelations` resource; `spark_indicator` module (sibling
   of `placement_indicator`, `Vec`-backed); `spawn_spark_on_first_observation`
   system.
3. Reset `SeenRelations` in **two** places, both sizing it from
   `world.active_tags.len()`: `run_flow::start_world` (`WorldResetParams`,
   alongside `GraceProgress`) for every world after the first, and
   `menu::start_run` for the very first world of a run — don't skip the
   second one, it's not optional (see Technical Context).
4. Register the new resource/systems in `GridRenderPlugin::build`, same
   `Update`/`EguiPrimaryContextPass` split as `placement_indicator`.
5. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
6. Live verification via `cargo run` — if screenshot timing is unreliable
   against the real 0.6s fade, temporarily bump `spark_indicator`'s
   `DURATION_SECS` for the screenshot and revert it before committing;
   don't ship the widened value.

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
