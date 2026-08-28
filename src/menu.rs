// Main menu (task 044, TECH_DESIGN.md §2: "seed selection and run start").
// `run_seed` is born here — the one legitimate point outside the simulation
// where a run's variety originates (invariant 1, TECH_DESIGN.md §5, only
// binds `sim`/`world`/`config`; this module lives alongside `ui.rs`/
// `input.rs` in the binary crate and is presentation, not simulation). From
// the moment "New run" is pressed, everything derives deterministically from
// `run_seed` (`RunProgress::start`) — no second point in the code reads
// system randomness again.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::notebook::{
    NotebookHasUnseenConfirmation, ObservationLog, PlayerPlacedCells, TerrainKnowledge,
};
use crate::render::SeenRelations;
use crate::text;
use crate::ui::{IsolationHint, SelectedSpecies, SpliceDraft, StallHint};
use abiogenesis::config::SimConfig;
use abiogenesis::knowledge::MatrixKnowledge;
use abiogenesis::objectives::{CurrentObjective, CurrentWorldOutcome, ObjectiveProgress};
use abiogenesis::run::{MetaProgress, RunProgress};
use abiogenesis::sim::ActionBudget;
use abiogenesis::state::GameState;
use abiogenesis::world::SpeciesId;
use abiogenesis::worldgen::build_world;

/// The seed text field's contents, kept across frames while the menu is
/// open. UI-local state, not simulation state — nothing here is read by the
/// sim until "New run" parses it into a `run_seed`.
#[derive(Resource, Default)]
struct SeedInput(String);

/// Max height of the "How to play" panel's scroll area (task 056) —
/// presentation-only, not a simulation coefficient, same rationale
/// `ui.rs::HUD_WIDTH` gives for its own layout constant.
const HOW_TO_PLAY_PANEL_HEIGHT: f32 = 260.0;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SeedInput>().add_systems(
            EguiPrimaryContextPass,
            main_menu_ui.run_if(in_state(GameState::MainMenu)),
        );
    }
}

/// Title screen: a seed field (typed seed reproduces a specific world, blank
/// generates one) and a "New run" button. Reads/writes only `SeedInput`
/// directly; the heavier "build the first world" work is factored into
/// `start_run` so it stays testable independent of egui.
fn main_menu_ui(
    mut contexts: EguiContexts,
    mut seed_input: ResMut<SeedInput>,
    mut commands: Commands,
    config: Res<SimConfig>,
    meta: Res<MetaProgress>,
    mut next_state: ResMut<NextState<GameState>>,
    mut show_guide: Local<bool>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "menu-viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    egui::CentralPanel::default().show(&mut viewport_ui, |ui| {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.heading(text::MENU_TITLE);
            ui.add_space(16.0);
            ui.label(text::MENU_SEED_LABEL);
            ui.add(
                egui::TextEdit::singleline(&mut seed_input.0)
                    .hint_text(text::MENU_SEED_HINT)
                    .desired_width(160.0),
            );
            ui.add_space(12.0);
            if ui.button(text::MENU_NEW_RUN_BUTTON).clicked() {
                let run_seed = parse_or_generate_seed(&seed_input.0);
                start_run(&mut commands, &config, &meta, run_seed);
                if meta.seen_intro {
                    next_state.set(GameState::Playing);
                } else {
                    next_state.set(GameState::Intro);
                }
            }
            ui.add_space(16.0);
            ui.label(text::unlocks_summary(meta.bonus_available_species));
            ui.add_space(16.0);

            let toggle_label = if *show_guide {
                text::HOW_TO_PLAY_HIDE_BUTTON
            } else {
                text::HOW_TO_PLAY_SHOW_BUTTON
            };
            if ui.button(toggle_label).clicked() {
                *show_guide = !*show_guide;
            }
        });

        if *show_guide {
            ui.add_space(16.0);
            egui::ScrollArea::vertical()
                .id_salt("how_to_play")
                .max_height(HOW_TO_PLAY_PANEL_HEIGHT)
                .show(ui, |ui| {
                    for (heading, body) in text::HOW_TO_PLAY_SECTIONS {
                        ui.strong(*heading);
                        ui.label(*body);
                        ui.add_space(8.0);
                    }
                });
        }
    });
    Ok(())
}

/// A typed seed reproduces that exact run; a blank (or unparsable) field
/// generates one via `rand`'s thread-local generator — the menu's sanctioned
/// exception to "no system randomness" (see module docs).
fn parse_or_generate_seed(input: &str) -> u64 {
    input.trim().parse().unwrap_or_else(|_| rand::random())
}

/// Builds the run's first world and installs every resource
/// `GameState::Playing`'s systems expect to already exist — the equivalent
/// of the old `Startup`-time `spawn_world`, now triggered by the player
/// instead of the process booting. Unlike `run_flow::start_world` (task
/// 045, used for every *later* world), this doesn't touch `SeasonProgress`/
/// `EraState`: those are substates of `Playing`, which hasn't been entered
/// yet, so they're already at their fresh `Default` the moment it is.
/// `MatrixKnowledge` still needs an explicit fresh instance sized to this
/// world's actual `active_tags.len()` — the `Startup`-time one `notebook.rs`
/// inserts is sized from a fixed config constant that worldgen (task 038)
/// no longer guarantees matches world 0.
fn start_run(commands: &mut Commands, config: &SimConfig, meta: &MetaProgress, run_seed: u64) {
    let run_progress = RunProgress::start(run_seed, meta);
    let (world, objectives) = build_world(
        run_progress.world_seed,
        run_progress.world_index,
        config,
        run_progress.unlocks.bonus_available_species,
    );
    commands.insert_resource(MatrixKnowledge::new(
        world.active_tags.len(),
        config.notebook.confirmation_threshold,
    ));
    // Same staleness risk `MatrixKnowledge` above guards against — task
    // 097's `TerrainKnowledge` is sized off the same `active_tags_early`
    // constant at `Startup`, which world 0's actual tag count isn't
    // guaranteed to match.
    commands.insert_resource(TerrainKnowledge::new(
        world.active_tags.len(),
        config.notebook.confirmation_threshold,
    ));
    // Same staleness risk `MatrixKnowledge` above is already guarding
    // against: `GridRenderPlugin::build`'s `SeenRelations` is sized from
    // `TagConfig::active_tags_early`, a fixed config constant worldgen
    // (task 038) doesn't guarantee matches world 0's actual tag count.
    commands.insert_resource(SeenRelations::new(world.active_tags.len()));
    commands.insert_resource(world);
    commands.insert_resource(run_progress);
    commands.insert_resource(ObservationLog::default());
    commands.insert_resource(PlayerPlacedCells::default());
    commands.insert_resource(NotebookHasUnseenConfirmation::default());
    commands.insert_resource(IsolationHint::default());
    commands.insert_resource(StallHint::default());
    commands.insert_resource(ActionBudget {
        points_remaining: config.time.point_budget_per_season,
    });
    commands.insert_resource(SelectedSpecies(SpeciesId(0)));
    commands.insert_resource(SpliceDraft::default());
    commands.insert_resource(CurrentObjective::new(objectives));
    commands.insert_resource(ObjectiveProgress::default());
    commands.insert_resource(CurrentWorldOutcome::default());
}
