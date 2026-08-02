use bevy::camera::{Camera, Viewport};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use abiogenesis::state::EraState;
use abiogenesis::world::{SimWorld, SpeciesId};

/// On-screen width of the HUD panel, reserved from the camera's viewport so
/// the panel never draws over the grid (task 008 acceptance criterion).
/// Presentation-only, not a simulation coefficient (see `render::CELL_SIZE`
/// for the same rationale).
const HUD_WIDTH: f32 = 260.0;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, hud_panel)
            .add_systems(Update, reserve_hud_viewport);
    }
}

/// Side panel with the numeric readout of GDD §11. Reads `SimWorld`
/// read-only: the UI never writes simulation state (TECH_DESIGN.md §3.3).
fn hud_panel(
    mut contexts: EguiContexts,
    world: Res<SimWorld>,
    era_state: Res<State<EraState>>,
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
            // Placeholder: objective and action budget arrive in Phase 3 (GDD §8, §6).

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.weak("space era · s tick · r reseed · Esc quit");
            });
        });

    Ok(())
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

/// Shrinks the camera's viewport by `HUD_WIDTH` so the grid renders next to
/// the panel instead of underneath it. Runs every frame to track window
/// resizes; cheap at this scale and avoids a second source of truth for the
/// window size.
fn reserve_hud_viewport(windows: Query<&Window>, mut cameras: Query<&mut Camera>) {
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
