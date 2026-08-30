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
use crate::render::{metabolism_glyph, metabolism_mask, species_label, GridCamera, MapViewMode};
use crate::text;
use abiogenesis::config::SimConfig;
use abiogenesis::knowledge::MatrixKnowledge;
use abiogenesis::objectives::{
    is_grace_active, CurrentObjective, GraceProgress, Objective, ObjectiveProgress, ZoneKind,
};
use abiogenesis::run::{MetaProgress, RunProgress};
use abiogenesis::sim::{
    any_evolution_maturing, cell_energy_breakdown, ActionBudget, EraCompleted, OrganismDied,
    SeasonProgress,
};
use abiogenesis::state::{EraState, GameState};
use abiogenesis::world::{
    Cell, Metabolism, Population, SimWorld, SpeciesId, SpeciesOrigin, StressAxis, TagSlot,
};
use abiogenesis::worldgen::season_pulses_for;

/// Task 055's guided first-isolation hint: the message to show (isolated vs.
/// clustered first placement), the `SimWorld::tick` it was set at, and how
/// long it stays up — the tick counter the HUD's own "Era X · Season Y ·
/// tick Z" readout already exposes, not a wall-clock timer. Written once, by
/// `input.rs`'s `seed_organism_on_click` at the player's first-ever
/// placement of their first-ever run (gated on
/// `MetaProgress::seen_isolation_hint`); read here.
///
/// `duration_ticks` (task 092) is computed once at set-time from
/// `worldgen::season_pulses_for` for whatever season is current *then* — not
/// a fixed constant, and not re-derived on later frames even if the season
/// changes underneath it before the hint dismisses. Before task 092 this was
/// a fixed `30`, which read as "about one season" back when every season was
/// `25` ticks; task 082's shortened onboarding seasons (`8` ticks each) made
/// that stale, since the hint kept outliving the season it was shown in and
/// no longer read as anchored to anything. Pinning the duration to the
/// *shown-in* season (rather than re-deriving it against whatever season is
/// current each frame) is the deliberate choice here: a hint that started
/// during an 8-tick onboarding season and is still up when season 4's
/// 25-tick length begins should still resolve on its own short original
/// schedule, not suddenly gain 17 more ticks of life because the season
/// around it got longer.
#[derive(Resource, Default)]
pub struct IsolationHint {
    pub text: Option<&'static str>,
    pub shown_at_tick: u64,
    pub duration_ticks: u64,
}

/// Task 143's second contextual hint, same one-shot/`MetaProgress`-gated,
/// era-derived-duration shape as `IsolationHint` — set once by
/// `check_stall_hint` the first time `sim::any_population_stalled` reports
/// true, self-dismissed by `viewport_hint` exactly like `IsolationHint`
/// (`isolation_hint_active` is reused for both, same timing rule).
#[derive(Resource, Default)]
pub struct StallHint {
    pub active: bool,
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

/// `HudControlIntents` bundled with `ContinuousAdvance` (task 152) — folded
/// into one `SystemParam` purely to keep `hud_panel` under Bevy's
/// per-system parameter ceiling (it was already at 16), the same rationale
/// `SpliceReadouts`/`ObjectiveReadouts` give for their own bundling.
/// `ContinuousAdvance` doesn't share `HudControlIntents`' one-shot
/// "consumed and reset every frame" contract — it's a persisting toggle —
/// so it stays a distinct field rather than a fourth intent flag.
#[derive(SystemParam)]
pub struct AdvanceControls<'w> {
    pub intents: ResMut<'w, HudControlIntents>,
    pub continuous: ResMut<'w, ContinuousAdvance>,
}

/// The species the seed action (task 017) places on click. A UI intent, not
/// simulation state: owned here, read by `input.rs`'s click-to-place system,
/// same rationale as `SeasonProgress` living in `sim.rs` but written by
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

/// The currently-selected click action, or `None` if no action is armed
/// (task 149's minimal seam for task 150's full Esc-cascade/action-armed
/// scheme: clicking the already-selected action's button again deselects
/// it). Defaults to `None` (task 177) — a fresh world starts with nothing
/// armed, so a player's first instinct to click a cell and look at it goes
/// to the click-inspect card (`select_cell_on_click`'s "no action armed"
/// path) instead of accidentally seeding a species
/// (`playtest_outcome.md` issue I.2).
#[derive(Resource)]
pub struct SelectedAction(pub Option<ActionMode>);

/// Which cell the cursor is currently over (task 149), updated every frame
/// regardless of `SelectedAction`/`ActionBudget` — the hover tooltip is
/// free and always on. `SimWorld::index`-flattened, not `(x, y)`, matching
/// what `sim::cell_energy_breakdown` and the click-inspect card both need.
#[derive(Resource, Default)]
pub struct HoveredCell(pub Option<usize>);

/// The cell the click-to-inspect card is showing (task 149) — distinct from
/// `HoveredCell`: the card follows the last click, not the cursor, and
/// stays open until another cell is selected or Esc is pressed.
#[derive(Resource, Default)]
pub struct SelectedCell(pub Option<usize>);

/// Whether the pause menu is open (task 150). A resource, not a
/// `GameState` variant: `EraState` is a substate of `GameState::Playing`
/// (`state.rs`), so leaving `Playing` to "pause" would reset it back to its
/// `Observing` default — losing exactly the mid-season/mid-era progress a
/// pause is supposed to preserve. `input.rs`'s Esc cascade is the only
/// writer; `sim.rs`'s `advance_tick`/`objectives.rs`'s objective evaluation
/// both gate on this too, so simulation time genuinely stops while it's up.
#[derive(Resource, Default)]
pub struct PauseMenuOpen(pub bool);

/// True once the current world has taken its first successful player
/// action (task 150, `input.rs`'s four click/splice systems are the only
/// writers) — `r` requires an explicit `PendingConfirmation` once this is
/// set, instead of reseeding instantly (nothing to lose before this is
/// true). Reset alongside every other per-world resource in
/// `run_flow::start_world`'s `WorldResetParams` and `menu::start_run`.
#[derive(Resource, Default)]
pub struct WorldTouched(pub bool);

/// Which destructive action `PendingConfirmation` is gating (task 150) —
/// `r`'s reseed-when-touched gate and the pause menu's "Abbandona senza
/// salvare"/"Salva ed esci" items share this one dialog primitive, per the
/// design doc's own reasoning for why a second one shouldn't exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationKind {
    ReseedWorld,
    AbandonRun,
}

/// The in-flight confirmation dialog, if any (task 150). `confirmation_dialog`
/// (this module) only ever sets `kind`/toggles `confirmed` — it never
/// performs the action itself, matching TECH_DESIGN.md §3.3's "`ui.rs`
/// writes only intents": `input.rs`'s `reseed_world`/pause-menu systems own
/// applying the effect once `confirmed` is seen, then clear this back to
/// `default()`.
#[derive(Resource, Default)]
pub struct PendingConfirmation {
    pub kind: Option<ConfirmationKind>,
    pub confirmed: bool,
}

/// Continuous pulse advancement (task 152, GDD §11's `p` keybind) — a third
/// advance mechanism alongside the manual `n`/`space`/`shift+space`
/// controls (`input.rs`), mutually exclusive with them while active.
/// Deliberately reuses `TimeConfig::era_tick_hz`'s existing `Time<Fixed>`
/// pacing (`sim.rs::SimPlugin`) rather than a new rate — this is a
/// presentation-layer cadence, not a simulation coefficient (advancing
/// wall-clock speed must never change simulation outcomes, TECH_DESIGN.md
/// invariant 1). `input.rs::continuous_advance` is the only reader that
/// acts on it; `p` and this module's HUD button both flip it.
#[derive(Resource, Default)]
pub struct ContinuousAdvance(pub bool);

/// Task 176: `Time<Fixed>` steps at `TimeConfig::era_tick_hz`, but
/// continuous-advance should play back slower
/// (`TimeConfig::continuous_advance_tick_hz`) — this counts elapsed
/// `FixedUpdate` steps since the last pulse so `input.rs::continuous_advance`
/// can skip most of them instead of advancing on every one, without
/// touching the shared `Time<Fixed>` rate the manual controls also use.
#[derive(Resource, Default)]
pub struct ContinuousAdvancePulseCounter(pub u32);

/// Which axis `Stress` targets (task 145) — a UI intent read by both this
/// module (the axis sub-selector, shown only while `Stress` is the active
/// action) and `input.rs`'s `stress_on_click`. Defaults to `Temperature` so
/// a player who never touches the selector keeps the old single-axis
/// behaviour.
#[derive(Resource, Default)]
pub struct SelectedStressAxis(pub StressAxis);

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
pub(crate) const HUD_WIDTH: f32 = 340.0;

/// `RenderLayers` for the dedicated egui camera: no grid entity is ever
/// assigned to it, so this camera draws nothing of the scene, only the
/// egui overlay (TECH_DESIGN.md §6 "HUD camera").
const HUD_CAMERA_LAYER: usize = 1;

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

/// Side length of the Biosphere row's metabolism icon (task 180) — small
/// enough to sit inline with the row's text at normal line height;
/// `paint_metabolism_icon`'s own `BIOSPHERE_ICON_SAMPLE_GRID` (not the
/// map's coarser `SHAPE_BLOCK_GRID`) is what keeps the shape legible at
/// this size.
const BIOSPHERE_ICON_SIZE: f32 = 12.0;

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
/// row of dots sized to an unbounded `seasons_required` would either overflow
/// `HUD_WIDTH` or shrink past legibility. Chosen so a dot row still fits
/// comfortably within the panel width at `DOT_SIZE`/`DOT_GAP`.
const SEASON_PROGRESS_DOT_CAP: u32 = 8;

/// Color for the objective's narrative-quoted line (task 064 §5) — a warm,
/// slightly desaturated off-white distinct from the panel's neutral-gray
/// monospace body text, so the one deliberate font/style break also reads
/// as visually distinct in color, not just in shape.
const OBJECTIVE_NARRATIVE_COLOR: egui::Color32 = egui::Color32::from_rgb(180, 178, 169);

pub struct UiPlugin;

