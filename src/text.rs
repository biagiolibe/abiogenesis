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
use abiogenesis::world::Metabolism;

// --- Main menu (`menu.rs::main_menu_ui`) ---

pub const MENU_TITLE: &str = "Abiogenesis";
pub const MENU_SEED_LABEL: &str = "Run seed (leave blank to generate one)";
pub const MENU_SEED_HINT: &str = "e.g. 42";
pub const MENU_NEW_RUN_BUTTON: &str = "New run";

// --- How-to-play guide (task 056) ---
//
// A condensed companion to `player_guide.md` (the repo's full manual) — not
// a runtime render of that file: the engine has no markdown renderer, and
// task 034's "every player-facing string lives in text.rs" convention means
// this panel gets its own copy rather than reaching outside the crate for
// content. Rendered in two places, same `HOW_TO_PLAY_SECTIONS` array both
// times (one heading+body pair per entry, in an `egui::ScrollArea`):
// `screens.rs::intro_screen_ui` shows it automatically, once, before the
// player's very first "Begin"; `menu.rs::main_menu_ui`'s toggle button makes
// it available again on every later visit to the main menu, since the intro
// screen itself never shows a second time (`MetaProgress::seen_intro`).

pub const HOW_TO_PLAY_SHOW_BUTTON: &str = "How to play";
pub const HOW_TO_PLAY_HIDE_BUTTON: &str = "Hide guide";

pub const HOW_TO_PLAY_SECTIONS: &[(&str, &str)] = &[
    (
        "The premise",
        "You're a xenobiologist seeding an alien ecosystem whose biochemistry is hidden. \
         Species interact through tags you can't see directly, only infer from what happens \
         when they meet — that hidden tag × tag matrix is different every run.",
    ),
    (
        "Controls",
        "Left click: perform the selected action on a cell. Space: advance one era. \
         N: advance a single tick. Arrow keys or WASD: pan the camera. Tab: open your notebook. \
         R: reseed the current world. Esc: quit.",
    ),
    (
        "The loop",
        "Seed organisms, advance an era, observe what happened, form a hypothesis and spend \
         your budget testing it, repeat until the world's objective is met.",
    ),
    (
        "Metabolism and temperature",
        "Photolithic draws energy from light, Predator from neighboring organisms, \
         Decomposer from residue in its own or a neighboring cell. Every one of these gains \
         is then multiplied by how close that cell's temperature is to the species' comfort \
         zone — not added as a separate cost. An organism can die right next to its fuel \
         (residue, light, prey) if the temperature fit is poor; check the death log's gain \
         number before suspecting a hidden matrix effect.",
    ),
    (
        "Actions and budget",
        "Each era gives a small budget of points: Seed and Stress and Cull cost 1, Splice \
         (editing a species' genome) costs 2. You can't do everything — bet on your best \
         hypothesis.",
    ),
    (
        "The notebook",
        "Every adjacency between tagged organisms is a data point, weighted by how isolated \
         it was — a clean, uncrowded pairing counts far more than one buried in a crowd. \
         Once evidence for a tag pair adds up enough, it's confirmed and lights up in your \
         hypothesis grid.",
    ),
    (
        "Objectives and failure",
        "Each world sets a sequence of 2-3 goals (coexistence, surviving a hostile zone, or \
         triggering a bloom), cleared one after another. Total extinction retries the same \
         world; running out of eras ends the run. You only need to decode the part of the \
         matrix relevant to your objectives. Every world also grants a grace period: it can't \
         end from extinction until you've kept a population alive for a full era at least once.",
    ),
];

// --- Intro screen (`screens.rs::intro_screen_ui`, task 052) ---

pub const INTRO_TITLE: &str = "A sterile world";
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

/// Shown while `objectives::is_grace_active` is true (task 079's onboarding
/// grace period). No countdown number: the window is adaptive, not a fixed
/// length, so a number would go stale/misleading once it's extended past
/// `grace_eras`.
pub const GRACE_PERIOD_LINE: &str = "Grace period — this world can't fail from extinction yet";

// --- HUD — time control row (task 094) ---
//
// On-screen equivalents of the keyboard-only tick/era/notebook shortcuts —
// additive, the shortcuts keep working unchanged. `disabled` variants
// append a reason when `EraState::Advancing` already has the disabled
// button greyed out, same "why is this not clickable" affordance
// `DETAIL_MODE_ONLY_HINT` gives the action icon row.

