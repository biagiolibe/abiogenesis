// The observation log (GDD §7, §11): a curated feed of salient events, built
// by consuming `sim`'s `Message`s (task 018), not by inspecting the grid
// (TECH_DESIGN.md §4). Read-only with respect to `SimWorld` (TECH_DESIGN.md
// §3.3) — this module never mutates simulation state.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use abiogenesis::config::SimConfig;
use abiogenesis::objectives::ObjectiveAdvanced;
use abiogenesis::sim::{
    AdjacencyObserved, EraCompleted, OrganismBorn, OrganismDied, SpeciesExtinct,
    TerrainGateObserved, TerrainRevealed,
};
use abiogenesis::state::GameState;
use abiogenesis::world::{Metabolism, Mode, SimWorld, SpeciesId, TagId, TagSlot, TerrainKind};

use crate::render::{metabolism_glyph, species_color, species_label, terrain_glyph};
use crate::text;
use crate::ui::{hud_panel, HudControlIntents};

/// One curated log line, tagged with the era it happened in. `text`
/// describes the event only; the window prepends the era. `species` is the
/// entry's subject, if it has one — carried separately from `text` (rather
/// than baked into the string) so the window can render a `species_color`
/// swatch alongside the line, the same glyph+color pattern `ui.rs`'s
/// Population/Seed Palette panels use. `None` for entry kinds with no
/// species subject (task 054's matrix-confirmation lines are about a tag
/// pair, not any one species) — the window renders those with a neutral
/// glyph instead of a colored swatch.
pub struct LogEntry {
    pub era: u32,
    pub species: Option<SpeciesId>,
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

/// Whether the notebook window has ever been opened, set once on the first
/// `NotebookWindowOpen` transition to `true` and never reset. Drives the
/// second onboarding hint (task 053): shown until the player opens the
/// notebook for the first time, then gone for good.
#[derive(Resource, Default)]
pub struct NotebookEverOpened(pub bool);

/// Whether a matrix-cell confirmation (task 054, GDD §7's "aha" moment) has
/// happened since the player last opened the notebook — drives a badge on
/// the HUD's notebook affordance so the event is noticeable even with the
/// window closed. Set on every fresh confirmation, cleared on the next
/// `NotebookWindowOpen` transition to `true`.
#[derive(Resource, Default)]
pub struct NotebookHasUnseenConfirmation(pub bool);

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

/// Whether the player has ever placed an organism via `Seed`, set once by
/// `input.rs::seed_organism_on_click` on the first successful placement and
/// never reset. Unlike `PlayerPlacedCells`, this never empties back out — a
/// later death removes the cell from `PlayerPlacedCells` (its membership is
/// per-organism, consumed on death) but must not make it look like the
/// player has never seeded anything, which is what the first onboarding
/// hint (task 053) needs to key off of.
#[derive(Resource, Default)]
pub struct EverSeeded(pub bool);

/// Running count of `OrganismBorn` events per species within the current
/// era (task 063), indexed by `SpeciesId`, growing on demand the same way
/// `ui.rs::PopulationTrends` tolerates `Splice` adding new species mid-run.
/// Drained into one curated `LogEntry` per species on `EraCompleted`
/// (`tally_births`), then reset to zero — this never accumulates across
/// eras, unlike the death/extinction logs it sits alongside.
#[derive(Resource, Default)]
pub struct BirthTally(Vec<u32>);

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

    /// Adds `weight` to a pair's evidence, returning `true` if this call is
    /// the one that pushed it from below `threshold` to at/above it (GDD
    /// §7's "aha" moment) — `false` on every other call, including repeat
    /// evidence for an already-confirmed pair.
    pub fn record(&mut self, exerter: TagSlot, receiver: TagSlot, weight: f32) -> bool {
        let idx = exerter.0 as usize * self.size + receiver.0 as usize;
        let was_confirmed = self.evidence[idx] >= self.threshold;
        self.evidence[idx] += weight;
        !was_confirmed && self.evidence[idx] >= self.threshold
    }

    pub fn evidence(&self, exerter: TagSlot, receiver: TagSlot) -> f32 {
        self.evidence[exerter.0 as usize * self.size + receiver.0 as usize]
    }

    pub fn is_confirmed(&self, exerter: TagSlot, receiver: TagSlot) -> bool {
        self.evidence(exerter, receiver) >= self.threshold
    }

    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// The real matrix value for a confirmed pair, `None` if not yet
    /// confirmed. Reads through `world.matrix`, not a stored snapshot.
    pub fn revealed_value(
        &self,
        exerter: TagSlot,
        receiver: TagSlot,
        world: &SimWorld,
    ) -> Option<i8> {
        self.is_confirmed(exerter, receiver)
            .then(|| world.matrix.get(exerter, receiver))
    }
}

/// Number of `TerrainKind` variants (`Sea`/`Plain`/`Hill`/`Mountain`,
/// `world.rs:163-168`) — `TerrainKnowledge`'s per-tag row width.
const TERRAIN_KIND_COUNT: usize = 4;

fn terrain_index(terrain: TerrainKind) -> usize {
    match terrain {
        TerrainKind::Sea => 0,
        TerrainKind::Plain => 1,
        TerrainKind::Hill => 2,
        TerrainKind::Mountain => 3,
    }
}

/// Cumulative weighted evidence for every `(TagSlot, TerrainKind)` pair
/// (task 097, `redesign/abiogenesis-living-world.md` §1) — the gradual,
/// exposure-based counterpart to `MatrixKnowledge`, but keyed on a tag's
/// terrain gate instead of a tag pair. A separate, small resource rather
/// than a `MatrixKnowledge` rework: conditionality is a per-tag fact ("does
/// conditional glyph G require/exclude which terrain"), much smaller than
/// the per-pair matrix hypothesis space, so the two stay additive instead
/// of inflating the pair matrix with a terrain axis it was never designed
/// to carry.
///
/// Distinct from task 099's zone-entry reveal: that one fires once,
/// deterministically, the first time a species' lineage sets foot on a
/// trigger terrain. This one accumulates evidence from every
/// `TerrainGateObserved` (`sim.rs`'s `tag_gate_satisfied`), pass or fail,
/// the same "an observation is evidence regardless of outcome" reasoning
/// `AdjacencyObserved`/`MatrixKnowledge` already use — so a species that
/// never enters the trigger terrain can still, eventually, have its
/// catalog badge confirmed purely from evaluations on other terrain (each
/// a data point about where the tag *isn't* active, per `Mode::Repressible`'s
/// framing).
#[derive(Resource)]
pub struct TerrainKnowledge {
    threshold: f32,
    evidence: Vec<f32>,
}

impl TerrainKnowledge {
    pub fn new(active_tags: usize, threshold: f32) -> Self {
        Self {
            threshold,
            evidence: vec![0.0; active_tags * TERRAIN_KIND_COUNT],
        }
    }

