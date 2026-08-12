// Advances the simulation by one tick (GDD §5.6). Pure Rust, no Bevy App
// required, so determinism and balance can be tested headless.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::RngExt;

use crate::config::{EvolutionConfig, SimConfig};
use crate::state::{EraState, GameState};
use crate::world::{
    draw_species_name, net_self_interaction, ConditionalTag, Metabolism, Mode, Organism,
    SelectionPressure, SimWorld, SpeciesId, TagId, TagSlot, TerrainKind, TerrainOccupancy,
};

/// One organism's energy reached zero and it was removed this tick, with
/// the exact terms of the energy update (GDD §5.6 step 5) that led there —
/// consumed by `notebook.rs` to explain a salient death instead of leaving
/// the player to re-derive it from the tick code.
#[derive(Debug, Clone, Copy, Message)]
pub struct OrganismDied {
    pub cell: usize,
    pub species: SpeciesId,
    /// Metabolic gain (step 1-2, `Metabolism`-dependent).
    pub gain: f32,
    /// Environmental fitness (step 1, `env_fit`) at the organism's own cell
    /// this tick — task 104: distinguishes a poor temperature fit from a
    /// genuinely absent resource, which otherwise collapse into the same
    /// small `gain` number (Predator/Decomposer fold `fit` into
    /// `predation_gain`/`decomposition_gain` before `gain` is set, so this
    /// field is the only place it survives to `player_organism_death_message`).
    pub env_fit: f32,
    /// Net hidden-matrix effect summed over Moore neighbours (step 3).
    pub interaction_delta: f32,
    /// Base/predator/decomposer upkeep (step 4).
    pub upkeep: f32,
    /// `crowd_factor * occupied_neighbours` (step 4).
    pub crowding_penalty: f32,
    /// Loss to a predating neighbour, if any (`0.0` otherwise).
    pub predation_loss: f32,
    /// Energy at the start of this tick, before any of the above applied.
    pub energy_before: f32,
}

/// A species' population went from `> 0` at the start of the tick to `0` at
/// the end of it. Emitted alongside the `OrganismDied` events for its last
/// individuals, not instead of them.
#[derive(Debug, Clone, Copy, Message)]
pub struct SpeciesExtinct {
    pub species: SpeciesId,
}

/// A new organism was born via reproduction this tick (GDD §5.3's
/// `repro_threshold` mechanic) — the real "an individual crossed the
/// threshold" signal (task 063), mirroring `OrganismDied`/`SpeciesExtinct`'s
/// existing event pattern rather than leaving births to be inferred from a
/// population average.
#[derive(Debug, Clone, Copy, Message)]
pub struct OrganismBorn {
    pub species: SpeciesId,
}

/// A new descendant species was actually created by `speciate` (task 107)
/// — fired only on success, not on every `SelectionThresholdCrossed`
/// (which can no-op: cap reached, no valid tag left, self-interacting tag
/// set). Read-only consumers (`notebook.rs`'s logging) need this "it
/// actually happened" signal rather than reacting to the crossing event
/// directly, since `notebook.rs` is documented read-only with respect to
/// `SimWorld` and must not call the mutating `speciate` itself — mirrors
/// `OrganismBorn`'s relationship to the tick's raw per-organism data.
#[derive(Debug, Clone, Copy, Message)]
pub struct SpeciesEvolved {
    pub species: SpeciesId,
}

/// An era finished (TECH_DESIGN.md §4): `era` is `world.era` after the
/// increment, i.e. how many eras this world has completed so far. The hook
/// run-flow logic (failure conditions, task 041; world transition, task 045)
/// uses instead of duplicating the "era just ended" check that already lives
/// in `advance_tick`.
#[derive(Debug, Clone, Copy, Message)]
pub struct EraCompleted {
    pub era: u32,
}

/// One raw piece of evidence for the confirmation engine (task 020, GDD
/// §7): a receiver organism of `receiver_species` had a Moore neighbour
/// carrying `exerter_tag` while it itself carried `receiver_tag`.
///
/// `n_confounders` is the count of *other* distinct tags — besides
/// `exerter_tag` — present among the receiver's occupied Moore neighbours
/// this tick, i.e. how many other things could plausibly have affected the
/// receiver's energy besides the `exerter_tag -> receiver_tag` relationship
/// this record is evidence for. This is per-organism-per-hypothesis, not
/// per-neighbour-pair: an organism with exactly one neighbour, carrying only
/// `exerter_tag`, yields `n_confounders = 0` (weight 1.0, GDD §7's
/// "isolated observation" example); three other confounding tags among its
/// neighbours yields `n_confounders = 3` (weight 0.25). Only the
/// *receiver's own* tags are excluded from the count — if a neighbour
/// happens to also carry `receiver_tag`, that neighbour still counts as a
/// confounder, since a same-tag neighbour is just as plausible a source of
/// some other effect as any other tag would be.
#[derive(Debug, Clone, Copy, Message)]
pub struct AdjacencyObserved {
    pub receiver_species: SpeciesId,
    pub exerter_tag: TagSlot,
    pub receiver_tag: TagSlot,
    pub n_confounders: u32,
    /// The receiver's cell index (`SimWorld::index`), for presentation-only
    /// consumers (task 080's interaction spark) that need to know *where*
    /// this happened, not just which tags/species were involved.
    pub cell: usize,
}

/// A species' lineage set foot on a `TerrainKind` it has never occupied
/// before this run, and it carries a tag task 096 conditions on that exact
/// terrain (task 099, `redesign/abiogenesis-living-world.md` §2b) — the
/// zone-entry reveal itself, distinct from `AdjacencyObserved`'s tag-pair
/// evidence: this fires once per (species, tag, terrain), not accumulated.
#[derive(Debug, Clone, Copy, Message)]
pub struct TerrainRevealed {
    pub species: SpeciesId,
    pub tag: TagId,
    pub terrain: TerrainKind,
}

/// One raw piece of evidence for task 097's `TerrainKnowledge`: a
/// conditional tag's gate was evaluated for an organism standing on
/// `terrain`, and `passed` records whether the gate let the tag
/// participate this time. Emitted regardless of pass/fail — a failed gate
/// is itself evidence about the terrain's exclusion (`Mode::Repressible`'s
/// "turns off here" framing) — and only for tags that actually are
/// conditional (`SimWorld::conditional_gate` returns `Some`); an
/// unconditional tag never produces this event at all, so
/// `TerrainKnowledge` never accumulates evidence for it. Distinct from
/// `TerrainRevealed` (task 099): that one fires once per (species, tag,
/// terrain) the instant a lineage first sets foot on the trigger terrain, a
/// deterministic narrative beat; this one is the gradual,
/// exposure-weighted evidence track behind the catalog's confirmed-badge
/// reveal, mirroring `AdjacencyObserved`'s relationship to
/// `MatrixKnowledge`.
#[derive(Debug, Clone, Copy, Message)]
pub struct TerrainGateObserved {
    pub tag: TagSlot,
    pub terrain: TerrainKind,
    pub passed: bool,
}

/// A species' accumulated selection pressure (task 106,
/// `redesign/abiogenesis-evolution-xenotypes.md`) crossed
/// `EvolutionConfig::selection_pressure_threshold`. Mirrors `OrganismDied`'s
/// shape: a discrete signal carrying the exact terms that led to it, so a
/// consumer (task 107, speciation) doesn't have to re-derive them. Fires at
/// most once per species — see `SelectionPressure::crossed`.
///
/// **Deciding what happens on a crossing is explicitly out of scope for
/// this task** — that's task 107's job, the same way `OrganismDied` signals
/// a death without itself deciding what `notebook.rs` does with it.
#[derive(Debug, Clone, Copy, Message)]
pub struct SelectionThresholdCrossed {
    pub species: SpeciesId,
    /// Cell of the organism whose tick pushed this species' tally over the
    /// threshold — a representative location, not necessarily where the
    /// pressure was mostly accrued.
    pub cell: usize,
    /// This species' accumulated harm from negative `interaction_delta`, at
    /// the moment of crossing.
    pub interaction_harm: f32,
    /// This species' total accumulated temperature-mismatch pressure
    /// (summed across all `TerrainKind`s), at the moment of crossing.
    pub terrain_mismatch: f32,
    /// Which `TerrainKind` contributed the largest share of
    /// `terrain_mismatch` — task 107 needs this to know *which* terrain's
    /// temperature to shift `temp_optimum` toward, not just how much
    /// mismatch pressure built up.
    pub dominant_terrain: TerrainKind,
    /// This species' accumulated toxicity exposure, at the moment of
    /// crossing.
    pub toxicity: f32,
}

/// Everything `step()` produced this tick, for `advance_tick` to drain into
/// Bevy `MessageWriter`s. Kept as a plain struct (not `MessageWriter`
/// parameters on `step()` itself) so `step()` stays callable without a Bevy
/// `App` (invariant 2) — see existing `sim.rs` unit tests.
#[derive(Debug, Default)]
pub struct TickEvents {
    pub deaths: Vec<OrganismDied>,
    pub extinctions: Vec<SpeciesExtinct>,
    pub adjacencies: Vec<AdjacencyObserved>,
    pub births: Vec<OrganismBorn>,
    pub reveals: Vec<TerrainRevealed>,
    pub terrain_gates: Vec<TerrainGateObserved>,
    pub selection_thresholds: Vec<SelectionThresholdCrossed>,
}

/// Every `MessageWriter` `TickEvents` drains into, bundled into one
/// `SystemParam` (mirrors `objectives.rs`'s `ObjectiveOutcomeParams`,
/// task 059's fix for the same problem): `advance_tick`/`single_tick` were
/// already at Bevy's per-system parameter ceiling before task 106 added
/// `SelectionThresholdCrossed`'s writer, which pushed both over it.
/// Bundling here is what lets a ninth event type get added later without
/// hitting the ceiling again.
#[derive(SystemParam)]
pub struct TickEventWriters<'w> {
    died: MessageWriter<'w, OrganismDied>,
    extinct: MessageWriter<'w, SpeciesExtinct>,
    adjacencies: MessageWriter<'w, AdjacencyObserved>,
    born: MessageWriter<'w, OrganismBorn>,
    revealed: MessageWriter<'w, TerrainRevealed>,
    terrain_gates: MessageWriter<'w, TerrainGateObserved>,
    selection_thresholds: MessageWriter<'w, SelectionThresholdCrossed>,
}

impl TickEventWriters<'_> {
    pub fn write_all(&mut self, events: TickEvents) {
        self.died.write_batch(events.deaths);
        self.extinct.write_batch(events.extinctions);
        self.adjacencies.write_batch(events.adjacencies);
        self.born.write_batch(events.births);
        self.revealed.write_batch(events.reveals);
        self.terrain_gates.write_batch(events.terrain_gates);
        self.selection_thresholds
            .write_batch(events.selection_thresholds);
    }
}

/// Advances the simulation by one tick: for each occupied cell, computes
/// metabolic gain, applies costs, resolves death and reproduction. Reads
/// from `world.cells` (the previous tick's snapshot) and writes into
/// `world.scratch`, then swaps the two — no "acted this tick" guard needed,
/// no dependency on iteration order, newborns don't act this tick by
/// construction (TECH_DESIGN.md §6).
/// Whether `tag` (this world's `TagSlot`) is allowed to participate in the
/// matrix lookup for an organism whose current cell has `carrier_terrain`
/// (task 096). Unconditional tags (no entry in `world.conditional_tags`)
/// always pass, matching pre-096 behaviour exactly. `Inducible` requires the
/// carrier to be on the trigger terrain; `Repressible` requires the
/// opposite.
/// Also pushes a `TerrainGateObserved` event when `tag` is conditional
/// (task 097) — the raw evidence behind `TerrainKnowledge`'s catalog badge,
/// gathered at the exact point the gate is evaluated. Unconditional tags
/// keep short-circuiting to `true` with no event, same as before 097.
fn tag_gate_satisfied(
    world: &SimWorld,
    tag: TagSlot,
    carrier_terrain: TerrainKind,
    gate_events: &mut Vec<TerrainGateObserved>,
) -> bool {
    let tag_id = world.active_tags[tag.0 as usize];
    let Some(conditional) = world.conditional_gate(tag_id) else {
        return true;
    };
    let passed = match conditional.mode {
        Mode::Inducible => carrier_terrain == conditional.terrain,
        Mode::Repressible => carrier_terrain != conditional.terrain,
    };
    gate_events.push(TerrainGateObserved {
        tag,
        terrain: carrier_terrain,
        passed,
    });
    passed
}

