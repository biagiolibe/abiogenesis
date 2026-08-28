// Deliberate exception to "one module = one Plugin": these are shared type
// definitions with no systems of their own, used by main.rs (registration),
// sim.rs (run condition + era-end transition), and input.rs (player intents).

// `GameState::MainMenu` and `EraState::Planning` are unreachable in Phase 0
// (no real main menu yet; planning has no actions to spend). Both are part
// of the task 007 contract and become reachable in later phases.
#![allow(dead_code)]

use bevy::prelude::*;

/// Top-level app state (TECH_DESIGN.md §2). `Loading` and `MainMenu` are
/// stubs in Phase 0 — `MainMenu` becomes real in Phase 3. `WorldCleared` and
/// `Defeat` are Phase 3 additions (task 035): interstitials for, respectively,
/// "the world's objective was met, advance to the next one" and "the run
/// ended" (GDD §8) — the run itself is endless-until-failure, there is no
/// terminal "Victory" state. Both are unreachable until task 041 (failure
/// conditions) and task 045 (world transition) wire them up.
///
/// `WorldFailed` (task 051) is a third, narrower interstitial: only
/// `FailureReason::TotalExtinction` lands here, not `EraBudgetExhausted`.
/// Losing every organism doesn't end the run — the player retries the exact
/// same world (same seed) — while running out the era budget still does,
/// via `Defeat`. See `objectives.rs::evaluate_current_objective`.
///
/// `Intro` (task 052) is a one-time framing interstitial reachable only
/// between `MainMenu`'s "New run" and `Playing`, gated by
/// `MetaProgress.seen_intro` — every run after the first (within the same
/// process session) skips straight to `Playing`.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Intro,
    Playing,
    WorldCleared,
    WorldFailed,
    Defeat,
}

/// Mirrors the player-facing loop of GDD §16.4: Plan → Advance → Observe.
/// Only exists while `GameState::Playing` is active.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Playing)]
pub enum EraState {
    Planning,
    #[default]
    Observing,
    Advancing,
    /// Task 140: entered instead of `Observing` the instant an era actually
    /// completes (not every season) — the world halts on its own and shows
    /// the end-of-era reveal card (`screens::era_reveal_screen_ui`) until
    /// the player dismisses it, the same "must be seen before acting again"
    /// gate every action/tick-advance system already applies to `Advancing`.
    Reveal,
}
