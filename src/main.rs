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
                // Fills whatever screen the window opens on (task raised
                // directly by the user: the fixed default window size didn't
                // use the available display). Windowed-but-maximized for now,
                // during development — keeps title bar/decorations so the
                // window can be moved/resized/alt-tabbed normally while
                // iterating; switch `mode` to
                // `WindowMode::BorderlessFullscreen(MonitorSelection::Current)`
                // for the shipped build.
                //
                // `fullsize_content_view` + `titlebar_transparent` (macOS):
                // without these, the default title bar reserves an extra
                // "toolbar" band below the traffic lights, in a visibly
                // different shade from both the title bar and the game's own
                // background — a dead, non-interactive gray strip eating into
                // the grid's vertical space (raised directly by the user,
                // mistaken at first for a grid-proportion bug; confirmed via
                // screenshot diffing that the game's own rendering already
                // filled 100% of the space *below* that OS-reserved band).
                // Drawing content full-size under a transparent title bar
                // removes the band entirely — traffic lights float over the
                // grid instead of sitting above a separate reserved area.
                fullsize_content_view: true,
                titlebar_transparent: true,
                // Deliberately oversized so macOS clamps it to the screen's
                // visible frame at window-creation time, instead of using a
                // runtime `Window::set_maximized` call — combining
                // `set_maximized` with `fullsize_content_view` hits a known
                // winit/macOS bug where the post-maximize `inner_size` under-
                // reports the window's true (reclaimed-titlebar) height,
                // which showed up as the same dead gray band reappearing at
                // the *bottom* of the grid instead of the top. Requesting a
                // size no real display can satisfy sidesteps the buggy
                // runtime resize path entirely: the window is correctly
                // clamped and sized once, at creation.
                resolution: bevy::window::WindowResolution::new(10000, 10000),
                position: WindowPosition::Centered(bevy::window::MonitorSelection::Primary),
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
