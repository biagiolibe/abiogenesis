mod config;
mod input;
mod render;
mod sim;
mod ui;
mod world;

use bevy::prelude::*;
use config::ConfigPlugin;
use input::InputPlugin;
use render::GridRenderPlugin;
use sim::SimPlugin;
use ui::UiPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Abiogenesis".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            ConfigPlugin,
            WorldPlugin,
            SimPlugin,
            GridRenderPlugin,
            UiPlugin,
            InputPlugin,
        ))
        .run();
}
