// The observation log (GDD §7, §11): a curated feed of salient events, built
// by consuming `sim`'s `Message`s (task 018), not by inspecting the grid
// (TECH_DESIGN.md §4). Read-only with respect to `SimWorld` (TECH_DESIGN.md
// §3.3) — this module never mutates simulation state.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use abiogenesis::config::SimConfig;
use abiogenesis::sim::{AdjacencyObserved, OrganismDied, SpeciesExtinct};
use abiogenesis::world::{SimWorld, TagId};

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

/// Cell indices currently holding a player-`Seed`-ed organism (task 026),
/// written by `input.rs::seed_organism_on_click` on a successful placement.
/// A raw cell index alone can't tell "the organism I placed" from "whatever
/// happens to occupy this cell later" — every consumer that checks this set
/// must remove the entry it matched against (`record_events` on death,
/// `input.rs::cull_on_click` on removal), so a cell's "player-placed-ness"
/// never survives past the one organism it was recorded for. This is
/// presentation-side bookkeeping, not simulation state, so the "no
/// `HashMap`/`HashSet` iteration in sim/world/config" determinism rule
/// (`TECH_DESIGN.md` §5) doesn't apply — only membership checks and
/// removals are ever performed on it, never iteration.
#[derive(Resource, Default)]
pub struct PlayerPlacedCells(pub HashSet<usize>);

/// Cumulative weighted evidence for every `(exerter_tag, receiver_tag)`
/// pair (GDD §7's "B with a hint of C" confirmation model) — the mechanism
/// that progressively reveals `world.matrix`, not a second opacity layer.
/// Sized and laid out exactly like `TagMatrix`
/// (`exerter.0 * size + receiver.0`) so the two stay trivially parallel for
/// task 021's UI.
///
/// Reads through to `world.matrix` for the revealed sign once a pair is
/// confirmed, rather than storing its own copy of the value — simpler, and
/// there's only ever one `SimWorld` to read from.
///
/// There is no "confirmed zero effect" state (GDD §5.9's `0` cell): task
/// 018 only emits `AdjacencyObserved` for pairs with a non-zero matrix
/// entry, so evidence never accumulates for a genuinely-zero pair — the
/// hypothesis grid (021) only distinguishes `?` (unconfirmed) from `±!`
/// (confirmed non-zero, sign shown).
#[derive(Resource)]
pub struct MatrixKnowledge {
    size: usize,
    threshold: f32,
    evidence: Vec<f32>,
}

impl MatrixKnowledge {
    pub fn new(active_tags: usize, threshold: f32) -> Self {
        Self {
            size: active_tags,
            threshold,
            evidence: vec![0.0; active_tags * active_tags],
        }
    }

    pub fn record(&mut self, exerter: TagId, receiver: TagId, weight: f32) {
        let idx = exerter.0 as usize * self.size + receiver.0 as usize;
        self.evidence[idx] += weight;
    }

    pub fn evidence(&self, exerter: TagId, receiver: TagId) -> f32 {
        self.evidence[exerter.0 as usize * self.size + receiver.0 as usize]
    }

    pub fn is_confirmed(&self, exerter: TagId, receiver: TagId) -> bool {
        self.evidence(exerter, receiver) >= self.threshold
    }

    /// The real matrix value for a confirmed pair, `None` if not yet
    /// confirmed. Reads through `world.matrix`, not a stored snapshot.
    pub fn revealed_value(&self, exerter: TagId, receiver: TagId, world: &SimWorld) -> Option<i8> {
        self.is_confirmed(exerter, receiver)
            .then(|| world.matrix.get(exerter, receiver))
    }
}

pub struct NotebookPlugin;

impl Plugin for NotebookPlugin {
    fn build(&self, app: &mut App) {
        // `MatrixKnowledge`'s size depends on `SimConfig::tags`, but not on
        // any randomness `SimWorld` generation adds — `active_tags_early`
        // fixes the *count* of active tags, only the matrix's values are
        // seed-dependent (see `world::SimWorld::new`). Reading `SimConfig`
        // here (already inserted synchronously by `ConfigPlugin::build`,
        // which runs first in `main.rs`'s plugin tuple) avoids needing
        // `SimWorld` to exist yet, so this doesn't need `Startup` ordering
        // against `WorldPlugin` — same pattern as `SimPlugin::build` reading
        // `era_tick_hz` off `SimConfig` directly.
        let config = app.world().resource::<SimConfig>();
        let knowledge = MatrixKnowledge::new(
            config.tags.active_tags_early as usize,
            config.notebook.confirmation_threshold,
        );
        app.insert_resource(knowledge)
            .init_resource::<ObservationLog>()
            .init_resource::<NotebookWindowOpen>()
            .init_resource::<PlayerPlacedCells>()
            .add_systems(
                Update,
                (toggle_notebook, record_events, accumulate_evidence),
            )
            .add_systems(EguiPrimaryContextPass, notebook_window);
    }
}

