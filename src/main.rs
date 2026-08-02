mod input;
mod render;
mod ui;

use abiogenesis::config::ConfigPlugin;
use abiogenesis::sim::SimPlugin;
use abiogenesis::state::{EraState, GameState};
use abiogenesis::world::WorldPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use input::InputPlugin;
use render::GridRenderPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Abiogenesis".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins((
            ConfigPlugin,
            WorldPlugin,
            SimPlugin,
            GridRenderPlugin,
            UiPlugin,
            InputPlugin,
        ))
        .init_state::<GameState>()
        .add_sub_state::<EraState>()
        .add_systems(OnEnter(GameState::Loading), enter_playing)
        .run();
}

/// Phase 0: `Loading` transitions straight to `Playing`; `MainMenu` becomes
/// real in Phase 3 (TECH_DESIGN.md §2).
fn enter_playing(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}