    fn index(&self, tag: TagSlot, terrain: TerrainKind) -> usize {
        tag.0 as usize * TERRAIN_KIND_COUNT + terrain_index(terrain)
    }

    /// Adds `weight` to a `(tag, terrain)` pair's evidence, returning `true`
    /// if this call is the one that pushed it from below `threshold` to
    /// at/above it — `false` on every other call, including repeat
    /// evidence for an already-confirmed pair. Mirrors
    /// `MatrixKnowledge::record` exactly.
    pub fn record(&mut self, tag: TagSlot, terrain: TerrainKind, weight: f32) -> bool {
        let idx = self.index(tag, terrain);
        let was_confirmed = self.evidence[idx] >= self.threshold;
        self.evidence[idx] += weight;
        !was_confirmed && self.evidence[idx] >= self.threshold
    }

    pub fn evidence(&self, tag: TagSlot, terrain: TerrainKind) -> f32 {
        self.evidence[self.index(tag, terrain)]
    }

    pub fn is_confirmed(&self, tag: TagSlot, terrain: TerrainKind) -> bool {
        self.evidence(tag, terrain) >= self.threshold
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
        let terrain_knowledge = TerrainKnowledge::new(
            config.tags.active_tags_early as usize,
            config.notebook.confirmation_threshold,
        );
        app.insert_resource(knowledge)
            .insert_resource(terrain_knowledge)
            .init_resource::<ObservationLog>()
            .init_resource::<NotebookWindowOpen>()
            .init_resource::<NotebookEverOpened>()
            .init_resource::<PlayerPlacedCells>()
            .init_resource::<EverSeeded>()
            .init_resource::<NotebookHasUnseenConfirmation>()
            .init_resource::<BirthTally>()
            .add_systems(
                Update,
                (
                    toggle_notebook,
                    record_events,
                    accumulate_evidence,
                    accumulate_terrain_evidence,
                    tally_births,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (notebook_window, clear_stray_tab_focus.after(hud_panel))
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Drains `AdjacencyObserved` (task 018) into `MatrixKnowledge`, weighting
/// each observation `observation_weight_numerator / (1 + n_confounders)`
/// (GDD §7). Every pair `record` reports as newly confirmed gets a log
/// entry and raises the HUD badge (task 054) — the game's central discovery
/// beat, previously silent outside the hypothesis grid itself.
fn accumulate_evidence(
    config: Res<SimConfig>,
    world: Res<SimWorld>,
    mut observed: MessageReader<AdjacencyObserved>,
    mut knowledge: ResMut<MatrixKnowledge>,
    mut log: ResMut<ObservationLog>,
    mut unseen: ResMut<NotebookHasUnseenConfirmation>,
) {
    for event in observed.read() {
        let from_glyph = tag_glyph(world.active_tags[event.exerter_tag.0 as usize]);
        let to_glyph = tag_glyph(world.active_tags[event.receiver_tag.0 as usize]);

        let weight =
            config.notebook.observation_weight_numerator / (1.0 + event.n_confounders as f32);
        let newly_confirmed = knowledge.record(event.exerter_tag, event.receiver_tag, weight);
        if newly_confirmed {
            let positive = world.matrix.get(event.exerter_tag, event.receiver_tag) > 0;
            log.entries.push(LogEntry {
                era: world.era,
                species: None,
                text: text::confirmation_message(from_glyph, to_glyph, positive),
            });
            unseen.0 = true;
        }
    }
}

/// Drains `TerrainGateObserved` (task 097, `sim.rs`'s `tag_gate_satisfied`)
/// into `TerrainKnowledge`, mirroring `accumulate_evidence`'s shape with a
/// flat `observation_weight_numerator` weight — a gate evaluation has no
/// `AdjacencyObserved`-style confounder count to divide by, so this is the
/// `n_confounders = 0` case of the same formula. Every pair `record`
/// reports as newly confirmed gets its own log entry (distinct from task
/// 099's zone-entry reveal, which fires on a different, deterministic
/// trigger) and raises the same HUD unseen-confirmation badge.
fn accumulate_terrain_evidence(
    config: Res<SimConfig>,
    world: Res<SimWorld>,
    mut observed: MessageReader<TerrainGateObserved>,
    mut knowledge: ResMut<TerrainKnowledge>,
    mut log: ResMut<ObservationLog>,
    mut unseen: ResMut<NotebookHasUnseenConfirmation>,
) {
    for event in observed.read() {
        let newly_confirmed = knowledge.record(
            event.tag,
            event.terrain,
            config.notebook.observation_weight_numerator,
        );
        if newly_confirmed {
            let glyph = tag_glyph(world.active_tags[event.tag.0 as usize]);
            let tag_id = world.active_tags[event.tag.0 as usize];
            // `event.tag` only ever carries a `TerrainGateObserved` when
            // `SimWorld::conditional_gate` returned `Some` at emission time
            // (`sim.rs::tag_gate_satisfied`) — the mode is always available
            // here.
            if let Some(conditional) = world.conditional_gate(tag_id) {
                log.entries.push(LogEntry {
                    era: world.era,
                    species: None,
                    text: text::terrain_gate_confirmed_message(
                        glyph,
                        event.terrain,
                        conditional.mode,
                    ),
                });
                unseen.0 = true;
            }
        }
    }
}

/// `tab` (or the HUD's Notebook button, task 094 — `HudControlIntents::
/// toggle_notebook`, consumed here so the button and the shortcut share this
/// one implementation): opens/closes the notebook window, mirroring
/// `input.rs`'s `keys.just_pressed(...)` key-handling pattern. Opening also
/// latches `NotebookEverOpened` (task 053) and clears the confirmation badge
/// (task 054) — the player has now seen whatever the badge was pointing at.
fn toggle_notebook(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<NotebookWindowOpen>,
    mut ever_opened: ResMut<NotebookEverOpened>,
    mut unseen_confirmation: ResMut<NotebookHasUnseenConfirmation>,
    mut intents: ResMut<HudControlIntents>,
) {
    let triggered = keys.just_pressed(KeyCode::Tab) || intents.toggle_notebook;
    intents.toggle_notebook = false;
    if triggered {
        open.0 = !open.0;
        if open.0 {
            ever_opened.0 = true;
            unseen_confirmation.0 = false;
        }
    }
}

/// Task 091: `Tab` is meant to be exclusively "toggle the notebook," but
/// egui's own keyboard navigation *also* treats `Tab` as "focus the next
/// widget" — with nothing telling it otherwise, the same keypress that
/// opens/closes the notebook (`toggle_notebook`, `Update`) also leaves
/// keyboard focus sitting on whichever sidebar action button
/// (`ui::hud_panel`'s `action_icon_row`) egui's navigation landed on this
/// frame. Runs in `EguiPrimaryContextPass`, ordered `.after(hud_panel)` so
/// it observes focus *after* the sidebar's buttons have had their chance to
/// claim it via egui's own `Tab` handling that frame, then surrenders
/// whatever ended up focused — this app has no keyboard-editable widget
/// during `GameState::Playing` (menu.rs's seed `TextEdit` is a different
/// screen), so clearing focus here can never interrupt real text input.
fn clear_stray_tab_focus(keys: Res<ButtonInput<KeyCode>>, mut contexts: EguiContexts) -> Result {
    if !keys.just_pressed(KeyCode::Tab) {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    ctx.memory_mut(|memory| {
        if let Some(focused) = memory.focused() {
            memory.surrender_focus(focused);
        }
    });
    Ok(())
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
    mut objectives_advanced: MessageReader<ObjectiveAdvanced>,
    mut terrain_revealed: MessageReader<TerrainRevealed>,
    mut placed: ResMut<PlayerPlacedCells>,
    mut log: ResMut<ObservationLog>,
) {
    for event in extinctions.read() {
        log.entries.push(LogEntry {
            era: world.era,
            species: Some(event.species),
            text: text::extinction_message(&species_label(&world, event.species)),
        });
    }

    for event in objectives_advanced.read() {
        log.entries.push(LogEntry {
            era: world.era,
            species: None,
            text: text::objective_advanced_message(event.index),
        });
    }

    for event in terrain_revealed.read() {
        log.entries.push(LogEntry {
            era: world.era,
            species: Some(event.species),
            text: text::terrain_reveal_message(
                &species_label(&world, event.species),
                tag_glyph(event.tag),
                event.terrain,
            ),
        });
    }

    for event in deaths.read() {
        if placed.0.remove(&event.cell) {
            let (x, y) = (event.cell % world.width, event.cell / world.width);
            let metabolism = world.species[event.species.0 as usize].metabolism;
            log.entries.push(LogEntry {
                era: world.era,
                species: Some(event.species),
                text: text::player_organism_death_message(
                    &species_label(&world, event.species),
                    x,
                    y,
                    metabolism,
                    event.gain,
                    event.env_fit,
                    event.interaction_delta,
                    event.upkeep,
                    event.crowding_penalty,
                    event.predation_loss,
                ),
            });
        }
    }
}

/// Tallies `OrganismBorn` events per species as they happen, then — on
/// `EraCompleted` — pushes one curated `LogEntry` per species with at least
/// one birth that era ("Kael: +3 births this era") and resets the tally.
/// The real "individuals crossed `repro_threshold`" signal (task 063),
/// replacing the old HUD average-vs-threshold comparison that could never
/// actually show this: a zero-birth species gets no line, keeping this a
/// once-per-era summary, not a flood.
fn tally_births(
    world: Res<SimWorld>,
    mut born: MessageReader<OrganismBorn>,
    mut era_completed: MessageReader<EraCompleted>,
    mut tally: ResMut<BirthTally>,
    mut log: ResMut<ObservationLog>,
) {
    for event in born.read() {
        let idx = event.species.0 as usize;
        if tally.0.len() <= idx {
            tally.0.resize(idx + 1, 0);
        }
        tally.0[idx] += 1;
    }

    let mut completed_era = None;
    for event in era_completed.read() {
        completed_era = Some(event.era);
    }
    let Some(era) = completed_era else {
        return;
    };

    for (idx, &count) in tally.0.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let species = SpeciesId(idx as u8);
        log.entries.push(LogEntry {
            era,
            species: Some(species),
            text: text::birth_log_message(&species_label(&world, species), count),
        });
    }
    tally.0.fill(0);
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

/// Marks a matrix-confirmation log line (task 054) in place of the
/// species-color swatch other entries get — a confirmation is about a tag
/// pair, not any one species, so there's no `species_color` to render.
const CONFIRMATION_GLYPH: &str = "★";

/// Fixed Greek-letter alphabet for `tag_glyph` (task 029): opaque, stable
/// within a run, deterministic from `TagId` — never a hint at the tag's
/// effect (GDD §11 "nameless glyphs/colors, learned empirically").
const TAG_LETTERS: [&str; 24] = [
    "α", "β", "γ", "δ", "ε", "ζ", "η", "θ", "ι", "κ", "λ", "μ", "ν", "ξ", "ο", "π", "ρ", "σ", "τ",
    "υ", "φ", "χ", "ψ", "ω",
];

/// A tag's stable opaque letter, alongside its color, so a player can say
/// "tag α" instead of pointing at a colored dot — purely a more
/// distinguishable handle, not a hint at meaning (GDD §11).
pub fn tag_glyph(tag: TagId) -> &'static str {
    TAG_LETTERS[tag.0 as usize % TAG_LETTERS.len()]
}

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
    terrain_knowledge: Res<TerrainKnowledge>,
    config: Res<SimConfig>,
) -> Result {
    if !open.0 {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Notebook")
        .open(&mut open.0)
        .show(ctx, |ui| {
            ui.heading(text::HEADING_OBSERVATION_LOG);
            egui::ScrollArea::vertical()
                .id_salt("observation_log")
                .max_height(220.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if log.entries.is_empty() {
                        ui.weak(text::NO_OBSERVATIONS_YET);
                    }
                    for entry in &log.entries {
                        ui.horizontal(|ui| {
                            match entry.species {
                                Some(species) => {
                                    ui.colored_label(species_color(species), TAG_GLYPH);
                                }
                                None => {
                                    ui.label(CONFIRMATION_GLYPH);
                                }
                            }
                            ui.label(text::log_entry_line(entry.era, &entry.text));
                        });
                    }
                });

            ui.separator();
            ui.heading(text::HEADING_HYPOTHESIS_GRID);
            hypothesis_grid(ui, &world, &knowledge, &config);

            ui.separator();
            ui.heading(text::HEADING_CATALOG);
            catalog_panel(ui, &world, &config, &terrain_knowledge);
        });
    Ok(())
}

/// Node radius for the hypothesis graph (task 031), in points.
const NODE_RADIUS: f32 = 14.0;

const ARROW_LENGTH: f32 = 8.0;
const ARROW_WIDTH: f32 = 6.0;

const EDGE_POSITIVE_COLOR: egui::Color32 = egui::Color32::from_rgb(96, 200, 120);
const EDGE_NEGATIVE_COLOR: egui::Color32 = egui::Color32::from_rgb(220, 96, 96);

/// Color for a pair with `evidence > 0.0` but not yet confirmed (task 028,
/// dashed line since task 102): carries no sign/magnitude, only "something
/// has been observed here" — visually distinct from a confirmed edge (no
/// arrowhead, uniform thickness) so it can never be mistaken for one.
const PARTIAL_EVIDENCE_COLOR: egui::Color32 = egui::Color32::from_gray(130);
const PARTIAL_EDGE_STROKE: f32 = 1.5;

/// Perpendicular bow distance (task 102) applied to a confirmed/partial edge
/// when the reverse direction between the same two tags also has something
/// to draw — the two directions curve apart into two arcs instead of
/// overlapping straight lines. Tuned by eye against the densest live case
/// (a 5-6 active-tag world with several confirmed bidirectional pairs):
/// large enough that the two arcs and their arrowheads/dashes are
/// unambiguously separate at `NODE_RADIUS`-scale node spacing, small enough
/// that a curve still reads as "the same short hop, bent" rather than a
/// wide detour.
const EDGE_BOW: f32 = 16.0;

/// Points sampled along a curved edge to approximate it as short straight
/// segments for both dashing (`draw_dashed_line`) and arrowhead-tangent
/// lookup (`draw_edge`) — egui's `Painter` draws bezier shapes directly for
/// solid strokes, but dashing needs the flattened point list to step along.
const CURVE_FLATTEN_TOLERANCE: f32 = 0.5;

/// The hypothesis graph (GDD §7, §5.9), replacing task 021's
/// `active_tags x active_tags` spreadsheet table: `world.active_tags` as
/// nodes arranged around a circle, a directed edge drawn where
/// `MatrixKnowledge::revealed_value` returns `Some`, or a small dot (task
/// 028) where evidence has started accumulating but hasn't crossed the
/// confirmation threshold yet — distinguishing "no interaction exists" from
/// "an interaction exists but wasn't observed enough yet," which a plain
/// absence of edge couldn't tell apart. A pair with zero evidence draws
/// nothing at all, the same information boundary the old table honored
/// (never a hint beyond `is_confirmed`, and the partial marker never reveals
/// sign). The diagonal (`exerter == receiver`, always 0 by construction) is
/// skipped entirely rather than drawn as a self-loop — there was never a
/// real hypothesis there.
///
/// **Reveal-on-first-observation (task 101):** a tag with zero evidence in
/// every direction (`has_no_evidence`) doesn't occupy a node slot at all —
/// no ghost/placeholder ring, it simply isn't laid out yet. Positions are
/// recomputed each call over just the currently-visible subset, evenly
/// spaced, so a 2-visible-tag state and a 5-visible-tag state both read as
/// evenly laid out rather than a couple of dots lost in a mostly-empty ring.
/// This is a static recompute-on-reveal, not a force-directed layout —
/// deliberately, evidence is monotonic within a run (`MatrixKnowledge::
/// record` only adds) so a tag never needs to disappear again once visible,
/// and continuous physics was judged not worth the complexity here.
///
/// Player-authored conjectures (GDD §5.9's `±?` state — marking a guess
/// before it's confirmed) aren't implemented: cut for this task, left as a
/// follow-up rather than a half-built annotation feature.
fn hypothesis_grid(
    ui: &mut egui::Ui,
    world: &SimWorld,
    knowledge: &MatrixKnowledge,
    config: &SimConfig,
) {
    let tags = &world.active_tags;
    let desired_size = egui::vec2(ui.available_width().clamp(200.0, 320.0), 240.0);
    let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
    let rect = response.rect;
    let center = rect.center();
    let radius = (rect.width().min(rect.height()) / 2.0 - NODE_RADIUS - 6.0).max(10.0);

    // Only tags with at least some evidence occupy a slot on the ring — an
    // untouched tag is invisible rather than a dashed-ring placeholder (task
    // 101), so the layout is always evenly spaced for however many tags are
    // *currently* relevant instead of reserving a slot for the full pool.
    let visible: Vec<usize> = (0..tags.len())
        .filter(|&i| !has_no_evidence(TagSlot(i as u8), tags.len(), knowledge))
        .collect();

    if visible.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text::NO_OBSERVATIONS_YET,
            egui::FontId::proportional(13.0),
            ui.visuals().weak_text_color(),
        );
        return;
    }

    // Indexed by tag index (not position within `visible`) so lookups below
    // stay simple; entries for invisible tags are never read.
    let mut positions = vec![center; tags.len()];
    for (vi, &i) in visible.iter().enumerate() {
        let angle =
            vi as f32 / visible.len() as f32 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        positions[i] = center + radius * egui::vec2(angle.cos(), angle.sin());
    }

    // A pair has "something to draw" in a direction once it's confirmed or
    // has any partial evidence — used both to decide what to draw and,
    // below, whether the *reverse* direction also draws something (task
    // 102's bidirectional-bow case).
    let has_something = |exerter: TagSlot, receiver: TagSlot| {
        knowledge.revealed_value(exerter, receiver, world).is_some()
            || knowledge.evidence(exerter, receiver) > 0.0
    };

    for &ei in &visible {
        for &ri in &visible {
            if ei == ri {
                continue;
            }
            let (exerter, receiver) = (TagSlot(ei as u8), TagSlot(ri as u8));
            if !has_something(exerter, receiver) {
                continue;
            }
            // Bow apart only when the reverse direction also has something
            // to draw — a lone direction stays a straight line (no
            // unnecessary curvature). Sign keyed off index order (not which
            // direction is "first") so the two directions of the same pair
            // always bow to opposite sides regardless of draw order.
            let bidirectional = has_something(receiver, exerter);
            let curvature = if bidirectional {
                if ei < ri {
                    EDGE_BOW
                } else {
                    -EDGE_BOW
                }
            } else {
                0.0
            };

            if let Some(value) = knowledge.revealed_value(exerter, receiver, world) {
                let color = if value > 0 {
                    EDGE_POSITIVE_COLOR
                } else {
                    EDGE_NEGATIVE_COLOR
                };
                draw_edge(
                    &painter,
                    positions[ei],
                    positions[ri],
                    color,
                    value,
                    config,
                    curvature,
                );
            } else {
                draw_dashed_line(&painter, positions[ei], positions[ri], curvature);
            }
        }
    }

    for &i in &visible {
        let tag = tags[i];
        let slot = TagSlot(i as u8);
        let pos = positions[i];
        painter.circle_filled(pos, NODE_RADIUS, tag_color(tag));
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            tag_glyph(tag),
            egui::FontId::proportional(13.0),
            egui::Color32::BLACK,
        );

        let node_rect = egui::Rect::from_center_size(pos, egui::Vec2::splat(NODE_RADIUS * 2.0));
        let node_id = response.id.with(("hypothesis_node", tag.0));
        let node_response = ui.interact(node_rect, node_id, egui::Sense::hover());
        node_response.on_hover_text(node_tooltip_text(slot, tag, tags, world, knowledge));
    }
}

