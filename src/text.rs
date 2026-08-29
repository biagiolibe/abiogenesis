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
use abiogenesis::sim::{DominantStimulus, RevealTier};
use abiogenesis::world::{Metabolism, Mode, SpeciesOrigin, StressAxis, TerrainKind};

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
        "Left click: perform the selected action on a cell, or inspect it if no action is \
         armed. Right click: disarm the current action. Space: advance one season, Shift+Space: \
         a full era at once. N: advance a single pulse. Arrow keys or WASD: pan the camera. \
         Tab: open your notebook. R: reseed the current world. Esc: close whatever's on top, or \
         open the pause menu if nothing is.",
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
         (residue, light, prey) if the temperature fit is poor; check the death log's stated \
         cause before suspecting a hidden matrix effect.",
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

// --- End-of-era reveal (`screens.rs::era_reveal_screen_ui`, task 140) ---

pub const ERA_REVEAL_CONTINUE_BUTTON: &str = "Continue";

/// Heading text, scaled by `sim::RevealTier` (task 140 §3: "a minor event
/// can be a discreet badge, an epochal one can take the whole screen") —
/// the wording itself carries some of that weight difference until task
/// 157 builds real generated prose.
pub fn era_reveal_title(era: u32, tier: RevealTier) -> String {
    match tier {
        RevealTier::Epochal => format!("Era {era} — a new lineage emerges"),
        RevealTier::Notable => format!("Era {era} ends"),
        RevealTier::Minor => format!("Era {era} — a quiet era"),
    }
}

/// One line per evolution the reveal applied this era — the before/after
/// comparison §3 asks for, in text form (the swatch-based visual half is
/// drawn directly in `screens.rs`, not here), plus a clause naming *why*
/// (task 142, `redesign/processed/culture-shock-friction-fixes.md`
/// Intervento 3): the player sees the cause connect back to their own prior
/// choices, not just the "what happened" the line used to stop at. Same
/// clinical register as the rest of the reveal — natural language, no raw
/// numbers or formula.
pub fn era_reveal_evolution_line(
    parent_name: &str,
    parent_tag_count: usize,
    child_name: &str,
    child_tag_count: usize,
    stimulus: DominantStimulus,
) -> String {
    format!(
        "{parent_name} ({parent_tag_count} trait{}) evolved into {child_name} ({child_tag_count} trait{}), {}",
        if parent_tag_count == 1 { "" } else { "s" },
        if child_tag_count == 1 { "" } else { "s" },
        dominant_stimulus_clause(stimulus),
    )
}

/// The natural-language cause clause for each `DominantStimulus` (task 142).
/// Deliberately generic rather than naming a specific offending tag or
/// terrain — `SelectionThresholdCrossed` accumulates pressure as scalars,
/// not "which neighbour/terrain contributed most", so a more specific
/// clause would have to invent detail the sim doesn't actually track.
fn dominant_stimulus_clause(stimulus: DominantStimulus) -> &'static str {
    match stimulus {
        DominantStimulus::InteractionHarm => {
            "worn down by sustained harm from a neighbouring species"
        }
        DominantStimulus::TerrainMismatch => {
            "pushed past its limit by the terrain it was stuck occupying"
        }
        DominantStimulus::Toxicity => "pushed past its limit by prolonged toxic exposure",
    }
}

/// Task 140's adopted answer to "what happens if the species goes extinct
/// before its evolution matures": it's simply lost. Surfaced here rather
/// than silently dropped, so the player learns why an expected reveal
/// didn't show up.
pub fn era_reveal_evolutions_lost_line(lost: u32) -> String {
    if lost == 1 {
        "One maturing evolution was lost — its species went extinct first.".to_string()
    } else {
        format!("{lost} maturing evolutions were lost — their species went extinct first.")
    }
}

pub fn era_reveal_summary_line(births: u32, deaths: u32, extinctions: u32) -> String {
    format!("This era: {births} born, {deaths} died, {extinctions} species went extinct.")
}

pub const ERA_REVEAL_QUIET_LINE: &str = "Nothing dramatic this era — the populations held steady.";

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

