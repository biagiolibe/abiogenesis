// Advances the simulation by one tick (GDD §5.6). Pure Rust, no Bevy App
// required, so determinism and balance can be tested headless.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::RngExt;

use crate::config::{EvolutionConfig, SimConfig};
use crate::state::{EraState, GameState};
use crate::world::{
    draw_species_name, net_self_interaction, AdjacencyExposure, ConditionalTag, Metabolism, Mode,
    Population, SelectionPressure, SimWorld, SpeciesId, TagId, TagSlot, TerrainKind,
    TerrainOccupancy,
};

/// Which gate ended a population, distinct from `text::DominantDeathCause`
/// (task 105), which infers *which energy term* dominated a starvation
/// death from its numeric fields — this instead records *which pipeline
/// phase* fired (task 138). A habitat-gate death has no meaningful energy
/// terms at all (`gain`/`env_fit`/`interaction_delta`/upkeep are all `0.0`),
/// so numeric inference alone would silently misclassify it as starvation;
/// this field is why that can't happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    /// Energy reached zero (GDD §5.6 step 5) — the only cause before task 138.
    Starvation,
    /// Task 138 phase 0: the cell's biome is outright uninhabitable for this
    /// species, independent of any scalar fit.
    Habitat,
}

/// One organism's energy reached zero and it was removed this tick, with
/// the exact terms of the energy update (GDD §5.6 step 5) that led there —
/// consumed by `notebook.rs` to explain a salient death instead of leaving
/// the player to re-derive it from the tick code.
#[derive(Debug, Clone, Copy, Message)]
pub struct OrganismDied {
    pub cell: usize,
    pub species: SpeciesId,
    pub cause: DeathCause,
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
/// Emitted only on **onset** (task 136b) — the tick `exerter_tag` first
/// becomes adjacent to this cell's organism, tracked in
/// `SimWorld::adjacency_exposure`. An adjacency that persists tick after
/// tick emits exactly once, not once per tick it holds: the confirmation
/// engine counts distinct episodes of exposure, not elapsed time, matching
/// GDD §7's isolated-observation model rather than rewarding a population
/// that simply sits still long enough. `interaction_delta`, the *energy*
/// effect of the same adjacency, is unaffected by this and keeps applying
/// every tick regardless — only the evidence is onset-gated.
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
    /// This exact `(exerter_tag, receiver_tag)` pair's energy contribution
    /// this tick (`matrix entry × interaction_scale`, task 138) — the
    /// per-neighbouring-tag breakdown task 149's inspection card needs,
    /// kept alongside the pair rather than only folded into the cell's
    /// summed `interaction_delta`.
    pub contribution: f32,
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
///
/// This is the tick pipeline's phase 7, observation/event emission (task
/// 138): every per-cell observation the tick produces goes through here,
/// exactly once, rather than through a separate path. `Cull`'s knockout
/// observation (task 146) is the one exception: it's a click-driven,
/// one-shot event outside the tick pipeline, generated by
/// `cull_knockout_observations` and written directly by `input.rs`'s
/// `cull_on_click`, not folded into `TickEvents`.
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

/// Distinct tags carried by `(x, y)`'s occupied Moore neighbours (task
/// 136b) — the confounder-count basis in `AdjacencyObserved`'s doc comment,
/// gathered once so per-tag-pair observations don't rescan neighbours per
/// tag. Shared by `step`'s per-tick adjacency scan and Cull's one-shot
/// knockout observation (task 146, `cull_knockout_observations`).
fn distinct_neighbour_tags(world: &SimWorld, x: usize, y: usize) -> Vec<TagSlot> {
    let mut tags: Vec<TagSlot> = Vec::new();
    for neighbour_idx in world.moore_neighbours(x, y) {
        if let Some(neighbour) = world.cells[neighbour_idx].population {
            let neighbour_species = &world.species[neighbour.species.0 as usize];
            for &tag in &neighbour_species.tags {
                if !tags.contains(&tag) {
                    tags.push(tag);
                }
            }
        }
    }
    tags
}

/// Every `(exerter_tag, receiver_tag)` observation between one specific
/// occupied neighbour (`exerter_idx`) and a receiver (`receiver_idx`/
/// `receiver_species`/`receiver_tags`) — the per-neighbour-pair body shared
/// by `step`'s tick loop and Cull's one-shot knockout observation (task
/// 146), so the matrix-lookup/confounder formula exists exactly once.
/// `neighbour_tags` is the receiver's *entire* Moore-neighbourhood
/// confounder basis (`distinct_neighbour_tags`), not just `exerter_idx` —
/// see `AdjacencyObserved`'s doc comment. No onset gating here: `step`
/// filters the returned candidates by `onset_mask` itself; Cull takes every
/// candidate (`entry != 0`) as-is, since a knockout is a one-shot event, not
/// a persisting tick. Returns the summed contribution (folded into
/// `interaction_delta` by `step`) alongside the raw candidates.
#[allow(clippy::too_many_arguments)]
fn adjacency_pair_observations(
    world: &SimWorld,
    energy: &crate::config::EnergyConfig,
    receiver_idx: usize,
    receiver_species: SpeciesId,
    receiver_tags: &[TagSlot],
    exerter_idx: usize,
    neighbour_tags: &[TagSlot],
    gate_events: &mut Vec<TerrainGateObserved>,
) -> (f32, Vec<AdjacencyObserved>) {
    let mut interaction_delta = 0.0;
    let mut observations = Vec::new();
    let Some(exerter) = world.cells[exerter_idx].population else {
        return (interaction_delta, observations);
    };
    let exerter_species = &world.species[exerter.species.0 as usize];
    for &their_tag in &exerter_species.tags {
        if !tag_gate_satisfied(
            world,
            their_tag,
            world.cells[exerter_idx].terrain,
            gate_events,
        ) {
            continue;
        }
        for &my_tag in receiver_tags {
            if !tag_gate_satisfied(
                world,
                my_tag,
                world.cells[receiver_idx].terrain,
                gate_events,
            ) {
                continue;
            }
            let entry = world.matrix.get(their_tag, my_tag);
            let contribution = entry as f32 * energy.interaction_scale;
            interaction_delta += contribution;
            if entry != 0 {
                let n_confounders =
                    neighbour_tags.iter().filter(|&&t| t != their_tag).count() as u32;
                observations.push(AdjacencyObserved {
                    receiver_species,
                    exerter_tag: their_tag,
                    receiver_tag: my_tag,
                    contribution,
                    n_confounders,
                    cell: receiver_idx,
                });
            }
        }
    }
    (interaction_delta, observations)
}

/// One distinct neighbouring exerter tag's signed contribution to a cell's
/// current-tick energy balance (task 149's inspection card) — grouped by
/// tag identity, not by neighbour cell, matching the card's "one line per
/// tag" framing (a tag exerted by two neighbours folds into one line, its
/// contributions summed).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeighbourContribution {
    pub tag: TagId,
    pub contribution: f32,
}

/// A cell's last-pulse per-capita energy balance (task 149), computed
/// on-demand from the same inputs/formula `step`'s tick loop applies
/// (`~1095-1260`), not read from any persisted per-tick state — see the
/// task's "no new persistent per-tick state" constraint. Per-capita, like
/// `interaction_delta` in `step` itself, matching the card's own
/// "per-capita energy" line rather than an aggregate the player has no
/// other aggregate figure to compare it against.
#[derive(Debug, Clone, PartialEq)]
pub struct EnergyBreakdown {
    pub gain: f32,
    pub neighbours: Vec<NeighbourContribution>,
    pub upkeep: f32,
    pub crowding: f32,
    pub net: f32,
}

/// Pure recomputation of `cell_index`'s current energy balance (task 149).
/// `None` for an unoccupied cell. Deliberately reruns the tag-gate-checked
/// neighbour loop (via `adjacency_pair_observations`, the same helper
/// `step` uses) instead of reading `AdjacencyObserved`/`adjacency_exposure`:
/// those only ever carry onset evidence (task 136b), which goes silent for
/// a persisting neighbour after its first tick and so can't drive a "what's
/// happening right now" card. Gain for `Predator`/`Decomposer` is an
/// approximation post-tick: their shared-pool draw prepass in `step` reads
/// `world.scratch` mid-tick (already-decayed residue / not-yet-applied
/// deaths), which no longer exists once the tick has committed — this
/// recomputes the same formula from `world.cells`' current, settled state
/// instead.
pub fn cell_energy_breakdown(
    world: &SimWorld,
    config: &SimConfig,
    cell_index: usize,
) -> Option<EnergyBreakdown> {
    let energy = &config.energy;
    let cell = world.cells[cell_index];
    let occupant = cell.population?;
    let species = &world.species[occupant.species.0 as usize];
    let (x, y) = (cell_index % world.width, cell_index / world.width);

    let fit = env_fit(
        cell.temperature,
        species.temp_optimum,
        species.temp_tolerance,
    );
    let gain = match species.metabolism {
        Metabolism::Photolithic => cell.light * energy.photolithic_metabolism_gain * fit,
        Metabolism::Chemolithotroph => cell.toxicity * energy.chemolithotroph_metabolism_gain * fit,
        Metabolism::Predator => {
            let prey: Vec<usize> = world
                .moore_neighbours(x, y)
                .filter(|&n| world.cells[n].population.is_some())
                .collect();
            if prey.is_empty() {
                0.0
            } else {
                let available: f32 = prey
                    .iter()
                    .map(|&n| world.cells[n].population.unwrap().energy)
                    .sum();
                (energy.predator_drain_cap * fit).min(available)
            }
        }
        Metabolism::Decomposer => {
            let sources: Vec<usize> = std::iter::once(cell_index)
                .chain(world.moore_neighbours(x, y))
                .filter(|&n| world.cells[n].residue > 0.0)
                .collect();
            if sources.is_empty() {
                0.0
            } else {
                let available: f32 = sources.iter().map(|&n| world.cells[n].residue).sum();
                (energy.decomposer_extract_rate * fit).min(available)
            }
        }
    };

    let neighbour_tags = distinct_neighbour_tags(world, x, y);
    let mut gate_events = Vec::new();
    let mut neighbours: Vec<NeighbourContribution> = Vec::new();
    for neighbour_idx in world.moore_neighbours(x, y) {
        let (_, candidates) = adjacency_pair_observations(
            world,
            energy,
            cell_index,
            occupant.species,
            &species.tags,
            neighbour_idx,
            &neighbour_tags,
            &mut gate_events,
        );
        for candidate in candidates {
            let tag = world.active_tags[candidate.exerter_tag.0 as usize];
            if let Some(line) = neighbours.iter_mut().find(|line| line.tag == tag) {
                line.contribution += candidate.contribution;
            } else {
                neighbours.push(NeighbourContribution {
                    tag,
                    contribution: candidate.contribution,
                });
            }
        }
    }

    let occupied_neighbours = world
        .moore_neighbours(x, y)
        .filter(|&n| {
            world.cells[n]
                .population
                .is_some_and(|neighbour| neighbour.species != occupant.species)
        })
        .count();
    let upkeep = match species.metabolism {
        Metabolism::Photolithic => energy.base_upkeep,
        Metabolism::Predator => energy.predator_upkeep,
        Metabolism::Decomposer => energy.decomposer_upkeep,
        Metabolism::Chemolithotroph => energy.chemolithotroph_upkeep,
    };
    let crowding = energy.crowd_factor_for(cell.biome.index()) * occupied_neighbours as f32;
    let interaction: f32 = neighbours.iter().map(|line| line.contribution).sum();
    let net = gain + interaction - upkeep - crowding;

    Some(EnergyBreakdown {
        gain,
        neighbours,
        upkeep,
        crowding,
        net,
    })
}