/// Marks `species_id`'s occupancy of `terrain` (task 099) and, if this is
/// newly set and `species_tags` includes a tag task 096 conditions on this
/// exact terrain, queues a one-time reveal. Takes explicit, disjoint
/// borrows of `SimWorld`'s fields (rather than `&mut SimWorld`) so it can be
/// called from `step`'s per-organism loop while `species` stays borrowed
/// from `world.species` for the rest of that iteration (`repro_threshold`,
/// etc.) — a method taking `&mut self` would conflict with that borrow even
/// though the fields touched here are disjoint from it.
#[allow(clippy::too_many_arguments)]
fn mark_terrain_and_maybe_reveal(
    terrain_occupancy: &mut Vec<TerrainOccupancy>,
    active_tags: &[TagId],
    conditional_tags: &[ConditionalTag],
    species_len: usize,
    species_id: SpeciesId,
    species_tags: &[TagSlot],
    terrain: TerrainKind,
    reveals: &mut Vec<TerrainRevealed>,
) {
    if terrain_occupancy.len() < species_len {
        terrain_occupancy.resize(species_len, TerrainOccupancy::default());
    }
    let newly_occupied = terrain_occupancy[species_id.0 as usize].mark(terrain);
    if !newly_occupied {
        return;
    }
    for &slot in species_tags {
        let tag_id = active_tags[slot.0 as usize];
        if let Some(conditional) = conditional_tags.iter().find(|c| c.tag == tag_id) {
            if conditional.terrain == terrain {
                reveals.push(TerrainRevealed {
                    species: species_id,
                    tag: tag_id,
                    terrain,
                });
            }
        }
    }
}

/// Accumulates one organism-tick's worth of selection pressure (task 106)
/// into its species' running tally, growing `pressures` lazily the same way
/// `mark_terrain_and_maybe_reveal` grows `terrain_occupancy`. Returns
/// `Some(SelectionThresholdCrossed)` exactly on the tick this species'
/// total pressure first reaches `threshold` — `None` on every other call,
/// including repeat calls once already crossed (`SelectionPressure::crossed`
/// guards this, mirroring `MatrixKnowledge::record`'s `was_confirmed` guard).
///
/// Takes each stimulus pre-computed (`interaction_delta`, `fit`) rather than
/// recomputing them — both are already in scope in `step`'s per-organism
/// loop, so this must not re-derive them (no duplicate matrix-neighbour
/// scan).
#[allow(clippy::too_many_arguments)]
fn accumulate_selection_pressure(
    pressures: &mut Vec<SelectionPressure>,
    evolution: &EvolutionConfig,
    species_len: usize,
    species_id: SpeciesId,
    cell: usize,
    interaction_delta: f32,
    fit: f32,
    terrain: TerrainKind,
    toxicity: f32,
) -> Option<SelectionThresholdCrossed> {
    if pressures.len() < species_len {
        pressures.resize(species_len, SelectionPressure::default());
    }
    let pressure = &mut pressures[species_id.0 as usize];
    if pressure.crossed {
        return None;
    }

    pressure.interaction_harm += (-interaction_delta).max(0.0) * evolution.interaction_harm_weight;
    pressure.terrain_mismatch[terrain.index()] += (1.0 - fit) * evolution.terrain_mismatch_weight;
    pressure.toxicity += toxicity * evolution.toxicity_weight;

    if pressure.total() < evolution.selection_pressure_threshold {
        return None;
    }
    pressure.crossed = true;

    let dominant_terrain_idx = pressure
        .terrain_mismatch
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(idx, _)| idx)
        .expect("terrain_mismatch is a fixed non-empty array");

    Some(SelectionThresholdCrossed {
        species: species_id,
        cell,
        interaction_harm: pressure.interaction_harm,
        terrain_mismatch: pressure.terrain_mismatch.iter().sum(),
        dominant_terrain: TerrainKind::from_index(dominant_terrain_idx),
        toxicity: pressure.toxicity,
    })
}

/// Which of task 106's three stimuli contributed the largest share of the
/// pressure that crossed the threshold — decides which edit
/// `speciate` applies (task 107's first-pass stimulus→edit mapping,
/// `redesign/abiogenesis-evolution-xenotypes.md`, extended 2026-08-12).
/// Ties broken in a fixed preference order (interaction, then terrain, then
/// toxicity) — arbitrary, but deterministic, matching this codebase's "no
/// `rand::rng()`, reproducible from the world's own seed" invariant; a real
/// three-way tie is vanishingly unlikely given these are independent `f32`
/// accumulations, so which side of the tie-break it falls on barely
/// matters in practice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DominantStimulus {
    /// Sustained negative `interaction_delta` (repeatedly harmed by
    /// adjacency) → the descendant gains an additional tag.
    InteractionHarm,
    /// Sustained temperature mismatch on the terrain actually occupied →
    /// the descendant's `temp_optimum` shifts toward it.
    TerrainMismatch,
    /// Sustained `toxicity` exposure → the descendant gains `Sea`
    /// placement tolerance (see `SimWorld::sea_tolerant_species`'s doc
    /// comment for why this, not literal toxicity tolerance, is what's
    /// actually implemented here).
    Toxicity,
}

fn dominant_stimulus(event: &SelectionThresholdCrossed) -> DominantStimulus {
    if event.interaction_harm >= event.terrain_mismatch && event.interaction_harm >= event.toxicity
    {
        DominantStimulus::InteractionHarm
    } else if event.terrain_mismatch >= event.toxicity {
        DominantStimulus::TerrainMismatch
    } else {
        DominantStimulus::Toxicity
    }
}

/// GDD §5.3's tags-per-species cap, mirrored from `apply_splice`
/// (`input.rs`'s `AddTag` handling) rather than read from
/// `TagConfig::tags_per_species_max` — `apply_splice` itself hardcodes `3`
/// today, and this function is meant to behave identically to it for the
/// same edit, not introduce a second source of truth that could drift from
/// the one `Splice` already uses.
const MAX_TAGS_PER_SPECIES: usize = 3;

/// Task 107's descendant-creation path: on a qualifying
/// `SelectionThresholdCrossed`, appends a new `Species` to `world.species`
/// — the source species is never mutated, only a new entry is appended
/// (the source doc's load-bearing "evolution never mutates in place"
/// decision). Mirrors `apply_splice`'s exact shape (`input.rs:505-578`):
/// clone, edit, draw an independent name, push, allocate `SpeciesId`.
///
/// Returns `None` (a silent no-op, matching `apply_splice`'s own rejection
/// behavior) when: the triggering species no longer exists; the world is
/// already at `EvolutionConfig::max_species` (the hard `u8`-wraparound
/// safety cap — checked here, not merely documented); the dominant
/// stimulus is `InteractionHarm` but the species already carries every
/// active tag or is already at the cap; or the resulting tag set would
/// self-interact (mirrors `apply_splice`'s `net_self_interaction` guard).
///
/// **Founder placement**: per this task's own documented choice (the
/// simpler of two options the source doc left open), the triggering
/// organism's cell is reassigned to the new species — no new placement
/// logic. If that organism is no longer there by the time this runs (e.g.
/// it died in between the crossing and this system running), the
/// descendant is still created — it just starts at population 0, exactly
/// like a player `Splice` output before the player seeds one.
pub fn speciate(
    world: &mut SimWorld,
    config: &SimConfig,
    event: &SelectionThresholdCrossed,
) -> Option<SpeciesId> {
    if world.species.len() >= config.evolution.max_species {
        return None;
    }
    let source_species = world.species.get(event.species.0 as usize)?;
    let mut new_species = source_species.clone();
    new_species.name = draw_species_name(world);

    let grants_sea_tolerance = match dominant_stimulus(event) {
        DominantStimulus::InteractionHarm => {
            if new_species.tags.len() >= MAX_TAGS_PER_SPECIES {
                return None;
            }
            // First active tag not already on the species, in `TagSlot`
            // order — deterministic, no RNG draw needed for this pick.
            let slot = (0..world.active_tags.len())
                .map(|i| TagSlot(i as u8))
                .find(|slot| !new_species.tags.contains(slot))?;
            new_species.tags.push(slot);
            if net_self_interaction(&world.matrix, &new_species.tags) != 0 {
                return None;
            }
            false
        }
        DominantStimulus::TerrainMismatch => {
            let warmer = world.cells[event.cell].temperature > new_species.temp_optimum;
            let delta = if warmer {
                config.energy.splice_temp_shift
            } else {
                -config.energy.splice_temp_shift
            };
            new_species.temp_optimum = (new_species.temp_optimum + delta).clamp(0.0, 1.0);
            false
        }
        DominantStimulus::Toxicity => true,
    };

    let new_species_id = world.push_species(new_species);
    if grants_sea_tolerance {
        world.sea_tolerant_species.push(new_species_id);
    }
    if let Some(organism) = world.cells[event.cell].organism.as_mut() {
        if organism.species == event.species {
            organism.species = new_species_id;
        }
    }
    Some(new_species_id)
}