/// `current`/`total` are ticks elapsed / total ticks in the season
/// **currently being played** (task 117; moved from era to season by task
/// 135, since the season is now the unit the player actually advances
/// through) — not `SimWorld::tick`, the run-wide counter that never resets
/// between seasons and stops being readable at a glance after a couple of
/// them. `era` is still shown alongside it: rare and narrative now, but
/// still the number the end-of-era reveal (task 140) will refer to.
pub fn season_tick_line(era: u32, season: u32, current: u32, total: u32) -> String {
    format!("Era {era}  ·  Season {season}  ·  pulse {current}/{total}")
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

/// Task 140's indirect hint (`sim::any_evolution_maturing`): deliberately
/// vague — it never names which species or why, only that *something* is
/// building toward this era's reveal, so the reveal itself still lands as
/// the actual confirmation.
pub const MATURING_EVOLUTION_HINT: &str =
    "Something is building in a population — this era's end may bring a change";

// --- HUD — time control row (task 094) ---
//
// On-screen equivalents of the keyboard-only tick/era/notebook shortcuts —
// additive, the shortcuts keep working unchanged. `disabled` variants
// append a reason when `EraState::Advancing` already has the disabled
// button greyed out, same "why is this not clickable" affordance
// `DETAIL_MODE_ONLY_HINT` gives the action icon row.

pub const TICK_BUTTON_LABEL: &str = "⏵ Pulse";
pub const ERA_BUTTON_LABEL: &str = "⏩ Era";
pub const NOTEBOOK_BUTTON_LABEL: &str = "📓 Notebook";

pub const TICK_BUTTON_TOOLTIP: &str = "Advance one pulse (N)";
pub const ERA_BUTTON_TOOLTIP: &str = "Start/resume this era (Space)";
pub const NOTEBOOK_BUTTON_TOOLTIP: &str = "Open/close the notebook (Tab)";

pub const ADVANCING_DISABLED_HINT: &str = "\n(era already advancing)";
/// Task 152: same "why is this not clickable" affordance, for Tick/Era
/// greyed out by the *other* new advance mechanism instead.
pub const CONTINUOUS_ADVANCING_DISABLED_HINT: &str = "\n(continuous advance is running)";

pub const CONTINUOUS_ADVANCE_BUTTON_LABEL: &str = "▶ Auto";
pub const CONTINUOUS_ADVANCE_BUTTON_TOOLTIP: &str =
    "Toggle continuous pulse advancement, at the same rate era playback uses (P)";

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
        ActionMode::Stress => {
            "Shift the selected axis on the clicked cell — decays back over time."
        }
        ActionMode::Cull => "Remove the organism on the clicked cell, if any.",
        ActionMode::Splice => {
            "Edit a species' genome: swap or add a tag, or shift its thermal optimum."
        }
    }
}

