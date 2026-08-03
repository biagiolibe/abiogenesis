# Task 020 — Hypothesis confirmation engine

> **ID**: `020`
> **Category**: Architecture / Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Implement the "B with a hint of C" confirmation model (GDD §7): accumulate weighted evidence for every `(exerter_tag, receiver_tag)` pair from the `AdjacencyObserved` events task 018 produces, and mark a cell confirmed once cumulative evidence crosses the threshold. This is the mechanic that progressively reveals the hidden matrix (GDD §5.5) — "there isn't a second, separate opacity mechanic." Pure logic in this task; the UI that displays it is task 021.

---

## 📋 Acceptance Criteria

- [ ] A `MatrixKnowledge` resource (or plain struct, constructed with `active_tags.len()`), holding cumulative evidence per ordered tag pair (`Vec<f32>` sized `n*n`, mirroring `TagMatrix`'s own layout) and a derived confirmed/unconfirmed bool per cell.
- [ ] For each `AdjacencyObserved { exerter_tag, receiver_tag, n_confounders, .. }`, evidence added is `config.notebook.observation_weight_numerator / (1.0 + n_confounders as f32)` (GDD §7's formula, `SimConfig::notebook` already holds both constants — do not redefine them).
- [ ] A cell becomes confirmed when its cumulative evidence reaches `config.notebook.confirmation_threshold` (already `3.0` in `SimConfig::default()`) — confirmation is monotonic (evidence only accumulates, never decays or resets within a run).
- [ ] `MatrixKnowledge` exposes read access to (a) cumulative evidence for a pair, (b) whether a pair is confirmed, and (c) the *sign* of the confirmed effect — this needs the real `world.matrix.get(exerter, receiver)` value at confirmation time (the engine doesn't have to guess the sign from evidence direction: once a cell is confirmed, the game can safely reveal `world.matrix`'s actual value for that cell only). Decide and document whether `MatrixKnowledge` stores its own snapshot of the revealed value or reads through to `world.matrix` filtered by a confirmed-cells set — the latter is simpler and avoids duplicating the matrix.
- [ ] A Bevy system drains `MessageReader<AdjacencyObserved>` each frame/tick into `MatrixKnowledge` (lives in `notebook.rs` from task 019, or a new `src/confirmation.rs` if `notebook.rs` is getting large — reuse `NotebookPlugin` unless there's a good reason to split).
- [ ] Unit tests (pure logic, no Bevy `App` needed — construct `MatrixKnowledge` and feed it hand-built `AdjacencyObserved` values directly): confirms GDD §7's own worked numbers — 3 isolated observations (`n_confounders = 0`, weight 1.0 each) reach the threshold exactly; observations with 3 confounders (weight 0.25 each) need 12 to reach the same threshold; a cell just under threshold is not confirmed, one more observation that crosses it becomes confirmed.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | Where `MatrixKnowledge` and its update system likely live (extends task 019's module) |
| `src/config.rs` | `NotebookConfig` — already has `confirmation_threshold` (3.0) and `observation_weight_numerator` (1.0), no changes needed |
| `src/world.rs` | `TagMatrix::get`, `TagId` — read for sign revelation once confirmed |
| `src/sim.rs` | Source of `AdjacencyObserved` (task 018) |

---

## 🧩 Technical Context

`SimConfig::notebook` (`src/config.rs`) was already added ahead of this task with exactly the two constants GDD §5.9's "Notebook" table specifies — this task is purely about wiring the accumulation logic and events to those existing constants, not inventing new config surface.

`TagMatrix` (`src/world.rs`) is `pub(crate)` on its internals but exposes `get(exerter, receiver) -> i8` publicly, and is indexed by `TagId.0` directly over the *active* tag range — `MatrixKnowledge` should mirror that same `size = active_tags.len()`, `exerter * size + receiver` layout so the two structures stay trivially parallel (task 021's UI will likely iterate both side by side).

---

## 🔨 Suggested Implementation

1. In `notebook.rs`:
   ```rust
   #[derive(Resource)]
   pub struct MatrixKnowledge {
       size: usize,
       evidence: Vec<f32>,
   }

   impl MatrixKnowledge {
       pub fn new(active_tags: usize) -> Self {
           Self { size: active_tags, evidence: vec![0.0; active_tags * active_tags] }
       }

       pub fn record(&mut self, exerter: TagId, receiver: TagId, weight: f32) {
           let idx = exerter.0 as usize * self.size + receiver.0 as usize;
           self.evidence[idx] += weight;
       }

       pub fn evidence(&self, exerter: TagId, receiver: TagId) -> f32 {
           self.evidence[exerter.0 as usize * self.size + receiver.0 as usize]
       }

       pub fn is_confirmed(&self, exerter: TagId, receiver: TagId, threshold: f32) -> bool {
           self.evidence(exerter, receiver) >= threshold
       }
   }
   ```
2. A system `accumulate_evidence(mut knowledge: ResMut<MatrixKnowledge>, config: Res<SimConfig>, mut observed: MessageReader<AdjacencyObserved>)` that computes `weight` per the GDD formula and calls `knowledge.record(...)` for each drained event.
3. Initialize `MatrixKnowledge` sized to `world.active_tags.len()` in a `Startup`/`OnEnter` system once `SimWorld` exists (mirrors how `world.rs`'s `spawn_world` runs at `Startup`) — `init_resource` won't work here since the size depends on runtime config, so `insert_resource` from a system that reads `SimWorld`.
4. Unit tests directly instantiate `MatrixKnowledge::new(n)` and call `record`/`evidence`/`is_confirmed` — no `AdjacencyObserved`/Bevy plumbing required at the test level, keeping this fast and headless like the rest of the codebase's logic tests.

---

## ⚠️ Constraints and Caveats

- Don't build the hypothesis-grid UI here — task 021 owns rendering `MatrixKnowledge`. This task's surface is the resource + accumulation system + tests.
- Evidence must never decay or reset mid-run (GDD §7 doesn't describe a decay mechanic) — a fresh `MatrixKnowledge` only comes from a new world (`r` key / new run), matching how `world.matrix` itself resets.
- Reseeding the world (`r` key, `input.rs::reseed_world`) generates a *new* matrix — `MatrixKnowledge` must be reset alongside it, or old confirmed cells would apply to a matrix that no longer exists. Check whether `reseed_world` needs a small addition to also reset/reinsert `MatrixKnowledge`.

---

## 🔗 Dependencies

- **Depends on**: 018
- **Blocks**: 021

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/020-hypothesis-confirmation-engine.md)"$'\n\nExecute this task in the current project.'
```
