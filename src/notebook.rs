// The observation log (GDD §7, §11): a curated feed of salient events, built
// by consuming `sim`'s `Message`s (task 018), not by inspecting the grid
// (TECH_DESIGN.md §4). Read-only with respect to `SimWorld` (TECH_DESIGN.md
// §3.3) — this module never mutates simulation state.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use abiogenesis::config::SimConfig;
use abiogenesis::objectives::ObjectiveAdvanced;
use abiogenesis::sim::{AdjacencyObserved, OrganismDied, SpeciesExtinct};
use abiogenesis::state::GameState;
use abiogenesis::world::{SimWorld, SpeciesId, TagId, TagSlot};

use crate::render::{species_color, species_label};
use crate::text;

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
    pub evidence_quality: Option<EvidenceQuality>,
}

/// An `AdjacencyObserved` event's evidence quality (GDD §7: "an isolated
/// observation is worth more"), derived straight from `n_confounders` —
/// `Clean` when the observation had none, `Confounded` otherwise. `None` on
/// `LogEntry` for every other entry kind (deaths, extinctions, confirmations,
/// species-created), which carry no per-observation weight to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceQuality {
    Clean,
    Confounded,
}

impl EvidenceQuality {
    fn from_confounders(n_confounders: u32) -> Self {
        if n_confounders == 0 {
            EvidenceQuality::Clean
        } else {
            EvidenceQuality::Confounded
        }
    }
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
            .init_resource::<NotebookEverOpened>()
            .init_resource::<PlayerPlacedCells>()
            .init_resource::<EverSeeded>()
            .init_resource::<NotebookHasUnseenConfirmation>()
            .add_systems(
                Update,
                (toggle_notebook, record_events, accumulate_evidence)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                notebook_window.run_if(in_state(GameState::Playing)),
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
        log.entries.push(LogEntry {
            era: world.era,
            species: None,
            text: text::observation_message(from_glyph, to_glyph),
            evidence_quality: Some(EvidenceQuality::from_confounders(event.n_confounders)),
        });

        let weight =
            config.notebook.observation_weight_numerator / (1.0 + event.n_confounders as f32);
        let newly_confirmed = knowledge.record(event.exerter_tag, event.receiver_tag, weight);
        if newly_confirmed {
            let positive = world.matrix.get(event.exerter_tag, event.receiver_tag) > 0;
            log.entries.push(LogEntry {
                era: world.era,
                species: None,
                text: text::confirmation_message(from_glyph, to_glyph, positive),
                evidence_quality: None,
            });
            unseen.0 = true;
        }
    }
}

/// `tab`: opens/closes the notebook window, mirroring `input.rs`'s
/// `keys.just_pressed(...)` key-handling pattern. Opening also latches
/// `NotebookEverOpened` (task 053) and clears the confirmation badge (task
/// 054) — the player has now seen whatever the badge was pointing at.
fn toggle_notebook(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<NotebookWindowOpen>,
    mut ever_opened: ResMut<NotebookEverOpened>,
    mut unseen_confirmation: ResMut<NotebookHasUnseenConfirmation>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        open.0 = !open.0;
        if open.0 {
            ever_opened.0 = true;
            unseen_confirmation.0 = false;
        }
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
    mut objectives_advanced: MessageReader<ObjectiveAdvanced>,
    mut placed: ResMut<PlayerPlacedCells>,
    mut log: ResMut<ObservationLog>,
) {
    for event in extinctions.read() {
        log.entries.push(LogEntry {
            era: world.era,
            species: Some(event.species),
            text: text::extinction_message(&species_label(event.species)),
            evidence_quality: None,
        });
    }

    for event in objectives_advanced.read() {
        log.entries.push(LogEntry {
            era: world.era,
            species: None,
            text: text::objective_advanced_message(event.index),
            evidence_quality: None,
        });
    }

    for event in deaths.read() {
        if placed.0.remove(&event.cell) {
            let (x, y) = (event.cell % world.width, event.cell / world.width);
            log.entries.push(LogEntry {
                era: world.era,
                species: Some(event.species),
                text: text::player_organism_death_message(
                    &species_label(event.species),
                    x,
                    y,
                    event.gain,
                    event.interaction_delta,
                    event.upkeep,
                    event.crowding_penalty,
                    event.predation_loss,
                ),
                evidence_quality: None,
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

/// Marks a matrix-confirmation log line (task 054) in place of the
/// species-color swatch other entries get — a confirmation is about a tag
/// pair, not any one species, so there's no `species_color` to render.
const CONFIRMATION_GLYPH: &str = "★";

/// Marks a per-observation log line (task 061) in place of the species-color
/// swatch or the confirmation glyph — colored by `EvidenceQuality` so the
/// isolation principle (GDD §7) is visible at a glance, not just in prose.
const EVIDENCE_DOT_GLYPH: &str = "●";

fn evidence_quality_color(quality: EvidenceQuality) -> egui::Color32 {
    match quality {
        EvidenceQuality::Clean => EDGE_POSITIVE_COLOR,
        EvidenceQuality::Confounded => PARTIAL_EVIDENCE_COLOR,
    }
}

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
                            match (entry.species, entry.evidence_quality) {
                                (Some(species), _) => {
                                    ui.colored_label(species_color(species), TAG_GLYPH);
                                }
                                (None, Some(quality)) => {
                                    ui.colored_label(
                                        evidence_quality_color(quality),
                                        EVIDENCE_DOT_GLYPH,
                                    );
                                }
                                (None, None) => {
                                    ui.label(CONFIRMATION_GLYPH);
                                }
                            }
                            ui.label(text::log_entry_line(entry.era, &entry.text));
                        });
                    }
                });

            ui.separator();
            ui.heading(text::HEADING_HYPOTHESIS_GRID);
            hypothesis_grid(ui, &world, &knowledge);

            ui.separator();
            ui.heading(text::HEADING_CATALOG);
            catalog_panel(ui, &world, &config);
        });
    Ok(())
}

