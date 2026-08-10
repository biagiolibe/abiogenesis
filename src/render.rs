use bevy::asset::RenderAssetUsages;
use bevy::camera::ScalingMode;
use bevy::color::Mix;
use bevy::image::Image;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_egui::egui;
use bevy_egui::EguiPrimaryContextPass;

use abiogenesis::cluster::compute_cluster_density;
use abiogenesis::config::SimConfig;
use abiogenesis::sim::AdjacencyObserved;
use abiogenesis::state::GameState;
use abiogenesis::world::{Metabolism, SimWorld, SpeciesId, TagSlot, TerrainKind};

/// Pixel size of one grid cell on screen. Presentation-only, not a
/// simulation coefficient, so it stays local instead of living in
/// `SimConfig` (invariant 3 covers balance numbers, not pixel sizes).
const CELL_SIZE: f32 = 16.0;

/// Golden-angle hue step: successive `SpeciesId`s get visually distinct
/// hues without any per-species color configuration.
const SPECIES_HUE_STEP: f32 = 137.5;

/// Fixed, unordered word list for `species_label` (task 029). Species carry
/// no design secrecy constraint (unlike tags, GDD §11), so a generated name
/// is purely a legibility upgrade over bare "species N" — deterministic
/// from `SpeciesId`, no stored/editable state, same "derive from id"
/// approach as `SPECIES_HUE_STEP`/`TAG_HUE_STEP`.
const SPECIES_NAMES: [&str; 16] = [
    "Nyx", "Kael", "Sable", "Rook", "Vesk", "Lira", "Thorn", "Onyx", "Fenn", "Skye", "Brakk",
    "Cass", "Drys", "Elm", "Fira", "Grix",
];

/// A species' display label: a stable generated name plus its numeric id,
/// e.g. "Nyx (species 0)" — legible without requiring the player to
/// memorize raw indices, while keeping the number for disambiguation once
/// `Splice` (task 025) pushes the species count past the word list's length.
pub fn species_label(id: SpeciesId) -> String {
    let name = SPECIES_NAMES[id.0 as usize % SPECIES_NAMES.len()];
    format!("{name} (species {})", id.0)
}

/// Hue for a species' identity color, shared by `cell_color`'s grid
/// rendering and `species_color`'s egui swatch, so the two always agree.
fn species_hue(id: SpeciesId) -> f32 {
    (id.0 as f32 * SPECIES_HUE_STEP) % 360.0
}

/// A species' identity color as an egui swatch (playtest finding, task 041
/// session: the HUD's Seed Palette/Population panels listed species by name
/// only, with no way to connect a name to its on-grid dot color). Same hue
/// as `cell_color`'s organism base color; fixed saturation/value rather
/// than energy-scaled, so it reads as a stable identity marker, not an
/// energy readout.
pub fn species_color(id: SpeciesId) -> egui::Color32 {
    let hue = species_hue(id) / 360.0;
    egui::ecolor::Hsva::new(hue, 0.75, 0.9, 1.0).into()
}

/// A single glyph for a species' metabolism (task 065 — the sidebar's
/// Species list needs a species identifiable without opening the notebook,
/// since metabolism is a *readable* trait, GDD §5.3, not a hidden one like
/// tags). Indicative, unlike `notebook::tag_glyph`'s deliberately opaque
/// Greek letters: sun for light-drawing Photolithic, a strike for
/// prey-drawing Predator, a recycling mark for residue-drawing Decomposer.
pub fn metabolism_glyph(metabolism: Metabolism) -> &'static str {
    match metabolism {
        Metabolism::Photolithic => "☀",
        Metabolism::Predator => "⚔",
        Metabolism::Decomposer => "♻",
    }
}

/// Blue (0.0, cold/low) to red (1.0, hot/high) through the hue wheel — a
/// standard heatmap gradient, not tied to any in-game color meaning. Shared
/// by the dev-only `debug_view` cycle and the player-facing temperature/light
/// overlay toggles (task 058) — both want the same visual language for the
/// same raw scalars.
fn heat_color(value: f32) -> Color {
    let hue = 240.0 * (1.0 - value.clamp(0.0, 1.0));
    Color::hsl(hue, 0.85, 0.5)
}

/// Which environmental scalar, if any, the player has toggled a full-grid
/// heatmap overlay for (task 058 — `debug_view`'s `F1` cycle is dev-only and
/// undiscoverable; temperature and light are declared Phase-0 environmental
/// scalars, not part of the hidden interaction matrix GDD §7/§11 guards, so
/// making them player-readable on demand doesn't leak deduction-pillar
/// state). Mutually exclusive by construction: `toggle_environment_overlay`
/// only ever sets one variant at a time.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentOverlay {
    #[default]
    Off,
    Temperature,
    Light,
}

/// `T`/`L` toggle the temperature/light overlays on and off; pressing one
/// while the other is active switches directly instead of requiring the
/// player to turn the first one off.
fn toggle_environment_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<EnvironmentOverlay>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        *overlay = if *overlay == EnvironmentOverlay::Temperature {
            EnvironmentOverlay::Off
        } else {
            EnvironmentOverlay::Temperature
        };
    } else if keys.just_pressed(KeyCode::KeyL) {
        *overlay = if *overlay == EnvironmentOverlay::Light {
            EnvironmentOverlay::Off
        } else {
            EnvironmentOverlay::Light
        };
    }
}

/// Runs after `sync_grid_colors` and overwrites its output when a toggle is
/// active, same shape as `debug_view::apply_debug_view` — the two overlays
/// are independent features (one dev-only and covers all raw scalars incl.
/// `toxicity`, this one is player-facing and only ever shows temperature or
/// light) that happen to share `heat_color`.
///
/// Used to skip unplaceable cells (task 068) under the old fixed-gradient
/// model, where Sea/peaks carried the same uninteresting boilerplate value
/// as their surroundings — hiding them protected the terrain read without
/// losing anything. Task 085's source model made that stale: `Sea` cells now
/// act as a real passive coolant (`SimWorld::reinject_environment_sources`)
/// that measurably shapes the field, so skipping them tore a black gap
/// through the middle of an otherwise continuous gradient — task 086
/// playtest finding, it read as broken/discontinuous data rather than "this
/// terrain is just excluded." Every cell now renders its real scalar,
/// Sea/peaks included.
fn apply_environment_overlay(
    world: Res<SimWorld>,
    overlay: Res<EnvironmentOverlay>,
    mut cells: Query<(&GridCell, &mut Sprite)>,
) {
    if *overlay == EnvironmentOverlay::Off {
        return;
    }
    for (cell, mut sprite) in &mut cells {
        let scalar = match *overlay {
            EnvironmentOverlay::Off => unreachable!(),
            EnvironmentOverlay::Temperature => world.get(cell.x, cell.y).temperature,
            EnvironmentOverlay::Light => world.get(cell.x, cell.y).light,
        };
        sprite.color = heat_color(scalar);
    }
}

/// Links a rendered sprite back to its cell in `SimWorld`. The sprite is a
/// view: simulation state lives in the resource, never here (TECH_DESIGN §3.1).
#[derive(Component)]
struct GridCell {
    x: usize,
    y: usize,
}

/// Marks the camera that renders the grid, as opposed to the dedicated
/// camera `ui::UiPlugin` spawns for egui (TECH_DESIGN.md §6 "HUD camera").
/// Lets `ui::reserve_hud_viewport` crop this camera's `Viewport` without
/// touching the egui camera's, whose own viewport doubles as egui's paint
/// canvas.
#[derive(Component)]
pub struct GridCamera;

pub struct GridRenderPlugin;

