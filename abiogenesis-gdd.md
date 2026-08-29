# Abiogenesis — Game Design Document

**Working title:** Abiogenesis *(provisional, to be confirmed)*
**Genre:** Emergent-simulation roguelike / xenobiology lab
**Platform:** Desktop, 2D graphical window
**Tech:** Rust + Bevy (ECS)
**Mode:** Single-player, era-based, with objectives and light meta-progression
**Document status:** v0.7 — design-pass alignment: time scale, actions, balance, objectives, notebook

> Legend for the status of each decision:
> **[DECIDED]** agreed and stable · **[PROPOSED]** baseline I'm proposing, to be approved/corrected · **[OPEN]** to be decided together

### Changelog

- **v0.7** — **Design-pass alignment.** A round of design work produced revisions across several systems, plus corrections where this document had drifted from the build. **Time (§4):** a three-level scale — pulse → **season** → era — where the *season* becomes the player's unit of decision (the action budget refills per season) and the era becomes the unit of *narration*, much longer and closed by a dedicated reveal beat. True real-time is now explicitly **rejected** rather than deferred; a pausable *continuous advancement* is adopted instead. **Actions (§6):** the "in an area" prose is corrected to per-cell, matching §11 and the build; `Splice` is clarified as *synthesising a new species* into the seedable roster rather than editing a live one; `Stress` gains a selectable axis (thermal/light/toxicity); `Cull` is reframed as a knockout experiment that feeds the notebook. **Balance (§5.9):** metabolic gains cut so that environmental fitness alone yields near-breakeven — the hidden matrix, not the environment, decides growth or decline. **Objectives (§8):** clearing a world's sequence now sets a *victory flag* instead of force-ending the world; objectives measure from activation rather than from world state; five new objective kinds. **Notebook (§7):** relationship graph replaces the matrix grid, plus a fourth section (Chronicle). Design work also produced several **[PROPOSED]** systems not yet decided — trait families, dynamic biomes, emergence, world events — flagged as such throughout. Companion design documents hold the detail; open questions are tracked in `abiogenesis-open-points.md`.
- **v0.6** — **"Mondo vivo," biomes, and evolution land (tasks 096-118).** §5.2/§5.4/§5.6 document `Chemolithotroph`, the fourth metabolism, now reading `toxicity` as a live input (the v0.5 "declared but inert" caveat no longer applies). New §5.10 documents biomes (areal classification layered on the scalar grid) and §5.11 documents evolution by speciation from accumulated selection pressure, including the always-final `Speciation` long-term objective (§8). §5.5/§9 document terrain-conditional tags and wild (pre-placed) species — a narrow, documented exception to the "nothing is auto-placed" rule. §4/§11/§15 rename the player-facing time unit from "tick" to "pulse" (task 118); internal/mechanical uses of "tick" (§5.6, §5.9) are unchanged by design — see the note in §15. §5.9 gains rows for the new metabolism and evolution coefficients.
- **v0.5** — **Alignment with Phase 3 ("the run") and two rounds of playtest tuning (tasks 035–059).** §5.6 makes explicit that `env_fit` gates all three metabolisms' gain, not just Photolithic's. §5.9 era budget updated to its retuned value. §8 documents the sequential per-world objectives (2→3), the world-level (not run-level) retry on total extinction, and the removal of auto-placed starting species. §5.2 flags `toxicity` as a declared-but-currently-inert scalar. None of this changes the pillars or the core mechanical model — it's the numeric baseline and a few rules catching up to what playtesting settled on.
- **v0.4** — **Stack change: from terminal (`ratatui`) to a 2D graphical window with Bevy and an ECS model.** This drives revisions to §5.1 (grid size), §11 (presentation), and §12 (stack and architecture), plus a correction to §13 on the scaffold's status. **The design itself does not change:** §§1–10 and §§14–16 — pillars, core loop, simulation model, tick formulas, numeric baseline §5.9, notebook, objectives, playthrough example — remain valid word for word. Pillar 3 ("the fun is in the system, not the graphics") remains fully in effect: colored squares, zero art assets.
- **v0.3** — Closed design decisions, numeric baseline (§5.9), playthrough example (§16).

---

## 1. Vision

You are a xenobiologist seeding life on alien worlds, tasked with discovering, through experiments, what biochemistry emerges from it. Each world has **hidden, different biochemical interaction rules**: the game consists of reverse-engineering those rules by seeding organisms, watching an ecosystem live its own life, forming hypotheses, and testing them with targeted interventions — all in pursuit of the objectives each world sets.

The central pleasure is the double mystery: the mystery of **what will happen** (the dynamic emergence of an unpredictable ecosystem) layered over the mystery of **the rules themselves** (deducing the secret biochemical matrix). It's Liu Cixin in the shape of a Petri dish.

### Design pillars **[DECIDED]**

1. **Unpredictability born from my choices.** Evolution isn't scripted: it emerges from the player's decisions crossed with hidden rules. No two playthroughs are alike.
2. **Discovery as progression.** You don't advance by accumulating numbers, but by *understanding*. The notebook filling up is the progress bar.
3. **The fun is in the system, not the graphics.** Minimal presentation (colored squares, zero art assets), depth in the simulation.
4. **Replayability from constrained procedural generation.** Each world's rules are generated, not hand-written: infinite content without months of authoring.
5. **A constant sense of wonder and discovery [PROPOSED — see `culture-shock-wonder.md`].** Not only the central mystery of the matrix — the game should always be signalling, through many small and a few large touches, that *there is more here to see*. Curious micro-events, strange biochemistry, rare anomalies that reward exploring rather than only decoding. The standard to hold every new proposal against, small or large: does this give the player another reason to think "I want to see what else is out there," or is it only functional? A filter to apply to every future addition, not a one-time content list. **Held in permanent tension with the spectator test (§system-hierarchy):** wonder content must never crowd out genuine play. Most of it should remain traceable to a past player choice or open a new decision (a threat to react to, a consequence of where something was seeded); a small deliberately spectator-only class (an unexplained silence, an unclaimed derelict) is allowed precisely because this pillar calls for wonder without a lever too — but that class must stay exceptional, or an accent becomes a pattern that starts displacing the game itself.

### What it is NOT **[DECIDED]**

- Not a clicker/idle game with ever-growing numbers.
- Not a purely contemplative sandbox: there is pressure, objectives, victory and defeat. *(We explicitly chose the objectives-driven version over the zen version.)*
- Not a graphically driven game.

---

## 2. The player and the fantasy

**Title [DECIDED]: `Culture Shock`** — subtitle *A sterile world*. "Abiogenesis" was the working title.

**Identity [PROPOSED — see `culture-shock-identity.md`].** "Liu Cixin in a Petri dish" is good *positioning* — it says what the game resembles — but it isn't an identity, which says what the game *is*. Read back from the decisions actually taken rather than the intentions stated, something more specific emerges: **this is a game where the mechanics *are* epistemology**, not a game with a scientific theme layered on top. Environmental fitness alone can't grow a population, so understanding is the only route to success and growth *is* the evidence you understood. Isolated observation outweighs confounded observation, making experimental control a resource. The matrix is re-rolled every world, so generalising from past runs is actively punished. Causal ambiguity is a legitimate output — the game will say "unclear" rather than invent a cause. `Splice` reaches only what you've confirmed: you cannot build what you haven't understood. The implicit antagonist worth making explicit: **games that reward optimisation without comprehension.** The identity is conceptually strong but still weakly *felt* — the player lives all of the above without ever hearing it named, which is what the post-MVP declared-hypothesis mechanic is for.

**Fantasy:** "I'm a scientist facing an alien biochemistry I don't understand, and I decode it by cultivating it." Curiosity, the scientific method, that moment when a hypothesis is confirmed and a piece of the system lights up.

**Target player:** people who love deduction puzzles, emergent simulation (Dwarf Fortress in miniature), hard SF, systems optimization. People who find it hypnotic to watch a self-organizing system.

---

## 3. Core loop **[DECIDED]**

The fundamental cycle, repeated within each world:

1. **Seed** — place organisms on the grid (what, where, when).
2. **Advance time** — the simulation proceeds by *N* pulses (a season, or a whole era); watch the ecosystem move and evolve.
3. **Record** — note in the notebook what you observed (who bloomed, who collapsed, which adjacencies seem to have an effect).
4. **Hypothesize & intervene** — form a hypothesis about the hidden rules and put it to the test with a targeted intervention (environmental stress, removal, mutation, new seeding).
5. **Objective** — once the world's objective is met, move to the next world, deeper and more hostile.

The experiential fulcrum is the sub-loop **hypothesis → experiment → *aha***, embedded in the unpredictability of an ecosystem that lives on its own.

---

## 4. Time model: pulse → season → era **[structure DECIDED / durations to be tuned]**

Time runs on **three levels**, so that the unit the player *decides* in and the unit the world *narrates* in are no longer forced to be the same thing:

- **Pulse** — the atomic step of advancement (the player-facing name for a tick, task 118).
- **Season [PROPOSED]** — a block of pulses. **The player's unit of decision:** the action budget refills per season, and the player's plan-act-observe cycle happens at this scale.
- **Era** — a block of seasons, **considerably longer than the old 25-pulse baseline**. The unit of *narration*: the scale at which the world takes stock and tells the player what happened (see the reveal beat below).

**Why the split.** Eras aren't only time — they're *decisions*: the action budget is spent per era in the old model, and the design deliberately made the time model coincide with the mental loop. Simply lengthening the era without moving the budget would lower the density of decisions per unit of time and make the game more passive; keeping the old 60→45 era budget alongside much longer eras would blow up a world's duration. Moving the *decision* to the season and leaving *narration* to the era solves both: decisions per world stay in the same order of magnitude, total duration stays comparable, and the era becomes rare and heavy enough that a per-era reveal reads as a moment rather than as noise.

- **Animation [DECIDED]:** advancement is shown pulse-by-pulse (fast), preserving the feeling of a "breathing system," while *control* remains deliberately step-wise.
- **Durations [to be tuned]:** `ERA_TICKS = 25` is the current implemented default and no longer reflects the intended shape. Season length, era length (in seasons), and the per-world era budget must be retuned together — see §5.9. Indicatively, if an era is ~4 seasons, the 60→45 era budget drops to roughly **15→11**.
- **End-of-era reveal [PROPOSED]:** each era closes with a dedicated beat — the simulation stops on its own and presents what happened, at one of three tiers of significance (minor / notable / epochal). Any evolution matured during the era is applied at this moment, which turns the wait for the era's end into anticipation rather than dead time. Text for the beat is generated procedurally, not authored — see the narrative-generation design document.
- **Continuous advancement [PROPOSED]:** a pausable toggle that advances pulses automatically. With long eras this is an **ergonomic requirement**, not a convenience — without it the player would hammer a key to cross a season. A companion control, "advance to the next notable event," lets the player skip forward until something above a chosen significance tier occurs.
- **True real-time [REJECTED]:** time advancing *while the player plans and acts* is explicitly **not** being built. It would contradict "one unit of time = one deliberate experiment," make the per-season budget hard to even read, and break the reveal beats that must halt the action. This closes what v0.6 listed as `[OPEN / future]`.

---

## 5. Simulation model

