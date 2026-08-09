// The headless half of the simulation: no `bevy::render`, no `bevy_egui`
// (TECH_DESIGN.md §5). Exported as a library so integration tests in
// `tests/` exercise it exactly as `main.rs` does, without building a Bevy
// `App` — proof, not assertion, that the sim is render-independent.

pub mod cluster;
pub mod config;
pub mod objectives;
pub mod run;
pub mod sim;
pub mod state;
pub mod world;
pub mod worldgen;
