use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera, ClearColorConfig, Viewport};
use bevy::prelude::*;
use bevy_egui::{
    egui, EguiContexts, EguiGlobalSettings, EguiPrimaryContextPass, PrimaryEguiContext,
};

use crate::notebook::{tag_glyph, EverSeeded, NotebookEverOpened, NotebookHasUnseenConfirmation};
use crate::render::{species_color, species_label, GridCamera};
use crate::text;
use abiogenesis::config::SimConfig;
use abiogenesis::objectives::{CurrentObjective, Objective, ObjectiveProgress};
use abiogenesis::sim::ActionBudget;
use abiogenesis::state::{EraState, GameState};
use abiogenesis::world::{SimWorld, SpeciesId, TagSlot};

/// Task 055's guided first-isolation hint: the message to show (isolated vs.
/// clustered first placement) and the `SimWorld::tick` it was set at, so
/// `viewport_hint` can self-dismiss it after `ISOLATION_HINT_DURATION_TICKS`
/// — the tick counter the HUD's own "Era X · tick Y" readout already
/// exposes, not a wall-clock timer. Written once, by `input.rs`'s
/// `seed_organism_on_click` at the player's first-ever placement of their
/// first-ever run (gated on `MetaProgress::seen_isolation_hint`); read here.
#[derive(Resource, Default)]
pub struct IsolationHint {
    pub text: Option<&'static str>,
    pub shown_at_tick: u64,
}

/// The species the seed action (task 017) places on click. A UI intent, not
/// simulation state: owned here, read by `input.rs`'s click-to-place system,
/// same rationale as `EraProgress` living in `sim.rs` but written by
/// `input.rs` (TECH_DESIGN.md §3.3 — `Ui` writes only intents).
#[derive(Resource)]
pub struct SelectedSpecies(pub SpeciesId);

/// What a left-click does (GDD §6). `Splice` (task 025) doesn't act on a
/// grid click at all — it drives a small editor panel instead — but stays
/// in this enum so the mode selector treats all four actions uniformly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionMode {
    Seed,
    Stress,
    Cull,
    Splice,
}

/// The currently-selected click action. A UI intent like `SelectedSpecies`,
/// defaulting to `Seed` so existing click behavior is unchanged for anyone
/// who never touches the new selector.
#[derive(Resource)]
pub struct SelectedAction(pub ActionMode);

/// One in-progress `Splice` edit (GDD §6): swap one tag for another, add a
/// tag to a species with room under the 1-3 tag cap (GDD §5.3, task 027), or
/// shift the thermal optimum by `config.energy.splice_temp_shift`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpliceEditChoice {
    SwapTag {
        old: Option<TagSlot>,
        new: Option<TagSlot>,
    },
    AddTag {
        tag: Option<TagSlot>,
    },
    ShiftTempOptimum {
        warmer: bool,
    },
}

impl Default for SpliceEditChoice {
    fn default() -> Self {
        Self::SwapTag {
            old: None,
            new: None,
        }
    }
}

/// The player's in-progress `Splice` selections (task 025) — a UI intent
/// like `SelectedSpecies`/`SelectedAction`, not simulation state. `Splice`
/// targets a *species definition* rather than a grid cell, so unlike
/// Seed/Stress/Cull it has no single click to resolve; `apply_requested`
/// is the intent `input.rs`'s `apply_splice` system reads and clears once
/// consumed, keeping the actual `SimWorld` mutation (TECH_DESIGN.md §3.3 —
/// `Ui` writes only intents) out of this egui panel.
#[derive(Resource, Default)]
pub struct SpliceDraft {
    pub source: Option<SpeciesId>,
    pub edit: SpliceEditChoice,
    pub apply_requested: bool,
}

/// On-screen width of the HUD panel, reserved from the grid camera's
/// viewport so the panel never draws over the grid (task 008 acceptance
/// criterion). Presentation-only, not a simulation coefficient (see
/// `render::CELL_SIZE` for the same rationale).
const HUD_WIDTH: f32 = 260.0;