/// The observation(s) a `Cull` click on `(x, y)` generates (task 146, GDD
/// §6 "Cull — knockout, non sterminio") — call *before* removing the
/// organism there. The culled organism is treated as the sole exerter onto
/// each still-living neighbour (the design doc's framing: "Y behaves
/// differently after removing X", not the reverse), one `AdjacencyObserved`
/// per `(exerter_tag, receiver_tag)` pair with a non-zero matrix entry, no
/// onset gating (a knockout is a one-shot event, not a persisting tick —
/// every currently-adjacent pair counts, once). A culled organism with no
/// living neighbours returns an empty vec: no panic, no zero-weight noise.
/// `TerrainGateObserved` evidence from the underlying tag-gate checks is
/// deliberately discarded — out of this task's scope, terrain evidence
/// stays a tick-loop concern.
pub fn cull_knockout_observations(
    world: &SimWorld,
    config: &SimConfig,
    x: usize,
    y: usize,
) -> Vec<AdjacencyObserved> {
    let idx = world.index(x, y);
    if world.cells[idx].population.is_none() {
        return Vec::new();
    }
    let mut discarded_gate_events = Vec::new();
    let mut observations = Vec::new();
    for neighbour_idx in world.moore_neighbours(x, y) {
        let Some(neighbour) = world.cells[neighbour_idx].population else {
            continue;
        };
        let neighbour_species = &world.species[neighbour.species.0 as usize];
        let (nx, ny) = (neighbour_idx % world.width, neighbour_idx / world.width);
        let neighbour_tags = distinct_neighbour_tags(world, nx, ny);
        let (_, candidates) = adjacency_pair_observations(
            world,
            &config.energy,
            neighbour_idx,
            neighbour.species,
            &neighbour_species.tags,
            idx,
            &neighbour_tags,
            &mut discarded_gate_events,
        );
        observations.extend(candidates);
    }
    observations
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
    // Task 137: a saturated-with-no-outlet population (GDD §5.11, "the
    // existing environmental-mismatch stimulus") folds its blocked growth in
    // here, bucketed by the same `terrain` this call already buckets
    // temperature mismatch by. `0.0` outside that case.
    extra_terrain_mismatch: f32,
) -> Option<SelectionThresholdCrossed> {
    if pressures.len() < species_len {
        pressures.resize(species_len, SelectionPressure::default());
    }
    let pressure = &mut pressures[species_id.0 as usize];
    if pressure.crossed {
        return None;
    }

    pressure.interaction_harm += (-interaction_delta).max(0.0) * evolution.interaction_harm_weight;
    pressure.terrain_mismatch[terrain.index()] +=
        (1.0 - fit) * evolution.terrain_mismatch_weight + extra_terrain_mismatch;
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
/// Task 142: also read by `build_era_reveal`/`text::era_reveal_evolution_line`
/// to name the cause in the end-of-era reveal card, not just to pick
/// `speciate`'s edit — hence `pub` since task 107.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DominantStimulus {
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

pub fn dominant_stimulus(event: &SelectionThresholdCrossed) -> DominantStimulus {
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
    if let Some(population) = world.cells[event.cell].population.as_mut() {
        if population.species == event.species {
            population.species = new_species_id;
        }
    }
    // Task 109: backs `Objective::Speciation`, the long-term objective's
    // default content.
    world.has_speciated = true;
    Some(new_species_id)
}

/// Task 140: `SelectionThresholdCrossed` events accumulated during the
/// current era, *not* applied immediately — `speciate` only runs once the
/// era closes (`build_era_reveal`), so the resulting descendant is revealed
/// alongside everything else that happened this era rather than the
/// instant its pressure crossed the threshold. Drained (via
/// `std::mem::take`) exactly once per era, in `build_era_reveal`.
#[derive(Resource, Default)]
pub struct PendingEvolutions(pub Vec<SelectionThresholdCrossed>);

/// Running per-era tallies (task 140), accumulated tick-by-tick in
/// `tick_and_complete_season` and drained into `EraReveal` when the era
/// closes — the raw counts a reveal card summarizes, kept separate from
/// `EraReveal` itself since they must survive across every tick of the era,
/// not just the one the reveal is built on.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct EraTally {
    pub births: u32,
    pub deaths: u32,
    pub extinctions: u32,
}

/// How much presentation weight an end-of-era reveal earns (task 140,
/// `redesign/processed/abiogenesis-time-scale-reveal.md` §3): "a minor event
/// can be a discreet badge, an epochal one can take the whole screen." This
/// is a first-pass, deliberately simple heuristic — task 157 builds the real
/// event-ranking score everything (including this tier) should eventually
/// read from instead; do not grow a second, competing scoring system here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RevealTier {
    #[default]
    Minor,
    Notable,
    Epochal,
}

/// One matured evolution applied at the reveal (task 140): the before/after
/// comparison the reveal card shows alongside its generated text
/// (`redesign/processed/abiogenesis-time-scale-reveal.md` §3's "an icon that
/// changes if it evolved a trait"), plus `dominant_stimulus` (task 142) so
/// the reveal's generated text can name *why* the descendant evolved. Still
/// only carries data, not a rendered sentence — `text::era_reveal_evolution_line`
/// owns the wording.
#[derive(Debug, Clone)]
pub struct EraEvolutionReveal {
    pub parent: SpeciesId,
    pub parent_name: String,
    pub parent_tag_count: usize,
    pub child: SpeciesId,
    pub child_name: String,
    pub child_tag_count: usize,
    /// Task 142: which stimulus dominated the pressure that triggered this
    /// evolution — computed from the same `SelectionThresholdCrossed` event
    /// `speciate` itself reads (`dominant_stimulus`), so the reveal's
    /// generated cause clause and the actual edit `speciate` applied can
    /// never disagree.
    pub dominant_stimulus: DominantStimulus,
}

/// The end-of-era reveal's full content (task 140), built exactly once per
/// era by `build_era_reveal` and read by `screens::era_reveal_screen_ui`
/// until the player dismisses it.
#[derive(Resource, Default, Debug, Clone)]
pub struct EraReveal {
    pub era: u32,
    pub tier: RevealTier,
    pub evolutions: Vec<EraEvolutionReveal>,
    /// Task 140's adopted answer to the design doc's second open question:
    /// a pending evolution whose species went fully extinct before the era
    /// closed is simply lost, not applied — counted here rather than
    /// silently dropped, so the reveal can at least say "N evolutions were
    /// lost this era." Data-design note for a future "stratigraphic record"
    /// (a cell remembering who died there and when, not implemented by this
    /// task): a lost evolution is exactly the kind of fact such a record
    /// would want to keep — keep that in mind if this field's shape is ever
    /// revisited, rather than needing to reintroduce the concept from
    /// scratch.
    pub evolutions_lost: u32,
    pub births: u32,
    pub deaths: u32,
    pub extinctions: u32,
}

/// Task 140's adopted answer to "do maturing evolutions give hints before
/// the reveal, or is it a total surprise": yes — an indirect signal (here,
/// a HUD hint; `ui::hud_panel` is the actual consumer) that *something* is
/// nearing a threshold, without saying which species or why, so the reveal
/// lands as a confirmation of something the player already noticed rather
/// than a twist from nowhere. First-pass and deliberately simple: a single
/// fixed fraction, not exposed in `SimConfig` — the design doc explicitly
/// leaves exact relevance thresholds for playtest-time tuning, and task 157
/// is where a real ranked-hint system belongs, not a second one grown here.
const MATURING_HINT_FRACTION: f32 = 0.6;

/// True if any species not yet past `SelectionPressure::crossed` has
/// already accumulated `MATURING_HINT_FRACTION` of
/// `EvolutionConfig::selection_pressure_threshold` — see
/// `MATURING_HINT_FRACTION`'s own doc comment for the design intent.
pub fn any_evolution_maturing(world: &SimWorld, config: &EvolutionConfig) -> bool {
    world.selection_pressure.iter().any(|pressure| {
        !pressure.crossed
            && pressure.total() >= config.selection_pressure_threshold * MATURING_HINT_FRACTION
    })
}

/// Task 143 (`redesign/processed/culture-shock-friction-fixes.md`,
/// Intervento 1): a population's per-capita net energy is "at an apparent
/// stall" once it's stayed within this band of zero — proposed value from
/// the design doc itself, indicative and left for playtest tuning like
/// `MATURING_HINT_FRACTION`, hence a local const rather than a `SimConfig`
/// field.
const STALL_BAND: f32 = 0.1;

/// Consecutive ticks within `STALL_BAND` before the second contextual hint
/// (`ui::stall_hint_text`) is allowed to fire — same source, same
/// "indicative, playtest-tuned" status as `STALL_BAND`.
const STALL_TICKS_THRESHOLD: u32 = 15;

/// True once any occupied cell has held a stalled net energy for
/// `STALL_TICKS_THRESHOLD` consecutive ticks — read by `ui`'s stall-hint
/// system to decide whether to show it (once per process session, mirroring
/// `IsolationHint`'s `MetaProgress`-gated one-shot).
pub fn any_population_stalled(world: &SimWorld) -> bool {
    world
        .stall_ticks
        .iter()
        .any(|&ticks| ticks >= STALL_TICKS_THRESHOLD)
}

/// True if `species` still has at least one living individual anywhere on
/// the grid — the exact check task 140's "extinct before it matures" rule
/// needs, distinct from `event.cell` specifically (a population can move
/// via breakout, or simply die and be replaced elsewhere, well before the
/// era that started its speciation actually closes).
fn species_still_extant(world: &SimWorld, species: SpeciesId) -> bool {
    world
        .cells
        .iter()
        .any(|cell| cell.population.is_some_and(|p| p.species == species))
}

