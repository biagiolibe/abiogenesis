# Task 139 — Overview map: real density, and the removal of the pictorial version

> **ID**: `139`
> **Category**: Refactor / Rendering
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-27 redesign adoption planning

---

## 🎯 Objective

Make the Overview view read **real** data — population ÷ carrying capacity per
cell — instead of an artfully drawn approximation, and delete the machinery that
existed only to make that approximation look right.

Design source: `redesign/processed/culture-shock-population-model-aesthetic.md`.

---

## 🧨 This deletes shipped work, on purpose

Tasks 076 and 078 built `cluster::compute_cluster_render`: connected-component
blobs, interior hole-filling (flood-fill the border-connected exterior), and
erosion (`blob_erosion_iterations`, `blob_erosion_min_size`) so a blob read as a
smaller solid abstracted shape rather than a 1:1 trace of the occupied-cell
footprint.

All of that exists **because** density was a pictorial illusion over a model
that did not support it. With task 137's per-cell population, each cell already
carries a real density value. The hole-filling and erosion are not adapted —
they are **removed**. Do not preserve them "just in case"; a half-real,
half-drawn density is worse than either.

What may survive: the connected-component grouping itself, if the Overview still
needs per-species cluster identity for anything (species colour keying, the
Biosphere panel). Check before deleting `cluster.rs` wholesale.

---

## 📋 Acceptance Criteria

- [ ] Overview renders density as `population / carrying_capacity` per cell.
- [ ] `compute_cluster_render`'s hole-filling and erosion are gone, along with
      their config knobs (`ClusterConfig::blob_erosion_*`) and their tests.
- [ ] `render.rs`'s Overview branch of `cell_color` no longer keys off blob
      membership as a stand-in for occupancy — it reads the real per-cell value.
- [ ] An isolated single-individual cell is still visible in Overview (this was
      an explicit property of task 076's design and must not regress).
- [ ] Two zoom levels only, no intermediate stage with its own encoding.
- [ ] `assets/sim_config.ron` updated for any removed field.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] **Live `cargo run` visual check across 2-3 seeds** — this is a rendering
      task, offscreen colour-function checks are not sufficient.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/cluster.rs` | `compute_cluster_render`, `ClusterRender`, erosion/hole-fill. |
| `src/render.rs` | `cell_color`, the Overview/Detail branch, `MapViewMode`. |
| `src/config.rs` | `ClusterConfig` (`density_saturation`, `blob_erosion_*`). |
| `assets/sim_config.ron` | Sync. |

---

## ⚠️ Constraints and Caveats

- Colour encoding stays as it is here. The "colour = environment, shape = life"
  rule and the pixel-grain register are task 151.
- Do not change the zoom threshold or camera behaviour (task 075's work) — only
  what is drawn.

---

## 🔗 Dependencies

- **Depends on**: 137
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/139-overview-real-density.md)"$'\n\nExecute this task in the current project.'
```