/// `RenderLayers` for the dedicated egui camera: no grid entity is ever
/// assigned to it, so this camera draws nothing of the scene, only the
/// egui overlay (TECH_DESIGN.md §6 "HUD camera").
const HUD_CAMERA_LAYER: usize = 1;

/// Color-swatch glyph preceding a species' name in Population/Seed Palette
/// (playtest finding, task 041 session), matching `species_color`'s hue to
/// the same organism's on-grid dot — same technique `notebook.rs::TAG_GLYPH`
/// uses for tags.
const SPECIES_GLYPH: &str = "●";

/// Color of the notebook's unseen-confirmation badge (task 054) — matches
/// no existing semantic (it's neither a species nor a tag color), so it
/// gets its own constant rather than reusing one of those.
const NOTEBOOK_BADGE_COLOR: egui::Color32 = egui::Color32::from_rgb(230, 190, 60);

/// How many ticks the guided first-isolation hint (task 055) stays on
/// screen before self-dismissing — long enough to read over a few ticks of
/// the freshly-placed organism's energy, short enough not to linger once
/// the moment it refers to has passed.
const ISOLATION_HINT_DURATION_TICKS: u64 = 30;

pub struct UiPlugin;

/// DejaVu Sans (Bitstream Vera License, see `assets/fonts/DejaVu-LICENSE.txt`)
/// covers Greek script and the `●` bullet that egui's built-in default font
/// lacks — those glyphs were rendering as tofu boxes (playtest finding,
/// task 041 session): `notebook.rs::TAG_LETTERS`/`TAG_GLYPH` and this
/// module's `SPECIES_GLYPH`. It does not add color-emoji support: egui has
/// no COLR/bitmap glyph rendering path at all, so `ACTION_GLYPHS`' 🌱💀🔬
/// stay unresolved regardless of font (⚡ happens to have a monochrome
/// dingbat glyph and already renders).
const DEJAVU_SANS: &[u8] = include_bytes!("../assets/fonts/DejaVuSans.ttf");

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // bevy_egui otherwise auto-attaches the primary egui context to the
        // first camera spawned — the grid camera. Since egui derives its own
        // paint canvas (`screen_rect`) from that camera's `Viewport`
        // (TECH_DESIGN.md §6 "HUD camera"), sharing it with the grid camera
        // means cropping the grid camera's viewport to make room for the
        // HUD also crops away the HUD's own paint canvas. A dedicated,
        // full-viewport camera for egui avoids the conflict.
        app.world_mut()
            .resource_mut::<EguiGlobalSettings>()
            .auto_create_primary_context = false;
        app.insert_resource(SelectedSpecies(SpeciesId(0)))
            .insert_resource(SelectedAction(ActionMode::Seed))
            .init_resource::<SpliceDraft>()
            .init_resource::<IsolationHint>()
            .add_systems(Startup, spawn_hud_camera)
            .add_systems(
                Update,
                reserve_hud_viewport.run_if(in_state(GameState::Playing)),
            )
            .add_systems(EguiPrimaryContextPass, configure_fonts)
            .add_systems(
                EguiPrimaryContextPass,
                (hud_panel, viewport_hint).run_if(in_state(GameState::Playing)),
            );
    }
}

/// Registers `DEJAVU_SANS` as a lowest-priority fallback for the
/// proportional font family: egui's own default font is tried first for
/// every glyph, DejaVu Sans only fills the gaps (Greek letters, `●`). The
/// egui context doesn't exist until the first `EguiPrimaryContextPass` run
/// (it's attached to the camera spawned in `spawn_hud_camera`), so this
/// can't run at `Startup`; `done` makes it a one-shot within that schedule.
fn configure_fonts(mut contexts: EguiContexts, mut done: Local<bool>) -> Result {
    if *done {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    ctx.add_font(egui::epaint::text::FontInsert::new(
        "DejaVuSans",
        egui::FontData::from_static(DEJAVU_SANS),
        vec![egui::epaint::text::InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));
    *done = true;
    Ok(())
}

/// Full-viewport camera dedicated to egui (TECH_DESIGN.md §6 "HUD camera").
/// Renders after the grid camera (`order: 1`) without clearing its output
/// (`ClearColorConfig::None`), and on a `RenderLayers` no grid entity uses,
/// so it composites only the egui overlay on top of the grid.
fn spawn_hud_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PrimaryEguiContext,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(HUD_CAMERA_LAYER),
    ));
}

