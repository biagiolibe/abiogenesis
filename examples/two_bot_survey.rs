//! Task 134 — does experimenting actually pay off?
//!
//! Runs two automatic strategies against the same worlds and reports which
//! one gets further, faster:
//!
//! - **Exploiter** — acts only on matrix relations it has already confirmed.
//!   Places where the known evidence predicts a good outcome, and falls back
//!   to a safe isolated spot when it knows nothing useful. It never spends a
//!   point to *learn* something.
//! - **Explorer** — deliberately probes unknown tag pairs, preferring the
//!   placement that would produce the cleanest possible observation (one
//!   unconfirmed pair, no confounders), and only then falls back to
//!   exploiting.
//!
//! **Failure criterion** (`redesign/processed/culture-shock-experiment-incentive.md`):
//! if the Exploiter wins *systematically*, the incentives are wrong — the game
//! is rewarding the player who doesn't play the game it was designed around.
//! The Explorer does not have to win. Two close strategies, with the Explorer
//! ahead where information matters most, is the healthy result.
//!
//! This is a **measurement tool, not a regression guard**: it lives in
//! `examples/` so a multi-minute sweep never runs as part of `cargo test`.
//! It changes no balance and asserts nothing.
//!
//! ```text
//! cargo run --release --example two_bot_survey -- [seed_count]
//! ```
//!
//! Task 136 lowers the metabolic gains so the hidden matrix, not the
//! environment, decides growth. Re-run this afterwards and compare against the
//! baseline recorded in `tasks/134-two-bot-experiment-incentive-harness.md`.

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use abiogenesis::actions::attempt_seed;
use abiogenesis::config::SimConfig;
use abiogenesis::knowledge::{accumulate_adjacency_evidence, MatrixKnowledge};
use abiogenesis::objectives::{
    evaluate_world, is_grace_active, update_grace_progress, FailureReason, GraceProgress,
    Objective, ObjectiveProgress, WorldOutcome,
};
use abiogenesis::sim::{speciate, step, ActionBudget};
use abiogenesis::world::{SimWorld, SpeciesId, TagSlot};
use abiogenesis::worldgen::{build_world, season_pulses_for, world_params};

/// Every run is world 0 of a fresh run: the same world the balance work in
/// task 136 is tuned against, and the one a player actually meets first.
const WORLD_INDEX: u32 = 0;
/// Default sweep size. Task 074's own survey used 50 seeds and task 131's
/// relational tests 20-30; this sits in the same range, and the distributions
/// below are reported in full so it's visible whether the sample is enough.
const DEFAULT_SEED_COUNT: u64 = 40;
/// Placeable cells considered per placement decision. The grid is 10240 cells
/// and a full scan per action, per species, per era, across two policies and
/// dozens of seeds is minutes of pointless work — a random sample of this size
/// is plenty to find a good cell and models a player who looks around a bit
/// rather than solving the whole board.
const CANDIDATE_SAMPLE: usize = 256;
/// Below this thermal fitness a placement is a waste of a point under any
/// strategy — `gain = light * metabolism_gain * env_fit` can't clear upkeep.
/// Both policies filter on it, so neither wins by simply placing better.
const MIN_VIABLE_FIT: f32 = 0.6;
/// Ceiling on how much the information dimension can move a placement score,
/// relative to `viability`'s own 0..1 range: a tie-breaker between
/// comparably-good cells, never enough to make a mediocre cell in a
/// favoured context beat a genuinely better one in a disfavoured context.
/// That inversion — bucket beats fitness — is what kept the pre-134b bots
/// from ever chasing the population scale a real player (or the greedy
/// diagnostic) reaches.
const INFO_WEIGHT: f32 = 0.15;
/// Scales `known_sum` (a raw sum of matrix values, `TagConfig`'s range) down
/// into the same tie-breaker band as `INFO_WEIGHT`.
const KNOWN_SUM_SCALE: f32 = 0.05;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Policy {
    Exploiter,
    Explorer,
}

impl Policy {
    fn label(self) -> &'static str {
        match self {
            Policy::Exploiter => "exploiter",
            Policy::Explorer => "explorer",
        }
    }
}