/// DejaVu Sans (Bitstream Vera License, see `assets/fonts/DejaVu-LICENSE.txt`)
/// covers the `●` bullet that egui's built-in default font lacks — that
/// glyph was rendering as a tofu box (playtest finding, task 041 session):
/// `notebook.rs::TAG_GLYPH` still relies on this font's coverage. This
/// module's own `SPECIES_GLYPH` bullet (task 041) and `ACTION_GLYPHS`'
/// unrendered emoji (`🌱⚡💀🔬` — egui has no COLR/bitmap glyph path at all,
/// so no font choice ever fixed these) are both gone as of task 180: the
/// Biosphere row and the action buttons now paint block-pattern icons
/// directly via `egui::Painter`, not font glyphs.
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
            .insert_resource(SelectedAction(None))
            .init_resource::<SelectedStressAxis>()
            .init_resource::<SpliceDraft>()
            .init_resource::<IsolationHint>()
            .init_resource::<StallHint>()
            .init_resource::<PopulationTrends>()
            .init_resource::<DeathCauseTally>()
            .init_resource::<HudControlIntents>()
            .init_resource::<HoveredCell>()
            .init_resource::<SelectedCell>()
            .init_resource::<PauseMenuOpen>()
            .init_resource::<WorldTouched>()
            .init_resource::<PendingConfirmation>()
            .init_resource::<ContinuousAdvance>()
            .init_resource::<ContinuousAdvancePulseCounter>()
            .add_systems(Startup, spawn_hud_camera)
            .add_systems(
                Update,
                (
                    reserve_hud_viewport,
                    update_population_trends,
                    tally_death_causes,
                    check_stall_hint,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(EguiPrimaryContextPass, configure_fonts)
            .add_systems(
                EguiPrimaryContextPass,
                // Task 140: suppressed while `EraState::Reveal` is up rather
                // than merely disabled — the reveal card is a full-viewport
                // `CentralPanel` on the same background egui layer
                // `hud_panel` itself paints on (`screens::interstitial`),
                // so drawing both in the same frame would overlap rather
                // than layer cleanly the way the notebook's dim-not-hide
                // overlay does.
                (
                    hud_panel,
                    viewport_hint,
                    hover_tooltip,
                    inspect_card,
                    pause_menu,
                    confirmation_dialog,
                )
                    .run_if(
                        in_state(GameState::Playing).and_eager(not(in_state(EraState::Reveal))),
                    ),
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
    // Task 151's pixel-grain register: egui's default theme rounds window/
    // menu/widget corners — squared off here, once, globally, rather than
    // hunting down a `corner_radius` call on every individual panel/button/
    // window this and other modules (`screens.rs`, `notebook.rs`) build.
    // Frame-level radii set explicitly elsewhere (e.g. `screens.rs`'s era-
    // reveal card) are separate calls and unaffected by this default.
    ctx.all_styles_mut(|style| {
        let visuals = &mut style.visuals;
        visuals.window_corner_radius = egui::CornerRadius::ZERO;
        visuals.menu_corner_radius = egui::CornerRadius::ZERO;
        // Task 182 follow-up: the HUD sidebar, notebook window, and every
        // popup/tooltip/inspect-window draw through egui's own
        // `panel_fill`/`window_fill` defaults, which are close but not the
        // guide's exact `#1c2229` — the same kind of hex-drift task 182
        // already fixed for the interstitial/menu screens' `CentralPanel`s
        // (those set an explicit per-frame `.fill(PANEL_BG)` instead of
        // relying on this default, since they predate it). Set once here
        // rather than hunting down every `Panel`/`Window`/`Frame::popup`
        // call site across `ui.rs`/`notebook.rs`.
        visuals.panel_fill = PANEL_BG;
        visuals.window_fill = PANEL_BG;
        // Task 184: same reasoning as the fill override above — every
        // `Frame::window`/`Frame::popup`/tooltip (including the built-in
        // `.on_hover_text()` tooltip, which always uses `Frame::popup`
        // internally with no per-call override point) draws its border
        // from `window_stroke` and its drop shadow from `window_shadow`/
        // `popup_shadow`. Set once here instead of hand-rolling a `.frame()`
        // override on every popup/tooltip call site — `VISUAL_STYLE_GUIDE.md`
        // §1 rule 4 forbids the default's blurred shadow everywhere, not
        // just on the three surfaces this task named explicitly.
        visuals.window_stroke = egui::Stroke::new(1.0, OUTLINE_STROKE);
        visuals.window_shadow = egui::Shadow::NONE;
        visuals.popup_shadow = egui::Shadow::NONE;
        for widgets in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widgets.corner_radius = egui::CornerRadius::ZERO;
        }
    });
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
///
/// Deliberately **not** extended for the notebook panel (task 116, tried and
/// reverted live 2026-08-13): the HUD sidebar is permanent, so resizing the
/// map to exactly fill what's left of it is the right call — but the
/// notebook is a toggleable overlay, and shrinking/re-centering the camera
/// every time it opens and closes made the map visibly resize and rescale
/// each toggle, which isn't what `redesign/abiogenesis-hud-notebook.md` §9
/// describes ("the map stays there, dimmed behind it" — same size, just
/// partially covered). The notebook's own coverage of the map is handled
/// entirely by drawing on top of it instead: `notebook_window`'s opaque
/// panel plus dimming rect, and `render.rs::draw_terrain_overlay`'s own clip
/// (task 116) excluding the notebook's reserved rect so its boundary/tree
/// painting doesn't bleed through underneath.
fn reserve_hud_viewport(
    windows: Query<&Window>,
    mut cameras: Query<&mut Camera, With<GridCamera>>,
    era_state: Res<State<EraState>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    let full_width = window.physical_width();
    let full_height = window.physical_height();
    if full_height == 0 {
        return;
    }

    // `hud_panel` doesn't draw during `EraState::Reveal` (task 140), so
    // reserving its strip here would leave a blank, un-rendered gap on the
    // right rather than letting the grid — and the reveal card's dim
    // backdrop over it — use the full window.
    if *era_state.get() == EraState::Reveal {
        camera.viewport = Some(Viewport {
            physical_position: UVec2::ZERO,
            physical_size: UVec2::new(full_width, full_height),
            ..default()
        });
        return;
    }

    let hud_px = (HUD_WIDTH * window.scale_factor()) as u32;
    if full_width <= hud_px {
        return;
    }

    camera.viewport = Some(Viewport {
        physical_position: UVec2::ZERO,
        physical_size: UVec2::new(full_width - hud_px, full_height),
        ..default()
    });
}

/// True when `cursor` (logical window coordinates, the space
/// `Window::cursor_position` returns) falls inside the HUD panel's on-screen
/// rect — the right `HUD_WIDTH`-wide, full-height strip `hud_panel` reserves.
///
/// Task 115: grid click/scroll-zoom handlers (`input.rs::clicked_cell`,
/// `render.rs::zoom_camera`) can't rely on `EguiWantsInput` alone to exclude
/// this area. `hud_panel` shows its `Panel::right` into a `Ui` built on
/// `egui::LayerId::background()` (`ui.rs:384-390`) — the same layer/order the
/// grid's own terrain overlay paints on — and bevy_egui 0.41's multi-pass
/// driver (`run_egui_context_pass_loop_system`) discards the real top-level
/// root `Ui` egui's `Context::is_pointer_over_egui` measures
/// `root_ui_available_rect` against (it runs `EguiPrimaryContextPass`
/// directly instead of drawing into that root `Ui`), so that rect never
/// shrinks to exclude the panel. The upshot: `is_pointer_over_egui` (and thus
/// `EguiWantsInput::wants_pointer_input`/`is_pointer_over_area`) reports
/// `false` for the *entire* viewport, panel included, whenever the pointer
/// resolves to a Background-order layer — plain rect math against the window
/// sidesteps that egui/bevy_egui interaction entirely instead of working
/// around it.
pub(crate) fn cursor_over_hud_panel(cursor: Vec2, window_width: f32) -> bool {
    cursor.x >= window_width - HUD_WIDTH
}

#[cfg(test)]
mod hud_panel_rect_tests {
    use super::*;

    #[test]
    fn cursor_inside_the_reserved_hud_strip_is_flagged() {
        let window_width = 1200.0;
        assert!(cursor_over_hud_panel(
            Vec2::new(window_width - 1.0, 400.0),
            window_width
        ));
        assert!(cursor_over_hud_panel(
            Vec2::new(window_width - HUD_WIDTH, 0.0),
            window_width
        ));
    }

    #[test]
    fn cursor_left_of_the_reserved_hud_strip_is_not_flagged() {
        let window_width = 1200.0;
        assert!(!cursor_over_hud_panel(
            Vec2::new(window_width - HUD_WIDTH - 1.0, 400.0),
            window_width
        ));
        assert!(!cursor_over_hud_panel(Vec2::ZERO, window_width));
    }
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

/// Objective/run-progress readouts bundled into one `SystemParam` (task
/// 109, same rationale as `WorldResetParams`/`ObjectiveOutcomeParams`):
/// adding `RunProgress` as `hud_panel`'s own parameter for the energy
/// readout pushed it to 17 individual parameters, past Bevy's tuple-based
/// `IntoSystem` ceiling of 16 — bundling these four (which `objective_panel`
/// already reads together) brings it back under.
#[derive(SystemParam)]
pub struct ObjectiveReadouts<'w> {
    pub run_progress: Res<'w, RunProgress>,
    pub objective: Res<'w, CurrentObjective>,
    pub objective_progress: Res<'w, ObjectiveProgress>,
    pub grace: Res<'w, GraceProgress>,
}

/// `Splice` editor state bundled into one `SystemParam` (task 147, same
/// rationale as `ObjectiveReadouts` above): adding `MatrixKnowledge` as
/// `hud_panel`'s own parameter (the confirmed-tag filter `splice_panel`
/// needs) pushed it back past Bevy's 16-parameter ceiling.
#[derive(SystemParam)]
pub struct SpliceReadouts<'w> {
    pub draft: ResMut<'w, SpliceDraft>,
    pub knowledge: Res<'w, MatrixKnowledge>,
}

#[allow(clippy::too_many_arguments)]
/// `pub(crate)` (task 091) so `notebook.rs` can order its stray-focus-clear
/// system `.after(hud_panel)` — the sidebar action buttons this draws are
/// what egui's own `Tab` keyboard-navigation grabs focus onto, so the fix
/// must run after they've had their chance to claim it this frame.
pub(crate) fn hud_panel(
    mut contexts: EguiContexts,
    world: Res<SimWorld>,
    season_progress: Res<SeasonProgress>,
    era_state: Res<State<EraState>>,
    mut selected: ResMut<SelectedSpecies>,
    mut selected_action: ResMut<SelectedAction>,
    mut selected_stress_axis: ResMut<SelectedStressAxis>,
    mut splice: SpliceReadouts,
    budget: Res<ActionBudget>,
    config: Res<SimConfig>,
    readouts: ObjectiveReadouts,
    unseen_confirmation: Res<NotebookHasUnseenConfirmation>,
    biosphere: BiosphereReadouts,
    mode: Res<MapViewMode>,
    mut controls: AdvanceControls,
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
            let (season_current, season_total) = season_readout_values(
                readouts.run_progress.world_index,
                world.season,
                season_progress.remaining(),
                &config,
            );
            ui.label(text::season_tick_line(
                world.era,
                world.season,
                season_current,
                season_total,
            ));
            ui.label(text::state_line(era_state.get()));
            if is_grace_active(world.season, config.time.grace_seasons, &readouts.grace) {
                ui.weak(text::GRACE_PERIOD_LINE);
            }
            if any_evolution_maturing(&world, &config.evolution) {
                ui.weak(text::MATURING_EVOLUTION_HINT);
            }
            time_control_row(
                ui,
                &mut controls.intents,
                *era_state.get() == EraState::Advancing,
                notebook_open.0,
                &mut controls.continuous,
            );

            hairline(ui);
            section_header(ui, text::HEADING_ACTION);
            let splice_confirmed_tags = world
                .active_tags
                .iter()
                .enumerate()
                .filter(|&(i, _)| splice.knowledge.is_tag_confirmed(TagSlot(i as u8)))
                .count();
            action_icon_row(
                ui,
                &mut selected_action,
                &config,
                &readouts.run_progress,
                *mode,
                splice_confirmed_tags,
            );
            if selected_action.0 == Some(ActionMode::Stress) {
                stress_axis_row(ui, &mut selected_stress_axis);
            }

            let total = config.time.point_budget_per_season;
            dot_row(ui, budget.points_remaining, total, DotShape::Tick)
                .on_hover_text(text::BUDGET_HOVER);
            ui.weak(text::budget_bar_text(budget.points_remaining, total));

            if selected_action.0 == Some(ActionMode::Splice) {
                ui.indent("splice_panel", |ui| {
                    splice_panel(ui, &world, &mut splice.draft, &splice.knowledge);
                });
            }

            hairline(ui);
            section_header(ui, text::HEADING_POPULATION);
            if stats.is_empty() {
                ui.weak(text::NO_POPULATION);
            }
            egui::ScrollArea::vertical()
                .id_salt("biosphere_list")
                .max_height(BIOSPHERE_VISIBLE_ROWS as f32 * console_row_height(ui))
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (species, population, avg_energy) in &stats {
                        ui.horizontal(|ui| {
                            // Task 180: species identity is text-only here
                            // (the name, below) — the icon carries
                            // metabolism, neutral amber ink, never a
                            // per-species hue (`VISUAL_STYLE_GUIDE.md` §1
                            // rule 3, §3.2), matching
                            // `pixel-full-scene.svg:6815-6826`.
                            let (icon_rect, _) = ui.allocate_exact_size(
                                egui::vec2(BIOSPHERE_ICON_SIZE, BIOSPHERE_ICON_SIZE),
                                egui::Sense::hover(),
                            );
                            let metabolism = world.species[species.0 as usize].metabolism;
                            paint_metabolism_icon(
                                ui.painter(),
                                icon_rect,
                                metabolism,
                                ICON_INK_SELECTED,
                            );
                            // Bare species name, not `species_label`'s "Name
                            // (species N)" (task 105 follow-up): the row
                            // already carries population, energy, a trend
                            // glyph, and sometimes a cause label — the
                            // "(species N)" suffix cost width without
                            // adding information here (it disambiguates in
                            // logs where the same-named case matters, not
                            // in a live per-tick readout) and was a
                            // contributor to the row overflowing `HUD_WIDTH`.
                            ui.label(text::population_line(
                                &world.species[species.0 as usize].name,
                                *population,
                                *avg_energy,
                            ));
                            let trend = biosphere.trends.trend_for(*species);
                            paint_trend_arrow(ui, trend, trend_color(trend));
                            let delta = biosphere.trends.population_delta_for(*species);
                            let delta_label = text::population_delta_label(delta);
                            if !delta_label.is_empty() {
                                ui.weak(delta_label);
                            }
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
            section_header(ui, text::HEADING_SEED_PALETTE).on_hover_text(text::SEED_PALETTE_HOVER);
            egui::ScrollArea::vertical()
                .id_salt("species_list")
                // Task 187: unlike a Biosphere row (one line), `species_row`
                // is two lines — the selectable name label plus `ui.weak`'s
                // metabolism/temperature-fit subtext — so the budget needs
                // twice `console_row_height`, or the list scrolled after
                // ~2-3 species instead of `SPECIES_VISIBLE_ROWS` (5).
                .max_height(SPECIES_VISIBLE_ROWS as f32 * console_row_height(ui) * 2.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for i in 0..world.species.len() as u8 {
                        // Wild populations (task 098) aren't a player
                        // choice — they already exist, elsewhere on the
                        // grid — so they're excluded from the seed palette.
                        if world.is_wild(SpeciesId(i)) {
                            continue;
                        }
                        species_row(ui, SpeciesId(i), &world, &config, &mut selected.0);
                    }
                });
            if world.species.len() > SPECIES_VISIBLE_ROWS {
                ui.weak(text::SCROLL_FOR_MORE);
            }

            hairline(ui);
            section_header(ui, text::HEADING_OBJECTIVE);
            objective_panel(
                ui,
                &world,
                readouts.objective.current(),
                readouts.objective.index,
                readouts.objective.total(),
                &readouts.objective_progress,
                config.time.season_pulses,
            );

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.weak(text::KEYBOARD_HINT_PRIMARY);
                ui.weak(text::KEYBOARD_HINT_SECONDARY);
                ui.weak(text::seed_line(world.seed));
                ui.weak(text::energy_line(readouts.run_progress.energy));
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

/// Latches `StallHint` the first time `sim::any_population_stalled` reports
/// true this session (task 143) — mirrors `input.rs::seed_organism_on_click`'s
/// `IsolationHint` latch: one-shot via `MetaProgress::seen_stall_hint`, same
/// era-derived `duration_ticks` (`worldgen::season_pulses_for`).
fn check_stall_hint(
    world: Res<SimWorld>,
    config: Res<SimConfig>,
    run_progress: Res<RunProgress>,
    mut meta: ResMut<MetaProgress>,
    mut stall_hint: ResMut<StallHint>,
) {
    if meta.seen_stall_hint || !abiogenesis::sim::any_population_stalled(&world) {
        return;
    }
    stall_hint.active = true;
    stall_hint.shown_at_tick = world.tick;
    stall_hint.duration_ticks =
        season_pulses_for(run_progress.world_index, world.season, &config) as u64;
    meta.seen_stall_hint = true;
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
    mut stall_hint: ResMut<StallHint>,
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
    if stall_hint.active
        && !isolation_hint_active(
            stall_hint.shown_at_tick,
            stall_hint.duration_ticks,
            world.tick,
        )
    {
        stall_hint.active = false;
    }

    // Task 143: the stall hint sits below both task-053 milestone hints and
    // the guided first-isolation hint — it never overlaps or preempts the
    // hints a fresh player sees first, only fills in once those have
    // already resolved (or never applied, e.g. later in the same run).
    let hint = if let Some(text) = isolation_hint.text {
        text
    } else if let Some(text) = hint_text(ever_seeded.0, notebook_ever_opened.0) {
        text
    } else if stall_hint.active {
        text::HINT_APPARENT_STALL
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
                apply_monospace(ui);
                ui.label(hint);
            });
        });

    Ok(())
}

/// Task 149's hover tooltip: free, always on, independent of `ActionMode`/
/// `ActionBudget` — a minimal label next to the cursor (biome name for any
/// cell, plus species/population/trend for a populated one). Reuses
/// `PopulationTrends`' own trend glyph/color and `text::population_delta_label`
/// (task 120) rather than inventing a second trend convention.
fn hover_tooltip(
    mut contexts: EguiContexts,
    hovered: Res<HoveredCell>,
    world: Res<SimWorld>,
    trends: Res<PopulationTrends>,
) -> Result {
    let Some(idx) = hovered.0 else {
        return Ok(());
    };
    let cell = world.cells[idx];
    let ctx = contexts.ctx_mut()?;
    let Some(cursor_pos) = ctx.pointer_hover_pos() else {
        return Ok(());
    };

    egui::Area::new("hover_tooltip".into())
        .order(egui::Order::Foreground)
        .interactable(false)
        .fixed_pos(cursor_pos + egui::vec2(16.0, 16.0))
        .show(ctx, |ui| {
            // Task 172: a shared Area id is fine (and keeps the position/size
            // memory stable frame-to-frame) as long as text can never wrap —
            // `Extend` disables wrapping outright, which is the actual fix;
            // switching to a per-cell id here previously "fixed" the 1-char
            // wrap by giving every newly-hovered cell a freshly-sized (and
            // therefore *un*-cached) Area, at the cost of minting a new
            // egui id per grid cell every frame.
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                apply_monospace(ui);
                ui.label(cell.biome.label());
                if let Some(population) = cell.population {
                    let trend = trends.trend_for(population.species);
                    ui.horizontal(|ui| {
                        ui.label(species_label(&world, population.species));
                        paint_trend_arrow(ui, trend, trend_color(trend));
                    });
                    let delta = trends.population_delta_for(population.species);
                    ui.label(text::hover_population_line(population.count, delta));
                }
            });
        });

    Ok(())
}