/// Shrinks the grid camera's viewport by `HUD_WIDTH` so the grid renders
/// next to the panel instead of underneath it. Runs every frame to track
/// window resizes; cheap at this scale and avoids a second source of truth
/// for the window size. Targets only `GridCamera` — the HUD camera's
/// viewport must stay full-size, since it doubles as egui's paint canvas.
fn reserve_hud_viewport(
    windows: Query<&Window>,
    mut cameras: Query<&mut Camera, With<GridCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    let hud_px = (HUD_WIDTH * window.scale_factor()) as u32;
    let full_width = window.physical_width();
    let full_height = window.physical_height();
    if full_width <= hud_px || full_height == 0 {
        return;
    }

    camera.viewport = Some(Viewport {
        physical_position: UVec2::ZERO,
        physical_size: UVec2::new(full_width - hud_px, full_height),
        ..default()
    });
}

/// Side panel with the numeric readout of GDD §11. Reads `SimWorld`
/// read-only: the UI never writes simulation state (TECH_DESIGN.md §3.3).
#[allow(clippy::too_many_arguments)]
fn hud_panel(
    mut contexts: EguiContexts,
    world: Res<SimWorld>,
    era_state: Res<State<EraState>>,
    mut selected: ResMut<SelectedSpecies>,
    mut selected_action: ResMut<SelectedAction>,
    mut splice_draft: ResMut<SpliceDraft>,
    budget: Res<ActionBudget>,
    config: Res<SimConfig>,
    objective: Res<CurrentObjective>,
    objective_progress: Res<ObjectiveProgress>,
    unseen_confirmation: Res<NotebookHasUnseenConfirmation>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    let stats = species_stats(&world);

    egui::Panel::right("hud")
        .exact_size(HUD_WIDTH)
        .resizable(false)
        .show(&mut viewport_ui, |ui| {
            ui.heading(text::HEADING_TITLE);

            group_frame(ui, |ui| {
                ui.label(text::era_tick_line(world.era, world.tick));
                ui.label(text::seed_line(world.seed));
                ui.label(text::state_line(era_state.get()));
            });

            ui.add_space(6.0);
            group_frame(ui, |ui| {
                ui.strong(text::HEADING_ACTION);
                action_icon_row(ui, &mut selected_action, &config);

                let total = config.time.point_budget_per_era;
                let fraction = if total == 0 {
                    0.0
                } else {
                    budget.points_remaining as f32 / total as f32
                };
                ui.add(
                    egui::ProgressBar::new(fraction)
                        .text(text::budget_bar_text(budget.points_remaining, total)),
                )
                .on_hover_text(text::BUDGET_HOVER);

                if selected_action.0 == ActionMode::Splice {
                    ui.separator();
                    splice_panel(ui, &world, &mut splice_draft);
                }
            });

            ui.add_space(6.0);
            group_frame(ui, |ui| {
                ui.strong(text::HEADING_POPULATION);
                if stats.is_empty() {
                    ui.weak(text::NO_POPULATION);
                }
                for (species, population, avg_energy) in &stats {
                    ui.horizontal(|ui| {
                        ui.colored_label(species_color(*species), SPECIES_GLYPH);
                        ui.label(text::population_line(
                            &species_label(*species),
                            *population,
                            *avg_energy,
                            config.energy.repro_threshold,
                        ));
                    });
                }
            });

            ui.add_space(6.0);
            group_frame(ui, |ui| {
                ui.strong(text::HEADING_SEED_PALETTE)
                    .on_hover_text(text::SEED_PALETTE_HOVER);
                for i in 0..world.species.len() as u8 {
                    let species = SpeciesId(i);
                    ui.horizontal(|ui| {
                        ui.colored_label(species_color(species), SPECIES_GLYPH);
                        ui.radio_value(&mut selected.0, species, species_label(species));
                    });
                }
            });
            ui.add_space(6.0);
            group_frame(ui, |ui| {
                ui.strong(text::HEADING_OBJECTIVE);
                objective_panel(
                    ui,
                    objective.0.as_ref(),
                    &objective_progress,
                    config.time.era_ticks,
                );
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.weak(text::KEYBOARD_HINT);
                ui.horizontal(|ui| {
                    ui.weak(text::NOTEBOOK_AFFORDANCE_LABEL);
                    if unseen_confirmation.0 {
                        ui.colored_label(NOTEBOOK_BADGE_COLOR, text::NOTEBOOK_BADGE_GLYPH)
                            .on_hover_text(text::NOTEBOOK_BADGE_HOVER);
                    }
                });
            });
        });

    Ok(())
}