/// Display name for one `StressAxis` (task 145) — the icon row's hover
/// text and the axis sub-selector's button labels.
pub fn stress_axis_name(axis: StressAxis) -> &'static str {
    match axis {
        StressAxis::Temperature => "Thermal",
        StressAxis::Light => "Light",
        StressAxis::Toxicity => "Toxicity",
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

/// Appended to Splice's tooltip by its capability-tier badge (task 152):
/// `confirmed_tags` is how many of this world's tags have a confirmed
/// matrix entry (`MatrixKnowledge::is_tag_confirmed`) — `splice_panel`
/// only ever offers *those* as swap/add candidates, so zero confirmed
/// tags means Splice can currently do nothing but shift a thermal
/// optimum, and this is the one place that real restriction gets a
/// visible marker.
pub fn splice_tier_hint(confirmed_tags: usize) -> String {
    if confirmed_tags == 0 {
        "\n(no confirmed traits yet — only the thermal shift is available)".to_string()
    } else {
        format!("\n({confirmed_tags} confirmed trait(s) available to splice in)")
    }
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

/// Population delta since the previous era (task 120,
/// `redesign/abiogenesis-hud-notebook.md` §4), shown next to the
/// energy-based trend arrow — sign-prefixed (`+4`/`-2`), `±0` for exactly
/// no change. `None` (a species' first era with any population, no prior
/// snapshot to diff against) renders as an empty string rather than an
/// implicit "+N from zero", which would misread as a population explosion.
/// Splits `[low, high]` into thirds and picks `labels[0/1/2]` (cold/dim/
/// low, temperate/moderate, hot/bright/high) — the qualitative-band idiom
/// task 149's inspection card shares with `notebook::temperature_label`
/// (task 149 lifted the shared math here so both call sites stay in sync
/// instead of drifting apart). Thresholds are always derived from the
/// caller's own config bounds, never hardcoded (no-magic-numbers rule).
pub fn band_label(value: f32, low: f32, high: f32, labels: [&'static str; 3]) -> &'static str {
    let band = (high - low) / 3.0;
    if value <= low + band {
        labels[0]
    } else if value >= high - band {
        labels[2]
    } else {
        labels[1]
    }
}

// --- Inspection tool (`ui.rs`'s hover tooltip / click-to-inspect card, task 149) ---

pub const SATURATED_NO_OUTLET_WARNING: &str = "Saturated, no room to grow";
pub const HABITABLE_LABEL: &str = "Habitable";
pub const NOT_HABITABLE_LABEL: &str = "Not habitable";

// --- Pause menu (`ui.rs::pause_menu`, task 150) ---

pub const PAUSE_MENU_TITLE: &str = "Paused";
pub const PAUSE_RESUME_BUTTON: &str = "Resume";
pub const PAUSE_SETTINGS_BUTTON: &str = "Settings";
pub const PAUSE_SETTINGS_UNAVAILABLE_HINT: &str = "Not yet available";
pub const PAUSE_SAVE_AND_EXIT_BUTTON: &str = "Save and exit";
pub const PAUSE_ABANDON_BUTTON: &str = "Abandon without saving";

// --- Shared confirm/cancel dialog (`ui.rs::confirmation_dialog`, task 150) ---

pub const CONFIRM_BUTTON: &str = "Confirm";
pub const CANCEL_BUTTON: &str = "Cancel";
pub const CONFIRM_RESEED_TITLE: &str = "Reseed this world?";
pub const CONFIRM_RESEED_BODY: &str =
    "This world has already been touched by your actions. Reseeding discards its current state.";
pub const CONFIRM_ABANDON_TITLE: &str = "Abandon this run?";
pub const CONFIRM_ABANDON_BODY: &str =
    "There is no save system yet — leaving now discards this run's progress.";

pub fn population_delta_label(delta: Option<i64>) -> String {
    match delta {
        None => String::new(),
        Some(0) => "±0".to_string(),
        Some(d) if d > 0 => format!("+{d}"),
        Some(d) => format!("{d}"),
    }
}

// --- HUD — seed palette group ---

/// "Species" (task 064): the horizontally scrollable species-selection strip.
pub const HEADING_SEED_PALETTE: &str = "Species";
pub const SEED_PALETTE_HOVER: &str = "Click an empty cell to place the selected species";
/// Split into two lines (task 057/058 lengthened this past a single line's
/// room at `ui::HUD_WIDTH`) rather than relying on `egui`'s label wrap, which
/// broke mid-word ("t/l temp/light · E" / "quit") instead of at a natural
/// boundary once the combined text got too long.
pub const KEYBOARD_HINT_PRIMARY: &str = "space season · shift+space era · n pulse · p auto";
pub const KEYBOARD_HINT_SECONDARY: &str = "r reseed · wasd pan · t/l temp/light · Esc menu";

/// Short metabolism abbreviation for `species_row_subtext` (task 152) —
/// distinct from `metabolism_glyph` (an icon) and `species_catalog_line`'s
/// `{metabolism:?}` (the full Debug name, too wide for the seed-palette
/// row); `Predator`/`Decomposer` are already short enough to keep as-is.
pub fn metabolism_short_label(metabolism: Metabolism) -> &'static str {
    match metabolism {
        Metabolism::Photolithic => "photo",
        Metabolism::Chemolithotroph => "chemo",
        Metabolism::Predator => "predator",
        Metabolism::Decomposer => "decomposer",
    }
}

/// The seed-palette row's abbreviated preview (task 152, e.g. `photo ·
/// cold`) — `temp_label` is the same qualitative band
/// `notebook::temperature_label` computes for the Catalog, passed in rather
/// than re-derived so both call sites share one threshold implementation.
pub fn species_row_subtext(metabolism: Metabolism, temp_label: &str) -> String {
    format!("{} · {temp_label}", metabolism_short_label(metabolism))
}

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
    "You isolated this species — watch its energy over the next few pulses for a clean first reading";
pub const HINT_CLUSTERED_FIRST_PLACEMENT: &str =
    "Tip: an isolated species gives cleaner readings — try it in a future era";

// --- Viewport onboarding hints — apparent-stall second hint (task 143) ---
//
// `redesign/processed/culture-shock-friction-fixes.md` Intervento 1: a
// population sitting near energy break-even shows no visible change for
// many ticks, and a new player can't tell that apart from "I did something
// wrong."

pub const HINT_APPARENT_STALL: &str =
    "No change isn't a mistake — it's worth watching what happens when two species touch";

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
/// `Objective::Speciation`'s state label (task 109) — a one-shot event like
/// `TriggerBloom`'s, so the same "not yet" phrasing applies.
pub const SPECIATION_NOT_TRIGGERED: &str = "no speciation event yet";

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

/// `Objective::Speciation`'s narrative line (task 109): the long-term
/// objective tier, always the sequence's final entry.
pub fn speciation_objective_line() -> String {
    "A species evolves through natural selection".to_string()
}

/// Within-run energy readout (task 109), shown alongside the seed line —
/// `RunProgress::energy`, whole units (the reward is always a round number
/// today, but this formats defensively in case a future balance pass makes
/// it fractional).
pub fn energy_line(energy: f32) -> String {
    format!("Energy: {energy:.0}")
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
/// misleading average-vs-threshold comparison on the HUD: a zero-birth
/// species gets no line, keeping this a summary rather than a per-birth
/// flood.
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

/// Task 107's simulation-driven counterpart to `species_created_message` —
/// distinct wording so a player can tell a `Splice` they made apart from a
/// descendant the simulation produced on its own from sustained selection
/// pressure.
pub fn species_evolved_message(species_label: &str) -> String {
    format!("{species_label} evolved from sustained selection pressure")
}

/// `env_fit` below this counts as a "poor" fit rather than a "decent" one
/// (task 104) — `env_fit` is a Gaussian in `(0, 1]`, `1.0` at the species'
/// exact temperature optimum, so the midpoint is a reasonable first-pass
/// cutoff between "temperature actively hurt this organism" and "roughly
/// fine, something else (an absent resource) is the real problem." Tune if
/// playtesting finds it misclassifies.
const POOR_ENV_FIT_THRESHOLD: f32 = 0.5;

/// Which energy-update term (GDD §5.6 step 5) dominated a death, for
/// `player_organism_death_message`'s qualitative phrasing (task 104) and,
/// reused rather than reimplemented, task 105's per-era Biosphere cause
/// label. `TemperatureOrResource` covers `gain` shortfall — a single
/// upstream signal `env_fit` then splits into the two player-facing causes
/// below. `PartialEq`/`Eq`/`Copy` (task 105) so `ui.rs::DeathCauseTally` can
/// tally and compare causes without cloning strings around.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DominantDeathCause {
    Temperature,
    ResourceAbsence,
    Predation,
    Crowding,
    /// The hidden tag×tag matrix (GDD §11), phrased to the player as a
    /// vague "interaction" — the game's own diegetic term for it (GDD's
    /// "hidden biochemical interaction rules", `abiogenesis-gdd.md:23`) —
    /// deliberately the one cause that stays vague: no number, sign, or tag
    /// identity, unlike every other branch.
    Interaction,
}

/// Picks the single dominant negative contributor to a death (task 104),
/// GDD §7's "metabolisms and environmental ranges always remain readable as
/// anchors — only the hidden matrix stays a mystery" applied to the death
/// message. First-pass dominance rule (tune if playtesting finds it
/// unconvincing): compare four magnitudes —
/// - a gain shortfall, `(upkeep - gain).max(0.0)` (how much of the baseline
///   upkeep cost this organism's metabolism failed to cover, folding
///   `Predator`/`Decomposer`'s `env_fit`-multiplied gain and
///   `Photolithic`'s directly into one comparable number);
/// - `predation_loss`;
/// - `crowding_penalty`;
/// - the matrix's harmful share, `(-interaction_delta).max(0.0)` (a
///   *positive* `interaction_delta` helped, so it's never a cause of
///   death);
///
/// the largest wins; ties are broken by checking in this fixed order
/// (temperature/resource, predation, crowding, interaction) and only replacing
/// the current leader on a strictly larger magnitude. The gain-shortfall
/// branch further splits into `Temperature` vs. `ResourceAbsence` purely
/// from `env_fit` (`POOR_ENV_FIT_THRESHOLD`), independent of its own
/// magnitude in the comparison above.
pub fn dominant_death_cause(
    gain: f32,
    env_fit: f32,
    interaction_delta: f32,
    upkeep: f32,
    crowding_penalty: f32,
    predation_loss: f32,
) -> DominantDeathCause {
    let gain_shortfall = (upkeep - gain).max(0.0);
    let interaction_harm = (-interaction_delta).max(0.0);

    let mut magnitude = gain_shortfall;
    let mut cause = if env_fit < POOR_ENV_FIT_THRESHOLD {
        DominantDeathCause::Temperature
    } else {
        DominantDeathCause::ResourceAbsence
    };
    if predation_loss > magnitude {
        magnitude = predation_loss;
        cause = DominantDeathCause::Predation;
    }
    if crowding_penalty > magnitude {
        magnitude = crowding_penalty;
        cause = DominantDeathCause::Crowding;
    }
    if interaction_harm > magnitude {
        cause = DominantDeathCause::Interaction;
    }
    cause
}

/// A metabolism's plain-language "what wasn't there" phrase for
/// `DominantDeathCause::ResourceAbsence` (task 104, GDD §7's readable-anchor
/// rule) — the resource each metabolism actually draws gain from (GDD §5.3).
fn resource_absence_phrase(metabolism: Metabolism) -> &'static str {
    match metabolism {
        Metabolism::Photolithic => "there was no light here",
        Metabolism::Predator => "there was no prey nearby",
        Metabolism::Decomposer => "there was no residue to feed on",
        Metabolism::Chemolithotroph => "there was no toxicity here to draw on",
    }
}

/// The plain-language phrase for one `DominantDeathCause` (task 104) — split
/// out from `player_organism_death_message` so the interaction branch's "no
/// numeric value, no tag identity, no sign" rule is directly unit-testable
/// on its own, without the surrounding sentence's `(x, y)` position (which
/// legitimately does contain digits and always has, pre-104).
fn death_cause_phrase(cause: DominantDeathCause, metabolism: Metabolism) -> &'static str {
    match cause {
        DominantDeathCause::Temperature => "the temperature here didn't suit it",
        DominantDeathCause::ResourceAbsence => resource_absence_phrase(metabolism),
        DominantDeathCause::Predation => "eaten by a predator",
        DominantDeathCause::Crowding => "too crowded here",
        DominantDeathCause::Interaction => "harmed by an interaction with a nearby species",
    }
}

/// A short (one-to-two word) label for one `DominantDeathCause` (task 105) —
/// the Biosphere panel's per-species cause tag next to the trend glyph
/// (`▼ predation`, `▲ crowded`), too tight for `death_cause_phrase`'s full
/// sentence fragments. Same qualitative taxonomy, same vagueness rule for
/// `Interaction` (no number/sign/tag identity), just compressed.
pub fn death_cause_short_label(cause: DominantDeathCause, metabolism: Metabolism) -> &'static str {
    match cause {
        DominantDeathCause::Temperature => "temperature",
        DominantDeathCause::ResourceAbsence => match metabolism {
            Metabolism::Photolithic => "no light",
            Metabolism::Predator => "no prey",
            Metabolism::Decomposer => "no residue",
            Metabolism::Chemolithotroph => "no toxicity",
        },
        DominantDeathCause::Predation => "predation",
        DominantDeathCause::Crowding => "crowded",
        DominantDeathCause::Interaction => "interaction",
    }
}