/// Task 149's click-to-inspect card: opens on `SelectedCell`, only reachable
/// today via a click while no action is armed (see `SelectedAction`'s doc
/// comment) — stays open until another cell is selected or Esc clears it
/// (`quit`, `input.rs`), not tied to the hovered cell at all.
fn inspect_card(
    mut contexts: EguiContexts,
    selected: Res<SelectedCell>,
    world: Res<SimWorld>,
    config: Res<SimConfig>,
) -> Result {
    let Some(idx) = selected.0 else {
        return Ok(());
    };
    let cell = world.cells[idx];
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Inspect")
        .resizable(false)
        .collapsible(false)
        .default_pos(egui::pos2(HUD_WIDTH + 24.0, 80.0))
        .show(ctx, |ui| {
            apply_monospace(ui);
            match cell.population {
                Some(population) => populated_cell_card(ui, &world, &config, idx, population),
                None => empty_cell_card(ui, &config, &cell),
            }
        });
    Ok(())
}

/// Populated-cell body of `inspect_card` (task 149): species, origin,
/// population, per-capita energy, a discrete tick indicator toward
/// `repro_threshold` (`dot_row`, the same idiom the action-budget/era-progress
/// rows already use — not a continuous bar), active tags, the saturated-
/// no-outlet warning (`Population::blocked`, task 137/141), and the
/// last-pulse balance breakdown (`sim::cell_energy_breakdown`).
fn populated_cell_card(
    ui: &mut egui::Ui,
    world: &SimWorld,
    config: &SimConfig,
    idx: usize,
    population: Population,
) {
    let species = &world.species[population.species.0 as usize];
    ui.heading(species_label(world, population.species));
    ui.label(format!("Biome: {}", world.cells[idx].biome.label()));
    biome_env_lines(ui, config, &world.cells[idx]);
    hairline(ui);
    ui.label(format!(
        "Origin: {}",
        text::species_origin_label(world.species_origin(population.species))
    ));
    ui.label(format!("Population: {}", population.count));
    let per_capita = population.energy / population.count as f32;
    ui.label(format!("Per-capita energy: {per_capita:.2}"));

    let notches = config.energy.repro_threshold.round().max(1.0) as u32;
    let filled = per_capita.floor().clamp(0.0, notches as f32) as u32;
    dot_row(ui, filled, notches, DotShape::Tick);

    ui.horizontal(|ui| {
        for &slot in &species.tags {
            let tag = world.active_tags[slot.0 as usize];
            ui.label(tag_glyph(tag));
        }
    });

    if population.blocked {
        ui.colored_label(STATE_NEGATIVE, text::SATURATED_NO_OUTLET_WARNING);
    }

    hairline(ui);
    if let Some(breakdown) = cell_energy_breakdown(world, config, idx) {
        ui.label(format!("Gain: {:+.2}", breakdown.gain));
        for line in &breakdown.neighbours {
            ui.label(format!(
                "{}: {:+.2}",
                tag_glyph(line.tag),
                line.contribution
            ));
        }
        ui.label(format!("Upkeep: {:.2}", -breakdown.upkeep));
        ui.label(format!("Crowding: {:.2}", -breakdown.crowding));
        ui.label(format!("Net: {:+.2}", breakdown.net));
    }
}