/// Vertical gap between the grid viewport's top edge and the onboarding
/// hint area (task 053), keeping it clear of the grid's own top row.
const VIEWPORT_HINT_TOP_MARGIN: f32 = 24.0;

/// Which onboarding hint to show, if any, given the two milestones task 053
/// guides the player through. `ever_seeded` rather than `PlayerPlacedCells`
/// emptiness: a placed organism's death removes it from `PlayerPlacedCells`
/// (per-organism, consumed on death), which would otherwise make the first
/// hint reappear after the player's first placement dies.
fn hint_text(ever_seeded: bool, notebook_ever_opened: bool) -> Option<&'static str> {
    if !ever_seeded {
        Some(text::HINT_PLACE_FIRST_ORGANISM)
    } else if !notebook_ever_opened {
        Some(text::HINT_OPEN_NOTEBOOK)
    } else {
        None
    }
}

/// Whether task 055's guided first-isolation hint, shown at `shown_at_tick`,
/// is still within its `ISOLATION_HINT_DURATION_TICKS` display window — pure
/// so the self-dismiss timing is unit-testable without a `World`.
fn isolation_hint_active(shown_at_tick: u64, current_tick: u64) -> bool {
    current_tick.saturating_sub(shown_at_tick) < ISOLATION_HINT_DURATION_TICKS
}

/// Non-interactive onboarding hint over the grid viewport (task 053),
/// guiding a fresh player through their first two actions: placing an
/// organism, then opening the notebook. Purely state-driven — no dismiss
/// affordance — and anchored above the grid area (excluding `HUD_WIDTH`) so
/// it never overlaps the cells the player needs to click.
///
/// Task 055's guided first-isolation hint takes priority over the two
/// task-053 hints while it's active: it's a short-lived, more specific
/// message set by `input.rs::seed_organism_on_click` at the player's
/// first-ever placement, self-dismissed here by clearing `IsolationHint`
/// once `ISOLATION_HINT_DURATION_TICKS` have passed, after which the
/// task-053 flow (e.g. "open the notebook") resumes underneath it.
fn viewport_hint(
    mut contexts: EguiContexts,
    ever_seeded: Res<EverSeeded>,
    notebook_ever_opened: Res<NotebookEverOpened>,
    mut isolation_hint: ResMut<IsolationHint>,
    world: Res<SimWorld>,
) -> Result {
    if isolation_hint.text.is_some()
        && !isolation_hint_active(isolation_hint.shown_at_tick, world.tick)
    {
        isolation_hint.text = None;
    }

    let hint = if let Some(text) = isolation_hint.text {
        text
    } else if let Some(text) = hint_text(ever_seeded.0, notebook_ever_opened.0) {
        text
    } else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    let grid_rect = {
        let mut rect = ctx.viewport_rect();
        rect.max.x -= HUD_WIDTH;
        rect
    };

    egui::Area::new("viewport_hint".into())
        .order(egui::Order::Foreground)
        .interactable(false)
        .fixed_pos(grid_rect.center_top() + egui::vec2(0.0, VIEWPORT_HINT_TOP_MARGIN))
        .pivot(egui::Align2::CENTER_TOP)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.label(hint);
            });
        });

    Ok(())
}

