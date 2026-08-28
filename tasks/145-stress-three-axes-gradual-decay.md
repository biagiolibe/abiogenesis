# Task 145 — Stress on three selectable axes + gradual decay

> **ID**: `145`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~2h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

`Stress` today only ever shifts temperature (`input.rs::stress_on_click`,
`config.environment.stress_delta`), an asymmetry nobody chose deliberately —
light and toxicity matter just as much for `Photolithic` and
`Chemolithotroph` respectively, but the player has no direct lever on either.
Turn `Stress` into **one action with three selectable axes** (thermal /
light / toxicity) — same slot in the roster, same icon, same budget cost —
and make a single application **temporary**: the shifted scalar decays back
toward the cell's own pre-stress baseline over a handful of ticks, instead
of the permanent shift it is today.

Design source: `redesign/processed/abiogenesis-actions.md`, "Stress —
corretto"; GDD §6 (`Environmental stress`, currently `[PROPOSED]` for the
axis choice) and §6's `[CORRECTION]` note on per-cell scope (unaffected,
already correct).

**Explicitly out of scope**: the doc's second consequence tier — repeated
Stress on the same cell across eras forcing a permanent biome transition.
That depends on the dynamic-biomes trigger mechanism, which doesn't exist
yet (task 164, Phase 5). Build the decay so it's the natural foundation for
that later hook (a per-cell "how long has this been held stressed" signal
would slot on top), but don't build the accumulation or the transition
itself now.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] `ActionMode::Stress` gains a selectable axis (thermal / light /
      toxicity) surfaced in the UI while Stress is the active action —
      same pattern the codebase already uses for an in-progress action
      config (see `SpliceEdit`/the Splice panel in `ui.rs`), not a new
      top-level `ActionMode` variant. Persisted as its own `Resource`
      (e.g. `SelectedStressAxis`), defaulting to thermal so existing
      keybind/click behaviour (`1`-`4` quick-select, left-click) is
      unchanged for a player who never touches the new selector.
- [ ] `stress_on_click` (`input.rs`) shifts `cell.temperature`,
      `cell.light`, or `cell.toxicity` depending on the selected axis,
      each clamped to `[0,1]`, using a per-axis delta config (see below) —
      not a single shared `stress_delta` applied to whichever field.
- [ ] A stressed scalar decays back toward the cell's own **pre-stress
      value** over subsequent ticks, distinct from and in addition to
      `diffuse_environment`'s neighbour-blur (which alone does not pull an
      isolated cell back to where it started — only source-adjacent cells
      get that today, via `reinject_environment_sources`). New per-cell
      baseline state + a new tick step that exponentially relaxes the
      current value toward it at a configured rate.
- [ ] Decay must not fight `reinject_environment_sources` on cells it
      already pulls (heat sources, sea-coolant band, toxic Swamp cells):
      either skip decay on those cells for the affected axis, or make the
      baseline for them the source-driven value rather than the
      pre-stress one — pick whichever keeps the existing
      `reinject_environment_sources` invariants (`reinject_strength >
      diffusion_rate`, etc.) meaningful. Document the choice.
- [ ] New config lives in `EnvironmentConfig` (or a small owned substruct),
      no magic numbers: per-axis stress delta, decay rate. `stress_delta`
      is replaced/renamed rather than kept alongside dead duplicates.
- [ ] `sim_config.ron` stays in sync (`tests/config_ron_sync.rs`).
- [ ] At least one unit/integration test exercises: (a) a stressed cell's
      scalar visibly shifts on the chosen axis and not the others; (b) left
      alone, it measurably relaxes back toward its pre-stress value over N
      ticks without external interference (isolated cell, no source nearby).
- [ ] GDD §6's `Stress` entry: axis selection flips from `[PROPOSED]` to
      `[DECIDED]` once implemented; note the temporary/decay behaviour.
      Player guide updated if it documents Stress's current single-axis
      behaviour.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `stress_on_click` — reworked to read the selected axis and apply a per-axis delta. |
