# Abiogenesis — Game Design Document

**Working title:** Abiogenesis *(provisional, to be confirmed)*
**Genre:** Emergent-simulation roguelike / xenobiology lab
**Platform:** Desktop, 2D graphical window
**Tech:** Rust + Bevy (ECS)
**Mode:** Single-player, era-based, with objectives and light meta-progression
**Document status:** v0.5 — post-Phase-3/playtest alignment (closed decisions + numeric baseline + playthrough example)

> Legend for the status of each decision:
> **[DECIDED]** agreed and stable · **[PROPOSED]** baseline I'm proposing, to be approved/corrected · **[OPEN]** to be decided together

### Changelog

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

### What it is NOT **[DECIDED]**

- Not a clicker/idle game with ever-growing numbers.
- Not a purely contemplative sandbox: there is pressure, objectives, victory and defeat. *(We explicitly chose the objectives-driven version over the zen version.)*
- Not a graphically driven game.

---

## 2. The player and the fantasy

**Fantasy:** "I'm a scientist facing an alien biochemistry I don't understand, and I decode it by cultivating it." Curiosity, the scientific method, that moment when a hypothesis is confirmed and a piece of the system lights up.

**Target player:** people who love deduction puzzles, emergent simulation (Dwarf Fortress in miniature), hard SF, systems optimization. People who find it hypnotic to watch a self-organizing system.

---

## 3. Core loop **[DECIDED]**

The fundamental cycle, repeated within each world:

1. **Seed** — place organisms on the grid (what, where, when).
2. **Advance an era** — the simulation proceeds by *N* ticks; watch the ecosystem move and evolve.
3. **Record** — note in the notebook what you observed (who bloomed, who collapsed, which adjacencies seem to have an effect).
4. **Hypothesize & intervene** — form a hypothesis about the hidden rules and put it to the test with a targeted intervention (environmental stress, removal, mutation, new seeding).
5. **Objective** — once the world's objective is met, move to the next world, deeper and more hostile.

The experiential fulcrum is the sub-loop **hypothesis → experiment → *aha***, embedded in the unpredictability of an ecosystem that lives on its own.

---

## 4. Time model: eras **[DECIDED]**

Time advances in **eras**: the player queues one or more actions, then advances the simulation by *N* ticks in a block and observes the result. This **makes the time model coincide with the mental loop**: plan (hypothesis), execute (era), observe (result).

- **Animation during the era [DECIDED]:** tick advancement is shown tick-by-tick (fast), preserving the feeling of a "breathing system," while *control* remains deliberately step-wise.
- **Era length [structure DECIDED / coefficient to be tuned]:** `ERA_TICKS = 25` as default, adjustable; exposed to the player as a choice ("advance by 1 / 10 / 25 ticks"). The exact value is a coefficient to validate in playtesting.
- **Real-time mode [OPEN / future]:** on top of this architecture it's cheap to add later as an option. Not in the MVP.

---

## 5. Simulation model

### 5.1 Grid and cells **[DECIDED]**

2D grid of cells. Each cell contains:

- an **environmental layer** (a few continuous scalars);
- at most **one organism** (single occupancy per cell), with an energy/population level.

**Size [DECIDED / final size empirical]:** **48×32** from the start, as a configuration constant. *(v0.4 revision: the old plan "40×20 in Phase 0 → half-block rendering to reach ~48×32" was entirely a terminal-width constraint — 80 columns at 2 characters per cell. With a graphical window that constraint disappears and we start directly at the target.)* Emergence needs more room: spatial patterns (Lotka–Volterra-like) need breathing space, and grids that are too small die from stochastic noise. The final size remains partly empirical.
**Neighborhood [DECIDED]:** Moore (8 neighbors) for interactions and reproduction.

### 5.2 Environmental layer **[structure DECIDED / parameters in §5.9]**

A few scalars per cell, in `[0,1]`:

- `temperature`
- `light`
- `toxicity`