/// Applies every evolution that matured this era (task 140) — drains
/// `PendingEvolutions`, skips any whose species already went fully extinct
/// (`species_still_extant`), and builds `EraReveal` from the result plus
/// this era's `EraTally`. Runs once per era, on `OnEnter(EraState::Reveal)`
/// (`SimPlugin::build`) — by construction that's exactly once, since
/// `EraState` only transitions there when an era actually just closed.
fn build_era_reveal(
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut pending: ResMut<PendingEvolutions>,
    mut tally: ResMut<EraTally>,
    mut reveal: ResMut<EraReveal>,
    mut evolved: MessageWriter<SpeciesEvolved>,
) {
    let crossed = std::mem::take(&mut pending.0);
    let mut evolutions = Vec::new();
    let mut lost = 0;
    for event in crossed {
        if !species_still_extant(&world, event.species) {
            lost += 1;
            continue;
        }
        let parent_name = world.species[event.species.0 as usize].name.clone();
        let parent_tag_count = world.species[event.species.0 as usize].tags.len();
        let stimulus = dominant_stimulus(&event);
        if let Some(child) = speciate(&mut world, &config, &event) {
            evolutions.push(EraEvolutionReveal {
                parent: event.species,
                parent_name,
                parent_tag_count,
                child,
                child_name: world.species[child.0 as usize].name.clone(),
                child_tag_count: world.species[child.0 as usize].tags.len(),
                dominant_stimulus: stimulus,
            });
            evolved.write(SpeciesEvolved { species: child });
        }
    }

    let tally = std::mem::take(&mut *tally);
    let tier = if !evolutions.is_empty() {
        RevealTier::Epochal
    } else if tally.extinctions > 0 || lost > 0 {
        RevealTier::Notable
    } else {
        RevealTier::Minor
    };

    *reveal = EraReveal {
        era: world.era,
        tier,
        evolutions,
        evolutions_lost: lost,
        births: tally.births,
        deaths: tally.deaths,
        extinctions: tally.extinctions,
    };
}

