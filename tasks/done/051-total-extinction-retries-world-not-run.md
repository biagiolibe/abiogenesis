# Task 051 — Total extinction retries the world, not the whole run

> **ID**: `051`
> **Category**: Bugfix / Design
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-06/07 playtest follow-up (post task 050)

---

## 🎯 Objective

Task 050 removed auto-placed starting organisms — the player now seeds the
first world's organisms themselves. This exposed a design cliff: if the
player seeds only one or two organisms and they die within the first era,
`SimWorld::ever_populated && no living organism` trips
`FailureReason::TotalExtinction`, which currently sends the player straight
to `GameState::Defeat` — ending the *entire run* (`meta.absorb`, back to the
main menu) over a single early misstep, often in the first minute of play.

Before task 050 two organisms were auto-placed and both had to die to
trigger this; now one bad click can end the run. Decision (discussed with
the user): total extinction should end the *world* the player is on, not the
run — the player retries that same world, the run itself continues.
`EraBudgetExhausted` (failing to meet the objective in time) is unaffected —
that remains a real run-ending failure.

---

## 📋 Acceptance Criteria

- [x] `WorldOutcome::Failed(FailureReason::TotalExtinction)` no longer
      transitions to `GameState::Defeat`. It transitions to a new
      `GameState::WorldFailed` interstitial instead, and does **not** call
      `MetaProgress::absorb` (the run hasn't ended).
- [x] `WorldOutcome::Failed(FailureReason::EraBudgetExhausted)` keeps its
      current behavior unchanged: `GameState::Defeat`, `meta.absorb` runs.
- [x] `GameState::WorldFailed` shows a screen with a "Retry" action that
      rebuilds the *same* `world_index` from the *same* seed
      (`run_progress.world_seed`) — an exact do-over of the world just lost,
      not a new one, and not counted toward `worlds_cleared`.
- [x] `cargo clippy --all-targets -- -D warnings` and `cargo test` stay clean.
- [x] Manually verified via `cargo run`: seed one organism, let it die,
      confirm "World failed" appears (not "Run ended"), Retry returns to the
      same world with the same seed/objective, playing normally.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/state.rs` | Add `GameState::WorldFailed` variant. |
| `src/objectives.rs` | `evaluate_current_objective`: branch on `FailureReason` instead of transitioning both to `Defeat`. |
| `src/run_flow.rs` | Add `retry_world`: rebuild the same `world_index`/seed via `start_world`, no `worlds_cleared` bump. |
| `src/screens.rs` | Add `world_failed_screen_ui` interstitial, wire into `ScreensPlugin`. |
| `src/text.rs` | Add `WORLD_FAILED_TITLE`, `RETRY_BUTTON`, `world_failed_body()`. |

---

## ⚠️ Constraints and Caveats

- Determinism: retrying must reproduce the identical world (same active
  tags, matrix, objective) — `build_world(seed, world_index, config, bonus)`
  with the same seed is already pure/deterministic, so this falls out for
  free as long as `retry_world` doesn't touch `run_progress.world_seed`.
- Style: follow `TECH_DESIGN.md` conventions; `run_flow.rs` stays a
  Plugin-less exception per its existing header comment.