/// How a placement's information context is classified, decided *before* the
/// placement from what the bot can actually see.
///
/// The pair set of a placement is every `(neighbour tag, seeded species tag)`
/// combination that `sim::step` will evaluate at that cell — the tags of
/// occupied Moore neighbours crossed with the seeded species' own tags.
///
/// The `Isolated` bucket is not a hedge: a placement with no occupied
/// neighbours has an *empty* pair set, so it is neither known nor unknown, yet
/// it is the cleanest observation the game offers (GDD §7's weight-1.0 case)
/// once anything grows next to it. Folding it into either of the other two
/// would misreport both.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Context {
    /// No occupied Moore neighbour: empty pair set.
    Isolated,
    /// Every pair in the set is already confirmed in `MatrixKnowledge`.
    Known,
    /// At least one pair in the set is still unconfirmed.
    Unknown,
}

#[derive(Default, Clone, Copy)]
struct Ledger {
    isolated: u32,
    known: u32,
    unknown: u32,
}

impl Ledger {
    fn record(&mut self, context: Context, points: u32) {
        match context {
            Context::Isolated => self.isolated += points,
            Context::Known => self.known += points,
            Context::Unknown => self.unknown += points,
        }
    }

    fn total(&self) -> u32 {
        self.isolated + self.known + self.unknown
    }
}

struct RunResult {
    /// Seasons taken to clear every short-term objective (task 135: the
    /// decision unit, so this stays at season granularity rather than
    /// coarsening to the now-rare era) — i.e. to reach the final
    /// `Speciation` entry `worldgen::generate_objectives` always appends.
    /// `None` if the world ended first.
    ///
    /// Reported separately from `full_seasons` on purpose: `Speciation`
    /// fires off accumulated selection pressure, which is only weakly under
    /// a player's control, so the full-sequence time can be dominated by how
    /// long until *any* lineage crosses the threshold — swamping the
    /// strategy difference this survey exists to measure. If the signal is
    /// anywhere, it is here.
    short_term_seasons: Option<u32>,
    /// Seasons taken to clear the whole sequence, `Speciation` included.
    /// `None` if the world ended first. This is what a real run feels like.
    full_seasons: Option<u32>,
    outcome: WorldOutcome,
    objectives_cleared: u32,
    ledger: Ledger,
    confirmed_pairs: u32,
    confirmable_pairs: u32,
    /// Highest cell occupancy seen at any season boundary. Sampled per
    /// season, not only at world end, because an extinct or budget-exhausted
    /// world ends at (near) zero — the end-state tells nothing about whether
    /// the run ever reached a working population.
    peak_population: u32,
}

fn main() {
    let seed_count = std::env::args()
        .nth(1)
        .and_then(|a| a.parse::<u64>().ok())
        .unwrap_or(DEFAULT_SEED_COUNT);
    let config = SimConfig::default();

    println!(
        "two-bot survey — world {WORLD_INDEX}, seeds 0..{seed_count}, \
         era budget {}, season pulses {}, seasons per era {}",
        world_params(WORLD_INDEX, &config).era_budget,
        config.time.season_pulses,
        config.time.seasons_per_era
    );
    println!();

    let mut exploiter = Vec::new();
    let mut explorer = Vec::new();
    for seed in 0..seed_count {
        exploiter.push(play(seed, Policy::Exploiter, &config));
        explorer.push(play(seed, Policy::Explorer, &config));
    }

    report(Policy::Exploiter, &exploiter);
    report(Policy::Explorer, &explorer);
    verdict(&exploiter, &explorer);
}