**Population model [PROPOSED — see `culture-shock-population-model-aesthetic.md`].** A cell is proposed to hold a **population** of one species (count + aggregated energy, up to a carrying capacity) rather than a single optional organism. Per-capita gain/upkeep keep the coefficients already retuned (§5.9) unchanged; reproduction becomes continuous growth; **overflow** past capacity spills into an adjacent empty or same-species cell — never into a cell held by another species, so cells stay single-species. Matrix interaction counts **by presence, not by quantity** (a neighbouring population with a trait contributes once, regardless of size) specifically so the §5.9 retuning survives unchanged. A saturated cell with nowhere to overflow feeds local selection pressure under the existing "environmental mismatch" stimulus (§5.11) — a real cause for a stimulus the system already had, not a new one. `Cull` now clears an entire local population rather than one organism, reads as a cleaner knockout. Clean-observation weighting (§7) barely changes: "isolated" again means neighbouring cells with a different trait, just counted per cell-population instead of per organism. Rendering follows directly: one shape per cell-population (no arbitrary crowding threshold needed, since the aggregate *is* the datum), and the overview becomes a direct read of real density (population ÷ capacity) instead of an artistically drawn approximation.

### 5.1 Grid and cells **[DECIDED]**

2D grid of cells. Each cell contains:

- an **environmental layer** (a few continuous scalars);
- at most **one organism** (single occupancy per cell), with an energy/population level.

**Size [DECIDED]:** **128×80**, as a configuration constant (raised from the original `48×32` baseline by task 074). Emergence needs room: spatial patterns (Lotka–Volterra-like) need breathing space, and grids that are too small die from stochastic noise. *(This document's §14/§16 examples and some older prose below still reference `48×32` — a documentation lag, not a live decision; the config default and every current worldgen/balance test run at `128×80`.)*
**Neighborhood [DECIDED]:** Moore (8 neighbors) for interactions and reproduction.

### 5.2 Environmental layer **[structure DECIDED / parameters in §5.9]**

A few scalars per cell, in `[0,1]`:

- `temperature`
- `light`
- `toxicity`

**Current status:** `toxicity` is a live input. Toxic zones exist in worldgen, the Stress action's target scalar can raise it, and it now directly drives one metabolism's gain (`Chemolithotroph`, §5.4/§5.6) as well as one of the three stimuli the evolution system (§5.11) accumulates toward speciation. `temperature`, `light`, and `toxicity` are all read in the tick loop today.

**Phase 0:** static gradients (e.g., high light at the top, temperature on a different axis) to create spatial heterogeneity → niches.
**Phase 1+:** slow diffusion of scalars (averaging with neighbors at a low rate), so environmental interventions propagate over time.

### 5.3 Species genome **[DECIDED]**

Each species is defined by a small genome:

- **Metabolism** (one of the types below) — how it derives energy.
- **Preferred environmental range** — e.g., `temp_optimum` + `temp_tolerance` (Gaussian fitness around the optimum).
- **Reproduction threshold** — energy above which it reproduces.
- **1 to 3 biochemical tags** — *the only thing that matters for interactions between species.*

Metabolisms and environmental ranges are **readable** (anchors for the player). Tags are **opaque** (see §5.5).

### 5.4 Metabolisms **[DECIDED]**

- **Photolithic** (`Photolithic`) — derives energy from local `light`. The primary producer.
- **Predator** (`Predator`) — derives energy from neighboring organisms (consumes their energy).
- **Decomposer** (`Decomposer`) — derives energy from dead matter / residue.
- **Chemolithotroph** (`Chemolithotroph`, task 108) — derives energy from local `toxicity`, the same role `light` plays for Photolithic. Its niche is the inverse of everyone else's: cells too toxic for other metabolisms to tolerate are where it thrives.

### 5.5 Tags and the hidden matrix **[DECIDED]** — *the heart of the game*

Two levels are distinguished, so as not to conflate variety and difficulty:

- **Global tag pool [DECIDED: 10 / PROPOSED: 15]:** ~**10 glyphs** total biochemical tags in the game, to give visual variety between worlds. *A design pass proposes raising this to **15** (a launch pool, with a further 10 documented in reserve). Pool size does **not** affect the difficulty of any single run — only how much two different worlds resemble each other. At 10, a late world with 8 active tags covers 80% of the pool; at 15 it covers ~53%, leaving real room for two late worlds to feel chemically distinct. See the trait-archetypes design document.*
- **Active tags per world [DECIDED]:** only a subset is actually in play in a given world. **Difficulty grows by increasing active tags, not the pool.** Baseline: **5 active tags** in the first worlds, up to **~8** in late worlds. *A design pass proposes **[PROPOSED, to validate in playtest]** softening world 0 to 4, raising the late ceiling to 9, and making the ramp gradual (4→5→6→7→8→9) rather than two steps — but this is the one genuinely tuned difficulty lever in the game, so it warrants more caution than the pool change above.*

**Named trait archetypes [PROPOSED]:** the nameless glyphs can be replaced with real biochemical terms (chitinous wall, ionic pore, quorum pheromone, structural prion, catalytic ribozyme…), each with a three-letter code in the style of real gene/protein abbreviations, grouped into five **families** (structural, metabolic, signalling, genetic, storage). The naming rule is strict: a name describes what something *is or does structurally*, **never** whether its effect is good or bad — the matrix stays independent of the name. Real names carry real-world associations, but that is a feature here rather than a leak: worlds are alien, chemistries differ, and the matrix is re-rolled every world, so a player who assumes a prion behaves as it does on Earth gets corrected by the evidence — reinforcing the game's central epistemic lesson instead of undermining it. See the trait-archetypes design document.

**Dominant family per world [PROPOSED]:** each world draws a dominant trait family from its seed, which biases **the intensity distribution of that family's matrix relationships** toward the extremes (`±2` likelier than `±1`) rather than biasing which traits get selected — selection bias loses its grip once a world activates most of the pool, whereas intensity bias reads identically at 5 or 9 active traits. The dominant family is never disclosed; the player infers it from play. It never biases the *sign* of effects, only how sharp they are.

**Xenotraits [PROPOSED]:** a second, rarer set of traits with a different structural rule — they do not exist in known chemistry, they are **never player-placeable** (not by seeding, not by `Splice`), and they can only enter a world as a rare alternative outcome of a speciation event (§5.11). They participate in the same hidden matrix as ordinary traits. In a world where evolution never matures, the player may encounter none at all: the rarity is the point.

Each species carries 1–3 tags (from those active in the world).

**Terrain-conditional tags [DECIDED, task 096]:** a small subset (~1-2 per world) of the active tags are *conditional*: induced or repressed depending on the biome (§5.10) the organism currently occupies, rather than fixed for the species' whole lifetime. A conditional tag's matrix effect only applies while its gating condition holds, adding a spatial dimension to an already-directional mystery — the same species pair can interact differently depending on where on the map they meet.

At the start of each world, a **secret `tag × tag` matrix** is rolled: for each ordered pair of tags, an effect (positive/negative, with intensity) that applies when two organisms carrying those tags are **adjacent**.

- **Directional effects [DECIDED]:** the matrix is **asymmetric** — "A poisons B" doesn't imply "B poisons A." So off-diagonal relationships are ~**T²**: with 5 active tags ≈ **20 relationships** (decodable in one run); with 8 ≈ **56** (too many to decode *all* of them, but that's not needed — the player only needs the part relevant to the objective and the species in play). It's a directional generalization of rock-paper-scissors: who catalyzes whom, who poisons whom.
- Tags are shown as **nameless alien glyphs/colors**: the player learns their effect only empirically.
- The matrix is **the thing the player decodes** through experiments. It's the mystery of the rules.
- **Degree of opacity [DECIDED]:** the matrix starts hidden, but **is progressively revealed** as the notebook confirms relationships (see §7). Metabolisms and environmental ranges always remain readable as anchors. The notebook's progressive revelation *is* the solution to the opacity — these aren't two separate mechanics.

### 5.6 Tick algorithm **[structure DECIDED / coefficients to be tuned]**

The **structure** is decided; the **numeric coefficients** are a baseline to validate in playtesting (tuning is the real work — see §13). Two structural decisions, which are *design* choices, not tuning:

- **Additive and linear matrix effect [DECIDED]:** the adjacency effect is **additive and independent for each adjacent pair** (every A-next-to-B = a fixed ±k, linear in the number of neighbors), **not multiplicative**. Multiplicative would be more "realistic" but couples the effects and makes deduction nearly impossible; additive is *readable* — the player can reason "each A next to me costs about 2 energy." The design serves deduction.
- **Centralized coefficients [DECIDED]:** all coefficients are named constants in a single place (or config file, ideally hot-reloadable), so final tuning is fast.


For each occupied cell:

1. **Environmental fitness:** `env_fit = gaussian(temperature, temp_optimum, temp_tolerance)` ∈ `[0,1]`.
2. **Metabolic gain** (depends on metabolism). `env_fit` gates all four the same way — it multiplies the raw gain, not just Photolithic's — so an organism can be sitting on abundant fuel (light, prey, residue, toxicity) and still starve if the cell's temperature is far from its optimum:
   - *Photolithic:* `gain = light * metabolism_gain * env_fit`.
   - *Predator:* draws energy from occupied neighbors (within a cap), weighted by `env_fit`.
   - *Decomposer:* draws from residue/dead matter in the cell or its neighbors, weighted by `env_fit`.
   - *Chemolithotroph:* `gain = toxicity * chemolithotroph_metabolism_gain * env_fit`.