impl Plugin for GridRenderPlugin {
    fn build(&self, app: &mut App) {
        // `spawn_camera`/`spawn_grid`/`spawn_metabolism_shapes` only need
        // `SimConfig` (grid size), so they stay at `Startup` unconditionally
        // — the sprites just sit uncolored (`Sprite::from_color(BLACK, ..)`)
        // until `sync_grid_colors` starts running once `Playing` begins
        // (task 044: `SimWorld` doesn't exist before then).
        // `SeenRelations`' size depends on `SimConfig::tags`, same rationale
        // and same "fixed at build, resized on every world (re)start" split
        // as `NotebookPlugin`'s `MatrixKnowledge` init.
        let config = app.world().resource::<SimConfig>();
        let seen_relations = SeenRelations::new(config.tags.active_tags_early as usize);

        app.insert_resource(seen_relations)
            .add_systems(Startup, (spawn_camera, spawn_grid, spawn_metabolism_shapes))
            .init_resource::<EnvironmentOverlay>()
            .init_resource::<MapViewMode>()
            .init_resource::<ClusterDensity>()
            .init_resource::<placement_indicator::PlacementIndicator>()
            .init_resource::<SparkIndicators>()
            .add_systems(
                Update,
                (
                    zoom_camera.run_if(in_state(GameState::Playing)),
                    pan_camera
                        .after(zoom_camera)
                        .run_if(in_state(GameState::Playing)),
                    update_map_view_mode
                        .after(pan_camera)
                        .run_if(in_state(GameState::Playing)),
                    update_cluster_density
                        .after(update_map_view_mode)
                        .run_if(in_state(GameState::Playing)),
                    sync_grid_colors
                        .after(update_cluster_density)
                        .run_if(in_state(GameState::Playing)),
                    toggle_environment_overlay,
                    apply_environment_overlay
                        .after(sync_grid_colors)
                        .run_if(in_state(GameState::Playing)),
                    placement_indicator::tick_placement_indicator
                        .run_if(in_state(GameState::Playing)),
                    spawn_spark_on_first_observation.run_if(in_state(GameState::Playing)),
                    spark_indicator::tick_spark_indicators.run_if(in_state(GameState::Playing)),
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    terrain_overlay::draw_terrain_overlay,
                    placement_indicator::draw_placement_indicator,
                    spark_indicator::draw_spark_indicators,
                )
                    .run_if(in_state(GameState::Playing)),
            );
        #[cfg(debug_assertions)]
        {
            app.init_resource::<debug_view::DebugView>()
                .init_resource::<energy_overlay::EnergyOverlay>()
                .add_systems(
                    Update,
                    (
                        debug_view::toggle_debug_view,
                        debug_view::apply_debug_view
                            .after(sync_grid_colors)
                            .run_if(in_state(GameState::Playing)),
                        energy_overlay::toggle_energy_overlay,
                    ),
                )
                .add_systems(
                    EguiPrimaryContextPass,
                    energy_overlay::draw_energy_overlay.run_if(in_state(GameState::Playing)),
                );
        }
    }
}

/// A dev-only heatmap overlay for raw `toxicity` (task 023 revealed the need:
/// before task 033's always-on tint, it had no visible effect anywhere,
/// discoverable only by grepping the tick code). `Temperature`/`Light` used
/// to be part of this same `F1` cycle, but task 058 gave them dedicated
/// player-facing `T`/`L` toggles (`EnvironmentOverlay`) that render the exact
/// same `heat_color` heatmap, so keeping them here too would just be the same
/// view behind two different keys — trimmed down to the one scalar that
/// still lacks a full, undiluted (non-tinted) heatmap anywhere else. Never
/// compiled into a release build — the game's whole deduction pillar (GDD
/// §7, §11) is built on the player *not* having direct instrument readouts
/// for the hidden interaction matrix, so this must not leak past
/// development (toxicity itself isn't part of that hidden matrix, but this
/// isolated view is still a dev convenience, not meant to ship).
#[cfg(debug_assertions)]
mod debug_view {
    use super::{heat_color, GridCell};
    use bevy::prelude::*;

    use abiogenesis::world::SimWorld;

    /// `F1` cycles through these. `Normal` defers entirely to `cell_color` —
    /// this module never touches rendering unless a non-`Normal` view is
    /// active.
    #[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
    pub enum DebugView {
        #[default]
        Normal,
        Toxicity,
    }

    impl DebugView {
        fn next(self) -> Self {
            match self {
                DebugView::Normal => DebugView::Toxicity,
                DebugView::Toxicity => DebugView::Normal,
            }
        }
    }

    pub fn toggle_debug_view(keys: Res<ButtonInput<KeyCode>>, mut view: ResMut<DebugView>) {
        if keys.just_pressed(KeyCode::F1) {
            *view = view.next();
        }
    }

    /// Runs after `sync_grid_colors` and overwrites its output when a
    /// non-`Normal` view is selected, so the normal rendering path in
    /// `cell_color` stays untouched by this module's existence.
    pub fn apply_debug_view(
        world: Res<SimWorld>,
        view: Res<DebugView>,
        mut cells: Query<(&GridCell, &mut Sprite)>,
    ) {
        if *view == DebugView::Normal {
            return;
        }
        for (cell, mut sprite) in &mut cells {
            let scalar = match *view {
                DebugView::Normal => unreachable!(),
                DebugView::Toxicity => world.get(cell.x, cell.y).toxicity,
            };
            sprite.color = heat_color(scalar);
        }
    }
}

/// A dev-only overlay printing each occupied cell's current energy as a
/// number over the grid (playtest finding: a predator seeded next to
/// tag-neutral neighbours still died, and nothing on screen explained why —
/// this makes the raw energy trajectory visible without re-deriving it from
/// `sim.rs`). Never compiled into a release build, same rationale as
/// `debug_view`.
#[cfg(debug_assertions)]
mod energy_overlay {
    use super::{cell_position, GridCamera};
    use abiogenesis::world::SimWorld;
    use bevy::prelude::*;
    use bevy_egui::{egui, EguiContexts};

    /// Font size for the overlay's energy numbers. Small enough that the
    /// worst case — a 4-character label like `10.2` (energies range 0 to
    /// `repro_threshold`, typically 10.0) — fits inside one grid cell
    /// (`CELL_SIZE = 16.0` world units, ~17 logical points at the default
    /// 48x32 grid's `AutoMin` scale ≈ 1.0), so adjacent occupied cells —
    /// the normal case once reproduction clusters form — don't overlap.
    /// Presentation-only, same category as `CELL_SIZE`/`SPECIES_HUE_STEP` —
    /// not a `SimConfig` coefficient.
    const ENERGY_OVERLAY_FONT_SIZE: f32 = 6.0;

    /// Whether the `F2` overlay is shown. Kept separate from `DebugView`
    /// (not folded into its cycle): the two are orthogonal — this can be on
    /// alongside any `DebugView` variant, unlike `DebugView`'s own
    /// mutually-exclusive recolorings.
    #[derive(Resource, Default)]
    pub struct EnergyOverlay(pub bool);

    pub fn toggle_energy_overlay(
        keys: Res<ButtonInput<KeyCode>>,
        mut overlay: ResMut<EnergyOverlay>,
    ) {
        if keys.just_pressed(KeyCode::F2) {
            overlay.0 = !overlay.0;
        }
    }

    /// Draws every occupied cell's current energy as text over the grid, via
    /// a background-layer egui painter — the first thing in the codebase to
    /// need a world-to-screen projection (the reverse, `world_to_cell`, is
    /// used for click handling).
    pub fn draw_energy_overlay(
        overlay: Res<EnergyOverlay>,
        world: Res<SimWorld>,
        cameras: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
        mut contexts: EguiContexts,
    ) -> Result {
        if !overlay.0 {
            return Ok(());
        }
        let Ok((camera, camera_transform)) = cameras.single() else {
            return Ok(());
        };
        let ctx = contexts.ctx_mut()?;
        let painter = ctx.layer_painter(egui::LayerId::background());
        // See `terrain_overlay::draw_terrain_overlay`'s identical clip: task
        // 075's zoom means `world_to_viewport` can project an off-frame
        // cell into the HUD sidebar's screen area otherwise.
        let painter = match camera.logical_viewport_rect() {
            Some(viewport_rect) => painter.with_clip_rect(egui::Rect::from_min_max(
                egui::pos2(viewport_rect.min.x, viewport_rect.min.y),
                egui::pos2(viewport_rect.max.x, viewport_rect.max.y),
            )),
            None => painter,
        };

        // `Camera::world_to_viewport` already returns logical (window-point)
        // coordinates — the same space `Window::cursor_position()` uses, per
        // `input.rs`'s reverse conversion (`viewport_to_world_2d`). Dividing
        // by `window.scale_factor()` again here double-applies the HiDPI
        // scale and compresses every position toward the origin (this was
        // caught by an in-game screenshot on a Retina display: numbers
        // clustered in the top-left instead of sitting on their cells).
        for y in 0..world.height {
            for x in 0..world.width {
                let Some(organism) = world.get(x, y).organism else {
                    continue;
                };
                let world_pos = cell_position(x, y, world.width, world.height);
                let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) else {
                    continue;
                };
                let pos = egui::pos2(viewport_pos.x, viewport_pos.y);
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    format!("{:.1}", organism.energy),
                    egui::FontId::monospace(ENERGY_OVERLAY_FONT_SIZE),
                    egui::Color32::WHITE,
                );
            }
        }
        Ok(())
    }
}

