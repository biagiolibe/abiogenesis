# Task 040 — Objectives: type + evaluation engine

> **ID**: `040`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

GDD §8: ogni mondo pone uno o più requisiti espliciti. Esempi letterali dal GDD: "Achieve a biosphere with ≥3 coexisting species for 50 ticks", "Grow a species that survives in the toxic zone", "Trigger a bloom of a specific type". Il successo porta al mondo successivo (task 045); il fallimento chiude la run (task 041).

Questo task introduce il tipo `Objective` e il motore che ne valuta il soddisfacimento tick per tick, **indipendentemente dal worldgen** — è testabile contro il mondo hardcoded attuale (Fase 0-2), non deve aspettare che il task 038/039 esistano. La generazione procedurale di *quale* obiettivo assegnare a ciascun mondo è il task 042, che consuma il tipo definito qui.

---

## 📋 Acceptance Criteria

- [ ] Nuovo modulo `src/objectives.rs` con `enum Objective`, almeno le tre varianti degli esempi GDD: `Coexistence { min_species: u32, ticks: u32 }`, `SurviveIn { zone: ZoneKind, ticks: u32 }` (dove `ZoneKind` copre almeno la zona tossica), `TriggerBloom { .. }` (definire i parametri minimi necessari a riconoscere un "bloom" — vedere se esiste già un concetto di bloom nel codice, altrimenti definirlo qui in termini osservabili, es. popolazione di una specie sopra una soglia in un'area).
- [ ] Resource `ObjectiveProgress` che traccia lo stato di avanzamento verso l'obiettivo corrente (es. contatore di tick consecutivi in cui la condizione è vera).
- [ ] Funzione pura `pub fn evaluate(objective: &Objective, world: &SimWorld, progress: &mut ObjectiveProgress) -> WorldOutcome` (o simile — vedere task 041 per `WorldOutcome` condiviso), chiamata da un sistema in `src/sim.rs` dopo `advance_tick`.
- [ ] **"≥3 specie per 50 tick" usa un conteggio consecutivo**: se la condizione si interrompe (es. una specie si estingue), il contatore si resetta a zero, non decrementa — deve essere gestito esplicitamente al confine di era (50 tick = 2 ere a `ERA_TICKS=25`), non assunto implicitamente.
- [ ] Gli obiettivi sono espressi **solo su quantità osservabili dal giocatore** (conteggio specie, popolazione, occupazione di zona, eventi di bloom) — **mai su celle della matrice biochimica nascosta**: un obiettivo che rivelasse indirettamente un valore della matrice romperebbe il pilastro della deduzione (GDD §11).
- [ ] `evaluate` è headless-testabile: nessuna dipendenza da `bevy::render`/`bevy_egui` (invariante 2), testabile con un `SimWorld` costruito a mano, senza aspettare il worldgen dei task 038/039.
- [ ] Test: almeno un test per ciascuna variante di `Objective`, incluso un test che verifica il reset del conteggio consecutivo quando la condizione si interrompe.
- [ ] `cargo clippy -- -D warnings` pulito, `cargo test` verde.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/objectives.rs` (nuovo) | `Objective`, `ObjectiveProgress`, `evaluate()`. |
| `src/sim.rs` | Hook che chiama `evaluate` dopo `advance_tick`; eventuale lettura di `EraCompleted` (task 035) per gestire il confine di era. |

---

## 🧩 Technical Context

**GDD §8, esempi letterali**:
> - "Achieve a biosphere with ≥3 coexisting species for 50 ticks."
> - "Grow a species that survives in the toxic zone."
> - "Trigger a bloom of a specific type."

**GDD §11**: la lista dei pannelli HUD sempre visibili include "current objective" — questo task non tocca la HUD (task 043), ma la resource `ObjectiveProgress` che espone è ciò che 043 leggerà.

**`advance_tick`** (`src/sim.rs`, task 035 la estende con `EraCompleted`): questo task aggiunge un sistema separato (non necessariamente dentro `advance_tick` stesso) che legge lo stato di `SimWorld` dopo ogni tick e aggiorna `ObjectiveProgress`.

- **Comportamento attuale**: nessun concetto di obiettivo esiste nel codice — `ui.rs:243` ha un commento placeholder letterale (`// Placeholder: objective arrives in Phase 3 (GDD §8).`) che segna dove la UI dovrà collegarsi (task 043).
- **Comportamento desiderato**: dato un `Objective` assegnato (per ora, in questo task, anche uno costruito a mano nei test — l'assegnazione procedurale è task 042), il motore traccia correttamente l'avanzamento tick per tick e produce un esito quando la condizione è soddisfatta.

---

## 🔨 Suggested Implementation

1. Definire `Objective` e `ZoneKind` (o riusare un tipo zona già esistente in `world.rs` se presente — verificare prima di introdurne uno nuovo).
2. Definire `ObjectiveProgress` con i campi necessari a tracciare il conteggio consecutivo (es. `consecutive_ticks: u32`, `satisfied: bool`).
3. Scrivere `evaluate`: per `Coexistence`, contare le specie con popolazione > 0 nella griglia; per `SurviveIn`, verificare se esiste un organismo di una specie designata nella zona indicata; per `TriggerBloom`, definire una soglia osservabile (es. popolazione di una specie sopra N in un'area in un singolo tick) — evitare di introdurre concetti nuovi non ancorati a dati già presenti in `SimWorld`.
4. Collegare `evaluate` a un sistema Bevy che gira dopo `advance_tick`, leggendo `SimWorld` in sola lettura (coerente con `SimSet::Sync`, TECH_DESIGN.md §3.3).
5. Scrivere i test con `SimWorld` costruiti a mano (pattern già usato nei test esistenti di `sim.rs`/`world.rs`).

---

## ⚠️ Constraints and Caveats

- **Non rivelare la matrice**: nessun obiettivo deve dipendere da un valore di `TagMatrix`/`MatrixKnowledge` — solo da stato osservabile (popolazione, posizione, eventi).
- **Determinismo**: `evaluate` è una funzione pura su `&SimWorld`, niente RNG proprio, niente stato esterno.
- **Non generare ancora obiettivi proceduralmente**: questo task definisce il tipo e il motore di valutazione; *quale* obiettivo assegnare a un mondo è il task 042.
- **Non toccare la UI**: il consumo di `ObjectiveProgress` per la HUD è il task 043.

---

## 🔗 Dependencies

- **Depends on**: nessuno (usa eventi Fase 2 esistenti, testabile contro il mondo hardcoded attuale).
- **Blocks**: 041 (failure conditions condivide `WorldOutcome`), 042 (worldgen genera istanze di `Objective`), 043 (HUD legge `ObjectiveProgress`).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/040-objectives-type-evaluation-engine.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
