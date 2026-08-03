// The observation log (GDD §7, §11): a curated feed of salient events, built
// by consuming `sim`'s `Message`s (task 018), not by inspecting the grid
// (TECH_DESIGN.md §4). Read-only with respect to `SimWorld` (TECH_DESIGN.md
// §3.3) — this module never mutates simulation state.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use abiogenesis::sim::SpeciesExtinct;
use abiogenesis::world::SimWorld;

/// One curated log line, tagged with the era it happened in. `text`
/// describes the event only; the window prepends the era.
pub struct LogEntry {
    pub era: u32,
    pub text: String,
}

/// Ordered (oldest first), ever-growing record of salient events.
#[derive(Resource, Default)]
pub struct ObservationLog {
    pub entries: Vec<LogEntry>,
}

/// Whether the notebook window is currently shown. A plain UI toggle, not
/// `EraState` — opening the notebook must not block or interact with era
/// advancement.
#[derive(Resource, Default)]
pub struct NotebookWindowOpen(pub bool);

pub struct NotebookPlugin;

impl Plugin for NotebookPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObservationLog>()
            .init_resource::<NotebookWindowOpen>()
            .add_systems(Update, (toggle_notebook, record_events))
            .add_systems(EguiPrimaryContextPass, notebook_window);
    }
}

/// `tab`: opens/closes the notebook window, mirroring `input.rs`'s
/// `keys.just_pressed(...)` key-handling pattern.
fn toggle_notebook(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<NotebookWindowOpen>) {
    if keys.just_pressed(KeyCode::Tab) {
        open.0 = !open.0;
    }
}

/// Appends salient events to the log. Species extinctions always log;
/// individual `OrganismDied` events don't (GDD §7: the log is curated, not
/// an unfiltered per-tick feed).
///
/// TODO: bloom detection (population of a species crossing some multiple of
/// its starting count within an era) as a second salient-event type.
fn record_events(
    world: Res<SimWorld>,
    mut extinctions: MessageReader<SpeciesExtinct>,
    mut log: ResMut<ObservationLog>,
) {
    for event in extinctions.read() {
        log.entries.push(LogEntry {
            era: world.era,
            text: format!("species {} went extinct", event.species.0),
        });
    }
}

/// Draws the notebook as its own `egui::Window`, sharing the HUD's egui
/// context (`ui.rs`'s dedicated full-viewport camera) rather than a second
/// camera — `bevy_egui` supports multiple windows/panels per frame from the
/// same `EguiPrimaryContextPass` context.
fn notebook_window(
    mut contexts: EguiContexts,
    mut open: ResMut<NotebookWindowOpen>,
    log: Res<ObservationLog>,
) -> Result {
    if !open.0 {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Notebook")
        .open(&mut open.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if log.entries.is_empty() {
                    ui.weak("(no observations yet)");
                }
                for entry in &log.entries {
                    ui.label(format!("Era {}: {}", entry.era, entry.text));
                }
            });
        });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use abiogenesis::config::SimConfig;
    use abiogenesis::world::SpeciesId;

    #[test]
    fn extinction_message_appends_a_log_entry() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.era = 3;

        let mut app = App::new();
        app.insert_resource(world);
        app.init_resource::<ObservationLog>();
        app.add_message::<SpeciesExtinct>();
        app.add_systems(Update, record_events);

        app.world_mut()
            .resource_mut::<Messages<SpeciesExtinct>>()
            .write(SpeciesExtinct {
                species: SpeciesId(2),
            });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].era, 3);
        assert!(log.entries[0].text.contains("species 2"));
    }
}
