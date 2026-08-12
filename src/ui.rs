use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera, ClearColorConfig, Viewport};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{
    egui, EguiContexts, EguiGlobalSettings, EguiPrimaryContextPass, PrimaryEguiContext,
};

use crate::notebook::{
    tag_glyph, EverSeeded, NotebookEverOpened, NotebookHasUnseenConfirmation, NotebookWindowOpen,
};
use crate::render::{metabolism_glyph, species_color, species_label, GridCamera, MapViewMode};
use crate::text;
use abiogenesis::config::SimConfig;
use abiogenesis::objectives::{
    is_grace_active, CurrentObjective, GraceProgress, Objective, ObjectiveProgress,
};
use abiogenesis::sim::{ActionBudget, EraCompleted, OrganismDied};
use abiogenesis::state::{EraState, GameState};
use abiogenesis::world::{SimWorld, SpeciesId, TagSlot};

/// Task 055's guided first-isolation hint: the message to show (isolated vs.
/// clustered first placement), the `SimWorld::tick` it was set at, and how
/// long it stays up — the tick counter the HUD's own "Era X · tick Y"
/// readout already exposes, not a wall-clock timer. Written once, by
/// `input.rs`'s `seed_organism_on_click` at the player's first-ever
/// placement of their first-ever run (gated on
/// `MetaProgress::seen_isolation_hint`); read here.
///
/// `duration_ticks` (task 092) is computed once at set-time from
/// `worldgen::era_ticks_for` for whatever era is current *then* — not a
/// fixed constant, and not re-derived on later frames even if the era
/// changes underneath it before the hint dismisses. Before task 092 this was
/// a fixed `30`, which read as "about one era" back when every era was `25`
/// ticks; task 082's shortened onboarding eras (`8` ticks each) made that
/// stale, since the hint kept outliving the era it was shown in and no
/// longer read as anchored to anything. Pinning the duration to the
/// *shown-in* era (rather than re-deriving it against whatever era is
/// current each frame) is the deliberate choice here: a hint that started
/// during an 8-tick onboarding era and is still up when era 4's 25-tick era
/// begins should still resolve on its own short original schedule, not
/// suddenly gain 17 more ticks of life because the era around it got
/// longer.
#[derive(Resource, Default)]
pub struct IsolationHint {
    pub text: Option<&'static str>,
    pub shown_at_tick: u64,
    pub duration_ticks: u64,
}