/// Renders the terrain data from task 066 that the grid's flat cell colors
/// alone can't show (task 068, `redesign/abiogenesis-terrain-map.md`):
/// boundaries between differently-classed neighbours, sparse peak glyphs,
/// and the toxic zone's dashed outline. Same background-layer egui painter
/// technique as `energy_overlay::draw_energy_overlay` — the grid has no
/// other mechanism for a shared-cell-edge hairline or a text glyph — but
/// always on, not a debug/dev toggle: this is the terrain itself made
/// visible, not an instrument reading.
mod terrain_overlay {
    use super::{cell_position, GridCamera, CELL_SIZE};
    use abiogenesis::world::{SimWorld, TerrainKind};
    use bevy::prelude::*;
    use bevy_egui::{egui, EguiContexts};

    const BOUNDARY_INTERNAL_WIDTH: f32 = 0.6;
    const BOUNDARY_COASTLINE_WIDTH: f32 = 1.4;

    /// Internal band-to-band boundary (e.g. Plain/Hill): thin and low-alpha
    /// — "present but secondary" per the design doc. Not a `const`:
    /// `Color32::from_rgba_unmultiplied` isn't a `const fn`.
    fn boundary_internal_color() -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(190, 190, 178, 70)
    }

    /// Sea↔land boundary ("coastline"): thicker and brighter, read as the
    /// limit of the explorable world.
    fn boundary_coastline_color() -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(220, 220, 208, 210)
    }

    const PEAK_GLYPH: &str = "^";
    const PEAK_GLYPH_FONT_SIZE: f32 = 9.0;
    const PEAK_GLYPH_COLOR: egui::Color32 = egui::Color32::from_gray(225);

    /// Matches the reference mockup's `#7F77DD` — a deliberate new color for
    /// "toxic/hazardous," not yet used elsewhere in the codebase (the
    /// notebook's zero-evidence dashed ring is gray, not purple, despite the
    /// design doc's claim of cross-screen reuse — the dash *technique* is
    /// what's actually shared, per `draw_dashed_line` below).
    const TOXIC_ZONE_OUTLINE_COLOR: egui::Color32 = egui::Color32::from_rgb(127, 119, 221);
    const TOXIC_ZONE_OUTLINE_WIDTH: f32 = 1.4;
    const TOXIC_ZONE_DASH_LENGTH: f32 = 4.0;
    const TOXIC_ZONE_GAP_LENGTH: f32 = 3.0;

    pub fn draw_terrain_overlay(
        world: Res<SimWorld>,
        cameras: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
        mut contexts: EguiContexts,
    ) -> Result {
        let Ok((camera, camera_transform)) = cameras.single() else {
            return Ok(());
        };
        let ctx = contexts.ctx_mut()?;
        let painter = ctx.layer_painter(egui::LayerId::background());
        // `Camera::world_to_viewport` doesn't clip to the viewport's actual
        // bounds — a cell just outside the camera's cropped viewport (task
        // 075's zoom can put most of the grid off-frame) still projects to
        // *some* pixel position, which without this clip can land inside
        // the HUD sidebar's screen area and bleed boundary/toxic-zone lines
        // into it. Restricting the painter to the camera's own logical
        // viewport rect matches what the Bevy-rendered sprites already show
        // (the renderer clips those to `Camera::viewport` for free).
        let painter = match camera.logical_viewport_rect() {
            Some(viewport_rect) => painter.with_clip_rect(egui::Rect::from_min_max(
                egui::pos2(viewport_rect.min.x, viewport_rect.min.y),
                egui::pos2(viewport_rect.max.x, viewport_rect.max.y),
            )),
            None => painter,
        };

        let project = |pos: Vec3| -> Option<egui::Pos2> {
            camera
                .world_to_viewport(camera_transform, pos)
                .ok()
                .map(|p| egui::pos2(p.x, p.y))
        };

        draw_boundaries(&world, &project, &painter);
        draw_peaks(&world, &project, &painter);
        draw_toxic_zone(&world, &project, &painter);
        Ok(())
    }

    /// World-space corners of cell `(x, y)`, `(top_left, top_right,
    /// bottom_right, bottom_left)` — `cell_position` gives the center, this
    /// offsets by half a cell in each direction.
    fn cell_corners(x: usize, y: usize, width: usize, height: usize) -> (Vec3, Vec3, Vec3, Vec3) {
        let center = cell_position(x, y, width, height);
        let half = CELL_SIZE / 2.0;
        (
            center + Vec3::new(-half, half, 0.0),
            center + Vec3::new(half, half, 0.0),
            center + Vec3::new(half, -half, 0.0),
            center + Vec3::new(-half, -half, 0.0),
        )
    }

    /// Compares each cell against its right and bottom neighbours only
    /// (task doc: "sufficient to avoid drawing every shared edge twice").
    fn draw_boundaries(
        world: &SimWorld,
        project: &impl Fn(Vec3) -> Option<egui::Pos2>,
        painter: &egui::Painter,
    ) {
        for y in 0..world.height {
            for x in 0..world.width {
                let here = world.get(x, y).terrain;
                if x + 1 < world.width {
                    let right = world.get(x + 1, y).terrain;
                    if right != here {
                        let (_, top_right, bottom_right, _) =
                            cell_corners(x, y, world.width, world.height);
                        draw_edge(project, painter, top_right, bottom_right, here, right);
                    }
                }
                if y + 1 < world.height {
                    let below = world.get(x, y + 1).terrain;
                    if below != here {
                        let (_, _, bottom_right, bottom_left) =
                            cell_corners(x, y, world.width, world.height);
                        draw_edge(project, painter, bottom_left, bottom_right, here, below);
                    }
                }
            }
        }
    }

    fn draw_edge(
        project: &impl Fn(Vec3) -> Option<egui::Pos2>,
        painter: &egui::Painter,
        from: Vec3,
        to: Vec3,
        a: TerrainKind,
        b: TerrainKind,
    ) {
        let (Some(p0), Some(p1)) = (project(from), project(to)) else {
            return;
        };
        let is_coastline = a == TerrainKind::Sea || b == TerrainKind::Sea;
        let (color, width) = if is_coastline {
            (boundary_coastline_color(), BOUNDARY_COASTLINE_WIDTH)
        } else {
            (boundary_internal_color(), BOUNDARY_INTERNAL_WIDTH)
        };
        painter.line_segment([p0, p1], egui::Stroke::new(width, color));
    }

    /// One glyph per stored peak cell (task 066 decides peaks at generation
    /// time; this reads that directly, no render-time local-maximum scan).
    fn draw_peaks(
        world: &SimWorld,
        project: &impl Fn(Vec3) -> Option<egui::Pos2>,
        painter: &egui::Painter,
    ) {
        for y in 0..world.height {
            for x in 0..world.width {
                if !world.get(x, y).is_peak {
                    continue;
                }
                let Some(pos) = project(cell_position(x, y, world.width, world.height)) else {
                    continue;
                };
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    PEAK_GLYPH,
                    egui::FontId::monospace(PEAK_GLYPH_FONT_SIZE),
                    PEAK_GLYPH_COLOR,
                );
            }
        }
    }

    fn draw_toxic_zone(
        world: &SimWorld,
        project: &impl Fn(Vec3) -> Option<egui::Pos2>,
        painter: &egui::Painter,
    ) {
        let zone = world.toxic_zone;
        if zone.width == 0 || zone.height == 0 {
            return;
        }
        let (top_left, _, _, _) = cell_corners(zone.x0, zone.y0, world.width, world.height);
        let (_, _, bottom_right, _) = cell_corners(
            zone.x0 + zone.width - 1,
            zone.y0 + zone.height - 1,
            world.width,
            world.height,
        );
        let (Some(p0), Some(p1)) = (project(top_left), project(bottom_right)) else {
            return;
        };
        let top_right = egui::pos2(p1.x, p0.y);
        let bottom_left = egui::pos2(p0.x, p1.y);
        draw_dashed_line(painter, p0, top_right);
        draw_dashed_line(painter, top_right, p1);
        draw_dashed_line(painter, p1, bottom_left);
        draw_dashed_line(painter, bottom_left, p0);
    }

    /// Approximates a dashed line the same way `notebook.rs`'s
    /// `draw_dashed_ring` approximates a dashed circle — alternating short
    /// straight segments — adapted from a ring to a rectangle's four edges.
    fn draw_dashed_line(painter: &egui::Painter, from: egui::Pos2, to: egui::Pos2) {
        let delta = to - from;
        let length = delta.length();
        if length < f32::EPSILON {
            return;
        }
        let dir = delta / length;
        let step = TOXIC_ZONE_DASH_LENGTH + TOXIC_ZONE_GAP_LENGTH;
        let mut travelled = 0.0;
        while travelled < length {
            let dash_end = (travelled + TOXIC_ZONE_DASH_LENGTH).min(length);
            let p0 = from + dir * travelled;
            let p1 = from + dir * dash_end;
            painter.line_segment(
                [p0, p1],
                egui::Stroke::new(TOXIC_ZONE_OUTLINE_WIDTH, TOXIC_ZONE_OUTLINE_COLOR),
            );
            travelled += step;
        }
    }
}

