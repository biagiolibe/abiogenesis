# Task 016 — Environmental diffusion

> **ID**: `016`
> **Category**: Feature
> **Priority**: 🟢 P3
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Implement the slow diffusion of environmental scalars (GDD §5.2, "Phase 1+: slow diffusion of scalars, so environmental interventions propagate over time") and actually call it from the tick. This is the only Phase 1 item fully independent of tags/matrix/metabolisms — it can be done any time after task 005.

---

## 📋 Acceptance Criteria

- [ ] `SimWorld::diffuse_environment` (currently an empty stub in `src/world.rs`, with a comment marking it Phase 1+) blends each of `temperature`, `light`, `toxicity` toward the mean of its Moore neighbours, at rate `config.environment.diffusion_rate` per tick.
- [ ] `sim::step` actually calls it — today it never does, despite the stub existing.
- [ ] A uniform field (all cells identical) is a fixed point: diffusion leaves it unchanged.
- [ ] A single perturbed cell smooths toward its neighbours' values over successive ticks, without overshoot or oscillation (each scalar stays in `[0,1]`).
- [ ] Diffusion doesn't touch the RNG and doesn't break determinism (same seed ⇒ same environment trajectory).
- [ ] Diffusion uses **double buffering** consistent with the rest of `step` (read neighbour values from the snapshot `world.cells`, write into `world.scratch`), not in-place mutation which would make the result depend on iteration order.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `SimWorld::diffuse_environment` |
| `src/sim.rs` | `step()` — call site |

---

## 🧩 Technical Context

- **Current behavior**: `diffuse_environment` exists with the doc comment *"Not active in Phase 0: gradients are static, so the environment stays a fixed target while the tick algorithm is tuned"* and an empty body. `sim::step` never calls it at all.
- **Desired behavior**: each tick, every scalar drifts a little toward its Moore neighbourhood's average, so environmental stress actions (Phase 2) or matrix-driven changes propagate spatially instead of staying pinned to a single cell forever.

### Why this must double-buffer like the rest of the tick

`world.scratch` already starts each tick as a copy of `world.cells` (`world.scratch.copy_from_slice(&world.cells)`), and organism/residue fields get overwritten afterward. Diffusion should compute each cell's new scalar value from **neighbours in the snapshot** (`world.cells`), writing into `world.scratch` — exactly the same discipline already used for organism energy and residue decay. In-place mutation (`world.cells[i].temperature = ...` while iterating) would make a cell's new value depend on whether its neighbours were already updated this pass, breaking order-independence.

---

## 🔨 Suggested Implementation

```rust
fn diffuse_environment(&mut self, config: &SimConfig) {
    let rate = config.environment.diffusion_rate;
    for y in 0..self.height {
        for x in 0..self.width {
            let idx = self.index(x, y);
            let neighbours: Vec<usize> = self.moore_neighbours(x, y).collect();
            let n = neighbours.len() as f32;
            let mean = |get: fn(&Cell) -> f32| {
                neighbours.iter().map(|&i| get(&self.cells[i])).sum::<f32>() / n
            };
            let cell = &self.cells[idx];
            self.scratch[idx].temperature = cell.temperature + rate * (mean(|c| c.temperature) - cell.temperature);
            self.scratch[idx].light = cell.light + rate * (mean(|c| c.light) - cell.light);
            self.scratch[idx].toxicity = cell.toxicity + rate * (mean(|c| c.toxicity) - cell.toxicity);
        }
    }
}
```

Call it from `sim::step`, early (it only needs the snapshot, so ordering relative to the organism loop doesn't matter as long as both write into the same `scratch` before the final swap):

```rust
world.diffuse_environment(config);
```

Tests in `src/world.rs`: uniform field stays uniform after N ticks; a single hot/bright/toxic cell in an otherwise uniform field decreases in magnitude and its neighbours increase, over several calls; values stay within `[0,1]` (reuse the existing `environment_scalars_stay_in_unit_range` test's assertions as a template, run after several `step` calls instead of just at construction).

---

## ⚠️ Constraints and Caveats

- **Invariant 3**: `diffusion_rate` from `SimConfig` (already defined, task 002) — no new constant.
- Border cells have fewer than 8 neighbours (`moore_neighbours` already handles clipping, task 003) — the mean must divide by the *actual* neighbour count, not a hardcoded 8.
- Don't diffuse `residue` — it's not an "environmental scalar" in the GDD §5.2 sense; its own decay (task 005) is a separate, already-implemented mechanic.

---

## 🔗 Dependencies

- **Depends on**: 004
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/016-environmental-diffusion.md)"$'\n\nExecute this task in the current project.'
```
