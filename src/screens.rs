// World-cleared/defeat interstitials (task 045): where a run's per-world
// loop closes. `GameState::WorldCleared` (objective satisfied) leads to the
// next, harder world via `start_world`; `GameState::Defeat` (a failure
// condition tripped, task 041) ends the run and returns to the main menu —
// not back to `Playing`, since a run that ended requires going through the
// menu again (`menu.rs`) to start a new one.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use abiogenesis::config::SimConfig;
use abiogenesis::objectives::{CurrentObjective, CurrentWorldOutcome, ObjectiveProgress};
use abiogenesis::run::{MetaProgress, RunProgress};
use abiogenesis::sim::{ActionBudget, EraProgress};
use abiogenesis::state::{EraState, GameState};
use abiogenesis::world::SimWorld;

use crate::notebook::{MatrixKnowledge, ObservationLog, PlayerPlacedCells};
use crate::run_flow::start_world;
use crate::text;
use crate::ui::{SelectedSpecies, SpliceDraft};

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (
                world_cleared_screen_ui.run_if(in_state(GameState::WorldCleared)),
                defeat_screen_ui.run_if(in_state(GameState::Defeat)),
            ),
        );
    }
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
    mut era_progress: ResMut<EraProgress>,
    mut era_next_state: ResMut<NextState<EraState>>,
    mut knowledge: ResMut<MatrixKnowledge>,
    mut log: ResMut<ObservationLog>,
    mut budget: ResMut<ActionBudget>,
    mut selected: ResMut<SelectedSpecies>,
    mut splice_draft: ResMut<SpliceDraft>,
    mut placed: ResMut<PlayerPlacedCells>,
    mut objective: ResMut<CurrentObjective>,
    mut objective_progress: ResMut<ObjectiveProgress>,
    mut outcome: ResMut<CurrentWorldOutcome>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    interstitial(ctx, "world-cleared-viewport", |ui| {
        ui.heading(text::WORLD_CLEARED_TITLE);
        ui.label(text::world_cleared_body(run_progress.world_index));
        if ui.button(text::CONTINUE_BUTTON).clicked() {
            let next_world_index = run_progress.world_index + 1;
            let next_seed = world.next_seed();
            start_world(
                &mut world,
                next_world_index,
                next_seed,
                &config,
                run_progress.unlocks.bonus_available_species,
                &mut era_progress,
                &mut era_next_state,
                &mut knowledge,
                &mut log,
                &mut budget,
                &mut selected,
                &mut splice_draft,
                &mut placed,
                &mut objective,
                &mut objective_progress,
                &mut outcome,
            );
            run_progress.world_index = next_world_index;
            run_progress.world_seed = next_seed;
            run_progress.worlds_cleared += 1;
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
