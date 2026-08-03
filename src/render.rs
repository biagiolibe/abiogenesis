use bevy::camera::ScalingMode;
use bevy::prelude::*;

use abiogenesis::config::SimConfig;
use abiogenesis::world::SimWorld;

/// Pixel size of one grid cell on screen. Presentation-only, not a
/// simulation coefficient, so it stays local instead of living in
/// `SimConfig` (invariant 3 covers balance numbers, not pixel sizes).
const CELL_SIZE: f32 = 16.0;

/// Golden-angle hue step: successive `SpeciesId`s get visually distinct
/// hues without any per-species color configuration.
const SPECIES_HUE_STEP: f32 = 137.5;

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
        app.add_systems(Startup, (spawn_camera, spawn_grid))
            .add_systems(Update, sync_grid_colors);
        #[cfg(debug_assertions)]
        {
            app.init_resource::<debug_view::DebugView>().add_systems(
                Update,
                (
                    debug_view::toggle_debug_view,
                    debug_view::apply_debug_view.after(sync_grid_colors),
                ),
            );
        }
    }
}

/// A dev-only heatmap overlay for the raw environment scalars (`Stress`,
/// task 023, revealed the need: `toxicity` has no visible effect anywhere,
/// which was only discoverable by grepping the tick code, not by playing).
/// Never compiled into a release build — the game's whole deduction pillar
/// (GDD §7, §11) is built on the player *not* having direct instrument
/// readouts, so this must not leak past development.
#[cfg(debug_assertions)]
mod debug_view {
    use super::GridCell;
    use bevy::prelude::*;

    use abiogenesis::world::SimWorld;

    /// `F1` cycles through these. `Normal` defers entirely to `cell_color` —
    /// this module never touches rendering unless a non-`Normal` view is
    /// active.
    #[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
    pub enum DebugView {
        #[default]
        Normal,
        Temperature,
        Toxicity,
        Light,
    }

    impl DebugView {
        fn next(self) -> Self {
            match self {
                DebugView::Normal => DebugView::Temperature,
                DebugView::Temperature => DebugView::Toxicity,
                DebugView::Toxicity => DebugView::Light,
                DebugView::Light => DebugView::Normal,
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
                DebugView::Temperature => world.get(cell.x, cell.y).temperature,
                DebugView::Toxicity => world.get(cell.x, cell.y).toxicity,
                DebugView::Light => world.get(cell.x, cell.y).light,
            };
            sprite.color = heat_color(scalar);
        }
    }

    /// Blue (0.0, cold/low) to red (1.0, hot/high) through the hue wheel —
    /// a standard heatmap gradient, not tied to any in-game color meaning.
    fn heat_color(value: f32) -> Color {
        let hue = 240.0 * (1.0 - value.clamp(0.0, 1.0));
        Color::hsl(hue, 0.85, 0.5)
    }
}

fn spawn_camera(mut commands: Commands, config: Res<SimConfig>) {
    let width = config.grid.width as f32 * CELL_SIZE;
    let height = config.grid.height as f32 * CELL_SIZE;
    commands.spawn((
        Camera2d,
        GridCamera,
        Projection::Orthographic(OrthographicProjection {
            // Never smaller than the grid, so it never clips on resize; a
            // wider/taller window just shows more letterboxed space.
            scaling_mode: ScalingMode::AutoMin {
                min_width: width,
                min_height: height,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
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

/// Sprites are spawned once (`Startup`); every tick only updates `Sprite::color`,
/// read-only against `SimWorld` (never `ResMut` here — rendering must not
/// mutate simulation state).
fn sync_grid_colors(
    world: Res<SimWorld>,
    config: Res<SimConfig>,
    mut cells: Query<(&GridCell, &mut Sprite)>,
) {
    for (cell, mut sprite) in &mut cells {
        sprite.color = cell_color(&world, &config, cell.x, cell.y);
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
/// species hue and energy, cells with leftover residue by a neutral hue
/// scaled by how much is left, empty cells by a faint shading of `light`.
fn cell_color(world: &SimWorld, config: &SimConfig, x: usize, y: usize) -> Color {
    let cell = world.get(x, y);

    if let Some(organism) = cell.organism {
        let hue = (organism.species.0 as f32 * SPECIES_HUE_STEP) % 360.0;
        // Energy can exceed repro_threshold right before reproduction; clamp.
        let fill = (organism.energy / config.energy.repro_threshold).clamp(0.0, 1.0);
        return Color::hsl(hue, 0.75, 0.15 + fill * 0.35);
    }

    if cell.residue > 0.0 {
        let intensity = (cell.residue / config.energy.residue_on_death).clamp(0.0, 1.0);
        return Color::hsl(30.0, 0.2, 0.08 + intensity * 0.22);
    }

    Color::hsl(0.0, 0.0, 0.03 + cell.light * 0.12)
}

#[cfg(test)]
mod tests {
    use super::*;
    use abiogenesis::world::{Cell, Organism, SpeciesId};

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
            }),
            ..world.cells[idx]
        };
        let Color::Hsla(occupied) = cell_color(&world, &config, x, y) else {
            panic!("expected an HSL color");
        };
        assert!(occupied.saturation > 0.5);

        world.cells[idx].organism = None;
        world.cells[idx].residue = config.energy.residue_on_death;
        let Color::Hsla(residue) = cell_color(&world, &config, x, y) else {
            panic!("expected an HSL color");
        };
        assert!(residue.saturation < occupied.saturation);

        world.cells[idx].residue = 0.0;
        let Color::Hsla(empty) = cell_color(&world, &config, x, y) else {
            panic!("expected an HSL color");
        };
        assert_eq!(empty.saturation, 0.0);
        assert!(empty.lightness < residue.lightness);
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
}