/// Empty-cell body of `inspect_card` (task 149): biome name, qualitative
/// temperature/light/toxicity bands (`text::band_label`, config-bound
/// thresholds), and a habitability flag. Deliberately never touches
/// `world.conditional_tags`/tag data at all — the hard constraint this
/// task calls out: the card must never leak whether a terrain-conditional
/// tag gate exists on this biome.
fn empty_cell_card(ui: &mut egui::Ui, config: &SimConfig, cell: &Cell) {
    ui.heading(cell.biome.label());
    biome_env_lines(ui, config, cell);
    let habitable = config.energy.is_habitable(cell.biome.index());
    ui.label(if habitable {
        text::HABITABLE_LABEL
    } else {
        text::NOT_HABITABLE_LABEL
    });
}

/// Biome/temperature/light/toxicity band lines shared by both `inspect_card`
/// bodies (task 172) — a populated cell lost this context entirely before,
/// since `populated_cell_card`/`empty_cell_card` were mutually exclusive.
/// Deliberately never touches `world.conditional_tags`/tag data, same
/// constraint `empty_cell_card` already followed.
fn biome_env_lines(ui: &mut egui::Ui, config: &SimConfig, cell: &Cell) {
    let env = &config.environment;
    ui.label(format!(
        "Temperature: {}",
        text::band_label(
            cell.temperature,
            env.ambient_temperature,
            env.source_temperature,
            ["cold", "temperate", "hot"],
        )
    ));
    ui.label(format!(
        "Light: {}",
        text::band_label(
            cell.light,
            env.light_low,
            env.light_high,
            ["dim", "moderate", "bright"]
        )
    ));
    ui.label(format!(
        "Toxicity: {}",
        text::band_label(
            cell.toxicity,
            0.0,
            env.swamp_toxicity_value,
            ["low", "moderate", "high"],
        )
    ));
}

/// Task 150's pause menu: reachable via the Esc cascade's last tier
/// (`input.rs::escape_cascade`) when nothing else is open/armed, or the
/// notebook/inspect-card/action-armed layers being closed one at a time.
/// Reuses egui's own default window chrome rather than `menu.rs`'s
/// full-viewport `CentralPanel` layout — that panel assumes it owns the
/// whole screen, which the pause menu must not (it overlays `Playing`, it
/// doesn't replace it). "Settings" has no panel to reuse yet, per the task's
/// own note — disabled with a hint rather than a silent no-op. "Save and
/// exit" has no save system yet (task 161) so it takes the same path as
/// "Abandon without saving" for now (see `input.rs::resolve_abandon_confirmation`).
fn pause_menu(
    mut contexts: EguiContexts,
    mut open: ResMut<PauseMenuOpen>,
    mut pending: ResMut<PendingConfirmation>,
) -> Result {
    if !open.0 {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let frame = egui::Frame::window(&ctx.style_of(ctx.theme()))
        .fill(PANEL_BG)
        .stroke(egui::Stroke::new(1.0, OUTLINE_STROKE));
    egui::Window::new(text::PAUSE_MENU_TITLE)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .frame(frame)
        .show(ctx, |ui| {
            apply_monospace(ui);
            ui.vertical_centered(|ui| {
                if outline_button_auto(ui, text::PAUSE_RESUME_BUTTON, true).clicked() {
                    open.0 = false;
                }
                outline_button_auto(ui, text::PAUSE_SETTINGS_BUTTON, false)
                    .on_hover_text(text::PAUSE_SETTINGS_UNAVAILABLE_HINT);
                if outline_button_auto(ui, text::PAUSE_SAVE_AND_EXIT_BUTTON, true).clicked() {
                    pending.kind = Some(ConfirmationKind::AbandonRun);
                    pending.confirmed = false;
                }
                if outline_button_auto_colored(ui, text::PAUSE_ABANDON_BUTTON, STATE_NEGATIVE, true)
                    .clicked()
                {
                    pending.kind = Some(ConfirmationKind::AbandonRun);
                    pending.confirmed = false;
                }
            });
        });
    Ok(())
}

/// Task 150's shared confirm/cancel dialog — `r`'s reseed-when-touched gate
/// and the pause menu's "Save and exit"/"Abandon without saving" all set
/// `PendingConfirmation::kind` and read this back; this system only ever
/// flips `confirmed` or clears `kind`, never applies the effect itself
/// (TECH_DESIGN.md §3.3, `ui.rs` writes only intents) — `input.rs`'s
/// `reseed_world`/`resolve_abandon_confirmation` own that.
fn confirmation_dialog(
    mut contexts: EguiContexts,
    mut pending: ResMut<PendingConfirmation>,
) -> Result {
    let Some(kind) = pending.kind else {
        return Ok(());
    };
    let (title, body) = match kind {
        ConfirmationKind::ReseedWorld => (text::CONFIRM_RESEED_TITLE, text::CONFIRM_RESEED_BODY),
        ConfirmationKind::AbandonRun => (text::CONFIRM_ABANDON_TITLE, text::CONFIRM_ABANDON_BODY),
    };
    let ctx = contexts.ctx_mut()?;
    let frame = egui::Frame::window(&ctx.style_of(ctx.theme()))
        .fill(PANEL_BG)
        .stroke(egui::Stroke::new(1.0, OUTLINE_STROKE));
    egui::Window::new(title)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 40.0))
        .frame(frame)
        .show(ctx, |ui| {
            apply_monospace(ui);
            ui.label(body);
            ui.horizontal(|ui| {
                // Both current `ConfirmationKind` variants are destructive
                // (reseed discards world state, abandon discards a run), so
                // Confirm gets the negative/destructive register — a future
                // non-destructive kind should switch this to `STATE_POSITIVE`
                // per-kind rather than hardcoding red for every Confirm.
                if outline_button_auto_colored(ui, text::CONFIRM_BUTTON, STATE_NEGATIVE, true)
                    .clicked()
                {
                    pending.confirmed = true;
                }
                if outline_button_auto(ui, text::CANCEL_BUTTON, true).clicked() {
                    *pending = PendingConfirmation::default();
                }
            });
        });
    Ok(())
}

/// Current-objective panel (task 043, GDD §11; restyled task 064 per the
/// sidebar redesign's §5 "narrative accent"): the objective's sentence as an
/// italicized, quoted line in the panel's one deliberate non-monospace
/// accent, followed by a discrete progress indicator instead of a
/// continuous bar. Formats the concrete numbers/labels here (species name,
/// season counts) and hands them to `text.rs`'s parametrized templates,
/// since `text.rs` never reaches into `Objective`/`SimWorld`-derived data on
/// its own (task 034's constraint).
fn objective_panel(
    ui: &mut egui::Ui,
    world: &SimWorld,
    objective: Option<&Objective>,
    index: usize,
    total: usize,
    progress: &ObjectiveProgress,
    season_pulses: u32,
) {
    let Some(objective) = objective else {
        ui.weak(text::NO_OBJECTIVE);
        return;
    };

    if total > 1 {
        ui.weak(text::objective_sequence_position(index, total));
    }

    let description = match *objective {
        Objective::Coexistence {
            min_species,
            min_population,
            ..
        } => text::coexistence_objective_line(min_species, min_population),
        Objective::SurviveIn { species, zone, .. } => {
            text::survive_in_objective_line(&species_label(world, species), text::zone_label(zone))
        }
        Objective::TriggerBloom {
            species,
            population_threshold,
        } => {
            text::trigger_bloom_objective_line(&species_label(world, species), population_threshold)
        }
        Objective::Speciation => text::speciation_objective_line(
            progress
                .speciation_target
                .map(|target| species_label(world, target)),
        ),
        Objective::Homeostasis {
            species,
            min_mean_energy,
            max_mean_energy,
            ..
        } => text::homeostasis_objective_line(
            &species_label(world, species),
            min_mean_energy,
            max_mean_energy,
        ),
        Objective::Tolerance { species, zone, .. } => {
            text::tolerance_objective_line(&species_label(world, species), text::zone_label(zone))
        }
        Objective::WildCoexistence { wild_species, .. } => {
            text::wild_coexistence_objective_line(&species_label(world, wild_species))
        }
        Objective::Rootedness {
            species, terrain, ..
        } => text::rootedness_objective_line(
            &species_label(world, species),
            text::terrain_label(terrain),
        ),
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

    if let Objective::SurviveIn {
        zone: ZoneKind::Toxic,
        ..
    } = *objective
    {
        ui.weak(text::SURVIVE_IN_TOXIC_BORDER_HINT);
    }

    if progress.satisfied {
        ui.weak(text::OBJECTIVE_CLEARED);
        return;
    }

    match *objective {
        Objective::Coexistence { ticks, .. }
        | Objective::SurviveIn { ticks, .. }
        | Objective::Homeostasis { ticks, .. }
        | Objective::Tolerance { ticks, .. }
        | Objective::WildCoexistence { ticks, .. }
        | Objective::Rootedness { ticks, .. } => {
            let (seasons_held, seasons_required) =
                seasons_progress(progress.consecutive_ticks, ticks, season_pulses);
            match season_progress_display(seasons_required) {
                SeasonProgressDisplay::Dots => {
                    dot_row(ui, seasons_held, seasons_required, DotShape::Circle);
                }
                SeasonProgressDisplay::Numeric => {
                    ui.weak(text::sustained_progress_bar_text(
                        seasons_held,
                        seasons_required,
                    ));
                }
            }
        }
        // A bloom is a single triggering event, not a sustained count (see
        // `Objective::TriggerBloom`'s own doc comment) — a state label is
        // the whole story, no indicator of any shape needed.
        Objective::TriggerBloom { .. } => {
            ui.weak(text::BLOOM_NOT_TRIGGERED);
        }
        // Also a one-shot triggering event, not a sustained count — same
        // reasoning as `TriggerBloom` above.
        Objective::Speciation => {
            ui.weak(text::SPECIATION_NOT_TRIGGERED);
        }
    }
}

/// Which shape an objective's season-progress indicator takes, given how
/// many seasons it requires — the pure decision behind `objective_panel`'s
/// dots-vs-text branch, split out so the `SEASON_PROGRESS_DOT_CAP` boundary
/// is unit-testable without an `egui::Ui`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeasonProgressDisplay {
    Dots,
    Numeric,
}