pub fn step(world: &mut SimWorld, config: &SimConfig) -> TickEvents {
    let energy = &config.energy;
    debug_assert!(
        energy.residue_ambient_trickle < energy.residue_decay,
        "residue_ambient_trickle must stay below residue_decay, or residue grows unboundedly"
    );
    let mut events = TickEvents::default();

    // Pre-tick population count per species, for extinction detection
    // (1 -> 0 transition) once deaths are recorded below.
    let mut population = vec![0u32; world.species.len()];
    for cell in world.cells.iter() {
        if let Some(organism) = cell.organism {
            population[organism.species.0 as usize] += 1;
        }
    }
    // Task 050: worlds start with nothing placed, so `ever_populated` only
    // flips once the player's first `Seed` actually lands on the grid —
    // `objectives::is_total_extinction` reads it to avoid failing a world
    // that simply hasn't been seeded yet. Task 098's wild populations are
    // excluded from this scan: they exist on the grid from world start
    // regardless of anything the player does, so counting them here would
    // flip `ever_populated` on tick 0 and defeat its purpose.
    let any_player_population = population
        .iter()
        .enumerate()
        .any(|(idx, &count)| count > 0 && !world.is_wild(SpeciesId(idx as u8)));
    if !world.ever_populated && any_player_population {
        world.ever_populated = true;
    }
    // Task 103 follow-up: per-species "era first placed" — distinct from
    // `species_origin_era` (registry-creation time, always the world's
    // starting era for the initial roster regardless of when the player
    // actually seeds one). Set once, the first tick this species' pre-tick
    // population is observed positive; `world.era` hasn't advanced yet this
    // tick (that only happens in `tick_and_complete_era`, after `step`
    // returns), so this is the era the player was actually looking at when
    // they placed it. Unlike `ever_populated` above, wild populations do
    // count here — the catalog row only needs to know when the species
    // first existed on the grid, not whether the player specifically
    // placed it.
    if world.species_seeded_era.len() < world.species.len() {
        world.species_seeded_era.resize(world.species.len(), None);
    }
    for (idx, &count) in population.iter().enumerate() {
        if count > 0 && world.species_seeded_era[idx].is_none() {
            world.species_seeded_era[idx] = Some(world.era);
        }
    }

    // Start the write side as a copy of the snapshot; every field below
    // (environment scalars, residue, organism) gets overwritten in place.
    world.scratch.copy_from_slice(&world.cells);

    // Environmental diffusion (GDD §5.2): reads neighbours from the
    // snapshot (`world.cells`), writes into `world.scratch`, so it must run
    // after the copy above or its writes would be clobbered by it.
    world.diffuse_environment(config);
    // Counters diffusion's erosion of heat sources / Sea coolant (task 085):
    // a distinct step from the blur above, so `diffuse_environment`'s own
    // fixed-point tests stay unaffected by it.
    world.reinject_environment_sources(config);

    // Residue decays every tick unless a death overwrites it further down.
    // A small ambient trickle is added after decay so residue settles to a
    // low equilibrium everywhere, keeping an isolated Decomposer readable
    // instead of starving out with zero information.
    for cell in world.scratch.iter_mut() {
        cell.residue =
            (cell.residue - energy.residue_decay).max(0.0) + energy.residue_ambient_trickle;
    }

    // Predation pre-pass (GDD §5.4): a shared-resource drain computed from
    // the immutable snapshot, into per-cell accumulators, before the main
    // loop — see TECH_DESIGN.md §6 "Shared resource drain". This keeps the
    // tick order-independent: a predator never writes directly into a prey's
    // scratch entry while iterating.
    let mut predation_gain = vec![0.0f32; world.cells.len()];
    let mut predation_loss = vec![0.0f32; world.cells.len()];
    for (idx, cell) in world.cells.iter().enumerate() {
        let Some(organism) = cell.organism else {
            continue;
        };
        let species = &world.species[organism.species.0 as usize];
        if species.metabolism != Metabolism::Predator {
            continue;
        }
        let (x, y) = (idx % world.width, idx / world.width);
        let prey: Vec<usize> = world
            .moore_neighbours(x, y)
            .filter(|&n| world.cells[n].organism.is_some())
            .collect();
        if prey.is_empty() {
            continue;
        }
        let fit = env_fit(
            cell.temperature,
            species.temp_optimum,
            species.temp_tolerance,
        );
        let available: f32 = prey
            .iter()
            .map(|&n| world.cells[n].organism.unwrap().energy)
            .sum();
        let drawn = (energy.predator_drain_cap * fit).min(available);
        predation_gain[idx] = drawn;
        // Split evenly across prey neighbours (GDD doesn't pin down a
        // species-specific targeting rule for Phase 1).
        let share = drawn / prey.len() as f32;
        for &n in &prey {
            predation_loss[n] += share;
        }
    }

    // Decomposition pre-pass (GDD §5.4): extends the shared-resource drain
    // pattern above to residue. Draws from the decomposer's own cell plus
    // its Moore neighbours, reading `world.scratch`'s residue — which has
    // already been decayed above — so decay and extraction compose in a
    // fixed, documented order (decay first) rather than one overwriting the
    // other (TECH_DESIGN.md §6 "Shared resource drain").
    let mut decomposition_gain = vec![0.0f32; world.cells.len()];
    let mut residue_loss = vec![0.0f32; world.cells.len()];
    for (idx, cell) in world.cells.iter().enumerate() {
        let Some(organism) = cell.organism else {
            continue;
        };
        let species = &world.species[organism.species.0 as usize];
        if species.metabolism != Metabolism::Decomposer {
            continue;
        }
        let (x, y) = (idx % world.width, idx / world.width);
        let sources: Vec<usize> = std::iter::once(idx)
            .chain(world.moore_neighbours(x, y))
            .filter(|&n| world.scratch[n].residue > 0.0)
            .collect();
        if sources.is_empty() {
            continue;
        }
        let fit = env_fit(
            cell.temperature,
            species.temp_optimum,
            species.temp_tolerance,
        );
        let available: f32 = sources.iter().map(|&n| world.scratch[n].residue).sum();
        let drawn = (energy.decomposer_extract_rate * fit).min(available);
        decomposition_gain[idx] = drawn;
        // Distribute proportionally to each source's residue share, capped
        // at that source's own residue so a single decomposer's draw can
        // never overdraw one of its sources.
        for &n in &sources {
            let share = drawn * world.scratch[n].residue / available;
            residue_loss[n] += share.min(world.scratch[n].residue);
        }
    }

    // Apply the accumulated extraction. A final clamp to 0.0 is the
    // multi-decomposer safety net: two decomposers competing for the same
    // residue each compute their share against the same pre-extraction
    // snapshot, so their combined draw can exceed what's actually there —
    // residue must never go negative regardless.
    for (cell, loss) in world.scratch.iter_mut().zip(residue_loss.iter()) {
        cell.residue = (cell.residue - loss).max(0.0);
    }

    for idx in 0..world.cells.len() {
        let cell = world.cells[idx];
        let Some(organism) = cell.organism else {
            continue;
        };
        let species = &world.species[organism.species.0 as usize];

        // 1-2. Environmental fitness and metabolic gain.
        let fit = env_fit(
            cell.temperature,
            species.temp_optimum,
            species.temp_tolerance,
        );
        let gain = match species.metabolism {
            Metabolism::Photolithic => cell.light * energy.photolithic_metabolism_gain * fit,
            Metabolism::Predator => predation_gain[idx],
            Metabolism::Decomposer => decomposition_gain[idx],
            Metabolism::Chemolithotroph => {
                cell.toxicity * energy.chemolithotroph_metabolism_gain * fit
            }
        };

        // 3. Hidden matrix effect (GDD §5.6 step 3, §5.5): additive and
        // linear (invariant 4), read only from the snapshot like everything
        // else here, so the tick stays order-independent. For every
        // occupied Moore neighbour, for every (their tag, my tag) pair, sum
        // the matrix entry — row = exerting tag, column = receiving tag.
        let (x, y) = (idx % world.width, idx / world.width);
        let mut interaction_delta = 0.0;

        // Distinct exerter tags carried by this organism's occupied Moore
        // neighbours, gathered up front so the confounder count (see
        // `AdjacencyObserved`'s doc comment) is available for every
        // observation emitted below without re-scanning neighbours per tag.
        let mut neighbour_tags: Vec<TagSlot> = Vec::new();
        for neighbour_idx in world.moore_neighbours(x, y) {
            if let Some(neighbour) = world.cells[neighbour_idx].organism {
                let neighbour_species = &world.species[neighbour.species.0 as usize];
                for &tag in &neighbour_species.tags {
                    if !neighbour_tags.contains(&tag) {
                        neighbour_tags.push(tag);
                    }
                }
            }
        }

        for neighbour_idx in world.moore_neighbours(x, y) {
            let Some(neighbour) = world.cells[neighbour_idx].organism else {
                continue;
            };
            let neighbour_species = &world.species[neighbour.species.0 as usize];
            for &their_tag in &neighbour_species.tags {
                if !tag_gate_satisfied(
                    world,
                    their_tag,
                    world.cells[neighbour_idx].terrain,
                    &mut events.terrain_gates,
                ) {
                    continue;
                }
                for &my_tag in &species.tags {
                    if !tag_gate_satisfied(world, my_tag, cell.terrain, &mut events.terrain_gates) {
                        continue;
                    }
                    let entry = world.matrix.get(their_tag, my_tag);
                    interaction_delta += entry as f32;
                    if entry != 0 {
                        let n_confounders =
                            neighbour_tags.iter().filter(|&&t| t != their_tag).count() as u32;
                        events.adjacencies.push(AdjacencyObserved {
                            receiver_species: organism.species,
                            exerter_tag: their_tag,
                            receiver_tag: my_tag,
                            n_confounders,
                            cell: idx,
                        });
                    }
                }
            }
        }

        // Task 106: accumulate this tick's selection-pressure stimuli
        // (interaction harm, terrain/temp-optimum mismatch, toxicity) into
        // the organism's species tally, before costs/death are resolved —
        // the pressure reflects exposure this tick regardless of whether
        // the organism survives it.
        if let Some(crossed) = accumulate_selection_pressure(
            &mut world.selection_pressure,
            &config.evolution,
            world.species.len(),
            organism.species,
            idx,
            interaction_delta,
            fit,
            cell.terrain,
            cell.toxicity,
        ) {
            events.selection_thresholds.push(crossed);
        }

        // 4. Costs: base upkeep plus a carrying-capacity penalty per
        // occupied neighbour, read from the snapshot so the tick stays
        // order-independent.
        let occupied_neighbours = world
            .moore_neighbours(x, y)
            .filter(|&n| world.cells[n].organism.is_some())
            .count();
        let upkeep = match species.metabolism {
            Metabolism::Photolithic => energy.base_upkeep,
            Metabolism::Predator => energy.predator_upkeep,
            Metabolism::Decomposer => energy.decomposer_upkeep,
            Metabolism::Chemolithotroph => energy.chemolithotroph_upkeep,
        };
        let crowding_penalty = energy.crowd_factor * occupied_neighbours as f32;

        // 5. Energy update.
        let new_energy = organism.energy + gain + interaction_delta
            - upkeep
            - crowding_penalty
            - predation_loss[idx];

        // 6. Death.
        if new_energy <= 0.0 {
            world.scratch[idx].organism = None;
            // Fixed on death, not decayed leftover + this value (invariant
            // in task spec): a death this tick overwrites the residue.
            world.scratch[idx].residue = energy.residue_on_death;
            events.deaths.push(OrganismDied {
                cell: idx,
                species: organism.species,
                gain,
                env_fit: fit,
                interaction_delta,
                upkeep,
                crowding_penalty,
                predation_loss: predation_loss[idx],
                energy_before: organism.energy,
            });
            let species_idx = organism.species.0 as usize;
            population[species_idx] -= 1;
            if population[species_idx] == 0 {
                events.extinctions.push(SpeciesExtinct {
                    species: organism.species,
                });
            }
            continue;
        }

        world.scratch[idx].organism = Some(Organism {
            energy: new_energy,
            ..organism
        });
        mark_terrain_and_maybe_reveal(
            &mut world.terrain_occupancy,
            &world.active_tags,
            &world.conditional_tags,
            world.species.len(),
            organism.species,
            &species.tags,
            cell.terrain,
            &mut events.reveals,
        );

        // 7. Reproduction: only if there's still an empty, placeable
        // neighbour once the birth cell is picked (task 067 — offspring
        // never spawn onto Sea or a mountain peak). Empty neighbours are
        // collected from the snapshot in index order, so the RNG draw is
        // reproducible; the scratch buffer is re-checked to resolve
        // contention between two parents claiming the same cell this tick.
        if new_energy >= species.repro_threshold && organism.born_era < world.era {
            // Cloned rather than borrowed: `world.rng_mut()` below needs
            // `&mut world` as a whole (it's a method, so the borrow checker
            // can't see that it's disjoint from `world.species`), which
            // would conflict with a borrow still reaching into `species`
            // afterwards for `mark_terrain_and_maybe_reveal`.
            let repro_tags = species.tags.clone();
            let empty_neighbours: Vec<usize> = world
                .moore_neighbours(x, y)
                .filter(|&n| {
                    world.cells[n].organism.is_none()
                        && world.is_placeable_index_for(n, organism.species)
                })
                .collect();
            if !empty_neighbours.is_empty() {
                let pick = world.rng_mut().random_range(0..empty_neighbours.len());
                let target = empty_neighbours[pick];
                if world.scratch[target].organism.is_none() {
                    world.scratch[target].organism = Some(Organism {
                        species: organism.species,
                        energy: energy.repro_cost,
                        born_era: world.era,
                    });
                    world.scratch[idx].organism = Some(Organism {
                        energy: new_energy - energy.repro_cost,
                        ..organism
                    });
                    mark_terrain_and_maybe_reveal(
                        &mut world.terrain_occupancy,
                        &world.active_tags,
                        &world.conditional_tags,
                        world.species.len(),
                        organism.species,
                        &repro_tags,
                        world.cells[target].terrain,
                        &mut events.reveals,
                    );
                    events.births.push(OrganismBorn {
                        species: organism.species,
                    });
                }
            }
        }
    }

    std::mem::swap(&mut world.cells, &mut world.scratch);
    world.tick += 1;
    events
}

/// Gaussian environmental fitness around the species' thermal optimum (GDD §5.9).
fn env_fit(temperature: f32, optimum: f32, tolerance: f32) -> f32 {
    let d = temperature - optimum;
    (-(d * d) / (2.0 * tolerance * tolerance)).exp()
}

/// Groups the per-era simulation advancement (TECH_DESIGN.md §3.4).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    Advance,
}

/// Ticks left in the era currently being animated (TECH_DESIGN.md §3.4).
#[derive(Resource, Default)]
pub struct EraProgress {
    pub(crate) remaining: u32,
}

impl EraProgress {
    pub fn start(&mut self, ticks: u32) {
        self.remaining = ticks;
    }

    pub fn remaining(&self) -> u32 {
        self.remaining
    }

    /// Used by the `r` key (world reset) to cancel any era in progress.
    pub fn cancel(&mut self) {
        self.remaining = 0;
    }
}

/// The player's action points for the current `EraState::Observing` window
/// (GDD §6): `Seed` and, from tasks 023-025, Stress/Cull/Splice each spend
/// from this pool. Refilled to `config.time.point_budget_per_era` whenever
/// an era finishes (`advance_tick`'s "era just ended" branch) and by the
/// `r` key alongside every other piece of fresh-world state.
#[derive(Resource, Default)]
pub struct ActionBudget {
    pub points_remaining: u32,
}