/// A brief on-map ring marking exactly where a Seed placement landed while
/// `MapViewMode::Overview` is active (task 077): Overview's cluster-heatmap
/// aggregation (task 076) doesn't show individual cells, so without this the
/// player has no way to tell which cell of a blob they just placed into —
/// the cluster blob then carries visibility forward on its own once it grows
/// large enough to render (`redesign/abiogenesis-two-tier-view.md`). Uses
/// real (`Res<Time>`) rather than sim-tick duration: `SimWorld::tick` only
/// advances on era ticks, so a tick-based flash could linger a whole era if
/// the player doesn't advance time right away. Same egui-painter/
/// `world_to_viewport` overlay technique as `terrain_overlay` and
/// `energy_overlay` above.
pub use placement_indicator::PlacementIndicator;

mod placement_indicator {
    use super::{cell_position, GridCamera, CELL_SIZE};
    use abiogenesis::world::SimWorld;
    use bevy::prelude::*;
    use bevy_egui::{egui, EguiContexts};

    const DURATION_SECS: f32 = 0.6;
    const RING_RADIUS: f32 = CELL_SIZE * 0.6;
    const RING_WIDTH: f32 = 2.0;
    const RING_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 220, 90);

    /// Set by `input.rs`'s `seed_organism_on_click` on a successful Seed
    /// resolved in Overview mode; ticks down and self-clears in
    /// `tick_placement_indicator` — purely cosmetic, never read by `sim`/
    /// `world`/`input.rs`'s action logic itself.
    #[derive(Resource, Default)]
    pub struct PlacementIndicator(pub Option<PlacementIndicatorState>);

    pub struct PlacementIndicatorState {
        pub x: usize,
        pub y: usize,
        remaining_secs: f32,
    }

    impl PlacementIndicator {
        pub fn show(&mut self, x: usize, y: usize) {
            self.0 = Some(PlacementIndicatorState {
                x,
                y,
                remaining_secs: DURATION_SECS,
            });
        }
    }

    pub fn tick_placement_indicator(time: Res<Time>, mut indicator: ResMut<PlacementIndicator>) {
        let Some(state) = indicator.0.as_mut() else {
            return;
        };
        state.remaining_secs -= time.delta_secs();
        if state.remaining_secs <= 0.0 {
            indicator.0 = None;
        }
    }

    /// Same viewport-clip and `world_to_viewport` projection as
    /// `terrain_overlay::draw_terrain_overlay` — see that function's comment
    /// for why the clip is needed once task 075's zoom can put a cell
    /// off-frame.
    pub fn draw_placement_indicator(
        indicator: Res<PlacementIndicator>,
        world: Res<SimWorld>,
        cameras: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
        mut contexts: EguiContexts,
    ) -> Result {
        let Some(state) = indicator.0.as_ref() else {
            return Ok(());
        };
        let Ok((camera, camera_transform)) = cameras.single() else {
            return Ok(());
        };
        let ctx = contexts.ctx_mut()?;
        let painter = ctx.layer_painter(egui::LayerId::background());
        let painter = match camera.logical_viewport_rect() {
            Some(viewport_rect) => painter.with_clip_rect(egui::Rect::from_min_max(
                egui::pos2(viewport_rect.min.x, viewport_rect.min.y),
                egui::pos2(viewport_rect.max.x, viewport_rect.max.y),
            )),
            None => painter,
        };

        let world_pos = cell_position(state.x, state.y, world.width, world.height);
        let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) else {
            return Ok(());
        };
        // Fades out over the ring's lifetime rather than a hard cut.
        let alpha = (state.remaining_secs / DURATION_SECS).clamp(0.0, 1.0);
        painter.circle_stroke(
            egui::pos2(viewport_pos.x, viewport_pos.y),
            RING_RADIUS,
            egui::Stroke::new(RING_WIDTH, RING_COLOR.gamma_multiply(alpha)),
        );
        Ok(())
    }
}

/// A brief on-map ring marking every cell where a tag-pair relation was just
/// observed for the *first time ever* in this world (task 080, GDD §5.6's
/// `interaction_delta` — invisible otherwise until enough `AdjacencyObserved`
/// evidence accumulates in the Notebook, task 018/020). Same visual language
/// as `placement_indicator` (fixed radius, alpha-only fade) but `Vec`-backed
/// rather than single-slot: several relations can be confirmed on the same
/// tick, unlike a Seed placement which only ever happens one at a time.
/// Renders at the exact cell in both `MapViewMode::Detail` and `Overview` —
/// Overview reuses the same per-cell sprites as Detail, just recolored by
/// cluster density (`cluster::compute_cluster_density`), so there is no
/// separate blob/centroid geometry to aggregate onto; the cell position
/// alone is correct in either mode.
pub use spark_indicator::SparkIndicators;

mod spark_indicator {
    use super::{cell_position, GridCamera, CELL_SIZE};
    use abiogenesis::world::SimWorld;
    use bevy::prelude::*;
    use bevy_egui::{egui, EguiContexts};

    const DURATION_SECS: f32 = 0.6;
    const RING_RADIUS: f32 = CELL_SIZE * 0.6;
    const RING_WIDTH: f32 = 2.0;
    const RING_COLOR: egui::Color32 = egui::Color32::from_rgb(120, 220, 255);

    #[derive(Resource, Default)]
    pub struct SparkIndicators(Vec<SparkIndicatorState>);

    struct SparkIndicatorState {
        x: usize,
        y: usize,
        remaining_secs: f32,
    }

    impl SparkIndicators {
        pub fn spawn(&mut self, x: usize, y: usize) {
            self.0.push(SparkIndicatorState {
                x,
                y,
                remaining_secs: DURATION_SECS,
            });
        }

        /// How many spark rings are currently live — test-only observation
        /// point for `spawn_spark_on_first_observation`'s gating behaviour.
        #[cfg(test)]
        pub fn len(&self) -> usize {
            self.0.len()
        }
    }

    pub fn tick_spark_indicators(time: Res<Time>, mut indicators: ResMut<SparkIndicators>) {
        for state in &mut indicators.0 {
            state.remaining_secs -= time.delta_secs();
        }
        indicators.0.retain(|state| state.remaining_secs > 0.0);
    }

    /// Same viewport-clip and `world_to_viewport` projection as
    /// `placement_indicator::draw_placement_indicator`; draws once per
    /// still-live indicator instead of at most one.
    pub fn draw_spark_indicators(
        indicators: Res<SparkIndicators>,
        world: Res<SimWorld>,
        cameras: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
        mut contexts: EguiContexts,
    ) -> Result {
        if indicators.0.is_empty() {
            return Ok(());
        }
        let Ok((camera, camera_transform)) = cameras.single() else {
            return Ok(());
        };
        let ctx = contexts.ctx_mut()?;
        let painter = ctx.layer_painter(egui::LayerId::background());
        let painter = match camera.logical_viewport_rect() {
            Some(viewport_rect) => painter.with_clip_rect(egui::Rect::from_min_max(
                egui::pos2(viewport_rect.min.x, viewport_rect.min.y),
                egui::pos2(viewport_rect.max.x, viewport_rect.max.y),
            )),
            None => painter,
        };

        for state in &indicators.0 {
            let world_pos = cell_position(state.x, state.y, world.width, world.height);
            let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) else {
                continue;
            };
            let alpha = (state.remaining_secs / DURATION_SECS).clamp(0.0, 1.0);
            painter.circle_stroke(
                egui::pos2(viewport_pos.x, viewport_pos.y),
                RING_RADIUS,
                egui::Stroke::new(RING_WIDTH, RING_COLOR.gamma_multiply(alpha)),
            );
        }
        Ok(())
    }
}