/// One world, one strategy. Replicates the minimal slice of the Bevy schedule
/// that matters here — tick, accumulate evidence, speciate on a crossed
/// threshold, evaluate the objective, close the season — rather than reusing
/// the systems, which need an `App` and a window.
fn play(seed: u64, policy: Policy, config: &SimConfig) -> RunResult {
    let (mut world, objectives) = build_world(seed, WORLD_INDEX, config, 0);
    let params = world_params(WORLD_INDEX, config);
    let mut knowledge = MatrixKnowledge::new(
        world.active_tags.len(),
        config.notebook.confirmation_threshold,
    );
    let mut budget = ActionBudget {
        points_remaining: config.time.point_budget_per_season,
    };
    let mut progress = ObjectiveProgress::default();
    let mut grace = GraceProgress::default();
    let mut objective_index = 0usize;
    let mut outcome = WorldOutcome::Ongoing;
    let mut ledger = Ledger::default();
    let mut short_term_seasons = None;
    let mut full_seasons = None;
    let mut peak_population = 0u32;
    // The bot's own RNG, never `world.rng`: a player's choices are not drawn
    // from the simulation's stream, and borrowing it would make the two arms
    // diverge for a reason that has nothing to do with their strategies.
    // Seeded identically for both policies, so the decision rule is the only
    // difference between the two runs of a given seed.
    let mut rng = StdRng::seed_from_u64(seed);

    let placeable = placeable_cells(&world);
    let seedable = seedable_species(&world);

    'world: while outcome == WorldOutcome::Ongoing {
        // Observing window: spend the season's points.
        while budget.points_remaining >= config.time.action_costs.seed {
            let Some((species, x, y, context)) = choose_placement(
                policy, &world, config, &knowledge, &placeable, &seedable, &mut rng,
            ) else {
                break;
            };
            let cost = config.time.action_costs.seed;
            if attempt_seed(&mut world, config, &mut budget, species, x, y).is_none() {
                break;
            }
            ledger.record(context, cost);
        }

        let ticks = season_pulses_for(WORLD_INDEX, world.season, config);
        for _ in 0..ticks {
            let events = step(&mut world, config);
            accumulate_adjacency_evidence(events.adjacencies, config, &mut knowledge);
            for crossed in &events.selection_thresholds {
                speciate(&mut world, config, crossed);
            }

            update_grace_progress(&world, &mut grace, config.time.season_pulses);
            let grace_active = is_grace_active(world.season, config.time.grace_seasons, &grace);
            let tick_outcome = evaluate_world(
                objectives.get(objective_index),
                &world,
                &mut progress,
                params.era_budget,
                grace_active,
            );
            match tick_outcome {
                WorldOutcome::Ongoing => {}
                WorldOutcome::Failed(_) => {
                    outcome = tick_outcome;
                    break 'world;
                }
                WorldOutcome::Cleared => {
                    // The final entry is always `Objective::Speciation`
                    // (`worldgen::generate_objectives`), so crossing into it
                    // is exactly "every short-term objective is done".
                    if objectives.get(objective_index + 1) == Some(&Objective::Speciation) {
                        short_term_seasons.get_or_insert(world.season);
                    }
                    if objective_index + 1 >= objectives.len() {
                        full_seasons = Some(world.season);
                        outcome = WorldOutcome::Cleared;
                        objective_index += 1;
                        break 'world;
                    }
                    objective_index += 1;
                    progress = ObjectiveProgress::default();
                }
            }
        }

        let population = world
            .cells
            .iter()
            .filter(|cell| cell.population.is_some())
            .count() as u32;
        peak_population = peak_population.max(population);

        world.season += 1;
        if world.season.is_multiple_of(config.time.seasons_per_era) {
            world.era += 1;
        }
        budget.refill(config.time.point_budget_per_season);
    }

    RunResult {
        short_term_seasons,
        full_seasons,
        outcome,
        objectives_cleared: objective_index as u32,
        ledger,
        confirmed_pairs: confirmed_pairs(&knowledge, &world),
        confirmable_pairs: confirmable_pairs(&world),
        peak_population,
    }
}