/// Stroke width range for a confirmed edge (task 061, re-tuned task 102):
/// since the `±N` numeric label is gone, thickness is the *only* remaining
/// carrier of magnitude, so the weak↔strong gap widens from the original
/// 1.5pt to 4.5pt — clearly distinguishable at a glance without text.
const EDGE_STROKE_WEAK: f32 = 1.5;
const EDGE_STROKE_STRONG: f32 = 6.0;

/// Draws one directed edge, straight or bowed (`curvature != 0.0`, task
/// 102), capped with a small triangular arrowhead at the receiver end.
/// `value` is the confirmed matrix entry (`±1` or `±2` at the default
/// config): its magnitude, scaled against `config.tags.effect_intensity_max`
/// rather than a hardcoded `2`, linearly interpolates the stroke width
/// between `EDGE_STROKE_WEAK` and `EDGE_STROKE_STRONG` — color (sign) and
/// thickness (magnitude) are the edge's entire grammar now, no on-grid text.
fn draw_edge(
    painter: &egui::Painter,
    from: egui::Pos2,
    to: egui::Pos2,
    color: egui::Color32,
    value: i8,
    config: &SimConfig,
    curvature: f32,
) {
    let dir = (to - from).normalized();
    let start = from + dir * NODE_RADIUS;
    let end = to - dir * NODE_RADIUS;

    let intensity_max = (config.tags.effect_intensity_max.max(1)) as f32;
    let t = (value.unsigned_abs() as f32 / intensity_max).clamp(0.0, 1.0);
    let width = EDGE_STROKE_WEAK + (EDGE_STROKE_STRONG - EDGE_STROKE_WEAK) * t;
    let stroke = egui::Stroke::new(width, color);

    // Tangent direction at the endpoint, used for the arrowhead — the
    // straight chord direction for an uncurved edge, or the curve's local
    // direction just before `end` once it's bowed (an arrowhead aligned to
    // the chord instead would visibly point off the curve).
    let tangent = if curvature.abs() < f32::EPSILON {
        painter.line_segment([start, end], stroke);
        dir
    } else {
        let control = start.lerp(end, 0.5) + perp(dir) * curvature;
        let bezier = egui::epaint::QuadraticBezierShape::from_points_stroke(
            [start, control, end],
            false,
            egui::Color32::TRANSPARENT,
            stroke,
        );
        let near_end = bezier.sample(0.9);
        painter.add(egui::Shape::QuadraticBezier(bezier));
        (end - near_end).normalized()
    };

    let perp_t = perp(tangent);
    painter.add(egui::Shape::convex_polygon(
        vec![
            end,
            end - tangent * ARROW_LENGTH + perp_t * ARROW_WIDTH * 0.5,
            end - tangent * ARROW_LENGTH - perp_t * ARROW_WIDTH * 0.5,
        ],
        color,
        egui::Stroke::NONE,
    ));
}

