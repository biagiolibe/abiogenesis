// Every player-facing string in the game — HUD, notebook, tooltips, event
// log — lives here (task 034), so a future localization pass has one module
// to touch instead of `ui.rs` + `notebook.rs` + wherever text creeps in
// next. No actual i18n/loader yet: strings stay hardcoded in English, this
// is purely about *where* the text lives, not how it's chosen at runtime.
//
// Out of scope, deliberately: `render::species_label`, `notebook::tag_glyph`
// and `notebook::tag_color` (task 029) generate per-entity display
// names/glyphs from `SimWorld` state — those are data, not static copy, and
// this module never reaches into `SimWorld` itself.

use crate::ui::ActionMode;

// --- HUD — world state (`ui.rs::hud_panel`) ---

pub const HEADING_TITLE: &str = "Abiogenesis";

pub fn era_tick_line(era: u32, tick: u64) -> String {
    format!("Era {era}  ·  tick {tick}")
}

pub fn seed_line(seed: u64) -> String {
    format!("Seed {seed}")
}

pub fn state_line(state: impl std::fmt::Debug) -> String {
    format!("State: {state:?}")
}

// --- HUD — action group ---

pub const HEADING_ACTION: &str = "Action";
pub const BUDGET_HOVER: &str = "Action points remaining this era";

pub fn budget_bar_text(remaining: u32, total: u32) -> String {
    format!("{remaining} / {total}")
}

/// Display name for one `ActionMode`, used both in the icon row's hover
/// text and anywhere else an action needs a human label.
pub fn action_name(mode: ActionMode) -> &'static str {
    match mode {
        ActionMode::Seed => "Seed",
        ActionMode::Stress => "Stress",
        ActionMode::Cull => "Cull",
        ActionMode::Splice => "Splice",
    }
}

/// One-line description of what clicking with this `ActionMode` selected
/// actually does (`input.rs`'s `*_on_click` systems) — not GDD §6's more
/// general "an area" wording, since every action here targets the single
/// clicked cell.
pub fn action_description(mode: ActionMode) -> &'static str {
    match mode {
        ActionMode::Seed => "Place an organism of the selected species on an empty cell.",
        ActionMode::Stress => "Raise the temperature of the clicked cell.",
        ActionMode::Cull => "Remove the organism on the clicked cell, if any.",
        ActionMode::Splice => {
            "Edit a species' genome: swap or add a tag, or shift its thermal optimum."
        }
    }
}

pub fn action_tooltip(mode: ActionMode, cost: u32) -> String {
    let name = action_name(mode);
    let description = action_description(mode);
    format!("{name} · cost {cost}\n{description}")
}

// --- HUD — population group ---

pub const HEADING_POPULATION: &str = "Population";
pub const NO_POPULATION: &str = "  (none)";

pub fn population_line(species_label: &str, population: usize, avg_energy: f32) -> String {
    format!("  {species_label}: {population} · avg energy {avg_energy:.2}")
}

// --- HUD — seed palette group ---

pub const HEADING_SEED_PALETTE: &str = "Seed palette";
pub const SEED_PALETTE_HOVER: &str = "Click an empty cell to place the selected species";
pub const KEYBOARD_HINT: &str = "space era · s tick · r reseed · Esc quit";

// --- HUD — Splice editor (`ui.rs::splice_panel`) ---

pub const SPLICE_SOURCE_LABEL: &str = "Splice: source species";
pub const EDIT_LABEL: &str = "Edit";
pub const SWAP_TAG_OPTION: &str = "Swap a tag";
pub const ADD_TAG_OPTION: &str = "Add a tag";
pub const TAG_CAP_HINT: &str = "  (source already has 3 tags)";
pub const SHIFT_TEMP_OPTION: &str = "Shift temperature optimum";
pub const REMOVE_TAG_LABEL: &str = "Remove tag:";
pub const ADD_TAG_LABEL: &str = "Add tag:";
pub const PICK_SOURCE_HINT: &str = "  (pick a source species first)";
pub const WARMER_OPTION: &str = "warmer";
pub const COLDER_OPTION: &str = "colder";
pub const APPLY_SPLICE_BUTTON: &str = "Apply splice";

pub fn tag_option_label(glyph: &str) -> String {
    format!("tag {glyph}")
}

// --- Notebook — observation log (`notebook.rs::notebook_window`) ---

pub const HEADING_OBSERVATION_LOG: &str = "Observation log";
pub const NO_OBSERVATIONS_YET: &str = "(no observations yet)";

pub fn log_entry_line(era: u32, text: &str) -> String {
    format!("Era {era}: {text}")
}

pub fn extinction_message(species_label: &str) -> String {
    format!("{species_label} went extinct")
}

pub fn player_organism_death_message(species_label: &str, x: usize, y: usize) -> String {
    format!("your {species_label} organism at ({x}, {y}) died")
}

// --- Notebook — hypothesis graph (`notebook.rs::hypothesis_grid`) ---

pub const HEADING_HYPOTHESIS_GRID: &str = "Hypothesis grid";

pub fn node_tag_line(glyph: &str) -> String {
    format!("Tag {glyph}")
}

pub fn confirmed_relation_line(from_glyph: &str, to_glyph: &str, positive: bool) -> String {
    let sign = if positive { "+" } else { "-" };
    format!("{from_glyph} → {to_glyph} ({sign})")
}

pub fn partial_relation_line(from_glyph: &str, to_glyph: &str) -> String {
    format!("{from_glyph} → {to_glyph} (some evidence)")
}

// --- Notebook — catalog (`notebook.rs::catalog_panel`) ---

pub const HEADING_CATALOG: &str = "Catalog";
pub const ACTIVE_TAGS_LABEL: &str = "Active tags";
pub const SPECIES_HEADING: &str = "Species";

pub fn species_catalog_line(
    species_label: &str,
    metabolism: impl std::fmt::Debug,
    temp_optimum: f32,
    temp_tolerance: f32,
) -> String {
    format!("{species_label}: {metabolism:?} · temp {temp_optimum:.2}±{temp_tolerance:.2}")
}
