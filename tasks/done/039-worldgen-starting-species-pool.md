# Task 039 — Worldgen: starting species pool

> **ID**: `039`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

`seed_starting_palette` (`src/world.rs`, righe ~325-356) è un placeholder esplicito del task 013: seeda sempre 2 specie fotolitiche fisse agli estremi del gradiente termico, con un commento che dice chiaramente *"Phase 3's procedural world generation replaces this with a real generator — do not extend this function, replace its call sites."*

Questo task lo sostituisce con un generatore reale che, dato il mondo generato dal task 038 (tag attivi, matrice, ambiente), produce specie di partenza coerenti. Introduce anche la distinzione tra **pool disponibile** (il superset di specie che il giocatore potrebbe scegliere di seedare, punto di aggancio per la meta-progressione del task 046) e **specie effettivamente pre-piazzate** nel mondo all'inizio della run.

---

## 📋 Acceptance Criteria

- [x] `seed_starting_palette` rimossa da `src/world.rs`; sostituita da `generate_starting_palette` in `src/worldgen.rs` — tutti i call site (`spawn_world`, `input.rs::reseed_world`, `tests/{determinism,balance,action_effects}.rs`) aggiornati, nessuna estensione del placeholder.
- [x] Il generatore produce specie deterministicamente dal seed del mondo — verificato dal test `starting_palette_is_deterministic_for_the_same_seed`.
- [x] Le specie generate rispettano i vincoli esistenti: metabolismi diversificati (Photolithic per le specie piazzate, Predator/Decomposer in alternanza per il pool extra), tag da `draw_species_tags` (1-3, dal sottoinsieme attivo del mondo), posizionamento coerente con l'ambiente generato (`temp_optimum` letto direttamente da `world.get(x, 0).temperature`, non da un valore statico ricalcolato).
- [x] Introdotto `StartingPalette { available: Vec<Species>, placed: Vec<(usize, (usize, usize))> }`: `available` (specie piazzate + extra) è sempre strettamente più grande di `placed` (solo le specie piazzate) — la distinzione è reale, non solo nominale, pronta per l'estensione del task 046.
- [x] `cargo clippy --all-targets -- -D warnings` pulito, `cargo test` verde (77 test totali).
- [x] Test: determinismo, vincoli di `TagConfig` sulle specie piazzate, posizionamento corretto sulla griglia, `available.len() > placed.len()` con almeno un metabolismo non-fotolitico nel pool.

## Nota su design: perché solo le specie piazzate sono fotolitiche

Il criterio originale chiedeva "metabolismi diversificati" senza specificare se anche le specie *piazzate* dovessero variare. Ho scelto di mantenere tutte le specie effettivamente piazzate sulla griglia come `Photolithic` (l'unico metabolismo autosufficiente dalla sola luce, senza bisogno di prede/residui già presenti, GDD §5.4) e di introdurre la diversità di metabolismo solo nel pool `available` extra (Predator/Decomposer, selezionabili dal giocatore via `Seed` ma non pre-piazzati). Questo evita di introdurre organismi pre-piazzati con morte quasi garantita nei primi tick (un predatore isolato senza prede muore in ~8 tick per la formula GDD §5.9), che avrebbe rischiato di alzare sistematicamente il tasso di estinzione scoperto dal task 038. Verificato empiricamente: i test di bilanciamento su 50 seed restano entro le soglie già impostate.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `seed_starting_palette` (righe ~325-356) e i suoi call site, da sostituire. |
| `src/worldgen.rs` | Nuovo generatore di specie di partenza + tipo pool disponibile/pre-piazzate. |

---

## 🧩 Technical Context

**`seed_starting_palette` attuale** (`src/world.rs`, righe ~325-356): seeda 2 specie fotolitiche fisse agli estremi del gradiente termico. Placeholder del task 013, esplicitamente marcato come da sostituire (non estendere) in Fase 3.

- **Comportamento attuale**: ogni run inizia con esattamente le stesse 2 specie fotolitiche, indipendentemente dal mondo generato.
- **Comportamento desiderato**: le specie di partenza sono generate dal `world_seed`, coerenti coi tag attivi/ambiente di quel mondo specifico (task 038), con varietà di metabolismo. Esiste uno shape dati che distingue "cosa il giocatore potrebbe seedare in questo mondo" da "cosa parte già piazzato", pronto per essere ampliato dagli sblocchi del task 046 (che aggiungeranno specie extra al pool disponibile senza toccare la logica di generazione di questo task).

---

## 🔨 Suggested Implementation

1. Leggere per intero `seed_starting_palette` e i suoi call site in `world.rs` prima di rimuoverla.
2. Progettare un tipo minimale, es. `struct StartingPalette { available: Vec<Species>, placed: Vec<(Species, GridPos)> }` (nomi indicativi, adattare allo stile del codice esistente) in `worldgen.rs`.
3. Scrivere la generazione: campionare metabolismi/tag/posizioni dall'RNG del mondo, rispettando `TagConfig.tags_per_species_min/max` e la coerenza con l'ambiente (es. temp_optimum distribuiti sul gradiente generato dal task 038, non fissi).
4. Sostituire i call site di `seed_starting_palette` con il nuovo generatore.
5. Aggiungere i test di determinismo e vincoli.

---

## ⚠️ Constraints and Caveats

- **Determinismo**: solo RNG interno di `SimWorld`, nessuna dipendenza da orologio/thread_rng.
- **Non introdurre ancora meccaniche di sblocco**: il pool disponibile in questo task è generato interamente dal worldgen — gli sblocchi (task 046) lo *estenderanno* in un secondo momento, questo task deve solo lasciare lo spazio per farlo senza un refactor dello shape dati.
- **Non estendere il placeholder esistente**: sostituire i call site, come indicato dal commento originale — non aggiungere rami condizionali a `seed_starting_palette`.
- **Nessun magic number fuori da `SimConfig`**: se servono nuovi coefficienti (es. numero di specie di partenza), vanno in `SimConfig`, non hardcoded in `worldgen.rs`.

---

## 🔗 Dependencies

- **Depends on**: 038 (tag attivi/ambiente del mondo).
- **Blocks**: 046 (meta-progressione estende il pool disponibile qui introdotto).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/039-worldgen-starting-species-pool.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
