use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera, ClearColorConfig, Viewport};
use bevy::prelude::*;
use bevy_egui::{
    egui, EguiContexts, EguiGlobalSettings, EguiPrimaryContextPass, PrimaryEguiContext,
};

use crate::render::GridCamera;
use abiogenesis::config::SimConfig;
use abiogenesis::sim::ActionBudget;
use abiogenesis::state::EraState;
use abiogenesis::world::{SimWorld, SpeciesId, TagId};

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
        old: Option<TagId>,
        new: Option<TagId>,
    },
    AddTag {
        tag: Option<TagId>,
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
            ui.heading("Abiogenesis");
            ui.label(format!("Era {}  ·  tick {}", world.era, world.tick));
            ui.label(format!("Seed {}", world.seed));
            ui.label(format!("State: {:?}", era_state.get()));

            ui.separator();
            ui.label("Population");
            if stats.is_empty() {
                ui.weak("  (none)");
            }
            for (species, population, avg_energy) in &stats {
                ui.label(format!(
                    "  species {}: {} · avg energy {:.2}",
                    species.0, population, avg_energy
                ));
            }

            ui.separator();
            ui.label("Action");
            ui.radio_value(&mut selected_action.0, ActionMode::Seed, "Seed");
            ui.radio_value(&mut selected_action.0, ActionMode::Stress, "Stress");
            ui.radio_value(&mut selected_action.0, ActionMode::Cull, "Cull");
            ui.radio_value(
                &mut selected_action.0,
                ActionMode::Splice,
                "Splice (cost 2)",
            );

            ui.separator();
            ui.label("Seed species (click an empty cell)");
            for i in 0..world.species.len() as u8 {
                ui.radio_value(&mut selected.0, SpeciesId(i), format!("species {i}"));
            }

            if selected_action.0 == ActionMode::Splice {
                ui.separator();
                splice_panel(ui, &world, &mut splice_draft);
            }

            ui.separator();
            ui.label(format!(
                "Actions: {} / {}",
                budget.points_remaining, config.time.point_budget_per_era
            ));
            // Placeholder: objective arrives in Phase 3 (GDD §8).

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.weak("space era · s tick · r reseed · Esc quit");
            });
        });

    Ok(())
}

/// The `Splice` editor (task 025, GDD §6 — "the most powerful and most
/// expensive experimental tool"): pick a source species and one edit,
/// staged in `SpliceDraft` until "Apply" is pressed. Read-only against
/// `SimWorld`, same as `hud_panel` — the actual mutation happens in
/// `input.rs`'s `apply_splice`, reading `apply_requested` as an intent.
fn splice_panel(ui: &mut egui::Ui, world: &SimWorld, draft: &mut SpliceDraft) {
    ui.label("Splice: source species");
    for i in 0..world.species.len() as u8 {
        ui.radio_value(
            &mut draft.source,
            Some(SpeciesId(i)),
            format!("species {i}"),
        );
    }

    // A species already at GDD §5.3's 3-tag cap has no room to grow, so
    // "Add a tag" is only offered when the selected source has fewer than 3.
    let has_room = draft
        .source
        .and_then(|source| world.species.get(source.0 as usize))
        .is_some_and(|species| species.tags.len() < 3);

    ui.label("Edit");
    let is_swap = matches!(draft.edit, SpliceEditChoice::SwapTag { .. });
    let is_add = matches!(draft.edit, SpliceEditChoice::AddTag { .. });
    if ui.radio(is_swap, "Swap a tag").clicked() {
        draft.edit = SpliceEditChoice::SwapTag {
            old: None,
            new: None,
        };
    }
    ui.add_enabled_ui(has_room, |ui| {
        if ui.radio(is_add, "Add a tag").clicked() {
            draft.edit = SpliceEditChoice::AddTag { tag: None };
        }
    });
    if !has_room {
        ui.weak("  (source already has 3 tags)");
    }
    if ui
        .radio(!is_swap && !is_add, "Shift temperature optimum")
        .clicked()
    {
        draft.edit = SpliceEditChoice::ShiftTempOptimum { warmer: true };
    }

    match &mut draft.edit {
        SpliceEditChoice::SwapTag { old, new } => {
            if let Some(source) = draft.source {
                let species = &world.species[source.0 as usize];
                ui.label("Remove tag:");
                for &tag in &species.tags {
                    ui.radio_value(old, Some(tag), format!("tag {}", tag.0));
                }
                ui.label("Add tag:");
                for &tag in &world.active_tags {
                    ui.radio_value(new, Some(tag), format!("tag {}", tag.0));
                }
            } else {
                ui.weak("  (pick a source species first)");
            }
        }
        SpliceEditChoice::AddTag { tag } => {
            if let Some(source) = draft.source {
                let species = &world.species[source.0 as usize];
                ui.label("Add tag:");
                for &candidate in &world.active_tags {
                    if species.tags.contains(&candidate) {
                        continue;
                    }
                    ui.radio_value(tag, Some(candidate), format!("tag {}", candidate.0));
                }
            } else {
                ui.weak("  (pick a source species first)");
            }
        }
        SpliceEditChoice::ShiftTempOptimum { warmer } => {
            ui.radio_value(warmer, true, "warmer");
            ui.radio_value(warmer, false, "colder");
        }
    }

    if ui.button("Apply splice").clicked() {
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
