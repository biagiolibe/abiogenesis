# Task 092 — Isolation hint duration should scale with era length

> **ID**: `092`
> **Category**: Bugfix / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~30min
> **Assigned to**: unassigned
> **Session**: 2026-08-10, user-reported live: "the 'you isolated this
> species' tooltip doesn't disappear after the era passes" — correctly
> self-diagnosed as a task 082 side effect before this task was scoped.

---

## 🎯 Objective

`ui.rs`'s `ISOLATION_HINT_DURATION_TICKS = 30` is a fixed absolute tick
count, never tied to era length. Before task 082, `era_ticks = 25` for
every era, so a 30-tick hint window read as "about one era" — close enough
to feel anchored to the era boundary. Task 082 shortened `world_index ==
0`'s first `onboarding_eras` (`3`) eras to `onboarding_era_ticks = 8` each,
so the same fixed 30-tick window now spans three-and-a-bit onboarding eras
instead of roughly one — the hint no longer reads as "goes away with the
era," it just lingers through most of the player's opening moves, which is
exactly what got reported as "doesn't disappear."

The fix isn't lengthening or shortening the fixed constant — it's making
the hint's lifetime track whatever era length is *actually active* when
it's shown, the same way task 082 itself made `era_ticks_for` era-aware
instead of reading `config.time.era_ticks` directly.

---

## 📋 Acceptance Criteria

- [x] The isolation hint's effective display duration is derived from
      `worldgen::era_ticks_for(world_index, era, config)` at the moment
      it's shown (`input.rs::seed_organism_on_click`, where
      `isolation_hint.shown_at_tick` is currently set), not the fixed
      `ISOLATION_HINT_DURATION_TICKS` constant — e.g. store the computed
      duration alongside `shown_at_tick` on `IsolationHint`, or compute the
      dismiss tick directly at set-time.

      `IsolationHint` gained a `duration_ticks: u64` field, set alongside
      `shown_at_tick` from `era_ticks_for(run_progress.world_index,
      world.era, &config)`. `seed_organism_on_click` was already at Bevy's
      per-system param limit, so the three isolation-hint-related params
      (`isolation_hint`, `meta`, the new `run_progress`) got bundled into a
      `#[derive(SystemParam)]` struct, `IsolationHintParams` — same pattern
      `objectives.rs`'s `ObjectiveOutcomeParams` already uses.
- [x] `isolation_hint_active` (`ui.rs:456`, currently pure and unit-tested)
      stays pure and unit-testable — update its signature/tests to take
      whatever duration value the new derivation produces, rather than
      hardcoding `ISOLATION_HINT_DURATION_TICKS` inside it.

      Now `isolation_hint_active(shown_at_tick, duration_ticks,
      current_tick)`; existing tests updated to pass an explicit duration,
      plus a new `isolation_hint_active_respects_a_shorter_duration` test
      proving an 8-tick duration dismisses sooner than a 25-tick one for the
      same `shown_at_tick`.
- [x] Decide and document a coherent rule for what happens if the hint is
      shown near the end of a short onboarding era and the *next* era has a
      different length (e.g. onboarding era 2 → standard-length era 3) —
      does the hint's window stay pinned to the era it was shown in, or
      re-derive per-frame against whatever era is current? Pick one, note
      why, since this is exactly the kind of edge case that reads as a new
      bug if left ambiguous.

      Pinned to the era it was shown in: `duration_ticks` is computed once
      at set-time and never re-derived, documented on `IsolationHint`'s own
      doc comment. A hint that started in an 8-tick onboarding era resolves
      on that original short schedule even if a longer standard era begins
      before it dismisses — it doesn't retroactively gain more lifetime
      just because the surrounding era got longer.
- [x] `run_flow.rs`'s existing world-transition `IsolationHint` reset
      (`advancing_to_the_next_world_clears_a_stale_isolation_hint` test)
      keeps passing unmodified — this task changes *duration*, not the
      existing stale-hint-on-world-transition guard.
- [x] `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test` clean.
- [x] Verified live via `cargo run` on a fresh world 0: the hint now visibly
      resolves within roughly the onboarding era it was shown in, not three
      eras later.

      Confirmed by the user (2026-08-10): "funziona."

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `ISOLATION_HINT_DURATION_TICKS` (`~128`), `isolation_hint_active` (`~456`), `viewport_hint` (`~472`) — display/dismiss logic. |
| `src/input.rs` | `seed_organism_on_click` (`~292`, `isolation_hint.shown_at_tick = world.tick`) — where the hint is armed; needs to also record whatever duration basis this task adds. |
| `src/worldgen.rs` | `era_ticks_for` (`~101`) — the existing era-length source of truth to reuse, not reinvent. |

---

## 🧩 Technical Context

**Current behavior**: `IsolationHint { text: Option<&'static str>,
shown_at_tick: u64 }` (`ui.rs:27-29`). `seed_organism_on_click` sets
`shown_at_tick = world.tick` on the player's first isolated placement.
`viewport_hint` clears `isolation_hint.text` once
`current_tick.saturating_sub(shown_at_tick) >=
ISOLATION_HINT_DURATION_TICKS` (`30`, fixed). `world.tick` is a monotonic
counter never reset within a world (only `run_flow.rs::start_world` resets
it, at a genuine world transition — confirmed no era-boundary reset exists,
so this is not a reset bug, purely a duration-vs-era-length mismatch).

**Desired behavior**: the hint's window scales with the era it was shown
in — long enough to read the organism's energy over "a few ticks" (the
hint's own copy, `text.rs:249`), short enough to still resolve within
roughly that era, at any era length (onboarding or standard).

---

## 🔨 Suggested Implementation

1. Add a `duration_ticks: u64` field to `IsolationHint` (or compute
   `dismiss_at_tick` directly), populated from `era_ticks_for(world_index,
   world.era, config)` at the same call site that sets `shown_at_tick`.
2. Update `isolation_hint_active` to take the duration as a parameter
   instead of the module constant; update its existing unit tests
   (`isolation_hint_active_within_and_at_the_edge_of_its_window`,
   `isolation_hint_active_never_underflows_if_current_tick_precedes_shown_at`)
   to pass an explicit duration.
3. Decide the cross-era-boundary edge case (see acceptance criteria) and
   document the choice in a doc comment.
4. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`, then a live
   `cargo run` check on a fresh world 0.

---

## ⚠️ Constraints and Caveats

- Don't remove `ISOLATION_HINT_DURATION_TICKS` if it's still useful as a
  fallback/default for contexts without a clean `world_index`/`era` at
  hand — implementer's call whether it survives as a named default or is
  fully replaced by the derived value.
- This is presentation/onboarding-pacing tuning, not a simulation change —
  stays entirely in `ui.rs`/`input.rs`, no `sim.rs`/`world.rs` involvement.

---

## 🔗 Dependencies

- **Depends on**: 082 (the onboarding era shortening this compensates for),
  055 (original isolation hint feature).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/092-isolation-hint-duration-scales-with-era.md)"$'\n\nExecute this task in the current project.'
```
