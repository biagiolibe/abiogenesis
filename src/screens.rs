// World-cleared/failed/defeat interstitials (tasks 045, 051): where a run's
// per-world loop closes. `GameState::WorldCleared` (objective satisfied)
// leads to the next, harder world via `start_world`; `GameState::WorldFailed`
// (total extinction, task 051) retries the *same* world via `retry_world`,
// the run continues; `GameState::Defeat` (era budget exhausted, task 041)
// ends the run and returns to the main menu — not back to `Playing`, since a
// run that ended requires going through the menu again (`menu.rs`) to start
// a new one.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use abiogenesis::config::SimConfig;
use abiogenesis::run::{MetaProgress, RunProgress};
use abiogenesis::sim::SeasonProgress;
use abiogenesis::state::{EraState, GameState};
use abiogenesis::world::SimWorld;

use crate::run_flow::{advance_to_next_world, retry_world, WorldResetParams};
use crate::text;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (
                intro_screen_ui.run_if(in_state(GameState::Intro)),
                world_cleared_screen_ui.run_if(in_state(GameState::WorldCleared)),
                world_failed_screen_ui.run_if(in_state(GameState::WorldFailed)),
                defeat_screen_ui.run_if(in_state(GameState::Defeat)),
            ),
        );
    }
}

/// Max height of the intro screen's how-to-play scroll area (task 056) —
/// presentation-only, not a simulation coefficient, same rationale
/// `ui.rs::HUD_WIDTH` gives for its own layout constant.
const INTRO_GUIDE_HEIGHT: f32 = 340.0;

/// One-time framing interstitial (task 052) between `MainMenu`'s "New run"
/// and `Playing`: the world is already built by the time this shows (`menu.
/// rs::start_run` runs first), so unlike the other screens here there's
/// nothing to rebuild on continue — just flip `seen_intro` and move on.
/// Shows the full how-to-play guide (task 056, `text::HOW_TO_PLAY_SECTIONS`)
/// before "Begin" rather than a separate short blurb, since this is exactly
/// the moment a first-time player needs it — the old one-paragraph
/// `INTRO_BODY` duplicated what the guide already says at more length.
fn intro_screen_ui(
    mut contexts: EguiContexts,
    mut meta: ResMut<MetaProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    interstitial(ctx, "intro-viewport", |ui| {
        ui.heading(text::INTRO_TITLE);
        ui.add_space(12.0);
        egui::ScrollArea::vertical()
            .id_salt("intro_how_to_play")
            .max_height(INTRO_GUIDE_HEIGHT)
            .show(ui, |ui| {
                for (heading, body) in text::HOW_TO_PLAY_SECTIONS {
                    ui.strong(*heading);
                    ui.label(*body);
                    ui.add_space(8.0);
                }
            });
        ui.add_space(12.0);
        if ui.button(text::INTRO_CONTINUE_BUTTON).clicked() {
            meta.seen_intro = true;
            next_state.set(GameState::Playing);
        }
    });
    Ok(())
}

/// "Objective met" interstitial: advancing generates world `world_index + 1`
/// (harder, per `worldgen`'s difficulty curve) seeded from the *finishing*
/// world's own RNG (`SimWorld::next_seed`) — the same chaining scheme
/// `RunProgress::start`'s doc comment promises, never `run_seed` again.
#[allow(clippy::too_many_arguments)]
fn world_cleared_screen_ui(
    mut contexts: EguiContexts,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut run_progress: ResMut<RunProgress>,
    mut season_progress: ResMut<SeasonProgress>,
    mut era_next_state: ResMut<NextState<EraState>>,
    mut reset: WorldResetParams,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    interstitial(ctx, "world-cleared-viewport", |ui| {
        ui.heading(text::WORLD_CLEARED_TITLE);
        ui.label(text::world_cleared_body(run_progress.world_index));
        if ui.button(text::CONTINUE_BUTTON).clicked() {
            advance_to_next_world(
                &mut world,
                &mut run_progress,
                &config,
                &mut season_progress,
                &mut era_next_state,
                &mut reset,
            );
            next_state.set(GameState::Playing);
        }
    });
    Ok(())
}

/// "World failed" interstitial (task 051): total extinction only — the run
/// isn't over, so `run_progress` isn't touched here, unlike
/// `world_cleared_screen_ui`. `retry_world` rebuilds the exact same
/// `world_index`/seed the player just lost.
#[allow(clippy::too_many_arguments)]
fn world_failed_screen_ui(
    mut contexts: EguiContexts,
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    run_progress: Res<RunProgress>,
    mut season_progress: ResMut<SeasonProgress>,
    mut era_next_state: ResMut<NextState<EraState>>,
    mut reset: WorldResetParams,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    interstitial(ctx, "world-failed-viewport", |ui| {
        ui.heading(text::WORLD_FAILED_TITLE);
        ui.label(text::WORLD_FAILED_BODY);
        if ui.button(text::RETRY_BUTTON).clicked() {
            retry_world(
                &mut world,
                &run_progress,
                &config,
                &mut season_progress,
                &mut era_next_state,
                &mut reset,
            );
            next_state.set(GameState::Playing);
        }
    });
    Ok(())
}

/// "Run ended" interstitial: no world rebuild here — `menu.rs::start_run`
/// builds the next run's first world when the player presses "New run".
fn defeat_screen_ui(
    mut contexts: EguiContexts,
    run_progress: Res<RunProgress>,
    meta: Res<MetaProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    interstitial(ctx, "defeat-viewport", |ui| {
        ui.heading(text::DEFEAT_TITLE);
        ui.label(text::defeat_body(run_progress.worlds_cleared));
        // `MetaProgress::absorb` (task 046) already ran for this run's
        // result by the time this screen shows — the totals here already
        // include whatever this run just earned.
        ui.label(text::unlocks_summary(meta.bonus_available_species));
        if ui.button(text::RETURN_TO_MENU_BUTTON).clicked() {
            next_state.set(GameState::MainMenu);
        }
    });
    Ok(())
}

/// Shared centered-panel layout for both interstitials — same viewport `Ui`
/// construction `menu.rs::main_menu_ui` uses, since both draw straight onto
/// the egui viewport rather than a HUD-anchored panel.
fn interstitial(ctx: &egui::Context, id: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new(id),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    egui::CentralPanel::default().show(&mut viewport_ui, |ui| {
        ui.add_space(40.0);
        ui.vertical_centered(add_contents);
    });
}