/// Past `SEASON_PROGRESS_DOT_CAP`, a row of dots would either grow wider than
/// `HUD_WIDTH` or shrink unreadably small — a compact "X / Y seasons" readout
/// degrades gracefully instead.
fn season_progress_display(seasons_required: u32) -> SeasonProgressDisplay {
    if seasons_required <= SEASON_PROGRESS_DOT_CAP {
        SeasonProgressDisplay::Dots
    } else {
        SeasonProgressDisplay::Numeric
    }
}

/// Converts a sustained objective's tick counts to whole seasons for display
/// (task 049, unit moved from era to season by task 135): `seasons_held`
/// floors (a season in progress doesn't count until complete),
/// `seasons_required` ceils (a requirement of, say, 70 ticks with a 25-tick
/// season still genuinely needs 3 full seasons, not 2) — chosen so the
/// displayed count never claims "done" before `progress.satisfied` actually
/// flips.
fn seasons_progress(consecutive_ticks: u32, required_ticks: u32, season_pulses: u32) -> (u32, u32) {
    if season_pulses == 0 {
        return (0, 0);
    }
    (
        consecutive_ticks / season_pulses,
        required_ticks.div_ceil(season_pulses),
    )
}

/// Low-contrast divider between HUD sections (task 064's redesign,
/// replacing `group_frame`'s bordered boxes): a thin painted line rather
/// than `egui::Separator`, so its color is independent of `Visuals`'
/// widget-stroke styling and stays a deliberately faint hairline instead of
/// picking up whatever contrast egui's default separator uses elsewhere.
const HAIRLINE_COLOR: egui::Color32 = egui::Color32::from_rgb(35, 38, 46);

/// Switches every `TextStyle`'s font family to monospace, scoped to `ui`
/// and its children only (same technique the HUD panel uses inline, above)
/// — §2's "monospace panel-wide" applies to every surface, not just the
/// HUD, so `screens.rs`/`menu.rs` call this on their own top-level `Ui`
/// (task 182) since neither inherits the HUD panel's own override.
pub(crate) fn apply_monospace(ui: &mut egui::Ui) {
    for font_id in ui.style_mut().text_styles.values_mut() {
        font_id.family = egui::FontFamily::Monospace;
    }
}

pub(crate) fn hairline(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    let rect = ui.available_rect_before_wrap();
    let y = ui.cursor().top();
    ui.painter()
        .hline(rect.x_range(), y, egui::Stroke::new(0.5, HAIRLINE_COLOR));
    ui.add_space(6.0);
}

/// Which shape `dot_row` paints per slot — `Tick` for the action budget
/// (task 151: squared off from the redesign mockup's original rounded-rect
/// ticks, matching the rest of the pixel-grain HUD chrome), `Circle` for
/// objective era-progress (matching its own mockup's hollow/filled dots —
/// a deliberately distinct round shape from `Tick`, not chrome rounding,
/// so left untouched by task 151's corner-squaring).
/// Two call sites, two mockup-specified shapes, kept as one function rather
/// than two near-duplicates.
#[derive(Clone, Copy)]
enum DotShape {
    Tick,
    Circle,
}

const DOT_EMPTY_COLOR: egui::Color32 = egui::Color32::from_gray(60);
const DOT_SIZE: f32 = 8.0;
const DOT_GAP: f32 = 5.0;

/// HUD chrome tokens transcribed from `VISUAL_STYLE_GUIDE.md` §3 (task 180)
/// — shared by the action-mode buttons and the time-control outline
/// buttons below, the two "button registers" §6 describes. `OUTLINE_STROKE`
/// is `pub(crate)` (task 182): `screens.rs`/`menu.rs` reuse it for their
/// own outline buttons, having no HUD `Ui` of their own to inherit it from.
pub(crate) const OUTLINE_STROKE: egui::Color32 = egui::Color32::from_rgb(0x3a, 0x40, 0x48);
const CHROME_SELECTED_FILL: egui::Color32 = egui::Color32::from_rgb(0x16, 0x24, 0x1a);
const CHROME_SELECTED_STROKE: egui::Color32 = egui::Color32::from_rgb(0x7f, 0xae, 0x6a);

/// Shared exact-hex tokens from `VISUAL_STYLE_GUIDE.md` §3.1/3.2 (task 182),
/// exposed at crate visibility for `screens.rs`/`menu.rs`.
pub(crate) const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0x1c, 0x22, 0x29);
/// Same token as `CHROME_SELECTED_STROKE` above, under the guide's own §3.2
/// name.
pub(crate) const STATE_POSITIVE: egui::Color32 = CHROME_SELECTED_STROKE;
pub(crate) const STATE_NEGATIVE: egui::Color32 = egui::Color32::from_rgb(0xc9, 0x6a, 0x5c);
/// Icon ink, unselected/selected (`pixel-full-scene.svg:6798` vs.
/// `6801/6804/6807`) — gray until a mode-select button is armed, then the
/// same amber the map's organism ink uses (§3.2), never a per-action hue.
const ICON_INK_UNSELECTED: egui::Color32 = egui::Color32::from_rgb(0x9a, 0xa0, 0xa6);
/// Also the notebook's Catalog icon ink (task 181) and, per §3.2, the exact
/// token for organism ink anywhere it's needed (`ORGANISM_INK` in the
/// guide's own naming) — one amber value for every use, not a separate
/// alias per surface.
pub(crate) const ICON_INK_SELECTED: egui::Color32 = egui::Color32::from_rgb(0xe0, 0xc9, 0x9a);
/// Muted-gray section-label ink (§2, §3.1) — uppercase headers distinct
/// from plain body/heading text.
const SECTION_LABEL_COLOR: egui::Color32 = egui::Color32::from_rgb(0x7d, 0x84, 0x8a);

/// Small, uppercase, muted-gray section header (task 180), replacing plain
/// `ui.strong(...)` at the HUD's section boundaries (`TIME`/`INTERVIENI`/
/// `BIOSPHERE` etc. in `pixel-full-scene.svg:6787,6796,6814`). The mockup
/// also letter-spaces these labels; egui has no direct letter-spacing knob
/// and the source itself is a design reference, not a pixel spec (same
/// latitude every other pixel-grain task in this queue has taken), so this
/// only reproduces the uppercase transform, small size, and muted color.
pub(crate) fn section_header(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.label(
        egui::RichText::new(label.to_uppercase())
            .small()
            .color(SECTION_LABEL_COLOR),
    )
}

/// Paints one 4×4-point block at `origin + offset` — the mockup's action-
/// icon unit (`pixel-full-scene.svg`'s `<rect width="4" height="4">`
/// blocks). `action_icon_row` calls this per icon; the offsets themselves
/// (`SEED_ICON_BLOCKS` etc. below) are transcribed verbatim from the
/// mockup's four button patterns (`pixel-full-scene.svg:6798,6801,6804,
/// 6807`) rather than procedurally generated, since — unlike the
/// metabolism shapes below — no mask function for these four exists
/// anywhere else to reuse.
const ACTION_ICON_BLOCK: f32 = 4.0;

fn paint_action_blocks(
    painter: &egui::Painter,
    origin: egui::Pos2,
    offsets: &[(f32, f32)],
    color: egui::Color32,
) {
    for &(dx, dy) in offsets {
        let block = egui::Rect::from_min_size(
            origin + egui::vec2(dx, dy),
            egui::vec2(ACTION_ICON_BLOCK, ACTION_ICON_BLOCK),
        );
        painter.rect_filled(block, 0.0, color);
    }
}

/// A plus-cluster, `ActionMode::Seed` — `pixel-full-scene.svg:6798`
/// (offsets relative to the button rect's own top-left corner, which is
/// identical across all four buttons since they share `ACTION_BUTTON_SIZE`).
const SEED_ICON_BLOCKS: [(f32, f32); 8] = [
    (26.0, 16.0),
    (26.0, 20.0),
    (22.0, 24.0),
    (30.0, 24.0),
    (26.0, 24.0),
    (26.0, 28.0),
    (22.0, 32.0),
    (30.0, 32.0),
];

/// A jagged bolt, `ActionMode::Stress` — `pixel-full-scene.svg:6801`.
const STRESS_ICON_BLOCKS: [(f32, f32); 9] = [
    (30.0, 16.0),
    (26.0, 20.0),
    (30.0, 20.0),
    (22.0, 24.0),
    (26.0, 24.0),
    (30.0, 24.0),
    (26.0, 28.0),
    (22.0, 32.0),
    (26.0, 32.0),
];