/// Current-objective panel (task 043, GDD §11): the objective's sentence
/// plus a progress bar, reusing the `ActionBudget` bar's visual pattern
/// (`egui::ProgressBar` with an overlaid text). Formats the concrete
/// numbers/labels here (species name, era counts) and hands them to
/// `text.rs`'s parametrized templates, since `text.rs` never reaches into
/// `Objective`/`SimWorld`-derived data on its own (task 034's constraint).
///
/// The bar's *fill* stays tick-precise (`fraction`, straight from
/// `consecutive_ticks`/`ticks`) for smooth per-tick animation; only the
/// *label* overlaid on it is reformatted in eras (task 049, GDD §11: ticks
/// aren't a unit the player should need to think in — `era_ticks` is the
/// player's own "advance era" button).
fn objective_panel(
    ui: &mut egui::Ui,
    objective: Option<&Objective>,
    progress: &ObjectiveProgress,
    era_ticks: u32,
) {
    let Some(objective) = objective else {
        ui.weak(text::NO_OBJECTIVE);
        return;
    };

    let (description, fraction, bar_text) = match *objective {
        Objective::Coexistence { min_species, ticks } => {
            let fraction = if ticks == 0 {
                1.0
            } else {
                progress.consecutive_ticks as f32 / ticks as f32
            };
            let (eras_held, eras_required) =
                eras_progress(progress.consecutive_ticks, ticks, era_ticks);
            (
                text::coexistence_objective_line(min_species),
                fraction,
                text::sustained_progress_bar_text(eras_held, eras_required),
            )
        }
        Objective::SurviveIn {
            species,
            zone,
            ticks,
        } => {
            let fraction = if ticks == 0 {
                1.0
            } else {
                progress.consecutive_ticks as f32 / ticks as f32
            };
            let (eras_held, eras_required) =
                eras_progress(progress.consecutive_ticks, ticks, era_ticks);
            (
                text::survive_in_objective_line(&species_label(species), text::zone_label(zone)),
                fraction,
                text::sustained_progress_bar_text(eras_held, eras_required),
            )
        }
        Objective::TriggerBloom {
            species,
            population_threshold,
        } => {
            let fraction = if progress.satisfied { 1.0 } else { 0.0 };
            let bar_text = if progress.satisfied {
                text::BLOOM_TRIGGERED
            } else {
                text::BLOOM_NOT_TRIGGERED
            };
            (
                text::trigger_bloom_objective_line(&species_label(species), population_threshold),
                fraction,
                bar_text.to_string(),
            )
        }
    };

    ui.label(description);
    let bar_text = if progress.satisfied {
        text::OBJECTIVE_CLEARED.to_string()
    } else {
        bar_text
    };
    ui.add(egui::ProgressBar::new(fraction.clamp(0.0, 1.0)).text(bar_text));
}

/// Converts a sustained objective's tick counts to whole eras for display
/// (task 049): `eras_held` floors (an era in progress doesn't count until
/// complete), `eras_required` ceils (a requirement of, say, 70 ticks with a
/// 25-tick era still genuinely needs 3 full eras, not 2) — chosen so the
/// displayed fraction never claims "done" before `progress.satisfied`
/// actually flips.
fn eras_progress(consecutive_ticks: u32, required_ticks: u32, era_ticks: u32) -> (u32, u32) {
    if era_ticks == 0 {
        return (0, 0);
    }
    (
        consecutive_ticks / era_ticks,
        required_ticks.div_ceil(era_ticks),
    )
}

/// Visually separates one HUD zone from the next (task 030): a bordered,
/// rounded `egui::Frame` instead of the flat `ui.separator()` list this
/// replaced, so World state / Action / Population / Seed palette read as
/// distinct groups rather than one undifferentiated stack.
fn group_frame<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::group(ui.style())
        .inner_margin(6.0)
        .show(ui, |ui| add_contents(ui))
        .inner
}

/// One glyph per `ActionMode`, in selector order. Names/descriptions/costs
/// live in `text::action_name`/`action_description`/`action_cost` — only
/// the glyph (not player-facing copy, task 034) stays here.
const ACTION_GLYPHS: [(ActionMode, &str); 4] = [
    (ActionMode::Seed, "🌱"),
    (ActionMode::Stress, "⚡"),
    (ActionMode::Cull, "💀"),
    (ActionMode::Splice, "🔬"),
];