impl ActionBudget {
    pub fn refill(&mut self, points: u32) {
        self.points_remaining = points;
    }

    /// Spends `cost` points if affordable, returning whether it succeeded.
    /// Leaves `points_remaining` untouched on failure — callers should
    /// treat a `false` return as "do nothing", not a partial spend.
    pub fn try_spend(&mut self, cost: u32) -> bool {
        if self.points_remaining >= cost {
            self.points_remaining -= cost;
            true
        } else {
            false
        }
    }
}

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        let config = app.world().resource::<SimConfig>();
        let era_tick_hz = config.time.era_tick_hz as f64;
        let point_budget_per_era = config.time.point_budget_per_era;
        app.insert_resource(Time::<Fixed>::from_hz(era_tick_hz));
        app.init_resource::<EraProgress>();
        // `init_resource` alone would leave `points_remaining = 0` (the
        // `Default` impl) until the first era finishes — the very first
        // `Observing` window needs a full budget too.
        app.insert_resource(ActionBudget {
            points_remaining: point_budget_per_era,
        });
        app.add_message::<OrganismDied>();
        app.add_message::<SpeciesExtinct>();
        app.add_message::<AdjacencyObserved>();
        app.add_message::<EraCompleted>();
        app.add_message::<OrganismBorn>();
        app.add_message::<TerrainRevealed>();
        app.add_message::<TerrainGateObserved>();
        app.add_message::<SelectionThresholdCrossed>();
        app.add_message::<SpeciesEvolved>();
        app.add_systems(
            FixedUpdate,
            advance_tick
                .in_set(SimSet::Advance)
                .run_if(in_state(EraState::Advancing)),
        );
        // `Update`, not `FixedUpdate`/`SimSet::Advance` (task 107): this
        // only reads already-drained `SelectionThresholdCrossed` messages
        // and mutates `SimWorld` in response — it doesn't need to run in
        // lockstep with tick advancement, and gating it to
        // `EraState::Advancing` would miss crossings produced by
        // `single_tick`'s manual-tick path (`input.rs`), which runs
        // outside `Advancing`. Mirrors `notebook.rs`'s message-consuming
        // systems' scheduling (`Update`, gated on `GameState::Playing`
        // only) even though this one mutates `SimWorld` and they don't —
        // `notebook.rs` is documented read-only, so the mutating half of
        // task 107 belongs here in `sim`, not there.
        app.add_systems(
            Update,
            speciate_on_threshold_crossed.run_if(in_state(GameState::Playing)),
        );
    }
}

/// Advances one tick of the currently-animating era, then transitions back
/// to `Observing` once `era_ticks` have been played out. Guards against a
/// stray extra `FixedUpdate` execution landing before the state transition
/// takes effect, which would otherwise run one tick too many.
///
/// Each parameter is a distinct Bevy resource/writer this system needs, not
/// incidental complexity — splitting it wouldn't reduce the coupling, only
/// hide it, so the arg count is allowed rather than fought.
/// Runs one tick and, if it was the era's last tick, performs the
/// era-completion bookkeeping (`world.era` advance, budget refill,
/// `EraCompleted`). Shared by `advance_tick`'s auto-play and `single_tick`'s
/// manual step (`input.rs`) so an era is exactly `era_ticks` ticks — and
/// completes identically — regardless of which key triggered them.
pub fn tick_and_complete_era(
    world: &mut SimWorld,
    config: &SimConfig,
    progress: &mut EraProgress,
    budget: &mut ActionBudget,
    era_completed: &mut MessageWriter<EraCompleted>,
) -> TickEvents {
    let events = step(world, config);
    progress.remaining -= 1;
    if progress.remaining() == 0 {
        world.era += 1;
        budget.refill(config.time.point_budget_per_era);
        era_completed.write(EraCompleted { era: world.era });
    }
    events
}

fn advance_tick(
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    mut budget: ResMut<ActionBudget>,
    mut era_completed: MessageWriter<EraCompleted>,
    mut writers: TickEventWriters,
) {
    if progress.remaining() == 0 {
        return;
    }
    let events = tick_and_complete_era(
        &mut world,
        &config,
        &mut progress,
        &mut budget,
        &mut era_completed,
    );
    writers.write_all(events);
    if progress.remaining() == 0 {
        next_state.set(EraState::Observing);
    }
}