/// An X/skull cluster, `ActionMode::Cull` — `pixel-full-scene.svg:6804`.
const CULL_ICON_BLOCKS: [(f32, f32); 12] = [
    (22.0, 16.0),
    (26.0, 16.0),
    (30.0, 16.0),
    (18.0, 20.0),
    (34.0, 20.0),
    (18.0, 24.0),
    (26.0, 24.0),
    (34.0, 24.0),
    (22.0, 28.0),
    (30.0, 28.0),
    (22.0, 32.0),
    (30.0, 32.0),
];

/// A flask cluster, `ActionMode::Splice` — `pixel-full-scene.svg:6807`.
const SPLICE_ICON_BLOCKS: [(f32, f32); 11] = [
    (26.0, 16.0),
    (26.0, 20.0),
    (22.0, 24.0),
    (30.0, 24.0),
    (18.0, 28.0),
    (22.0, 28.0),
    (26.0, 28.0),
    (30.0, 28.0),
    (34.0, 28.0),
    (18.0, 32.0),
    (34.0, 32.0),
];

fn action_icon_blocks(action: ActionMode) -> &'static [(f32, f32)] {
    match action {
        ActionMode::Seed => &SEED_ICON_BLOCKS,
        ActionMode::Stress => &STRESS_ICON_BLOCKS,
        ActionMode::Cull => &CULL_ICON_BLOCKS,
        ActionMode::Splice => &SPLICE_ICON_BLOCKS,
    }
}

/// Paints a metabolism's shared block-pattern shape (`render::metabolism_mask`,
/// the same geometry the map's sprite masks use) inside `rect` (task 180's
/// Biosphere row — species identity goes text-only, the icon carries
/// metabolism, never a per-species hue). Sampled at `BIOSPHERE_ICON_SAMPLE_GRID`,
/// not the map's own coarser `SHAPE_BLOCK_GRID` — see that constant's doc
/// comment for why.
/// Sampling density for `paint_metabolism_icon`, deliberately **higher**
/// than the map's `SHAPE_BLOCK_GRID` (task 180 fix, 2026-08-30 playtest
/// finding): at `BIOSPHERE_ICON_SIZE` (`12.0`), the map's own block
/// coarseness reads as a blur/plain blob at this much smaller size — a
/// regression from the round `●` bullet this icon replaced. This samples
/// the *same* mask functions (`metabolism_mask`) more finely, not a second
/// geometry definition — the map's own coarseness stays untouched, driven
/// by its own texture/cell size, not this constant.
const BIOSPHERE_ICON_SAMPLE_GRID: u32 = 16;

/// `pub(crate)`: task 181 reuses this for the notebook's Species Catalog
/// icon — same shared-icon-painter requirement, one implementation.
pub(crate) fn paint_metabolism_icon(
    painter: &egui::Painter,
    rect: egui::Rect,
    metabolism: Metabolism,
    color: egui::Color32,
) {
    let mask = metabolism_mask(metabolism);
    let blocks = BIOSPHERE_ICON_SAMPLE_GRID;
    let block_size = rect.size() / blocks as f32;
    for row in 0..blocks {
        for col in 0..blocks {
            let nx = (col as f32 + 0.5) / blocks as f32 * 2.0 - 1.0;
            let ny = (row as f32 + 0.5) / blocks as f32 * 2.0 - 1.0;
            if mask(nx, ny) {
                let min =
                    rect.min + egui::vec2(col as f32 * block_size.x, row as f32 * block_size.y);
                painter.rect_filled(egui::Rect::from_min_size(min, block_size), 0.0, color);
            }
        }
    }
}

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
                    STATE_POSITIVE
                } else {
                    DOT_EMPTY_COLOR
                };
                painter.rect_filled(tick_rect, 0.0, color);
            }
            DotShape::Circle => {
                let center = egui::pos2(x + DOT_SIZE / 2.0, rect.center().y);
                let radius = DOT_SIZE / 2.0;
                if is_filled {
                    painter.circle_filled(center, radius, STATE_POSITIVE);
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
/// Marks a `Splice`-synthesised species in the Seed Palette (task 147) —
/// visible without opening the notebook, per the design doc's own framing
/// ("the HUD must anticipate this"). Only the glyph lives here (task 034:
/// player-facing copy belongs in `text.rs`, the origin *word* is
/// `text::species_origin_label`, used by the Catalog instead).
const SYNTHESISED_MARKER: &str = "⚗";

fn species_row(
    ui: &mut egui::Ui,
    species: SpeciesId,
    world: &SimWorld,
    config: &SimConfig,
    selected: &mut SpeciesId,
) {
    let is_selected = *selected == species;
    let source = &world.species[species.0 as usize];
    let metabolism = source.metabolism;
    let marker = if world.species_origin(species) == SpeciesOrigin::Synthesised {
        format!("{SYNTHESISED_MARKER} ")
    } else {
        String::new()
    };
    let text = format!(
        "{marker}{} {}",
        metabolism_glyph(metabolism),
        species_label(world, species)
    );
    if ui.selectable_label(is_selected, text).clicked() {
        *selected = species;
    }
    // Task 152: an abbreviated metabolism/temperature-fit preview so the
    // player can judge a species' fit without opening the notebook —
    // reuses `notebook::temperature_label`'s own band thresholds
    // (`text::band_label`) rather than re-deriving them.
    let temp_label = text::band_label(
        source.temp_optimum,
        config.environment.ambient_temperature,
        config.environment.source_temperature,
        ["cold", "temperate", "hot"],
    );
    ui.weak(text::species_row_subtext(metabolism, temp_label));
}

/// Fixed size for the time-control outline buttons (task 180) —
/// `pixel-full-scene.svg:6789/6791/6793`'s `76×20` boxes.
const TIME_BUTTON_SIZE: egui::Vec2 = egui::vec2(76.0, 20.0);

/// An outline-chrome button (task 180, §6's "plain action" register): no
/// fill, `#3a4048` stroke, text-only — filled with the mode-select
/// green pair only while `active` (continuous-advance armed, the notebook
/// open), matching the mockup's own framing that these buttons stay
/// outline "until an active state applies a distinct fill" (the Notebook
/// button's separate unseen-observation badge, painted by the caller, is
/// unaffected by this). A disabled button dims rather than disappears,
/// consistent with `add_enabled_ui`'s convention elsewhere in this file.
fn outline_button(ui: &mut egui::Ui, label: &str, active: bool, enabled: bool) -> egui::Response {
    outline_button_sized(ui, label, TIME_BUTTON_SIZE, active, enabled)
}

/// [`outline_button`]'s painting, factored out to take an explicit `size`
/// instead of assuming the HUD's fixed `TIME_BUTTON_SIZE` (task 182) — the
/// interstitial/menu screens' buttons carry longer labels ("Return to
/// menu") that don't fit that box.
fn outline_button_sized(
    ui: &mut egui::Ui,
    label: &str,
    size: egui::Vec2,
    active: bool,
    enabled: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let dim = |color: egui::Color32| {
        if enabled {
            color
        } else {
            color.gamma_multiply(0.4)
        }
    };
    let painter = ui.painter();
    if active {
        painter.rect_filled(rect, 0.0, dim(CHROME_SELECTED_FILL));
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, dim(CHROME_SELECTED_STROKE)),
            egui::StrokeKind::Outside,
        );
    } else {
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, dim(OUTLINE_STROKE)),
            egui::StrokeKind::Outside,
        );
    }
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(11.0),
        dim(egui::Color32::from_rgb(0xc3, 0xc9, 0xcf)),
    );
    response
}

/// Padding around the label for [`outline_button_auto`]'s self-sized box.
const AUTO_BUTTON_PADDING: egui::Vec2 = egui::vec2(16.0, 10.0);

/// An outline-chrome button sized to its own label (task 182) — the
/// interstitial screens' and main menu's buttons ("Continue", "Retry",
/// "Return to menu", ...) vary too much in length to share `outline_button`'s
/// fixed HUD box. Always the plain "no active state" register: none of
/// these surfaces have a mode-select concept.
pub(crate) fn outline_button_auto(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    let font = egui::FontId::monospace(11.0);
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        font,
        egui::Color32::WHITE, // discarded — outline_button_sized repaints the text itself
    );
    let size = galley.size() + AUTO_BUTTON_PADDING * 2.0;
    outline_button_sized(ui, label, size, false, enabled)
}

/// [`outline_button_auto`], but with an explicit stroke/label color instead
/// of the neutral `OUTLINE_STROKE` register (task 183) — used where a
/// button's chrome itself must carry state (destructive vs. safe), not just
/// its label color, e.g. the pause menu's "Abandon" and the confirmation
/// dialog's Confirm action.
pub(crate) fn outline_button_auto_colored(
    ui: &mut egui::Ui,
    label: &str,
    color: egui::Color32,
    enabled: bool,
) -> egui::Response {
    let font = egui::FontId::monospace(11.0);
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        font,
        egui::Color32::WHITE, // discarded — repainted below
    );
    let size = galley.size() + AUTO_BUTTON_PADDING * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let dim = |c: egui::Color32| {
        if enabled {
            c
        } else {
            c.gamma_multiply(0.4)
        }
    };
    let painter = ui.painter();
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, dim(color)),
        egui::StrokeKind::Outside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(11.0),
        dim(color),
    );
    response
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
    continuous: &mut ContinuousAdvance,
) {
    // Task 152: continuous advance and the manual controls are mutually
    // exclusive — "one in-flight advance mechanism at a time", same rule
    // `advancing` (a `space`-triggered era block) already enforces on its
    // own.
    let blocked = advancing || continuous.0;
    ui.horizontal(|ui| {
        let tick_response = outline_button(ui, text::TICK_BUTTON_LABEL, false, !blocked);
        if tick_response.clicked() && !blocked {
            intents.advance_tick = true;
        }
        tick_response.on_hover_text(if advancing {
            format!(
                "{}{}",
                text::TICK_BUTTON_TOOLTIP,
                text::ADVANCING_DISABLED_HINT
            )
        } else if continuous.0 {
            format!(
                "{}{}",
                text::TICK_BUTTON_TOOLTIP,
                text::CONTINUOUS_ADVANCING_DISABLED_HINT
            )
        } else {
            text::TICK_BUTTON_TOOLTIP.to_string()
        });

        let era_response = outline_button(ui, text::ERA_BUTTON_LABEL, false, !blocked);
        if era_response.clicked() && !blocked {
            intents.advance_era = true;
        }
        era_response.on_hover_text(if advancing {
            format!(
                "{}{}",
                text::ERA_BUTTON_TOOLTIP,
                text::ADVANCING_DISABLED_HINT
            )
        } else if continuous.0 {
            format!(
                "{}{}",
                text::ERA_BUTTON_TOOLTIP,
                text::CONTINUOUS_ADVANCING_DISABLED_HINT
            )
        } else {
            text::ERA_BUTTON_TOOLTIP.to_string()
        });

        let continuous_response = outline_button(
            ui,
            text::CONTINUOUS_ADVANCE_BUTTON_LABEL,
            continuous.0,
            !advancing,
        );
        if continuous_response.clicked() && !advancing {
            continuous.0 = !continuous.0;
        }
        continuous_response.on_hover_text(text::CONTINUOUS_ADVANCE_BUTTON_TOOLTIP);

        let notebook_response =
            outline_button(ui, text::NOTEBOOK_BUTTON_LABEL, notebook_open, true);
        if notebook_response.clicked() {
            intents.toggle_notebook = true;
        }
        notebook_response.on_hover_text(text::NOTEBOOK_BUTTON_TOOLTIP);
    });
}