/// Node radius for the hypothesis graph (task 031), in points.
const NODE_RADIUS: f32 = 14.0;

/// Perpendicular offset applied to an edge so that `A → B` and `B → A`
/// (when both are confirmed) render as two parallel lines instead of
/// overlapping into one — the offset direction is derived from each edge's
/// own travel direction, so the pair separates symmetrically without
/// needing to special-case "is the reverse edge also present."
const EDGE_OFFSET: f32 = 4.0;

const ARROW_LENGTH: f32 = 8.0;
const ARROW_WIDTH: f32 = 6.0;

const EDGE_POSITIVE_COLOR: egui::Color32 = egui::Color32::from_rgb(96, 200, 120);
const EDGE_NEGATIVE_COLOR: egui::Color32 = egui::Color32::from_rgb(220, 96, 96);

/// Marker for a pair with `evidence > 0.0` but not yet confirmed (task 028):
/// a small neutral-gray dot, never a line or arrowhead — visually distinct
/// from a confirmed edge at a glance, so it can never be mistaken for one.
/// Carries no sign/magnitude, only "something has been observed here."
const PARTIAL_EVIDENCE_COLOR: egui::Color32 = egui::Color32::from_gray(130);
const PARTIAL_MARKER_RADIUS: f32 = 3.0;
/// Placed most of the way toward the receiver (rather than centered) so the
/// marker still reads as "evidence exists in *this* direction," the same
/// directionality the pre-031 spreadsheet table conveyed via row/column —
/// without a sign or an arrowhead, so it can't be confused for confirmation.
const PARTIAL_MARKER_T: f32 = 0.65;

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
/// Player-authored conjectures (GDD §5.9's `±?` state — marking a guess
/// before it's confirmed) aren't implemented: cut for this task, left as a
/// follow-up rather than a half-built annotation feature.
fn hypothesis_grid(ui: &mut egui::Ui, world: &SimWorld, knowledge: &MatrixKnowledge) {
    let tags = &world.active_tags;
    let desired_size = egui::vec2(ui.available_width().clamp(200.0, 320.0), 240.0);
    let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
    let rect = response.rect;
    let center = rect.center();
    let radius = (rect.width().min(rect.height()) / 2.0 - NODE_RADIUS - 6.0).max(10.0);

    let positions: Vec<egui::Pos2> = (0..tags.len())
        .map(|i| {
            let angle =
                i as f32 / tags.len() as f32 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            center + radius * egui::vec2(angle.cos(), angle.sin())
        })
        .collect();

    for ei in 0..tags.len() {
        for ri in 0..tags.len() {
            if ei == ri {
                continue;
            }
            let (exerter, receiver) = (TagSlot(ei as u8), TagSlot(ri as u8));
            if let Some(value) = knowledge.revealed_value(exerter, receiver, world) {
                let color = if value > 0 {
                    EDGE_POSITIVE_COLOR
                } else {
                    EDGE_NEGATIVE_COLOR
                };
                draw_edge(&painter, positions[ei], positions[ri], color, value);
            } else if knowledge.evidence(exerter, receiver) > 0.0 {
                draw_partial_marker(&painter, positions[ei], positions[ri]);
            }
        }
    }

    for (i, &tag) in tags.iter().enumerate() {
        let slot = TagSlot(i as u8);
        let pos = positions[i];
        if has_no_evidence(slot, tags.len(), knowledge) {
            draw_dashed_ring(&painter, pos, NODE_RADIUS + DASHED_RING_MARGIN);
        }
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

/// Stroke width for a confirmed edge of magnitude 1 vs. magnitude 2 (task
/// 061) — tuned by eye, pure presentation, no config entry warranted.
const EDGE_STROKE_WEAK: f32 = 1.5;
const EDGE_STROKE_STRONG: f32 = 3.0;

/// Draws one directed edge as a line stopping short of both node
/// boundaries, capped with a small triangular arrowhead at the receiver
/// end. Offset perpendicular to its own travel direction by `EDGE_OFFSET`
/// so a confirmed `A → B` and a confirmed `B → A` render as two distinct
/// parallel lines rather than overlapping. `value` is the confirmed matrix
/// entry (`±1` or `±2`): its magnitude sets the stroke width, and a
/// magnitude-2 edge additionally gets a signed numeric label near its
/// midpoint — magnitude-1 edges stay unlabeled to avoid clutter.
fn draw_edge(
    painter: &egui::Painter,
    from: egui::Pos2,
    to: egui::Pos2,
    color: egui::Color32,
    value: i8,
) {
    let dir = (to - from).normalized();
    let perp = egui::vec2(-dir.y, dir.x);
    let offset = perp * EDGE_OFFSET;

    let start = from + offset + dir * NODE_RADIUS;
    let tip = to + offset - dir * NODE_RADIUS;
    let base = tip - dir * ARROW_LENGTH;

    let magnitude = value.unsigned_abs();
    let width = if magnitude >= 2 {
        EDGE_STROKE_STRONG
    } else {
        EDGE_STROKE_WEAK
    };
    painter.line_segment([start, base], egui::Stroke::new(width, color));
    painter.add(egui::Shape::convex_polygon(
        vec![
            tip,
            base + perp * ARROW_WIDTH * 0.5,
            base - perp * ARROW_WIDTH * 0.5,
        ],
        color,
        egui::Stroke::NONE,
    ));

    if magnitude >= 2 {
        let mid = start.lerp(base, 0.5) + perp * 6.0;
        painter.text(
            mid,
            egui::Align2::CENTER_CENTER,
            format!("{value:+}"),
            egui::FontId::proportional(10.0),
            color,
        );
    }
}

/// Whether a tag has zero evidence in *every* direction against every other
/// active tag — drives the dashed-ring marker (task 061) distinguishing "no
/// observation has ever touched this tag" from a tag with at least some
/// partial or confirmed evidence.
fn has_no_evidence(slot: TagSlot, tag_count: usize, knowledge: &MatrixKnowledge) -> bool {
    (0..tag_count).all(|oi| {
        if oi == slot.0 as usize {
            return true;
        }
        let other = TagSlot(oi as u8);
        knowledge.evidence(slot, other) == 0.0 && knowledge.evidence(other, slot) == 0.0
    })
}

/// Margin between a zero-evidence node's dashed ring and its filled circle.
const DASHED_RING_MARGIN: f32 = 3.0;
/// Number of dash segments around the ring's circumference.
const DASHED_RING_SEGMENTS: usize = 16;
const DASHED_RING_COLOR: egui::Color32 = egui::Color32::from_gray(160);

/// Approximates a dashed circle outline (egui's `Painter` has no built-in
/// dashed-circle primitive) as alternating short arcs, each drawn as a small
/// straight line segment — accurate enough at `NODE_RADIUS` scale, cheaper
/// than a true arc tessellation.
fn draw_dashed_ring(painter: &egui::Painter, center: egui::Pos2, radius: f32) {
    for i in 0..DASHED_RING_SEGMENTS {
        if i % 2 != 0 {
            continue;
        }
        let a0 = i as f32 / DASHED_RING_SEGMENTS as f32 * std::f32::consts::TAU;
        let a1 = (i as f32 + 0.6) / DASHED_RING_SEGMENTS as f32 * std::f32::consts::TAU;
        let p0 = center + radius * egui::vec2(a0.cos(), a0.sin());
        let p1 = center + radius * egui::vec2(a1.cos(), a1.sin());
        painter.line_segment([p0, p1], egui::Stroke::new(1.5, DASHED_RING_COLOR));
    }
}

/// A small, dim, lineless dot (task 028) marking a pair with `evidence >
/// 0.0` but not yet confirmed — deliberately nothing like `draw_edge`'s
/// line-plus-arrowhead, so it reads as a different kind of thing at a
/// glance rather than a weaker version of a confirmed edge.
fn draw_partial_marker(painter: &egui::Painter, from: egui::Pos2, to: egui::Pos2) {
    let point = from.lerp(to, PARTIAL_MARKER_T);
    painter.circle_filled(point, PARTIAL_MARKER_RADIUS, PARTIAL_EVIDENCE_COLOR);
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
            ));
        } else if knowledge.evidence(slot, other_slot) > 0.0 {
            lines.push(text::partial_relation_line(
                tag_glyph(tag),
                tag_glyph(other),
            ));
        }
        if let Some(value) = knowledge.revealed_value(other_slot, slot, world) {
            lines.push(text::confirmed_relation_line(
                tag_glyph(other),
                tag_glyph(tag),
                value > 0,
            ));
        } else if knowledge.evidence(other_slot, slot) > 0.0 {
            lines.push(text::partial_relation_line(
                tag_glyph(other),
                tag_glyph(tag),
            ));
        }
    }
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
/// derived from `EnvironmentConfig`'s actual gradient bounds (not a hardcoded
/// cutoff, per CLAUDE.md's no-magic-numbers rule) so it stays correct if the
/// gradient range is retuned. The raw `temp_optimum`/`temp_tolerance` numbers
/// stay in `species_catalog_line` alongside this label — `Splice`'s
/// `ShiftTempOptimum` math needs the precise value, this is just an aid to
/// read it at a glance.
fn temperature_label(
    temp_optimum: f32,
    env: &abiogenesis::config::EnvironmentConfig,
) -> &'static str {
    let low = env.temperature_gradient_left;
    let high = env.temperature_gradient_right;
    let band = (high - low) / 3.0;
    if temp_optimum <= low + band {
        "cold"
    } else if temp_optimum >= high - band {
        "hot"
    } else {
        "temperate"
    }
}