/// One plain-language sentence naming the single dominant cause of a death
/// (task 104), replacing the previous five-raw-number breakdown — GDD §7's
/// "metabolisms and environmental ranges always remain readable as anchors"
/// applies directly here, so every cause except the hidden matrix is spelled
/// out in direct language with no numbers at all. The matrix cause is the
/// one deliberate exception (GDD §11): no number, sign, or tag identity,
/// phrased as a vague "interaction" (the game's own diegetic term for it) —
/// see `dominant_death_cause` for the classification rule.
#[allow(clippy::too_many_arguments)]
pub fn player_organism_death_message(
    species_label: &str,
    x: usize,
    y: usize,
    metabolism: Metabolism,
    gain: f32,
    env_fit: f32,
    interaction_delta: f32,
    upkeep: f32,
    crowding_penalty: f32,
    predation_loss: f32,
) -> String {
    let cause = dominant_death_cause(
        gain,
        env_fit,
        interaction_delta,
        upkeep,
        crowding_penalty,
        predation_loss,
    );
    let cause = death_cause_phrase(cause, metabolism);
    format!("your {species_label} organism at ({x}, {y}) died: {cause}")
}

/// A matrix cell crossing the confirmation threshold (GDD §7's "aha"
/// moment, task 054) — distinguished from ordinary log lines by a leading
/// glyph the caller renders separately (`notebook.rs`'s `CONFIRMATION_GLYPH`),
/// same split `LogEntry`/`species_color` uses for species-subject lines.
pub fn confirmation_message(from_glyph: &str, to_glyph: &str, positive: bool) -> String {
    let sign = if positive { "boosts" } else { "harms" };
    format!("Confirmed: tag {from_glyph} {sign} tag {to_glyph}")
}