/// Drains `AdjacencyObserved` (task 018) into `MatrixKnowledge`, weighting
/// each observation `observation_weight_numerator / (1 + n_confounders)`
/// (GDD §7).
fn accumulate_evidence(
    config: Res<SimConfig>,
    mut observed: MessageReader<AdjacencyObserved>,
    mut knowledge: ResMut<MatrixKnowledge>,
) {
    for event in observed.read() {
        let weight =
            config.notebook.observation_weight_numerator / (1.0 + event.n_confounders as f32);
        knowledge.record(event.exerter_tag, event.receiver_tag, weight);
    }
}

/// `tab`: opens/closes the notebook window, mirroring `input.rs`'s
/// `keys.just_pressed(...)` key-handling pattern.
fn toggle_notebook(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<NotebookWindowOpen>) {
    if keys.just_pressed(KeyCode::Tab) {
        open.0 = !open.0;
    }
}

/// Appends salient events to the log. Species extinctions always log.
/// Individual `OrganismDied` events don't, *except* for an organism the
/// player placed via `Seed` (task 026) — GDD §7 wants the log curated, not
/// an unfiltered per-tick feed, but a death of something the player
/// deliberately spent a budget point on is exactly the kind of salient
/// signal that filter should let through. `PlayerPlacedCells` is the
/// marker: every death's cell is checked against it and removed either way
/// (whether or not it matched), so a cell's "player-placed" status is
/// consumed by the first thing that happens to it, never stale.
///
/// TODO: bloom detection (population of a species crossing some multiple of
/// its starting count within an era) as a second salient-event type.
fn record_events(
    world: Res<SimWorld>,
    mut extinctions: MessageReader<SpeciesExtinct>,
    mut deaths: MessageReader<OrganismDied>,
    mut placed: ResMut<PlayerPlacedCells>,
    mut log: ResMut<ObservationLog>,
) {
    for event in extinctions.read() {
        log.entries.push(LogEntry {
            era: world.era,
            text: format!("species {} went extinct", event.species.0),
        });
    }

    for event in deaths.read() {
        if placed.0.remove(&event.cell) {
            let (x, y) = (event.cell % world.width, event.cell / world.width);
            log.entries.push(LogEntry {
                era: world.era,
                text: format!(
                    "your species {} organism at ({x}, {y}) died",
                    event.species.0
                ),
            });
        }
    }
}

/// Golden-angle hue step for tags, same technique `render.rs` uses for
/// `SpeciesId` (`SPECIES_HUE_STEP`) — successive `TagId`s get visually
/// distinct colors with no per-tag configuration.
const TAG_HUE_STEP: f32 = 137.5;

/// A tag's color, deterministic from its id. Tags stay "nameless
/// glyphs/colors, learned empirically" (GDD §11) — never rendered as a raw
/// number.
fn tag_color(tag: TagId) -> egui::Color32 {
    let hue = (tag.0 as f32 * TAG_HUE_STEP % 360.0) / 360.0;
    egui::ecolor::Hsva::new(hue, 0.75, 0.9, 1.0).into()
}

const TAG_GLYPH: &str = "●";