/// Fixed square size for the action-mode buttons (task 180) —
/// `pixel-full-scene.svg:6797`'s `52×52` boxes.
const ACTION_BUTTON_SIZE: f32 = 52.0;

/// The four `ActionMode` options as a row of square icon buttons (task
/// 030, box chrome/block icons task 180). Still single-selection,
/// immediate-mode — a custom-painted rect plays the role `selectable_label`
/// used to, since neither of egui's default button visuals matches the
/// mockup's box chrome (§6): unselected is an outline-only box with gray
/// ink, selected is a filled dark-green box with amber ink
/// (`pixel-full-scene.svg:6797-6807`). A hover tooltip still carries the
/// name, cost, and a one-line description.
fn action_icon_row(
    ui: &mut egui::Ui,
    selected_action: &mut SelectedAction,
    config: &SimConfig,
    run_progress: &RunProgress,
    view_mode: MapViewMode,
    splice_confirmed_tags: usize,
) {
    ui.horizontal(|ui| {
        for action_mode in [
            ActionMode::Seed,
            ActionMode::Stress,
            ActionMode::Cull,
            ActionMode::Splice,
        ] {
            let cost = action_cost(action_mode, config, run_progress);
            // Stress/Cull need per-organism precision Overview's real-density
            // coloring (task 076/139) doesn't preserve, so
            // they're disabled outside Detail (task 077) — same
            // `add_enabled_ui` pattern `splice_panel` already uses for its
            // tag-cap gating, rather than leaving a clickable button that
            // silently does nothing.
            let detail_only = matches!(action_mode, ActionMode::Stress | ActionMode::Cull);
            let enabled = !detail_only || view_mode == MapViewMode::Detail;
            let selected = selected_action.0 == Some(action_mode);

            let response = ui
                .add_enabled_ui(enabled, |ui| {
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ACTION_BUTTON_SIZE, ACTION_BUTTON_SIZE),
                        egui::Sense::click(),
                    );
                    // `add_enabled_ui` blocks the click but paints nothing
                    // itself since this button is custom-painted, not a
                    // standard widget reading `ui.visuals()` — dim
                    // explicitly so a disabled (Stress/Cull outside
                    // Detail) button still reads as disabled, matching the
                    // grayed-out convention `outline_button` uses below.
                    let dim = |color: egui::Color32| {
                        if enabled {
                            color
                        } else {
                            color.gamma_multiply(0.4)
                        }
                    };
                    let painter = ui.painter();
                    if selected {
                        painter.rect_filled(rect, 0.0, dim(CHROME_SELECTED_FILL));
                        painter.rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(1.0, dim(CHROME_SELECTED_STROKE)),
                            egui::StrokeKind::Outside,
                        );
                    } else {
                        painter.rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(1.0, dim(OUTLINE_STROKE)),
                            egui::StrokeKind::Outside,
                        );
                    }
                    let icon_color = dim(if selected {
                        ICON_INK_SELECTED
                    } else {
                        ICON_INK_UNSELECTED
                    });
                    paint_action_blocks(
                        painter,
                        rect.min,
                        action_icon_blocks(action_mode),
                        icon_color,
                    );
                    response
                })
                .inner;
            if response.clicked() {
                selected_action.0 = if selected {
                    // Task 149's minimal deselect seam: clicking the
                    // already-armed action again disarms it, the only way
                    // today to reach "no action armed" (task 150 owns the
                    // full Esc-cascade/explicit-none scheme).
                    None
                } else {
                    Some(action_mode)
                };
            }
            let mut tooltip = if enabled {
                text::action_tooltip(action_mode, cost)
            } else {
                format!(
                    "{}{}",
                    text::action_tooltip(action_mode, cost),
                    text::DETAIL_MODE_ONLY_HINT
                )
            };
            // Task 152's mutation-level badge: a small corner marker on
            // Splice reflecting how many of this world's tags are actually
            // usable in `splice_panel` today (only confirmed ones are
            // offered, task 147) — the real restriction, not an invented
            // placeholder tier. Painted directly on the button rect rather
            // than as a separate row item, so it reads as a badge on the
            // icon, not a fifth control.
            if action_mode == ActionMode::Splice {
                let unlocked = splice_confirmed_tags > 0;
                let badge_color = if unlocked {
                    STATE_POSITIVE
                } else {
                    DOT_EMPTY_COLOR
                };
                let badge_center = response.rect.right_top() + egui::vec2(-3.0, 3.0);
                ui.painter().circle_filled(badge_center, 3.0, badge_color);
                tooltip.push_str(&text::splice_tier_hint(splice_confirmed_tags));
            }
            response.on_hover_text(tooltip);
        }
    });
}

/// Three-way axis toggle (task 145) shown only while `Stress` is the active
/// action — same slot in the roster, same icon, same cost; the axis choice
/// lives *inside* the action rather than adding a fifth `ActionMode`, per
/// `abiogenesis-actions.md`'s explicit framing.
const STRESS_AXES: [StressAxis; 3] = [
    StressAxis::Temperature,
    StressAxis::Light,
    StressAxis::Toxicity,
];

fn stress_axis_row(ui: &mut egui::Ui, selected: &mut SelectedStressAxis) {
    ui.horizontal(|ui| {
        for axis in STRESS_AXES {
            if ui
                .selectable_label(selected.0 == axis, text::stress_axis_name(axis))
                .clicked()
            {
                selected.0 = axis;
            }
        }
    });
}

/// Action-point cost for one `ActionMode`, read from `SimConfig` so the
/// icon tooltips (task 030) never hardcode a number that could drift from
/// `ActionCosts` (GDD §5.9). `Splice`'s cost goes through
/// `RunProgress::splice_cost` (task 109) rather than reading
/// `action_costs.splice` directly, so the tooltip reflects the same
/// energy-gated upgraded tier `input.rs::apply_splice` actually charges.
fn action_cost(mode: ActionMode, config: &SimConfig, run_progress: &RunProgress) -> u32 {
    match mode {
        ActionMode::Seed => config.time.action_costs.seed,
        ActionMode::Stress => config.time.action_costs.stress,
        ActionMode::Cull => config.time.action_costs.cull,
        ActionMode::Splice => run_progress.splice_cost(config),
    }
}