/// The four `ActionMode` options as a row of icon buttons (task 030),
/// replacing the vertical `ui.radio_value(..., "Seed")`-style text list.
/// Still single-selection, immediate-mode — `selectable_label` plays the
/// same role `ui.radio_value` did, just rendered as a compact glyph with a
/// hover tooltip carrying the name, cost, and a one-line description
/// instead of inline text.
fn action_icon_row(ui: &mut egui::Ui, selected_action: &mut SelectedAction, config: &SimConfig) {
    ui.horizontal(|ui| {
        for (mode, glyph) in ACTION_GLYPHS {
            let cost = action_cost(mode, config);
            let response = ui.selectable_label(
                selected_action.0 == mode,
                egui::RichText::new(glyph).size(20.0),
            );
            if response.clicked() {
                selected_action.0 = mode;
            }
            response.on_hover_text(text::action_tooltip(mode, cost));
        }
    });
}

/// Action-point cost for one `ActionMode`, read from `SimConfig` so the
/// icon tooltips (task 030) never hardcode a number that could drift from
/// `ActionCosts` (GDD §5.9).
fn action_cost(mode: ActionMode, config: &SimConfig) -> u32 {
    match mode {
        ActionMode::Seed => config.time.action_costs.seed,
        ActionMode::Stress => config.time.action_costs.stress,
        ActionMode::Cull => config.time.action_costs.cull,
        ActionMode::Splice => config.time.action_costs.splice,
    }
}

/// The `Splice` editor (task 025, GDD §6 — "the most powerful and most
/// expensive experimental tool"): pick a source species and one edit,
/// staged in `SpliceDraft` until "Apply" is pressed. Read-only against
/// `SimWorld`, same as `hud_panel` — the actual mutation happens in
/// `input.rs`'s `apply_splice`, reading `apply_requested` as an intent.
fn splice_panel(ui: &mut egui::Ui, world: &SimWorld, draft: &mut SpliceDraft) {
    ui.label(text::SPLICE_SOURCE_LABEL);
    for i in 0..world.species.len() as u8 {
        ui.radio_value(
            &mut draft.source,
            Some(SpeciesId(i)),
            species_label(SpeciesId(i)),
        );
    }

    // A species already at GDD §5.3's 3-tag cap has no room to grow, so
    // "Add a tag" is only offered when the selected source has fewer than 3.
    let has_room = draft
        .source
        .and_then(|source| world.species.get(source.0 as usize))
        .is_some_and(|species| species.tags.len() < 3);

    ui.label(text::EDIT_LABEL);
    let is_swap = matches!(draft.edit, SpliceEditChoice::SwapTag { .. });
    let is_add = matches!(draft.edit, SpliceEditChoice::AddTag { .. });
    if ui.radio(is_swap, text::SWAP_TAG_OPTION).clicked() {
        draft.edit = SpliceEditChoice::SwapTag {
            old: None,
            new: None,
        };
    }
    ui.add_enabled_ui(has_room, |ui| {
        if ui.radio(is_add, text::ADD_TAG_OPTION).clicked() {
            draft.edit = SpliceEditChoice::AddTag { tag: None };
        }
    });
    if !has_room {
        ui.weak(text::TAG_CAP_HINT);
    }
    if ui
        .radio(!is_swap && !is_add, text::SHIFT_TEMP_OPTION)
        .clicked()
    {
        draft.edit = SpliceEditChoice::ShiftTempOptimum { warmer: true };
    }

    match &mut draft.edit {
        SpliceEditChoice::SwapTag { old, new } => {
            if let Some(source) = draft.source {
                let species = &world.species[source.0 as usize];
                ui.label(text::REMOVE_TAG_LABEL);
                for &slot in &species.tags {
                    let tag = world.active_tags[slot.0 as usize];
                    ui.radio_value(old, Some(slot), text::tag_option_label(tag_glyph(tag)));
                }
                ui.label(text::ADD_TAG_LABEL);
                for (i, &tag) in world.active_tags.iter().enumerate() {
                    let slot = TagSlot(i as u8);
                    ui.radio_value(new, Some(slot), text::tag_option_label(tag_glyph(tag)));
                }
            } else {
                ui.weak(text::PICK_SOURCE_HINT);
            }
        }
        SpliceEditChoice::AddTag { tag } => {
            if let Some(source) = draft.source {
                let species = &world.species[source.0 as usize];
                ui.label(text::ADD_TAG_LABEL);
                for (i, &candidate) in world.active_tags.iter().enumerate() {
                    let slot = TagSlot(i as u8);
                    if species.tags.contains(&slot) {
                        continue;
                    }
                    ui.radio_value(
                        tag,
                        Some(slot),
                        text::tag_option_label(tag_glyph(candidate)),
                    );
                }
            } else {
                ui.weak(text::PICK_SOURCE_HINT);
            }
        }
        SpliceEditChoice::ShiftTempOptimum { warmer } => {
            ui.radio_value(warmer, true, text::WARMER_OPTION);
            ui.radio_value(warmer, false, text::COLDER_OPTION);
        }
    }

    if ui.button(text::APPLY_SPLICE_BUTTON).clicked() {
        draft.apply_requested = true;
    }
}