/// Perpendicular of a normalized direction vector, rotated 90° — the same
/// "which side does this bow/offset toward" primitive `draw_edge` and
/// `draw_dashed_line` both need.
fn perp(dir: egui::Vec2) -> egui::Vec2 {
    egui::vec2(-dir.y, dir.x)
}

/// Whether a tag has zero evidence in *every* direction against every other
/// active tag — drives node visibility on the hypothesis grid (task 101):
/// distinguishes "no observation has ever touched this tag" (not drawn) from
/// a tag with at least some partial or confirmed evidence (drawn).
fn has_no_evidence(slot: TagSlot, tag_count: usize, knowledge: &MatrixKnowledge) -> bool {
    (0..tag_count).all(|oi| {
        if oi == slot.0 as usize {
            return true;
        }
        let other = TagSlot(oi as u8);
        knowledge.evidence(slot, other) == 0.0 && knowledge.evidence(other, slot) == 0.0
    })
}

/// Dash segment/gap length (task 102) — tuned by eye alongside `EDGE_BOW`:
/// short enough to read as "dashed, not solid" even at the shortest edges
/// this grid draws, even-ish so a dash never degenerates to a stub at
/// awkward edge lengths.
const DASH_LENGTH: f32 = 5.0;
const DASH_GAP: f32 = 4.0;