/// A `TerrainKind`'s player-facing name (task 099), for the zone-entry
/// reveal message below — no other player-facing terrain label exists yet
/// in this codebase (terrain otherwise only shows up as color, task 066).
pub fn terrain_label(kind: TerrainKind) -> &'static str {
    match kind {
        TerrainKind::Sea => "sea",
        TerrainKind::Plain => "plains",
        TerrainKind::Hill => "hills",
        TerrainKind::Mountain => "mountains",
    }
}

/// A species' lineage set foot on a terrain that conditions one of its tags,
/// for the first time this run (task 099, GDD-adjacent "reveal-on-first-
/// zone-entry" — the terrain-conditional counterpart to `confirmation_message`'s
/// matrix-cell "aha" moment). Deliberately its own function rather than
/// reusing `confirmation_message`: that one is phrased around a matrix
/// tag-pair boost/harm, which doesn't fit "this tag only matters on this
/// terrain."
pub fn terrain_reveal_message(
    species_label: &str,
    tag_glyph: &str,
    terrain: TerrainKind,
) -> String {
    let terrain = terrain_label(terrain);
    format!("Revealed: {species_label}'s tag {tag_glyph} is conditioned by {terrain}")
}

/// A `(TagSlot, TerrainKind)` pair's evidence crossing task 097's
/// confirmation threshold — the gradual, exposure-based counterpart to
/// `terrain_reveal_message`'s deterministic zone-entry beat (task 099).
/// Not about any one species (the fact is world-level, like
/// `confirmation_message`'s tag pairs), so it's phrased the same
/// "Confirmed: ..." way rather than reusing `terrain_reveal_message`'s
/// species-subject wording.
pub fn terrain_gate_confirmed_message(tag_glyph: &str, terrain: TerrainKind, mode: Mode) -> String {
    let terrain = terrain_label(terrain);
    let effect = match mode {
        Mode::Inducible => "turns on",
        Mode::Repressible => "turns off",
    };
    format!("Confirmed: tag {tag_glyph} {effect} on {terrain}")
}