3. **Hidden matrix effect:** for each occupied neighbor, for each tag pair (mine × theirs), sum the effect from the secret matrix → `interaction_delta` (can be + or −).
4. **Costs:** `upkeep` (base cost per tick) + `crowding_penalty = crowd_factor * n_occupied_neighbors` (carrying capacity).
5. **Energy update:** `energy += gain + interaction_delta − upkeep − crowding_penalty`.
6. **Death:** if `energy <= 0` → the organism dies (the cell frees up; optional: leaves residue for decomposers).
7. **Reproduction:** if `energy >= repro_threshold` and an empty neighbor exists → spawn a child in an empty neighbor (seeded random choice) with `repro_cost` energy, subtracted from the parent. *(Future phase: possible mutation of the child's genome.)*

**Processing order [PROPOSED]:** iteration in shuffled (seeded) order with a "born/acted this tick" guard so newborns don't act in the same tick; or double buffering (snapshot → next). To be chosen at implementation time, favoring correctness and determinism.

**Design principle for tuning these coefficients [PROPOSED, see §5.9]: environmental fitness alone must not be enough to grow.** With the v0.6 baseline, an organism placed by reading only the *visible* layer (temperature, light, toxicity) reaches the reproduction threshold in ~6 ticks without the hidden matrix ever mattering — which makes the game's central mystery *optional* for basic success. The correction is to tune metabolic gain so that a well-placed but isolated organism sits at near-breakeven: it survives, it doesn't reproduce. A **positive** matrix interaction supplies the margin that makes growth possible; a **negative** one pushes from breakeven into decline. This makes growing a population *itself* evidence that the player understood something real about the world, and produces trial-and-error by construction rather than by an imposed rule.

**Extension: the tick is a pipeline with three outputs, not one formula [PROPOSED].** This algorithm predates biomes (§5.10), terrain-conditional traits (§5.5), and evolution by speciation (§5.11), and it is written as though a tick produced only updated energy. It in fact has to produce three things, all fed by the same intermediate values: **updated energy**; **accumulated selection pressure** (§5.11 — whose three stimuli are values already computed here); and **emitted observations and event candidates** (needed by the notebook, by the reveal ranking, and by `Cull`-as-knockout). Left as separate systems these recompute the same quantities in three places and can diverge — the exact incoherence risk already flagged for narrative generation. Restructuring the tick as an explicit phase pipeline that carries its intermediates forward removes it by construction.

**Where biome enters — as constraint and cost, never as gain.** A biome *is already* a combination of scalars (a swamp is high toxicity, a desert is high light and heat), so multiplying gain by a biome factor would double-count the same information and make the two impossible to tune independently. But three things are genuinely *not* reducible to the scalars and do belong in the formula: a **habitability gate** (deep water isn't "dim and cold" — it's somewhere a terrestrial organism cannot be, a binary constraint with no current expression); **`crowd_factor` per biome** (a forest should carry more density than a peak); and **residue decay rate per biome** (still water retains, a slope disperses — which finally gives certain biomes a mechanical reason to be decomposer niches). All three are constraints or costs, so none double-counts the scalars.

**Selection pressure stays a separate accumulator, deliberately.** Energy and pressure could be merged, but they are two clocks with different meanings — energy is short-term survival, pressure is long-term adaptation. Keeping them distinct is what lets an organism thrive *while* accumulating pressure, or the reverse, which is the interesting tension. They share inputs; they must not share a value.

See `abiogenesis-tick-pipeline.md` for the phase-by-phase specification, including the hook point for `Isolate` (§6) and the intermediate values each phase must carry forward.

### 5.7 Determinism **[DECIDED]**

The simulation is **deterministic** given the same seed: seeded RNG kept in the world state. Essential for debugging emergence, reproducing bugs, and (down the line) sharing interesting seeds.

### 5.8 Anti-degeneration **[DECIDED]** — *the defense against the main risk*

The number-one risk of emergence is collapsing into two boring outcomes: **"everything dies"** or **"one species dominates."** Systemic levers to avoid them:

- **Cyclicity constraint on the matrix:** generation guarantees at least one **non-transitive cyclic relationship** (A beats B, B beats C, C beats A). Mathematically, this is what sustains coexistence.
- **Environmental heterogeneity → niches:** gradients/diffusion make different species thrive in different zones.
- **Carrying capacity:** the crowding penalty prevents unbounded growth of a single species.

These three levers are the main object of final tuning.

### 5.9 Starting constants (baseline) **[plausible baseline / to be validated in playtesting]**

Initial values that are mutually coherent (conceptually verified so that a photolithic bloom grows in open space and stabilizes from crowding, and so that a matrix −2 is visible but not insta-lethal). In implementation they all live in a single config location (§5.6), ideally hot-reloadable.

**Environment**

| Constant | Value | Notes |
|---|---|---|
| Scalar range | `[0,1]` | temperature, light, toxicity |
| Environmental diffusion (Phase 1+) | `0.05` / tick | slow blend with neighbor average |
| Light gradient (Phase 0) | `0.9` (high) → `0.2` (low) | creates a vertical niche |
| Temperature gradient (Phase 0) | `0.2` (left) → `0.8` (right) | creates a horizontal niche |
| Toxic zone | `toxicity = 0.7` | elsewhere `0.0` |

**Time and actions**

| Constant | Value | Notes |
|---|---|---|
| `ERA_TICKS` | `25` | pulses per era — **current implementation; superseded in intent by §4's three-level scale, awaiting retune** |
| Season length | *to be tuned* | **[PROPOSED]** new intermediate unit; the player's decision scale |
| Era length | *to be tuned* | **[PROPOSED]** in seasons; indicatively ~4 |
| Era budget / world | `60` (early) → `45` (late) | finite: gives roguelike tension. Retuned twice in playtest (tasks 049, 059). **[PROPOSED]** with longer eras this drops to roughly `15` → `11`, keeping total duration comparable |
| Point budget | `3` **per era** | **[PROPOSED]** refills **per season** instead, so decision density survives the longer era |
| Action costs | seed `1`, stress `1`, cull `1`, splice `2` | splice tunable up to `3`; with `Splice` clarified as *synthesise-then-seed* (§6) the two-step cost makes `2` defensible without raising it |
| Grace period / world | `3` eras, adaptively extended | task 079; suppresses total-extinction failure only (§8) |

**Energy and metabolism** (per organism)

| Constant | Value | Proposed | Notes |
|---|---|---|---|
| Energy at seeding | `5.0` | unchanged | |
| Base `upkeep` | `0.5` / tick | unchanged | maintenance cost |
| `crowd_factor` | `0.15` / occupied neighbor | unchanged | carrying capacity |
| `repro_threshold` | `10.0` | unchanged | energy needed to reproduce |
| `repro_cost` (to the child) | `5.0` | unchanged | subtracted from the parent |
| Photolithic `metabolism_gain` | `2.0` | **`0.8`** | `gain = light · gain · env_fit` |
| Predator `drain_cap` | `2.0` / tick, `upkeep 0.7` | **`0.8`**, upkeep unchanged | draws from neighbors |
| Decomposer `extract_rate` | `1.5` / tick, `upkeep 0.5` | **`0.6`**, upkeep unchanged | from residue |
| Chemolithotroph `metabolism_gain` | `2.0` / tick, `upkeep 0.5` | **`0.8`**, upkeep unchanged | `gain = toxicity · gain · env_fit` (task 108) |
| `interaction_delta` scale | *verify in build* | **`0.15`** per unit of matrix intensity | **[OPEN]** whether a scale coefficient exists today, or `{−2..+2}` enters the energy sum raw — this changes the diagnosis materially and is the first thing to check |
| Residue on death | `3.0`, decays `0.2` / tick | feeds decomposers |
| Residue ambient trickle | `0.05` / tick, per cell | floor against an isolated decomposer starving uninformatively; stays well below the decay rate so it never makes decomposer self-sufficient |
| `env_fit` | `exp(−(temp−temp_opt)² / (2·temp_tol²))` | `temp_tol` (σ) default `0.15` |

*Quick check, current values:* an isolated photolithic organism with `light≈0.7`, `env_fit≈1` → `gain≈1.4`, net `≈+0.9`/tick (grows); with 6–8 neighbors → net `≈−0.15`/tick (stalls → carrying capacity). In a dark zone (`light 0.2`) → `gain 0.4 < upkeep 0.5` → doesn't survive (light niche). A predator with no prey: `gain 0 − upkeep 0.7` → collapses in ~7 ticks (prey-predator dynamic).

*Quick check, proposed values (§5.6's balance principle):* the same isolated photolithic organism → `gain = 0.7 × 0.8 × 1 = 0.56`, net **`≈+0.05`/tick** — near-breakeven, ~100 ticks to reproduce, i.e. it effectively doesn't without help. With one `+2` neighbour → `0.05 + 0.30 − 0.15 = +0.20`/tick (reproduces in ~25 ticks). With one `−2` neighbour → `0.05 − 0.30 − 0.15 = −0.40`/tick (dies in ~12 ticks). In a dark zone → `gain 0.16 < upkeep 0.5` → still dies, so the light niche and the §5.8 defences survive the change. Note that at these values `crowd_factor` (`0.15`) is the same order of magnitude as a `+1` interaction: **a neighbour is never free — only a strongly positive one pays for itself.** All of the above assumes `ERA_TICKS = 25` and must be retuned alongside §4's longer eras.

**Tags and matrix**

| Constant | Value | Notes |
|---|---|---|
| Global tag pool | `10` glyphs | variety across worlds — **[PROPOSED]** `15` launch pool (+10 documented reserve), see §5.5 |
| Active tags / world | `5` (early) → `8` (late) | difficulty lever — **[PROPOSED, validate in playtest]** `4` (world 0) → `9` (late), gradual ramp |
| Tags per species | `1–3` | |
| Effect intensity / adjacency | integers in `{−2,−1,0,+1,+2}` | additive (§5.6) |
| Matrix density | ~`40%` non-zero pairs | rest `0` |
| Generation constraint | ≥1 negative RPS cycle guaranteed | coexistence (§5.8) |

**Notebook**

| Constant | Value | Notes |
|---|---|---|
| Confirmation threshold / cell | `3.0` cumulative evidence | |
| Weight of one observation | `1 / (1 + n_adjacent_confounders)` | rewards clean experiments |
| Cell states | `?` unknown · `0` no effect · `±?` hypothesis · `±!` confirmed | |

**Grid**

| Constant | Value | Notes |
|---|---|---|
| Size | `128×80` | task 074 |
| Neighborhood | Moore (8) | |

**Evolution (§5.11)**

| Constant | Value | Notes |
|---|---|---|
| `selection_pressure_threshold` | `20.0` | cumulative weighted pressure that fires a speciation event — **tuned against 25-pulse eras; must be retuned with §4's longer eras, or speciations will fire many times per era and trivialise both the `Speciation` objective and emergence (§5.12)** |
| `interaction_harm_weight` | `1.0` | weight on a tick's harmful `interaction_delta` share |
| `terrain_mismatch_weight` | `1.0` | weight on a tick's `1.0 - env_fit` |
| `toxicity_weight` | `1.0` | weight on a tick's `toxicity` exposure |
| `max_species` | `40` | hard cap on `world.species.len()`, correctness bound (`SpeciesId` is a `u8`) |

### 5.10 Biomes **[DECIDED, tasks 110-112]**

On top of the continuous scalar layer (§5.2), each cell also carries a discrete **biome** — an areal classification (water depth, elevation bands, and feature biomes like `Forest`, `Swamp`, `Crater`, `CrystalField`, `Lake`) assigned by a **two-stage** generation pass: a base classification derived from elevation/moisture-like scalars, then explicit feature placement (bounded-retry rectangles, the same pattern the toxic zone already used) for the biomes that read as discrete "spots" rather than smooth bands. Biomes are primarily a **legibility and worldbuilding layer** — dithered flat-color rendering with borders and a tree overlay for `Forest` (task 112) so the map reads as terrain, not a heatmap — but also the gating surface for terrain-conditional tags (§5.5) and the placement substrate wild species (§9) require. The separate, older `toxic_zone` (a fixed hostile rectangle, §5.9) is not yet folded into the biome enum — it currently coexists with it as its own mechanic. **A unification task is planned:** the toxic rectangle becomes an ordinary feature biome placed by the same bounded-retry pattern the other features already use. Worth doing *before* anything is built on top of biomes, not after.

**Full biome roster [PROPOSED]:** a design pass proposes a 16-biome set — deep water, shallow water, lake, swamp, plain, forest, hill, mountain, peak, bare rock, deep crater, desert, volcanic vent, geyser, tundra, crystal field — each with baseline `temperature` / `light` / `toxicity` values, and trees handled as an **overlay** independent of biome (dense on forest, sparse on plain/hill/mountain/swamp, absent elsewhere). Several tie directly into other systems: bare rock is the natural niche for `Chemolithotroph` (§5.4); vent and geyser are where point heat sources would physically live if scalar generation moves from fixed axis gradients to diffusing point sources; crater is the natural home for a "precursor" anomaly. See the biomes design document.

**Rendering [DECIDED, task 112]:** flat colour per biome with light two-tone dithering for material texture, crisp borders aligned to the grid (coastlines heavier than interior boundaries), and overlays drawn as flat tints rather than blended colours. The rule that matters: **never blend two pieces of information into an intermediate colour** — a blend produces an ambiguous hue that communicates neither. This is the same "discrete bands, not continuous gradients" principle the elevation layer uses.

**Dynamic biomes [PROPOSED]:** biomes are currently generated once and fixed. A design pass proposes letting some of them **change during play** — an impact crater forming where a bolide lands, a plain heating over several eras into a volcanic vent, sustained toxicity slowly crystallising, a seismic event raising an elevation band. One shared mechanism serves all of them: a trigger (instantaneous, or a condition sustained over several eras), a visible transition state (dashed border on the cell, colour switching only on completion — never a blended midpoint), and differentiated reversibility (instantaneous events are permanent; sustained-condition transitions revert if the condition lapses before completion, which gives the player a real lever rather than a spectacle). **Integration hazard:** if a cell changes biome, any terrain-conditional trait (§5.5) gated on that biome must deactivate or activate accordingly. See the world-events design document.

### 5.11 Evolution by speciation **[DECIDED, tasks 106-109]**

A species under sustained pressure can **speciate**: the simulation itself creates a new species, distinct from any player action (`Splice` remains the player's own, budgeted tool — this is separate and free).

- **Selection pressure accumulator:** each tick, every species accumulates weighted pressure from three stimuli — harmful `interaction_delta` share, environmental (temperature) mismatch (`1 - env_fit`), and `toxicity` exposure (§5.9's `EvolutionConfig` weights, default `1.0` each).
- **Threshold crossing:** once a species' cumulative pressure crosses `selection_pressure_threshold` (`20.0` baseline), a `SelectionThresholdCrossed` event fires, tagged with whichever of the three stimuli was **dominant** for that species.
- **Speciation:** the dominant stimulus determines the kind of edit made to a copy of the parent species' genome (e.g. a toxicity-dominant crossing can grant the new species tolerance to hostile terrain) — the new species is added to `world.species` (capped at `max_species`, a correctness bound: `SpeciesId` is a `u8`) and starts appearing in the population going forward.
- **Long-term objective:** `Objective::Speciation` (§8) is a fourth objective kind — a within-world **long-term** tier, always appended as the sequence's final entry regardless of what the earlier (short-term, randomly drawn) objectives are. It clears once the world has produced at least one speciation event (`SimWorld::has_speciated`).

This is the game's only source of genuinely new species mid-run: worldgen (§9) decides the *starting* palette, but a hostile-enough world can grow its own species the player never seeded. *(With `Splice` clarified as synthesising a new species rather than editing a live one — §6 — the game now has **two** routes to a new species: one emergent and free, one deliberate and budgeted. Both add to `world.species` under the same `max_species` cap.)*

**Retuning note:** the pressure threshold is calibrated against 25-pulse eras. §4's longer eras require retuning it alongside the energy coefficients — see §5.9.

**Rare xenotrait outcome [PROPOSED]:** with a low, separate probability, a `SelectionThresholdCrossed` edit produces a **xenotrait** (§5.5) instead of the ordinary genome edit. This gives xenotraits a precise mechanical origin inside an existing system rather than leaving them as a free-floating event.

### 5.12 Emergence: from microscopic to macroscopic **[PROPOSED]**

A proposed terminal, rare transition: a lineage that has been reshaped enough by its world **collapses from a population into a single macroscopic organism**. Thematically it is the natural conclusion of a game called *Abiogenesis* — life beginning simple and becoming complex — and mechanically it is a second, rarer kind of world ending alongside the objective sequence.

- **Trigger, condition 1 (necessary, sufficient on its own):** the traits that appeared along a **lineage** of speciations — measured against the genome of the original species that lineage descends from, not accumulated on one species — cover **at least 3 of the 5 trait families** (§5.5). Depth isn't imposed as a separate gate: covering three families almost always requires several successive speciations, so it emerges from the coverage requirement. A rarer alternate path: **2 families plus a xenotrait**.
- **Trigger, condition 2 (optional accelerator):** surviving a catastrophic event while condition 1 already holds lowers the family requirement by one, or doubles the probability ramp below. Deliberately *not* a hard requirement, since catastrophic events are themselves only `[PROPOSED]` — emergence must not be blocked waiting on a system that may never be built.
- **Probabilistic firing:** once condition 1 holds, a roll fires with a base chance that ramps each subsequent era it keeps holding, up to a ceiling short of certainty. The ramp resets if the condition lapses. This avoids emergence snapping the instant conditions are technically met, and gives the experienced player a legitimate indirect signal ("something seems about to happen to this lineage") without ever showing a progress bar.
- **On firing:** the species stops being a population count and becomes a **single organism** with its own identity, inheriting for each covered family the most strongly confirmed trait along the lineage. It always takes the highest reveal tier (§4), overriding the ordinary ranking.
- **Why it stays out of the objective pool, permanently:** objectives are *required* to win and must remain achievable by deliberate action within a finite budget; emergence keeps a probabilistic component even when its conditions hold. Acceptable for an optional payoff, unacceptable for a victory requirement.

**Not specified here:** what happens to the simulation state (energy, reproduction, matrix interaction) once a population collapses into a single entity — that is a redesign for a special case, to be handled as its own task. See the emergence design document.

---

## 6. Player actions (interventions) **[DECIDED]**

**Inspection is not an action [PROPOSED — see `culture-shock-inspect-tool.md`].** A long-standing gap: the player must be able to *observe* in order to deduce, but the only direct reading has been "fill/brightness = energy." A free, budget-exempt tool — a light hover tooltip (species, population, trend) plus a click-anchored card with the **last pulse's energy balance broken down line by line** (gain, each neighbouring trait's signed contribution, upkeep, crowding, net) — closes it. It costs nothing and consumes no season budget, deliberately kept separate from the four paid actions below. It requires no new computation: every figure it shows is already produced and carried forward by the tick pipeline (§5.6) — it is pure exposure, not a parallel system. The hover tooltip also names the biome under the cursor on **any** cell, populated or not — closing a real gap (biome was previously only readable from colour/texture, never from text, breaking the "colour is never the only channel" rule applied elsewhere) that matters most while `Seed` is armed, since the player needs to know where they're about to place before clicking, not after. Clicking an **empty** cell pins a biome-characteristics card instead of an energy breakdown — biome name, temperature/light/toxicity as qualitative bands (not raw GDD values, which stay internal), and habitability if the biome cannot host terrestrial organisms at all, so a wasted `Seed` isn't the only way to find out. **Deliberately excluded from that card: any hint of a terrain-conditional trait** (§5.5) tied to the biome — that link stays part of the same opacity protecting the matrix, discoverable only by observing an anomaly in play, never by consulting a panel. The gap widens with the per-cell population model (above): a falling count alone says even less than a single dying organism used to, since there's no longer an individual to have watched.

Actions are what the player queues before advancing time:

- **Seed** (`Seed`) — place an organism of an available species in a cell.
- **Environmental stress** (`Stress`) — alter an environmental scalar in the selected cell. **[DECIDED, task 145]** the axis is *selectable* — thermal, light, or toxicity — rather than thermal only. This isn't a new action but a choice inside the existing one: it removes an arbitrary asymmetry (light and toxicity matter as much as temperature, for `Photolithic` and `Chemolithotroph` respectively) without inflating the roster. A single application is temporary: the shifted axis decays back toward its pre-stress value over subsequent ticks (`EnvironmentConfig::stress_decay_rate`), distinct from environmental diffusion's neighbour-blur.
- **Removal / cull** (`Cull`) — eliminate the organism in the selected cell.
- **Synthesis / splice** (`Splice`) — **synthesise a new species** with chosen traits, added to the seedable roster. The most powerful and most expensive experimental tool.

**Scope: per cell, not per area [CORRECTION].** Earlier revisions of this section described `Stress` and `Cull` as operating "in an area." The implementation, and §11's controls ("mouse cell selection", "left click: perform the selected action on a cell"), are **per cell** — this prose had drifted, and the build is authoritative. Per-cell is also the better design: `Stress` on one cell combined with environmental diffusion (§5.2, Phase 1+) produces an area effect that *spreads over time*, more elegant than an instant area decided up front; and a `Cull` that could wipe a whole species in an area would hand the player a shortcut to remove what bothers them instead of understanding it — bypassing the core loop, and badly underpriced at 1 point.

**`Splice` creates a species, it does not edit one [CLARIFICATION].** It synthesises a **new** species with the chosen traits into the seedable roster; that species then has to be **placed with an ordinary `Seed` action** like any other. It is a laboratory experiment: you design a strain from the biochemistry you've decoded, then implant it and see whether it holds. This resolves what would otherwise be an awkward question (do living organisms of an edited species change retroactively?) — nothing living is touched. It also keeps the cost honest: 2 points to synthesise, another to seed, and it still has to survive. Architecturally it adds a `world.species` entry exactly as speciation does (§5.11), under the same `max_species` cap.

- **Usable traits [PROPOSED]:** `Splice` can only assign traits the player has **already confirmed** in that world's matrix (§7) — not the whole active set. Under the laboratory framing this stops being an arbitrary balance rule and becomes a consequence of the fantasy: you can't synthesise what you haven't yet understood. It also turns `Splice` into a *reward for having decoded something*, giving one more reason to keep experimenting rather than a way to stop needing to.
- **Never xenotraits [PROPOSED]:** `Splice` never reaches the xenotrait pool (§5.5) at any unlock level — they remain reachable only through evolution.
- **Consequence:** the seedable roster is **no longer fixed** by worldgen; it grows during play as the player synthesises. §11's UI must account for new entries appearing mid-world, distinguishable from the original palette.

**`Cull` as a knockout experiment [PROPOSED]:** removing an organism and watching what changes in its neighbours is conceptually a gene-knockout experiment — the third experimental mode alongside "place and observe" and "stress and observe". `Cull` should therefore **generate a tracked observation** in the notebook when it removes an organism adjacent to others, using the existing weighting (§7). The cost stays at 1; the budget is already the natural brake against using it as an undo button.

**Action budget [structure DECIDED / baseline in §5.9]:** a tight point budget reinforces the mental model "one unit of time = one deliberate experiment" — you can't blanket the grid, you have to bet on your best hypothesis. Baseline: **3 points**, costs **seed 1, stress 1, cull 1, splice 2**. With a budget of 3 and splice at 2 you can combine one splice + one cheap action, or three cheap actions: the choice is interesting. **[PROPOSED]** with §4's three-level time scale the budget refills **per season** rather than per era, so decision density survives the longer era.

**Proposed fifth action — `Isolate` [PROPOSED]:** protects a cell from neighbouring influence for one season. It addresses a real gap: once §5.6's balance principle makes clean observations necessary to decode the matrix, the player has no tool to *guarantee* one — only to hope an organism stays isolated long enough. It gives the player control over the quality of the experiment itself, not just its ingredients, and pairs with the existing observation weighting rather than duplicating any current action. Weigh against the "few, heavy levers" principle before adopting.

**Considered and rejected:** `Move` (repositioning a seeded organism) risks making a bad placement cheap to correct, diluting the weight of the original decision — only viable at a high cost. `Analyse` (revealing a hidden value on demand) contradicts the central pillar of indirect deduction and would be the single most damaging addition to the game's identity. `Direct hybridisation` (forcing a speciation on command) breaks the same rule that makes both speciation and xenotraits meaningful: evolution emerges from pressure, it is never chosen.

---

## 7. The discovery layer: the notebook **[DECIDED]**

This is the heart of progression (§ pillar 2) and turns observation into a *deduction game*.

- **Observation log:** salient events recorded era by era (blooms, collapses, extinctions, notable adjacencies). **[PROPOSED]** each entry carries a *clean vs confounded* indicator, surfacing the observation weight below without stating the formula — teaching by reinforcement that isolating an experiment pays.
- **Relationship view:** the player's current picture of the `tag × tag` matrix, with cells **confirmed** by accumulated evidence. **[PROPOSED]** render this as a **directed graph** rather than a grid: nodes are traits, an edge exists *only where an observation exists*, colour encodes sign, solid vs dashed encodes confirmed vs hypothesis, and a dashed node ring marks a trait not yet involved in anything. A grid forces the player to scan `T²` mostly-empty cells; a graph grows with the player's knowledge and makes chains and cycles — the actual *aha* — visible as shape. Trait **family** is deliberately not shown (it would leak §5.5's dominant-family bias); xenotraits are marked distinctly. Note the layout risk at 8–9 active traits: a fixed circular layout crowds, so a force-directed layout or a focus-on-one-trait view may be needed.
- **Species catalog:** the species encountered and what's known of each — metabolism, thermal range, current population, traits, era of origin, its **origin** (seeded from the starting roster / wild / synthesised via `Splice`) and, if it arose by speciation, its **parent species** ("descends from").
- **Chronicle:** a fourth section, distinct from the observation log — where the log holds raw scientific data for inference, the chronicle holds the world's **narrated history**: each end-of-era reveal (§4) archived once closed, at the tier it fired at, so the player can revisit what happened rather than only catching it live. No separate text generation: it stores what the reveal already produced. Consecutive quiet eras collapse into a single line rather than repeating. Speciation entries name the parent species, which makes a lineage reconstructable by scrolling — enough to serve §5.12 without building a dedicated tree view.

**Confirmation model [DECIDED] — "B with a hint of C":** the game **accumulates evidence** from observations and **confirms a cell** in the matrix when the evidence crosses a threshold. The confirmed cell "lights up": it's the explicit *aha* and the progress bar of pillar 2. The hint of C rewards **good experimental method**: a *clean* observation (isolated adjacency, typically produced by a deliberate experiment) **weighs much more** than a confused one. Concretely, the weight of the evidence is **inversely proportional to the number of other adjacent tags** that could confound the signal — `weight = 1 / (1 + n_confounders)` — and a cell confirms at **cumulative evidence ≥ 3.0** (baseline, §5.9). This way there's no need to build a fragile "clean experiment" detector: it's simply weighted by how many confounders were present (an isolated observation is worth 1.0, one with three confounding tags is worth 0.25). This mechanic **is** the progressive revelation of the matrix (§5.5): there isn't a second, separate opacity mechanic.

**Gap worth naming — the player has no hypothesis [POST-MVP].** Despite the "hypothesis grid" label, what the notebook shows is confirmations the *system* accrues once evidence crosses threshold. The player reads them; they never declare anything. So nothing can ever be *contradicted* — their reasoning stays in their head, invisible to the game, and the strongest identity moment available goes missing. The fix is a **declared prediction**: the player may mark what they expect on a trait pair before knowing, kept visibly distinct from system confirmations, resolving later as confirmed / **contradicted** / ambiguous. Three rules make it good rather than tedious: it must be **optional** (mandatory hypothesising becomes paperwork), it must carry **no cost or penalty** (the moment being wrong costs something, players only hypothesise once already certain — killing the exact behaviour it was meant to encourage), and it must give something back — **attention**, not a bonus: marked relationships get watched, so incoming evidence on them is surfaced rather than lost among the rest. A contradiction is worth the top reveal tier the first time it happens in a run, and must read as a report rather than a reproach. This also gives the end-of-run summary something it currently cannot do: place what the player understood and what they got wrong **side by side, without hierarchy** — *being wrong isn't a failure, it's the work.* See `culture-shock-identity.md`.

**Opening the notebook [PROPOSED]:** a panel sliding in from the left over part of the map, with the map still visible but dimmed behind it and the HUD sidebar still readable and usable on the right — so consulting the notebook never means losing sight of the run. **Time pauses while it is open**, consistent with §4's rule that time never advances while the player is planning.

---

## 8. Objectives, victory, and defeat **[DECIDED]**

Each world poses a **sequence of explicit requirements**, resolved in order: **2** objectives in early worlds, ramping to **3** in late worlds (task 059), drawn from `Coexistence`/`SurviveIn`/`TriggerBloom` — plus, since task 109, a **long-term** `Speciation` objective (§5.11) always appended as the sequence's true final entry, on every world, regardless of the earlier draw. Clearing a non-final objective in the sequence advances to the next one and resets its progress — it does not clear the world by itself. No two consecutive short-term objectives in a world share the same kind. Examples of the kind of objective:

- "Achieve a biosphere with **≥3 coexisting species**, each holding **≥3 individuals**, for **4 seasons**." *(2026-08-06 playtest: the requirement is tuned and displayed in the player's own unit of interaction — originally eras, restated in seasons per task 178 once §4 made the season that unit. 2026-08-29 playtest: the population floor is task 178, closing a loophole where mere presence — any count > 0 per species — satisfied the objective regardless of how small.)*
- "Grow a species that **survives in the toxic zone**."
- "**Trigger a bloom** of a specific type."
- (always last) "**Force a speciation event** through sustained pressure."

**Leaving a world, entering the next, ending a run [PROPOSED].** Victory-as-flag turns "move to the next world" from an automatic consequence into a *player choice* — which currently comes with nothing to base it on. Leaving a world should therefore surface what the player understood (relationships confirmed out of those active), what remains open (eras left, and at most a hint that *something* is in progress — never what, consistent with §5.12 having no progress bar), and the generated closing line; and it must state plainly that leaving is irreversible before it happens. Entering a world stays deliberately **minimal** — world number, seed, objectives — because previewing a world's difficulty tells the player what they should be discovering by observing. Ending a run is not a game over but the close of a research session: a cumulative summary across worlds (worlds visited and how each ended, relationships confirmed, speciations induced, species synthesised, biomes and wild species met, whether an emergence ever occurred), which must stand on its own in MVP without promising or gesturing at unlocks.

- **Success** (every objective in the world's sequence cleared) → the world is **won**. **[DECIDED, task 154] Victory is a flag, not a forced ending:** the player stays free to keep playing the same world until its era budget runs out, and moves on to the next world (more active tags, meaner matrix, more hostile environment) when *they* decide. This is not only consistent with the existing rule that a *run* ends only by the player's choice — it is **required** for emergence (§5.12) to be reachable at all, since a lineage of three speciations always arrives after the single speciation that clears the final objective. It also creates a good tension: *you've already won — but do you want to see whether something rarer happens?*
- **Failure** → the current world must be retried; a new attempt keeps the run's meta-progression but re-seeds the world (task 051). The run itself only ends by the player's own choice to stop, not automatically on a single world's failure.

**Failure conditions [DECIDED]:**

- **Total extinction** → immediate failure of the current world, not the run (`GameState::WorldFailed`, task 051): the world is retried, not the whole run.
- **Era budget per world** generous but **finite** (baseline: 60 eras in early worlds, dropping toward 45 in late worlds — retuned from the original 40/25 to absorb the cost of multi-objective worlds, see §5.9): a stuck player fails instead of grinding forever. This is what gives the roguelike tension.

**Onboarding grace period [DECIDED, task 079]:** total-extinction failure is suppressed at the start of every world — for a fixed `grace_eras` window (§5.9), then *adaptively extended* past it until the player has kept a population alive for a full era (`era_ticks` consecutive ticks) at least once. A fixed window alone would still let a world fail instantly and without warning the moment it expires on an empty grid; the extension removes that cliff, guaranteeing every world gives the player at least one real look at a living ecosystem before extinction can end it. Once that foothold is reached, it's spent — a later extinction in the same world fails normally. The era-budget-exhaustion failure condition is deliberately untouched by grace (its magnitude is always far smaller than the budget).

**Correctness rule — measure from activation, not from world state [DECIDED, task 154].** An objective that checks a *state* rather than a *change since it became active* can be satisfied the instant it appears: speciation, for instance, can fire for any species at any time, so a `Speciation` objective checking "has a speciation ever occurred" may show up already ticked. Every objective should therefore snapshot the world when it activates and require change or persistence **after** that point — a `Coexistence` timer starts at zero on activation even if the species were already there. Implemented for `Speciation` (`SimWorld::species_parent.len()` snapshotted at activation); `Coexistence`/`SurviveIn` already satisfied this by construction (their consecutive-tick counter starts at zero on activation).

**`Speciation` with a named target [DECIDED, task 179].** Applying that rule to the final objective: if no speciation has yet occurred, it stays generic. If one already has, the objective becomes *"induce a speciation in species X"*, X drawn deterministically from the living species that haven't speciated and are above a minimum viability. The player can no longer clear it by waiting for one to fire somewhere — they have to find X and apply targeted pressure. If X goes extinct first, the system re-targets so the objective stays solvable. This is permissible where emergence is not, because speciation pressure is *deterministic* and the player has direct levers to induce it. Implemented as a field on `ObjectiveProgress` (`speciation_target`), not a new resource — selection is a deterministic hash of the world's seed and a reselection counter, since the pure `evaluate` function only ever sees `&SimWorld`, not the RNG.

**Procedural parameters [PROPOSED].** Objective parameters should be drawn from what the world actually contains — `SurviveIn` targeting a biome that exists on *this* map, `TriggerBloom` naming a species actually in *this* roster — so an objective reads as written for that world. The constraint: an objective states **what** it measures, never **why** that parameter was chosen (naming the dominant family, for instance, would leak §5.5).

**Additional objective kinds [DECIDED, task 179]** — widening the pool the generator draws from, not the number active at once: **Homeostasis** (hold a species' average energy inside a stable band — the only kind that touches energy, and the only one that rewards active correction rather than diversity, endurance or growth), **Tolerance** (keep a species alive in a high-toxicity zone — mechanically the same `ZoneKind::Toxic` presence check `SurviveIn` already uses, differing only in name and generation-pool membership, since the GDD calls for it as a distinct, harder-tier requirement rather than a new mechanism), **Wild coexistence** (keep a wild species alive alongside a seeded one — gives wild species a role past first contact), and **Rootedness** (keep a species alive on the terrain a conditional trait it actually carries is tied to). All four implemented and drawn by the generator. **First confirmation** (confirm one matrix relationship within N units of time — intended mainly for world 0, reinforcing the core loop before anything exotic) stays **[PROPOSED]**: it needs `MatrixKnowledge`, a separate `Resource` not reachable from `evaluate`'s `&SimWorld` — deferred rather than widening that signature or special-casing it in `apply_tick_outcome` ahead of a clearer need.

**Deliberately excluded:** an objective of the form "decode X% of the matrix." It contradicts §16.5 — the game is built so that decoding 3–4 relevant cells out of ~20 suffices — and would turn the notebook from a tool used as needed into a completion checklist.

**Durations in seasons [DECIDED, task 178]:** objectives expressed in eras were tuned that way because eras were the player's unit of interaction (see the 2026-08-06 note above). With §4 making the *season* that unit, duration-based objectives are now tuned in seasons at the config/generation layer (`ObjectiveConfig::coexistence_seasons_base`/`survive_in_seasons_base`) — converted to the underlying tick count only at generation time, case by case rather than by mechanical conversion, since a few may remain more natural at era scale.

**Bonus objectives [DECIDED as direction / low priority]:** planned in principle (grant meta-progression currency), but **after** the clean "primary objective → advance" core. Not in the minimal MVP.

---

## 9. World generation and difficulty curve **[DECIDED]**

Each world is generated procedurally:

- New **biochemical matrix** (asymmetric, with the cyclicity constraint of §5.8).
- **Environment** (gradients, extreme zones, biomes — §5.10) with increasing hostility.
- **Active tags** (subset of the pool, including any terrain-conditional ones, §5.5) and a pool of **available starting species**. *(Task 050: the player seeds every organism in the world manually, including the first ones — worldgen only decides what's available to seed, not what's on the grid, **with one narrow exception**: task 098's **wild species**, a small pre-existing population placed directly onto the grid at generation time, tracked separately from the player-seedable roster. Every other species still starts unplaced.)*
- World **objective(s)**, resolved in sequence (§8), always closed out by the long-term `Speciation` objective (§5.11).

**First-world softening [DECIDED, task 079]:** world 0's opening objective is always forced to the gentlest possible requirement — `Coexistence` with `min_species = 2` — rather than the normal random draw, which could otherwise open a run with a demanding `SurviveIn` (a hostile zone the player hasn't even seen yet) or a `Coexistence` requiring every generated species including a harder-to-keep-alive Decomposer.

**Curve [DECIDED direction]:** the first worlds with **5 active tags** and a mild environment; gradually up to **~8 active tags**, matrices with more "nasty" relationships, more extreme environments (large toxic zones, harsh thermal gradients), stricter objectives, and shorter era budgets.

**Biochemistry is fresh every run**: replayability comes from here, not from hand-written content.

**World events [PROPOSED].** A design pass proposes a catalogue of events a living world can produce, in two cost tiers. *Cheap ones reusing data the sim already computes:* positive threshold events (a first confirmed relationship, a population reaching sustained stability), extinction cascades (an extinction changing the pressure on a species that depended on it — already computable from §5.11's stimuli, it just needs naming and surfacing), colonisation windows (a biome briefly hospitable as scalars drift), silent genetic drift below the speciation threshold, and dormant wild species waking on an environmental condition. *More ambitious ones:* convergent evolution (two independent lineages arriving at the same trait by chance — a genuine "wow" precisely because nothing scripted it), echoes of the past (a cell heavy with accumulated death producing an anomalous bloom when revisited by a decomposer), predictable environmental cycles the veteran can learn and exploit, and catastrophes — bolide impact, volcanic awakening, self-inflicted toxic bloom (the only catastrophe that is the player's own fault, and the better for it), magnetic storm suppressing one trait *family*, advancing glaciation, seismic upheaval. Several of these are the dynamic-biome mechanism of §5.10 wearing different clothes.

**Two constraints on any event system.** First, **frequency scales with the same world-tier dial** that already drives active traits, environmental hostility and era budget — a new independent difficulty parameter for each system is how a game becomes untunable. Second, **anything that runs on its own must remain traceable to a past player choice**; where it can't be, the event belongs in the rare/ceremonial tier rather than the recurring one, because that's where the absence of direct control is acceptable.

---

## 10. Meta-progression **[DECIDED light / persistence deferred post-MVP]**

**Design note [PROPOSED].** The "capabilities, not answers" rule cuts deeper than it first appears: in a game about discovery, *anything* that reduces the work of discovering erodes the game itself — an unlock like "the notebook confirms on half the evidence" makes the game easier by removing precisely what makes it good. The usable question is therefore *what can grow run over run without making discovery cheaper?*, and three categories answer it. **Breadth** — more to discover, not easier discovery: new starting species, new biomes in the possible set, xenotraits entering the pool, new objective kinds. **Tools** — new ways to experiment, not to know: `Isolate` (§6) and the full manual mutation tier of `Splice`, both of which grant control over the experiment rather than information about the result. **Challenge** — access to harder starts, not shortcuts. What to rule out: anything accelerating the notebook. One instructive exception: pre-filled matrix cells of which *some are wrong* are not answers but **hypotheses to falsify** — they add epistemic work rather than removing it, so they remain legitimate under this section's rule. Unlocks should key off cumulative *understanding* (relationships confirmed, speciations induced, biomes and wild species met) rather than objectives cleared, so that exploring is rewarded over rushing — and those are exactly the figures the end-of-run summary already reports.

Progression *between* runs, deliberately **light**:

- Unlocking **more starting species** or **tools** (e.g., one extra action, or a known tag).
- The **matrix always remains to be deciphered** from scratch: you don't unlock "answers," you unlock *capabilities*.

**Persistence [DECIDED: deferred]:** the MVP is built **without persistence** (everything within a single run). Whether to save unlocks (profile/save) is decided **only after** verifying the loop is fun. It's trivial to add later and doesn't constrain the architecture.

---

## 11. Presentation and UX **[DECIDED direction]**

*(v0.4 revision: rendering moves from the terminal to a 2D graphical window. Pillar 3 doesn't change — no art assets, just colored squares: the window gives more room and a readable UI, not "graphics.")*

- **Rendering:** 2D window. Grid of cells as colored squares.
  - Occupied cells: color = species/tag; brightness = energy.
  - Empty cells: faint background reflecting the environment (e.g., brightness = `light`).
- **Alien tags:** nameless glyphs/colors, learned empirically.
- **UI panels:** time readout (era + **[PROPOSED]** season + current pulse), populations per species, current objective, action budget, command hints.
- **Notebook:** dedicated panel (log + relationship view + catalog + **[PROPOSED]** chronicle). The relationship view is dense and interactive: the use case where immediate-mode UI (egui) is clearly better suited than a persistent-widget UI.

### HUD design direction **[PROPOSED]**

A design pass proposes a sidebar that reads as a *laboratory console* rather than a management dashboard — the difference is mostly structural, not decorative, and costs nothing against pillar 3:

- **One continuous panel divided by hairlines**, not four separately bordered boxes; monospace throughout.
- **Diegetic labels** — "Intervene", "Census", "Genome bank", "Directive" rather than Action / Population / Seed palette / Objective.
- **Discrete counters instead of continuous progress bars** for small countable resources (moves left, eras elapsed): ticks and pips communicate "countable" where a percentage bar communicates "loading".
- **Census rows carry a trend arrow *and* a numeric delta** against the previous era — the trend alone gives direction, the delta gives magnitude, which is what actually lets a player catch a runaway or a collapse in time.
- **Energy is a population average, not an individual value**, so it must not be rendered against the individual `repro_threshold`: a mean of 7.4 against a threshold of 10 does *not* mean nobody reproduced. Show it as a qualitative vitality trend; put the reproduction threshold in the catalog as the static per-species trait it is; let the event log report actual births.
- **Everything scales past 3 species**: compact single-line rows with a fixed-height scrolling container, and a horizontally scrolling genome bank — which must also accept entries appearing mid-world now that `Splice` synthesises species (§6).
- **Consistent species colour and metabolism iconography** across map, census, genome bank and catalog — one visual grammar, not one per screen. Icon shape encodes metabolism; colour always encodes the *species*, never the metabolism.

### Main menu and onboarding **[PROPOSED — see `abiogenesis-menu-onboarding.md`]**

**Menu:** deliberately spare — title, then *Resume* (shown only when a save slot exists, and stating plainly that resuming **consumes** it), *New run*, *Settings*, *Quit*. Everything instructional moves off this screen: it was previously one wall of onboarding text, which a player on their tenth run meets every single time.

**New-run setup**, a separate screen: two lines on the mechanism, controls as key caps, a **pre-filled random seed** the player can edit or reroll (§5.7's determinism makes seeds worth exposing), a discreet optional link to a fuller "how to play" recap, and the start button.

**Stress-tested against a player with zero prior knowledge [PROPOSED — see `culture-shock-naive-player-example.md`].** §16's worked example shows the game at its best, written by someone who already knows every system; replaying the same opening as a naive player surfaces five friction points concentrated in the first ~20 minutes — the same window this section's fantasy is meant to hook. The common thread: the game always *has* the relevant data, but never surfaces it proactively at the moment it would matter, requiring the player to already know it's worth going to look. This doesn't contradict "no guided tutorial" below — it's a different gap: not missing instruction, missing a nudge toward *where to look*, distinct from *what to do*. Four minimal fixes follow that principle: a second contextual hint keyed to the first apparent stall (near-zero net energy for N ticks); a lightweight on-map indicator for saturation-without-outlet the first time it occurs in a run (today visible only inside the inspect card); the speciation reveal naming the dominant stimulus, not just the outcome (the data already exists in the pipeline); and trait codes in the first handful of a run's log entries showing the involved species name alongside the code, until the player has learned to read them unaided. None of these tell the player what to do. Specified for implementation, prioritised, and placed in the build order (right after the core-loop playtest checkpoint, before content work) in `culture-shock-friction-fixes.md`.

**No guided tutorial [PROPOSED].** A hand-holding tutorial would be actively harmful here, and not for stylistic reasons: this game teaches that you form hypotheses and verify them by observing. A tutorial that tells the player *what to do* teaches the opposite — that answers arrive from outside rather than from observation — and it would break that compact in the first five minutes, exactly when it's being established. What's needed instead is for **the first world to be legible**, and most of that is already designed elsewhere without being called onboarding: the softened world-0 objective (§9), fewer active traits in world 0, a guaranteed early strong interaction, a visual spark on interaction, and a world-0-only "first confirmation" objective (§8). Two small additions do the rest: **one non-blocking contextual hint** before the first advance, phrased as *what to watch* rather than what to do (the distinction is the whole point — the first teaches a method of observation, the second replaces the player's reasoning); and the "to get started" tip seeded as **the first entry of the world-0 observation log**, so the player finds it where they'll learn to look rather than on a screen they'll learn to skip.

### Visual language **[PROPOSED — see `culture-shock-population-model-aesthetic.md`]**

**Colour belongs to environment, shape belongs to life.** Biomes and scalars keep colour; organisms are encoded by metabolism shape (the existing icon set) in a single neutral ink — proposed as a muted amber rather than white, which read too clinical — filled or hollow for whether the local population sits above or below its energy threshold. This resolves a real tension: species colour on the map was competing with biome colour and doesn't scale past a handful of species, and the map now carries information (health, at a glance) it previously couldn't. The trade-off, accepted deliberately: the map no longer distinguishes two species sharing a metabolism — that identity stays in the census and notebook. Notebook panel material: a cool dark slate, lighter than the map but still dark — considered and rejected paper (read as a school notebook) and white (too clinical) — staying inside the "instrument" world rather than crossing into paper.

**Pixel-grain rendering [DECIDED — see `culture-shock-population-model-aesthetic.md`], refining *how* the above is drawn, not revising it.** Smooth vector read as "technical instrument" but not "warm" — the brief was to move toward the tactility of games like Stardew Valley or Dwarf Fortress **without reintroducing a hand-drawn asset pipeline**, which would break the near-zero-cost-content condition that made the trait/biome/xenotrait roster possible in the first place. Two fully procedural techniques, combined: organism shapes snapped to a pixel grid instead of smooth vector paths (same encoded identities, quantised rather than drawn on curves); and a lightweight algorithmic noise texture over each biome's flat colour, replacing the fixed two-tone dither. The same register extends across the whole interface, not just the map — action icons, tick indicators and borders all go square — because a smooth HUD beside a pixel map would read as two different products. In the notebook, the treatment reaches the relationship graph's edges too: **stepped (orthogonal) paths** instead of smooth diagonals, the "no smooth lines" principle carried all the way through, not just applied to node shapes. Text stays crisp monospace throughout — it never needed the treatment. A full hand-drawn pixel-art tileset (one sprite per biome/metabolism/transition) was considered and rejected: it would reintroduce exactly the production cost the project has avoided from the start.

### Accessibility **[OPEN]**

The matrix currently encodes sign as red/green, the worst possible pairing for the most common colour vision deficiencies. A partial non-chromatic channel already exists (dashed vs solid for hypothesis vs confirmed) but isn't systematic. Worth resolving before it becomes a diffuse rewrite.

### Controls **[PROPOSED — full scheme, resolving conflicts across earlier drafts, in `culture-shock-controls.md`]**

Earlier drafts of this section, the actions document, the inspect-tool document and the notebook document each specified controls independently and drifted out of sync — no control for the two-level zoom, left-click overloaded between "fire the armed action" and "open the inspector" with no way to tell them apart, no way to disarm an action once selected, `Esc` doing three unreconciled things (quit / close inspector / — in the notebook doc — nothing at all), `space` still bound to the era after the season became the player's unit of decision, and `r` with no protection despite a save slot now existing to lose by mistake. The scheme below is the resolved version; full rationale in the companion document.

- **Mouse:** hover shows a light tooltip always; left-click with nothing armed opens/pins the inspector card (default); left-click with an action armed fires it; **right-click disarms** the current action without firing it; **scroll wheel zooms** between overview and detail; drag pans.
- **Keyboard:** `1`–`4` quick-select Seed/Stress/Cull/Splice; `space` advances one **season** (the unit that changed with §4); `Shift+space` advances a full era; `n` advances a single pulse; `p` toggles continuous advancement; `g` advances to the next notable event; `tab` opens/closes the notebook; `wasd`/arrows pan; `r` reseeds the world, **prompting for confirmation if the world has already received a player action** (armed to reset instantly only before anything is touched).
- **`Esc` is layered, never a direct quit:** closes the notebook if open, else the inspector card if pinned, else disarms an armed action, else opens the new **pause menu** (Resume / Settings / Save & quit / Abandon without saving) — this is where quitting actually lives now. No single `Esc` press ever exits the game directly, which matters specifically because of the single, load-consumed save slot (§cross-cutting).

---

## 12. Tech stack **[DECIDED]**

*(v0.4 revision: from `ratatui`/TUI to Bevy with an ECS model.)*

- **Language:** Rust, 2021 edition (code and comments in English). Toolchain pinned to **1.97.1**.
- **Engine:** **Bevy 0.19** — ECS, scheduling, states, plugins, input, window, 2D rendering.
- **UI:** **`bevy_egui` 0.41** (egui 0.35) for HUD and notebook.
- **RNG:** `rand` with an explicit seed kept in the world state.
- **Architecture [DECIDED]:** one module = one Bevy `Plugin` — `ConfigPlugin`, `WorldPlugin`, `SimPlugin`, `GridRenderPlugin`, `UiPlugin`, `InputPlugin`. The simulation is separate from rendering and input.
  - **The grid is a `Resource`, not ECS entities.** State lives in `SimWorld` as dense double-buffered arrays; Bevy entities exist **only for rendering** (one sprite per cell, synced read-only). The reason is the determinism of §5.7: parallel ECS query iteration is the fastest way to lose it.
  - **Tick logic is pure Rust**, callable without the Bevy `App`: this is what makes determinism and balance (§5.8) testable headlessly, and what makes final tuning practical.

Architectural detail — states, `SystemSets`, events, invariants — lives in `TECH_DESIGN.md`, not here.

---

## 13. Development plan (~2 weeks, phased) **[DECIDED]**

Actual implementation happens in Claude Code, with this GDD as reference.

### Phase 0 — Walking skeleton *(~2–3 days)*
Grid + environment (static gradients) + **one** metabolism (photolithic) + reproduction + death + colored-sprite rendering in a 2D window + HUD + era / single-tick advancement.
**Milestone:** watch a photolithic species bloom and stabilize thanks to carrying capacity. *(v0.4 revision: v0.3 assumed the project was already scaffolded — it wasn't. The Cargo scaffold + Bevy app is the phase's first task, along with the toolchain update from 1.90 to 1.97.1 required by Bevy 0.19.)*

### Phase 1 — Emergence *(~3–4 days)*
Tags + **hidden matrix** + multiple species + predation and decomposition + **seed** action.
**Milestone:** true emergence appears; multiple species interact via the matrix.

### Phase 2 — Deduction *(~3–4 days)*
Notebook + observation log + hypothesis grid + **stress / cull / splice** actions.
**Milestone:** the *deduction game* is born, not just the simulation.

### Phase 3 — The run *(~2–3 days)*
**Objectives** system + world generation + win/lose + run flow and minimal meta-progression.
**Milestone:** a complete game cycle, world after world.

### Final tuning *(remaining time)* — *the real art*
Balancing emergence: the levers of §5.8 (cyclicity, heterogeneity, carrying capacity) + the tick formulas (§5.6). Goal: emergence that is *interesting and readable*, avoiding "everything dies" and "one dominates."

---

## 14. Risks and open questions

### Main risk **[DECIDED as priority]**
**Making emergence interesting rather than boring or unreadable.** This is *system* tuning, not art. Mitigations in §5.8. Starts from known dynamics (predation/reproduction on a lattice, spatial Lotka–Volterra-like) with the hidden rules as "spice."

### Readability risk
Maximum mystery (§5.5) may end up too raw. Mitigation: progressive revelation of the matrix via the notebook; keep metabolisms and environment always readable.

### Newly identified risk — the matrix being optional **[PROPOSED fix in §5.6/§5.9]**
With the v0.6 baseline, reading only the visible environmental layer is enough to grow a population: the hidden matrix — the game's whole point — is bypassable for basic success. This is arguably more dangerous than the readability risk below, because the game *appears* to work while its central mechanic goes unused. The fix is a tuning one (near-breakeven from environment alone), but it should be verified before further systems are layered on top, since biomes, family bias and the new objective kinds all assume a player who *must* experiment.

### Cross-cutting concerns **[PROPOSED — see `abiogenesis-cross-cutting.md`]**

Six areas that no design document previously covered. They aren't mechanics, but if they're missing the game is unusable or exclusionary regardless of how good the design is.

- **Saving [PROPOSED]:** snapshot rather than journal — a journal (seed + action sequence, replayed on load) is invalidated by *every* coefficient change, and this document is full of coefficients still to be tuned. Keep the journal as metadata beside the snapshot anyway: with a deterministic sim it buys shareable replays and reproducible bug reports. **The design question inside the technical one:** free save/reload lets the player undo a failed experiment, which dissolves the action budget's whole tension — so **one slot, consumed on load**: suspend-and-resume, not checkpointing. Two easy-to-miss details: persist the RNG *state*, not the seed (otherwise resuming diverges from the run the player was playing), and put the notebook and chronicle in the snapshot — they *are* the in-run progression.
- **Audio [PROPOSED — detailed in `abiogenesis-audio.md`]:** pillar 3 constrains *graphics*, not sound, and §2 already aims at something hypnotic to watch. Generative rather than composed, in three layers of **decreasing density**, because a game with no natural silence will otherwise fatigue the player into muting it — and muting costs the informational value too. **The bed** is designed to be *forgotten*: modulated by the viewport's mean scalars (heat → body, light → filter opening, toxicity → instability), noticed only when it changes. **The rhythm** marks the **season boundary**, not the pulse — a beat per pulse under continuous advancement becomes an obsessive tick; the sound should mark when the player must return to *decide*. **Events** are sonified in proportion to their significance, reusing the reveal ranking so audio and notebook can't disagree by construction, and **spatialised** by position relative to the viewport. That last part is why audio isn't decorative here: under continuous advancement the player cannot watch 10,240 cells at once, so a cue for something happening off-screen is information no visual channel can carry. Avoid: composed music (it fights a system of unpredictable duration), and UI feedback on every interaction (with a 3-point budget every click is already weighted). Synthesised prototypes exist in `audio-prototipi/`.
- **Colour accessibility [PROPOSED]:** relationship sign is encoded red/green — the worst possible pairing for the commonest colour vision deficiencies. The rule to adopt is not an option to add: **colour must never be the only channel.** Sign needs an explicit `+`/`−` glyph; confirmed-vs-hypothesis and species already have second channels (dash/solid, icon shape); the log's clean/confounded dot currently doesn't. An alternate palette in settings comes *after* that rule, not instead of it.
- **Performance [PROPOSED]:** the simulation isn't the risk (~10⁵ ops/pulse is trivial for native Rust). Three real ones: rendering 10,240 cells per frame in immediate mode (needs batching or a single texture — verify first, it's what degrades visibly); continuous advancement multiplying pulses per second; and **unbounded growth of observation data** — every tagged adjacency emits one, so a long run needs compaction, since what's needed is cumulative evidence per trait pair, not an integral list kept forever.
- **Language [PROPOSED]:** English for the game, Italian for internal docs only. The decision that must be made *now* rather than later: procedurally assembled text is far costlier to localise than fixed strings — grammatical gender and agreement break fragment concatenation designed for English — so if localisation is even possible, fragments must be kept as **structured data with grammatical attributes**, not pre-concatenated strings.
- **Settings [PROPOSED]:** audio levels, continuous-advancement speed (not a detail — it's the perceived tempo of the whole game), alternate palette, language, reduced motion. Reachable from the main menu and from an in-run pause.

### Closed questions
- **Tags:** global pool ~10, active 5→~8; **asymmetric/directional** matrix (§5.5). *(Both counts have [PROPOSED] revisions — see §5.5.)*
- **Grid:** `128×80` (§5.1) — task 074 raised it from the original `48×32` baseline; this line previously lagged.
- **Stack:** Rust + Bevy 0.19 (ECS), 2D window, `bevy_egui` UI; grid as a `Resource`, entities only for rendering (§12).
- **Action budget:** points per era (baseline 3), differentiated costs (§6).
- **Hypothesis confirmation:** **B with a hint of C** model, weight inverse to confounders (§7).
- **Failure:** total extinction + finite era budget per world (§8).
- **Tick formula structure:** **additive/linear** matrix effect, centralized coefficients (§5.6).
- **Meta-progression persistence:** deferred post-MVP (§10).

### Still to be validated in playtesting (coefficients, not structure)
- Numeric values now have a **plausible baseline in §5.9** (`ERA_TICKS`, budgets, confirmation thresholds, `metabolism_gain`, `upkeep`, `crowd_factor`, `repro_*`, matrix intensities, etc.): they need to be confirmed or adjusted through playtesting, not reinvented.
- Final grid size (partly empirical).
- **Season and era durations, and the per-world era budget** (§4) — the whole numeric baseline hangs off these.
- ~~Final title~~ — **decided: `Culture Shock`**, subtitle *A sterile world* retained. "Abiogenesis" was the working title and survives only in document filenames. See `culture-shock-identity.md`.

---

## 14b. Companion design documents

This GDD holds decisions and structure; the design pass that produced v0.7 left detail in standalone documents, each self-contained. **Start from `abiogenesis-INDEX.md`**, which maps every document to what it covers and proposes a working order with dependencies and a play-test checkpoint after the core loop.

This GDD holds decisions and structure; the design pass that produced v0.7 left detail in standalone documents, each self-contained:

| Document | Covers |
|---|---|
| `abiogenesis-system-hierarchy.md` | Three-tier classification of every system (core / structural variation / rare payoff), the two cross-cutting principles (agency preserved, single difficulty dial), and a resolution log of GDD discrepancies |
| `abiogenesis-open-points.md` | Everything still undecided, including the §5.6 formula question and the never-addressed areas listed in §14 |
| `abiogenesis-actions.md` | `Splice`/`Cull`/`Stress` revisions, proposed and rejected new actions |
| `abiogenesis-matrix-necessity-balance.md` | The "matrix must be necessary" correction and its recomputed coefficients |
| `abiogenesis-time-scale-reveal.md` | Pulse/season/era, the reveal beat, continuous advancement |
| `abiogenesis-objectives.md` | Procedural objective generation, the five new kinds, victory-as-flag |
| `abiogenesis-tag-archetypes.md` | The 15+10 trait roster, families, codes, dominant-family bias, xenotraits |
| `abiogenesis-emersione.md` | Emergence trigger, inheritance, integration requirements |
| `abiogenesis-biomes.md` | The 16-biome roster with environmental baselines and tree overlay rules |
| `abiogenesis-world-events.md`, `abiogenesis-world-events-catastrophes.md` | Event catalogue and the dynamic-biome mechanism |
| `abiogenesis-narrative-generation.md` | How reveal and chronicle text is generated procedurally |
| `abiogenesis-tick-pipeline.md` | Phase-by-phase specification of the tick as a three-output pipeline (§5.6) |
| `abiogenesis-transitions-metaprogression.md` | Leaving/entering a world, end-of-run summary (MVP), meta-progression categories (post-MVP) |
| `abiogenesis-cross-cutting.md` | Saving, audio, colour accessibility, performance, language, settings |
| `abiogenesis-audio.md` | Audio design in depth: three-layer structure, simulation hooks, synthesised prototypes (`audio-prototipi/`) |
| `abiogenesis-menu-onboarding.md` | Main menu, new-run setup, and why there is deliberately no guided tutorial |
| `culture-shock-identity.md` | The title decision, the game's identity, and the post-MVP declared-hypothesis mechanic |
| `culture-shock-population-model-aesthetic.md` | Per-cell population model (capacity + overflow), its rendering consequences, and general aesthetic decisions (notebook material, colour-vs-shape) |
| `culture-shock-inspect-tool.md` | Free, budget-exempt inspection tool: hover tooltip + click card with a per-tick energy breakdown |
| `culture-shock-worked-example.md` | Italian-language mirror of §16's worked example, kept for design-discussion continuity — the GDD's English version is canonical |
| `culture-shock-naive-player-example.md` | The same first session replayed as a player with zero prior knowledge — a stress test of the "no guided tutorial" decision, surfacing five friction points and four minimal, tutorial-free fixes |
| `culture-shock-friction-fixes.md` | Implementable spec for the four friction fixes above, with priority and where they sit in the build order |
| `culture-shock-distribution.md` | Distribution strategy: platform, channels (itch.io then Steam Early Access), marketing (shareable world seeds/summaries), pricing — outside this document's technical scope, tracked here for discoverability |
| `culture-shock-controls.md` | Full, conflict-resolved control scheme: mouse (hover/click/right-click/scroll), keyboard, layered `Esc`, protected `R`, and the new pause menu |
| `culture-shock-wonder.md` | Pillar 5 (constant sense of wonder/discovery): small/medium/large/strange content proposals, tiered and prioritised |
| `culture-shock-biome-cosmic-events.md` | Third chapter on world events: biome-signature behaviours (Swamp, Crystal Field, Peak, Vent/Geyser, Crater) and cosmic-origin events (micrometeorite rain, cosmic ray, great dimming, derelict) — implementable specs |
| `culture-shock-experiment-incentive.md` | A headless two-bot test for whether the game actually rewards experimenting, plus operational lessons from the nearest comparable games (roguelike item ID, Alchemists, Eleusis, Understand) |
| `culture-shock-identity-visual-inspirations.md` | A wry narrator tone for generated text, wordmark directions, and declared (never quoted) science-fiction inspirations — including *Children of Time*, the direct source of the Emergence idea and the intended guide for a possible post-Emergence macro phase |
| `abiogenesis-hud-notebook.md`, `abiogenesis-notebook-cronaca.md` | HUD and notebook layout, the chronicle section |

---

## 15. Glossary

- **Tick:** the atomic unit of simulation (internal/mechanical term — `SimWorld::tick`, `sim::step`, GDD §5.6's algorithm). Since task 118, the player never sees this word: the UI and player-facing prose say **pulse** instead. Same concept, two names for two audiences.
- **Pulse:** the player-facing name for a tick (task 118). What advances one at a time with `N`, or `ERA_TICKS` at a time with `Space`.
- **Season [PROPOSED]:** a block of pulses; **the player's unit of decision** — the action budget refills at this scale (§4).
- **Era:** a block of seasons; the unit of **narration**, closed by a reveal beat (§4). Historically the player's unit of interaction, a role the season now takes over.
- **Tag:** an abstract biochemical marker of a species; the only thing that matters for interactions between species. May be terrain-conditional (§5.5).
- **Hidden matrix:** the secret `tag × tag` table of adjacency effects, different for every world.
- **Metabolism:** how a species derives energy (photolithic / predator / decomposer / chemolithotroph, §5.4).
- **Biome:** a discrete areal classification of a cell (water, elevation band, or feature biome), layered on top of the continuous environmental scalars (§5.10).
- **Carrying capacity:** the population ceiling imposed by the crowding penalty.
- **env_fit:** an organism's environmental fitness for the cell it occupies.
- **Speciation:** a simulation-driven creation of a new species from sustained selection pressure, distinct from the player's `Splice` action (§5.11).
- **Trait family [PROPOSED]:** one of five groupings (structural, metabolic, signalling, genetic, storage) over the trait pool (§5.5).
- **Xenotrait [PROPOSED]:** a trait outside known chemistry, never player-placeable, arising only as a rare outcome of speciation (§5.5, §5.11).
- **Lineage:** the chain of descent linking a species back through successive speciations to the original species it descends from (§5.12).
- **Emergence [PROPOSED]:** the terminal transition of a sufficiently reshaped lineage from a population into a single macroscopic organism (§5.12).
- **Reveal [PROPOSED]:** the beat closing an era, presenting what happened at one of three significance tiers, and applying any matured evolution (§4).

---

## 16. Anatomy of a playthrough (illustrated example, rewritten for v0.7)

*(The original example predated the v0.7 pass and used eras as the decision unit, the old energy coefficients, and Greek-letter glyphs. A mirror of this rewrite exists in Italian as `culture-shock-worked-example.md`, kept for design-discussion continuity — this English version is canonical.)*

Grid reduced for readability (in-game 128×80). Species: **Halo** (photolithic), **Rask** (predator), **Muck** (decomposer). Traits shown with their real codes: `CHL`, `QRM`, `PRN`, `LIP`.

### 16.1 World setup

World 0, so already softened: **4 active traits** instead of 5, the softened starting objective (`Coexistence`, min 2 species), plus the world-0-only `First confirmation` objective. Full sequence: 2 objectives + final `Speciation`.

Environment: a light gradient top-to-bottom, a thermal gradient cold-to-hot, a small high-toxicity pocket in one corner (Swamp biome). The world "breathes" before anything is seeded — the ambient drone is audible but faint, and the overview shows zero density everywhere: no life yet, but an environment that already exists.

### 16.2 The playthrough, season by season

**Season 1 — environment alone.** `Halo` is seeded in a bright cell (Plain biome). No neighbours, so the matrix has nothing to act on yet. Advancing pulse by pulse and inspecting the cell (click) shows the tick breakdown:

```
gain (light)      +0.56
upkeep            −0.50
crowding           0.00
──────────────────────
net               +0.06
```

Near-breakeven — exactly the balance correction at work: environment alone sustains, it doesn't grow. No alarm; it's the intended honest signal, not a bug.

**Season 2 — the matrix enters.** `Rask` is seeded in the adjacent cell, carrying trait `QRM`. Advancing, the next inspection of `Halo` shows a new line:

```
gain (light)      +0.56
QRM (neighbour)   +0.30
upkeep            −0.50
crowding           0.00
──────────────────────
net               +0.36
```

`QRM` turns out to have a `+2` interaction on `Halo` — discovered from the sign of the contribution, not told in advance. The notebook logs the observation: isolated adjacency, no other neighbour, weight 1.0 — as clean as it gets. The relationship graph gains a dashed amber edge (hypothesis, not yet confirmed): `CHL → QRM`, positive. This nearly satisfies world 0's `First confirmation` objective — evidence just needs to cross threshold.

**Seasons 3–4 — growth, then saturation.** `Halo`'s cell population grows (per-cell population, not a single organism) as per-capita energy crosses the reproduction threshold repeatedly. `Muck` was seeded on `Halo`'s other side — a tactical mistake, discovered soon after: once `Halo`'s cell saturates, it has nowhere to overflow, since both adjacent cells are held by `Rask` and `Muck`. The inspector flags it: *saturated, no outlet → pressure accumulating* — under the existing "environmental mismatch" stimulus (§5.11), caused by space rather than climate.

**Season 5 — wait or force.** Two legitimate paths. *Wait*: do nothing, let saturation pressure keep accumulating. *Force*: apply `Stress` (thermal axis) to `Halo`'s cell repeatedly across a couple of seasons, pushing it out of its comfort zone on purpose — accelerating the mismatch stimulus independently of saturation. Forcing is chosen here: it costs budget (1 point per application, refilled per season), but it's worth it to see a speciation before the world's era budget runs out.

**Season 7 — reveal, notable tier.** The era closes; the simulation halts on its own and shows the generated reveal:

> *"CHL, under sustained pressure, has crossed the reorganisation threshold. Halo-B is distinct from Halo."*

Tier **notable** (not epochal — a first speciation, but not one touching a rare trait). The beat shows a before/after: `Halo-B`'s pixel icon carries a slight shape variation, signalling a different trait. The chronicle archives it with the parent species: *"era 2 · Halo-B ⟵ speciated from Halo (environmental mismatch)"*. `Halo-B`'s catalog card shows "descends from: Halo". The final `Speciation` objective, if active, would update here — but as the world's first speciation it stays in its generic form (a new speciation after activation, surviving one era), not yet needing a named target.

**Season 9 — the laboratory route.** The notebook has confirmed `CHL → QRM` past threshold (solid green edge, no longer dashed). Rather than wait for another natural speciation, `Splice` is used: since `QRM` is confirmed, a new species can be synthesised carrying it by default — cost 2 points. `Halo-C` enters the genome bank, not yet on the world. Seeding it (1 more point) in a separate area, the catalog marks its origin "synthesised" — distinct from both "seeded" (the original `Halo`) and "descends from" (`Halo-B`).

**Later seasons — closing objectives.** `Coexistence` (min 2 species) has long been satisfied. `First confirmation` closed back in season 2. The final `Speciation` objective is already in progress; suppose the sequence's second objective is `Tolerance` — keep a species alive in the high-toxicity zone. `Muck` is seeded there (decomposers tolerate residue better and depend less on light) and sustained for the required duration.

**Victory as a flag, not an ending.** With every objective satisfied, the world is **won** — but it doesn't end. Era budget remains. `Halo-B`'s lineage already covers one trait family beyond `Halo`'s original; another related speciation adding a second family would make it emergence-candidate. No guarantee — probabilistic even once conditions hold — but a few more seasons are spent watching before moving on rather than leaving immediately. It doesn't happen this run. The world is left: the exit screen shows relationships confirmed (3 of 16 active in that world), remaining era budget, and the generated closing line — something like *"in this biochemistry, QRM acted as a growth catalyst wherever encountered."*

### 16.3 Patterns decoded in this run

- **Environmental niche** (season 1): environment alone sustains, doesn't grow.
- **Matrix interaction found via clean observation** (season 2): an isolated adjacency, weight 1.0, hypothesis then confirmation.
- **Spatial competition emerging from the model**, not scripted (seasons 3–4): a population blocked by differently-species neighbours.
- **Selection pressure as a deliberate choice** (season 5): wait or force with `Stress`, both legitimate.
- **Two routes to a new species, kept distinct by design**: natural speciation (free, emergent, unpredictable in outcome) vs `Splice` (costly, deliberate, limited to what's confirmed).
- **Victory as a milestone, not an end**: the tension of staying on for emergence even after winning.

Out of 16 possible relationships in that world (4 active traits, T²), only 3 were confirmed — **and that was enough to win**, consistent with the principle that the game never requires decoding the full matrix, only the relevant part.

### 16.4 What this example verifies

Walking it through end to end shows the pieces decided across separate design sessions do talk to each other correctly: the population model feeds local selection pressure, pressure feeds the reveal, the reveal feeds the chronicle, the chronicle feeds the catalog (lineage), and the two evolution routes (natural / `Splice`) stay mechanically distinct as intended, without overlapping. It surfaced no incoherence here — but this is the kind of check worth re-running whenever a major piece changes, not a one-time proof.

*End of document — v0.6. All design decisions are closed and backed by a numeric baseline (§5.9) and a played example (§16). Ongoing work tracks against `tasks/QUEUE.md`, with this GDD as the design reference and `TECH_DESIGN.md` as the architecture reference.*
