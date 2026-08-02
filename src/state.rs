// Deliberate exception to "one module = one Plugin": these are shared type
// definitions with no systems of their own, used by main.rs (registration),
// sim.rs (run condition + era-end transition), and input.rs (player intents).

// `GameState::MainMenu` and `EraState::Planning` are unreachable in Phase 0
// (no real main menu yet; planning has no actions to spend). Both are part
// of the task 007 contract and become reachable in later phases.
#![allow(dead_code)]

use bevy::prelude::*;

/// Top-level app state (TECH_DESIGN.md §2). `Loading` and `MainMenu` are
/// stubs in Phase 0 — `MainMenu` becomes real in Phase 3.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
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
}