// --- Notebook — hypothesis graph (`notebook.rs::hypothesis_grid`) ---

pub const HEADING_HYPOTHESIS_GRID: &str = "Hypothesis grid";

pub fn node_tag_line(glyph: &str) -> String {
    format!("Tag {glyph}")
}

/// `magnitude` is the confirmed matrix entry's absolute value (task 102:
/// once the on-grid edge label is gone, this tooltip line is the only place
/// the number is still readable, per the "graph is never the only source"
/// constraint task 031 established).
pub fn confirmed_relation_line(
    from_glyph: &str,
    to_glyph: &str,
    positive: bool,
    magnitude: i8,
) -> String {
    let sign = if positive { "+" } else { "-" };
    format!("{from_glyph} → {to_glyph} ({sign}{magnitude})")
}

/// `confidence_pct` is `evidence / threshold` as a rounded percentage (task
/// 102) — how close this pair is to crossing the confirmation threshold.
pub fn partial_relation_line(from_glyph: &str, to_glyph: &str, confidence_pct: u32) -> String {
    format!("{from_glyph} → {to_glyph} (some evidence, ~{confidence_pct}%)")
}

// --- Notebook — catalog (`notebook.rs::catalog_panel`) ---

pub const HEADING_CATALOG: &str = "Catalog";
pub const ACTIVE_TAGS_LABEL: &str = "Active tags";
pub const SPECIES_HEADING: &str = "Species";
pub const METABOLISM_LEGEND_HEADING: &str = "Metabolisms";

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

/// Population and seeded era for one species' catalog row (task 103's
/// extension, corrected by a 103 follow-up) — separate from
/// `species_catalog_line` since those two fields come from different
/// sources (`world.cells` scan vs. `SimWorld::species_seeded_era`) and are
/// easier to reason about as their own small line. `origin_era` is `None`
/// for a species still sitting in the available roster with nothing ever
/// placed (`SimWorld::species_seeded_era` stays `None` until then) —
/// showing a "seeded era" for a species that was never actually seeded
/// would be misleading, not merely incomplete.
pub fn species_population_line(population: usize, origin_era: Option<u32>) -> String {
    match origin_era {
        Some(era) => format!("Population {population} · seeded era {era}"),
        None => format!("Population {population}"),
    }
}