| `src/ui.rs` | `ActionMode`, `SelectedAction`, action bar (`~1075`) — add the axis sub-selector while Stress is active. |
| `src/config.rs` | `EnvironmentConfig` — replace `stress_delta` with per-axis deltas + a decay-rate config. |
| `src/world.rs` | `diffuse_environment`, `reinject_environment_sources` (`~2151`-`2220`) — new decay-toward-baseline step lives alongside these; per-cell baseline storage. |
| `src/sim.rs` | Tick pipeline (task 138's explicit phases) — wire the new decay step into the right phase. |
| `sim_config.ron` | Mirror any new/renamed `EnvironmentConfig` fields. |
| `abiogenesis-gdd.md` | §6 — flip axis selection to `[DECIDED]`. |

---

## 🧩 Technical Context

- **Current behavior**: `Stress` always shifts `cell.temperature` by a fixed
  `config.environment.stress_delta` (0.3), clamped to `[0,1]`, and the shift
  is permanent except for whatever `diffuse_environment`'s neighbour-blur
  does to it incidentally (blurs toward the local mean, not back toward the
  cell's own prior value — an isolated stressed cell that's far from any
  heat/coolant source stays shifted indefinitely).
- **Desired behavior**: player picks an axis (thermal/light/toxicity) before
  clicking; the click shifts that scalar only; over subsequent ticks the
  scalar relaxes back toward what it was immediately before the stress was
  applied, at a tunable rate — a single application is tactical and
  self-erasing, repeated applications on the same cell keep it pinned away
  from baseline for as long as the player keeps paying for it (budget is
  already the natural brake, per the design doc — no extra cooldown needed).
- `light` and `toxicity` are already live tick inputs per the GDD §6 note
  quoted above (`toxicity` line 111: "temperature, light, and toxicity are
  all read in the tick loop today") — this task doesn't need to make either
  scalar meaningful first, only needs a per-axis delta and a baseline/decay
  mechanism that treats all three uniformly.

---

## 🔨 Suggested Implementation

1. Add per-cell baseline storage (grid-sized `Vec<f32>` per axis, or a small
   struct bundling all three — mirror `SimWorld::stall_ticks`'s shape).
   Populate it once at world generation / world reset, and update it to the
   *current* value whenever a cell is **not** actively decaying (so a slow
   natural drift from other systems — e.g. seasonal light cycles, if any —
   doesn't get read as "stress" and erased).
2. In `stress_on_click`, before applying the delta, snapshot the cell's
   current value on the chosen axis as the new decay target if it isn't
   already mid-decay (repeated stress on the same cell within its decay
   window shouldn't reset the target back to an already-shifted value).
3. Add a tick step (near `diffuse_environment`/`reinject_environment_sources`
   in `world.rs`, called from `sim.rs`'s pipeline) that pulls each
   axis-value a fraction of the way toward its stored baseline, skipping
   cells `reinject_environment_sources` already owns for that axis.
4. UI: a small always-visible (only while `ActionMode::Stress` is selected)
   3-way toggle, `SelectedStressAxis` resource read by both `ui.rs` (render)
   and `input.rs` (apply).
5. Rename/replace `stress_delta` in `EnvironmentConfig` with per-axis values
   (can share one number initially if no design reason to differ — leave a
   comment either way) plus `stress_decay_rate`. Update `sim_config.ron`.
6. Tests: isolated-cell stress-then-relax on each axis; confirm a
   source-adjacent cell's decay doesn't fight existing reinjection tests.

---

## ⚠️ Constraints and Caveats

- **Determinism**: no RNG involved; keep the decay step reading `scratch`/
  writing in the same double-buffer pattern `diffuse_environment` uses if
  order-independence matters (check whether sequential in-place update is
  safe here or needs the same scratch-buffer treatment).
- **No magic numbers**: every new constant into `EnvironmentConfig`.
- Don't build the era-spanning permanent-transition mechanic — no dynamic
  biomes system exists to hook it to yet (task 164).
- Keep `sim`/`world`/`config` free of `bevy::render`/`bevy_egui` deps per
  `TECH_DESIGN.md` §5 — the axis selector's `Resource` type belongs in
  `ui.rs`, only read (not defined) by `input.rs`.

---

## 🔗 Dependencies

- **Depends on**: 136 (environment already tick-relevant on all three axes),
  138 (explicit tick pipeline to hook the new decay step into).
- **Blocks**: none directly; the decay/baseline mechanism this task builds
  is the natural foundation task 164 (Phase 5, dynamic biomes) will extend
  with an accumulation-toward-transition signal, but 164 isn't blocked on
  145 being done first — it's just easier once this exists.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/145-stress-three-axes-gradual-decay.md)"$'\n\nExecute this task in the current project.'
```