fn catalog_panel(ui: &mut egui::Ui, world: &SimWorld, config: &SimConfig) {
    ui.label(text::ACTIVE_TAGS_LABEL);
    ui.horizontal(|ui| {
        for &tag in &world.active_tags {
            ui.colored_label(tag_color(tag), format!("{TAG_GLYPH} {}", tag_glyph(tag)));
        }
    });

    ui.add_space(4.0);
    ui.label(text::SPECIES_HEADING);
    for (id, species) in world.species.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.colored_label(species_color(SpeciesId(id as u8)), TAG_GLYPH);
            ui.label(text::species_catalog_line(
                &species_label(SpeciesId(id as u8)),
                species.metabolism,
                species.temp_optimum,
                species.temp_tolerance,
                temperature_label(species.temp_optimum, &config.environment),
            ));
            for &slot in &species.tags {
                let tag = world.active_tags[slot.0 as usize];
                ui.colored_label(tag_color(tag), format!("{TAG_GLYPH} {}", tag_glyph(tag)));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abiogenesis::config::SimConfig;
    use abiogenesis::world::SpeciesId;

    #[test]
    fn temperature_label_splits_the_gradient_range_into_thirds() {
        let env = SimConfig::default().environment;
        // Default gradient: left 0.2, right 0.8, so bands are [0.2, 0.4],
        // (0.4, 0.6), [0.6, 0.8].
        assert_eq!(temperature_label(0.2, &env), "cold");
        assert_eq!(temperature_label(0.4, &env), "cold");
        assert_eq!(temperature_label(0.5, &env), "temperate");
        assert_eq!(temperature_label(0.6, &env), "hot");
        assert_eq!(temperature_label(0.8, &env), "hot");
    }

    fn app_for_record_events(world: SimWorld) -> App {
        let mut app = App::new();
        app.insert_resource(world);
        app.init_resource::<ObservationLog>();
        app.init_resource::<PlayerPlacedCells>();
        app.add_message::<SpeciesExtinct>();
        app.add_message::<OrganismDied>();
        app.add_message::<ObjectiveAdvanced>();
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
            log.entries[0].text.contains("gain") && log.entries[0].text.contains("upkeep"),
            "the death message should carry the energy breakdown: {}",
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
    fn a_clean_observation_logs_an_evidence_quality_clean_entry() {
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
            });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(log.entries.len(), 1);
        assert_eq!(
            log.entries[0].evidence_quality,
            Some(EvidenceQuality::Clean)
        );
        assert_eq!(log.entries[0].species, None);
    }

    #[test]
    fn a_confounded_observation_logs_an_evidence_quality_confounded_entry() {
        let config = SimConfig::default();
        let world = SimWorld::new(42, &config);
        let mut app = app_for_accumulate_evidence(config, world);

        app.world_mut()
            .resource_mut::<Messages<AdjacencyObserved>>()
            .write(AdjacencyObserved {
                receiver_species: SpeciesId(0),
                exerter_tag: TagSlot(0),
                receiver_tag: TagSlot(1),
                n_confounders: 2,
            });
        app.update();

        let log = app.world().resource::<ObservationLog>();
        assert_eq!(log.entries.len(), 1);
        assert_eq!(
            log.entries[0].evidence_quality,
            Some(EvidenceQuality::Confounded)
        );
    }

    #[test]
    fn non_adjacency_log_entries_carry_no_evidence_quality() {
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
        assert_eq!(log.entries[0].evidence_quality, None);
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
}