/// Momentary "the player clicked a HUD button for this" flags (task 094),
/// read and cleared by the same systems that already act on the matching
/// keyboard shortcut (`input.rs::start_era`/`single_tick`,
/// `notebook.rs::toggle_notebook`) — one action, two ways to trigger it,
/// never two independent implementations that could drift apart. Written by
/// `hud_panel`'s new time-control button row, in `EguiPrimaryContextPass`
/// (`PostUpdate`); read by `Update`-scheduled systems, so a button click is
/// consumed on the *next* frame's `Update` pass, not the same frame it was
/// clicked in — an imperceptible one-frame delay, not a functional gap.
#[derive(Resource, Default)]
pub struct HudControlIntents {
    pub advance_era: bool,
    pub advance_tick: bool,
    pub toggle_notebook: bool,
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
/// `render::CELL_SIZE` for the same rationale). Raised from `260.0`
/// (playtest finding, 2026-08-07, tasks 057/058) to `300.0`, then to
/// `340.0` (task 064): switching the panel to a monospace font made every
/// line measurably wider than the same string in the proportional font this
/// budget was tuned against — the Biosphere row (species label + energy +
/// trend glyph) was clipping its trailing glyph inside the vertical
/// `ScrollArea`, which also reserves its own scrollbar width.
const HUD_WIDTH: f32 = 340.0;

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

/// How many Biosphere rows (task 064) stay visible before the list scrolls
/// internally, matching the redesign mockup's "~4-5 visible rows" call.
///
/// `BIOSPHERE_VISIBLE_ROWS * console_row_height(ui)` must stay above 64.0:
/// `egui::ScrollArea` silently floors its scrolled axis at
/// `min_scrolled_size` (default `64.0`, egui 0.35 `scroll_area.rs`) — ask
/// for a smaller `max_height` and the area renders at 64.0 regardless,
/// which reads as "the cap doesn't work" instead of an early scroll trigger.
/// Confirmed by instrumenting `ui.clip_rect()` at runtime: a `max_height`
/// of `20.0` or `40.0` both measured an actual clip height of exactly
/// `64.0`. The row height below is measured from the panel's own style
/// rather than hardcoded (measured at `~18.1pt`, `max_height ~90.6pt` —
/// safely above the floor) so the cap and the `SCROLL_FOR_MORE` threshold
/// stay consistent with each other if the font or spacing ever changes.
const BIOSPHERE_VISIBLE_ROWS: usize = 5;

/// How many Species rows (task 065) stay visible before the list scrolls
/// internally — same fixed-height/internal-scroll pattern as
/// `BIOSPHERE_VISIBLE_ROWS`, adopted after the horizontal scrollable chip
/// strip this replaced turned out less discoverable in practice (no visible
/// scrollbar, needed a dedicated `›` overflow cue) than the vertical list
/// pattern already used for Biosphere.
const SPECIES_VISIBLE_ROWS: usize = 5;

/// The height, in points, of one list row (glyph + label, optionally a
/// trailing glyph) in the Biosphere or Species sections, measured from this
/// panel's own text style and spacing rather than guessed — see
/// `BIOSPHERE_VISIBLE_ROWS`'s doc comment for why a hardcoded value is
/// unsafe here.
fn console_row_height(ui: &egui::Ui) -> f32 {
    ui.text_style_height(&egui::TextStyle::Body) + ui.spacing().item_spacing.y
}

/// Past this many required eras, an objective's progress indicator falls
/// back from a dot row to a compact "X / Y eras" readout (task 064) — a
/// row of dots sized to an unbounded `eras_required` would either overflow
/// `HUD_WIDTH` or shrink past legibility. Chosen so a dot row still fits
/// comfortably within the panel width at `DOT_SIZE`/`DOT_GAP`.
const ERA_PROGRESS_DOT_CAP: u32 = 8;

/// Color for the objective's narrative-quoted line (task 064 §5) — a warm,
/// slightly desaturated off-white distinct from the panel's neutral-gray
/// monospace body text, so the one deliberate font/style break also reads
/// as visually distinct in color, not just in shape.
const OBJECTIVE_NARRATIVE_COLOR: egui::Color32 = egui::Color32::from_rgb(180, 178, 169);

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
            .init_resource::<PopulationTrends>()
            .init_resource::<DeathCauseTally>()
            .init_resource::<HudControlIntents>()
            .add_systems(Startup, spawn_hud_camera)
            .add_systems(
                Update,
                (
                    reserve_hud_viewport,
                    update_population_trends,
                    tally_death_causes,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(EguiPrimaryContextPass, configure_fonts)
            .add_systems(
                EguiPrimaryContextPass,
                (hud_panel, viewport_hint).run_if(in_state(GameState::Playing)),
            );
    }
}

/// Registers `DEJAVU_SANS` as a lowest-priority fallback for both the
/// proportional and monospace font families: egui's own default font for
/// each family is tried first for every glyph, DejaVu Sans only fills the
/// gaps (Greek letters, `●`, and — since task 064's HUD console adopted
/// monospace panel-wide — `▲`/`▼`/`▬` if the built-in monospace family turns
/// out not to cover them). The egui context doesn't exist until the first
/// `EguiPrimaryContextPass` run (it's attached to the camera spawned in
/// `spawn_hud_camera`), so this can't run at `Startup`; `done` makes it a
/// one-shot within that schedule.
fn configure_fonts(mut contexts: EguiContexts, mut done: Local<bool>) -> Result {
    if *done {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    ctx.add_font(egui::epaint::text::FontInsert::new(
        "DejaVuSans",
        egui::FontData::from_static(DEJAVU_SANS),
        vec![
            egui::epaint::text::InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Lowest,
            },
            egui::epaint::text::InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Lowest,
            },
        ],
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
/// The Biosphere row's two per-era readouts, bundled into one `SystemParam`
/// (task 105, mirrors `run_flow.rs`'s `WorldResetParams`/`objectives.rs`'s
/// `ObjectiveOutcomeParams`): `hud_panel` already sat at Bevy's ~16-parameter
/// system ceiling before `DeathCauseTally` joined `PopulationTrends` as a
/// second trend-glyph-adjacent readout, so the two are folded into one
/// parameter here instead of pushing the function over it.
#[derive(SystemParam)]
pub struct BiosphereReadouts<'w> {
    pub trends: Res<'w, PopulationTrends>,
    pub death_causes: Res<'w, DeathCauseTally>,
}

#[allow(clippy::too_many_arguments)]
/// `pub(crate)` (task 091) so `notebook.rs` can order its stray-focus-clear
/// system `.after(hud_panel)` — the sidebar action buttons this draws are
/// what egui's own `Tab` keyboard-navigation grabs focus onto, so the fix
/// must run after they've had their chance to claim it this frame.
pub(crate) fn hud_panel(
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
    grace: Res<GraceProgress>,
    unseen_confirmation: Res<NotebookHasUnseenConfirmation>,
    biosphere: BiosphereReadouts,
    mode: Res<MapViewMode>,
    mut intents: ResMut<HudControlIntents>,
    notebook_open: Res<NotebookWindowOpen>,
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
            // One continuous "lab console" panel (task 064): monospace
            // panel-wide, scoped to this `Ui` and its children only — egui's
            // per-`TextStyle` font map is copy-on-write, so this never
            // touches the notebook window, menu screens, or the grid itself
            // (pillar-3 scoping the redesign doc calls for explicitly).
            // Family only, not size, so headings/body/small text keep their
            // existing relative proportions.
            for font_id in ui.style_mut().text_styles.values_mut() {
                font_id.family = egui::FontFamily::Monospace;
            }

            ui.heading(text::HEADING_TITLE);
            ui.label(text::era_tick_line(world.era, world.tick));
            ui.label(text::state_line(era_state.get()));
            if is_grace_active(world.era, config.time.grace_eras, &grace) {
                ui.weak(text::GRACE_PERIOD_LINE);
            }
            time_control_row(
                ui,
                &mut intents,
                *era_state.get() == EraState::Advancing,
                notebook_open.0,
            );

            hairline(ui);
            ui.strong(text::HEADING_ACTION);
            action_icon_row(ui, &mut selected_action, &config, *mode);

            let total = config.time.point_budget_per_era;
            dot_row(ui, budget.points_remaining, total, DotShape::Tick)
                .on_hover_text(text::BUDGET_HOVER);
            ui.weak(text::budget_bar_text(budget.points_remaining, total));

            if selected_action.0 == ActionMode::Splice {
                ui.indent("splice_panel", |ui| {
                    splice_panel(ui, &world, &mut splice_draft);
                });
            }

            hairline(ui);
            ui.strong(text::HEADING_POPULATION);
            if stats.is_empty() {
                ui.weak(text::NO_POPULATION);
            }
            egui::ScrollArea::vertical()
                .id_salt("biosphere_list")
                .max_height(BIOSPHERE_VISIBLE_ROWS as f32 * console_row_height(ui))
                .show(ui, |ui| {
                    for (species, population, avg_energy) in &stats {
                        ui.horizontal(|ui| {
                            ui.colored_label(species_color(*species), SPECIES_GLYPH);
                            ui.label(text::population_line(
                                &species_label(&world, *species),
                                *population,
                                *avg_energy,
                            ));
                            let trend = biosphere.trends.trend_for(*species);
                            ui.colored_label(trend_color(trend), trend_glyph(trend));
                            if let Some(cause) = biosphere.death_causes.dominant_for(*species) {
                                let metabolism = world.species[species.0 as usize].metabolism;
                                ui.weak(text::death_cause_short_label(cause, metabolism));
                            }
                        });
                    }
                });
            if stats.len() > BIOSPHERE_VISIBLE_ROWS {
                ui.weak(text::SCROLL_FOR_MORE);
            }

            hairline(ui);
            ui.strong(text::HEADING_SEED_PALETTE)
                .on_hover_text(text::SEED_PALETTE_HOVER);
            egui::ScrollArea::vertical()
                .id_salt("species_list")
                .max_height(SPECIES_VISIBLE_ROWS as f32 * console_row_height(ui))
                .show(ui, |ui| {
                    for i in 0..world.species.len() as u8 {
                        // Wild populations (task 098) aren't a player
                        // choice — they already exist, elsewhere on the
                        // grid — so they're excluded from the seed palette.
                        if world.is_wild(SpeciesId(i)) {
                            continue;
                        }
                        species_row(ui, SpeciesId(i), &world, &mut selected.0);
                    }
                });
            if world.species.len() > SPECIES_VISIBLE_ROWS {
                ui.weak(text::SCROLL_FOR_MORE);
            }

            hairline(ui);
            ui.strong(text::HEADING_OBJECTIVE);
            objective_panel(
                ui,
                &world,
                objective.current(),
                objective.index,
                objective.total(),
                &objective_progress,
                config.time.era_ticks,
            );

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.weak(text::KEYBOARD_HINT_PRIMARY);
                ui.weak(text::KEYBOARD_HINT_SECONDARY);
                ui.weak(text::seed_line(world.seed));
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

/// Whether task 055's guided first-isolation hint, shown at `shown_at_tick`
/// for `duration_ticks` (task 092: derived from the era it was shown in, see
/// `IsolationHint`'s own doc comment), is still within its display window —
/// pure so the self-dismiss timing is unit-testable without a `World`.
fn isolation_hint_active(shown_at_tick: u64, duration_ticks: u64, current_tick: u64) -> bool {
    current_tick.saturating_sub(shown_at_tick) < duration_ticks
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
/// once its `duration_ticks` have passed, after which the task-053 flow
/// (e.g. "open the notebook") resumes underneath it.
fn viewport_hint(
    mut contexts: EguiContexts,
    ever_seeded: Res<EverSeeded>,
    notebook_ever_opened: Res<NotebookEverOpened>,
    mut isolation_hint: ResMut<IsolationHint>,
    world: Res<SimWorld>,
) -> Result {
    if isolation_hint.text.is_some()
        && !isolation_hint_active(
            isolation_hint.shown_at_tick,
            isolation_hint.duration_ticks,
            world.tick,
        )
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

/// Current-objective panel (task 043, GDD §11; restyled task 064 per the
/// sidebar redesign's §5 "narrative accent"): the objective's sentence as an
/// italicized, quoted line in the panel's one deliberate non-monospace
/// accent, followed by a discrete progress indicator instead of a
/// continuous bar. Formats the concrete numbers/labels here (species name,
/// era counts) and hands them to `text.rs`'s parametrized templates, since
/// `text.rs` never reaches into `Objective`/`SimWorld`-derived data on its
/// own (task 034's constraint).
fn objective_panel(
    ui: &mut egui::Ui,
    world: &SimWorld,
    objective: Option<&Objective>,
    index: usize,
    total: usize,
    progress: &ObjectiveProgress,
    era_ticks: u32,
) {
    let Some(objective) = objective else {
        ui.weak(text::NO_OBJECTIVE);
        return;
    };

    if total > 1 {
        ui.weak(text::objective_sequence_position(index, total));
    }

    let description = match *objective {
        Objective::Coexistence { min_species, .. } => text::coexistence_objective_line(min_species),
        Objective::SurviveIn { species, zone, .. } => {
            text::survive_in_objective_line(&species_label(world, species), text::zone_label(zone))
        }
        Objective::TriggerBloom {
            species,
            population_threshold,
        } => {
            text::trigger_bloom_objective_line(&species_label(world, species), population_threshold)
        }
    };
    // The redesign's one deliberate break from the panel-wide monospace
    // style (task 064 §5, "da usare con parsimonia" — used sparingly,
    // nowhere else in the panel): switched back to the game's normal
    // proportional font and italicized, so it reads as a narrative quote
    // rather than console output, without needing to bundle a dedicated
    // serif font asset for a single line (`DejaVuSans` stays a glyph-gap
    // fallback, not a stylistic choice, so isn't reused here).
    ui.label(
        egui::RichText::new(text::narrative_quote(&description))
            .italics()
            .family(egui::FontFamily::Proportional)
            .color(OBJECTIVE_NARRATIVE_COLOR),
    );

    if progress.satisfied {
        ui.weak(text::OBJECTIVE_CLEARED);
        return;
    }

    match *objective {
        Objective::Coexistence { ticks, .. } | Objective::SurviveIn { ticks, .. } => {
            let (eras_held, eras_required) =
                eras_progress(progress.consecutive_ticks, ticks, era_ticks);
            match era_progress_display(eras_required) {
                EraProgressDisplay::Dots => {
                    dot_row(ui, eras_held, eras_required, DotShape::Circle);
                }
                EraProgressDisplay::Numeric => {
                    ui.weak(text::sustained_progress_bar_text(eras_held, eras_required));
                }
            }
        }
        // A bloom is a single triggering event, not a sustained count (see
        // `Objective::TriggerBloom`'s own doc comment) — a state label is
        // the whole story, no indicator of any shape needed.
        Objective::TriggerBloom { .. } => {
            ui.weak(text::BLOOM_NOT_TRIGGERED);
        }
    }
}

/// Which shape an objective's era-progress indicator takes, given how many
/// eras it requires — the pure decision behind `objective_panel`'s dots-vs-
/// text branch, split out so the `ERA_PROGRESS_DOT_CAP` boundary is
/// unit-testable without an `egui::Ui`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EraProgressDisplay {
    Dots,
    Numeric,
}

/// Past `ERA_PROGRESS_DOT_CAP`, a row of dots would either grow wider than
/// `HUD_WIDTH` or shrink unreadably small — a compact "X / Y eras" readout
/// degrades gracefully instead.
fn era_progress_display(eras_required: u32) -> EraProgressDisplay {
    if eras_required <= ERA_PROGRESS_DOT_CAP {
        EraProgressDisplay::Dots
    } else {
        EraProgressDisplay::Numeric
    }
}

/// Converts a sustained objective's tick counts to whole eras for display
/// (task 049): `eras_held` floors (an era in progress doesn't count until
/// complete), `eras_required` ceils (a requirement of, say, 70 ticks with a
/// 25-tick era still genuinely needs 3 full eras, not 2) — chosen so the
/// displayed count never claims "done" before `progress.satisfied` actually
/// flips.
fn eras_progress(consecutive_ticks: u32, required_ticks: u32, era_ticks: u32) -> (u32, u32) {
    if era_ticks == 0 {
        return (0, 0);
    }
    (
        consecutive_ticks / era_ticks,
        required_ticks.div_ceil(era_ticks),
    )
}

/// Low-contrast divider between HUD sections (task 064's redesign,
/// replacing `group_frame`'s bordered boxes): a thin painted line rather
/// than `egui::Separator`, so its color is independent of `Visuals`'
/// widget-stroke styling and stays a deliberately faint hairline instead of
/// picking up whatever contrast egui's default separator uses elsewhere.
const HAIRLINE_COLOR: egui::Color32 = egui::Color32::from_rgb(35, 38, 46);

fn hairline(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    let rect = ui.available_rect_before_wrap();
    let y = ui.cursor().top();
    ui.painter()
        .hline(rect.x_range(), y, egui::Stroke::new(0.5, HAIRLINE_COLOR));
    ui.add_space(6.0);
}

/// Which shape `dot_row` paints per slot — `Tick` for the action budget
/// (matching the redesign mockup's rounded-rect ticks), `Circle` for
/// objective era-progress (matching its own mockup's hollow/filled dots).
/// Two call sites, two mockup-specified shapes, kept as one function rather
/// than two near-duplicates.
#[derive(Clone, Copy)]
enum DotShape {
    Tick,
    Circle,
}

/// Color for a filled slot in `dot_row` — the same green `trend_color` uses
/// for `Rising`, reused here as this panel's one "available/progressing"
/// accent rather than inventing a second green.
const DOT_FILLED_COLOR: egui::Color32 = egui::Color32::from_rgb(96, 200, 120);
const DOT_EMPTY_COLOR: egui::Color32 = egui::Color32::from_gray(60);
const DOT_SIZE: f32 = 8.0;
const DOT_GAP: f32 = 5.0;

/// Renders `total` discrete slots, the first `filled` of them filled — the
/// task 064 replacement for continuous `egui::ProgressBar`s on small,
/// countable resources (action budget, era progress) that a percentage bar
/// misrepresents as a continuous metric. `total == 0` (a zero-budget config,
/// or an objective needing zero eras) draws nothing rather than a
/// zero-width allocation.
fn dot_row(ui: &mut egui::Ui, filled: u32, total: u32, shape: DotShape) -> egui::Response {
    let filled = filled.min(total);
    let width = if total == 0 {
        0.0
    } else {
        total as f32 * (DOT_SIZE + DOT_GAP) - DOT_GAP
    };
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width.max(0.0), DOT_SIZE), egui::Sense::hover());
    let painter = ui.painter();
    for i in 0..total {
        let is_filled = i < filled;
        let x = rect.left() + i as f32 * (DOT_SIZE + DOT_GAP);
        match shape {
            DotShape::Tick => {
                let tick_rect = egui::Rect::from_min_size(
                    egui::pos2(x, rect.center().y - 1.5),
                    egui::vec2(DOT_SIZE, 3.0),
                );
                let color = if is_filled {
                    DOT_FILLED_COLOR
                } else {
                    DOT_EMPTY_COLOR
                };
                painter.rect_filled(tick_rect, 1.5, color);
            }
            DotShape::Circle => {
                let center = egui::pos2(x + DOT_SIZE / 2.0, rect.center().y);
                let radius = DOT_SIZE / 2.0;
                if is_filled {
                    painter.circle_filled(center, radius, DOT_FILLED_COLOR);
                } else {
                    painter.circle_stroke(center, radius, egui::Stroke::new(1.0, DOT_EMPTY_COLOR));
                }
            }
        }
    }
    response
}

/// One species as a selectable row in the Species list (task 065 —
/// converted from a horizontally scrollable chip strip, task 064's original
/// choice, after playtest feedback: the strip's hidden scrollbar needed a
/// dedicated `›` overflow cue and was less discoverable than the vertical,
/// internally-scrolling pattern already used for Biosphere). Shows the
/// species' metabolism (task 065) alongside its color swatch and name — a
/// *readable* trait per GDD §5.3, but previously only visible in the
/// notebook's species catalog, which a fresh player hasn't opened yet at
/// their first placement. Clicking sets `SelectedSpecies`, same behavior as
/// the old chip/radio list.
fn species_row(ui: &mut egui::Ui, species: SpeciesId, world: &SimWorld, selected: &mut SpeciesId) {
    let is_selected = *selected == species;
    let metabolism = world.species[species.0 as usize].metabolism;
    let text = egui::RichText::new(format!(
        "{} {}",
        metabolism_glyph(metabolism),
        species_label(world, species)
    ))
    .color(species_color(species));
    if ui.selectable_label(is_selected, text).clicked() {
        *selected = species;
    }
}

/// On-screen equivalents of the tick/era/notebook keyboard shortcuts (task
/// 094), coexisting with them — clicking a button here only sets a
/// `HudControlIntents` flag, the same `input.rs::start_era`/`single_tick`/
/// `notebook.rs::toggle_notebook` systems that already handle the matching
/// key act on it, so there is exactly one implementation of each action
/// regardless of which input triggered it. Tick/Era grey out while an era
/// is already advancing, mirroring their keyboard equivalents' own
/// early-return guard rather than leaving a clickable button that silently
/// does nothing (same pattern `action_icon_row` uses for Stress/Cull
/// outside Detail).
fn time_control_row(
    ui: &mut egui::Ui,
    intents: &mut HudControlIntents,
    advancing: bool,
    notebook_open: bool,
) {
    ui.horizontal(|ui| {
        let tick_response = ui
            .add_enabled_ui(!advancing, |ui| ui.button(text::TICK_BUTTON_LABEL))
            .inner;
        if tick_response.clicked() {
            intents.advance_tick = true;
        }
        tick_response.on_hover_text(if advancing {
            format!(
                "{}{}",
                text::TICK_BUTTON_TOOLTIP,
                text::ADVANCING_DISABLED_HINT
            )
        } else {
            text::TICK_BUTTON_TOOLTIP.to_string()
        });

        let era_response = ui
            .add_enabled_ui(!advancing, |ui| ui.button(text::ERA_BUTTON_LABEL))
            .inner;
        if era_response.clicked() {
            intents.advance_era = true;
        }
        era_response.on_hover_text(if advancing {
            format!(
                "{}{}",
                text::ERA_BUTTON_TOOLTIP,
                text::ADVANCING_DISABLED_HINT
            )
        } else {
            text::ERA_BUTTON_TOOLTIP.to_string()
        });

        let notebook_response = ui.selectable_label(notebook_open, text::NOTEBOOK_BUTTON_LABEL);
        if notebook_response.clicked() {
            intents.toggle_notebook = true;
        }
        notebook_response.on_hover_text(text::NOTEBOOK_BUTTON_TOOLTIP);
    });
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
fn action_icon_row(
    ui: &mut egui::Ui,
    selected_action: &mut SelectedAction,
    config: &SimConfig,
    view_mode: MapViewMode,
) {
    ui.horizontal(|ui| {
        for (action_mode, glyph) in ACTION_GLYPHS {
            let cost = action_cost(action_mode, config);
            // Stress/Cull need per-organism precision Overview's
            // cluster-heatmap aggregation (task 076) doesn't preserve, so
            // they're disabled outside Detail (task 077) — same
            // `add_enabled_ui` pattern `splice_panel` already uses for its
            // tag-cap gating, rather than leaving a clickable button that
            // silently does nothing.
            let detail_only = matches!(action_mode, ActionMode::Stress | ActionMode::Cull);
            let enabled = !detail_only || view_mode == MapViewMode::Detail;
            let response = ui
                .add_enabled_ui(enabled, |ui| {
                    ui.selectable_label(
                        selected_action.0 == action_mode,
                        egui::RichText::new(glyph).size(20.0),
                    )
                })
                .inner;
            if response.clicked() {
                selected_action.0 = action_mode;
            }
            let tooltip = if enabled {
                text::action_tooltip(action_mode, cost)
            } else {
                format!(
                    "{}{}",
                    text::action_tooltip(action_mode, cost),
                    text::DETAIL_MODE_ONLY_HINT
                )
            };
            response.on_hover_text(tooltip);
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
            species_label(world, SpeciesId(i)),
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

/// A species' average-energy direction since the last completed era (task
/// 063), replacing the old `avg_energy/repro_threshold` HUD comparison —
/// that compared a population *average* against a per-*individual*
/// reproduction trait, which could read as "nobody's close" while
/// individuals had already crossed the threshold. `Stable` is also the
/// default for a species with no prior-era snapshot yet (first era it
/// appears in), since there's nothing yet to compare against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopulationTrend {
    Rising,
    Falling,
    Stable,
}

/// Pure classification: `previous` is `None` for a species with no
/// prior-era snapshot, `Some` otherwise. `epsilon` (`config.energy.
/// trend_epsilon`) is the minimum absolute change to call it `Rising`/
/// `Falling` — without it, energy's natural tick-to-tick noise would flip
/// the indicator most eras even when the population is actually holding
/// steady.
fn classify_trend(previous: Option<f32>, current: f32, epsilon: f32) -> PopulationTrend {
    let Some(previous) = previous else {
        return PopulationTrend::Stable;
    };
    let delta = current - previous;
    if delta > epsilon {
        PopulationTrend::Rising
    } else if delta < -epsilon {
        PopulationTrend::Falling
    } else {
        PopulationTrend::Stable
    }
}

/// Per-species average energy as of the last completed era, and the trend
/// that comparison produced. `species_stats` recomputes the live average
/// every frame for the raw number the HUD still shows, but the *trend* must
/// only update once per era (`update_population_trends`, on
/// `EraCompleted`) or it would flicker tick-to-tick instead of reading as a
/// stable per-era signal. Indexed by `SpeciesId`, growing on demand the same
/// way `species_stats` itself tolerates `Splice` (task 025) adding new
/// species mid-run.
#[derive(Resource, Default)]
pub struct PopulationTrends {
    previous_avg_energy: Vec<Option<f32>>,
    current: Vec<PopulationTrend>,
}

impl PopulationTrends {
    pub fn trend_for(&self, species: SpeciesId) -> PopulationTrend {
        self.current
            .get(species.0 as usize)
            .copied()
            .unwrap_or(PopulationTrend::Stable)
    }
}

fn update_population_trends(
    world: Res<SimWorld>,
    config: Res<SimConfig>,
    mut era_completed: MessageReader<EraCompleted>,
    mut trends: ResMut<PopulationTrends>,
) {
    let mut fired = false;
    for _ in era_completed.read() {
        fired = true;
    }
    if !fired {
        return;
    }

    let species_count = world.species.len();
    if trends.previous_avg_energy.len() < species_count {
        trends.previous_avg_energy.resize(species_count, None);
        trends
            .current
            .resize(species_count, PopulationTrend::Stable);
    }
    for (species, _population, avg_energy) in species_stats(&world) {
        let idx = species.0 as usize;
        trends.current[idx] = classify_trend(
            trends.previous_avg_energy[idx],
            avg_energy,
            config.energy.trend_epsilon,
        );
        trends.previous_avg_energy[idx] = Some(avg_energy);
    }
}

/// Glyph for a `PopulationTrend`, matching the sidebar-redesign mockup's
/// convention (▲ rising, ▼ falling, ▬ stable).
fn trend_glyph(trend: PopulationTrend) -> &'static str {
    match trend {
        PopulationTrend::Rising => "▲",
        PopulationTrend::Falling => "▼",
        PopulationTrend::Stable => "▬",
    }
}

/// Color for a `PopulationTrend`, same mockup convention: green rising, red
/// falling, gray stable — gray rather than either the positive/negative edge
/// colors `notebook.rs` uses for confirmed matrix effects, since "stable" is
/// neutral information, not the absence of one.
fn trend_color(trend: PopulationTrend) -> egui::Color32 {
    match trend {
        PopulationTrend::Rising => egui::Color32::from_rgb(96, 200, 120),
        PopulationTrend::Falling => egui::Color32::from_rgb(220, 96, 96),
        PopulationTrend::Stable => egui::Color32::from_gray(130),
    }
}

/// Per-species dominant death cause for the Biosphere panel (task 105,
/// `redesign/abiogenesis-death-legibility.md`) — a population-wide
/// aggregate over *every* death of that species this era (not just
/// player-placed ones, unlike `notebook.rs`'s death log, GDD §7's
/// curated-log principle keeping per-organism lines out of scope here).
/// Gated on deaths actually recorded this era, deliberately independent of
/// `PopulationTrend`: `species_stats`' average-energy trend can read
/// `▲`/`▬` while a species is actively being culled (the survivors' average
/// goes *up* as the weakest die), which is exactly the "predator quietly
/// starving somewhere on the map" case this label exists to catch.
#[derive(Resource, Default)]
pub struct DeathCauseTally {
    /// Every dominant cause recorded this era, per species, in occurrence
    /// order — cleared right after each `EraCompleted` is processed below.
    causes_this_era: Vec<Vec<text::DominantDeathCause>>,
    /// The cause exposed to the HUD, computed once per `EraCompleted` from
    /// `causes_this_era` and held stable until the next one (mirrors
    /// `PopulationTrends::current`'s "only updates once per era" shape).
    /// `None` for a species with zero deaths that era — not a fabricated
    /// carry-over from whatever the previous era's dominant cause was.
    dominant_last_era: Vec<Option<text::DominantDeathCause>>,
}

impl DeathCauseTally {
    pub fn dominant_for(&self, species: SpeciesId) -> Option<text::DominantDeathCause> {
        self.dominant_last_era
            .get(species.0 as usize)
            .copied()
            .flatten()
    }
}

/// The most-tallied cause in `causes`, ties broken by "most recent dominant
/// cause wins" (task 105's explicit first-pass tie-break rule — the doc left
/// this open, so this is the stated choice, not an accidental one). Linear
/// aggregation over a handful of `DominantDeathCause` variants, not a
/// `HashMap` (`CLAUDE.md`'s no-`HashMap`-iteration rule for sim/HUD state
/// alike in this codebase's convention). `causes` must be non-empty.
fn dominant_cause_with_tiebreak(causes: &[text::DominantDeathCause]) -> text::DominantDeathCause {
    let mut tallied: Vec<(text::DominantDeathCause, u32, usize)> = Vec::new();
    for (i, &cause) in causes.iter().enumerate() {
        match tallied.iter_mut().find(|(c, ..)| *c == cause) {
            Some(entry) => {
                entry.1 += 1;
                entry.2 = i;
            }
            None => tallied.push((cause, 1, i)),
        }
    }
    tallied
        .into_iter()
        .max_by_key(|&(_, count, last_index)| (count, last_index))
        .map(|(cause, ..)| cause)
        .expect("causes is non-empty")
}

/// Drains `OrganismDied` into `DeathCauseTally` every tick (task 105) —
/// scheduled exactly like `record_events`/`update_population_trends`, not
/// gated to era-advance ticks only, so deaths from the manual single-tick
/// path (`input.rs::single_tick`) are tallied too, not silently dropped.
/// On `EraCompleted`, computes and exposes each species' dominant cause for
/// the era that just ended, then resets the accumulator for the next one.
fn tally_death_causes(
    mut deaths: MessageReader<OrganismDied>,
    mut era_completed: MessageReader<EraCompleted>,
    mut tally: ResMut<DeathCauseTally>,
) {
    for event in deaths.read() {
        let idx = event.species.0 as usize;
        if tally.causes_this_era.len() <= idx {
            tally.causes_this_era.resize(idx + 1, Vec::new());
        }
        let cause = text::dominant_death_cause(
            event.gain,
            event.env_fit,
            event.interaction_delta,
            event.upkeep,
            event.crowding_penalty,
            event.predation_loss,
        );
        tally.causes_this_era[idx].push(cause);
    }

    let mut era_ended = false;
    for _ in era_completed.read() {
        era_ended = true;
    }
    if !era_ended {
        return;
    }

    let species_count = tally.causes_this_era.len();
    if tally.dominant_last_era.len() < species_count {
        tally.dominant_last_era.resize(species_count, None);
    }
    let DeathCauseTally {
        causes_this_era,
        dominant_last_era,
    } = &mut *tally;
    for (idx, dominant) in dominant_last_era.iter_mut().enumerate() {
        *dominant = causes_this_era
            .get(idx)
            .filter(|causes| !causes.is_empty())
            .map(|causes| dominant_cause_with_tiebreak(causes));
    }
    for causes in causes_this_era {
        causes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dominant_cause_with_tiebreak_picks_the_highest_count() {
        use text::DominantDeathCause::*;
        let causes = [Predation, Crowding, Predation, Predation, Crowding];
        assert_eq!(dominant_cause_with_tiebreak(&causes), Predation);
    }

    #[test]
    fn dominant_cause_with_tiebreak_breaks_ties_by_most_recent_occurrence() {
        use text::DominantDeathCause::*;
        // Predation and Crowding both occur twice; Crowding's second
        // occurrence is later in the sequence, so — per task 105's explicit
        // "most recent dominant cause wins" rule — it wins the tie even
        // though neither count is larger.
        let causes = [Predation, Crowding, Predation, Crowding];
        assert_eq!(dominant_cause_with_tiebreak(&causes), Crowding);
    }

    /// One `OrganismDied` whose energy terms are hand-picked to make
    /// `text::dominant_death_cause` classify it as `cause` — used to drive
    /// `tally_death_causes` in the system-level tests below without needing
    /// a full `SimWorld`/`step()` tick.
    fn organism_died_with_cause(
        species: SpeciesId,
        cause: text::DominantDeathCause,
    ) -> OrganismDied {
        let base = OrganismDied {
            cell: 0,
            species,
            gain: 0.5,
            env_fit: 0.9,
            interaction_delta: 0.0,
            upkeep: 0.5,
            crowding_penalty: 0.0,
            predation_loss: 0.0,
            energy_before: 0.1,
        };
        match cause {
            text::DominantDeathCause::Predation => OrganismDied {
                predation_loss: 2.0,
                ..base
            },
            text::DominantDeathCause::Crowding => OrganismDied {
                crowding_penalty: 2.0,
                ..base
            },
            _ => unimplemented!("not needed by these tests"),
        }
    }

    fn app_for_tally_death_causes() -> App {
        let mut app = App::new();
        app.init_resource::<DeathCauseTally>();
        app.add_message::<OrganismDied>();
        app.add_message::<EraCompleted>();
        app.add_systems(Update, tally_death_causes);
        app
    }

    #[test]
    fn tally_death_causes_exposes_the_most_tallied_cause_after_era_completes() {
        let mut app = app_for_tally_death_causes();
        for _ in 0..2 {
            app.world_mut()
                .resource_mut::<Messages<OrganismDied>>()
                .write(organism_died_with_cause(
                    SpeciesId(0),
                    text::DominantDeathCause::Predation,
                ));
        }
        app.world_mut()
            .resource_mut::<Messages<OrganismDied>>()
            .write(organism_died_with_cause(
                SpeciesId(0),
                text::DominantDeathCause::Crowding,
            ));
        app.world_mut()
            .resource_mut::<Messages<EraCompleted>>()
            .write(EraCompleted { era: 1 });
        app.update();

        let tally = app.world().resource::<DeathCauseTally>();
        assert_eq!(
            tally.dominant_for(SpeciesId(0)),
            Some(text::DominantDeathCause::Predation),
            "predation (2 deaths) should dominate over crowding (1 death) this era"
        );
    }

    #[test]
    fn tally_death_causes_resets_between_eras_and_clears_species_with_no_deaths() {
        let mut app = app_for_tally_death_causes();
        app.world_mut()
            .resource_mut::<Messages<OrganismDied>>()
            .write(organism_died_with_cause(
                SpeciesId(0),
                text::DominantDeathCause::Predation,
            ));
        app.world_mut()
            .resource_mut::<Messages<EraCompleted>>()
            .write(EraCompleted { era: 1 });
        app.update();
        assert_eq!(
            app.world()
                .resource::<DeathCauseTally>()
                .dominant_for(SpeciesId(0)),
            Some(text::DominantDeathCause::Predation)
        );

        // A second era completes with no deaths at all.
        app.world_mut()
            .resource_mut::<Messages<EraCompleted>>()
            .write(EraCompleted { era: 2 });
        app.update();
        assert_eq!(
            app.world().resource::<DeathCauseTally>().dominant_for(SpeciesId(0)),
            None,
            "a species with zero deaths this era must show no label, not a stale carry-over from the previous era"
        );
    }

    #[test]
    fn era_progress_display_shows_dots_at_and_under_the_cap() {
        assert_eq!(
            era_progress_display(ERA_PROGRESS_DOT_CAP),
            EraProgressDisplay::Dots
        );
        assert_eq!(era_progress_display(1), EraProgressDisplay::Dots);
    }

    #[test]
    fn era_progress_display_falls_back_to_numeric_past_the_cap() {
        assert_eq!(
            era_progress_display(ERA_PROGRESS_DOT_CAP + 1),
            EraProgressDisplay::Numeric
        );
    }

    #[test]
    fn classify_trend_detects_a_clear_rise() {
        assert_eq!(classify_trend(Some(5.0), 7.0, 0.5), PopulationTrend::Rising);
    }

    #[test]
    fn classify_trend_detects_a_clear_fall() {
        assert_eq!(
            classify_trend(Some(7.0), 5.0, 0.5),
            PopulationTrend::Falling
        );
    }

    #[test]
    fn classify_trend_within_epsilon_reads_as_stable() {
        assert_eq!(classify_trend(Some(5.0), 5.3, 0.5), PopulationTrend::Stable);
        assert_eq!(classify_trend(Some(5.0), 4.7, 0.5), PopulationTrend::Stable);
    }

    #[test]
    fn classify_trend_with_no_previous_snapshot_reads_as_stable() {
        assert_eq!(classify_trend(None, 100.0, 0.5), PopulationTrend::Stable);
    }

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
        const DURATION: u64 = 30;
        assert!(isolation_hint_active(100, DURATION, 100));
        assert!(isolation_hint_active(100, DURATION, 100 + DURATION - 1));
        assert!(!isolation_hint_active(100, DURATION, 100 + DURATION));
    }

    /// Task 092: a shorter onboarding-era duration dismisses sooner than a
    /// standard-era one, for the same `shown_at_tick` — the whole point of
    /// deriving `duration_ticks` from the era it was shown in instead of a
    /// fixed constant.
    #[test]
    fn isolation_hint_active_respects_a_shorter_duration() {
        assert!(!isolation_hint_active(100, 8, 108));
        assert!(isolation_hint_active(100, 25, 108));
    }

    #[test]
    fn isolation_hint_active_never_underflows_if_current_tick_precedes_shown_at() {
        // Shouldn't happen in practice (`world.tick` only increases), but
        // `saturating_sub` must not panic if it ever does — elapsed
        // saturates to 0, so the hint reads as freshly active rather than
        // crashing.
        assert!(isolation_hint_active(100, 30, 0));
    }
}