/// The Catalog's origin label per species (task 147): seeded (this run's
/// ordinary roster), indigenous (a wild, pre-existing population), or
/// synthesised (created via `Splice`).
pub fn species_origin_label(origin: SpeciesOrigin) -> &'static str {
    match origin {
        SpeciesOrigin::Seeded => "Seeded",
        SpeciesOrigin::Indigenous => "Indigenous",
        SpeciesOrigin::Synthesised => "Synthesised",
    }
}

/// The Catalog's "descends from" line (task 153) — shown only when
/// `SimWorld::parent_of` returns `Some`, omitted otherwise.
pub fn descends_from_line(parent_name: &str) -> String {
    format!("descends from: {parent_name}")
}

// --- Notebook — Chronicle (`notebook.rs`, task 153) ---

pub const HEADING_CHRONICLE: &str = "Chronicle";
pub const NO_CHRONICLE_YET: &str =
    "Nothing archived yet — dismiss an era's reveal to add to the Chronicle.";

/// One compressed row for a run of consecutive quiet (`RevealTier::Minor`)
/// eras (task 153) — a single line regardless of how many eras it spans.
pub fn chronicle_quiet_line(era_start: u32, era_end: u32) -> String {
    if era_start == era_end {
        format!("Era {era_start}: quiet")
    } else {
        format!("Eras {era_start}-{era_end}: quiet")
    }
}