/// A dashed line (task 102, replacing task 028's lineless dot) marking a
/// pair with `evidence > 0.0` but not yet confirmed: "a relationship exists
/// here, details pending." Deliberately no arrowhead and uniform thickness
/// — sign and magnitude genuinely aren't known pre-confirmation, so there's
/// nothing for either to encode, and the missing arrowhead keeps a partial
/// edge from reading as a weaker confirmed one at a glance. Straight when
/// `curvature == 0.0`, otherwise flattened from the same bow curve
/// `draw_edge` uses for its bidirectional case, so a confirmed/partial pair
/// bows apart consistently regardless of which direction is which.
fn draw_dashed_line(painter: &egui::Painter, from: egui::Pos2, to: egui::Pos2, curvature: f32) {
    let dir = (to - from).normalized();
    let start = from + dir * NODE_RADIUS;
    let end = to - dir * NODE_RADIUS;

    let points = if curvature.abs() < f32::EPSILON {
        vec![start, end]
    } else {
        let control = start.lerp(end, 0.5) + perp(dir) * curvature;
        let bezier = egui::epaint::QuadraticBezierShape::from_points_stroke(
            [start, control, end],
            false,
            egui::Color32::TRANSPARENT,
            egui::Stroke::NONE,
        );
        bezier.flatten(Some(CURVE_FLATTEN_TOLERANCE))
    };
    draw_dashed_polyline(
        painter,
        &points,
        egui::Stroke::new(PARTIAL_EDGE_STROKE, PARTIAL_EVIDENCE_COLOR),
    );
}

/// Steps along a polyline (straight, 2 points, or a flattened curve) in
/// alternating `DASH_LENGTH`/`DASH_GAP` stretches — the shared primitive
/// behind `draw_dashed_line`, working the same way regardless of whether
/// the underlying path is straight or curved.
fn draw_dashed_polyline(painter: &egui::Painter, points: &[egui::Pos2], stroke: egui::Stroke) {
    let mut on = true;
    let mut budget = DASH_LENGTH;
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let seg_len = (b - a).length();
        if seg_len <= f32::EPSILON {
            continue;
        }
        let dir = (b - a) / seg_len;
        let mut traveled = 0.0;
        while traveled < seg_len {
            let step = budget.min(seg_len - traveled);
            if on {
                painter.line_segment([a + dir * traveled, a + dir * (traveled + step)], stroke);
            }
            traveled += step;
            budget -= step;
            if budget <= f32::EPSILON {
                on = !on;
                budget = if on { DASH_LENGTH } else { DASH_GAP };
            }
        }
    }
}

/// `evidence / threshold` as a rounded percentage (task 102) — how close an
/// unconfirmed pair is to crossing `MatrixKnowledge`'s confirmation
/// threshold. Exposes numbers `MatrixKnowledge` already tracks; doesn't
/// change evidence accumulation itself.
fn confidence_pct(knowledge: &MatrixKnowledge, exerter: TagSlot, receiver: TagSlot) -> u32 {
    (knowledge.evidence(exerter, receiver) / knowledge.threshold() * 100.0).round() as u32
}