/// Draws the notebook as its own `egui::Window`, sharing the HUD's egui
/// context (`ui.rs`'s dedicated full-viewport camera) rather than a second
/// camera — `bevy_egui` supports multiple windows/panels per frame from the
/// same `EguiPrimaryContextPass` context.
fn notebook_window(
    mut contexts: EguiContexts,
    mut open: ResMut<NotebookWindowOpen>,
    log: Res<ObservationLog>,
    world: Res<SimWorld>,
    knowledge: Res<MatrixKnowledge>,
) -> Result {
    if !open.0 {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Notebook")
        .open(&mut open.0)
        .show(ctx, |ui| {
            ui.heading("Observation log");
            egui::ScrollArea::vertical()
                .id_salt("observation_log")
                .max_height(150.0)
                .show(ui, |ui| {
                    if log.entries.is_empty() {
                        ui.weak("(no observations yet)");
                    }
                    for entry in &log.entries {
                        ui.label(format!("Era {}: {}", entry.era, entry.text));
                    }
                });

            ui.separator();
            ui.heading("Hypothesis grid");
            hypothesis_grid(ui, &world, &knowledge);

            ui.separator();
            ui.heading("Catalog");
            catalog_panel(ui, &world);
        });
    Ok(())
}

/// The `active_tags x active_tags` evidence table (GDD §7, §5.9): row =
/// exerting tag, column = receiving tag, matching `TagMatrix::get`'s own
/// convention. Never renders an unconfirmed cell's real value — only
/// `MatrixKnowledge::is_confirmed` gates a sign reveal.
///
/// Player-authored conjectures (GDD §5.9's `±?` state — marking a guess
/// before it's confirmed) aren't implemented: cut for this task, left as a
/// follow-up rather than a half-built annotation feature.
fn hypothesis_grid(ui: &mut egui::Ui, world: &SimWorld, knowledge: &MatrixKnowledge) {
    egui::Grid::new("hypothesis_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label(""); // corner cell
            for &receiver in &world.active_tags {
                ui.colored_label(tag_color(receiver), TAG_GLYPH);
            }
            ui.end_row();

            for &exerter in &world.active_tags {
                ui.colored_label(tag_color(exerter), TAG_GLYPH);
                for &receiver in &world.active_tags {
                    if exerter == receiver {
                        // The diagonal is always 0 by construction
                        // (`world.rs`'s matrix generation): not a real
                        // hypothesis, so it's shown distinct from `?`.
                        ui.weak("·");
                    } else if let Some(value) = knowledge.revealed_value(exerter, receiver, world) {
                        ui.label(if value > 0 { "+!" } else { "-!" });
                    } else {
                        ui.weak("?");
                    }
                }
                ui.end_row();
            }
        });
}