**Current status:** `toxicity` is declared and rendered (toxic zones exist in worldgen, and the Stress action's target scalar could in principle be extended to it), but as implemented it is **not read anywhere in the tick loop** — it has no effect on `env_fit`, gain, or costs. Only `temperature` and `light` are live inputs to the simulation today. Toxicity-driven gameplay (e.g. a chemolithotroph metabolism tied to it) is tracked as a future addition in `VISION.md`, not part of the current baseline.

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

*(Starting set. Others can be added — e.g., a chemolithotroph tied to toxicity — as unlockable content.)*

### 5.5 Tags and the hidden matrix **[DECIDED]** — *the heart of the game*

Two levels are distinguished, so as not to conflate variety and difficulty:

- **Global tag pool [DECIDED]:** ~**10 glyphs** total biochemical tags in the game, to give visual variety between worlds.
- **Active tags per world [DECIDED]:** only a subset is actually in play in a given world. **Difficulty grows by increasing active tags, not the pool.** Baseline: **5 active tags** in the first worlds, up to **~8** in late worlds.

Each species carries 1–3 tags (from those active in the world).

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
2. **Metabolic gain** (depends on metabolism). `env_fit` gates all three the same way — it multiplies the raw gain, not just Photolithic's — so an organism can be sitting on abundant fuel (light, prey, residue) and still starve if the cell's temperature is far from its optimum:
   - *Photolithic:* `gain = light * metabolism_gain * env_fit`.
   - *Predator:* draws energy from occupied neighbors (within a cap), weighted by `env_fit`.
   - *Decomposer:* draws from residue/dead matter in the cell or its neighbors, weighted by `env_fit`.
3. **Hidden matrix effect:** for each occupied neighbor, for each tag pair (mine × theirs), sum the effect from the secret matrix → `interaction_delta` (can be + or −).
4. **Costs:** `upkeep` (base cost per tick) + `crowding_penalty = crowd_factor * n_occupied_neighbors` (carrying capacity).
5. **Energy update:** `energy += gain + interaction_delta − upkeep − crowding_penalty`.
6. **Death:** if `energy <= 0` → the organism dies (the cell frees up; optional: leaves residue for decomposers).
7. **Reproduction:** if `energy >= repro_threshold` and an empty neighbor exists → spawn a child in an empty neighbor (seeded random choice) with `repro_cost` energy, subtracted from the parent. *(Future phase: possible mutation of the child's genome.)*

**Processing order [PROPOSED]:** iteration in shuffled (seeded) order with a "born/acted this tick" guard so newborns don't act in the same tick; or double buffering (snapshot → next). To be chosen at implementation time, favoring correctness and determinism.

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
| `ERA_TICKS` | `25` | ticks per era |
| Era budget / world | `60` (early) → `45` (late) | finite: gives roguelike tension. Retuned twice in playtest (tasks 049, 059) — most recently to absorb the cost of worlds now posing 2–3 sequential objectives (§8) instead of one |
| Point budget / era | `3` | |
| Action costs | seed `1`, stress `1`, cull `1`, splice `2` | splice tunable up to `3` |

**Energy and metabolism** (per organism)

| Constant | Value | Notes |
|---|---|---|
| Energy at seeding | `5.0` | |
| Base `upkeep` | `0.5` / tick | maintenance cost |
| `crowd_factor` | `0.15` / occupied neighbor | carrying capacity |
| `repro_threshold` | `10.0` | energy needed to reproduce |
| `repro_cost` (to the child) | `5.0` | subtracted from the parent |
| Photolithic `metabolism_gain` | `2.0` | `gain = light · gain · env_fit` |
| Predator `drain_cap` | `2.0` / tick, `upkeep 0.7` | draws from neighbors |
| Decomposer `extract_rate` | `1.5` / tick, `upkeep 0.5` | from residue |
| Residue on death | `3.0`, decays `0.2` / tick | feeds decomposers |
| Residue ambient trickle | `0.05` / tick, per cell | floor against an isolated decomposer starving uninformatively; stays well below the decay rate so it never makes decomposer self-sufficient |
| `env_fit` | `exp(−(temp−temp_opt)² / (2·temp_tol²))` | `temp_tol` (σ) default `0.15` |

*Quick check:* an isolated photolithic organism with `light≈0.7`, `env_fit≈1` → `gain≈1.4`, net `≈+0.9`/tick (grows); with 6–8 neighbors → net `≈−0.15`/tick (stalls → carrying capacity). In a dark zone (`light 0.2`) → `gain 0.4 < upkeep 0.5` → doesn't survive (light niche). A predator with no prey: `gain 0 − upkeep 0.7` → collapses in ~7 ticks (prey-predator dynamic).

**Tags and matrix**

| Constant | Value | Notes |
|---|---|---|
| Global tag pool | `10` glyphs | variety across worlds |
| Active tags / world | `5` (early) → `8` (late) | difficulty lever |
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
| Size | `48×32` | from Phase 0 (v0.4) |
| Neighborhood | Moore (8) | |

---

## 6. Player actions (interventions) **[DECIDED]**

Actions are what the player queues before advancing an era:

- **Seed** (`Seed`) — place an organism of an available species in a cell.
- **Environmental stress** (`Stress`) — alter an environmental scalar in an area (e.g., raise toxicity, lower temperature).
- **Removal / cull** (`Cull`) — eliminate an organism or a species in an area.
- **Mutation / splice** (`Splice`) — modify a species' genome (e.g., change/add a tag, shift the thermal optimum). The most powerful and most expensive experimental tool.

**Action budget per era [structure DECIDED / baseline in §5.9]:** a tight point budget per era reinforces the mental model "an era = one deliberate experiment" — you can't blanket the grid, you have to bet on your best hypothesis. Baseline: **3 points per era**, costs **seed 1, stress 1, cull 1, splice 2** (mutation is the most powerful tool, hence the most expensive; tunable up to 3). With a budget of 3 and splice at 2 you can combine one splice + one cheap action, or three cheap actions: the choice is interesting. Exact numbers get refined in Phase 3.

---

## 7. The discovery layer: the notebook **[DECIDED]**

This is the heart of progression (§ pillar 2) and turns observation into a *deduction game*.

- **Observation log:** salient events recorded era by era (blooms, collapses, extinctions, notable adjacencies).
- **Hypothesis grid:** a view of the `tag × tag` matrix where the player notes their own conjectures about effects; the game marks which cells are **confirmed** by the evidence gathered.
- **Tag/species catalog:** alien glyphs encountered and what's known about them so far.

**Confirmation model [DECIDED] — "B with a hint of C":** the game **accumulates evidence** from observations and **confirms a cell** in the matrix when the evidence crosses a threshold. The confirmed cell "lights up": it's the explicit *aha* and the progress bar of pillar 2. The hint of C rewards **good experimental method**: a *clean* observation (isolated adjacency, typically produced by a deliberate experiment) **weighs much more** than a confused one. Concretely, the weight of the evidence is **inversely proportional to the number of other adjacent tags** that could confound the signal — `weight = 1 / (1 + n_confounders)` — and a cell confirms at **cumulative evidence ≥ 3.0** (baseline, §5.9). This way there's no need to build a fragile "clean experiment" detector: it's simply weighted by how many confounders were present (an isolated observation is worth 1.0, one with three confounding tags is worth 0.25). This mechanic **is** the progressive revelation of the matrix (§5.5): there isn't a second, separate opacity mechanic.

---

## 8. Objectives, victory, and defeat **[DECIDED]**

Each world poses a **sequence of explicit requirements**, resolved in order: **2** objectives in early worlds, ramping to **3** in late worlds (task 059). Clearing a non-final objective in the sequence advances to the next one and resets its progress — it does not clear the world by itself. No two consecutive objectives in a world share the same kind. Examples of the kind of objective:

- "Achieve a biosphere with **≥3 coexisting species** for **4 eras**." *(2026-08-06 playtest: the requirement is tuned and displayed in eras, the player's own unit of interaction — not raw ticks, which the player never consciously operates in.)*
- "Grow a species that **survives in the toxic zone**."
- "**Trigger a bloom** of a specific type."

- **Success** (every objective in the world's sequence cleared) → move to the next world (more active tags, meaner matrix, more hostile environment).
- **Failure** → the current world must be retried; a new attempt keeps the run's meta-progression but re-seeds the world (task 051). The run itself only ends by the player's own choice to stop, not automatically on a single world's failure.

**Failure conditions [DECIDED]:**

- **Total extinction** → immediate failure of the current world, not the run (`GameState::WorldFailed`, task 051): the world is retried, not the whole run.
- **Era budget per world** generous but **finite** (baseline: 60 eras in early worlds, dropping toward 45 in late worlds — retuned from the original 40/25 to absorb the cost of multi-objective worlds, see §5.9): a stuck player fails instead of grinding forever. This is what gives the roguelike tension.

**Bonus objectives [DECIDED as direction / low priority]:** planned in principle (grant meta-progression currency), but **after** the clean "primary objective → advance" core. Not in the minimal MVP.

---

## 9. World generation and difficulty curve **[DECIDED]**

Each world is generated procedurally:

- New **biochemical matrix** (asymmetric, with the cyclicity constraint of §5.8).
- **Environment** (gradients, extreme zones) with increasing hostility.
- **Active tags** (subset of the pool) and a pool of **available starting species**. *(Task 050: none of them are auto-placed — the player seeds every organism in the world manually, including the first ones. Worldgen only decides what's available to seed, not what's on the grid.)*
- World **objective(s)**, resolved in sequence (§8).

**Curve [DECIDED direction]:** the first worlds with **5 active tags** and a mild environment; gradually up to **~8 active tags**, matrices with more "nasty" relationships, more extreme environments (large toxic zones, harsh thermal gradients), stricter objectives, and shorter era budgets.

**Biochemistry is fresh every run**: replayability comes from here, not from hand-written content.

---

## 10. Meta-progression **[DECIDED light / persistence deferred post-MVP]**

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
- **UI panels:** current tick, era number, populations per species, average energy, current objective, action budget, command hints.
- **Notebook:** dedicated window (log + `tag × tag` hypothesis grid + catalog). The hypothesis grid is a dense, interactive table: it's the use case where immediate-mode UI (egui) is clearly better suited than a persistent-widget UI.

### Controls **[PROPOSED]**

- `space` — advance one era (*N* ticks).
- `s` — advance a single tick (fine observation / debug).
- (Phase 2+) keys to enter action mode: seed, stress, cull, splice; **mouse cell selection** (arrow keys as a keyboard alternative).
- `tab` — open/close notebook.
- `r` — reset / reseed the world.
- `Esc` — quit.

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

### Closed questions
- **Tags:** global pool ~10, active 5→~8; **asymmetric/directional** matrix (§5.5).
- **Grid:** 48×32 from Phase 0 (§5.1).
- **Stack:** Rust + Bevy 0.19 (ECS), 2D window, `bevy_egui` UI; grid as a `Resource`, entities only for rendering (§12).
- **Action budget:** points per era (baseline 3), differentiated costs (§6).
- **Hypothesis confirmation:** **B with a hint of C** model, weight inverse to confounders (§7).
- **Failure:** total extinction + finite era budget per world (§8).
- **Tick formula structure:** **additive/linear** matrix effect, centralized coefficients (§5.6).
- **Meta-progression persistence:** deferred post-MVP (§10).

### Still to be validated in playtesting (coefficients, not structure)
- Numeric values now have a **plausible baseline in §5.9** (`ERA_TICKS`, budgets, confirmation thresholds, `metabolism_gain`, `upkeep`, `crowd_factor`, `repro_*`, matrix intensities, etc.): they need to be confirmed or adjusted through playtesting, not reinvented.
- Final grid size (partly empirical).
- **Final title** (not urgent; "Abiogenesis" is the placeholder).

---

## 15. Glossary

- **Tick:** the atomic unit of simulation.
- **Era:** a block of *N* ticks advanced at once; the player's unit of interaction.
- **Tag:** an abstract biochemical marker of a species; the only thing that matters for interactions between species.
- **Hidden matrix:** the secret `tag × tag` table of adjacency effects, different for every world.
- **Metabolism:** how a species derives energy (photolithic / predator / decomposer).
- **Carrying capacity:** the population ceiling imposed by the crowding penalty.
- **env_fit:** an organism's environmental fitness for the cell it occupies.

---

## 16. Anatomy of a playthrough (illustrated example)

An example **World 1**, to show how the systems interweave in actual play. Grids here are reduced to **10×6** for readability (in-game 48×32). Tag glyphs are `◆ ○ ▲ ✦ ✚`; in-game they're nameless alien symbols — labeled here for the reader. Grid legend: `.` empty cell · letter = organism of that species · `+` = residue of a dead organism.

### 16.1 World setup

**Environment** (Phase 0, static gradients): light drops from top to bottom, temperature rises from left (cold) to right (hot). This produces spatial niches.

```
        cold ────────────────▶ hot
        col:  0  1  2  3  4  5  6  7  8  9
 light▲ r0    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·   high light (0.9)   ┐
 high  │ r1    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·                     │ vital
       │ r2    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·   mid light         │ band
       │ r3    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·                     ┘
 light │ r4    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·   low light (0.2) → too dark
 low     r5    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·      for photolithics
```

**Starting species palette** (4 available to the player):

| Species | Metabolism | Tags | Note |
|---|---|---|---|
| **P** | Photolithic | `◆ ○` | Balanced producer, `temp_opt 0.5` |
| **Q** | Photolithic | `▲` | Heat-loving, `temp_opt 0.7` |
| **R** | Predator | `✦` | Preys on neighbors |
| **D** | Decomposer | `✚` | Lives off residue |

**The hidden matrix** — the world's "solution," invisible to the player (row = tag that *exerts* the effect, column = tag that *receives* it; value = energy delta per adjacency):

| exerts ↓ / receives → | ◆ | ○ | ▲ | ✦ | ✚ |
|---|---|---|---|---|---|
| **◆** | · | · | **−2** | · | · |
| **○** | · | · | · | +1 | · |
| **▲** | · | · | · | **−2** | · |
| **✦** | **−2** | · | · | · | · |
| **✚** | **+2** | · | · | · | · |

Two hidden structures within these numbers:

- **An RPS cycle** (the three bolded `−2`s): `◆` suppresses `▲`, `▲` suppresses `✦`, `✦` suppresses `◆`. Translated into species: **P suppresses Q, Q suppresses R, R suppresses P**. This is the §5.8 constraint that keeps anyone from dominating → coexistence.
- **A mutualistic ring** (`✚→◆ = +2`): the decomposer **D fertilizes** the producer **P**. Plus `○→✦ = +1`, a minor effect that acts as "noise" to keep deduction non-trivial.

Note the **directionality**: `◆→▲ = −2` but `▲→◆ = ·` (0). **P harms Q, but Q doesn't harm P.** This asymmetry is what the player will discover first.

### 16.2 The playthrough, era by era

**Era 0 — initial seeding.** The player seeds P in the mild-bright band and Q in the hot corner (2 actions, budget 3).

```
 r0  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r1  ·  ·  ·  ·  P  P  ·  ·  Q  ·
 r2  ·  ·  ·  ·  P  P  ·  ·  Q  ·
 r3  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

**Era 1 — bloom in the niches.** Both grow where the environment favors them; the dark rows (r4–r5) stay empty (insufficient light). The two fronts approach each other around column 6–7.

```
 r0  ·  ·  ·  P  P  P  ·  ·  Q  ·
 r1  ·  ·  P  P  P  P  ·  Q  Q  Q
 r2  ·  ·  ·  P  P  P  ·  Q  Q  Q
 r3  ·  ·  ·  ·  P  P  ·  ·  Q  ·
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

The player wants a **clean experiment** on the P–Q interaction: spends 2 seedings to place an isolated P–Q pair in an empty, bright pocket in the top-left corner (r0, c0–c1), far from everything else.

**Era 2 — the first interaction reveals itself.** At the main front (col 6–7) and in the isolated pair, **Q withers wherever it touches P** (`◆→▲ = −2`). In the isolated pair, Q dies (leaves `+`) while P remains unharmed.

```
 r0  P  +  ·  P  P  P  P  Q  Q  ·     ← isolated pair: P alive, Q dead (+)
 r1  ·  ·  P  P  P  P  P  +  Q  Q     ← + at c7: Q died on contact with the P front
 r2  ·  ·  ·  P  P  P  P  +  Q  Q
 r3  ·  ·  ·  ·  P  P  P  ·  Q  Q
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

The isolated observation is **clean** (no confounders): weight `1.0`. The notebook records two facts — `◆→▲` is negative, and `▲→◆ = 0` (P remained intact: Q doesn't harm P). Here's the notebook's state:

| ↓ / → | ◆ | ○ | ▲ | ✦ | ✚ |
|---|---|---|---|---|---|
| ◆ | ? | ? | **−!** | ? | ? |
| ○ | ? | ? | ? | ? | ? |
| ▲ | **0** | ? | ? | ? | ? |
| ✦ | ? | ? | ? | ? | ? |
| ✚ | ? | ? | ? | ? | ? |

*(`−!` = confirmed negative · `0` = confirmed no effect · `?` = unknown)*

**Era 3 — introducing a predator.** P is spreading unchecked. The player seeds **R** (predator) into P's territory. R deals double damage to P: it **eats** it (metabolism) and **chemically suppresses** it (`✦→◆ = −2`). R explodes in numbers.

```
 r0  P  ·  ·  P  R  R  P  Q  Q  ·
 r1  ·  ·  R  R  R  R  +  +  Q  Q
 r2  ·  ·  ·  R  R  R  +  +  Q  Q
 r3  ·  ·  ·  R  R  P  +  ·  Q  Q
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

Meanwhile Q's hot corner remains **untouched**: `▲→✦ = −2` means **Q suppresses R**, so the predator can't invade the hot niche. (The player notices but doesn't yet understand why — it's a clue.)

**Era 4 — predator collapse and decomposer boom.** R has devoured almost all of P; with no prey left it **starves** (`gain 0 − upkeep 0.7`) and leaves a field of residue. The player seeds **D**, which blooms on the remains.

```
 r0  P  ·  ·  +  +  +  +  Q  Q  ·
 r1  ·  ·  +  D  D  +  +  ·  Q  Q
 r2  ·  ·  D  D  D  D  +  ·  Q  Q
 r3  ·  ·  ·  D  D  +  ·  ·  Q  Q
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

Now the ring closes: **D fertilizes P** (`✚→◆ = +2`), and P starts growing again from the patches of D. In the notebook, `✦→◆` is **suspected** but *confounded* by predation (R was eating P *and* suppressing it) → low weight → remains hypothesis `−?`. `✚→◆`, on the other hand, is seen cleanly in the D–P patches → about to be confirmed.

| ↓ / → | ◆ | ○ | ▲ | ✦ | ✚ |
|---|---|---|---|---|---|
| ◆ | ? | ? | **−!** | ? | ? |
| ○ | ? | ? | ? | +? | ? |
| ▲ | **0** | ? | ? | **−?** | ? |
| ✦ | **−?** | ? | ? | ? | ? |
| ✚ | **+?** | ? | ? | ? | ? |

**Era 5–6 — dynamic equilibrium → objective.** The player seeds a bit more R to restart predation and balance things out. The RPS cycle (P⊣Q, Q⊣R, R⊣P) plus the D–P mutualism settle into **spatial waves** rolling across the grid: no species dominates, four coexist.

```
 r0  P  P  D  R  R  P  Q  Q  Q  ·
 r1  P  D  D  R  P  P  ·  Q  Q  Q
 r2  D  D  P  P  R  R  ·  Q  Q  Q
 r3  ·  P  P  R  R  D  ·  ·  Q  Q
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

**World objective: "≥3 coexisting species for 4 eras" → met** (P, R, D in a cycle + Q in the corner). **Victory → World 2**, which adds a 6th active tag and a meaner matrix.

### 16.3 The patterns the player decoded

The **RPS cycle** that sustains coexistence:

```
        P (◆○) ──(◆→▲ : −2)──▶ Q (▲)
          ▲                       │
          │                  (▲→✦ : −2)
      (✦→◆ : −2)                  │
          │                       ▼
          └─────────────────── R (✦)

   P suppresses Q · Q suppresses R · R suppresses P → no one wins → they coexist.
```

The **decomposer ring** that recycles death into growth:

```
   death of P/Q/R ──▶ residue (+) ──▶ D (✚) blooms
                                          │
                                     (✚→◆ : +2)
                                          ▼
                              P (◆) reinvigorated ──▶ new biomass ──▶ (new deaths) ⟳
```

### 16.4 Anatomy of a single era

```
 ┌─ PLAN (budget: 3 points) ─────────────────────────────────┐
 │  e.g.  seed R (1)  +  stress: raise toxicity in an area (1) │
 │       +  cull P in a zone (1)        → budget spent          │
 └─────────────────────────────────────────────────────────────┘
                    │  press  SPACE
                    ▼
 ┌─ ADVANCE ERA (25 ticks, animated) ────────────────────────┐
 │  each tick, for every organism:                              │
 │  env_fit → metabolic gain → matrix effect →                  │
 │  costs (upkeep + crowding) → death / reproduction            │
 │  (deterministic given the same seed)                         │
 └─────────────────────────────────────────────────────────────┘
                    │
                    ▼
 ┌─ OBSERVE & RECORD ────────────────────────────────────────┐
 │  the notebook accumulates evidence; cells light up →         │
 │  you form the next hypothesis and the next era               │
 └─────────────────────────────────────────────────────────────┘
```

### 16.5 What the example demonstrates

In this run the player confirmed only **3–4 cells** out of the ~20 in the matrix — and that was enough to win: you don't need to decode *everything*, just the part relevant to the species in play and the objective. Every pillar was exercised: **environmental niches** (Era 1), **deduction via a clean experiment** (Era 2), **predator-prey** (Era 3–4), the **decomposer ring** and **RPS coexistence** (Era 5–6). And above all: in World 2 the matrix is **reshuffled**, so the facts learned **don't carry over** — only the *method* carries over. This is where replayability lives.

---

*End of document — v0.4. All design decisions are closed and backed by a numeric baseline (§5.9) and a played example (§16). Next step: implement Phase 0 following the operational queue in `tasks/QUEUE.md`, with this GDD as the design reference and `TECH_DESIGN.md` as the architecture reference.*
