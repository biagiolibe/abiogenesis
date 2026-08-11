# Evolutionary Ecosystem Discovery Game - Design Specification

## Purpose

This document formalizes a game design concept centered on:

- Species seeding
- Evolution
- Ecosystem simulation
- Scientific discovery
- Hidden interaction rules

The objective is to provide a design-level specification that can be analyzed and integrated into an existing game engine.

---

# High-Level Vision

The player acts as an exobiologist/ecosystem architect tasked with introducing life forms into an alien world.

The player does **not directly control creatures**.

Instead, they:

1. Create or deploy species seeds.
2. Observe ecosystem dynamics.
3. Collect data.
4. Form hypotheses.
5. Discover hidden biological laws.
6. Guide the long-term evolution of the planet.

The primary source of progression is **knowledge acquisition**, not resource accumulation.

---

# Core Design Pillars

## 1. Discovery

The ecosystem follows hidden rules.

The player must infer:

- Predator-prey relationships
- Cooperative relationships
- Symbiosis
- Competition
- Ecological dependencies
- Evolutionary pressures

The game should reward observation and experimentation.

---

## 2. Evolution

Species are dynamic entities.

They can:

- Adapt
- Mutate
- Specialize
- Hybridize
- Speciate
- Become extinct

Evolution should emerge from interactions rather than scripted progression.

---

## 3. Ecosystem Management

The player influences ecosystems indirectly.

Decisions involve:

- Which species to introduce
- Where to introduce them
- When to intervene
- Which evolutionary paths to encourage

The ecosystem should exhibit emergent behavior.

---

# Species Model

## Species Definition

Each species is represented by:

```text
Species
├── Genetic Archetype(s)
├── Population
├── Fitness
├── Traits
├── Environmental Preferences
├── Mutation Potential
└── Interaction History
```

---

## Genetic Archetypes

Archetypes represent high-level behavioral strategies.

Example archetypes:

- Predator
- Symbiont
- Scavenger
- Colonizer
- Parasite
- Defender
- Adapter
- Builder

A species may possess:

- One primary archetype
- Zero or more secondary archetypes

Example:

```text
Species: Sporex

Primary:
- Colonizer

Secondary:
- Symbiont
```

---

# Hidden Interaction Matrix

## Core Concept

A hidden matrix determines how archetypes influence one another.

Example:

|             | Predator | Symbiont | Parasite | Defender |
|-------------|-----------|-----------|-----------|-----------|
| Predator    | 0         | -1        | +2        | -2        |
| Symbiont    | +1        | +3        | -3        | +2        |
| Parasite    | +2        | +3        | +1        | -1        |
| Defender    | -2        | +1        | +2        | +2        |

Possible values:

```text
-3 = Strongly Negative
-2 = Negative
-1 = Slightly Negative
 0 = Neutral
+1 = Slightly Positive
+2 = Positive
+3 = Strongly Positive
```

The matrix remains hidden from the player.

---

# Interaction Resolution

Whenever species interact:

```text
Species A Archetype
          ×
Species B Archetype
          ↓
Interaction Modifier
```

This modifier affects:

- Survival rate
- Reproduction rate
- Fitness gain
- Resource efficiency
- Mutation likelihood

The exact implementation is left open.

---

# Scientific Discovery System

## Player Knowledge vs World Truth

The game maintains two separate layers:

### World Truth

The actual interaction matrix.

### Player Knowledge

The player's inferred understanding.

These layers should never be identical by default.

---

## Observation

The player observes outcomes such as:

```text
Population Growth
Population Collapse
Migration
Hybridization
Mutations
Extinctions
```

Observations generate evidence.

---

## Hypothesis Generation

The game converts evidence into hypotheses.

Example:

```text
Symbiont ↔ Defender

Confidence:
42%
```

Additional observations increase confidence.

Example:

```text
72%
```

Eventually:

```text
Confirmed
```

or

```text
Rejected
```

---

## Knowledge Database

The player gradually builds:

```text
Codex
├── Species
├── Traits
├── Mutations
├── Ecosystem Events
└── Interaction Matrix Entries
```

This becomes the primary progression system.

---

# Evolution System

## Fitness

Species possess a fitness value influenced by:

- Resources
- Environment
- Competition
- Predation
- Cooperation
- Climate

Example:

```text
Fitness =
Environment
+ Resource Access
+ Interaction Bonuses
- Threats
```

Exact formula is implementation-dependent.

---

## Mutation

Mutations emerge from evolutionary pressure.

Examples:

### Predator Pressure

```text
High Predation
→ Armor
```

### Climate Pressure

```text
Cold Environment
→ Thermal Adaptation
```

### Cooperative Pressure

```text
Strong Symbiosis
→ Social Adaptation
```

Mutations should ideally be partially deterministic rather than purely random.

---

## Speciation

When populations diverge sufficiently:

```text
Species A
      ↓
Species A'
```

The new species may:

- Gain archetypes
- Lose archetypes
- Gain traits
- Gain environmental adaptations

---

# Biomes

Biomes influence ecosystem rules.

Examples:

## Crystal Forest

```text
Symbiont Effectiveness +20%
Predator Efficiency -10%
```

## Methane Ocean

```text
Parasite Efficiency +15%
Colonizer Efficiency -20%
```

The same species interactions may produce different outcomes depending on biome.

---

# Resources

## Primary Resource: Knowledge

Knowledge replaces traditional economic resources.

Knowledge is earned by:

- Discovering species
- Confirming hypotheses
- Observing mutations
- Mapping ecosystems
- Understanding interactions

Knowledge is spent on:

- Advanced analysis
- Genetic engineering
- New seed creation
- Research
- Experimental interventions

---

# Gameplay Loops

## Short Loop (30s – 2min)

```text
Deploy Seed
    ↓
Observe
    ↓
Collect Data
    ↓
Update Hypotheses
    ↓
Repeat
```

---

## Medium Loop (10 – 20min)

```text
Build Ecosystem
    ↓
Introduce Species
    ↓
Observe Generations
    ↓
Discover Interactions
    ↓
Unlock New Possibilities
    ↓
Repeat
```

---

## Long Loop (Hours)

```text
Understand Ecosystem Rules
    ↓
Manipulate Evolution
    ↓
Create Stable Biospheres
    ↓
Discover Hidden Archetypes
    ↓
Reveal Planetary Biology
```

---

# Meta Progression

Persistent progress may include:

## Species Database

Previously discovered species.

---

## Genetic Knowledge

Confirmed interactions.

---

## Technologies

Scientific tools and research methods.

---

## Evolutionary Discoveries

Unique mutations and evolutionary branches.

---

# Endgame Vision

The player's ultimate objective is not merely survival.

The true goal is understanding the planet's biological operating system.

Possible late-game discoveries:

- Hidden archetypes
- Dynamic interaction matrices
- Planet-wide biological networks
- Emergent hive intelligences
- Self-modifying ecosystems

The final progression layer is the discovery of the fundamental laws governing life on the planet.

---

# Key Design Requirement

The most important design principle is:

> The player must feel like a scientist discovering hidden ecological laws rather than a commander directly controlling units.

Every major mechanic should reinforce observation, experimentation, hypothesis formation, and discovery.