/// Population and mean energy per species, computed from the grid. Average
/// energy is divided by population (living organisms only), not by cell
/// count, or it would always read as trending toward zero.
fn species_stats(world: &SimWorld) -> Vec<(SpeciesId, usize, f32)> {
    let mut totals = vec![(0usize, 0.0f32); world.species.len()];
    for cell in &world.cells {
        if let Some(organism) = cell.organism {
            let entry = &mut totals[organism.species.0 as usize];
            entry.0 += 1;
            entry.1 += organism.energy;
        }
    }
    totals
        .into_iter()
        .enumerate()
        .filter(|&(_, (population, _))| population > 0)
        .map(|(id, (population, total_energy))| {
            (
                SpeciesId(id as u8),
                population,
                total_energy / population as f32,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eras_progress_floors_held_and_ceils_required() {
        // 25-tick eras: 60 ticks held is 2 full eras (the 3rd is
        // incomplete), 75 ticks required is exactly 3, 70 would still need
        // a 3rd era despite not being an exact multiple.
        assert_eq!(eras_progress(60, 75, 25), (2, 3));
        assert_eq!(eras_progress(0, 70, 25), (0, 3));
    }

    #[test]
    fn eras_progress_handles_a_zero_era_length_without_dividing_by_zero() {
        assert_eq!(eras_progress(10, 50, 0), (0, 0));
    }

    #[test]
    fn hint_text_prioritizes_the_seed_hint_before_anything_is_placed() {
        assert_eq!(
            hint_text(false, false),
            Some(text::HINT_PLACE_FIRST_ORGANISM)
        );
        assert_eq!(
            hint_text(false, true),
            Some(text::HINT_PLACE_FIRST_ORGANISM)
        );
    }

    #[test]
    fn hint_text_switches_to_the_notebook_hint_once_seeded() {
        assert_eq!(hint_text(true, false), Some(text::HINT_OPEN_NOTEBOOK));
    }

    #[test]
    fn hint_text_is_none_once_both_milestones_are_reached() {
        assert_eq!(hint_text(true, true), None);
    }

    #[test]
    fn isolation_hint_active_within_and_at_the_edge_of_its_window() {
        assert!(isolation_hint_active(100, 100));
        assert!(isolation_hint_active(
            100,
            100 + ISOLATION_HINT_DURATION_TICKS - 1
        ));
        assert!(!isolation_hint_active(
            100,
            100 + ISOLATION_HINT_DURATION_TICKS
        ));
    }

    #[test]
    fn isolation_hint_active_never_underflows_if_current_tick_precedes_shown_at() {
        // Shouldn't happen in practice (`world.tick` only increases), but
        // `saturating_sub` must not panic if it ever does — elapsed
        // saturates to 0, so the hint reads as freshly active rather than
        // crashing.
        assert!(isolation_hint_active(100, 0));
    }
}
