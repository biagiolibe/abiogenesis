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
use abiogenesis::objectives::ZoneKind;

// --- Main menu (`menu.rs::main_menu_ui`) ---

pub const MENU_TITLE: &str = "Abiogenesis";
pub const MENU_SEED_LABEL: &str = "Run seed (leave blank to generate one)";
pub const MENU_SEED_HINT: &str = "e.g. 42";
pub const MENU_NEW_RUN_BUTTON: &str = "New run";

// --- Intro screen (`screens.rs::intro_screen_ui`, task 052) ---

pub const INTRO_TITLE: &str = "A sterile world";
pub const INTRO_BODY: &str = "You seed an alien ecosystem and watch it grow — but its biochemistry is hidden. Species interact through a tag-based matrix you can't see directly, only infer from what happens when they meet. Each era is one deliberate experiment: seed, stress, cull, or splice within a limited budget, then observe.";
pub const INTRO_CONTINUE_BUTTON: &str = "Begin";

// --- World-cleared / defeat screens (`screens.rs`) ---

pub const WORLD_CLEARED_TITLE: &str = "World cleared!";
pub const CONTINUE_BUTTON: &str = "Continue";
pub const WORLD_FAILED_TITLE: &str = "World failed";
pub const RETRY_BUTTON: &str = "Retry";
pub const DEFEAT_TITLE: &str = "Run ended";
pub const RETURN_TO_MENU_BUTTON: &str = "Return to menu";

pub fn world_cleared_body(world_index: u32) -> String {
    format!("World {world_index}'s objective is met. The next world will be harder.")
}

/// Task 051: total extinction ends the world, not the run — the player
/// retries the exact same world (same seed), the run itself is unaffected.
pub const WORLD_FAILED_BODY: &str = "Every organism on this world has died out. Retry this world.";

pub fn defeat_body(worlds_cleared: u32) -> String {
    format!("This run cleared {worlds_cleared} world(s) before ending.")
}

// --- Meta-progression summary (`menu.rs::main_menu_ui`) ---

pub const NO_UNLOCKS_YET: &str = "No unlocks yet — clear worlds to earn more starting species.";

pub fn unlocks_summary(bonus_available_species: u32) -> String {
    if bonus_available_species == 0 {
        NO_UNLOCKS_YET.to_string()
    } else {
        format!("Unlocked: {bonus_available_species} extra species available at the start of a run")
    }
}

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

// --- Viewport onboarding hints (task 053) ---

pub const HINT_PLACE_FIRST_ORGANISM: &str =
    "Pick a species in the Seed palette, then click an empty cell to place it";
pub const HINT_OPEN_NOTEBOOK: &str = "Press tab to open your notebook and log hypotheses";

// --- HUD — notebook affordance badge (task 054) ---

pub const NOTEBOOK_AFFORDANCE_LABEL: &str = "Notebook (tab)";
pub const NOTEBOOK_BADGE_GLYPH: &str = "★";
pub const NOTEBOOK_BADGE_HOVER: &str =
    "A hypothesis was just confirmed — open the notebook to see it";

// --- Viewport onboarding hints — guided first-isolation hint (task 055) ---

pub const HINT_ISOLATED_FIRST_PLACEMENT: &str =
    "You isolated this species — watch its energy over the next few ticks for a clean first reading";
pub const HINT_CLUSTERED_FIRST_PLACEMENT: &str =
    "Tip: an isolated species gives cleaner readings — try it in a future era";

// --- HUD — objective panel (`ui.rs::objective_panel`) ---

pub const HEADING_OBJECTIVE: &str = "Objective";
pub const NO_OBJECTIVE: &str = "(no objective assigned yet)";
pub const OBJECTIVE_CLEARED: &str = "Cleared!";
pub const BLOOM_NOT_TRIGGERED: &str = "not yet triggered";
pub const BLOOM_TRIGGERED: &str = "triggered!";

pub fn zone_label(zone: ZoneKind) -> &'static str {
    match zone {
        ZoneKind::Toxic => "toxic zone",
    }
}

pub fn coexistence_objective_line(min_species: u32) -> String {
    format!("Sustain {min_species} coexisting species")
}

pub fn survive_in_objective_line(species_label: &str, zone_label: &str) -> String {
    format!("{species_label} survives in the {zone_label}")
}

pub fn trigger_bloom_objective_line(species_label: &str, population_threshold: u32) -> String {
    format!("{species_label} population reaches {population_threshold}")
}

/// `eras_held`/`eras_required` (task 049) — whole eras, not raw ticks: the
/// player's own unit (GDD §11), converted by `ui.rs::eras_progress` before
/// this ever gets called.
pub fn sustained_progress_bar_text(eras_held: u32, eras_required: u32) -> String {
    format!("{eras_held} / {eras_required} eras")
}

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

/// Breaks a death down into the energy-update terms that caused it (GDD
/// §5.6 step 5), so the log answers "why" instead of only "what": each term
/// is shown as its actual net contribution (costs negated), not the raw
/// magnitude stored on `OrganismDied`.
#[allow(clippy::too_many_arguments)]
pub fn player_organism_death_message(
    species_label: &str,
    x: usize,
    y: usize,
    gain: f32,
    interaction_delta: f32,
    upkeep: f32,
    crowding_penalty: f32,
    predation_loss: f32,
) -> String {
    // `+ 0.0` folds away IEEE-754 negative zero (e.g. `-upkeep` when
    // `upkeep == 0.0`) so the message never reads "predation -0.00".
    format!(
        "your {species_label} organism at ({x}, {y}) died: gain {gain:+.2}, \
         matrix {interaction_delta:+.2}, upkeep {:+.2}, crowding {:+.2}, predation {:+.2}",
        -upkeep + 0.0,
        -crowding_penalty + 0.0,
        -predation_loss + 0.0,
    )
}

/// A matrix cell crossing the confirmation threshold (GDD §7's "aha"
/// moment, task 054) — distinguished from ordinary log lines by a leading
/// glyph the caller renders separately (`notebook.rs`'s `CONFIRMATION_GLYPH`),
/// same split `LogEntry`/`species_color` uses for species-subject lines.
pub fn confirmation_message(from_glyph: &str, to_glyph: &str, positive: bool) -> String {
    let sign = if positive { "boosts" } else { "harms" };
    format!("Confirmed: tag {from_glyph} {sign} tag {to_glyph}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn death_message_shows_costs_as_negative_net_contributions() {
        let message =
            player_organism_death_message("Nyx (species 0)", 3, 4, 0.40, 0.0, 0.70, 0.15, 0.0);
        assert!(
            message.contains("gain +0.40"),
            "gain should show its raw positive contribution: {message}"
        );
        assert!(
            message.contains("upkeep -0.70"),
            "upkeep is a cost, must show as negative: {message}"
        );
        assert!(
            message.contains("crowding -0.15"),
            "crowding is a cost, must show as negative: {message}"
        );
        assert!(
            message.contains("predation +0.00"),
            "zero predation loss must not print as negative zero: {message}"
        );
    }
}