/// Picks a placement under `policy`, or `None` if nothing worth placing was
/// found in the sampled candidates.
///
/// Both policies share the same viability filter, so neither can win merely by
/// placing organisms somewhere they survive better — the only difference
/// between them is what they do with the information dimension.
fn choose_placement(
    policy: Policy,
    world: &SimWorld,
    config: &SimConfig,
    knowledge: &MatrixKnowledge,
    placeable: &[usize],
    seedable: &[SpeciesId],
    rng: &mut StdRng,
) -> Option<(SpeciesId, usize, usize, Context)> {
    if seedable.is_empty() || placeable.is_empty() {
        return None;
    }

    let mut best: Option<(f32, SpeciesId, usize, usize, Context)> = None;
    for _ in 0..CANDIDATE_SAMPLE {
        let index = placeable[rng.random_range(0..placeable.len())];
        if world.cells[index].population.is_some() {
            continue;
        }
        let (x, y) = (index % world.width, index / world.width);
        let species = seedable[rng.random_range(0..seedable.len())];
        if !world.is_placeable_for(x, y, species) {
            continue;
        }
        let viability = viability(world, config, species, index);
        if viability < MIN_VIABLE_FIT {
            continue;
        }

        let pairs = pair_set(world, species, x, y);
        let context = classify(&pairs, knowledge);
        // `viability` (0..1, the environmental fitness) is the growth term
        // both policies share: a real player chases population first and
        // foremost, so it must be able to outweigh the information dimension
        // below rather than being drowned out by it. `INFO_WEIGHT` and
        // `KNOWN_SUM_SCALE` keep the info terms as tie-breakers within
        // viability's own range — never a categorical override — which is
        // what let the pre-134b scoring strand both bots on whichever
        // adjacency bucket their identity favoured regardless of how good the
        // cell actually was.
        let score = viability
            + match policy {
                // Acts only on what it has established: a placement next to
                // confirmed relations, valued by their known sum, beats a
                // safe isolated one, which in turn beats gambling on unknown
                // pairs — but only among cells of comparable viability.
                Policy::Exploiter => match context {
                    Context::Known => {
                        INFO_WEIGHT + known_sum(world, knowledge, &pairs) * KNOWN_SUM_SCALE
                    }
                    Context::Isolated => INFO_WEIGHT * 0.5,
                    Context::Unknown => 0.0,
                },
                // Buys information first: the fewer already-confirmed pairs
                // come along for the ride, the cleaner the resulting
                // observation and the more the point is worth. Falls back to
                // exploiting once there is nothing left to learn here.
                Policy::Explorer => match context {
                    Context::Unknown => {
                        let unknown = unknown_count(&pairs, knowledge) as f32;
                        let confounders = (pairs.len() as f32 - unknown).max(0.0);
                        INFO_WEIGHT * (1.0 + 1.0 / (1.0 + confounders))
                    }
                    Context::Known => known_sum(world, knowledge, &pairs) * KNOWN_SUM_SCALE,
                    Context::Isolated => INFO_WEIGHT * 0.5,
                },
            };
        if best.is_none_or(|(b, ..)| score > b) {
            best = Some((score, species, x, y, context));
        }
    }

    best.map(|(_, species, x, y, context)| (species, x, y, context))
}

/// Every `(exerter tag, receiver tag)` pair `sim::step` would evaluate for a
/// `species` organism placed at `(x, y)`: each occupied Moore neighbour's tags
/// against the placed species' own.
///
/// Deliberately *not* filtered through `tag_gate_satisfied` (task 096's
/// terrain-conditional gates): the bot models a player, and a player cannot
/// know which gates are active before observing them.
fn pair_set(world: &SimWorld, species: SpeciesId, x: usize, y: usize) -> Vec<(TagSlot, TagSlot)> {
    let own = &world.species[species.0 as usize].tags;
    let mut pairs = Vec::new();
    for neighbour in world.moore_neighbours(x, y) {
        let Some(organism) = world.cells[neighbour].population else {
            continue;
        };
        for &their_tag in &world.species[organism.species.0 as usize].tags {
            for &my_tag in own {
                if !pairs.contains(&(their_tag, my_tag)) {
                    pairs.push((their_tag, my_tag));
                }
            }
        }
    }
    pairs
}

fn classify(pairs: &[(TagSlot, TagSlot)], knowledge: &MatrixKnowledge) -> Context {
    if pairs.is_empty() {
        Context::Isolated
    } else if unknown_count(pairs, knowledge) == 0 {
        Context::Known
    } else {
        Context::Unknown
    }
}

fn unknown_count(pairs: &[(TagSlot, TagSlot)], knowledge: &MatrixKnowledge) -> usize {
    pairs
        .iter()
        .filter(|(exerter, receiver)| !knowledge.is_confirmed(*exerter, *receiver))
        .count()
}

/// Sum of the matrix values the bot is *entitled* to know — read through
/// `revealed_value`, which returns `None` for anything unconfirmed, so this
/// can never leak the hidden matrix into a decision.
fn known_sum(world: &SimWorld, knowledge: &MatrixKnowledge, pairs: &[(TagSlot, TagSlot)]) -> f32 {
    pairs
        .iter()
        .filter_map(|(exerter, receiver)| knowledge.revealed_value(*exerter, *receiver, world))
        .map(|value| value as f32)
        .sum()
}