/// One line per metabolism kind, shown once in the catalog's legend section
/// rather than repeated on every species row sharing that metabolism (task
/// 103 — a live screenshot review found the old per-row
/// `species_description` printing the same diet sentence for every species
/// of a given metabolism, differing only in the temp-band word and
/// threshold number, which is text noise once read once). The diet clause
/// itself is unchanged from `species_description`'s wording, just no longer
/// tied to a specific species' temp band or threshold.
pub fn metabolism_legend_line(metabolism: Metabolism) -> String {
    let diet = match metabolism {
        Metabolism::Photolithic => "draws its energy from light",
        Metabolism::Predator => "hunts adjacent organisms for energy",
        Metabolism::Decomposer => "feeds on residue left behind by the dead",
        Metabolism::Chemolithotroph => "draws its energy from environmental toxicity",
    };
    format!("{diet}, reproducing once its energy reaches its threshold.")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Task 095, updated by 103's legend consolidation: the line must
    /// actually reflect which metabolism it was given, not a generic
    /// sentence — a player reading the legend should see three different
    /// diets, one per metabolism.
    #[test]
    fn metabolism_legend_line_mentions_the_right_diet_per_metabolism() {
        let photolithic = metabolism_legend_line(Metabolism::Photolithic);
        let predator = metabolism_legend_line(Metabolism::Predator);
        let decomposer = metabolism_legend_line(Metabolism::Decomposer);

        assert!(photolithic.contains("light"), "got: {photolithic}");
        assert!(predator.contains("hunts"), "got: {predator}");
        assert!(decomposer.contains("residue"), "got: {decomposer}");
        assert_ne!(photolithic, predator);
        assert_ne!(predator, decomposer);
    }

    /// Task 142: the reveal line must name a different cause per
    /// `DominantStimulus`, so the player can connect the event to *why* it
    /// happened, not just see identical text with a shuffled species name.
    #[test]
    fn era_reveal_evolution_line_names_a_different_cause_per_dominant_stimulus() {
        let line = |stimulus| era_reveal_evolution_line("Alpha", 1, "Beta", 2, stimulus);
        let interaction = line(DominantStimulus::InteractionHarm);
        let terrain = line(DominantStimulus::TerrainMismatch);
        let toxicity = line(DominantStimulus::Toxicity);

        assert_ne!(interaction, terrain);
        assert_ne!(terrain, toxicity);
        assert_ne!(interaction, toxicity);
        for line in [&interaction, &terrain, &toxicity] {
            assert!(
                line.starts_with("Alpha (1 trait) evolved into Beta (2 traits)"),
                "got: {line}"
            );
        }
    }

    /// Task 120: sign-prefixed, `±0` for exactly no change, empty for a
    /// species with no prior-era snapshot yet.
    #[test]
    fn population_delta_label_formats_all_cases() {
        assert_eq!(population_delta_label(Some(4)), "+4");
        assert_eq!(population_delta_label(Some(-2)), "-2");
        assert_eq!(population_delta_label(Some(0)), "±0");
        assert_eq!(population_delta_label(None), "");
    }

    /// Task 103 follow-up: a species never placed on the grid
    /// (`SimWorld::species_ever_placed` false) must not show a "seeded
    /// era" — a species from the available roster that the player hasn't
    /// used yet isn't "seeded" in any meaningful sense.
    #[test]
    fn species_population_line_omits_seeded_era_when_never_placed() {
        let line = species_population_line(0, None);
        assert_eq!(line, "Population 0");
        assert!(!line.contains("seeded"), "got: {line}");
    }

    #[test]
    fn species_population_line_shows_seeded_era_once_placed() {
        let line = species_population_line(4, Some(2));
        assert_eq!(line, "Population 4 · seeded era 2");
    }

    #[test]
    fn death_message_names_temperature_when_gain_is_short_and_fit_is_poor() {
        // gain 0.0 vs. upkeep 0.7: a 0.7 shortfall, dominant over the
        // zeroed-out predation/crowding/matrix terms. env_fit 0.1 is below
        // `POOR_ENV_FIT_THRESHOLD`, so this reads as temperature, not
        // resource absence.
        let message = player_organism_death_message(
            "Nyx (species 0)",
            3,
            4,
            Metabolism::Photolithic,
            0.0,
            0.1,
            0.0,
            0.7,
            0.0,
            0.0,
        );
        assert!(message.contains("temperature"), "got: {message}");
    }

    #[test]
    fn death_message_names_the_missing_resource_per_metabolism_when_fit_is_decent() {
        // Same gain shortfall as above, but env_fit 0.9 is well above the
        // threshold — a fine temperature fit, so the gain shortfall reads as
        // an absent resource instead, phrased per metabolism.
        let photolithic = player_organism_death_message(
            "A",
            0,
            0,
            Metabolism::Photolithic,
            0.0,
            0.9,
            0.0,
            0.7,
            0.0,
            0.0,
        );
        let predator = player_organism_death_message(
            "B",
            0,
            0,
            Metabolism::Predator,
            0.0,
            0.9,
            0.0,
            0.7,
            0.0,
            0.0,
        );
        let decomposer = player_organism_death_message(
            "C",
            0,
            0,
            Metabolism::Decomposer,
            0.0,
            0.9,
            0.0,
            0.7,
            0.0,
            0.0,
        );
        assert!(photolithic.contains("light"), "got: {photolithic}");
        assert!(predator.contains("prey"), "got: {predator}");
        assert!(decomposer.contains("residue"), "got: {decomposer}");
    }

    #[test]
    fn death_message_names_predation_when_it_dominates() {
        // gain covers upkeep exactly (no shortfall), crowding/matrix are
        // zero, only predation_loss pulls energy down.
        let message = player_organism_death_message(
            "Nyx",
            0,
            0,
            Metabolism::Predator,
            0.5,
            0.9,
            0.0,
            0.5,
            0.0,
            2.0,
        );
        assert!(message.contains("predator"), "got: {message}");
    }

    #[test]
    fn death_message_names_crowding_when_it_dominates() {
        let message = player_organism_death_message(
            "Nyx",
            0,
            0,
            Metabolism::Photolithic,
            0.5,
            0.9,
            0.0,
            0.5,
            2.0,
            0.0,
        );
        assert!(message.contains("crowded"), "got: {message}");
    }

    #[test]
    fn death_message_names_the_interaction_when_it_dominates() {
        let message = player_organism_death_message(
            "Nyx",
            0,
            0,
            Metabolism::Photolithic,
            0.5,
            0.9,
            -2.0,
            0.5,
            0.0,
            0.0,
        );
        assert!(
            message.contains("interaction") && message.contains("nearby species"),
            "the hidden-matrix cause should stay vague, phrased as an interaction, no tag/sign: {message}"
        );
    }

    #[test]
    fn interaction_dominant_cause_phrase_contains_no_digit_characters() {
        // Task 104: the one deliberate mystery-preservation boundary — no
        // numeric value, no tag identity, no sign for the hidden-matrix
        // cause. Checked on the cause phrase itself (not the full sentence,
        // whose `(x, y)` position has always legitimately contained digits).
        let phrase = death_cause_phrase(DominantDeathCause::Interaction, Metabolism::Photolithic);
        assert!(
            !phrase.chars().any(|c| c.is_ascii_digit()),
            "interaction cause phrase must contain no numbers at all: {phrase}"
        );
    }
}