/// The `Splice` editor (task 025, GDD §6 — "the most powerful and most
/// expensive experimental tool"): pick a source species and one edit,
/// staged in `SpliceDraft` until "Apply" is pressed. Read-only against
/// `SimWorld`, same as `hud_panel` — the actual mutation happens in
/// `input.rs`'s `apply_splice`, reading `apply_requested` as an intent.
fn splice_panel(
    ui: &mut egui::Ui,
    world: &SimWorld,
    draft: &mut SpliceDraft,
    knowledge: &MatrixKnowledge,
) {
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
                // Task 147: only tags the player has already confirmed in
                // this world's matrix are offered — "you can't synthesise
                // what you haven't decoded." Sourced from `world.active_tags`
                // (this run's drawn subset), which structurally excludes any
                // future xenotrait pool (task 168) as long as that pool
                // stays a separate list `active_tags` never draws from — the
                // guard task 168 must respect, not a runtime check against a
                // pool that doesn't exist yet.
                for (i, &tag) in world.active_tags.iter().enumerate() {
                    let slot = TagSlot(i as u8);
                    if !knowledge.is_tag_confirmed(slot) {
                        continue;
                    }
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
                // Same confirmed-only filter as the SwapTag arm above.
                for (i, &candidate) in world.active_tags.iter().enumerate() {
                    let slot = TagSlot(i as u8);
                    if species.tags.contains(&slot) || !knowledge.is_tag_confirmed(slot) {
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
        if let Some(population) = cell.population {
            let entry = &mut totals[population.species.0 as usize];
            entry.0 += population.count as usize;
            entry.1 += population.energy;
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
/// Pure population-count delta since the last completed era (task 120):
/// `previous` is `None` for a species with no prior-era snapshot yet (its
/// first era with any population), in which case there's nothing to diff
/// against and this returns `None` rather than an implicit "+N from zero" —
/// same baseline-handling shape as `classify_trend`'s `None` case just
/// below.
fn population_delta(previous: Option<usize>, current: usize) -> Option<i64> {
    previous.map(|previous| current as i64 - previous as i64)
}

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
    /// Population count as of the last completed era, per species (task
    /// 120) — separate from `previous_avg_energy`: the delta this backs is
    /// a population-*count* reading, deliberately independent of the
    /// energy-based trend arrow above (see `population_delta_for`'s doc).
    previous_population: Vec<Option<usize>>,
    /// This era's population delta versus `previous_population`, or `None`
    /// for a species with no prior-era snapshot yet (its first era with any
    /// population) — shown as no delta rather than an implicit "+N from
    /// zero", which would misrepresent a species just appearing as a
    /// population explosion.
    current_population_delta: Vec<Option<i64>>,
}

impl PopulationTrends {
    pub fn trend_for(&self, species: SpeciesId) -> PopulationTrend {
        self.current
            .get(species.0 as usize)
            .copied()
            .unwrap_or(PopulationTrend::Stable)
    }

    /// Population-count delta since the last completed era (task 120,
    /// `redesign/abiogenesis-hud-notebook.md` §4). This is **not** the same
    /// signal as `trend_for`'s arrow — that one is energy-based (task 063)
    /// — so the two can legitimately disagree on a given row; that's by
    /// design, not a bug (see task 120's file). `None` means no prior-era
    /// snapshot exists yet (the species' first era with a nonzero
    /// population): the HUD shows no delta rather than a misleading jump
    /// from an implicit zero baseline.
    pub fn population_delta_for(&self, species: SpeciesId) -> Option<i64> {
        self.current_population_delta
            .get(species.0 as usize)
            .copied()
            .flatten()
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
        trends.previous_population.resize(species_count, None);
        trends.current_population_delta.resize(species_count, None);
    }
    for (species, population, avg_energy) in species_stats(&world) {
        let idx = species.0 as usize;
        trends.current[idx] = classify_trend(
            trends.previous_avg_energy[idx],
            avg_energy,
            config.energy.trend_epsilon,
        );
        trends.previous_avg_energy[idx] = Some(avg_energy);

        trends.current_population_delta[idx] =
            population_delta(trends.previous_population[idx], population);
        trends.previous_population[idx] = Some(population);
    }
}

/// Cell offsets for [`paint_trend_arrow`]'s painted 3x3 block glyph (task
/// 184) — replaces the raw Unicode `▲`/`▼`/`▬` `VISUAL_STYLE_GUIDE.md` §6
/// forbids (icons are painted blocks, never font glyphs) with the same
/// procedural technique the action-mode icons use, just at inline-text
/// scale instead of the 40x40 button icon scale.
const TREND_ARROW_BLOCK: f32 = 3.0;

fn trend_arrow_cells(trend: PopulationTrend) -> &'static [(f32, f32)] {
    const RISING: [(f32, f32); 4] = [(3.0, 0.0), (0.0, 3.0), (3.0, 3.0), (6.0, 3.0)];
    const FALLING: [(f32, f32); 4] = [(0.0, 0.0), (3.0, 0.0), (6.0, 0.0), (3.0, 3.0)];
    const STABLE: [(f32, f32); 3] = [(0.0, 1.5), (3.0, 1.5), (6.0, 1.5)];
    match trend {
        PopulationTrend::Rising => &RISING,
        PopulationTrend::Falling => &FALLING,
        PopulationTrend::Stable => &STABLE,
    }
}

/// Paints a `PopulationTrend` as a small block-pattern arrow instead of a
/// Unicode glyph — the sidebar's biosphere rows and the hover tooltip both
/// call this at their one trend indicator each.
fn paint_trend_arrow(ui: &mut egui::Ui, trend: PopulationTrend, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(TREND_ARROW_BLOCK * 3.0, TREND_ARROW_BLOCK * 2.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    for &(dx, dy) in trend_arrow_cells(trend) {
        let block = egui::Rect::from_min_size(
            rect.min + egui::vec2(dx, dy),
            egui::vec2(TREND_ARROW_BLOCK, TREND_ARROW_BLOCK),
        );
        painter.rect_filled(block, 0.0, color);
    }
}

/// Color for a `PopulationTrend`, same mockup convention: green rising, red
/// falling, gray stable — gray rather than either the positive/negative edge
/// colors `notebook.rs` uses for confirmed matrix effects, since "stable" is
/// neutral information, not the absence of one.
fn trend_color(trend: PopulationTrend) -> egui::Color32 {
    match trend {
        PopulationTrend::Rising => STATE_POSITIVE,
        PopulationTrend::Falling => STATE_NEGATIVE,
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

/// `(current, total)` ticks for the season readout (task 117, unit moved
/// from era to season by task 135): `total` is this season's own length
/// (`season_pulses_for`, already accounting for world 0's shortened
/// onboarding seasons, task 082).
///
/// `remaining == 0` is treated as "no progress yet in the season currently
/// shown," not "this season just finished" — `tick_and_complete_season`
/// (`sim.rs:991-1006`) increments `world.season` in the *same* call that
/// decrements `remaining` to `0`, atomically, so by the time any frame
/// renders `remaining == 0`, `world.season` has already moved on to the new
/// season. There is no observable frame where `remaining == 0` refers to
/// the season that just completed — every such frame is really "0 ticks
/// played in the season we're now looking at," whether that's because it
/// hasn't started (`Observing`, before the next `space`/`n`) or because
/// `SeasonProgress` was never started for this world yet. Found live
/// 2026-08-13 (twice): an `EraState`-based gate here was wrong in the
/// *other* direction — it hid real progress made via `n`
/// (`input.rs::single_tick`), which advances `remaining` while
/// deliberately staying in `Observing` (no `EraState` transition, by
/// design, for fine-grained single-tick observation). `EraState` doesn't
/// distinguish "stale" from "real" here at all; `remaining == 0` does.
fn season_readout_values(
    world_index: u32,
    season: u32,
    remaining: u32,
    config: &SimConfig,
) -> (u32, u32) {
    let total = season_pulses_for(world_index, season, config);
    let current = if remaining == 0 {
        0
    } else {
        total.saturating_sub(remaining)
    };
    (current, total)
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

    /// Task 117: at the instant an era starts (`SeasonProgress::start` sets
    /// `remaining` to the season's full length), the readout must show
    /// `0/{total}`, matching the semantics `tick_and_complete_season` already
    /// gives `remaining == 0` (see `season_readout_values`'s own doc comment).
    #[test]
    fn season_readout_values_shows_zero_progress_at_season_start() {
        let config = SimConfig::default();
        let total = season_pulses_for(0, 5, &config);
        let (current, shown_total) = season_readout_values(0, 5, total, &config);
        assert_eq!(current, 0);
        assert_eq!(shown_total, total);
    }

    /// Mid-era: `current` counts up as `remaining` counts down. Must hold
    /// whether progress came from `space` (`EraState::Advancing`) or `n`
    /// (`input.rs::single_tick`, which deliberately stays in `Observing`) —
    /// this function takes no `EraState` input at all, by design (found
    /// live 2026-08-13: an earlier version gated on `EraState::Advancing`
    /// and hid real progress made via `n`).
    #[test]
    fn season_readout_values_counts_up_mid_season() {
        let config = SimConfig::default();
        let total = season_pulses_for(0, 5, &config);
        let (current, _) = season_readout_values(0, 5, total / 2, &config);
        assert_eq!(current, total - total / 2);
    }

    /// Last tick before completion: `remaining == 1` must show
    /// `total - 1`, not off by one in either direction.
    #[test]
    fn season_readout_values_is_correct_on_the_last_tick_before_completion() {
        let config = SimConfig::default();
        let total = season_pulses_for(0, 5, &config);
        let (current, _) = season_readout_values(0, 5, 1, &config);
        assert_eq!(current, total - 1);
    }

    /// `remaining == 0` always reads as "0 progress in the era currently
    /// shown," never "the previous era, fully done" — `tick_and_complete_season`
    /// increments `world.era` atomically in the same call that drives
    /// `remaining` to `0` (`sim.rs:991-1006`), so no observable frame ever
    /// pairs `remaining == 0` with the era number it just finished. Found
    /// live 2026-08-13: without this, a fresh era showed e.g. "Era 2 · tick
    /// 8/8" the instant it began, reusing era 1's fully-depleted `remaining`.
    #[test]
    fn season_readout_values_shows_zero_progress_when_remaining_is_zero() {
        let config = SimConfig::default();
        let total = season_pulses_for(0, 5, &config);
        let (current, shown_total) = season_readout_values(0, 5, 0, &config);
        assert_eq!(current, 0);
        assert_eq!(shown_total, total);
    }

    /// The readout composes `season_pulses_for` correctly for both world 0's
    /// shortened onboarding eras (task 082) and the standard length —
    /// mirrors `season_pulses_for`'s own coverage, just asserting this
    /// function's composition of it, not re-deriving the thresholds.
    #[test]
    fn season_readout_values_uses_onboarding_length_for_world_zeros_opening_seasons() {
        let config = SimConfig::default();
        let onboarding_total = season_pulses_for(0, 0, &config);
        let standard_total = season_pulses_for(0, config.time.onboarding_seasons, &config);
        assert_eq!(
            season_readout_values(0, 0, onboarding_total, &config).1,
            onboarding_total
        );
        assert_eq!(
            season_readout_values(0, config.time.onboarding_seasons, standard_total, &config).1,
            standard_total
        );
        assert_ne!(
            onboarding_total, standard_total,
            "the two eras compared must actually use different lengths, or this test can't tell them apart"
        );
    }

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
            cause: abiogenesis::sim::DeathCause::Starvation,
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
    fn season_progress_display_shows_dots_at_and_under_the_cap() {
        assert_eq!(
            season_progress_display(SEASON_PROGRESS_DOT_CAP),
            SeasonProgressDisplay::Dots
        );
        assert_eq!(season_progress_display(1), SeasonProgressDisplay::Dots);
    }

    #[test]
    fn season_progress_display_falls_back_to_numeric_past_the_cap() {
        assert_eq!(
            season_progress_display(SEASON_PROGRESS_DOT_CAP + 1),
            SeasonProgressDisplay::Numeric
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
    fn population_delta_computed_correctly_across_consecutive_eras() {
        // Era 1 establishes a baseline of 10, era 2 reads 14 against it.
        let baseline = population_delta(None, 10);
        assert_eq!(baseline, None);
        let after_growth = population_delta(Some(10), 14);
        assert_eq!(after_growth, Some(4));
        // Era 3 crashes from 14 down to 9.
        let after_crash = population_delta(Some(14), 9);
        assert_eq!(after_crash, Some(-5));
        // Era 4 holds exactly steady.
        let after_steady = population_delta(Some(9), 9);
        assert_eq!(after_steady, Some(0));
    }

    #[test]
    fn population_delta_first_era_with_population_shows_no_delta() {
        // A species with no prior-era snapshot (its first era with any
        // population) must not report a misleading "+N from an implicit
        // zero" jump.
        assert_eq!(population_delta(None, 1), None);
        assert_eq!(population_delta(None, 500), None);
    }

    #[test]
    fn seasons_progress_floors_held_and_ceils_required() {
        // 25-tick eras: 60 ticks held is 2 full eras (the 3rd is
        // incomplete), 75 ticks required is exactly 3, 70 would still need
        // a 3rd era despite not being an exact multiple.
        assert_eq!(seasons_progress(60, 75, 25), (2, 3));
        assert_eq!(seasons_progress(0, 70, 25), (0, 3));
    }

    #[test]
    fn seasons_progress_handles_a_zero_season_length_without_dividing_by_zero() {
        assert_eq!(seasons_progress(10, 50, 0), (0, 0));
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