pub fn step(world: &mut SimWorld, config: &SimConfig) -> TickEvents {
    let energy = &config.energy;
    debug_assert!(
        energy
            .residue_decay
            .iter()
            .all(|&decay| energy.residue_ambient_trickle < decay),
        "residue_ambient_trickle must stay below every biome's residue_decay, or residue grows unboundedly"
    );
    debug_assert!(
        energy.repro_cost > 0.0,
        "repro_cost must be positive, or population growth (task 137) never terminates"
    );
    let mut events = TickEvents::default();

    // Pre-tick count of occupied cells per species, for extinction detection
    // (1 -> 0 transition) once deaths are recorded below. Counts cells, not
    // individuals (task 137): death is all-or-nothing per cell (see step 6
    // below), so a species goes extinct exactly when its last occupied cell
    // empties out, regardless of how many individuals were in it.
    let mut population = vec![0u32; world.species.len()];
    for cell in world.cells.iter() {
        if let Some(population_here) = cell.population {
            population[population_here.species.0 as usize] += 1;
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
    // tick (that only happens in `tick_and_complete_season`, after `step`
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
    // Relaxes any actively-`Stress`ed axis back toward its pre-stress value
    // (task 145) — after reinjection, so it can see (and skip) the cells
    // reinjection already owns for that axis.
    world.decay_environment_stress(config);

    // Residue decays every tick unless a death overwrites it further down.
    // A small ambient trickle is added after decay so residue settles to a
    // low equilibrium everywhere, keeping an isolated Decomposer readable
    // instead of starving out with zero information.
    for cell in world.scratch.iter_mut() {
        cell.residue = (cell.residue - energy.residue_decay_for(cell.biome.index())).max(0.0)
            + energy.residue_ambient_trickle;
    }

    // Predation pre-pass (GDD §5.4): a shared-resource drain computed from
    // the immutable snapshot, into per-cell accumulators, before the main
    // loop — see TECH_DESIGN.md §6 "Shared resource drain". This keeps the
    // tick order-independent: a predator never writes directly into a prey's
    // scratch entry while iterating.
    let mut predation_gain = vec![0.0f32; world.cells.len()];
    let mut predation_loss = vec![0.0f32; world.cells.len()];
    for (idx, cell) in world.cells.iter().enumerate() {
        let Some(population) = cell.population else {
            continue;
        };
        let species = &world.species[population.species.0 as usize];
        if species.metabolism != Metabolism::Predator {
            continue;
        }
        let (x, y) = (idx % world.width, idx / world.width);
        let prey: Vec<usize> = world
            .moore_neighbours(x, y)
            .filter(|&n| world.cells[n].population.is_some())
            .collect();
        if prey.is_empty() {
            continue;
        }
        let fit = env_fit(
            cell.temperature,
            species.temp_optimum,
            species.temp_tolerance,
        );
        // The drawable pool is each prey cell's aggregate energy, same as
        // before task 137 — an aggregate quantity, not scaled by predator
        // count, so more predators sharing a cell share the same fixed pool.
        let available: f32 = prey
            .iter()
            .map(|&n| world.cells[n].population.unwrap().energy)
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
        let Some(population) = cell.population else {
            continue;
        };
        let species = &world.species[population.species.0 as usize];
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

    // Task 136b: new `AdjacencyExposure` per occupied cell, collected here
    // rather than written straight into `world.adjacency_exposure` inside
    // the loop below — that loop already borrows `world.species` for the
    // whole iteration (`species`, read further down for upkeep/reproduction
    // too), and applying the update in one pass afterwards sidesteps that
    // entirely rather than fighting it with narrower borrows.
    let mut new_exposure: Vec<Option<AdjacencyExposure>> = vec![None; world.cells.len()];

    for idx in 0..world.cells.len() {
        let cell = world.cells[idx];
        let Some(occupant) = cell.population else {
            continue;
        };
        let species = &world.species[occupant.species.0 as usize];

        // 0. Habitat gate (task 138): a biome can make a cell outright
        // uninhabitable independent of scalars (deep water is not "low
        // light and cold", it is a place a land organism cannot be).
        // Ships neutral (all-habitable) by default — see
        // `EnergyConfig::biome_habitable`'s doc comment — so this is a
        // structural attachment point, not yet a real balance restriction.
        if !energy.is_habitable(cell.biome.index()) {
            world.scratch[idx].population = None;
            world.scratch[idx].residue = energy.residue_on_death;
            events.deaths.push(OrganismDied {
                cell: idx,
                species: occupant.species,
                cause: DeathCause::Habitat,
                gain: 0.0,
                env_fit: 0.0,
                interaction_delta: 0.0,
                upkeep: 0.0,
                crowding_penalty: 0.0,
                predation_loss: 0.0,
                energy_before: occupant.energy,
            });
            let species_idx = occupant.species.0 as usize;
            population[species_idx] -= 1;
            if population[species_idx] == 0 {
                events.extinctions.push(SpeciesExtinct {
                    species: occupant.species,
                });
            }
            continue;
        }

        // 1-2. Environmental fitness and metabolic gain. Photolithic/
        // Chemolithotroph gain is a genuine per-capita rate (each individual
        // independently reads the same `light`/`toxicity`), scaled to a
        // total below; Predator/Decomposer gain is already an aggregate
        // shared-resource draw from the pre-pass above, unaffected by how
        // many individuals split it (task 137: crowding self-limits those
        // two metabolisms for free, without touching the pre-pass code).
        let fit = env_fit(
            cell.temperature,
            species.temp_optimum,
            species.temp_tolerance,
        );
        let per_capita_gain = match species.metabolism {
            Metabolism::Photolithic => cell.light * energy.photolithic_metabolism_gain * fit,
            Metabolism::Chemolithotroph => {
                cell.toxicity * energy.chemolithotroph_metabolism_gain * fit
            }
            Metabolism::Predator | Metabolism::Decomposer => 0.0,
        };
        let aggregate_gain = match species.metabolism {
            Metabolism::Photolithic | Metabolism::Chemolithotroph => {
                per_capita_gain * occupant.count as f32
            }
            Metabolism::Predator => predation_gain[idx],
            Metabolism::Decomposer => decomposition_gain[idx],
        };

        // 3. Hidden matrix effect (GDD §5.6 step 3, §5.5): additive and
        // linear (invariant 4), read only from the snapshot like everything
        // else here, so the tick stays order-independent. Task 137: matrix
        // interaction is by *presence*, not by quantity — a neighbouring
        // cell contributes its tags exactly once regardless of how many
        // individuals occupy it — so this per-cell rate is unchanged from
        // before 137 and is scaled to a total by `occupant.count` below,
        // same as the per-capita gain above.
        let (x, y) = (idx % world.width, idx / world.width);

        // Protected-cell hook (task 138): the attachment point for the
        // future `Isola` action, not implemented here — only made
        // near-free to retrofit now, before it's expensive to. Always
        // `false` today; when a cell is protected, this phase contributes
        // zero.
        let protected = false;

        let mut interaction_delta = 0.0;

        // Distinct exerter tags carried by this cell's occupied Moore
        // neighbours, gathered up front so the confounder count (see
        // `AdjacencyObserved`'s doc comment) is available for every
        // observation emitted below without re-scanning neighbours per tag.
        let neighbour_tags = distinct_neighbour_tags(world, x, y);

        // Task 136b: evidence accrues on a tag's *onset* into adjacency, not
        // on every tick it persists. `prior_mask` is this cell's exposure as
        // of the last tick it was processed — discarded (treated as empty)
        // if a different species now occupies the cell (task 137: the
        // staleness key moved from `Organism::born_season`, which an
        // aggregate population no longer has one of, to the occupying
        // species — a fresh population has observed nothing yet). `onset_mask`
        // is exactly the tags that are adjacent now but weren't last time;
        // `current_mask` (all tags adjacent now) becomes next tick's
        // `prior_mask` regardless of which tags were newly onset.
        let prior = world.adjacency_exposure[idx];
        let prior_mask = if prior.owner_species == Some(occupant.species) {
            prior.exerter_tags
        } else {
            0
        };
        let mut current_mask: u32 = 0;
        for &tag in &neighbour_tags {
            current_mask |= 1 << tag.0;
        }
        let onset_mask = current_mask & !prior_mask;
        new_exposure[idx] = Some(AdjacencyExposure {
            owner_species: Some(occupant.species),
            exerter_tags: current_mask,
        });

        if !protected {
            for neighbour_idx in world.moore_neighbours(x, y) {
                let (contribution, candidates) = adjacency_pair_observations(
                    world,
                    energy,
                    idx,
                    occupant.species,
                    &species.tags,
                    neighbour_idx,
                    &neighbour_tags,
                    &mut events.terrain_gates,
                );
                interaction_delta += contribution;
                // Task 136b: only a tag's *onset* into adjacency is evidence
                // (see `onset_mask` above) — `adjacency_pair_observations`
                // itself is onset-agnostic (Cull's knockout call below needs
                // every currently-adjacent pair), so the tick loop applies
                // that filter here, on its own output.
                for candidate in candidates {
                    if onset_mask & (1 << candidate.exerter_tag.0) != 0 {
                        events.adjacencies.push(candidate);
                    }
                }
            }
        }
        let aggregate_interaction = interaction_delta * occupant.count as f32;

        // Task 106: accumulate this tick's selection-pressure stimuli
        // (interaction harm, terrain/temp-optimum mismatch, toxicity) into
        // the population's species tally, before costs/death are resolved —
        // the pressure reflects exposure this tick regardless of whether the
        // population survives it. Stays a per-capita reading (unaffected by
        // `occupant.count`), same as before task 137.
        if let Some(crossed) = accumulate_selection_pressure(
            &mut world.selection_pressure,
            &config.evolution,
            world.species.len(),
            occupant.species,
            idx,
            interaction_delta,
            fit,
            cell.terrain,
            cell.toxicity,
            0.0,
        ) {
            events.selection_thresholds.push(crossed);
        }

        // 4. Costs: base upkeep plus a carrying-capacity penalty per
        // occupied neighbour of a *different* species, read from the
        // snapshot so the tick stays order-independent.
        //
        // Task 136 restricted this to cross-species neighbours: a
        // same-species neighbour contributes exactly zero
        // `interaction_delta` by construction (the matrix diagonal is
        // always zero, GDD invariant, `net_self_interaction == 0`), so it is
        // a pair the matrix can never compensate for no matter how the
        // player plays. Charging it crowding too would double-penalize
        // exactly that case, on top of the already-thin margin the retuned
        // gains leave. Task 137 keeps this by-presence (one distinct
        // different-species neighbour cell = one charge, regardless of that
        // neighbour's own count) so it doesn't silently retune 136's
        // coefficients; same-species density is now capped directly by
        // `cell_carrying_capacity` instead.
        let occupied_neighbours = world
            .moore_neighbours(x, y)
            .filter(|&n| {
                world.cells[n]
                    .population
                    .is_some_and(|neighbour| neighbour.species != occupant.species)
            })
            .count();
        let per_capita_upkeep = match species.metabolism {
            Metabolism::Photolithic => energy.base_upkeep,
            Metabolism::Predator => energy.predator_upkeep,
            Metabolism::Decomposer => energy.decomposer_upkeep,
            Metabolism::Chemolithotroph => energy.chemolithotroph_upkeep,
        };
        let per_capita_crowding =
            energy.crowd_factor_for(cell.biome.index()) * occupied_neighbours as f32;
        let aggregate_upkeep = per_capita_upkeep * occupant.count as f32;
        let aggregate_crowding = per_capita_crowding * occupant.count as f32;

        // 5. Energy update, on the aggregate.
        let new_energy = occupant.energy + aggregate_gain + aggregate_interaction
            - aggregate_upkeep
            - aggregate_crowding
            - predation_loss[idx];

        // 6. Death: all-or-nothing per cell (task 137 keeps this shape from
        // the one-organism-per-cell model rather than introducing a partial
        // die-off rule) — once the aggregate can no longer sustain any of
        // its members, the local population collapses together. `Cull`
        // (`actions.rs`) already relies on emptying a cell being decisive;
        // starvation now works the same way.
        if new_energy <= 0.0 {
            world.scratch[idx].population = None;
            // Fixed on death, not decayed leftover + this value (invariant
            // in task spec): a death this tick overwrites the residue,
            // regardless of how many individuals were in the population.
            world.scratch[idx].residue = energy.residue_on_death;
            events.deaths.push(OrganismDied {
                cell: idx,
                species: occupant.species,
                cause: DeathCause::Starvation,
                gain: per_capita_gain,
                env_fit: fit,
                interaction_delta,
                upkeep: per_capita_upkeep,
                crowding_penalty: per_capita_crowding,
                predation_loss: predation_loss[idx],
                energy_before: occupant.energy,
            });
            let species_idx = occupant.species.0 as usize;
            population[species_idx] -= 1;
            if population[species_idx] == 0 {
                events.extinctions.push(SpeciesExtinct {
                    species: occupant.species,
                });
            }
            world.stall_ticks[idx] = 0;
            continue;
        }

        // Task 143: consecutive-tick "apparent stall" tracking — a
        // per-capita net energy that stays within `STALL_BAND` of zero
        // reads, to a new player, identically to "nothing is happening."
        // Deliberately measured against the pre-growth aggregate (`count`,
        // not whatever `count`/`energy_left` growth below produces) — this
        // is the same net that decided whether this population is
        // starving, growing, or holding, not a post-hoc reading.
        let net_per_capita = (new_energy - occupant.energy) / occupant.count as f32;
        world.stall_ticks[idx] = if net_per_capita.abs() < STALL_BAND {
            world.stall_ticks[idx].saturating_add(1)
        } else {
            0
        };

        mark_terrain_and_maybe_reveal(
            &mut world.terrain_occupancy,
            &world.active_tags,
            &world.conditional_tags,
            world.species.len(),
            occupant.species,
            &species.tags,
            cell.terrain,
            &mut events.reveals,
        );

        // 7. Growth (task 137, replacing single-organism reproduction):
        // continuous — while the population's average per-capita energy is
        // at or above `repro_threshold`, count grows by one and the
        // aggregate pays `repro_cost`, same as the old per-organism
        // reproduction cost, just looped. Gated by `born_season < world.season`
        // exactly as reproduction always was — a population founded this
        // tick (by breakout, below) doesn't also grow the same tick.
        let mut count = occupant.count;
        let mut energy_left = new_energy;
        if occupant.born_season < world.season {
            while energy_left / count as f32 >= species.repro_threshold {
                count += 1;
                energy_left -= energy.repro_cost;
                events.births.push(OrganismBorn {
                    species: occupant.species,
                });
            }
        }

        // 8. Breakout (task 137): once `count` exceeds the cell's carrying
        // capacity, the excess must migrate to a neighbouring cell that is
        // either empty (and placeable) or already holds the same species
        // under its own capacity — mirrors the old reproduction target
        // search exactly (snapshot-collected candidates in index order, one
        // RNG draw, scratch re-checked to resolve same-tick contention).
        // With no valid outlet, the excess is capped away and its energy
        // share feeds local selection pressure instead (GDD §5.11's
        // existing "environmental mismatch" stimulus) — `blocked` records
        // this for rendering (task 141).
        let mut blocked = false;
        if count > config.energy.cell_carrying_capacity {
            let excess = count - config.energy.cell_carrying_capacity;
            let excess_energy = energy_left * (excess as f32 / count as f32);
            energy_left -= excess_energy;
            count = config.energy.cell_carrying_capacity;
            // Cloned rather than borrowed: `world.rng_mut()` below needs
            // `&mut world` as a whole (it's a method, so the borrow checker
            // can't see that it's disjoint from `world.species`), which
            // would conflict with `species` still being borrowed for
            // `mark_terrain_and_maybe_reveal` afterwards.
            let breakout_tags = species.tags.clone();

            let candidates: Vec<usize> = world
                .moore_neighbours(x, y)
                .filter(|&n| match world.cells[n].population {
                    None => world.is_placeable_index_for(n, occupant.species),
                    Some(neighbour) => {
                        neighbour.species == occupant.species
                            && neighbour.count < config.energy.cell_carrying_capacity
                    }
                })
                .collect();

            if candidates.is_empty() {
                blocked = true;
                if let Some(crossed) = accumulate_selection_pressure(
                    &mut world.selection_pressure,
                    &config.evolution,
                    world.species.len(),
                    occupant.species,
                    idx,
                    0.0,
                    1.0,
                    cell.terrain,
                    0.0,
                    excess_energy * config.evolution.terrain_mismatch_weight,
                ) {
                    events.selection_thresholds.push(crossed);
                }
            } else {
                let pick = world.rng_mut().random_range(0..candidates.len());
                let target = candidates[pick];
                let placed = match world.scratch[target].population {
                    None => {
                        world.scratch[target].population = Some(Population {
                            species: occupant.species,
                            count: excess,
                            energy: excess_energy,
                            born_season: world.season,
                            blocked: false,
                        });
                        true
                    }
                    Some(existing)
                        if existing.species == occupant.species
                            && existing.count + excess <= config.energy.cell_carrying_capacity =>
                    {
                        world.scratch[target].population = Some(Population {
                            count: existing.count + excess,
                            energy: existing.energy + excess_energy,
                            ..existing
                        });
                        true
                    }
                    // Contention: another cell's breakout already claimed this
                    // target this tick, or it filled up in the meantime. The
                    // excess (and its energy) is lost this tick rather than
                    // invented a second target — rare, and the origin cell
                    // will simply re-attempt breakout next tick if still over
                    // capacity.
                    _ => false,
                };
                if placed {
                    mark_terrain_and_maybe_reveal(
                        &mut world.terrain_occupancy,
                        &world.active_tags,
                        &world.conditional_tags,
                        world.species.len(),
                        occupant.species,
                        &breakout_tags,
                        world.cells[target].terrain,
                        &mut events.reveals,
                    );
                    events.births.push(OrganismBorn {
                        species: occupant.species,
                    });
                }
            }
        }

        world.scratch[idx].population = Some(Population {
            species: occupant.species,
            count,
            energy: energy_left,
            born_season: occupant.born_season,
            blocked,
        });
    }

    for (idx, exposure) in new_exposure.into_iter().enumerate() {
        if let Some(exposure) = exposure {
            world.adjacency_exposure[idx] = exposure;
        }
    }

    std::mem::swap(&mut world.cells, &mut world.scratch);
    world.tick += 1;
    events
}

/// Gaussian environmental fitness around the species' thermal optimum (GDD §5.9).
///
/// `pub` since task 134: the headless bot survey needs to judge where a
/// species would be viable before placing it — the same reading a player
/// makes from the temperature overlay and the species' thermal range — and
/// re-deriving the curve in the harness would be a second copy of a §5.9
/// formula free to drift from this one.
pub fn env_fit(temperature: f32, optimum: f32, tolerance: f32) -> f32 {
    let d = temperature - optimum;
    (-(d * d) / (2.0 * tolerance * tolerance)).exp()
}

/// Groups the per-era simulation advancement (TECH_DESIGN.md §3.4).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    Advance,
}

/// Ticks left in the season currently being animated (TECH_DESIGN.md §3.4;
/// renamed from `EraProgress` by task 135 — this tracks the season, the
/// decision-cadence unit, not the era).
#[derive(Resource, Default)]
pub struct SeasonProgress {
    pub(crate) remaining: u32,
}

impl SeasonProgress {
    pub fn start(&mut self, ticks: u32) {
        self.remaining = ticks;
    }

    pub fn remaining(&self) -> u32 {
        self.remaining
    }

    /// Used by the `r` key (world reset) to cancel any season in progress.
    pub fn cancel(&mut self) {
        self.remaining = 0;
    }
}

/// The player's action points for the current `EraState::Observing` window
/// (GDD §6): `Seed` and, from tasks 023-025, Stress/Cull/Splice each spend
/// from this pool. Refilled to `config.time.point_budget_per_season`
/// whenever a season finishes (`advance_tick`'s "season just ended" branch,
/// task 135) and by the `r` key alongside every other piece of fresh-world
/// state.
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
        let point_budget_per_season = config.time.point_budget_per_season;
        app.insert_resource(Time::<Fixed>::from_hz(era_tick_hz));
        app.init_resource::<SeasonProgress>();
        // `init_resource` alone would leave `points_remaining = 0` (the
        // `Default` impl) until the first season finishes — the very first
        // `Observing` window needs a full budget too.
        app.insert_resource(ActionBudget {
            points_remaining: point_budget_per_season,
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
        app.init_resource::<PendingEvolutions>();
        app.init_resource::<EraTally>();
        app.init_resource::<EraReveal>();
        app.add_systems(
            FixedUpdate,
            advance_tick
                .in_set(SimSet::Advance)
                .run_if(in_state(EraState::Advancing)),
        );
        // `Update`, not `FixedUpdate`/`SimSet::Advance` (task 107): this
        // only reads already-drained `SelectionThresholdCrossed` messages
        // and buffers them — it doesn't need to run in lockstep with tick
        // advancement, and gating it to `EraState::Advancing` would miss
        // crossings produced by `single_tick`'s manual-tick path
        // (`input.rs`), which runs outside `Advancing`. Mirrors
        // `notebook.rs`'s message-consuming systems' scheduling (`Update`,
        // gated on `GameState::Playing` only) even though this one mutates
        // `SimWorld` and they don't — `notebook.rs` is documented
        // read-only, so the mutating half of task 107/140 belongs here in
        // `sim`, not there.
        app.add_systems(
            Update,
            buffer_pending_evolutions.run_if(in_state(GameState::Playing)),
        );
        // Task 140: applies every evolution buffered this era and builds
        // the reveal card's content exactly once, the instant `EraState`
        // transitions into `Reveal` (`advance_tick`/`input::single_tick`,
        // on an era — not just a season — closing).
        app.add_systems(OnEnter(EraState::Reveal), build_era_reveal);
    }
}

/// Advances one tick of the currently-animating season, then transitions
/// back to `Observing` once `season_pulses` have been played out. Guards
/// against a stray extra `FixedUpdate` execution landing before the state
/// transition takes effect, which would otherwise run one tick too many.
///
/// Each parameter is a distinct Bevy resource/writer this system needs, not
/// incidental complexity — splitting it wouldn't reduce the coupling, only
/// hide it, so the arg count is allowed rather than fought.
/// Runs one tick and, if it was the season's last tick, performs the
/// season-completion bookkeeping (`world.season` advance, budget refill)
/// plus, every `seasons_per_era` seasons, the era-completion bookkeeping
/// (`world.era` advance, `EraCompleted` — task 135 moved the season/era
/// split here). Shared by `advance_tick`'s auto-play and `single_tick`'s
/// manual step (`input.rs`) so a season is exactly `season_pulses` ticks —
/// and completes identically — regardless of which key triggered them.
/// `tally` accumulates this tick's births/deaths/extinctions (task 140)
/// regardless of season/era boundaries — `build_era_reveal` drains it
/// exactly once, when the era it was accruing for actually closes.
pub fn tick_and_complete_season(
    world: &mut SimWorld,
    config: &SimConfig,
    progress: &mut SeasonProgress,
    budget: &mut ActionBudget,
    era_completed: &mut MessageWriter<EraCompleted>,
    tally: &mut EraTally,
) -> TickEvents {
    let events = step(world, config);
    tally.births += events.births.len() as u32;
    tally.deaths += events.deaths.len() as u32;
    tally.extinctions += events.extinctions.len() as u32;
    progress.remaining -= 1;
    if progress.remaining() == 0 {
        world.season += 1;
        budget.refill(config.time.point_budget_per_season);
        if world.season.is_multiple_of(config.time.seasons_per_era) {
            world.era += 1;
            era_completed.write(EraCompleted { era: world.era });
        }
    }
    events
}

#[allow(clippy::too_many_arguments)]
fn advance_tick(
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut progress: ResMut<SeasonProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    mut budget: ResMut<ActionBudget>,
    mut era_completed: MessageWriter<EraCompleted>,
    mut tally: ResMut<EraTally>,
    mut writers: TickEventWriters,
) {
    if progress.remaining() == 0 {
        return;
    }
    let era_before = world.era;
    let events = tick_and_complete_season(
        &mut world,
        &config,
        &mut progress,
        &mut budget,
        &mut era_completed,
        &mut tally,
    );
    writers.write_all(events);
    if progress.remaining() == 0 {
        // Task 140: an era that just closed (not merely a season) halts on
        // its own for the reveal card instead of returning straight to
        // `Observing` — `build_era_reveal` (`OnEnter(EraState::Reveal)`)
        // does the rest.
        if world.era != era_before {
            next_state.set(EraState::Reveal);
        } else {
            next_state.set(EraState::Observing);
        }
    }
}

/// Task 140: `SelectionThresholdCrossed` events are no longer applied the
/// instant they cross (that was task 107's original behavior) — they're
/// buffered here and only turned into an actual descendant species at the
/// era's close (`build_era_reveal`), so the reveal card can show it
/// alongside everything else that happened this era.
fn buffer_pending_evolutions(
    mut pending: ResMut<PendingEvolutions>,
    mut crossed: MessageReader<SelectionThresholdCrossed>,
) {
    pending.0.extend(crossed.read().copied());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{
        Biome, Cell, ConditionalTag, SelectionPressure, Species, SpeciesId, TagId, TagMatrix,
        TagSlot, TerrainKind,
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
            population: Some(Population {
                species: SpeciesId(0),
                count: 1,
                energy,
                born_season: 0,
                blocked: false,
            }),
            ..world.cells[idx]
        };
        (world, config)
    }

    /// Task 143: a population whose gain and upkeep very nearly cancel
    /// (light chosen so net per-capita ≈ 0, same gain formula
    /// `isolated_photolithic_grows` documents: `light * 1.4 - upkeep`)
    /// accumulates `stall_ticks`; a population with a healthy net margin
    /// (the `isolated_photolithic_grows` setup itself, net +0.48) does not.
    #[test]
    fn step_accumulates_stall_ticks_only_for_a_near_equilibrium_population() {
        let (mut stalled_world, config) = world_with_one_organism(0.5 / 1.4, 0.5, 5.0);
        let idx = stalled_world.index(stalled_world.width / 2, stalled_world.height / 2);
        step(&mut stalled_world, &config);
        assert_eq!(
            stalled_world.stall_ticks[idx], 1,
            "a near-zero net per-capita tick must increment stall_ticks"
        );

        let (mut healthy_world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        let idx = healthy_world.index(healthy_world.width / 2, healthy_world.height / 2);
        step(&mut healthy_world, &config);
        assert_eq!(
            healthy_world.stall_ticks[idx], 0,
            "a healthy positive net per-capita tick must not increment stall_ticks"
        );
    }

    /// Task 143: `any_population_stalled` is a pure threshold read over
    /// `SimWorld::stall_ticks` — this only proves the threshold wiring,
    /// `step`'s own increment/reset logic is exercised indirectly by every
    /// other `step`-based test in this module continuing to pass unchanged.
    #[test]
    fn any_population_stalled_reads_the_threshold_correctly() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        assert!(
            !any_population_stalled(&world),
            "a freshly built world has no stalled cell"
        );

        world.stall_ticks[0] = STALL_TICKS_THRESHOLD - 1;
        assert!(
            !any_population_stalled(&world),
            "one tick short of the threshold must not count as stalled"
        );

        world.stall_ticks[0] = STALL_TICKS_THRESHOLD;
        assert!(
            any_population_stalled(&world),
            "reaching the threshold must count as stalled"
        );
    }

    #[test]
    fn isolated_photolithic_grows() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let organism = world.get(cx, cy).population.expect("organism survives");
        // Task 136: gain 0.7 * 1.4 * 1 - upkeep 0.5 = net +0.48 — a modest
        // margin by design, so a matrix interaction still meaningfully
        // speeds things up, but not so thin that ordinary environmental
        // drift (`diffuse_environment`) starves an isolated organism out on
        // its own before it ever gets the chance (see this constant's own
        // doc comment on `EnergyConfig::photolithic_metabolism_gain`).
        assert!(
            (organism.energy - 5.48).abs() < TOLERANCE,
            "expected net +0.48, got {}",
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
            world.cells[idx].population = Some(Population {
                species: SpeciesId(0),
                count: 1,
                energy: 1.0,
                born_season: 0,
                blocked: false,
            });
        }
        let target = neighbours[0];
        world.cells[target].terrain = TerrainKind::Sea;

        let events = step(&mut world, &config);

        assert!(
            world.cells[target].population.is_none(),
            "Sea must never receive an offspring, even as the only occupancy-empty neighbour"
        );
        assert!(
            events.births.is_empty(),
            "reproduction must not succeed with no placeable empty neighbour"
        );
    }

    /// Task 083 (moved from era to season by task 135): an organism born
    /// this season must not reproduce even with energy above
    /// `repro_threshold` — it has to survive into a later season first
    /// (`born_season < world.season`).
    #[test]
    fn newborn_cannot_reproduce_until_a_later_season() {
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 15.0);

        let events = step(&mut world, &config);
        assert!(
            events.births.is_empty(),
            "an organism born this season (born_season == world.season) must not reproduce yet"
        );

        world.season += 1;
        let events = step(&mut world, &config);
        assert!(
            !events.births.is_empty(),
            "once world.season has advanced past born_season, reproduction may proceed"
        );
    }

    /// Task 138 phase 0: an uninhabitable biome kills the population
    /// outright, with `DeathCause::Habitat`, before any energy formula
    /// runs — independent of `light`/`temperature`, which here are set to
    /// values that would otherwise net strongly positive energy.
    #[test]
    fn uninhabitable_biome_kills_regardless_of_scalars() {
        let (mut world, mut config) = world_with_one_organism(0.7, 0.5, 5.0);
        let (cx, cy) = (world.width / 2, world.height / 2);
        let idx = world.index(cx, cy);
        // `world_with_one_organism` forces `terrain` to `Plain` but doesn't
        // touch `Cell::biome` (classified independently at generation
        // time) — pin the exact biome under test rather than relying on
        // whatever this seed happened to classify at this cell.
        world.cells[idx].biome = Biome::Plain;
        config.energy.biome_habitable[Biome::Plain.index()] = false;

        let events = step(&mut world, &config);

        assert!(
            world.get(cx, cy).population.is_none(),
            "an uninhabitable biome must empty the cell regardless of scalars"
        );
        assert_eq!(events.deaths.len(), 1);
        assert_eq!(events.deaths[0].cause, DeathCause::Habitat);
        assert_eq!(
            events.extinctions.len(),
            1,
            "the only population of this species just died, so it must go extinct"
        );
    }

    #[test]
    fn crowded_photolithic_stalls_at_carrying_capacity() {
        // Task 136: `crowd_factor` now counts only occupied neighbours of a
        // *different* species (see `sim::step`'s cost section) — a
        // same-species cluster nets zero `interaction_delta` by construction
        // (the matrix diagonal is always zero) and would otherwise stall on
        // crowding alone at the retuned gains. A second, tagless species is
        // pushed here so the 7 neighbours are cross-species and still
        // exercise the carrying-capacity penalty this test is named for.
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        world.push_species(Species {
            name: "Neighbour".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: Vec::new(),
        });
        let (cx, cy) = (world.width / 2, world.height / 2);

        // Fill 7 of the 8 Moore neighbours with organisms of the second
        // species, far enough from repro_threshold that they don't
        // reproduce this tick.
        let neighbours: Vec<usize> = world.moore_neighbours(cx, cy).collect();
        for &idx in neighbours.iter().take(7) {
            world.cells[idx].population = Some(Population {
                species: SpeciesId(1),
                count: 1,
                energy: 1.0,
                born_season: 0,
                blocked: false,
            });
        }

        step(&mut world, &config);

        let organism = world.get(cx, cy).population.expect("organism survives");
        // gain 0.98 - upkeep 0.5 - crowding (7 * 0.15 = 1.05) = net -0.57.
        assert!(
            (organism.energy - 4.43).abs() < TOLERANCE,
            "expected net -0.57, got {}",
            organism.energy - 5.0
        );
    }

    #[test]
    fn photolithic_in_the_dark_eventually_dies() {
        // Task 136: gain 0.2 * 1.4 * 1 = 0.28 < upkeep 0.5 ⇒ net -0.22/tick,
        // doesn't survive. Starting energy is 5.0, so death takes ~23 ticks,
        // not the first one. Diffusion disabled (task 074): otherwise this
        // cell's forced-dark light blends back toward the ambient gradient
        // over the run, which eventually turns the net gain positive and
        // lets the organism survive/reproduce — a diffusion-timing artifact,
        // not the light niche this test means to isolate.
        let (mut world, mut config) = world_with_one_organism(0.2, 0.5, 5.0);
        config.environment.diffusion_rate = 0.0;
        let (cx, cy) = (world.width / 2, world.height / 2);

        step(&mut world, &config);
        let organism = world
            .get(cx, cy)
            .population
            .expect("survives the first tick");
        assert!(
            (organism.energy - 4.78).abs() < TOLERANCE,
            "expected net -0.22/tick in the dark, got {}",
            organism.energy - 5.0
        );

        for _ in 0..200 {
            if world.get(cx, cy).population.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).population.is_none(),
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
            .position(|c| c.population.is_none())
            .expect("an empty cell exists");
        world.cells[target].population = Some(Population {
            species: SpeciesId(1),
            count: 1,
            energy: config.energy.seed_energy,
            born_season: world.era,
            blocked: false,
        });
        step(&mut world, &config);
        assert_eq!(
            world.species_seeded_era[1],
            Some(3),
            "must record the era it was actually placed in, not registry-creation era 0"
        );

        let (cx, cy) = (world.width / 2, world.height / 2);
        for _ in 0..200 {
            if world.get(cx, cy).population.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).population.is_none(),
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
    fn season_advances_exactly_season_pulses_then_stops_at_observing() {
        let config = SimConfig::default();
        let (world, _) = world_with_one_organism(0.7, 0.5, 5.0);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.insert_resource(config.clone());
        app.insert_resource(world);
        app.init_state::<GameState>();
        app.add_sub_state::<EraState>();
        app.init_resource::<SeasonProgress>();
        app.init_resource::<ActionBudget>();
        app.init_resource::<EraTally>();
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
            .resource_mut::<SeasonProgress>()
            .start(config.time.season_pulses);
        app.world_mut()
            .resource_mut::<NextState<EraState>>()
            .set(EraState::Advancing);

        for _ in 0..config.time.season_pulses + 10 {
            app.update();
        }

        let sim_world = app.world().resource::<SimWorld>();
        assert_eq!(sim_world.tick, config.time.season_pulses as u64);
        assert_eq!(sim_world.season, 1);
        assert_eq!(
            sim_world.era, 0,
            "one season should not close an era on its own (seasons_per_era > 1)"
        );
        assert_eq!(
            *app.world().resource::<State<EraState>>().get(),
            EraState::Observing
        );
        assert_eq!(
            app.world().resource::<ActionBudget>().points_remaining,
            config.time.point_budget_per_season,
            "the budget should refill when the season ends"
        );

        for _ in 0..10 {
            app.update();
        }
        let sim_world = app.world().resource::<SimWorld>();
        assert_eq!(
            sim_world.tick, config.time.season_pulses as u64,
            "no extra ticks should run once the season has ended"
        );
    }

    /// Task 140's core acceptance criterion: an era that actually closes
    /// (not merely a season) halts the game in `EraState::Reveal`, not
    /// `Observing` — verified by running exactly `seasons_per_era` seasons'
    /// worth of ticks through the same `advance_tick`/`FixedUpdate`
    /// machinery `season_advances_exactly_season_pulses_then_stops_at_observing`
    /// exercises for a single season.
    #[test]
    fn era_closing_halts_in_reveal_not_observing() {
        let config = SimConfig::default();
        let (world, _) = world_with_one_organism(0.7, 0.5, 5.0);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.insert_resource(config.clone());
        app.insert_resource(world);
        app.init_state::<GameState>();
        app.add_sub_state::<EraState>();
        app.init_resource::<SeasonProgress>();
        app.init_resource::<ActionBudget>();
        app.init_resource::<EraTally>();
        app.init_resource::<PendingEvolutions>();
        app.init_resource::<EraReveal>();
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
        app.add_systems(OnEnter(EraState::Reveal), build_era_reveal);

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        app.world_mut()
            .resource_mut::<SeasonProgress>()
            .start(config.time.season_pulses);
        app.world_mut()
            .resource_mut::<NextState<EraState>>()
            .set(EraState::Advancing);

        // This minimal harness has no `input::start_era` system, so each
        // season past the first needs its own manual "player pressed space
        // again" — `SeasonProgress::start` plus flipping back to
        // `Advancing` — exactly like a real session's `space` key would,
        // just driven by the test instead of `ButtonInput`. Checked after
        // every single `app.update()` (not batched in blocks of
        // `season_pulses`): a state set via `NextState` only becomes
        // visible through `State::get()` starting the *next* `app.update()`
        // call, so batching the check would read a stale state on the
        // exact call where a season/era boundary lands.
        let mut seasons_started = 1;
        for _ in 0..(config.time.season_pulses * config.time.seasons_per_era) + 50 {
            app.update();
            match *app.world().resource::<State<EraState>>().get() {
                EraState::Observing if seasons_started < config.time.seasons_per_era => {
                    seasons_started += 1;
                    app.world_mut()
                        .resource_mut::<SeasonProgress>()
                        .start(config.time.season_pulses);
                    app.world_mut()
                        .resource_mut::<NextState<EraState>>()
                        .set(EraState::Advancing);
                }
                EraState::Reveal => break,
                _ => {}
            }
        }

        let sim_world = app.world().resource::<SimWorld>();
        assert_eq!(sim_world.era, 1, "exactly one era should have closed");
        assert_eq!(
            *app.world().resource::<State<EraState>>().get(),
            EraState::Reveal,
            "an era closing must halt in Reveal, not fall through to Observing"
        );
        let reveal = app.world().resource::<EraReveal>();
        assert_eq!(reveal.era, 1);
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
                population: Some(Population {
                    species,
                    count: 1,
                    energy,
                    born_season: 0,
                    blocked: false,
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
        let b = world.get(cx + 1, cy).population.expect("B survives");
        // Task 136: gain 0.7, upkeep 0.5, 1 occupied neighbour -> crowding
        // 0.15, interaction_delta -2 * interaction_scale 0.15 = -0.3:
        // net 5.0 + 0.7 - 0.5 - 0.15 - 0.3 = 5.03.
        assert!((b.energy - 5.03).abs() < TOLERANCE, "got {}", b.energy);
    }

    /// Task 136b: a persisting adjacency must not keep emitting
    /// `AdjacencyObserved` tick after tick — only its onset (the tick the
    /// tag first becomes adjacent) counts as evidence. `interaction_delta`
    /// itself is unaffected: it's an energy effect, not evidence, and must
    /// keep applying every tick regardless (checked via `b`'s energy delta
    /// being the same on both ticks).
    #[test]
    fn adjacency_evidence_fires_once_on_onset_not_every_tick() {
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };
        let (mut world, mut config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        // No conditional gating and no diffusion: this test isolates the
        // onset/energy behaviour, not terrain gates or environmental drift
        // (see `unconditional_tags_match_pre_096_formula` for the same
        // pattern).
        world.conditional_tags = Vec::new();
        config.environment.diffusion_rate = 0.0;

        let first = step(&mut world, &config);
        assert_eq!(
            first.adjacencies.len(),
            1,
            "the first tick this pair is adjacent must emit evidence"
        );

        let (cx, cy) = (world.width / 2, world.height / 2);
        let energy_after_first = world.get(cx + 1, cy).population.expect("B survives").energy;

        let second = step(&mut world, &config);
        assert!(
            second.adjacencies.is_empty(),
            "the same still-adjacent pair must not emit evidence again: {:?}",
            second.adjacencies
        );
        // `interaction_delta` is an energy effect, not evidence, so it must
        // keep applying every tick: without it B's isolated net would be
        // close to +0.48/tick (see `isolated_photolithic_grows`); with the
        // -2 interaction still in effect the second-tick delta should stay
        // far below that, not jump up just because evidence stopped firing.
        let energy_after_second = world.get(cx + 1, cy).population.expect("B survives").energy;
        let delta_second_tick = energy_after_second - energy_after_first;
        assert!(
            delta_second_tick < 0.2,
            "the negative interaction should still be dragging B down on the second \
             tick too, got delta {delta_second_tick}"
        );
    }

    /// Task 136b: a new exerter tag becoming adjacent (a second neighbour
    /// with a tag the receiver hasn't been next to before) is a fresh onset
    /// and must emit evidence, even though the *first* neighbour's tag —
    /// already adjacent since the previous tick — correctly stays silent.
    #[test]
    fn adjacency_evidence_refires_for_a_newly_adjacent_tag() {
        let matrix = TagMatrix {
            size: 3,
            values: vec![0, 0, -2, 0, 0, -2, 0, 0, 0],
        };
        let (mut world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(2)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.conditional_tags = Vec::new();
        let first = step(&mut world, &config);
        assert_eq!(first.adjacencies.len(), 1);

        // A third organism, carrying a different exerter tag (1) that also
        // harms receiver tag 2, placed newly adjacent to B.
        let (cx, cy) = (world.width / 2, world.height / 2);
        world.push_species(Species {
            name: "C".to_string(),
            metabolism: Metabolism::Photolithic,
            temp_optimum: 0.5,
            temp_tolerance: config.energy.default_temp_tolerance,
            repro_threshold: config.energy.repro_threshold,
            tags: vec![TagSlot(1)],
        });
        let c_idx = world.index(cx + 2, cy);
        world.cells[c_idx] = Cell {
            light: 0.7,
            temperature: 0.5,
            population: Some(Population {
                species: SpeciesId(2),
                count: 1,
                energy: 5.0,
                born_season: world.season,
                blocked: false,
            }),
            ..world.cells[c_idx]
        };

        let second = step(&mut world, &config);
        assert_eq!(
            second.adjacencies.len(),
            1,
            "the newly-adjacent tag (1) must emit evidence even though tag 0's \
             already-adjacent pair correctly stays silent: {:?}",
            second.adjacencies
        );
        assert_eq!(second.adjacencies[0].exerter_tag, TagSlot(1));
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
        let b = world.get(cx + 1, cy).population.expect("B survives");
        // Task 136: net 5.0 + 0.98 - 0.5 - 0.15 + 0.3 = 5.63.
        assert!((b.energy - 5.63).abs() < TOLERANCE, "got {}", b.energy);
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
        let b = world.get(cx + 1, cy).population.expect("B survives");
        // Task 136: net 5.0 + 0.7 - 0.5 - 0.15 + 0.0 = 5.33.
        assert!((b.energy - 5.33).abs() < TOLERANCE, "got {}", b.energy);
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
                population: Some(Population {
                    species,
                    count: 1,
                    energy,
                    born_season: 0,
                    blocked: false,
                }),
                ..world.cells[idx]
            };
        }
        step(&mut world, &config);
        let b = world.get(cx + 1, cy).population.expect("B survives");
        // Same result as negative_adjacency_effect_subtracts_energy: 5.03.
        assert!((b.energy - 5.03).abs() < TOLERANCE, "got {}", b.energy);
    }

    /// Task 138 caveat: `net_self_interaction == 0` (task 088's invariant on
    /// every drawn species' tag set) makes a same-species neighbour
    /// contribute exactly zero `interaction_delta` — *when both cells'
    /// `tag_gate_satisfied` verdicts agree*. Since task 096's conditional
    /// tags, that agreement isn't guaranteed: this pins the case where it
    /// doesn't hold, rather than letting the discrepancy surface later
    /// during balance work. Two organisms of the *same* species carry
    /// `{T0, T1}` with `matrix.get(T0, T1) = +2`, `matrix.get(T1, T0) = -2`
    /// (`net_self_interaction == 0`, satisfying the invariant a real drawn
    /// species would need). `T0` is `Mode::Inducible` on `Hill`; `T1` is
    /// unconditional. A sits on `Hill`, B on `Plain`, so the two cells'
    /// gate verdicts for `T0` disagree depending on which cell is asked to
    /// carry it as exerter vs. receiver — breaking the cancellation.
    #[test]
    fn same_species_neutrality_breaks_when_conditional_gates_disagree() {
        let conditional = ConditionalTag {
            tag: TagId(0),
            terrain: TerrainKind::Hill,
            mode: Mode::Inducible,
        };
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, 2, -2, 0],
        };
        assert_eq!(
            net_self_interaction(&matrix, &[TagSlot(0), TagSlot(1)]),
            0,
            "test setup: the tag set itself must satisfy the zero-self-interaction invariant"
        );

        let (mut world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0), TagSlot(1)],
            vec![TagSlot(0), TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        world.active_tags = vec![TagId(0), TagId(1)];
        world.conditional_tags = vec![conditional];
        let (cx, cy) = (world.width / 2, world.height / 2);
        let a_idx = world.index(cx, cy);
        let b_idx = world.index(cx + 1, cy);
        world.cells[a_idx].terrain = TerrainKind::Hill;
        world.cells[b_idx].terrain = TerrainKind::Plain;
        // Force both organisms to actually be the same species — the
        // helper gives each cell its own `Species` entry so their tag sets
        // can differ in general; here both are `[T0, T1]`, and this makes
        // it literally one species occupying both cells.
        world.cells[b_idx].population = Some(Population {
            species: SpeciesId(0),
            ..world.cells[b_idx].population.unwrap()
        });

        step(&mut world, &config);

        let a = world.get(cx, cy).population.expect("A survives");
        let b = world.get(cx + 1, cy).population.expect("B survives");
        // Same-species neighbours never count toward crowding (task 136:
        // `occupied_neighbours` filters to *different*-species only), so
        // this pair's baseline (no interaction, no crowding) is
        // 5.0 + gain(0.98) - upkeep(0.5) = 5.48 for both, not the
        // 5.33 a cross-species pair would see. Pre-096 (or with agreeing
        // gates), both would stay at that baseline: net effect zero on
        // each side. Here the gate disagreement leaves each side seeing
        // only one leg of the matrix pair instead of both cancelling: A
        // sees only `matrix.get(T1, T0) = -2` (T0 gates off as exerter
        // from B's Plain cell but on as receiver on A's own Hill cell), B
        // sees only `matrix.get(T0, T1) = +2` (the mirror asymmetry) —
        // nonzero and *different* on each side, despite
        // `net_self_interaction == 0`.
        assert!(
            (a.energy - 5.18).abs() < TOLERANCE,
            "expected A's same-species neighbour to cost -2*scale despite \
             net_self_interaction == 0, got {}",
            a.energy
        );
        assert!(
            (b.energy - 5.78).abs() < TOLERANCE,
            "expected B's same-species neighbour to add +2*scale despite \
             net_self_interaction == 0, got {}",
            b.energy
        );
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
        let b = world.get(cx + 1, cy).population.expect("B survives");
        assert!(
            (b.energy - 5.03).abs() < TOLERANCE,
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
        let b = world.get(cx + 1, cy).population.expect("B survives");
        assert!(
            (b.energy - 5.33).abs() < TOLERANCE,
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
        let b = world.get(cx + 1, cy).population.expect("B survives");
        assert!(
            (b.energy - 5.33).abs() < TOLERANCE,
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
        let b = world.get(cx + 1, cy).population.expect("B survives");
        assert!(
            (b.energy - 5.03).abs() < TOLERANCE,
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
        let b = world.get(cx + 1, cy).population.expect("B survives");
        assert!((b.energy - 5.03).abs() < TOLERANCE, "got {}", b.energy);
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
            population: Some(Population {
                species: SpeciesId(0),
                count: 1,
                energy,
                born_season: 0,
                blocked: false,
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
                population: Some(Population {
                    species,
                    count: 1,
                    energy,
                    born_season: 0,
                    blocked: false,
                }),
                ..world.cells[idx]
            };
        }

        step(&mut world, &config);

        let b = world.get(cx, cy).population.expect("B survives");
        // Task 136: net 5.0 + 0.98 - 0.5 - 0.15*2 + (-1 - 1) * 0.15 = 4.88.
        assert!((b.energy - 4.88).abs() < TOLERANCE, "got {}", b.energy);
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
            population: Some(Population {
                species: SpeciesId(0),
                count: 1,
                energy,
                born_season: 0,
                blocked: false,
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
            let alive = world.get(cx, cy).population.is_some();
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
            world.cells[idx].population = Some(Population {
                species: SpeciesId(1),
                count: 1,
                energy: 20.0,
                born_season: 0,
                blocked: false,
            });
        }

        step(&mut world, &config);

        let predator = world.get(cx, cy).population.expect("predator survives");
        // Task 136: drain = min(predator_drain_cap, available) * fit = 1.4
        // (fit=1.0, available huge), upkeep 0.7, 8 occupied neighbours ->
        // crowding 0.15*8=1.2: net 5.0 + 1.4 - 0.7 - 1.2 = 4.5.
        assert!(
            (predator.energy - 4.5).abs() < TOLERANCE,
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
            population: Some(Population {
                species: SpeciesId(1),
                count: 1,
                energy: 20.0,
                born_season: 0,
                blocked: false,
            }),
            ..world.cells[prey_idx]
        };
        for dx in [-1i32, 1] {
            let idx = world.index((cx as i32 + dx) as usize, cy);
            world.cells[idx] = Cell {
                light: 0.0,
                temperature: 0.5,
                population: Some(Population {
                    species: SpeciesId(0),
                    count: 1,
                    energy: 5.0,
                    born_season: 0,
                    blocked: false,
                }),
                ..world.cells[idx]
            };
        }

        step(&mut world, &config);

        let left = world
            .get(cx - 1, cy)
            .population
            .expect("left predator survives");
        let right = world
            .get(cx + 1, cy)
            .population
            .expect("right predator survives");
        assert!(
            (left.energy - right.energy).abs() < TOLERANCE,
            "left {} vs right {}",
            left.energy,
            right.energy
        );
        // Each predator's only prey neighbour is the shared one, so each
        // draws its full drain_cap (fit=1.0, prey has plenty of energy).
        // Task 136: net 5.0 + 1.4 - 0.7 - 0.15 = 5.55.
        assert!(
            (left.energy - 5.55).abs() < TOLERANCE,
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
            population: Some(Population {
                species: SpeciesId(0),
                count: 1,
                energy,
                born_season: 0,
                blocked: false,
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
        let organism = world
            .get(cx, cy)
            .population
            .expect("survives the first tick");
        assert!(
            (organism.energy - (config_seed_energy() - config.energy.decomposer_upkeep)).abs()
                < TOLERANCE,
            "expected net -upkeep/tick with no residue, got {}",
            organism.energy - config_seed_energy()
        );

        for _ in 0..200 {
            if world.get(cx, cy).population.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).population.is_none(),
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
            if world.get(cx, cy).population.is_none() {
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

        let decomposer = world.get(cx, cy).population.expect("decomposer survives");
        // Task 136: decay first: 10.0 - residue_decay(0.2) = 9.8 available;
        // drawn = min(decomposer_extract_rate(1.05) * fit(1.0), 9.8) = 1.05;
        // net 5.0 + 1.05 - decomposer_upkeep(0.5) = 5.55.
        assert!(
            (decomposer.energy - 5.55).abs() < TOLERANCE,
            "got {}",
            decomposer.energy
        );
        let (nx, ny) = (neighbour_idx % world.width, neighbour_idx / world.width);
        assert!(
            (world.get(nx, ny).residue - 8.75).abs() < TOLERANCE,
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
            population: Some(Population {
                species: SpeciesId(0),
                count: 1,
                energy,
                born_season: 0,
                blocked: false,
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
        let organism = world
            .get(cx, cy)
            .population
            .expect("survives the first tick");
        assert!(
            (organism.energy - (config_seed_energy() - config.energy.chemolithotroph_upkeep)).abs()
                < TOLERANCE,
            "expected net -upkeep/tick with no toxicity, got {}",
            organism.energy - config_seed_energy()
        );

        for _ in 0..200 {
            if world.get(cx, cy).population.is_none() {
                break;
            }
            step(&mut world, &config);
        }
        assert!(
            world.get(cx, cy).population.is_none(),
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

        let organism = world.get(cx, cy).population.expect("survives");
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
        let matched_energy = matched_world
            .get(cx, cy)
            .population
            .expect("survives")
            .energy;
        let mismatched_energy = mismatched_world
            .get(cx, cy)
            .population
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
        // Task 136: with `decomposer_extract_rate` at 0.6, each decomposer
        // draws its full rate only if at least 0.6 is available after decay
        // (residue_decay 0.2), and the two draws (1.2 total) must still
        // exceed what's left (residue 1.0 - decay 0.2 = 0.8) for this test
        // to exercise the over-draw clamp — a residue of 2.0 (right for the
        // old 1.5 extract_rate) would leave 1.8 available, under-drawn by
        // the new lower rate and never actually testing the clamp.
        // `residue_ambient_trickle` is zeroed for the same reason as
        // `decomposer_adjacent_to_residue_gains_and_residue_shrinks`: left
        // at its default, every background cell picks up a little residue,
        // which becomes an extra `source` diluting each decomposer's
        // proportional share of the one cell this test actually cares
        // about, so the combined draw no longer fully depletes it.
        let mut config = config;
        config.energy.residue_ambient_trickle = 0.0;
        let (cx, cy) = (world.width / 2, world.height / 2);
        let residue_idx = world.index(cx, cy);
        world.cells[residue_idx].residue = 1.0;
        for dx in [-1i32, 1] {
            let idx = world.index((cx as i32 + dx) as usize, cy);
            world.cells[idx] = Cell {
                light: 0.0,
                temperature: 0.5,
                population: Some(Population {
                    species: SpeciesId(0),
                    count: 1,
                    energy: 5.0,
                    born_season: 0,
                    blocked: false,
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
    fn cell_energy_breakdown_neighbour_lines_match_steps_own_interaction_delta() {
        // A strongly negative matrix entry (T0 exerted -> T1 received)
        // guarantees B starves this tick, so `step` reports its own
        // `interaction_delta` via `OrganismDied` to compare against.
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -20, 0, 0],
        };
        let (mut world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            1.0,
        );
        let (cx, cy) = (world.width / 2, world.height / 2);
        let b_idx = world.index(cx + 1, cy);

        let breakdown =
            cell_energy_breakdown(&world, &config, b_idx).expect("B is occupied before the tick");
        assert_eq!(breakdown.neighbours.len(), 1, "only T0 -> T1 is nonzero");
        let line = breakdown.neighbours[0];
        let expected_line =
            world.matrix.get(TagSlot(0), TagSlot(1)) as f32 * config.energy.interaction_scale;
        assert!(
            (line.contribution - expected_line).abs() < TOLERANCE,
            "got {}, expected {}",
            line.contribution,
            expected_line
        );
        assert_eq!(line.tag, world.active_tags[0]);

        let events = step(&mut world, &config);
        let death = events
            .deaths
            .iter()
            .find(|d| d.cell == b_idx)
            .expect("B starves this tick under a -20 matrix entry");
        let line_sum: f32 = breakdown.neighbours.iter().map(|l| l.contribution).sum();
        assert!(
            (line_sum - death.interaction_delta).abs() < TOLERANCE,
            "breakdown lines ({line_sum}) must sum to step's own interaction_delta ({})",
            death.interaction_delta
        );
    }

    #[test]
    fn phase_0_single_species_worlds_still_have_zero_interaction_delta() {
        // Regression guard: existing single-species tests build worlds with
        // no tags at all, so the matrix wiring must not change their result.
        let (mut world, config) = world_with_one_organism(0.7, 0.5, 5.0);
        step(&mut world, &config);

        let (cx, cy) = (world.width / 2, world.height / 2);
        let organism = world.get(cx, cy).population.expect("organism survives");
        // Task 136: same net as isolated_photolithic_grows, +0.48.
        assert!(
            (organism.energy - 5.48).abs() < TOLERANCE,
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
            population: Some(Population {
                species: SpeciesId(0),
                count: 1,
                energy: 0.05,
                born_season: 0,
                blocked: false,
            }),
            ..world.cells[dying_idx]
        };
        let (sx, sy) = (world.width - 2, world.height - 2);
        let surviving_idx = world.index(sx, sy);
        world.cells[surviving_idx] = Cell {
            light: 0.7,
            temperature: 0.5,
            population: Some(Population {
                species: SpeciesId(0),
                count: 1,
                energy: 5.0,
                born_season: 0,
                blocked: false,
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
        assert!(world.get(1, 1).population.is_none());
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

    /// Task 146: culling the exerter (A) must produce the same-shaped
    /// `AdjacencyObserved` a live tick would (same fixture as
    /// `adjacency_between_tagged_organisms_produces_expected_observation`),
    /// with B — the surviving neighbour — as receiver, since the knockout
    /// records A's effect on B, not the reverse.
    #[test]
    fn cull_emits_the_same_shaped_knockout_observation_as_the_tick_loop() {
        let matrix = TagMatrix {
            size: 2,
            values: vec![0, -2, 0, 0],
        };
        let (world, config) = world_with_two_neighbours(
            matrix,
            vec![TagSlot(0)],
            vec![TagSlot(1)],
            0.7,
            0.5,
            5.0,
            5.0,
        );
        let (cx, cy) = (world.width / 2, world.height / 2);

        let observations = cull_knockout_observations(&world, &config, cx, cy);

        assert_eq!(observations.len(), 1);
        let obs = observations[0];
        assert_eq!(obs.receiver_species, SpeciesId(1));
        assert_eq!(obs.exerter_tag, TagSlot(0));
        assert_eq!(obs.receiver_tag, TagSlot(1));
        assert_eq!(
            obs.n_confounders, 0,
            "B's only neighbour is the culled A, carrying only the exerter tag"
        );
    }

    /// Task 146: a culled organism with no living neighbours must emit
    /// nothing — no panic, no zero-weight noise.
    #[test]
    fn cull_on_an_isolated_organism_emits_nothing() {
        let (world, config) = world_with_one_organism(0.0, 0.5, 5.0);
        let (x, y) = (world.width / 2, world.height / 2);

        let observations = cull_knockout_observations(&world, &config, x, y);

        assert!(observations.is_empty());
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
                population: Some(Population {
                    species,
                    count: 1,
                    energy: 5.0,
                    born_season: 0,
                    blocked: false,
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
            0.0,
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
            0.0,
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
            // this cell's toxicity before the accumulator reads it. Task
            // 136 also pins light/temperature now — the lower metabolism
            // gains leave much less margin above upkeep, so `threshold 80.0`
            // needs ~80 ticks to cross and diffusion's slow drift away from
            // the pinned optimum, previously negligible next to the old
            // +0.9/tick margin, was enough to starve the organism out well
            // before then.
            world.cells[idx].toxicity = 1.0;
            world.cells[idx].light = 0.7;
            world.cells[idx].temperature = 0.5;
            let events = step(&mut world, &config);
            if let Some(event) = events.selection_thresholds.into_iter().next() {
                crossed = Some(event);
                break;
            }
            if world.get(cx, cy).population.is_none() {
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
        world.cells[idx].population = Some(Population {
            species: SpeciesId(0),
            count: 1,
            energy: 5.0,
            born_season: 0,
            blocked: false,
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
        assert_eq!(world.cells[idx].population.unwrap().species, new_id);
        // Task 109: backs `Objective::Speciation`.
        assert!(world.has_speciated);
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
        // Task 109: a no-op attempt must never set `has_speciated`.
        assert!(!world.has_speciated);
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
    /// parent — the full pure-Rust pipeline `sim::build_era_reveal` wires
    /// together, without needing a running `App`/GUI to confirm it.
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

    #[test]
    fn species_still_extant_is_false_once_every_cell_of_that_species_is_empty() {
        let (mut world, _config, idx) = world_for_speciation();
        assert!(species_still_extant(&world, SpeciesId(0)));
        world.cells[idx].population = None;
        assert!(!species_still_extant(&world, SpeciesId(0)));
    }

    #[test]
    fn any_evolution_maturing_only_true_past_the_fraction_and_never_once_crossed() {
        let config = SimConfig::default();
        let mut world = SimWorld::new(42, &config);
        let threshold = config.evolution.selection_pressure_threshold;

        assert!(
            !any_evolution_maturing(&world, &config.evolution),
            "no species has accrued any pressure yet"
        );

        world.selection_pressure = vec![SelectionPressure {
            toxicity: threshold * 0.5,
            ..Default::default()
        }];
        assert!(
            !any_evolution_maturing(&world, &config.evolution),
            "below MATURING_HINT_FRACTION of the threshold"
        );

        world.selection_pressure[0].toxicity = threshold * 0.9;
        assert!(
            any_evolution_maturing(&world, &config.evolution),
            "past MATURING_HINT_FRACTION of the threshold"
        );

        world.selection_pressure[0].crossed = true;
        assert!(
            !any_evolution_maturing(&world, &config.evolution),
            "a species that already crossed is done maturing, not still maturing"
        );
    }

    /// Task 140's core deferred-application behavior: `build_era_reveal`
    /// (not `step`/`buffer_pending_evolutions`) is the only place a buffered
    /// `SelectionThresholdCrossed` actually turns into a new species —
    /// exercised here as the plain Bevy system it is, via a minimal `App`,
    /// same pattern `a_real_threshold_crossing_from_step_produces_a_descendant_species`
    /// uses for `speciate` itself.
    #[test]
    fn build_era_reveal_applies_pending_evolutions_and_reports_the_tier() {
        let (world, config, idx) = world_for_speciation();
        let event = toxicity_dominant_event(SpeciesId(0), idx);

        let mut app = App::new();
        app.insert_resource(config);
        app.insert_resource(world);
        app.insert_resource(PendingEvolutions(vec![event]));
        app.init_resource::<EraTally>();
        app.init_resource::<EraReveal>();
        app.add_message::<SpeciesEvolved>();
        app.add_systems(Update, build_era_reveal);

        // Before `build_era_reveal` runs: still one species, buffered.
        assert_eq!(app.world().resource::<SimWorld>().species.len(), 1);

        app.update();

        let world = app.world().resource::<SimWorld>();
        assert_eq!(
            world.species.len(),
            2,
            "the pending crossing must be applied once the era closes"
        );
        assert!(
            app.world().resource::<PendingEvolutions>().0.is_empty(),
            "pending evolutions must be drained, not left to reapply next era"
        );
        let reveal = app.world().resource::<EraReveal>();
        assert_eq!(reveal.tier, RevealTier::Epochal);
        assert_eq!(reveal.evolutions.len(), 1);
        assert_eq!(reveal.evolutions[0].parent, SpeciesId(0));
        assert_eq!(reveal.evolutions[0].child, SpeciesId(1));
        assert_eq!(reveal.evolutions_lost, 0);
    }

    /// Task 140's adopted answer to "what if the species goes extinct before
    /// its evolution matures": the pending crossing is dropped, counted as
    /// `evolutions_lost`, not applied — and with nothing else notable this
    /// era, the tier reads `Notable` (an extinction/loss), not `Epochal`.
    #[test]
    fn build_era_reveal_drops_a_pending_evolution_for_an_extinct_species() {
        let (mut world, config, idx) = world_for_speciation();
        let event = toxicity_dominant_event(SpeciesId(0), idx);
        world.cells[idx].population = None; // the species has since gone extinct

        let mut app = App::new();
        app.insert_resource(config);
        app.insert_resource(world);
        app.insert_resource(PendingEvolutions(vec![event]));
        app.init_resource::<EraTally>();
        app.init_resource::<EraReveal>();
        app.add_message::<SpeciesEvolved>();
        app.add_systems(Update, build_era_reveal);

        app.update();

        assert_eq!(
            app.world().resource::<SimWorld>().species.len(),
            1,
            "an extinct species' pending evolution must not be applied"
        );
        let reveal = app.world().resource::<EraReveal>();
        assert!(reveal.evolutions.is_empty());
        assert_eq!(reveal.evolutions_lost, 1);
        assert_eq!(reveal.tier, RevealTier::Notable);
    }
}