/// Hover fallback for a node (acceptance criterion: the graph must not be
/// the *only* way to read this information): the tag's glyph, every
/// confirmed relationship it takes part in, and — without sign or magnitude
/// (task 028) — which pairs have partial, unconfirmed evidence.
fn node_tooltip_text(
    slot: TagSlot,
    tag: TagId,
    tags: &[TagId],
    world: &SimWorld,
    knowledge: &MatrixKnowledge,
) -> String {
    let mut lines = vec![text::node_tag_line(tag_glyph(tag))];
    for (oi, &other) in tags.iter().enumerate() {
        if oi == slot.0 as usize {
            continue;
        }
        let other_slot = TagSlot(oi as u8);
        if let Some(value) = knowledge.revealed_value(slot, other_slot, world) {
            lines.push(text::confirmed_relation_line(
                tag_glyph(tag),
                tag_glyph(other),
                value > 0,
                value.unsigned_abs() as i8,
            ));
        } else if knowledge.evidence(slot, other_slot) > 0.0 {
            lines.push(text::partial_relation_line(
                tag_glyph(tag),
                tag_glyph(other),
                confidence_pct(knowledge, slot, other_slot),
            ));
        }
        if let Some(value) = knowledge.revealed_value(other_slot, slot, world) {
            lines.push(text::confirmed_relation_line(
                tag_glyph(other),
                tag_glyph(tag),
                value > 0,
                value.unsigned_abs() as i8,
            ));
        } else if knowledge.evidence(other_slot, slot) > 0.0 {
            lines.push(text::partial_relation_line(
                tag_glyph(other),
                tag_glyph(tag),
                confidence_pct(knowledge, other_slot, slot),
            ));
        }
    }
    // Unreachable since task 101: a node with no evidence in either
    // direction never renders (see `hypothesis_grid`'s `visible` filter), so
    // this fallback can never actually be hovered. Left in rather than
    // removed — harmless, and still correct if a future caller ever invokes
    // `node_tooltip_text` for a non-visible tag.
    if lines.len() == 1 {
        lines.push(text::NO_OBSERVATIONS_YET.to_string());
    }
    lines.join("\n")
}

/// Lists the active tag pool and every species' readable genome fields
/// alongside its (still-opaque) tags. Phase 2's whole active pool is
/// visible from the seed selector already, so per-encounter tag discovery
/// isn't modeled here — that's a Phase 3 worldgen concern.
/// A readable band ("cold"/"temperate"/"hot") for a species' `temp_optimum`,
/// derived from `EnvironmentConfig`'s actual source/ambient bounds (not a
/// hardcoded cutoff, per CLAUDE.md's no-magic-numbers rule) so it stays
/// correct if those values are retuned. The raw `temp_optimum`/
/// `temp_tolerance` numbers stay in `species_catalog_line` alongside this
/// label — `Splice`'s `ShiftTempOptimum` math needs the precise value, this
/// is just an aid to read it at a glance.
fn temperature_label(
    temp_optimum: f32,
    env: &abiogenesis::config::EnvironmentConfig,
) -> &'static str {
    let low = env.ambient_temperature;
    let high = env.source_temperature;
    let band = (high - low) / 3.0;
    if temp_optimum <= low + band {
        "cold"
    } else if temp_optimum >= high - band {
        "hot"
    } else {
        "temperate"
    }
}

fn catalog_panel(
    ui: &mut egui::Ui,
    world: &SimWorld,
    config: &SimConfig,
    terrain_knowledge: &TerrainKnowledge,
) {
    ui.label(text::ACTIVE_TAGS_LABEL);
    ui.horizontal(|ui| {
        for &tag in &world.active_tags {
            ui.colored_label(tag_color(tag), format!("{TAG_GLYPH} {}", tag_glyph(tag)));
        }
    });

    ui.add_space(4.0);
    ui.label(text::METABOLISM_LEGEND_HEADING);
    let mut seen_metabolisms: Vec<Metabolism> = Vec::new();
    for species in &world.species {
        if !seen_metabolisms.contains(&species.metabolism) {
            seen_metabolisms.push(species.metabolism);
        }
    }
    for metabolism in seen_metabolisms {
        ui.horizontal(|ui| {
            ui.label(metabolism_glyph(metabolism));
            ui.weak(text::metabolism_legend_line(metabolism));
        });
    }

    ui.add_space(4.0);
    ui.label(text::SPECIES_HEADING);
    for (id, species) in world.species.iter().enumerate() {
        let species_id = SpeciesId(id as u8);
        let population = world
            .cells
            .iter()
            .filter(|c| c.organism.is_some_and(|o| o.species == species_id))
            .count();
        let origin_era = world
            .species_origin_era
            .get(id)
            .copied()
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.colored_label(species_color(species_id), TAG_GLYPH);
            ui.label(metabolism_glyph(species.metabolism));
            ui.vertical(|ui| {
                ui.label(text::species_catalog_line(
                    &species_label(world, species_id),
                    species.metabolism,
                    species.temp_optimum,
                    species.temp_tolerance,
                    temperature_label(species.temp_optimum, &config.environment),
                    species.repro_threshold,
                ));
                ui.weak(text::species_population_line(population, origin_era));
            });
            for &slot in &species.tags {
                let tag = world.active_tags[slot.0 as usize];
                ui.colored_label(tag_color(tag), format!("{TAG_GLYPH} {}", tag_glyph(tag)));
                conditional_tag_badge(ui, world, terrain_knowledge, slot, tag);
            }
        });
    }
}