/// Lists the active tag pool and every species' readable genome fields
/// alongside its (still-opaque) tags. Phase 2's whole active pool is
/// visible from the seed selector already, so per-encounter tag discovery
/// isn't modeled here — that's a Phase 3 worldgen concern.
fn catalog_panel(ui: &mut egui::Ui, world: &SimWorld) {
    ui.label("Active tags");
    ui.horizontal(|ui| {
        for &tag in &world.active_tags {
            ui.colored_label(tag_color(tag), TAG_GLYPH);
        }
    });

    ui.add_space(4.0);
    ui.label("Species");
    for (id, species) in world.species.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!(
                "species {id}: {:?} · temp {:.2}±{:.2}",
                species.metabolism, species.temp_optimum, species.temp_tolerance
            ));
            for &tag in &species.tags {
                ui.colored_label(tag_color(tag), TAG_GLYPH);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abiogenesis::config::SimConfig;
    use abiogenesis::world::SpeciesId;

    fn app_for_record_events(world: SimWorld) -> App {
        let mut app = App::new();
        app.insert_resource(world);
        app.init_resource::<ObservationLog>();
        app.init_resource::<PlayerPlacedCells>();
        app.add_message::<SpeciesExtinct>();
        app.add_message::<OrganismDied>();
        app.add_systems(Update, record_events);
        app
    }

    #[test]
    fn extinction_message_appends_a_log_entry() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.era = 3;
        let mut app = app_for_record_events(world);

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

    #[test]
    fn a_player_placed_organisms_death_logs_and_consumes_the_marker() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.era = 4;
        let mut app = app_for_record_events(world);
        app.world_mut()
            .resource_mut::<PlayerPlacedCells>()
            .0
            .insert(137);

        app.world_mut()
            .resource_mut::<Messages<OrganismDied>>()
            .write(OrganismDied {
                cell: 137,
                species: SpeciesId(0),
            });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].era, 4);
        assert!(
            log.entries[0].text.contains("species 0"),
            "got: {}",
            log.entries[0].text
        );
        let placed = app.world().resource::<PlayerPlacedCells>();
        assert!(
            !placed.0.contains(&137),
            "the marker must be consumed once logged"
        );
    }

    #[test]
    fn a_reproduced_organisms_death_does_not_log() {
        // Same death event, but the cell was never recorded as
        // player-placed (i.e. this organism was born via reproduction) —
        // task 019's "no per-tick flood" guarantee must hold.
        let config = SimConfig::default();
        let world = SimWorld::new(42, &config);
        let mut app = app_for_record_events(world);

        app.world_mut()
            .resource_mut::<Messages<OrganismDied>>()
            .write(OrganismDied {
                cell: 42,
                species: SpeciesId(0),
            });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert!(
            log.entries.is_empty(),
            "an untracked (reproduced) organism's death must not log"
        );
    }

    #[test]
    fn a_player_placed_organism_still_logs_after_surviving_several_eras() {
        // The marker must not expire on its own — only a matching death (or
        // an explicit clear, e.g. Cull/reseed) removes it.
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.era = 1;
        let mut app = app_for_record_events(world);
        app.world_mut()
            .resource_mut::<PlayerPlacedCells>()
            .0
            .insert(9);

        // A few eras pass with no death — nothing happens, marker persists.
        for era in 2..=5 {
            app.world_mut().resource_mut::<SimWorld>().era = era;
            app.update();
        }
        assert!(app.world().resource::<PlayerPlacedCells>().0.contains(&9));

        app.world_mut()
            .resource_mut::<Messages<OrganismDied>>()
            .write(OrganismDied {
                cell: 9,
                species: SpeciesId(1),
            });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].era, 5);
    }

    const THRESHOLD: f32 = 3.0;

    #[test]
    fn three_isolated_observations_reach_the_threshold_exactly() {
        // GDD §7: n_confounders = 0 -> weight 1.0 each; 3 * 1.0 == threshold.
        let mut knowledge = MatrixKnowledge::new(2, THRESHOLD);
        for _ in 0..3 {
            knowledge.record(TagId(0), TagId(1), 1.0);
        }
        assert!((knowledge.evidence(TagId(0), TagId(1)) - 3.0).abs() < 1e-6);
        assert!(knowledge.is_confirmed(TagId(0), TagId(1)));
    }

    #[test]
    fn confounded_observations_need_four_times_as_many() {
        // n_confounders = 3 -> weight 0.25 each; 12 * 0.25 == threshold.
        let mut knowledge = MatrixKnowledge::new(2, THRESHOLD);
        for _ in 0..11 {
            knowledge.record(TagId(0), TagId(1), 0.25);
        }
        assert!(
            !knowledge.is_confirmed(TagId(0), TagId(1)),
            "11 * 0.25 = 2.75, just under threshold"
        );

        knowledge.record(TagId(0), TagId(1), 0.25);
        assert!(
            knowledge.is_confirmed(TagId(0), TagId(1)),
            "the 12th observation crosses the threshold"
        );
    }

    #[test]
    fn accumulate_evidence_applies_the_confounder_weight() {
        let config = SimConfig::default();
        let mut app = App::new();
        app.insert_resource(config.clone());
        app.insert_resource(MatrixKnowledge::new(
            5,
            config.notebook.confirmation_threshold,
        ));
        app.add_message::<AdjacencyObserved>();
        app.add_systems(Update, accumulate_evidence);

        app.world_mut()
            .resource_mut::<Messages<AdjacencyObserved>>()
            .write(AdjacencyObserved {
                receiver_species: SpeciesId(0),
                exerter_tag: TagId(0),
                receiver_tag: TagId(1),
                n_confounders: 3,
            });
        app.update();

        let knowledge = app.world().resource::<MatrixKnowledge>();
        assert!(
            (knowledge.evidence(TagId(0), TagId(1)) - 0.25).abs() < 1e-6,
            "expected numerator(1.0) / (1 + 3 confounders) = 0.25, got {}",
            knowledge.evidence(TagId(0), TagId(1))
        );
    }

    #[test]
    fn unconfirmed_pairs_do_not_reveal_a_value() {
        let config = SimConfig::default();
        let world = SimWorld::new(42, &config);
        let (exerter, receiver) = (world.active_tags[0], world.active_tags[1]);
        let mut knowledge = MatrixKnowledge::new(world.active_tags.len(), THRESHOLD);

        assert_eq!(knowledge.revealed_value(exerter, receiver, &world), None);

        knowledge.record(exerter, receiver, THRESHOLD);
        assert_eq!(
            knowledge.revealed_value(exerter, receiver, &world),
            Some(world.matrix.get(exerter, receiver))
        );
    }
}
