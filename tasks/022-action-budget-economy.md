# Task 022 — Action budget economy

> **ID**: `022`
> **Category**: Architecture / Feature
> **Priority**: 🔴 P1
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Make GDD §6's action budget real: 3 points per era, `Seed` now costs 1 and is blocked at 0. This is the foundation Track B's other actions (023 Stress, 024 Cull, 025 Splice) build on — it introduces the budget resource and the "action mode" selector pattern they'll all reuse.

**Confirmed design decision** (2026-08-03 planning session): no new `EraState` variant. `EraState::Observing` already doubles as "observe last era's results + plan the next one" — the player spends budget there, same as `Seed` already works today, just no longer free. `EraState::Planning` stays an unreachable stub (per its existing doc comment in `state.rs`); this task does not touch `state.rs`.

---

## 📋 Acceptance Criteria

- [ ] `ActionBudget` resource: `points_remaining: u32`, refilled to `config.time.point_budget_per_era` (already `3` in `SimConfig::default()`) whenever `EraState::Observing` is (re-)entered after an era completes (i.e. the same transition point `advance_tick` already uses when it calls `next_state.set(EraState::Observing)`).
- [ ] The existing `seed_organism_on_click` system (`input.rs`) checks `ActionBudget.points_remaining >= config.time.action_costs.seed` before placing an organism, decrements by that cost on success, and does nothing (not even the empty-cell check) if the budget can't cover it.
- [ ] The HUD (`ui.rs::hud_panel`) shows the current budget, e.g. `"Actions: 2 / 3"`, replacing the "Placeholder: objective and action budget arrive in Phase 3" comment that's now wrong for the budget half (objective display stays a Phase 3 placeholder).
- [ ] Reseeding the world (`r` key) resets the budget to full, same as it resets everything else about a fresh world.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `advance_tick` — the point where a new era starts and the budget should refill |
| `src/input.rs` | `seed_organism_on_click` — gains the budget check/decrement; `reseed_world` — gains a budget reset |
| `src/ui.rs` | `hud_panel` — shows the budget readout |
| `src/config.rs` | No changes needed — `TimeConfig::point_budget_per_era` and `TimeConfig::action_costs` (`ActionCosts { seed: 1, stress: 1, cull: 1, splice: 2 }`) already exist |

---

## 🧩 Technical Context

`SimConfig::time` (`src/config.rs`) was already built out with `point_budget_per_era` and `ActionCosts` ahead of this task — this task wires behavior to config that already exists, it doesn't add new config surface.

`EraProgress` (`src/sim.rs`) is the closest existing precedent for a resource that resets on a well-defined lifecycle event (era start/end) and is read/written across module boundaries (`sim.rs` owns it, `input.rs` reads it) — `ActionBudget` should follow the same shape: owned in `sim.rs` (it's era-lifecycle state, same rationale as `EraProgress`), refilled inside `advance_tick`'s existing "era just ended" branch.

---

## 🔨 Suggested Implementation

1. `sim.rs`:
   ```rust
   #[derive(Resource, Default)]
   pub struct ActionBudget {
       pub points_remaining: u32,
   }

   impl ActionBudget {
       pub fn refill(&mut self, points: u32) { self.points_remaining = points; }
       pub fn try_spend(&mut self, cost: u32) -> bool {
           if self.points_remaining >= cost {
               self.points_remaining -= cost;
               true
           } else {
               false
           }
       }
   }
   ```
2. In `advance_tick`, inside the existing `if progress.remaining() == 0 { world.era += 1; next_state.set(EraState::Observing); }` branch, add `budget.refill(config.time.point_budget_per_era);` (needs `mut budget: ResMut<ActionBudget>` added to the system's parameters).
3. Register `ActionBudget` in `SimPlugin::build` via `init_resource`, and call `refill` once at startup too (so the very first `Observing` state has a full budget) — check whether `EraProgress`/`ActionBudget` need an explicit initial refill outside `advance_tick`, since `init_resource::<ActionBudget>()` alone leaves `points_remaining = 0`.
4. `input.rs::seed_organism_on_click`: add `mut budget: ResMut<ActionBudget>` to the system signature; before writing the organism, `if !budget.try_spend(config.time.action_costs.seed) { return; }`.
5. `input.rs::reseed_world`: add `budget.refill(config.time.point_budget_per_era);` alongside the existing reset logic.
6. `ui.rs::hud_panel`: add `budget: Res<ActionBudget>` parameter, render `format!("Actions: {} / {}", budget.points_remaining, config.time.point_budget_per_era)`.
7. Manual verification: seed 3 organisms (consumes the whole budget), confirm a 4th click does nothing; advance an era, confirm the budget shows full again; press `r`, confirm it's full immediately.

---

## ⚠️ Constraints and Caveats

- Do **not** add `EraState::Planning` handling or a new state transition — the confirmed design keeps this in `Observing`, minimizing the blast radius of this task.
- Don't build the Stress/Cull/Splice actions here — this task is budget plumbing plus retrofitting `Seed`. 023/024/025 add the new action types and the mode selector UI.
- `advance_tick` already has a "stray extra `FixedUpdate` tick" guard (see its doc comment) — make sure the budget refill only fires once per era transition, not once per frame while `Observing` is active.

---

## 🔗 Dependencies

- **Depends on**: 017 (existing `Seed` action)
- **Blocks**: 023, 024, 025

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/022-action-budget-economy.md)"$'\n\nExecute this task in the current project.'
```