/// Thin Bevy wrapper (task 107) around the pure `speciate`: reads every
/// `SelectionThresholdCrossed` drained this frame and, for each one that
/// actually produces a descendant, writes `SpeciesEvolved` for read-only
/// consumers (`notebook.rs`'s logging) to react to.
fn speciate_on_threshold_crossed(
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut crossed: MessageReader<SelectionThresholdCrossed>,
    mut evolved: MessageWriter<SpeciesEvolved>,
) {
    for event in crossed.read() {
        if let Some(species) = speciate(&mut world, &config, event) {
            evolved.write(SpeciesEvolved { species });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{
        Cell, ConditionalTag, Species, SpeciesId, TagId, TagMatrix, TagSlot, TerrainKind,
    };

    const TOLERANCE: f32 = 1e-4;

    fn world_with_one_organism(light: f32, temperature: f32, energy: f32) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        // These are pure energy-formula tests, not terrain tests: force
        // every cell to plain, unpeaked terrain so reproduction's placement
        // gating (task 067) never depends on the generated map's incidental
        // sea/mountain layout for this seed — otherwise a terrain retune
        // (e.g. task 069's follow-up sea-balance correction) can silently
        // change these tests' outcomes by blocking or enabling reproduction
        // spread near the organism under test.
        for cell in world.cells.iter_mut() {
            cell.terrain = TerrainKind::Plain;
            cell.is_peak = false;
        }
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: temperature,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        world.cells[idx] = Cell {
            light,
            temperature,
            organism: Some(Organism {
                species: SpeciesId(0),
                energy,
                born_era: 0,
            }),
            ..world.cells[idx]
        };
        (world, config)
    }

    #[test]
    fn isolated_photolithic_grows() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let organism = world.get(cx, cy).organism.expect("organism survives");
        assert!(
            (organism.energy - 5.9).abs() < TOLERANCE,
            "expected net +0.9, got {}",
            organism.energy - 5.0
        );
    }

    /// Task 067: an organism ready to reproduce, surrounded by occupied
    /// neighbours except a single cell — which is `Sea`, not truly empty.
    /// Reproduction must not treat it as a valid target just because it
    /// holds no organism.
    #[test]
    fn reproduction_never_spawns_onto_an_unplaceable_neighbour() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 9.5);
        let (cx, cy) = (world.width / 2, world.height / 2);

        let neighbours: Vec<usize> = world.moore_neighbours(cx, cy).collect();
        for &idx in neighbours.iter().skip(1) {
            world.cells[idx].organism = Some(Organism {
                species: SpeciesId(0),
                energy: 1.0,
                born_era: 0,
            });
        }
        let target = neighbours[0];
        world.cells[target].terrain = TerrainKind::Sea;

        let events = step(&mut world, &config);

        assert!(
            world.cells[target].organism.is_none(),
            "Sea must never receive an offspring, even as the only occupancy-empty neighbour"
        );
        assert!(
            events.births.is_empty(),
            "reproduction must not succeed with no placeable empty neighbour"
        );
    }

    /// Task 083: an organism born this era must not reproduce even with
    /// energy above `repro_threshold` — it has to survive into a later era
    /// first (`born_era < world.era`).
    #[test]
    fn newborn_cannot_reproduce_until_a_later_era() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 15.0);

        let events = step(&mut world, &config);
        assert!(
            events.births.is_empty(),
            "an organism born this era (born_era == world.era) must not reproduce yet"
        );

        world.era += 1;
        let events = step(&mut world, &config);
        assert!(
            !events.births.is_empty(),
            "once world.era has advanced past born_era, reproduction may proceed"
        );
    }

    #[test]
    fn crowded_photolithic_stalls_at_carrying_capacity() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        let (cx, cy) = (world.width / 2, world.height / 2);

        // Fill 7 of the 8 Moore neighbours with organisms of the same species,
        // far enough from repro_threshold that they don't reproduce this tick.
        let neighbours: Vec<usize> = world.moore_neighbours(cx, cy).collect();
        for &idx in neighbours.iter().take(7) {
            world.cells[idx].organism = Some(Organism {
                species: SpeciesId(0),
                energy: 1.0,
                born_era: 0,
            });
        }

        step(&mut world, &config);

        let organism = world.get(cx, cy).organism.expect("organism survives");
        assert!(
            (organism.energy - 4.85).abs() < TOLERANCE,
            "expected net -0.15, got {}",
            organism.energy - 5.0
        );
    }

    #[test]
    fn photolithic_in_the_dark_eventually_dies() {
        // GDD §5.9: gain 0.4 < upkeep 0.5 ⇒ net -0.1/tick, doesn't survive.
        // Starting energy is 5.0, so death takes ~50 ticks, not the first one.
        // Diffusion disabled (task 074): otherwise this cell's forced-dark
        // light blends back toward the ambient gradient over the run, which
        // eventually turns the net gain positive and lets the organism
        // survive/reproduce — a diffusion-timing artifact, not the light
        // niche this test means to isolate.
        let (mut world, mut config) = world_with_one_organism(0.2, 0.5, 5.0);
        config.environment.diffusion_rate = 0.0;
        let (cx, cy) = (world.width / 2, world.height / 2);

        step(&mut world, &config);
        let organism = world.get(cx, cy).organism.expect("survives the first tick");
        assert!(
            (organism.energy - 4.9).abs() < TOLERANCE,
            "expected net -0.1/tick in the dark, got {}",
            organism.energy - 5.0
        );

        for _ in 0..200 {
            if world.get(cx, cy).organism.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).organism.is_none(),
            "light niche: should not survive long-term"
        );
        assert!((world.get(cx, cy).residue - config.energy.residue_on_death).abs() < TOLERANCE);
    }

    #[test]
    fn tick_counter_increments() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        assert_eq!(world.tick, 0);
        step(&mut world, &config);
        assert_eq!(world.tick, 1);
    }

    #[test]
    fn step_marks_the_world_as_ever_populated_once_an_organism_exists() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        assert!(!world.ever_populated);
        step(&mut world, &config);
        assert!(world.ever_populated);
    }

    /// Task 103 follow-up: the catalog's "seeded era" label must not appear
    /// for a species still sitting in the available roster with nothing
    /// ever placed; must show the era it was *actually placed* in, not the
    /// era it was merely added to the registry (the original bug — a
    /// species from the starting roster is always registered at era 0
    /// regardless of when the player later seeds one); and must keep
    /// showing that era once set, even after the species later goes fully
    /// extinct.
    #[test]
    fn species_seeded_era_tracks_actual_placement_not_registry_creation() {
        let (mut world, mut config) = world_with_one_organism(0.2, 0.5, 5.0);
        config.environment.diffusion_rate = 0.0;
        // A second species, registered (era 0) but never placed on the grid.
        world.push_species(Species {
            name: "Unplaced".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });

        step(&mut world, &config);
        assert_eq!(world.species_seeded_era, vec![Some(0), None]);

        // Simulate several eras having passed, then the player finally
        // seeding species 1 mid-run — its seeded era must be the current
        // era, not the era it was registered in (0).
        world.era = 3;
        let target = world
            .cells
            .iter()
            .position(|c| c.organism.is_none())
            .expect("an empty cell exists");
        world.cells[target].organism = Some(Organism {
            species: SpeciesId(1),
            energy: config.energy.seed_energy,
            born_era: world.era,
        });
        step(&mut world, &config);
        assert_eq!(
            world.species_seeded_era[1],
            Some(3),
            "must record the era it was actually placed in, not registry-creation era 0"
        );

        let (cx, cy) = (world.width / 2, world.height / 2);
        for _ in 0..200 {
            if world.get(cx, cy).organism.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).organism.is_none(),
            "organism should have starved in the dark"
        );
        assert_eq!(
            world.species_seeded_era[0],
            Some(0),
            "species_seeded_era must not reset once set, even after extinction"
        );
    }

    /// Drives `advance_tick` on `Update` (rather than `FixedUpdate`) so the
    /// count doesn't depend on wall-clock accumulation — this isolates the
    /// counting/guard logic from schedule timing, which task 007 requires to
    /// be exact regardless of frame-rate hitches.
    #[test]
    fn era_advances_exactly_era_ticks_then_stops_at_observing() {
        let config = SimConfig::default();
        let (world, _) = world_with_one_organism(0.7, 0.5, 5.0);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.insert_resource(config.clone());
        app.insert_resource(world);
        app.init_state::<GameState>();
        app.add_sub_state::<EraState>();
        app.init_resource::<EraProgress>();
        app.init_resource::<ActionBudget>();
        app.add_message::<OrganismDied>();
        app.add_message::<SpeciesExtinct>();
        app.add_message::<AdjacencyObserved>();
        app.add_message::<EraCompleted>();
        app.add_message::<OrganismBorn>();
        app.add_message::<TerrainRevealed>();
        app.add_message::<TerrainGateObserved>();
        app.add_message::<SelectionThresholdCrossed>();
        app.add_message::<SpeciesEvolved>();
        app.add_systems(Update, advance_tick.run_if(in_state(EraState::Advancing)));

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // apply GameState transition, establishing EraState::Observing

        // Spend the whole starting budget before the era advances, so a
        // refill (vs. just leaving it untouched) is what's actually tested.
        app.world_mut()
            .resource_mut::<ActionBudget>()
            .points_remaining = 0;

        app.world_mut()
            .resource_mut::<EraProgress>()
            .start(config.time.era_ticks);
        app.world_mut()
            .resource_mut::<NextState<EraState>>()
            .set(EraState::Advancing);

        for _ in 0..config.time.era_ticks + 10 {
            app.update();
        }

        let sim_world = app.world().resource::<SimWorld>();
        assert_eq!(sim_world.tick, config.time.era_ticks as u64);
        assert_eq!(sim_world.era, 1);
        assert_eq!(
            *app.world().resource::<State<EraState>>().get(),
            EraState::Observing
        );
        assert_eq!(
            app.world().resource::<ActionBudget>().points_remaining,
            config.time.point_budget_per_era,
            "the budget should refill when the era ends"
        );

        for _ in 0..10 {
            app.update();
        }
        let sim_world = app.world().resource::<SimWorld>();
        assert_eq!(
            sim_world.tick, config.time.era_ticks as u64,
            "no extra ticks should run once the era has ended"
        );
    }

    /// Two adjacent photolithic organisms (species 0 at `(cx, cy)`, species 1
    /// at `(cx + 1, cy)`), sharing `light`/`temperature` so the photolithic
    /// gain term is identical to `world_with_one_organism`'s, and overriding
    /// `world.matrix` with a hand-built one so the adjacency effect (task
    /// 012) is exactly known.
    fn world_with_two_neighbours(
        matrix: TagMatrix,
        tags_a: Vec<TagSlot>,
        tags_b: Vec<TagSlot>,
        light: f32,
        temperature: f32,
        energy_a: f32,
        energy_b: f32,
    ) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.matrix = matrix;
        for tags in [tags_a, tags_b] {
            world.push_species(Species {
                name: "Test".to_string(),
                metabolism: Metabolism::Photolithic,
                temp_optimum: temperature,
                temp_tolerance: config.energy.default_temp_tolerance,
                repro_threshold: config.energy.repro_threshold,
                tags,
            });
        }
        let (cx, cy) = (world.width / 2, world.height / 2);
        for (dx, species, energy) in [(0, SpeciesId(0), energy_a), (1, SpeciesId(1), energy_b)] {
            let idx = world.index(cx + dx, cy);
            world.cells[idx] = Cell {
                light,
                temperature,
                organism: Some(Organism {
                    species,
                    energy,
                    born_era: 0,
                }),
                ..world.cells[idx]
            };
        }
        (world, config)
    }

    #[test]
    fn negative_adjacency_effect_subtracts_energy() {
        // 2-tag matrix: tag 0 (species A) harms tag 1 (species B) by -2;
        // the reverse direction stays 0.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };
        let (mut world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        // gain 1.4, upkeep 0.5, 1 occupied neighbour -> crowding 0.15,
        // interaction_delta -2: net 5.0 + 1.4 - 0.5 - 0.15 - 2.0 = 3.75.
        assert!((b.energy - 3.75).abs() < TOLERANCE, "got {}", b.energy);
    }

    #[test]
    fn positive_adjacency_effect_adds_energy() {
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, 2, 0, 0],
        };
        let (mut world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        // net 5.0 + 1.4 - 0.5 - 0.15 + 2.0 = 7.75.
        assert!((b.energy - 7.75).abs() < TOLERANCE, "got {}", b.energy);
    }

    #[test]
    fn no_matching_tags_yields_zero_adjacency_effect() {
        // Non-zero matrix entries exist, but B carries no tags at all, so
        // the (their tag x my tag) double loop contributes nothing.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, -2, 0],
        };
        let (mut world, config) =
            world_with_two_neighbours(matrix, vec![TagSlot(0)], Vec::new(), 0.7, 0.5, 5.0, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        // net 5.0 + 1.4 - 0.5 - 0.15 + 0.0 = 5.75.
        assert!((b.energy - 5.75).abs() < TOLERANCE, "got {}", b.energy);
    }

    #[test]
    fn unconditional_tags_match_pre_096_formula() {
        // conditional_tag_count = 0: no conditional tags at all, so the
        // adjacency effect must match the pre-096 formula exactly, the same
        // hand-crafted matrix/pair as `negative_adjacency_effect_subtracts_energy`.
        let mut config = SimConfig::default();
        config.tags.conditional_tag_count = 0;
        let mut world = SimWorld::new(42, &config);
        world.matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };
        assert!(
            world.conditional_tags.is_empty(),
            "conditional_tag_count = 0 should roll no conditional tags"
        );
        for tags in [vec![TagSlot(0)], vec![TagSlot(1)]] {
            world.push_species(Species {
                name: "Test".to_string(),
                metabolism: Metabolism::Photolithic,
                temp_optimum: 0.5,
                temp_tolerance: config.energy.default_temp_tolerance,
                repro_threshold: config.energy.repro_threshold,
                tags,
            });
        }
        let (cx, cy) = (world.width / 2, world.height / 2);
        for (dx, species, energy) in [(0, SpeciesId(0), 5.0), (1, SpeciesId(1), 5.0)] {
            let idx = world.index(cx + dx, cy);
            world.cells[idx] = Cell {
                light: 0.7,
                temperature: 0.5,
                organism: Some(Organism {
                    species,
                    energy,
                    born_era: 0,
                }),
                ..world.cells[idx]
            };
        }
        step(&mut world, &config);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        // Same result as negative_adjacency_effect_subtracts_energy: 3.75.
        assert!((b.energy - 3.75).abs() < TOLERANCE, "got {}", b.energy);
    }

    #[test]
    fn inducible_conditional_tag_gates_on_trigger_terrain() {
        let conditional = ConditionalTag {
            tag: TagId(0),
            terrain: TerrainKind::Hill,
            mode: Mode::Inducible,
        };
        let matrix = || TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };

        // A (the exerter, carrying the conditional tag) sits on the trigger
        // terrain: gate satisfied, full -2 interaction_delta applies to B.
        let (mut world, config) = world_with_two_neighbours(
            matrix(),
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.active_tags = vec![TagId(0), TagId(1)];
        world.conditional_tags = vec![conditional];
        let (cx, cy) = (world.width / 2, world.height / 2);
        let a_idx = world.index(cx, cy);
        world.cells[a_idx].terrain = TerrainKind::Hill;
        step(&mut world, &config);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        assert!(
            (b.energy - 3.75).abs() < TOLERANCE,
            "gate should be satisfied on trigger terrain, got {}",
            b.energy
        );

        // Same setup, A off the trigger terrain: gate fails, no effect.
        let (mut world, config) = world_with_two_neighbours(
            matrix(),
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.active_tags = vec![TagId(0), TagId(1)];
        world.conditional_tags = vec![conditional];
        let (cx, cy) = (world.width / 2, world.height / 2);
        let a_idx = world.index(cx, cy);
        world.cells[a_idx].terrain = TerrainKind::Plain;
        step(&mut world, &config);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        assert!(
            (b.energy - 5.75).abs() < TOLERANCE,
            "gate should fail off the trigger terrain, got {}",
            b.energy
        );
    }

    #[test]
    fn repressible_conditional_tag_gates_off_trigger_terrain() {
        let conditional = ConditionalTag {
            tag: TagId(0),
            terrain: TerrainKind::Hill,
            mode: Mode::Repressible,
        };
        let matrix = || TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };

        // A on the trigger terrain: repressible tag is switched off there,
        // gate fails, no effect.
        let (mut world, config) = world_with_two_neighbours(
            matrix(),
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.active_tags = vec![TagId(0), TagId(1)];
        world.conditional_tags = vec![conditional];
        let (cx, cy) = (world.width / 2, world.height / 2);
        let a_idx = world.index(cx, cy);
        world.cells[a_idx].terrain = TerrainKind::Hill;
        step(&mut world, &config);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        assert!(
            (b.energy - 5.75).abs() < TOLERANCE,
            "gate should fail on the trigger terrain, got {}",
            b.energy
        );

        // A off the trigger terrain: repressible tag is active everywhere
        // else, gate satisfied, full effect applies.
        let (mut world, config) = world_with_two_neighbours(
            matrix(),
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.active_tags = vec![TagId(0), TagId(1)];
        world.conditional_tags = vec![conditional];
        let (cx, cy) = (world.width / 2, world.height / 2);
        let a_idx = world.index(cx, cy);
        world.cells[a_idx].terrain = TerrainKind::Plain;
        step(&mut world, &config);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        assert!(
            (b.energy - 3.75).abs() < TOLERANCE,
            "gate should be satisfied off the trigger terrain, got {}",
            b.energy
        );
    }

    #[test]
    fn conditional_tag_gate_evaluation_emits_a_terrain_gate_observed_event() {
        // Task 097: `TerrainGateObserved` is the raw evidence
        // `TerrainKnowledge` accumulates for the catalog badge — it must
        // fire from both the pass and fail case, carrying the terrain the
        // gate was actually evaluated on.
        let conditional = ConditionalTag {
            tag: TagId(0),
            terrain: TerrainKind::Hill,
            mode: Mode::Inducible,
        };
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };

        let (mut world, config) = world_with_two_neighbours(
            matrix.clone(),
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.active_tags = vec![TagId(0), TagId(1)];
        world.conditional_tags = vec![conditional];
        let (cx, cy) = (world.width / 2, world.height / 2);
        let a_idx = world.index(cx, cy);
        world.cells[a_idx].terrain = TerrainKind::Hill;
        let events = step(&mut world, &config);
        assert!(
            events
                .terrain_gates
                .iter()
                .any(|e| e.tag == TagSlot(0) && e.terrain == TerrainKind::Hill && e.passed),
            "gate evaluated on the trigger terrain should be observed as passed: {:?}",
            events.terrain_gates
        );

        let (mut world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.active_tags = vec![TagId(0), TagId(1)];
        world.conditional_tags = vec![conditional];
        let (cx, cy) = (world.width / 2, world.height / 2);
        let a_idx = world.index(cx, cy);
        world.cells[a_idx].terrain = TerrainKind::Plain;
        let events = step(&mut world, &config);
        assert!(
            events
                .terrain_gates
                .iter()
                .any(|e| e.tag == TagSlot(0) && e.terrain == TerrainKind::Plain && !e.passed),
            "gate evaluated off the trigger terrain should be observed as failed: {:?}",
            events.terrain_gates
        );
    }

    #[test]
    fn unconditional_tag_never_emits_a_terrain_gate_observed_event() {
        // Task 097's `TerrainKnowledge` must never accumulate evidence for
        // an unconditional tag — verified here at the emission site itself,
        // not just at the accumulation system.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };
        let (mut world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.active_tags = vec![TagId(0), TagId(1)];
        world.conditional_tags = Vec::new();
        let events = step(&mut world, &config);
        assert!(
            events.terrain_gates.is_empty(),
            "no conditional tags in this world, so no terrain-gate evidence should be emitted: {:?}",
            events.terrain_gates
        );
    }

    #[test]
    fn conditional_tag_not_in_active_set_does_not_panic_or_affect_delta() {
        // TagId(0) is conditional in this world's roll, but the two active
        // slots resolve to TagId(5)/TagId(6) — neither is TagId(0), so the
        // gate lookup must find no match and the pair must behave exactly
        // like the unconditional case, regardless of terrain.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };
        let (mut world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.active_tags = vec![TagId(5), TagId(6)];
        world.conditional_tags = vec![ConditionalTag {
            tag: TagId(0),
            terrain: TerrainKind::Hill,
            mode: Mode::Inducible,
        }];
        step(&mut world, &config);
        let (cx, cy) = (world.width / 2, world.height / 2);
        let b = world.get(cx + 1, cy).organism.expect("B survives");
        assert!((b.energy - 3.75).abs() < TOLERANCE, "got {}", b.energy);
    }

    /// One organism, carrying a single conditional tag (`TagId(0)`), whose
    /// world-rolled trigger terrain is `Hill` (task 099). `repro_threshold`
    /// is set far out of reach so reproduction never complicates the tick
    /// being tested.
    fn world_with_conditional_species(terrain: TerrainKind, energy: f32) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        for cell in world.cells.iter_mut() {
            cell.terrain = TerrainKind::Plain;
            cell.is_peak = false;
        }
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: 1000.0,
            tags: vec![TagSlot(0)],
        });
        world.active_tags = vec![TagId(0)];
        world.conditional_tags = vec![ConditionalTag {
            tag: TagId(0),
            terrain: TerrainKind::Hill,
            mode: Mode::Inducible,
        }];
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        world.cells[idx] = Cell {
            light: 0.7,
            temperature: 0.5,
            terrain,
            organism: Some(Organism {
                species: SpeciesId(0),
                energy,
                born_era: 0,
            }),
            ..world.cells[idx]
        };
        (world, config)
    }

    #[test]
    fn zone_entry_reveal_fires_once_on_first_trigger_terrain_entry() {
        let (mut world, config) = world_with_conditional_species(TerrainKind::Hill, 5.0);

        let events = step(&mut world, &config);
        assert_eq!(events.reveals.len(), 1);
        assert_eq!(events.reveals[0].species, SpeciesId(0));
        assert_eq!(events.reveals[0].tag, TagId(0));
        assert_eq!(events.reveals[0].terrain, TerrainKind::Hill);
        assert!(world.has_occupied_terrain(SpeciesId(0), TerrainKind::Hill));

        let events = step(&mut world, &config);
        assert!(
            events.reveals.is_empty(),
            "staying on the same terrain must not re-fire the reveal"
        );
    }

    #[test]
    fn zone_entry_no_reveal_when_terrain_is_not_the_trigger() {
        let (mut world, config) = world_with_conditional_species(TerrainKind::Plain, 5.0);

        let events = step(&mut world, &config);
        assert!(
            events.reveals.is_empty(),
            "Plain isn't this tag's trigger terrain (Hill), so no reveal should fire"
        );
    }

    #[test]
    fn zone_entry_no_reveal_for_species_without_conditional_tags() {
        let (mut world, config) = world_with_conditional_species(TerrainKind::Hill, 5.0);
        world.conditional_tags = Vec::new();

        let events = step(&mut world, &config);
        assert!(
            events.reveals.is_empty(),
            "a species with no conditional tags in this world must never fire a reveal"
        );
    }

    #[test]
    fn adjacency_effect_sums_over_multiple_neighbours() {
        // Two A neighbours (west and east of B), each carrying tag 0, each
        // contributing -1 to B (tag 1): total interaction_delta -2, and
        // crowding now counts 2 occupied neighbours.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -1, 0, 0],
        };
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.matrix = matrix;
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagSlot(0)],
        });
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagSlot(1)],
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        for (dx, species, energy) in [
            (-1i32, SpeciesId(0), 5.0),
            (0, SpeciesId(1), 5.0),
            (1, SpeciesId(0), 5.0),
        ] {
            let idx = world.index((cx as i32 + dx) as usize, cy);
            world.cells[idx] = Cell {
                light: 0.7,
                temperature: 0.5,
                organism: Some(Organism {
                    species,
                    energy,
                    born_era: 0,
                }),
                ..world.cells[idx]
            };
        }

        step(&mut world, &config);

        let b = world.get(cx, cy).organism.expect("B survives");
        // net 5.0 + 1.4 - 0.5 - 0.15*2 + (-1 - 1) = 3.6.
        assert!((b.energy - 3.6).abs() < TOLERANCE, "got {}", b.energy);
    }

    fn world_with_one_predator(temperature: f32, energy: f32) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Predator,
            temp_optimum: temperature,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        world.cells[idx] = Cell {
            light: 0.0,
            temperature,
            organism: Some(Organism {
                species: SpeciesId(0),
                energy,
                born_era: 0,
            }),
            ..world.cells[idx]
        };
        (world, config)
    }

    #[test]
    fn isolated_predator_collapses_with_no_gain() {
        // GDD §5.9: seed_energy 5.0, predator_upkeep 0.7, no prey ⇒ dies
        // after ceil(5.0 / 0.7) = 8 ticks, with zero gain every tick.
        let (mut world, config) = world_with_one_predator(0.5, config_seed_energy());
        let (cx, cy) = (world.width / 2, world.height / 2);

        for tick in 1..=8 {
            step(&mut world, &config);
            let alive = world.get(cx, cy).organism.is_some();
            if tick < 8 {
                assert!(alive, "predator should still be alive at tick {tick}");
            } else {
                assert!(!alive, "predator should have died by tick {tick}");
            }
        }
    }

    fn config_seed_energy() -> f32 {
        SimConfig::default().energy.seed_energy
    }

    #[test]
    fn predator_with_abundant_prey_nets_positive_energy() {
        let (mut world, config) = world_with_one_predator(0.5, 5.0);
        let (cx, cy) = (world.width / 2, world.height / 2);

        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let neighbours: Vec<usize> = world.moore_neighbours(cx, cy).collect();
        for &idx in &neighbours {
            world.cells[idx].organism = Some(Organism {
                species: SpeciesId(1),
                energy: 20.0,
                born_era: 0,
            });
        }

        step(&mut world, &config);

        let predator = world.get(cx, cy).organism.expect("predator survives");
        // drain = min(predator_drain_cap, available) * fit = 2.0 (fit=1.0,
        // available huge), upkeep 0.7, 8 occupied neighbours -> crowding
        // 0.15*8=1.2: net 5.0 + 2.0 - 0.7 - 1.2 = 5.1.
        assert!(
            (predator.energy - 5.1).abs() < TOLERANCE,
            "got {}",
            predator.energy
        );
    }

    #[test]
    fn two_predators_sharing_one_prey_split_deterministically() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Predator,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        // Prey at (cx, cy), predators at (cx - 1, cy) and (cx + 1, cy), both
        // Moore-adjacent to the prey and sharing it as their only neighbour.
        let (cx, cy) = (world.width / 2, world.height / 2);
        let prey_idx = world.index(cx, cy);
        world.cells[prey_idx] = Cell {
            light: 0.0,
            temperature: 0.5,
            organism: Some(Organism {
                species: SpeciesId(1),
                energy: 20.0,
                born_era: 0,
            }),
            ..world.cells[prey_idx]
        };
        for dx in [-1i32, 1] {
            let idx = world.index((cx as i32 + dx) as usize, cy);
            world.cells[idx] = Cell {
                light: 0.0,
                temperature: 0.5,
                organism: Some(Organism {
                    species: SpeciesId(0),
                    energy: 5.0,
                    born_era: 0,
                }),
                ..world.cells[idx]
            };
        }

        step(&mut world, &config);

        let left = world
            .get(cx - 1, cy)
            .organism
            .expect("left predator survives");
        let right = world
            .get(cx + 1, cy)
            .organism
            .expect("right predator survives");
        assert!(
            (left.energy - right.energy).abs() < TOLERANCE,
            "left {} vs right {}",
            left.energy,
            right.energy
        );
        // Each predator's only prey neighbour is the shared one, so each
        // draws its full drain_cap (fit=1.0, prey has plenty of energy):
        // net 5.0 + 2.0 - 0.7 - 0.15 = 6.15.
        assert!(
            (left.energy - 6.15).abs() < TOLERANCE,
            "got {}",
            left.energy
        );
    }

    fn world_with_one_decomposer(temperature: f32, energy: f32) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Decomposer,
            temp_optimum: temperature,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        world.cells[idx] = Cell {
            light: 0.0,
            temperature,
            organism: Some(Organism {
                species: SpeciesId(0),
                energy,
                born_era: 0,
            }),
            ..world.cells[idx]
        };
        (world, config)
    }

    #[test]
    fn decomposer_with_no_residue_behaves_like_dark_photolithic() {
        // No residue anywhere ⇒ gain 0, loses decomposer_upkeep (0.5) per
        // tick, same shape as photolithic_in_the_dark_eventually_dies. This
        // is the true no-trickle baseline, so the ambient trickle (task 060)
        // is explicitly disabled here rather than folded into the default.
        let (mut world, mut config) = world_with_one_decomposer(0.5, config_seed_energy());
        config.energy.residue_ambient_trickle = 0.0;
        let (cx, cy) = (world.width / 2, world.height / 2);

        step(&mut world, &config);
        let organism = world.get(cx, cy).organism.expect("survives the first tick");
        assert!(
            (organism.energy - (config_seed_energy() - config.energy.decomposer_upkeep)).abs()
                < TOLERANCE,
            "expected net -upkeep/tick with no residue, got {}",
            organism.energy - config_seed_energy()
        );

        for _ in 0..200 {
            if world.get(cx, cy).organism.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).organism.is_none(),
            "decomposer with no residue in range should not survive long-term"
        );
    }

    #[test]
    fn decomposer_survives_much_longer_with_ambient_trickle() {
        // No-trickle baseline (decomposer_with_no_residue_behaves_like_dark_photolithic)
        // collapses at tick 10 (5.0 seed energy / 0.5 upkeep per tick). With
        // the default trickle, the isolated decomposer's own cell equilibrates
        // to a small residue supply and survives to tick 100 — well past the
        // GDD §5.9 ~7-tick "obviously starving" baseline, without becoming
        // self-sufficient (it still dies eventually, unlike Photolithic).
        let (mut world, mut config) = world_with_one_decomposer(0.5, config_seed_energy());
        let (cx, cy) = (world.width / 2, world.height / 2);
        // 150 ticks is long enough for task 085's environment (diffusion,
        // sea coastal cooling) to drift this cell's temperature away from
        // the `temp_optimum` match `world_with_one_decomposer` set up,
        // confounding the trickle-survival measurement this test isolates —
        // freeze both so the cell's temperature stays exactly where the
        // helper put it, same isolation principle
        // `decomposer_adjacent_to_residue_gains_and_residue_shrinks` already
        // applies to `residue_ambient_trickle`. Heat-source reinjection is
        // left as-is: with only 2-4 sources across 10240 cells it's not
        // going to coincide with this fixed test cell.
        config.environment.diffusion_rate = 0.0;
        config.source.sea_coolant_strength = 0.0;

        let mut ticks_survived = 0;
        for _ in 0..150 {
            step(&mut world, &config);
            if world.get(cx, cy).organism.is_none() {
                break;
            }
            ticks_survived += 1;
        }
        assert!(
            ticks_survived >= 50,
            "expected ambient trickle to meaningfully extend survival past the \
             no-trickle 10-tick collapse, got {} ticks",
            ticks_survived
        );
    }

    #[test]
    fn decomposer_adjacent_to_residue_gains_and_residue_shrinks() {
        // Exact residue-depletion math below assumes no ambient trickle.
        let (mut world, mut config) = world_with_one_decomposer(0.5, 5.0);
        config.energy.residue_ambient_trickle = 0.0;
        let (cx, cy) = (world.width / 2, world.height / 2);
        let neighbour_idx = world
            .moore_neighbours(cx, cy)
            .next()
            .expect("has a neighbour");
        world.cells[neighbour_idx].residue = 10.0;

        step(&mut world, &config);

        let decomposer = world.get(cx, cy).organism.expect("decomposer survives");
        // decay first: 10.0 - residue_decay(0.2) = 9.8 available; drawn =
        // min(decomposer_extract_rate(1.5) * fit(1.0), 9.8) = 1.5; net
        // 5.0 + 1.5 - decomposer_upkeep(0.5) = 6.0.
        assert!(
            (decomposer.energy - 6.0).abs() < TOLERANCE,
            "got {}",
            decomposer.energy
        );
        let (nx, ny) = (neighbour_idx % world.width, neighbour_idx / world.width);
        assert!(
            (world.get(nx, ny).residue - 8.3).abs() < TOLERANCE,
            "expected residue reduced by the drawn amount, got {}",
            world.get(nx, ny).residue
        );
    }

    fn world_with_one_chemolithotroph(
        temperature: f32,
        toxicity: f32,
        energy: f32,
    ) -> (SimWorld, SimConfig) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Chemolithotroph,
            temp_optimum: temperature,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        world.cells[idx] = Cell {
            light: 0.0,
            temperature,
            toxicity,
            organism: Some(Organism {
                species: SpeciesId(0),
                energy,
                born_era: 0,
            }),
            ..world.cells[idx]
        };
        (world, config)
    }

    #[test]
    fn chemolithotroph_with_no_toxicity_behaves_like_dark_photolithic() {
        // Mirrors `decomposer_with_no_residue_behaves_like_dark_photolithic`:
        // zero gain source ⇒ loses `chemolithotroph_upkeep` per tick, same
        // shape as an isolated photolithic organism in the dark.
        let (mut world, config) = world_with_one_chemolithotroph(0.5, 0.0, config_seed_energy());
        let (cx, cy) = (world.width / 2, world.height / 2);

        step(&mut world, &config);
        let organism = world.get(cx, cy).organism.expect("survives the first tick");
        assert!(
            (organism.energy - (config_seed_energy() - config.energy.chemolithotroph_upkeep)).abs()
                < TOLERANCE,
            "expected net -upkeep/tick with no toxicity, got {}",
            organism.energy - config_seed_energy()
        );

        for _ in 0..200 {
            if world.get(cx, cy).organism.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).organism.is_none(),
            "chemolithotroph with no toxicity should not survive long-term"
        );
    }

    #[test]
    fn chemolithotroph_in_a_toxic_cell_nets_positive_energy() {
        let (mut world, mut config) =
            world_with_one_chemolithotroph(0.5, 0.7, config_seed_energy());
        // Isolate from environmental drift, mirroring the decomposer
        // trickle-survival tests' rationale — the exact-math assertion
        // below assumes this cell's temperature/toxicity stay put.
        config.environment.diffusion_rate = 0.0;
        let (cx, cy) = (world.width / 2, world.height / 2);

        step(&mut world, &config);

        let organism = world.get(cx, cy).organism.expect("survives");
        // fit(temp_optimum == temperature) == 1.0: gain = 0.7 *
        // chemolithotroph_metabolism_gain(2.0) * 1.0 = 1.4; net =
        // 5.0 + 1.4 - chemolithotroph_upkeep(0.5) = 5.9.
        let expected = config_seed_energy() + 0.7 * config.energy.chemolithotroph_metabolism_gain
            - config.energy.chemolithotroph_upkeep;
        assert!(
            (organism.energy - expected).abs() < TOLERANCE,
            "expected net positive gain in a toxic cell, got {} (expected {})",
            organism.energy,
            expected
        );
        assert!(
            organism.energy > config_seed_energy(),
            "a chemolithotroph in toxicity should gain energy, not lose it"
        );
    }

    #[test]
    fn chemolithotroph_gain_scales_with_env_fit() {
        // Same toxicity, two different temperature mismatches: the
        // well-matched organism must net more energy than the poorly
        // matched one, confirming `fit` still gates chemolithotroph gain
        // exactly like it does for the other three metabolisms.
        let (mut matched_world, config) =
            world_with_one_chemolithotroph(0.5, 0.7, config_seed_energy());
        let (mut mismatched_world, _) =
            world_with_one_chemolithotroph(0.5, 0.7, config_seed_energy());
        mismatched_world.species[0].temp_optimum = 0.9;

        step(&mut matched_world, &config);
        step(&mut mismatched_world, &config);

        let (cx, cy) = (matched_world.width / 2, matched_world.height / 2);
        let matched_energy = matched_world.get(cx, cy).organism.expect("survives").energy;
        let mismatched_energy = mismatched_world
            .get(cx, cy)
            .organism
            .expect("survives")
            .energy;
        assert!(
            matched_energy > mismatched_energy,
            "a well-matched chemolithotroph ({matched_energy}) should out-gain a \
             poorly-matched one ({mismatched_energy})"
        );
    }

    #[test]
    fn residue_never_goes_negative_under_competing_decomposers() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Decomposer,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        // Residue cell at (cx, cy); two decomposers straddle it on the same
        // row so each is Moore-adjacent only to the residue cell, not to
        // each other, and each independently computes the full residue as
        // available — their combined draw exceeds what's actually there.
        let (cx, cy) = (world.width / 2, world.height / 2);
        let residue_idx = world.index(cx, cy);
        world.cells[residue_idx].residue = 2.0;
        for dx in [-1i32, 1] {
            let idx = world.index((cx as i32 + dx) as usize, cy);
            world.cells[idx] = Cell {
                light: 0.0,
                temperature: 0.5,
                organism: Some(Organism {
                    species: SpeciesId(0),
                    energy: 5.0,
                    born_era: 0,
                }),
                ..world.cells[idx]
            };
        }

        step(&mut world, &config);

        assert!(
            world.get(cx, cy).residue >= 0.0,
            "residue must never go negative, got {}",
            world.get(cx, cy).residue
        );
        assert!(
            world.get(cx, cy).residue.abs() < TOLERANCE,
            "expected residue fully depleted (over-drawn to 0), got {}",
            world.get(cx, cy).residue
        );
    }

    #[test]
    fn phase_0_single_species_worlds_still_have_zero_interaction_delta() {
        // Regression guard: existing single-species tests build worlds with
        // no tags at all, so the matrix wiring must not change their result.
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let organism = world.get(cx, cy).organism.expect("organism survives");
        assert!(
            (organism.energy - 5.9).abs() < TOLERANCE,
            "got {}",
            organism.energy
        );
    }

    #[test]
    fn death_produces_exactly_one_organism_died_event() {
        // Two same-species organisms, far enough apart not to interact: one
        // starved in the dark with near-zero energy (dies this tick), one
        // healthy (survives) — so exactly one death should be recorded, and
        // the species survives (population 2 -> 1), so no extinction.
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.push_species(Species {
            name: "Test".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let dying_idx = world.index(1, 1);
        world.cells[dying_idx] = Cell {
            light: 0.0,
            temperature: 0.5,
            organism: Some(Organism {
                species: SpeciesId(0),
                energy: 0.05,
                born_era: 0,
            }),
            ..world.cells[dying_idx]
        };
        let (sx, sy) = (world.width - 2, world.height - 2);
        let surviving_idx = world.index(sx, sy);
        world.cells[surviving_idx] = Cell {
            light: 0.7,
            temperature: 0.5,
            organism: Some(Organism {
                species: SpeciesId(0),
                energy: 5.0,
                born_era: 0,
            }),
            ..world.cells[surviving_idx]
        };

        let events = step(&mut world, &config);

        assert_eq!(events.deaths.len(), 1);
        assert_eq!(events.deaths[0].cell, dying_idx);
        assert_eq!(events.deaths[0].species, SpeciesId(0));
        // Isolated, dark cell: no light means no gain, no neighbours means
        // no matrix effect, no crowding, and no predation — the only cost
        // is the flat photolithic upkeep.
        assert_eq!(events.deaths[0].energy_before, 0.05);
        assert_eq!(events.deaths[0].gain, 0.0);
        assert_eq!(events.deaths[0].interaction_delta, 0.0);
        assert_eq!(events.deaths[0].upkeep, config.energy.base_upkeep);
        assert_eq!(events.deaths[0].crowding_penalty, 0.0);
        assert_eq!(events.deaths[0].predation_loss, 0.0);
        assert!(
            events.extinctions.is_empty(),
            "species still has a survivor, should not be extinct"
        );
        assert!(world.get(1, 1).organism.is_none());
    }

    #[test]
    fn death_event_carries_an_internally_consistent_energy_breakdown() {
        // Regression guard for "why did it die" confusion: whatever the
        // fixture, the reported terms must actually explain the death —
        // applying them to `energy_before` must reproduce a value at or
        // below zero.
        let (mut world, config) = world_with_one_organism(0.0, 0.5, 0.05);

        let events = step(&mut world, &config);

        assert_eq!(events.deaths.len(), 1);
        let death = events.deaths[0];
        let reconstructed = death.energy_before + death.gain + death.interaction_delta
            - death.upkeep
            - death.crowding_penalty
            - death.predation_loss;
        assert!(
            reconstructed <= 0.0,
            "breakdown terms must explain the death: reconstructed energy was {reconstructed}"
        );
    }

    #[test]
    fn last_organism_of_species_dying_emits_extinction() {
        // A single organism, alone in the world: its death (energy <= 0)
        // must also drop its species' population to 0.
        let (mut world, config) = world_with_one_organism(0.0, 0.5, 0.05);

        let events = step(&mut world, &config);

        assert_eq!(events.deaths.len(), 1);
        assert_eq!(events.extinctions.len(), 1);
        assert_eq!(events.extinctions[0].species, SpeciesId(0));
    }

    #[test]
    fn adjacency_between_tagged_organisms_produces_expected_observation() {
        // Matrix entry (exerter tag 0 -> receiver tag 1) is non-zero, the
        // reverse direction is zero: A -> B is observed, B -> A is not.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };
        let (mut world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );

        let events = step(&mut world, &config);

        assert_eq!(events.adjacencies.len(), 1);
        let obs = events.adjacencies[0];
        assert_eq!(obs.receiver_species, SpeciesId(1));
        assert_eq!(obs.exerter_tag, TagSlot(0));
        assert_eq!(obs.receiver_tag, TagSlot(1));
        assert_eq!(
            obs.n_confounders, 0,
            "B has exactly one neighbour, carrying only the exerter tag"
        );
    }

    #[test]
    fn confounder_count_matches_gdd_examples() {
        // Receiver R (tag 1) at the center, with four occupied Moore
        // neighbours: one carrying the exerter tag 0, three others carrying
        // three other distinct tags (2, 3, 4) that don't participate in any
        // non-zero matrix entry themselves — they're pure confounders. Only
        // matrix entry (tag 0 -> tag 1) is non-zero, so exactly one
        // AdjacencyObserved is produced, with n_confounders = 3 (GDD §7:
        // three confounding tags -> weight 0.25).
        let size = 5;
        let mut values = vec![0i8; size * size];
        values[1] = 3; // exerter tag 0 -> receiver tag 1
        let matrix = TagMatrix { size, values };

        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.matrix = matrix;
        for tags in [
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            vec![TagSlot(2)],
            vec![TagSlot(3)],
            vec![TagSlot(4)],
        ] {
            world.push_species(Species {
                name: "Test".to_string(),
                metabolism: Metabolism::Photolithic,
                temp_optimum: 0.5,
                temp_tolerance: config.energy.default_temp_tolerance,
                repro_threshold: config.energy.repro_threshold,
                tags,
            });
        }
        let (cx, cy) = (world.width / 2, world.height / 2);
        // Receiver at center (species 1, tag 1); four neighbours at the
        // cardinal Moore offsets, each a distinct species/tag.
        let placements = [
            (0i32, 0i32, SpeciesId(1)),
            (-1, 0, SpeciesId(0)),
            (1, 0, SpeciesId(2)),
            (0, -1, SpeciesId(3)),
            (0, 1, SpeciesId(4)),
        ];
        for (dx, dy, species) in placements {
            let idx = world.index((cx as i32 + dx) as usize, (cy as i32 + dy) as usize);
            world.cells[idx] = Cell {
                light: 0.7,
                temperature: 0.5,
                organism: Some(Organism {
                    species,
                    energy: 5.0,
                    born_era: 0,
                }),
                ..world.cells[idx]
            };
        }

        let events = step(&mut world, &config);

        let observed: Vec<_> = events
            .adjacencies
            .iter()
            .filter(|obs| obs.receiver_species == SpeciesId(1))
            .collect();
        assert_eq!(
            observed.len(),
            1,
            "only the (tag 0 -> tag 1) entry is non-zero"
        );
        assert_eq!(observed[0].exerter_tag, TagSlot(0));
        assert_eq!(observed[0].receiver_tag, TagSlot(1));
        assert_eq!(
            observed[0].n_confounders, 3,
            "three other distinct tags among the receiver's neighbours"
        );
        let weight = 1.0 / (1.0 + observed[0].n_confounders as f32);
        assert!((weight - 0.25).abs() < TOLERANCE);
    }

    #[test]
    fn action_budget_spends_when_affordable_and_refuses_when_not() {
        let mut budget = ActionBudget {
            points_remaining: 1,
        };
        assert!(budget.try_spend(1));
        assert_eq!(budget.points_remaining, 0);
        assert!(
            !budget.try_spend(1),
            "insufficient points should refuse the spend"
        );
        assert_eq!(
            budget.points_remaining, 0,
            "a refused spend must not touch the balance"
        );
    }

    #[test]
    fn action_budget_refill_resets_to_the_given_amount() {
        let mut budget = ActionBudget {
            points_remaining: 0,
        };
        budget.refill(3);
        assert_eq!(budget.points_remaining, 3);
    }

    fn test_evolution_config(threshold: f32) -> EvolutionConfig {
        EvolutionConfig {
            selection_pressure_threshold: threshold,
            interaction_harm_weight: 1.0,
            terrain_mismatch_weight: 1.0,
            toxicity_weight: 1.0,
            max_species: 40,
        }
    }

    #[test]
    fn selection_pressure_accumulates_from_each_stimulus_independently() {
        let evolution = test_evolution_config(1000.0);
        let mut pressures = Vec::new();

        // Negative interaction_delta contributes to interaction_harm only.
        accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            0,
            -3.0,
            1.0,
            TerrainKind::Plain,
            0.0,
        );
        assert_eq!(pressures[0].interaction_harm, 3.0);
        assert_eq!(pressures[0].terrain_mismatch, [0.0; 4]);
        assert_eq!(pressures[0].toxicity, 0.0);

        // Positive interaction_delta is a benefit, not harm — must not add.
        accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            0,
            5.0,
            1.0,
            TerrainKind::Plain,
            0.0,
        );
        assert_eq!(
            pressures[0].interaction_harm, 3.0,
            "positive interaction_delta must not add harm"
        );

        // Poor fit (0.4) while on Hill contributes to terrain_mismatch[Hill] only.
        accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            0,
            0.0,
            0.4,
            TerrainKind::Hill,
            0.0,
        );
        assert!((pressures[0].terrain_mismatch[TerrainKind::Hill.index()] - 0.6).abs() < TOLERANCE);
        assert_eq!(
            pressures[0].terrain_mismatch[TerrainKind::Plain.index()],
            0.0
        );

        // Toxicity exposure contributes to toxicity only.
        accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            0,
            0.0,
            1.0,
            TerrainKind::Plain,
            0.7,
        );
        assert!((pressures[0].toxicity - 0.7).abs() < TOLERANCE);
    }

    #[test]
    fn selection_threshold_crossed_fires_exactly_once_at_crossing() {
        let evolution = test_evolution_config(10.0);
        let mut pressures = Vec::new();

        // 9 units of harm: below threshold, no event yet.
        let below = accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            42,
            -9.0,
            1.0,
            TerrainKind::Plain,
            0.0,
        );
        assert!(below.is_none());

        // +2 more units of harm crosses 10.0: the event fires on this call.
        let crossed = accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            42,
            -2.0,
            1.0,
            TerrainKind::Plain,
            0.0,
        )
        .expect("threshold crossed this call");
        assert_eq!(crossed.species, SpeciesId(0));
        assert_eq!(crossed.cell, 42);
        assert!((crossed.interaction_harm - 11.0).abs() < TOLERANCE);
    }

    #[test]
    fn selection_pressure_does_not_refire_after_crossing() {
        let evolution = test_evolution_config(5.0);
        let mut pressures = Vec::new();

        let first = accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            0,
            -6.0,
            1.0,
            TerrainKind::Plain,
            0.0,
        );
        assert!(first.is_some(), "first call crosses the threshold");

        let second = accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            0,
            -6.0,
            1.0,
            TerrainKind::Plain,
            0.0,
        );
        assert!(second.is_none(), "must not re-fire once already crossed");
        assert!(pressures[0].crossed);
    }

    #[test]
    fn selection_threshold_crossed_reports_the_dominant_mismatch_terrain() {
        let evolution = test_evolution_config(1.0);
        let mut pressures = Vec::new();

        // Small mismatch on Plain (0.1), larger on Mountain (0.8) — total
        // 0.9, still below threshold.
        accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            0,
            0.0,
            0.9,
            TerrainKind::Plain,
            0.0,
        );
        accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            0,
            0.0,
            0.2,
            TerrainKind::Mountain,
            0.0,
        );
        // Toxicity 0.2 tips the total to 1.1, crossing the threshold.
        let crossed = accumulate_selection_pressure(
            &mut pressures,
            &evolution,
            1,
            SpeciesId(0),
            0,
            0.0,
            1.0,
            TerrainKind::Plain,
            0.2,
        )
        .expect("threshold crossed this call");
        assert_eq!(
            crossed.dominant_terrain,
            TerrainKind::Mountain,
            "Mountain accrued the larger mismatch share"
        );
    }

    /// End-to-end verification (task 106's live-verification acceptance
    /// criterion, exercised here as an automated integration test instead of
    /// a manual `cargo run` session, since this feature has no player-facing
    /// UI yet — presentation is task 107's concern): a lineage sustaining
    /// maximum toxicity exposure through `step` itself, not the pure
    /// accumulator directly, eventually crosses the threshold and the event
    /// surfaces through `TickEvents` exactly like `OrganismDied` does.
    #[test]
    fn sustained_toxicity_exposure_crosses_the_threshold_through_step() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);

        let mut crossed = None;
        for _ in 0..200 {
            // Reset every tick: `diffuse_environment` would otherwise erode
            // this cell's toxicity before the accumulator reads it.
            world.cells[idx].toxicity = 1.0;
            let events = step(&mut world, &config);
            if let Some(event) = events.selection_thresholds.into_iter().next() {
                crossed = Some(event);
                break;
            }
            if world.get(cx, cy).organism.is_none() {
                panic!("organism must not die from toxicity alone in this test setup");
            }
        }

        let crossed = crossed.expect("sustained toxicity exposure should cross the threshold");
        assert_eq!(crossed.species, SpeciesId(0));
        assert!(crossed.toxicity > 0.0);
    }

    /// A world with two active tags, a neutral (all-zero) matrix, and one
    /// species carrying only `TagSlot(0)`, placed at the grid's center —
    /// mirrors `input.rs`'s `world_with_one_taggable_species` fixture, the
    /// precedent for deterministic tag-edit tests independent of a seed's
    /// randomly-generated matrix.
    fn world_for_speciation() -> (SimWorld, SimConfig, usize) {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        world.active_tags = vec![TagId(0), TagId(1)];
        world.matrix = TagMatrix::from_values(2, vec![0, 0, 0, 0]);
        world.push_species(Species {
            name: "Source".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagSlot(0)],
        });
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        world.cells[idx].organism = Some(Organism {
            species: SpeciesId(0),
            energy: 5.0,
            born_era: 0,
        });
        (world, config, idx)
    }

    fn toxicity_dominant_event(species: SpeciesId, cell: usize) -> SelectionThresholdCrossed {
        SelectionThresholdCrossed {
            species,
            cell,
            interaction_harm: 0.0,
            terrain_mismatch: 0.0,
            dominant_terrain: TerrainKind::Plain,
            toxicity: 5.0,
        }
    }

    fn interaction_harm_dominant_event(
        species: SpeciesId,
        cell: usize,
    ) -> SelectionThresholdCrossed {
        SelectionThresholdCrossed {
            species,
            cell,
            interaction_harm: 5.0,
            terrain_mismatch: 0.0,
            dominant_terrain: TerrainKind::Plain,
            toxicity: 0.0,
        }
    }

    #[test]
    fn a_qualifying_crossing_produces_exactly_one_new_species_and_leaves_the_parent_unchanged() {
        let (mut world, config, idx) = world_for_speciation();
        let source_before = world.species[0].clone();
        let event = toxicity_dominant_event(SpeciesId(0), idx);

        let new_id = speciate(&mut world, &config, &event).expect("qualifying crossing");

        assert_eq!(world.species.len(), 2);
        assert_eq!(new_id, SpeciesId(1));
        assert_eq!(
            world.species[0], source_before,
            "the source species must never be mutated in place"
        );
        // Toxicity-dominant grants Sea tolerance, not a tag/temp edit.
        assert!(world.is_sea_tolerant(new_id));
        // Founder placement: the triggering organism's cell now belongs to
        // the descendant (this task's documented "simpler of two options"
        // choice — no new placement logic).
        assert_eq!(world.cells[idx].organism.unwrap().species, new_id);
    }

    #[test]
    fn interaction_harm_dominant_descendant_tags_stay_within_active_tags() {
        let (mut world, config, idx) = world_for_speciation();
        let event = interaction_harm_dominant_event(SpeciesId(0), idx);

        let new_id = speciate(&mut world, &config, &event).expect("qualifying crossing");

        let descendant = &world.species[new_id.0 as usize];
        assert!(descendant.tags.len() > world.species[0].tags.len());
        for &slot in &descendant.tags {
            assert!(
                (slot.0 as usize) < world.active_tags.len(),
                "descendant tag {slot:?} is outside world.active_tags"
            );
        }
    }

    #[test]
    fn a_self_interacting_tag_addition_is_rejected_not_partially_applied() {
        let (mut world, config, idx) = world_for_speciation();
        // tag0 -> tag1 = +1, tag1 -> tag0 = +1: adding tag1 to a species
        // that already carries tag0 makes it net self-reinforcing.
        world.matrix = TagMatrix::from_values(2, vec![0, 1, 1, 0]);
        let event = interaction_harm_dominant_event(SpeciesId(0), idx);

        let result = speciate(&mut world, &config, &event);

        assert!(
            result.is_none(),
            "a self-reinforcing tag addition must no-op, not partially apply"
        );
        assert_eq!(
            world.species.len(),
            1,
            "no descendant should have been appended"
        );
    }

    #[test]
    fn speciation_is_a_no_op_once_max_species_is_reached() {
        let (mut world, mut config, idx) = world_for_speciation();
        config.evolution.max_species = world.species.len();
        let event = toxicity_dominant_event(SpeciesId(0), idx);

        assert!(speciate(&mut world, &config, &event).is_none());
        assert_eq!(world.species.len(), 1);
    }

    #[test]
    fn speciation_no_ops_for_a_species_that_no_longer_exists() {
        let (mut world, config, idx) = world_for_speciation();
        let event = toxicity_dominant_event(SpeciesId(5), idx);

        assert!(speciate(&mut world, &config, &event).is_none());
        assert_eq!(world.species.len(), 1);
    }

    /// End-to-end verification (task 107's live-verification acceptance
    /// criterion, exercised headlessly like task 106's equivalent test):
    /// sustained toxicity exposure through real `step()` calls eventually
    /// crosses the threshold, and feeding that *actual* emitted event into
    /// `speciate` produces a distinct descendant without touching the
    /// parent — the full pure-Rust pipeline `sim::speciate_on_threshold_crossed`
    /// wires together, without needing a running `App`/GUI to confirm it.
    #[test]
    fn a_real_threshold_crossing_from_step_produces_a_descendant_species() {
        let (mut world, mut config) = world_with_one_organism(0.7, 0.5, 5.0);
        config.evolution.selection_pressure_threshold = 5.0;
        let source_name_before = world.species[0].name.clone();
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);

        let mut crossed = None;
        for _ in 0..200 {
            world.cells[idx].toxicity = 1.0;
            let events = step(&mut world, &config);
            if let Some(event) = events.selection_thresholds.into_iter().next() {
                crossed = Some(event);
                break;
            }
        }
        let event = crossed.expect("sustained toxicity exposure should cross the threshold");

        let new_id = speciate(&mut world, &config, &event).expect("qualifying crossing");

        assert_eq!(world.species.len(), 2);
        assert_ne!(
            world.species[new_id.0 as usize].name, source_name_before,
            "the descendant must draw its own independent name"
        );
        assert_eq!(
            world.species[0].name, source_name_before,
            "the source species must be unchanged"
        );
    }
}