/// How well `species` would do at `index` on the visible scalars alone —
/// thermal fitness times the resource its metabolism reads. This is the
/// environmental reading a player makes from the HUD overlays, nothing more:
/// it never consults the matrix.
fn viability(world: &SimWorld, config: &SimConfig, species: SpeciesId, index: usize) -> f32 {
    let cell = &world.cells[index];
    let species = &world.species[species.0 as usize];
    let fit = abiogenesis::sim::env_fit(
        cell.temperature,
        species.temp_optimum,
        species.temp_tolerance,
    );
    let resource = match species.metabolism {
        abiogenesis::world::Metabolism::Photolithic => cell.light,
        abiogenesis::world::Metabolism::Chemolithotroph => cell.toxicity,
        // Predators and decomposers depend on other organisms rather than on
        // a cell scalar, so thermal fitness is all the environment tells a
        // player up front. `config` stays in the signature because the
        // photolithic/chemolithotroph branches read it if this is ever
        // refined into a real expected-gain estimate.
        _ => {
            let _ = config;
            1.0
        }
    };
    fit * resource
}

/// Terrain-placeable cells, computed once per world: terrain never changes
/// during a world, so re-deriving this per action would be pure waste.
/// Occupancy is checked per candidate instead, since that does change.
fn placeable_cells(world: &SimWorld) -> Vec<usize> {
    (0..world.cells.len())
        .filter(|&index| {
            let (x, y) = (index % world.width, index / world.width);
            !world.get(x, y).is_peak
                && world
                    .species
                    .iter()
                    .enumerate()
                    .any(|(i, _)| world.is_placeable_for(x, y, SpeciesId(i as u8)))
        })
        .collect()
}

/// The seed palette a player sees: every species except this world's wild
/// populations, which already exist on the grid and were never a choice
/// (task 098, mirrored from `ui.rs`'s palette).
fn seedable_species(world: &SimWorld) -> Vec<SpeciesId> {
    (0..world.species.len() as u8)
        .map(SpeciesId)
        .filter(|&id| !world.is_wild(id))
        .collect()
}

/// Ordered off-diagonal `(exerter, receiver)` pairs confirmed so far. The
/// diagonal is excluded because `world::generate_matrix` always leaves it at
/// `0` (a tag never affects itself) — it is neither hidden nor confirmable,
/// so counting it would inflate both the numerator and (if it were ever
/// added there too) the denominator with pairs nobody needed to learn.
fn confirmed_pairs(knowledge: &MatrixKnowledge, world: &SimWorld) -> u32 {
    let tag_count = world.active_tags.len() as u8;
    let mut confirmed = 0;
    for exerter in 0..tag_count {
        for receiver in 0..tag_count {
            if exerter != receiver && knowledge.is_confirmed(TagSlot(exerter), TagSlot(receiver)) {
                confirmed += 1;
            }
        }
    }
    confirmed
}

/// Ordered off-diagonal pairs whose matrix value is nonzero — the only pairs
/// `confirmation_threshold` evidence can ever confirm. `matrix_density` (0.4
/// by default) plus the forced negative 3-cycle leave roughly half of the 20
/// off-diagonal pairs (for a 5-tag world) at exactly `0`; those are silent by
/// construction and can never cross the threshold, so reporting confirmed
/// pairs against all of them understates performance by about 2x.
fn confirmable_pairs(world: &SimWorld) -> u32 {
    let tag_count = world.active_tags.len() as u8;
    let mut confirmable = 0;
    for exerter in 0..tag_count {
        for receiver in 0..tag_count {
            if exerter != receiver && world.matrix.get(TagSlot(exerter), TagSlot(receiver)) != 0 {
                confirmable += 1;
            }
        }
    }
    confirmable
}

