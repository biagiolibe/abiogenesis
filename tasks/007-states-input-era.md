# Task 007 — `GameState`/`EraState`, input, animated era

> **ID**: `007`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: —

---

## 🎯 Objective

Give the player **control over time**: pressing `space` advances an era of `ERA_TICKS = 25` ticks **animated one by one**, then the game returns to waiting.

This task closes Phase 0's core loop. GDD §4's time model isn't a convenience detail: it **makes the time model coincide with the player's mental loop** — plan, execute, observe.

---

## 📋 Acceptance Criteria

- [ ] `GameState` (`Loading` → `MainMenu` → `Playing`) and the `EraState` sub-state (`Planning` / `Advancing` / `Observing`) exist.
- [ ] `space` in `Observing` (or `Planning`) starts an era: transitions to `Advancing`.
- [ ] The era advances by **exactly 25 ticks**, one per fixed frame, **visible as an animation**; then returns to `Observing`.
- [ ] `s` advances by **a single tick**, without going through `Advancing`.
- [ ] `r` regenerates the world with a new seed.
- [ ] `Esc` quits.
- [ ] Advancement inputs are **ignored during `Advancing`** (no eras queued by mistake).
- [ ] The simulation system **doesn't run** outside `Advancing`.
- [ ] Seeding a photolithic organism in a bright zone and pressing `space` shows a bloom growing tick by tick.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `InputPlugin`, key mapping |
| `src/sim.rs` | `SimPlugin`, run condition and `EraProgress` |
| `src/world.rs` | Reset/reseed |
| `src/main.rs` | State registration |

---

## 🧩 Technical Context

- **Current behavior**: `step()` exists and is correct (task 005), the grid is visible (task 006), but nothing invokes it in a controlled way.
- **Desired behavior**: the player governs the passage of time.

### GDD §4 — The era model

> Time advances in **eras**: the player queues one or more actions, then advances the simulation by *N* ticks in a block and observes the result.
>
> - **Animation during the era:** tick advancement is shown tick-by-tick (fast), preserving the feeling of a "breathing system," while *control* remains deliberately step-wise.
> - **Era length:** `ERA_TICKS = 25` as default, adjustable.

### The cycle (GDD §16.4)

```
PLAN (action budget)  →  [SPACE]  →  ADVANCE ERA (25 ticks, animated)  →  OBSERVE & RECORD
      ▲                                                                            │
      └────────────────────────────────────────────────────────────────────────────┘
```

`EraState` maps 1:1 onto this cycle (`TECH_DESIGN.md` §2). In Phase 0, `Planning` is effectively empty — it becomes meaningful in Phase 2, with actions.

### Implementation (`TECH_DESIGN.md` §3.4)

The advancement system runs in **`FixedUpdate`** with a configurable timestep — which here governs the **animation speed**, not the logic speed. An `EraProgress` resource counts remaining ticks; at zero, transition to `Observing`.

---

## 🔨 Suggested Implementation

1. **States**

   ```rust
   #[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
   pub enum GameState {
       #[default]
       Loading,
       MainMenu,
       Playing,
   }

   /// Mirrors the player-facing loop of GDD 16.4.
   #[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
   #[source(GameState = GameState::Playing)]
   pub enum EraState {
       Planning,
       #[default]
       Observing,
       Advancing,
   }
   ```

   In Phase 0, `Loading` can transition directly to `Playing`; `MainMenu` stays a stub (becomes real in Phase 3).

2. **Era counter**

   ```rust
   /// Ticks left in the era currently being animated.
   #[derive(Resource, Default)]
   pub struct EraProgress {
       remaining: u32,
   }
   ```

3. **Starting an era** — on `space`, only if **not** already in `Advancing`:

   ```rust
   fn start_era(/* ... */) {
       progress.remaining = config.time.era_ticks;
       next_state.set(EraState::Advancing);
   }
   ```

4. **Advancement** — in `FixedUpdate`, with a run condition on state:

   ```rust
   app.add_systems(
       FixedUpdate,
       advance_tick
           .in_set(SimSet::Advance)
           .run_if(in_state(EraState::Advancing)),
   );
   ```

   `advance_tick` calls `step`, decrements `remaining`, and at zero, increments `world.era` and transitions to `Observing`.

5. **Timestep.** Set `Time<Fixed>` to a rate that makes the era feel perceptible but fast: **~20 ticks/second** makes an era last about 1.2s, consistent with GDD §4's "fast advancement." The value goes in `SimConfig` — it's a *feel* knob, so it's tunable.

6. **Single tick.** `s` calls `step` once, no state changes: useful for fine observation and debugging (GDD §11).

7. **Reset.** `r` rebuilds `SimWorld` with a new seed. The next seed should be derived **from the current world's RNG**, not the system clock, to avoid introducing non-determinism (invariant 1). Alternatively, an incrementing counter on the initial seed.

8. **Starting seed.** For the animation to show anything, the world needs to start with at least one organism: seed a photolithic organism at the center of the bright band when the world is created. This is a Phase 0 placeholder — the real *seed* action arrives in Phase 1 — so mark it with a comment.

---

## ⚠️ Constraints and Caveats

- **Don't queue eras.** If `space` is pressed during `Advancing`, it must be ignored: the state run condition is the simplest defense.
- **The timestep governs animation, not logic.** Changing it must only alter playback speed, never the simulation's result. If changing the timestep changes the final state, invariant 1 has been violated somewhere.
- **Exactly 25 ticks per era.** Counting elapsed time instead of ticks leads to eras of variable length and destroys reproducibility.
- **No Bevy `Time` inside `step`** (invariant 1). Bevy's time decides *when* to call `step`, never *what* it does.
- `q` as a quit key was planned in GDD v0.3 but removed in v0.4: it's kept free for future text input. `Esc` remains.

---

## 🔗 Dependencies

- **Depends on**: 005, 006
- **Blocks**: 008

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/007-states-input-era.md)"$'\n\nExecute this task in the current project.'
```