pub const TICK_BUTTON_LABEL: &str = "⏵ Tick";
pub const ERA_BUTTON_LABEL: &str = "⏩ Era";
pub const NOTEBOOK_BUTTON_LABEL: &str = "📓 Notebook";

pub const TICK_BUTTON_TOOLTIP: &str = "Advance one tick (N)";
pub const ERA_BUTTON_TOOLTIP: &str = "Start/resume this era (Space)";
pub const NOTEBOOK_BUTTON_TOOLTIP: &str = "Open/close the notebook (Tab)";

pub const ADVANCING_DISABLED_HINT: &str = "\n(era already advancing)";

// --- HUD — action group ---

/// "Moves" (task 064's sidebar redesign — diegetic relabeling of the four
/// HUD sections, revised with the user past a first, too-formal English
/// pass): the action selector and per-era budget.
pub const HEADING_ACTION: &str = "Moves";
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

/// Appended to `action_tooltip`'s output when Stress/Cull are disabled
/// outside `MapViewMode::Detail` (task 077).
pub const DETAIL_MODE_ONLY_HINT: &str = "\n(Detail mode only — zoom in to use)";

pub fn action_tooltip(mode: ActionMode, cost: u32) -> String {
    let name = action_name(mode);
    let description = action_description(mode);
    format!("{name} · cost {cost}\n{description}")
}

// --- HUD — population group ---

/// "Biosphere" (task 064): the per-species population/trend list.
pub const HEADING_POPULATION: &str = "Biosphere";
pub const NO_POPULATION: &str = "  (none)";
/// Shown under the Biosphere list (task 064) only when it holds more rows
/// than fit in its fixed-height scroll area — the redesign mockup's
/// gradient-fade affordance, approximated as a plain hint label rather than
/// a true CSS-style fade (cheaper in egui's immediate-mode painter, and the
/// redesign doc explicitly allows the simplification).
pub const SCROLL_FOR_MORE: &str = "scroll for more";

/// The reproduction-threshold comparison this line used to show
/// (`avg_energy/repro_threshold`) compared a population *average* against a
/// per-*individual* trait — misleading, since the average could sit well
/// below threshold while individual organisms had already crossed it (task
/// 063). `repro_threshold` moved to `species_catalog_line`, where it
/// belongs as a static species trait; this line keeps the raw average and
/// lets `ui.rs` render the era-over-era trend (`PopulationTrend`) as a
/// separate colored glyph alongside it, the same way it already renders the
/// species-color swatch separately from this string.
pub fn population_line(species_label: &str, population: usize, avg_energy: f32) -> String {
    format!("  {species_label}: {population} · energy {avg_energy:.2}")
}

// --- HUD — seed palette group ---

/// "Species" (task 064): the horizontally scrollable species-selection strip.
pub const HEADING_SEED_PALETTE: &str = "Species";
pub const SEED_PALETTE_HOVER: &str = "Click an empty cell to place the selected species";
/// Split into two lines (task 057/058 lengthened this past a single line's
/// room at `ui::HUD_WIDTH`) rather than relying on `egui`'s label wrap, which
/// broke mid-word ("t/l temp/light · E" / "quit") instead of at a natural
/// boundary once the combined text got too long.
pub const KEYBOARD_HINT_PRIMARY: &str = "space era · n tick · r reseed · wasd pan";
pub const KEYBOARD_HINT_SECONDARY: &str = "t/l temp/light · Esc quit";

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

/// "This world wants" (task 064): the active objective, styled as a quoted
/// narrative line (`narrative_quote`) rather than a plain progress readout.
pub const HEADING_OBJECTIVE: &str = "This world wants";
pub const NO_OBJECTIVE: &str = "(no objective assigned yet)";

/// Wraps an objective's description in quotes for the narrative-styled
/// display (task 064, redesign doc §5) — the sentence itself still comes
/// from `coexistence_objective_line`/`survive_in_objective_line`/
/// `trigger_bloom_objective_line`, this only adds the quoting the italic
/// treatment implies.
pub fn narrative_quote(description: &str) -> String {
    format!("\"{description}\"")
}

