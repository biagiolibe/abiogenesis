use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera, ClearColorConfig, Viewport};
use bevy::prelude::*;
use bevy_egui::{
    egui, EguiContexts, EguiGlobalSettings, EguiPrimaryContextPass, PrimaryEguiContext,
};

use crate::notebook::tag_glyph;
use crate::render::{species_color, species_label, GridCamera};
use crate::text;
use abiogenesis::config::SimConfig;
use abiogenesis::sim::ActionBudget;
use abiogenesis::state::EraState;
use abiogenesis::world::{SimWorld, SpeciesId, TagSlot};

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

pub struct UiPlugin;

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
            .add_systems(Startup, spawn_hud_camera)
            .add_systems(Update, reserve_hud_viewport)
            .add_systems(EguiPrimaryContextPass, hud_panel);
    }
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
            // Placeholder: objective arrives in Phase 3 (GDD §8).

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.weak(text::KEYBOARD_HINT);
            });
        });

    Ok(())
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