/// Tracks which `(exerter_tag, receiver_tag)` pairs have already triggered
/// an interaction spark this world (task 080) — a first-observation-only
/// gate, deliberately a much lower bar than `MatrixKnowledge`'s confirmation
/// threshold (task 020, GDD §7): these are two independent signals sharing
/// the same `AdjacencyObserved` event stream. Same flat-`Vec`-indexed-by-
/// `TagSlot` shape as `TagMatrix`/`MatrixKnowledge` (`exerter * size +
/// receiver`), not a `HashSet` — a pure membership lookup, never iterated
/// (`CLAUDE.md`'s no-`HashMap`/`HashSet`-iteration determinism rule).
#[derive(Resource)]
pub struct SeenRelations {
    size: usize,
    seen: Vec<bool>,
}

impl SeenRelations {
    pub fn new(active_tags: usize) -> Self {
        Self {
            size: active_tags,
            seen: vec![false; active_tags * active_tags],
        }
    }

    /// Marks a pair as seen, returning `true` if this call is the first
    /// time this pair has ever been observed in this world.
    fn mark_seen(&mut self, exerter: TagSlot, receiver: TagSlot) -> bool {
        let idx = exerter.0 as usize * self.size + receiver.0 as usize;
        let first = !self.seen[idx];
        self.seen[idx] = true;
        first
    }
}

/// Drains `AdjacencyObserved` (task 018) and spawns a spark indicator the
/// first time ever a given `(exerter_tag, receiver_tag)` pair is observed in
/// this world — turns the hidden matrix into an observed event instead of a
/// purely deduced one (task 080, proposal 1.A). `event.cell` (task 080)
/// carries the receiver's cell index directly, so no neighbour lookup is
/// needed here.
fn spawn_spark_on_first_observation(
    world: Res<SimWorld>,
    mut observed: MessageReader<AdjacencyObserved>,
    mut seen: ResMut<SeenRelations>,
    mut indicators: ResMut<SparkIndicators>,
) {
    for event in observed.read() {
        if seen.mark_seen(event.exerter_tag, event.receiver_tag) {
            let x = event.cell % world.width;
            let y = event.cell / world.width;
            indicators.spawn(x, y);
        }
    }
}

/// Which representation the organism layer renders in — the hard-threshold
/// switch task 075 introduces (`redesign/abiogenesis-two-tier-view.md`):
/// `Overview` (default, matches the un-zoomed `scale == 1.0` whole-grid
/// framing) shows the per-species cluster heatmap (task 076); `Detail`
/// shows today's per-cell organism sprites, unchanged. Driven by
/// `update_map_view_mode` from the camera's current zoom (`CameraConfig::
/// zoom_threshold`), consumed here only for a debug indicator — task 076
/// reads it to pick a rendering path, task 077 to gate Stress/Cull.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapViewMode {
    #[default]
    Overview,
    Detail,
}