fn report(policy: Policy, results: &[RunResult]) {
    let n = results.len() as f32;
    let cleared = results
        .iter()
        .filter(|r| r.outcome == WorldOutcome::Cleared)
        .count();
    let extinct = results
        .iter()
        .filter(|r| r.outcome == WorldOutcome::Failed(FailureReason::TotalExtinction))
        .count();
    let exhausted = results
        .iter()
        .filter(|r| r.outcome == WorldOutcome::Failed(FailureReason::EraBudgetExhausted))
        .count();

    println!("## {}", policy.label());
    println!(
        "  outcomes            cleared {cleared}, extinct {extinct}, era budget exhausted {exhausted} (of {})",
        results.len()
    );
    print_distribution(
        "  short-term seasons ",
        results
            .iter()
            .filter_map(|r| r.short_term_seasons)
            .collect(),
        results.len(),
    );
    print_distribution(
        "  full-sequence seasons ",
        results.iter().filter_map(|r| r.full_seasons).collect(),
        results.len(),
    );
    print_distribution(
        "  peak population    ",
        results.iter().map(|r| r.peak_population).collect(),
        results.len(),
    );

    // Reported as a spend *split* plus one overall efficiency, deliberately
    // not as an efficiency per bucket. Dividing total objectives by one
    // bucket's points isn't an efficiency — it's the same numerator over an
    // arbitrary slice of the denominator, and it flatters whichever bucket a
    // policy barely uses. The comparison that means something is between the
    // two policies, which realise the two spend mixes by construction: does
    // the one that buys more information get more done per point, or less?
    let objectives: u32 = results.iter().map(|r| r.objectives_cleared).sum();
    let points: u32 = results.iter().map(|r| r.ledger.total()).sum();
    let isolated: u32 = results.iter().map(|r| r.ledger.isolated).sum();
    let known: u32 = results.iter().map(|r| r.ledger.known).sum();
    let unknown: u32 = results.iter().map(|r| r.ledger.unknown).sum();
    println!(
        "  objectives cleared  {objectives} total, {:.2} per world",
        objectives as f32 / n
    );
    println!(
        "  points spent        {points} total — isolated {isolated}, known {known}, unknown {unknown}"
    );
    if points > 0 {
        println!(
            "  objectives / point  {:.4}",
            objectives as f32 / points as f32
        );
    }
    let confirmed: u32 = results.iter().map(|r| r.confirmed_pairs).sum();
    let confirmable: u32 = results.iter().map(|r| r.confirmable_pairs).sum();
    println!(
        "  pairs confirmed     {:.2} per world ({}/{} confirmable, {:.1}%)",
        confirmed as f32 / n,
        confirmed,
        confirmable,
        100.0 * confirmed as f32 / confirmable as f32
    );
    println!();
}

/// Median and quartiles rather than a mean: a strategy that usually wins
/// narrowly but occasionally fails outright is a different animal from one
/// that wins consistently, and a mean hides exactly that.
fn print_distribution(label: &str, mut values: Vec<u32>, total_runs: usize) {
    if values.is_empty() {
        println!("{label} never reached (0/{total_runs})");
        return;
    }
    values.sort_unstable();
    let at = |q: f32| values[((values.len() - 1) as f32 * q).round() as usize];
    println!(
        "{label} reached {}/{total_runs} — median {}, p25 {}, p75 {}, min {}, max {}",
        values.len(),
        at(0.5),
        at(0.25),
        at(0.75),
        values[0],
        values[values.len() - 1],
    );
}

/// The one question this survey exists to answer.
fn verdict(exploiter: &[RunResult], explorer: &[RunResult]) {
    let mut exploiter_faster = 0;
    let mut explorer_faster = 0;
    let mut tied = 0;
    let mut compared = 0;
    for (a, b) in exploiter.iter().zip(explorer) {
        match (a.short_term_seasons, b.short_term_seasons) {
            (Some(x), Some(y)) => {
                compared += 1;
                match x.cmp(&y) {
                    std::cmp::Ordering::Less => exploiter_faster += 1,
                    std::cmp::Ordering::Greater => explorer_faster += 1,
                    std::cmp::Ordering::Equal => tied += 1,
                }
            }
            (Some(_), None) => {
                compared += 1;
                exploiter_faster += 1;
            }
            (None, Some(_)) => {
                compared += 1;
                explorer_faster += 1;
            }
            (None, None) => {}
        }
    }
    println!("## head to head (short-term objectives, same seed)");
    if compared == 0 {
        println!(
            "  neither strategy ever cleared the short-term objectives — no comparison possible"
        );
        return;
    }
    println!(
        "  exploiter faster on {exploiter_faster}/{compared}, explorer faster on {explorer_faster}/{compared}, tied {tied}"
    );
    println!(
        "  failure criterion: the exploiter winning systematically means the incentives are wrong.\n  \
         The explorer does not need to win — it needs to be competitive."
    );
}
