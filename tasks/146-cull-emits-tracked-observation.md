# Task 146 — Cull emits a tracked notebook observation

> **ID**: `146`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~1.5h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

`Cull` (`input.rs::cull_on_click`) removes the organism at a cell and does
nothing else — no evidence, no log entry, no notebook trace at all. The
design doc frames Cull as the game's third experimental mode alongside
Seed ("place and observe") and Stress ("perturb and observe"): a genetic
knockout — remove an element, observe the effect of its absence — and says
this should feed the **same weighted evidence system** GDD §7 already uses
for adjacency evidence, not a separate one. `sim.rs` already carries a
placeholder acknowledging this gap verbatim: `TickEvents`'s doc comment
(line ~222) reads *"Task 146's `Cull` knockout observation belongs here
too, once it exists — not implemented by this task."*

Design source: `redesign/processed/abiogenesis-actions.md`, "Cull —
corretto: knockout mirato, non sterminio" and the Cull bullet under "Cosa
serve per l'integrazione".

**Cost stays unchanged** (1 point, `config.time.action_costs.cull`) — the
doc is explicit the limited budget is already the brake against abuse, not
a price to raise.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] `cull_on_click` (`input.rs:452`), immediately before clearing
      `cell.population`, emits one `AdjacencyObserved` (`sim.rs:131`) per
      occupied Moore neighbour tag pair between the culled organism and its
      still-living neighbours — reusing the **exact same event shape**
      (`exerter_tag`/`receiver_tag`/`contribution`/`n_confounders`/`cell`)
      and the exact same confounder-count and matrix-lookup logic
      `sim::step`'s per-tick adjacency loop already uses (`sim.rs:1046`-
      `1087`), so it flows through `notebook::accumulate_evidence` /
      `knowledge::accumulate_adjacency_evidence` unmodified — no new
      evidence-weight formula, no new notebook system. Confirming a pair
      via a knockout observation must produce the same log entry
      (`text::confirmation_message`) and unseen-confirmation badge
      (`NotebookHasUnseenConfirmation`) an ordinary adjacency confirmation
      does today.
- [ ] The matrix-lookup/confounder logic used for this is a shared helper
      extracted from (or called by) both `sim::step`'s tick loop and
      `cull_on_click`, not a second hand-written copy — this codebase's own
      stated principle (`knowledge.rs`'s doc comment: a duplicated formula
      "is exactly the notebook/narration drift... argues against").
      Concretely: extract the per-neighbour-tag-pair scan (tag-gate check +
      `world.matrix.get` + contribution + confounder count) out of the
      `interaction_delta` loop in `sim::step` into a function taking
      `(&SimWorld, x, y, receiver_species) -> Vec<AdjacencyObserved>` (no
      onset-gating — Cull is a one-shot event, not a persisting tick, so
      unlike the ordinary tick loop it must **not** consult
      `adjacency_exposure`/`onset_mask`; every currently-adjacent tag pair
      counts, this is the one and only observation this event will ever
      produce for this pairing).
- [ ] `input.rs`'s Bevy system gains a `MessageWriter<AdjacencyObserved>`
      (or pushes through `TickEvents`/`TickEventWriters` if that plumbing
      is reachable outside `sim::step` — pick whichever fits the existing
      message-writer wiring with the least new surface, document the
      choice).
- [ ] A culled organism with **no** living neighbours emits nothing (no
      panic, no zero-weight noise) — mirrors `AdjacencyObserved`'s existing
      "only emitted for pairs with a non-zero matrix entry" behaviour
      (`knowledge.rs:31`).
- [ ] At least one test (in `sim.rs`'s existing unit-test module, next to
      the adjacency-evidence tests, or a new `input.rs` test if the click
      handler is directly testable) confirms: culling an organism next to
      a single differently-tagged neighbour produces the expected
      `AdjacencyObserved` event(s) with `n_confounders` computed from that
      neighbour's *other* neighbours, matching the tick-loop formula.
- [ ] `sim.rs:222`'s `TickEvents` doc comment note about task 146 is
      resolved — updated or removed once the mechanism lands elsewhere.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `cull_on_click` (`~452`-`500`) — emits the knockout observation before clearing the cell. |