fn spawn_camera(mut commands: Commands, config: Res<SimConfig>) {
    let width = config.grid.width as f32 * CELL_SIZE;
    let height = config.grid.height as f32 * CELL_SIZE;
    commands.spawn((
        Camera2d,
        GridCamera,
        Projection::Orthographic(OrthographicProjection {
            // Never smaller than the grid, so it never clips on resize; a
            // wider/taller window just shows more letterboxed space. Zoom
            // (task 075) multiplies on top of this via `scale`, `1.0` being
            // exactly this whole-grid framing (`CameraConfig::zoom_max`).
            scaling_mode: ScalingMode::AutoMin {
                min_width: width,
                min_height: height,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Mouse-wheel zoom, centered on the cursor's world position (task 075):
/// scrolling in keeps the point under the cursor fixed on screen, standard
/// map-zoom behavior. Multiplicative per scroll unit (`CameraConfig::
/// zoom_speed`) rather than additive, so it feels equally responsive at any
/// zoom level; clamped to `[zoom_min, zoom_max]`.
///
/// Cursor-centered zoom math: for a camera with a centered viewport_origin
/// and no rotation, `OrthographicProjection::scale` uniformly scales the
/// world-space area shown around the camera's own translation. Keeping a
/// world point `p` fixed under the cursor while `scale` changes from `old`
/// to `new` requires shifting the camera translation `t` by
/// `(p - t) * (1 - new/old)` — derived directly from `scale`'s effect on
/// `OrthographicProjection::area` (bevy_camera's `update`), not empirically
/// tuned, so it holds at any zoom level without re-deriving per frame.
fn zoom_camera(
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    config: Res<SimConfig>,
    mut cameras: Query<
        (&Camera, &GlobalTransform, &mut Projection, &mut Transform),
        With<GridCamera>,
    >,
) {
    let scroll: f32 = wheel.read().map(|event| event.y).sum();
    if scroll == 0.0 {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform, mut projection, mut transform)) = cameras.single_mut() else {
        return;
    };
    let Projection::Orthographic(ortho) = projection.as_mut() else {
        return;
    };
    let Ok(world_pos_before) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };

    let cam_cfg = &config.camera;
    // Scrolling in (positive `y`) zooms in: `zoom_speed < 1.0` shrinks
    // `scale`, and smaller `scale` means less world-space area per pixel.
    let factor = cam_cfg.zoom_speed.powf(scroll);
    let old_scale = ortho.scale;
    let new_scale = (old_scale * factor).clamp(cam_cfg.zoom_min, cam_cfg.zoom_max);
    if new_scale == old_scale {
        return;
    }

    // `ortho.area`'s size is `scale * (AutoMin's unscaled projection dims)`
    // (bevy_camera's `update`) — dividing out `old_scale` recovers those
    // dims without duplicating `AutoMin`'s own width/height math, so the
    // pan clamp below stays correct regardless of window/grid aspect ratio.
    let unscaled_area = ortho.area.size() / old_scale;
    ortho.scale = new_scale;

    let ratio = new_scale / old_scale;
    let translation = transform.translation.truncate();
    let shifted = translation + (world_pos_before - translation) * (1.0 - ratio);

    let grid_extent = grid_extent(&config);
    let clamped = clamp_camera_pan(shifted, new_scale, unscaled_area, grid_extent);

    transform.translation.x = clamped.x;
    transform.translation.y = clamped.y;
}

/// The grid's total extent in world units, for pan/zoom clamping — shared so
/// `zoom_camera` and `pan_camera` never drift out of sync on how it's derived.
fn grid_extent(config: &SimConfig) -> Vec2 {
    Vec2::new(
        config.grid.width as f32 * CELL_SIZE,
        config.grid.height as f32 * CELL_SIZE,
    )
}

/// Clamps a candidate camera translation so the viewport never shows area
/// outside the grid (task 087, factored out of `zoom_camera`'s original pan
/// clamp so `pan_camera` reuses the exact same rule instead of duplicating
/// it). On an axis where the visible span at `scale` is already >= the
/// grid's own extent (e.g. `zoom_max`'s whole-grid floor, or an axis
/// `AutoMin` over-extends to match the window's aspect), the only valid
/// translation is `0` — any pan there would clip grid content on one side
/// while revealing empty space on the other, exactly the "map scrolls under
/// the sidebar" bug task 075 fixed.
fn clamp_camera_pan(translation: Vec2, scale: f32, unscaled_area: Vec2, grid_extent: Vec2) -> Vec2 {
    let visible = unscaled_area * scale;
    let max_pan = ((grid_extent - visible) / 2.0).max(Vec2::ZERO);
    translation.clamp(-max_pan, max_pan)
}

/// Arrow-key/`WASD` camera pan (task 087), complementing `zoom_camera`'s
/// incidental translation shift with a dedicated way to reach any part of
/// the grid at a fixed zoom level. `input::single_tick` was moved off `KeyS`
/// onto `KeyN` (task 087 follow-up) precisely so `WASD` could be bound here
/// without colliding with it. Speed is in grid cells/second at `scale ==
/// 1.0` (`CameraConfig::pan_speed`), scaled by the camera's current `scale`
/// so panning feels equally responsive at any zoom level, same intent as
/// `zoom_speed`. Clamped through `clamp_camera_pan`, identical to zoom's
/// edge-of-grid guarantee.
fn pan_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<SimConfig>,
    mut cameras: Query<(&Projection, &mut Transform), With<GridCamera>>,
) {
    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if direction == Vec2::ZERO {
        return;
    }

    let Ok((projection, mut transform)) = cameras.single_mut() else {
        return;
    };
    let Projection::Orthographic(ortho) = projection else {
        return;
    };

    let speed = config.camera.pan_speed * CELL_SIZE * ortho.scale;
    let translation =
        transform.translation.truncate() + direction.normalize() * speed * time.delta_secs();

    let unscaled_area = ortho.area.size() / ortho.scale;
    let clamped = clamp_camera_pan(
        translation,
        ortho.scale,
        unscaled_area,
        grid_extent(&config),
    );

    transform.translation.x = clamped.x;
    transform.translation.y = clamped.y;
}

/// Syncs `MapViewMode` from the camera's current zoom (task 075) — a hard
/// threshold, not a blend, per `redesign/abiogenesis-two-tier-view.md`.
fn update_map_view_mode(
    config: Res<SimConfig>,
    cameras: Query<&Projection, With<GridCamera>>,
    mut mode: ResMut<MapViewMode>,
) {
    let Ok(Projection::Orthographic(ortho)) = cameras.single() else {
        return;
    };
    let new_mode = if ortho.scale < config.camera.zoom_threshold {
        MapViewMode::Detail
    } else {
        MapViewMode::Overview
    };
    if *mode != new_mode {
        *mode = new_mode;
    }
}

/// Per-cell cluster density (task 076, `cluster::compute_cluster_density`),
/// consumed by `cell_color` when `MapViewMode::Overview` is active. Indexed
/// like `SimWorld::cells` (`world.index(x, y)`); `0.0` for any cell with no
/// organism.
#[derive(Resource, Default)]
struct ClusterDensity(Vec<f32>);

/// Recomputes `ClusterDensity` only when there's something new to show: a
/// population-changing event (tick advance, action resolution) mutates
/// `SimWorld` via `ResMut` elsewhere, which is exactly what `Res<SimWorld>::
/// is_changed` reports here — connected-component clustering is O(cells) and
/// the task doc calls out not re-running it unconditionally every render
/// frame on a ~10000+ cell grid. Also recomputes on entering `Overview`
/// (`mode.is_changed()`) so switching from `Detail` never shows a stale
/// frame even if the population hasn't changed since the last time
/// `Overview` was active — recomputing then is cheap (it happens once per
/// mode switch, not per frame) and guarantees freshness without tracking a
/// separate "was this ever computed" flag.
fn update_cluster_density(
    world: Res<SimWorld>,
    config: Res<SimConfig>,
    mode: Res<MapViewMode>,
    mut density: ResMut<ClusterDensity>,
) {
    if *mode != MapViewMode::Overview {
        return;
    }
    if !world.is_changed() && !mode.is_changed() {
        return;
    }
    density.0 = compute_cluster_density(&world, &config);
}

/// Side length, in texels, of a generated shape mask (task 032). Independent
/// of `CELL_SIZE` — `Sprite::custom_size` stretches whatever texture
/// resolution to the on-screen cell, so this only controls how smooth the
/// shape's edge looks, not its footprint on the grid.
const SHAPE_TEXTURE_SIZE: u32 = 20;

/// One procedurally generated shape texture per `Metabolism` variant (task
/// 032 — a second visual dimension beyond species hue/energy lightness, so a
/// predator and a photolithic organism of similar hue stay distinguishable
/// at a glance). Generated once at `Startup`; `sync_grid_colors` swaps which
/// handle an occupied cell's `Sprite::image` points to every frame,
/// alongside its existing `Sprite::color` tint.
#[derive(Resource)]
struct MetabolismShapes {
    photolithic: Handle<Image>,
    predator: Handle<Image>,
    decomposer: Handle<Image>,
}

impl MetabolismShapes {
    fn handle_for(&self, metabolism: Metabolism) -> Handle<Image> {
        match metabolism {
            Metabolism::Photolithic => self.photolithic.clone(),
            Metabolism::Predator => self.predator.clone(),
            Metabolism::Decomposer => self.decomposer.clone(),
        }
    }
}

/// Builds a square RGBA texture from a coverage predicate evaluated over
/// normalized coordinates `[-1, 1]` (center of the texture at the origin):
/// opaque white where `inside` returns `true`, fully transparent elsewhere.
/// `Sprite::color`'s existing species/energy tint multiplies this alpha
/// mask, so the shape carries no color information of its own.
fn shape_mask_image(inside: impl Fn(f32, f32) -> bool) -> Image {
    let size = SHAPE_TEXTURE_SIZE;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for row in 0..size {
        for col in 0..size {
            let nx = (col as f32 + 0.5) / size as f32 * 2.0 - 1.0;
            let ny = (row as f32 + 0.5) / size as f32 * 2.0 - 1.0;
            let alpha = if inside(nx, ny) { 255 } else { 0 };
            data.extend_from_slice(&[255, 255, 255, alpha]);
        }
    }
    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// A filled circle — `Photolithic` (GDD §5.4's primary producer, the
/// "default" metabolism players see most).
fn circle_mask(nx: f32, ny: f32) -> bool {
    nx * nx + ny * ny <= 0.8 * 0.8
}

/// An upward-pointing triangle — `Predator`, evoking a fang/claw shape.
fn triangle_mask(nx: f32, ny: f32) -> bool {
    let apex_y = -0.9;
    let base_y = 0.8;
    let half_width = 0.85;
    if ny < apex_y || ny > base_y {
        return false;
    }
    let t = (ny - apex_y) / (base_y - apex_y);
    let bound = half_width * t;
    nx >= -bound && nx <= bound
}

/// A diamond (rotated square, Manhattan-norm disc) — `Decomposer`.
fn diamond_mask(nx: f32, ny: f32) -> bool {
    nx.abs() + ny.abs() <= 0.9
}

fn spawn_metabolism_shapes(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(MetabolismShapes {
        photolithic: images.add(shape_mask_image(circle_mask)),
        predator: images.add(shape_mask_image(triangle_mask)),
        decomposer: images.add(shape_mask_image(diamond_mask)),
    });
}

fn spawn_grid(mut commands: Commands, config: Res<SimConfig>) {
    let width = config.grid.width as usize;
    let height = config.grid.height as usize;
    for y in 0..height {
        for x in 0..width {
            commands.spawn((
                Sprite::from_color(Color::BLACK, Vec2::splat(CELL_SIZE)),
                Transform::from_translation(cell_position(x, y, width, height)),
                GridCell { x, y },
            ));
        }
    }
}

/// Sprites are spawned once (`Startup`); every tick only updates
/// `Sprite::color` and `Sprite::image` (task 032 — shape by metabolism),
/// read-only against `SimWorld` (never `ResMut` here — rendering must not
/// mutate simulation state).
/// `MapViewMode::Overview` (task 076) reuses these same per-cell sprites for
/// the cluster heatmap instead of spawning separate blob entities: coloring
/// every cell in a cluster by that cluster's shared density makes the
/// sprite union read as one blob whose shape is the cluster's real
/// footprint, and it comes with the click/viewport math (`world_to_cell`,
/// the zoom camera) already solved for free.
fn sync_grid_colors(
    world: Res<SimWorld>,
    config: Res<SimConfig>,
    shapes: Res<MetabolismShapes>,
    mode: Res<MapViewMode>,
    density: Res<ClusterDensity>,
    mut cells: Query<(&GridCell, &mut Sprite)>,
) {
    for (cell, mut sprite) in &mut cells {
        sprite.color = cell_color(&world, &config, cell.x, cell.y, *mode, &density);
        sprite.image = match *mode {
            MapViewMode::Detail => cell_shape(&world, &shapes, cell.x, cell.y),
            // Metabolism shapes are a Detail-only precision affordance
            // (task 032); Overview's aggregated blobs don't carry per-cell
            // identity, so cells fall back to the plain solid-square shape.
            MapViewMode::Overview => Handle::default(),
        };
    }
}

/// The shape texture for cell `(x, y)` (task 032): a metabolism-specific
/// mask for an occupied cell, or `Handle::default()` — the same implicit
/// solid-square fallback `Sprite::from_color` itself uses — for empty and
/// residue-only cells, which this task leaves visually unaffected.
fn cell_shape(world: &SimWorld, shapes: &MetabolismShapes, x: usize, y: usize) -> Handle<Image> {
    match world.get(x, y).organism {
        Some(organism) => {
            let metabolism = world.species[organism.species.0 as usize].metabolism;
            shapes.handle_for(metabolism)
        }
        None => Handle::default(),
    }
}

/// World-space position of cell `(x, y)`, grid centered at the origin.
/// Bevy's `y` grows upward while the row index grows downward, so row 0
/// (highest light, GDD §5.2) needs to land at the top of the screen.
fn cell_position(x: usize, y: usize, width: usize, height: usize) -> Vec3 {
    let wx = (x as f32 - (width as f32 - 1.0) / 2.0) * CELL_SIZE;
    let wy = ((height as f32 - 1.0) / 2.0 - y as f32) * CELL_SIZE;
    Vec3::new(wx, wy, 0.0)
}

/// Inverse of `cell_position`: the grid cell under a world-space point, or
/// `None` if the point falls outside `[0, width) x [0, height)` (task 017 —
/// clicks off the grid, e.g. on the HUD panel, must do nothing).
pub fn world_to_cell(world_pos: Vec2, width: usize, height: usize) -> Option<(usize, usize)> {
    let x = (world_pos.x / CELL_SIZE + (width as f32 - 1.0) / 2.0).round();
    let y = ((height as f32 - 1.0) / 2.0 - world_pos.y / CELL_SIZE).round();
    if x < 0.0 || y < 0.0 {
        return None;
    }
    let (x, y) = (x as usize, y as usize);
    if x >= width || y >= height {
        return None;
    }
    Some((x, y))
}

/// The single place that decides a cell's color (GDD §11): occupied cells by
/// species hue and energy (`Detail`) or cluster density (`Overview`, task
/// 076), cells with leftover residue by a neutral hue scaled by how much is
/// left, empty cells by their terrain band (task 066,
/// `redesign/abiogenesis-terrain-map.md` — flat color, no longer a
/// continuous `light` shading), then a toxicity tint (task 033) composited
/// on top of whichever of the three applies.
fn cell_color(
    world: &SimWorld,
    config: &SimConfig,
    x: usize,
    y: usize,
    mode: MapViewMode,
    density: &ClusterDensity,
) -> Color {
    let cell = world.get(x, y);

    let base = if let Some(organism) = cell.organism {
        let hue = species_hue(organism.species);
        let lightness = match mode {
            // Energy can exceed repro_threshold right before reproduction;
            // clamp.
            MapViewMode::Detail => {
                let fill = (organism.energy / config.energy.repro_threshold).clamp(0.0, 1.0);
                0.15 + fill * 0.35
            }
            // `compute_cluster_density` is a population-mass reading (large,
            // established colonies saturate toward `1.0`; a one-cell cluster
            // sits near the bottom), not Detail's per-organism energy fill,
            // so it gets its own lightness range rather than reusing
            // Detail's: a higher floor (`0.20`, above every `terrain_color`
            // band's lightness so a lone organism never blends into the
            // ground beneath it) keeps even a barely-above-zero density
            // "clearly visible" per the task's acceptance criteria, while
            // still reading strictly dimmer than a saturated colony's `0.50`
            // ceiling. Same hue formula as Detail either way, so the sidebar/
            // notebook swatches (`species_color`) still agree with the grid.
            MapViewMode::Overview => 0.20 + density.0[world.index(x, y)] * 0.30,
        };
        Color::hsl(hue, 0.75, lightness)
    } else if cell.residue > config.energy.residue_ambient_trickle {
        // `sim::step` settles every cell's residue at exactly
        // `residue_ambient_trickle` once no death has happened nearby (task
        // 060's ambient trickle, so an isolated Decomposer stays readable) —
        // that floor is present grid-wide, not just where something died.
        // Task 068's terrain colors made this visible for the first time:
        // treating any `residue > 0.0` as "there's a corpse here" painted
        // every single cell brown once the trickle established, hiding the
        // terrain underneath. Only residue *above* the ambient floor is a
        // real death's leftover worth showing.
        let intensity = (cell.residue / config.energy.residue_on_death).clamp(0.0, 1.0);
        Color::hsl(30.0, 0.2, 0.08 + intensity * 0.22)
    } else {
        terrain_color(cell.terrain)
    };

    toxicity_tint(base, cell.toxicity)
}

/// Flat per-band terrain color (task 068 — the mockup's "no gradient, one
/// flat color per elevation band" rule): `Sea` stays near-black, close to
/// the grid's own background, so it still reads as "void" the way the
/// pre-terrain empty cell did, even though it's now real data rather than
/// an absence of one. `Plain`/`Hill`/`Mountain` are desaturated tones from
/// the same console/lab palette family as the rest of the HUD, not a new
/// palette.
fn terrain_color(kind: TerrainKind) -> Color {
    match kind {
        TerrainKind::Sea => Color::hsl(0.0, 0.0, 0.02),
        TerrainKind::Plain => Color::hsl(130.0, 0.20, 0.09),
        TerrainKind::Hill => Color::hsl(125.0, 0.22, 0.14),
        TerrainKind::Mountain => Color::hsl(30.0, 0.10, 0.19),
    }
}

/// A warning-magenta hue to blend toward as `toxicity` rises (GDD §5.2's
/// toxic zone). Composited over the organism/residue/empty base color
/// rather than replacing it, so a toxic cell holding an organism still
/// reads as that organism, just visibly tainted — capped at a 45% blend
/// even at `toxicity = 1.0` so it stays a tint, not a wash.
///
/// Purely a legibility fix (task 033 — the toxic zone was previously
/// invisible outside the dev-only `F1` overlay): `toxicity` has no effect on
/// `sim::step`'s tick arithmetic today (task 023's finding), and whether it
/// ever should is a separate balance decision this doesn't make. The visual
/// and the (currently absent) mechanical effect are not the same thing.
fn toxicity_tint(base: Color, toxicity: f32) -> Color {
    let warning = Color::hsl(320.0, 0.9, 0.35);
    base.mix(&warning, toxicity.clamp(0.0, 1.0) * 0.45)
}

#[cfg(test)]
mod tests {
    use super::*;
    use abiogenesis::world::{Cell, Organism, SpeciesId, TerrainKind};

    /// `cell_color` in `Detail` mode, exactly as it behaved before task 076
    /// added the `Overview` branch — the density argument is unused on this
    /// path, so an empty `ClusterDensity` is fine for every test below.
    fn detail_color(world: &SimWorld, config: &SimConfig, x: usize, y: usize) -> Color {
        cell_color(
            world,
            config,
            x,
            y,
            MapViewMode::Detail,
            &ClusterDensity::default(),
        )
    }

    #[test]
    fn heat_color_pins_the_blue_cold_to_red_hot_endpoints() {
        let Color::Hsla(cold) = heat_color(0.0) else {
            panic!("expected Hsla");
        };
        let Color::Hsla(hot) = heat_color(1.0) else {
            panic!("expected Hsla");
        };
        assert!((cold.hue - 240.0).abs() < f32::EPSILON);
        assert!((hot.hue - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn occupied_cells_are_saturated_and_residue_desaturated() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let (x, y) = (0, 0);
        let idx = world.index(x, y);

        world.cells[idx] = Cell {
            organism: Some(Organism {
                species: SpeciesId(0),
                energy: 5.0,
                born_era: 0,
            }),
            ..world.cells[idx]
        };
        let Color::Hsla(occupied) = detail_color(&world, &config, x, y) else {
            panic!("expected an HSL color");
        };
        assert!(occupied.saturation > 0.5);

        world.cells[idx].organism = None;
        world.cells[idx].residue = config.energy.residue_on_death;
        let Color::Hsla(residue) = detail_color(&world, &config, x, y) else {
            panic!("expected an HSL color");
        };
        assert!(residue.saturation < occupied.saturation);

        // Task 068: empty cells render by terrain band (flat color), not a
        // continuous grayscale `light` shading — pin the terrain so the
        // assertion doesn't depend on whatever this seed's generator drew
        // for (0, 0).
        world.cells[idx].residue = 0.0;
        world.cells[idx].terrain = TerrainKind::Plain;
        let Color::Hsla(empty) = detail_color(&world, &config, x, y) else {
            panic!("expected an HSL color");
        };
        assert_eq!(empty, terrain_hsla(TerrainKind::Plain));
        assert!(empty.lightness < residue.lightness);
    }

    #[test]
    fn ambient_residue_trickle_does_not_hide_the_terrain_color() {
        // Task 060's ambient trickle settles every cell's residue at exactly
        // `residue_ambient_trickle` grid-wide, not just where something
        // died — `cell_color` must not treat that floor as "there's a
        // corpse here" (task 070 regression: it did, painting the whole map
        // brown once the trickle established after the first tick).
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let (x, y) = (0, 0);
        let idx = world.index(x, y);
        world.cells[idx].organism = None;
        world.cells[idx].residue = config.energy.residue_ambient_trickle;
        world.cells[idx].terrain = TerrainKind::Plain;
        world.cells[idx].toxicity = 0.0;

        assert_eq!(
            detail_color(&world, &config, x, y),
            terrain_color(TerrainKind::Plain)
        );
    }

    fn terrain_hsla(kind: TerrainKind) -> bevy::color::Hsla {
        let Color::Hsla(hsla) = terrain_color(kind) else {
            panic!("expected an HSL color");
        };
        hsla
    }

    #[test]
    fn each_terrain_band_has_a_distinct_flat_color() {
        let colors = [
            TerrainKind::Sea,
            TerrainKind::Plain,
            TerrainKind::Hill,
            TerrainKind::Mountain,
        ]
        .map(terrain_hsla);
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j], "bands {i} and {j} share a color");
            }
        }
    }

    #[test]
    fn sea_stays_near_black() {
        let sea = terrain_hsla(TerrainKind::Sea);
        assert!(
            sea.lightness < 0.05,
            "Sea must read as close to the grid's black background, not a water fill"
        );
    }

    #[test]
    fn toxic_cells_render_differently_than_clean_ones() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let (x, y) = (0, 0);
        let idx = world.index(x, y);
        world.cells[idx].toxicity = 0.0;
        let clean = detail_color(&world, &config, x, y);

        world.cells[idx].toxicity = 1.0;
        let toxic = detail_color(&world, &config, x, y);

        assert_ne!(clean, toxic);
    }

    #[test]
    fn world_to_cell_round_trips_with_cell_position() {
        let (width, height) = (48, 32);
        for (x, y) in [(0, 0), (47, 0), (0, 31), (47, 31), (24, 16)] {
            let pos = cell_position(x, y, width, height).truncate();
            assert_eq!(
                world_to_cell(pos, width, height),
                Some((x, y)),
                "cell ({x}, {y}) did not round-trip"
            );
        }
    }

    #[test]
    fn world_to_cell_rejects_points_outside_the_grid() {
        let (width, height) = (48, 32);
        let far_beyond = Vec2::splat(10_000.0);
        assert_eq!(world_to_cell(far_beyond, width, height), None);
        assert_eq!(world_to_cell(-far_beyond, width, height), None);
    }

    #[test]
    fn cell_position_centers_the_grid_and_flips_y() {
        // Row 0 (top light band) must render above row `height - 1`.
        let top = cell_position(0, 0, 48, 32);
        let bottom = cell_position(0, 31, 48, 32);
        assert!(top.y > bottom.y);
        // The grid is centered on the origin.
        assert_eq!(
            cell_position(0, 0, 48, 32).x,
            -cell_position(47, 0, 48, 32).x
        );
    }

    #[test]
    fn clamp_camera_pan_leaves_in_bounds_translation_untouched() {
        let unscaled_area = Vec2::new(2048.0, 1280.0);
        let grid_extent = Vec2::new(2048.0, 1280.0);
        // At `scale == 0.1`, the visible area is a small fraction of the
        // grid, so a modest translation stays well within bounds.
        let translation = Vec2::new(50.0, -20.0);
        assert_eq!(
            clamp_camera_pan(translation, 0.1, unscaled_area, grid_extent),
            translation
        );
    }

    #[test]
    fn clamp_camera_pan_stops_at_the_grid_edge() {
        let unscaled_area = Vec2::new(2048.0, 1280.0);
        let grid_extent = Vec2::new(2048.0, 1280.0);
        let max_pan = ((grid_extent - unscaled_area * 0.1) / 2.0).max(Vec2::ZERO);
        // A translation far beyond the grid's own extent clamps to the
        // computed edge, on both axes, in both directions (task 087).
        let clamped = clamp_camera_pan(Vec2::splat(10_000.0), 0.1, unscaled_area, grid_extent);
        assert_eq!(clamped, max_pan);
        let clamped_neg = clamp_camera_pan(Vec2::splat(-10_000.0), 0.1, unscaled_area, grid_extent);
        assert_eq!(clamped_neg, -max_pan);
    }

    #[test]
    fn clamp_camera_pan_locks_to_zero_at_whole_grid_zoom() {
        let unscaled_area = Vec2::new(2048.0, 1280.0);
        let grid_extent = Vec2::new(2048.0, 1280.0);
        // At `scale == zoom_max == 1.0`, the visible area already equals
        // the grid extent, so no pan is ever valid — this is what makes
        // Overview's "whole grid visible" guarantee hold even with the
        // pan system active.
        let clamped = clamp_camera_pan(Vec2::new(500.0, -500.0), 1.0, unscaled_area, grid_extent);
        assert_eq!(clamped, Vec2::ZERO);
    }

    #[test]
    fn seen_relations_marks_a_pair_seen_only_once() {
        let mut seen = SeenRelations::new(3);
        assert!(seen.mark_seen(TagSlot(0), TagSlot(1)));
        assert!(!seen.mark_seen(TagSlot(0), TagSlot(1)));
        // A different pair is tracked independently.
        assert!(seen.mark_seen(TagSlot(1), TagSlot(0)));
    }

    fn write_adjacency_observed(app: &mut App, event: AdjacencyObserved) {
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<AdjacencyObserved>>()
            .write(event);
    }

    /// Mirrors `notebook.rs`'s `accumulate_evidence` test pattern: a scratch
    /// `App` with just the resources/system under test, driven via
    /// `Messages<AdjacencyObserved>` directly.
    #[test]
    fn spawn_spark_on_first_observation_fires_once_per_pair_not_per_event() {
        let config = SimConfig::default();
        let world = SimWorld::new(1, &config);
        let mut app = App::new();
        app.insert_resource(world);
        app.insert_resource(SeenRelations::new(3));
        app.init_resource::<SparkIndicators>();
        app.add_message::<AdjacencyObserved>();
        app.add_systems(Update, spawn_spark_on_first_observation);

        let event = AdjacencyObserved {
            receiver_species: SpeciesId(0),
            exerter_tag: TagSlot(0),
            receiver_tag: TagSlot(1),
            n_confounders: 0,
            cell: 5,
        };
        write_adjacency_observed(&mut app, event);
        app.update();
        assert_eq!(
            app.world().resource::<SparkIndicators>().len(),
            1,
            "the first-ever observation of a pair must spawn exactly one spark"
        );

        // Same pair observed again (different cell) must not spawn a second
        // spark — the gate is per-pair, not per-event.
        write_adjacency_observed(&mut app, AdjacencyObserved { cell: 6, ..event });
        app.update();
        assert_eq!(
            app.world().resource::<SparkIndicators>().len(),
            1,
            "a repeat observation of an already-seen pair must not spawn another spark"
        );

        // A different pair must spawn a second, independent spark.
        write_adjacency_observed(
            &mut app,
            AdjacencyObserved {
                exerter_tag: TagSlot(1),
                receiver_tag: TagSlot(2),
                cell: 7,
                ..event
            },
        );
        app.update();
        assert_eq!(
            app.world().resource::<SparkIndicators>().len(),
            2,
            "a genuinely new pair must spawn its own spark"
        );
    }
}
