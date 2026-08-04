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

- [ ] `seed_starting_palette` sostituita da un generatore reale in `src/worldgen.rs` (non un'estensione della funzione esistente — i call site vengono sostituiti, coerente col commento originale).
- [ ] Il generatore produce specie deterministicamente dal `world_seed` (stesso seed → stesse specie).
- [ ] Le specie generate rispettano i vincoli esistenti già presenti in `SimConfig`/`world.rs`: metabolismi diversificati (non tutte fotolitiche come nel placeholder), tag assegnati dal sottoinsieme attivo del mondo (`TagConfig.tags_per_species_min/max`, 1-3 tag), posizionamento coerente con l'ambiente generato (es. non tutte nello stesso punto del gradiente termico).
- [ ] Introdotto un tipo/campo esplicito che distingue **pool disponibile** (superset di specie selezionabili) da **specie pre-piazzate** all'avvio del mondo — anche se nel Fase 3 minimo i due insiemi coincidono, la distinzione deve esistere nello shape dei dati per essere estesa dal task 046 senza un secondo refactor.
- [ ] `cargo clippy -- -D warnings` pulito, `cargo test` verde.
- [ ] Test: determinismo (stesso seed → stessa palette), rispetto dei vincoli di `TagConfig`.

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