| `src/sim.rs` | `AdjacencyObserved` (`131`), `TickEvents` (`225`, task-146 note at `222`), the tick-loop adjacency scan (`1001`-`1088`) to extract a shared helper from. |
| `src/knowledge.rs` | `accumulate_adjacency_evidence` — unmodified, reused as-is. |
| `src/notebook.rs` | `accumulate_evidence` (`226`) — unmodified, reused as-is; confirms the event shape is really compatible. |

---

## 🧩 Technical Context

- **Current behavior**: Cull clears `cell.population`, decrements the
  budget, and removes the cell from `PlayerPlacedCells` — no event of any
  kind is emitted (confirmed by reading `cull_on_click` in full: no
  `MessageWriter` parameter exists on that system today). A culled
  organism leaves zero trace in the notebook or the evidence store, even
  when it had living neighbours whose relationship to it was never
  confirmed.
- **Desired behavior**: at the instant of removal, if the culled organism
  has living neighbours of a different species, the removal itself counts
  as one observation per relevant tag pair — weighted and folded into
  `MatrixKnowledge` exactly like a live adjacency tick would be, capable of
  pushing a previously-unconfirmed pair over the confirmation threshold and
  producing the usual log entry.
- **Why not literally compare energy before/after removal** (the doc's
  narrative framing, "Y behaves differently after removing X"): the
  existing evidence model is presence-based (an `AdjacencyObserved` records
  that an exerter/receiver tag pair was adjacent this tick, weighted by
  confounders), not a before/after energy-delta measurement — there is no
  existing machinery to snapshot and diff a neighbour's energy trajectory
  across a Cull, and building one would be new evidence infrastructure the
  doc doesn't ask for ("con lo stesso sistema di peso già esistente",
  literally "with the same weighting system already in place"). Reusing
  `AdjacencyObserved` as-is satisfies the doc's actual request and keeps
  the notebook's evidence model single-track.

---

## 🔨 Suggested Implementation

1. In `sim.rs`, extract the per-neighbour-tag-pair body of the
   `interaction_delta` loop (tag-gate check, `world.matrix.get`,
   contribution, confounder count via `neighbour_tags`) into a standalone
   function, parameterized so it can run without `onset_mask` gating.
   `sim::step`'s existing call site keeps its onset check as a filter
   applied to this function's output (or keeps its own gated loop calling
   the shared inner piece — whichever avoids duplicating the tag-gate/
   matrix-lookup logic itself).
2. Give that function a signature usable from `input.rs`: something like
   `fn adjacency_observations_for(world: &SimWorld, x: usize, y: usize) ->
   Vec<AdjacencyObserved>`, called once with the culled organism's own
   species/tags as receiver *and* once with it as exerter for each
   neighbour, matching the tick loop's existing bidirectional
   `their_tag`/`my_tag` structure (the culled organism is simultaneously a
   receiver of its neighbours' tags and an exerter of its own onto them —
   confirm from the tick loop which direction(s) actually apply here, since
   only the culled organism's own removal is the "event", not its
   neighbours').
3. Wire a `MessageWriter<AdjacencyObserved>` into `cull_on_click`'s system
   params, call the helper before `cell.population = None`, write the
   results.
4. Remove/update the `sim.rs:222` comment once the mechanism exists.
5. Test as described in Acceptance Criteria.

---

## ⚠️ Constraints and Caveats

- **No onset-gating for the knockout observation** — this is a one-shot
  emission, unlike the tick loop's persistent-adjacency onset filter
  (`136b`). Don't accidentally import that gate.
- **Determinism**: no RNG involved.
- Keep `sim`/`world` free of `bevy::render`/`bevy_egui` deps
  (`TECH_DESIGN.md` §5) — the extracted helper stays plain Rust, only
  `input.rs`'s Bevy system touches `MessageWriter`.
- Don't touch Cull's cost, its Detail-only gating, or its
  `PlayerPlacedCells` cleanup — all correct and out of scope.

---

## 🔗 Dependencies

- **Depends on**: 136b (per-onset evidence weighting, already shipped —
  this task's helper deliberately does *not* reuse onset-gating, but it
  does reuse the confounder-count/weight formula 136b's model relies on).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/146-cull-emits-tracked-observation.md)"$'\n\nExecute this task in the current project.'
```