/// Task 097's catalog badge: a terrain glyph plus an inducible/repressible
/// marker next to a conditional tag, shown only once `terrain_knowledge`
/// reports the `(slot, terrain)` fact confirmed — no leak of which terrain
/// or mode before that, the same discovery discipline `MatrixKnowledge`
/// already enforces for tag pairs. `tag`/`conditional.terrain`/`.mode` are
/// read straight off `world.conditional_tags` (task 096's per-world roll),
/// gated purely by the confirmation check, the same "reads through instead
/// of storing a copy" pattern `MatrixKnowledge::revealed_value` uses. A
/// non-conditional tag (`conditional_gate` returns `None`) renders nothing.
fn conditional_tag_badge(
    ui: &mut egui::Ui,
    world: &SimWorld,
    terrain_knowledge: &TerrainKnowledge,
    slot: TagSlot,
    tag: TagId,
) {
    let Some(conditional) = world.conditional_gate(tag) else {
        return;
    };
    if !terrain_knowledge.is_confirmed(slot, conditional.terrain) {
        return;
    }
    let marker = match conditional.mode {
        Mode::Inducible => "↑",
        Mode::Repressible => "↓",
    };
    ui.colored_label(
        tag_color(tag),
        format!("{}{marker}", terrain_glyph(conditional.terrain)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use abiogenesis::config::SimConfig;
    use abiogenesis::world::SpeciesId;

    #[test]
    fn temperature_label_splits_the_gradient_range_into_thirds() {
        let env = SimConfig::default().environment;
        // Default source model: ambient 0.25, source 0.85, so bands are
        // roughly [0.25, 0.45], (0.45, 0.65), [0.65, 0.85] — the upper-band
        // test points sit a hair inside their band, not exactly on the
        // `high - band`/`low + band` boundary, since `f32` division doesn't
        // reproduce that boundary bit-for-bit.
        assert_eq!(temperature_label(0.25, &env), "cold");
        assert_eq!(temperature_label(0.44, &env), "cold");
        assert_eq!(temperature_label(0.55, &env), "temperate");
        assert_eq!(temperature_label(0.66, &env), "hot");
        assert_eq!(temperature_label(0.85, &env), "hot");
    }

    fn app_for_record_events(mut world: SimWorld) -> App {
        // Dummy species covering every `SpeciesId` these tests reference
        // (up to 2) — `species_label` (task 095) indexes `world.species` by
        // id, so events naming a species need a real entry there.
        let config = SimConfig::default();
        for _ in 0..3 {
            world.push_species(abiogenesis::world::Species {
                name: "Test".to_string(),
                metabolism: abiogenesis::world::Metabolism::Photolithic,
                temp_optimum: 0.5,
                temp_tolerance: config.energy.default_temp_tolerance,
                repro_threshold: config.energy.repro_threshold,
                tags: Vec::new(),
            });
        }
        let mut app = App::new();
        app.insert_resource(world);
        app.init_resource::<ObservationLog>();
        app.init_resource::<PlayerPlacedCells>();
        app.add_message::<SpeciesExtinct>();
        app.add_message::<OrganismDied>();
        app.add_message::<ObjectiveAdvanced>();
        app.add_message::<TerrainRevealed>();
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
    fn objective_advanced_appends_a_log_entry() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.era = 5;
        let mut app = app_for_record_events(world);

        app.world_mut()
            .resource_mut::<Messages<ObjectiveAdvanced>>()
            .write(ObjectiveAdvanced {
                index: 1,
                objective: abiogenesis::objectives::Objective::TriggerBloom {
                    species: SpeciesId(0),
                    population_threshold: 8,
                },
            });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].era, 5);
        assert_eq!(log.entries[0].species, None);
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
                gain: 0.4,
                env_fit: 0.9,
                interaction_delta: 0.0,
                upkeep: 0.5,
                crowding_penalty: 0.15,
                predation_loss: 0.0,
                energy_before: 0.1,
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
        assert!(
            log.entries[0].text.contains("crowded"),
            "crowding_penalty (0.15) dominates gain's 0.1 shortfall here, \
             so the message should name it: {}",
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
                gain: 0.0,
                env_fit: 0.9,
                interaction_delta: 0.0,
                upkeep: 0.5,
                crowding_penalty: 0.0,
                predation_loss: 0.0,
                energy_before: 0.1,
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
                gain: 0.0,
                env_fit: 0.9,
                interaction_delta: 0.0,
                upkeep: 0.7,
                crowding_penalty: 0.0,
                predation_loss: 0.0,
                energy_before: 0.1,
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
            knowledge.record(TagSlot(0), TagSlot(1), 1.0);
        }
        assert!((knowledge.evidence(TagSlot(0), TagSlot(1)) - 3.0).abs() < 1e-6);
        assert!(knowledge.is_confirmed(TagSlot(0), TagSlot(1)));
    }

    #[test]
    fn record_reports_the_confirmation_transition_exactly_once() {
        let mut knowledge = MatrixKnowledge::new(2, THRESHOLD);
        assert!(!knowledge.record(TagSlot(0), TagSlot(1), 1.0));
        assert!(!knowledge.record(TagSlot(0), TagSlot(1), 1.0));
        assert!(knowledge.record(TagSlot(0), TagSlot(1), 1.0));
        // Already confirmed: further evidence keeps accumulating but no
        // longer reports a fresh transition.
        assert!(!knowledge.record(TagSlot(0), TagSlot(1), 1.0));
    }

    #[test]
    fn confounded_observations_need_four_times_as_many() {
        // n_confounders = 3 -> weight 0.25 each; 12 * 0.25 == threshold.
        let mut knowledge = MatrixKnowledge::new(2, THRESHOLD);
        for _ in 0..11 {
            knowledge.record(TagSlot(0), TagSlot(1), 0.25);
        }
        assert!(
            !knowledge.is_confirmed(TagSlot(0), TagSlot(1)),
            "11 * 0.25 = 2.75, just under threshold"
        );

        knowledge.record(TagSlot(0), TagSlot(1), 0.25);
        assert!(
            knowledge.is_confirmed(TagSlot(0), TagSlot(1)),
            "the 12th observation crosses the threshold"
        );
    }

    #[test]
    fn terrain_knowledge_record_reports_the_confirmation_transition_exactly_once() {
        // Task 097: mirrors `record_reports_the_confirmation_transition_exactly_once`
        // above, just keyed on `(TagSlot, TerrainKind)` instead of a tag pair.
        let mut knowledge = TerrainKnowledge::new(2, THRESHOLD);
        assert!(!knowledge.record(TagSlot(0), TerrainKind::Hill, 1.0));
        assert!(!knowledge.record(TagSlot(0), TerrainKind::Hill, 1.0));
        assert!(knowledge.record(TagSlot(0), TerrainKind::Hill, 1.0));
        // Already confirmed: further evidence keeps accumulating but no
        // longer reports a fresh transition.
        assert!(!knowledge.record(TagSlot(0), TerrainKind::Hill, 1.0));
        // A different terrain for the same tag is a distinct cell — no
        // cross-contamination between terrains.
        assert!(!knowledge.is_confirmed(TagSlot(0), TerrainKind::Plain));
    }

    #[test]
    fn accumulate_terrain_evidence_confirms_and_logs_once() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.active_tags = vec![TagId(0)];
        world.conditional_tags = vec![abiogenesis::world::ConditionalTag {
            tag: TagId(0),
            terrain: TerrainKind::Hill,
            mode: Mode::Inducible,
        }];
        let mut app = App::new();
        app.insert_resource(config.clone());
        app.insert_resource(TerrainKnowledge::new(
            world.active_tags.len(),
            config.notebook.confirmation_threshold,
        ));
        app.insert_resource(world);
        app.init_resource::<ObservationLog>();
        app.init_resource::<NotebookHasUnseenConfirmation>();
        app.add_message::<TerrainGateObserved>();
        app.add_systems(Update, accumulate_terrain_evidence);

        for _ in 0..(config.notebook.confirmation_threshold
            / config.notebook.observation_weight_numerator)
            .ceil() as u32
        {
            app.world_mut()
                .resource_mut::<Messages<TerrainGateObserved>>()
                .write(TerrainGateObserved {
                    tag: TagSlot(0),
                    terrain: TerrainKind::Hill,
                    passed: true,
                });
            app.update();
        }

        let knowledge = app.world().resource::<TerrainKnowledge>();
        assert!(knowledge.is_confirmed(TagSlot(0), TerrainKind::Hill));
        let log = app.world().resource::<ObservationLog>();
        assert_eq!(
            log.entries.len(),
            1,
            "exactly one log entry for the confirmation transition, not one per observation"
        );
    }

    #[test]
    fn accumulate_evidence_applies_the_confounder_weight() {
        let config = SimConfig::default();
        let world = SimWorld::new(42, &config);
        let mut app = App::new();
        app.insert_resource(config.clone());
        app.insert_resource(MatrixKnowledge::new(
            world.active_tags.len(),
            config.notebook.confirmation_threshold,
        ));
        app.insert_resource(world);
        app.init_resource::<ObservationLog>();
        app.init_resource::<NotebookHasUnseenConfirmation>();
        app.add_message::<AdjacencyObserved>();
        app.add_systems(Update, accumulate_evidence);

        app.world_mut()
            .resource_mut::<Messages<AdjacencyObserved>>()
            .write(AdjacencyObserved {
                receiver_species: SpeciesId(0),
                exerter_tag: TagSlot(0),
                receiver_tag: TagSlot(1),
                n_confounders: 3,
                cell: 0,
            });
        app.update();

        let knowledge = app.world().resource::<MatrixKnowledge>();
        assert!(
            (knowledge.evidence(TagSlot(0), TagSlot(1)) - 0.25).abs() < 1e-6,
            "expected numerator(1.0) / (1 + 3 confounders) = 0.25, got {}",
            knowledge.evidence(TagSlot(0), TagSlot(1))
        );
    }

    fn app_for_accumulate_evidence(config: SimConfig, world: SimWorld) -> App {
        let mut app = App::new();
        app.insert_resource(MatrixKnowledge::new(
            world.active_tags.len(),
            config.notebook.confirmation_threshold,
        ));
        app.insert_resource(config);
        app.insert_resource(world);
        app.init_resource::<ObservationLog>();
        app.init_resource::<NotebookHasUnseenConfirmation>();
        app.add_message::<AdjacencyObserved>();
        app.add_systems(Update, accumulate_evidence);
        app
    }

    #[test]
    fn an_unconfirmed_observation_logs_nothing_but_still_accumulates_evidence() {
        let config = SimConfig::default();
        let world = SimWorld::new(42, &config);
        let mut app = app_for_accumulate_evidence(config, world);

        app.world_mut()
            .resource_mut::<Messages<AdjacencyObserved>>()
            .write(AdjacencyObserved {
                receiver_species: SpeciesId(0),
                exerter_tag: TagSlot(0),
                receiver_tag: TagSlot(1),
                n_confounders: 0,
                cell: 0,
            });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(
            log.entries.len(),
            0,
            "task 100: a raw, non-confirming observation must not log a line"
        );
        let knowledge = app.world().resource::<MatrixKnowledge>();
        assert!(
            knowledge.evidence(TagSlot(0), TagSlot(1)) > 0.0,
            "evidence still accumulates even though nothing was logged"
        );
    }

    #[test]
    fn unconfirmed_pairs_do_not_reveal_a_value() {
        let config = SimConfig::default();
        let world = SimWorld::new(42, &config);
        let (exerter, receiver) = (TagSlot(0), TagSlot(1));
        let mut knowledge = MatrixKnowledge::new(world.active_tags.len(), THRESHOLD);

        assert_eq!(knowledge.revealed_value(exerter, receiver, &world), None);

        knowledge.record(exerter, receiver, THRESHOLD);
        assert_eq!(
            knowledge.revealed_value(exerter, receiver, &world),
            Some(world.matrix.get(exerter, receiver))
        );
    }

    fn app_for_tally_births() -> App {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        // Two dummy species (tests below reference `SpeciesId(0)`/`(1)`) —
        // `species_label` (task 095) indexes `world.species` by id, so it
        // needs real entries here, not just the id numbers.
        for _ in 0..2 {
            world.push_species(abiogenesis::world::Species {
                name: "Test".to_string(),
                metabolism: abiogenesis::world::Metabolism::Photolithic,
                temp_optimum: 0.5,
                temp_tolerance: config.energy.default_temp_tolerance,
                repro_threshold: config.energy.repro_threshold,
                tags: Vec::new(),
            });
        }
        let mut app = App::new();
        app.insert_resource(world);
        app.init_resource::<BirthTally>();
        app.init_resource::<ObservationLog>();
        app.add_message::<OrganismBorn>();
        app.add_message::<EraCompleted>();
        app.add_systems(Update, tally_births);
        app
    }

    #[test]
    fn births_accumulate_without_logging_until_the_era_completes() {
        let mut app = app_for_tally_births();

        app.world_mut()
            .resource_mut::<Messages<OrganismBorn>>()
            .write(OrganismBorn {
                species: SpeciesId(0),
            });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert!(
            log.entries.is_empty(),
            "no log entry until EraCompleted fires"
        );
    }

    #[test]
    fn multiple_births_for_one_species_tally_to_a_single_correct_count() {
        let mut app = app_for_tally_births();

        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<Messages<OrganismBorn>>()
                .write(OrganismBorn {
                    species: SpeciesId(0),
                });
        }
        app.world_mut()
            .resource_mut::<Messages<OrganismBorn>>()
            .write(OrganismBorn {
                species: SpeciesId(1),
            });
        app.world_mut()
            .resource_mut::<Messages<EraCompleted>>()
            .write(EraCompleted { era: 5 });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(log.entries.len(), 2, "one entry per species with a birth");
        let species_0 = log
            .entries
            .iter()
            .find(|e| e.species == Some(SpeciesId(0)))
            .expect("species 0 has an entry");
        assert!(
            species_0.text.contains("+3"),
            "expected the tallied count of 3, got: {}",
            species_0.text
        );
        let species_1 = log
            .entries
            .iter()
            .find(|e| e.species == Some(SpeciesId(1)))
            .expect("species 1 has an entry");
        assert!(species_1.text.contains("+1"));
    }

    #[test]
    fn the_tally_resets_after_logging_with_no_carry_over_into_the_next_era() {
        let mut app = app_for_tally_births();

        app.world_mut()
            .resource_mut::<Messages<OrganismBorn>>()
            .write(OrganismBorn {
                species: SpeciesId(0),
            });
        app.world_mut()
            .resource_mut::<Messages<EraCompleted>>()
            .write(EraCompleted { era: 1 });
        app.update();

        // A second era completes with no further births at all.
        app.world_mut()
            .resource_mut::<Messages<EraCompleted>>()
            .write(EraCompleted { era: 2 });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(
            log.entries.len(),
            1,
            "the second era must not re-log the first era's births"
        );
    }
}