pub const OBJECTIVE_CLEARED: &str = "Cleared!";
pub const BLOOM_NOT_TRIGGERED: &str = "not yet triggered";

/// "Objective i/N" (task 059): shown only when a world poses more than one
/// objective in sequence, so the player knows there's more to come after the
/// current one clears — `index` is 0-based internally, shown 1-based.
pub fn objective_sequence_position(index: usize, total: usize) -> String {
    format!("Objective {} / {total}", index + 1)
}

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

/// A curated once-per-era summary of `OrganismBorn` events (task 063) — the
/// real "individuals crossed `repro_threshold`" signal, replacing the old
/// misleading average-vs-threshold comparison on the HUD. Deliberately the
/// opposite curation choice from `observation_message` (task 061, which
/// logs every single adjacency): a zero-birth species gets no line, keeping
/// this a summary rather than a per-birth flood.
pub fn birth_log_message(species_label: &str, count: u32) -> String {
    format!("{species_label}: +{count} births this era")
}

/// A world's objective sequence (task 059) advanced past a non-final entry —
/// `index` is the newly-current objective's 0-based position, shown 1-based
/// to match `objective_sequence_position`'s HUD display. Deliberately
/// doesn't describe the new objective's own content: the HUD's objective
/// panel already shows that in full; this log line only needs to mark the
/// transition happened.
pub fn objective_advanced_message(index: usize) -> String {
    format!("Objective cleared — moving on to objective {}", index + 1)
}

/// Reported the same way `extinction_message` is (a `LogEntry` with this
/// species as its subject) — a `Splice` (task 025) previously appended a new
/// species to `world.species` with no trace anywhere in the notebook, the
/// only feedback being that it silently became selectable in the Seed
/// palette. Raised directly by a playtester.
pub fn species_created_message(species_label: &str) -> String {
    format!("{species_label} created via Splice")
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

/// A single `AdjacencyObserved` event (task 061), logged for every raw
/// observation rather than only the ones that cross the confirmation
/// threshold — the isolation principle (GDD §7) made visible by
/// reinforcement, not just prose. Short and distinct from
/// `confirmation_message` so the two don't read as duplicates next to each
/// other in the log.
pub fn observation_message(from_glyph: &str, to_glyph: &str) -> String {
    format!("{from_glyph} → {to_glyph} observed")
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
    temp_label: &str,
    repro_threshold: f32,
) -> String {
    format!(
        "{species_label}: {metabolism:?} · temp {temp_optimum:.2}±{temp_tolerance:.2} ({temp_label}) · repro ≥{repro_threshold:.1}"
    )
}

/// A short natural-language description of a species' genome (task 095),
/// alongside — not replacing — `species_catalog_line`'s precise stat line
/// above (`Splice`'s `ShiftTempOptimum` math still needs the exact numbers
/// visible somewhere). A pure function of the same readable fields
/// `species_catalog_line` already takes (`temp_label` is
/// `notebook.rs::temperature_label`'s existing cold/temperate/hot band), so
/// it stays deterministic and unit-testable without touching `SimWorld`.
pub fn species_description(
    metabolism: Metabolism,
    temp_label: &str,
    repro_threshold: f32,
) -> String {
    let diet = match metabolism {
        Metabolism::Photolithic => "draws its energy from light",
        Metabolism::Predator => "hunts adjacent organisms for energy",
        Metabolism::Decomposer => "feeds on residue left behind by the dead",
    };
    format!(
        "A {temp_label}-adapted species that {diet}, reproducing once its energy reaches {repro_threshold:.1}."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Task 095: the description must actually reflect which metabolism it
    /// was given, not a generic sentence — a player reading three different
    /// species' catalog entries should see three different diets.
    #[test]
    fn species_description_mentions_the_right_diet_per_metabolism() {
        let photolithic = species_description(Metabolism::Photolithic, "temperate", 10.0);
        let predator = species_description(Metabolism::Predator, "temperate", 10.0);
        let decomposer = species_description(Metabolism::Decomposer, "temperate", 10.0);

        assert!(photolithic.contains("light"), "got: {photolithic}");
        assert!(predator.contains("hunts"), "got: {predator}");
        assert!(decomposer.contains("residue"), "got: {decomposer}");
        assert_ne!(photolithic, predator);
        assert_ne!(predator, decomposer);
    }

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
