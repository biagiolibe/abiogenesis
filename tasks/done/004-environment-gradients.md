# Task 004 — Environment: static gradients

> **ID**: `004`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Populate each cell's environmental scalars with Phase 0's **static gradients**, creating **spatial heterogeneity → niches**.

This isn't decoration: it's one of the GDD §5.8 anti-degeneration levers. Without environmental heterogeneity the system collapses into the two boring outcomes ("everything dies" / "one species dominates"), which the GDD flags as the project's number-one risk.

---

## 📋 Acceptance Criteria

- [ ] Every cell has `temperature`, `light`, `toxicity` in `[0,1]`.
- [ ] **Light**: `0.9` on the top row → `0.2` on the bottom row, linearly interpolated.
- [ ] **Temperature**: `0.2` on the leftmost column → `0.8` on the rightmost, linearly interpolated.
- [ ] **Toxicity**: `0.7` in a defined zone, `0.0` elsewhere.
- [ ] Values at the grid extremes exactly match the GDD §5.9 table (covered by a test).
- [ ] Generation is deterministic: same seed ⇒ same environment.
- [ ] The integration point for diffusion (Phase 1+) exists, not implemented.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | Environment generation, inside or next to `SimWorld::new` |
| `src/config.rs` | Gradient values (task 002) |

---

## 🧩 Technical Context

- **Current behavior**: `SimWorld` allocates the grid (task 003), but environmental scalars are all zero.
- **Desired behavior**: a grid with gradients that create spatial niches.

### GDD §5.2 — Environmental layer

> **Phase 0:** static gradients (e.g., high light at the top, temperature on a different axis) to create spatial heterogeneity → niches.
> **Phase 1+:** slow diffusion of scalars (averaging with neighbors at a low rate), so environmental interventions propagate over time.

The two gradients are **on different axes on purpose**: light vertical, temperature horizontal. This crossing is what generates distinct two-dimensional niches — a cold-loving photolithic species thrives at the top-left, a heat-loving one at the top-right.

### Why the dark band matters

With `metabolism_gain = 2.0` and `upkeep = 0.5` (GDD §5.9), a photolithic organism with `env_fit ≈ 1`:
- at `light = 0.7` gains `1.4` → net `+0.9`/tick, **grows**;
- at `light = 0.2` gains `0.4` < `0.5` upkeep → **doesn't survive**.

The survival threshold sits around `light = 0.25`. The bottom rows must therefore stay below that value: they're the **light niche**, verified by task 009.

---

## 🔨 Suggested Implementation

1. Linear interpolation along the relevant axis. Watch out for the `height == 1` case (division by zero) even though it doesn't occur at `48×32`:

   ```rust
   /// Static Phase 0 gradients: light falls top→bottom, temperature rises left→right.
   /// The two axes differ on purpose: their crossing is what creates 2D niches (GDD 5.2).
   fn apply_gradients(&mut self, config: &SimConfig) {
       let env = &config.environment;
       for y in 0..self.height {
           let ty = y as f32 / (self.height - 1).max(1) as f32;
           for x in 0..self.width {
               let tx = x as f32 / (self.width - 1).max(1) as f32;
               let cell = &mut self.cells[y * self.width + x];
               cell.light = lerp(env.light_top, env.light_bottom, ty);
               cell.temperature = lerp(env.temp_left, env.temp_right, tx);
               cell.toxicity = 0.0;
           }
       }
   }
   ```

2. **Toxic zone.** In Phase 0 a fixed, readable zone is enough — a rectangle in a corner, sized from `SimConfig`. Procedural generation of extreme zones is Phase 3 (GDD §9). Keep it far from the more fertile bright band, so it doesn't skew task 009's tests.

3. **Diffusion integration point** — declare it without implementing it, so Phase 1 knows where to hook in:

   ```rust
   /// Phase 1+: blend each scalar toward its neighbours' mean at `diffusion_rate`
   /// per tick (GDD 5.2). Not active in Phase 0: gradients are static.
   fn diffuse_environment(&mut self, _config: &SimConfig) {
       // Intentionally empty in Phase 0.
   }
   ```

4. **Tests**: `light` on row 0 is `0.9` and on the last row `0.2`; `temperature` on column 0 is `0.2` and on the last column `0.8`; all scalars stay in `[0,1]`; toxic-zone cells equal `0.7` and the rest `0.0`.

---

## ⚠️ Constraints and Caveats

- **All scalars must stay in `[0,1]`** (GDD §5.2): it's the assumption the tick formulas rest on.
- Values come from `SimConfig`, **not hand-written here** (invariant 3).
- **No diffusion in Phase 0**: gradients are static. Implementing it now would make the environment a moving target right while the tick is being tuned.
- The toxic zone has no effect on the simulation yet: no metabolism reads it in Phase 0. That's correct — it's there to make the environment's structure visible and will be used from Phase 1 onward.

---

## 🔗 Dependencies

- **Depends on**: 003
- **Blocks**: 005

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/004-environment-gradients.md)"$'\n\nExecute this task in the current project.'
```
