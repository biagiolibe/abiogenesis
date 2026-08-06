mod input;
mod menu;
mod notebook;
mod render;
mod run_flow;
mod screens;
mod text;
mod ui;

use abiogenesis::config::ConfigPlugin;
use abiogenesis::objectives::ObjectivesPlugin;
use abiogenesis::run::RunPlugin;
use abiogenesis::sim::SimPlugin;
use abiogenesis::state::{EraState, GameState};
use abiogenesis::world::WorldPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use input::InputPlugin;
use menu::MenuPlugin;
use notebook::NotebookPlugin;
use render::GridRenderPlugin;
use screens::ScreensPlugin;
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
            RunPlugin,
            WorldPlugin,
            SimPlugin,
            ObjectivesPlugin,
            GridRenderPlugin,
            UiPlugin,
            NotebookPlugin,
            InputPlugin,
            MenuPlugin,
            ScreensPlugin,
        ))
        .init_state::<GameState>()
        .add_sub_state::<EraState>()
        .add_systems(OnEnter(GameState::Loading), enter_main_menu)
        .run();
}

/// `Loading` transitions to `MainMenu` (task 044) — the player explicitly
/// starts a run from there (`menu.rs::start_run`) instead of the game
/// booting straight into a world.
fn enter_main_menu(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::MainMenu);
}
