# Task 041 — Failure conditions

> **ID**: `041`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

GDD §8, failure conditions [DECIDED]: estinzione totale → fallimento immediato (il floor ovvio); budget di ere per mondo generoso ma finito (baseline: 40 ere nei mondi iniziali, verso 25 nei mondi tardivi) — un giocatore bloccato fallisce invece di macinare all'infinito, ed è questo che dà tensione roguelike.

Oggi nessuno dei due controlli esiste: `SpeciesExtinct` (evento esistente, task 018) è emesso per-specie ma nessun sistema verifica se *tutte* le specie sono estinte simultaneamente; `TimeConfig::era_budget_early/late` è definito in `SimConfig` ma mai letto da `advance_tick`. Questo task collega entrambi al tipo `WorldOutcome` condiviso col task 040 e a `GameState::Defeat` (task 035).

---

## 📋 Acceptance Criteria

- [ ] Tipo condiviso con 040, es. `enum WorldOutcome { Ongoing, Cleared, Failed(FailureReason) }` (o forma equivalente) — se 040 non lo ha già introdotto, definirlo qui in `objectives.rs` o in un modulo condiviso.
- [ ] Controllo di **estinzione totale**: se l'occupazione della griglia è 0 (nessun organismo vivo), l'esito diventa `Failed` nello stesso tick in cui accade — con una guardia esplicita contro il falso positivo del frame prima del seeding iniziale (il mondo parte vuoto per un istante prima che le specie di partenza vengano piazzate).
- [ ] Controllo di **budget di ere esaurito**: `advance_tick` (o un sistema immediatamente successivo) confronta `world.era` con `WorldParams.era_budget` (task 037/038) — se il budget è esaurito senza che l'obiettivo sia stato soddisfatto, l'esito diventa `Failed`.
- [ ] Quando l'esito è `Failed`, il gioco transiziona a `GameState::Defeat` (varianti introdotte nel task 035).
- [ ] Test unitario: mondo hand-built che esaurisce il budget di ere senza soddisfare l'obiettivo → `Failed`.
- [ ] Test unitario: mondo hand-built che raggiunge occupazione zero → `Failed` nello stesso tick, non un tick di ritardo.
- [ ] Nessun falso positivo di estinzione totale nel tick di inizializzazione (prima che `seed_starting_palette`/il generatore del task 039 abbia piazzato le specie).
- [ ] `cargo clippy -- -D warnings` pulito, `cargo test` verde.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `advance_tick` — punto dove aggiungere il controllo di budget di ere; eventuale hook per l'estinzione totale. |
| `src/objectives.rs` | Tipo `WorldOutcome` condiviso, se non già presente dal task 040. |

---

## 🧩 Technical Context

**`advance_tick` attuale** (`src/sim.rs`, righe ~427-450, vedi anche task 035 per l'estensione con `EraCompleted`):
```rust
if progress.remaining() == 0 {
    world.era += 1;
    budget.refill(config.time.point_budget_per_era);
    next_state.set(EraState::Observing);
}
```
Questo è il punto naturale per il controllo di budget: subito dopo `world.era += 1`, confrontare con `WorldParams.era_budget` del mondo corrente.

**`SpeciesExtinct`** (`src/sim.rs`, righe ~20-25): emesso per-specie quando l'ultima organism di quella specie muore (rilevato via diff di popolazione pre/post-tick, riga ~286). Non implica estinzione *totale* — serve un controllo separato sull'occupazione complessiva della griglia.

- **Comportamento attuale**: un mondo può proseguire all'infinito anche con zero organismi vivi o ben oltre qualunque budget ragionevole di ere — non esiste alcun "game over".
- **Comportamento desiderato**: il giocatore riceve un fallimento esplicito e tempestivo in entrambi i casi, coerente col design "roguelike tension" del GDD §8.

---

## 🔨 Suggested Implementation

1. Verificare/definire `WorldOutcome` in coordinamento con quanto prodotto dal task 040 (se eseguito prima, riusarne il tipo; se questo task viene eseguito per primo tra i due, definirlo qui e farlo riusare da 040 — decidere in base all'ordine di esecuzione reale).
2. Aggiungere un sistema (o estendere quello del task 040) che, dopo `advance_tick`, controlla l'occupazione totale della griglia (`world.cells` — verificare il nome esatto del campo/metodo di conteggio occupazione in `world.rs`).
3. Nello stesso branch `if progress.remaining() == 0` di `advance_tick`, aggiungere il confronto `world.era >= era_budget_corrente` (da `WorldParams`, disponibile solo dopo il task 038 — se questo task viene eseguito prima di 038/039 essere completati nel branch di sviluppo, usare `WorldParams` con un `world_index` fisso/di test finché l'integrazione reale non è disponibile, ma il codice di produzione deve leggere il valore reale).
4. Collegare l'esito `Failed` alla transizione `next_state.set(GameState::Defeat)`.
5. Scrivere i test con mondi hand-built (pattern esistente in `sim.rs`).

---

## ⚠️ Constraints and Caveats

- **Determinismo**: nessun controllo qui introduce RNG o stato esterno.
- **Ordine di valutazione**: l'estinzione totale deve essere rilevabile anche a metà di un'era animata (`EraState::Advancing`), non solo al confine di era — altrimenti il giocatore vedrebbe una griglia vuota per fino a 25 tick prima del fallimento.
- **Non duplicare la logica di reset**: questo task rileva il fallimento e transiziona lo stato; il *reset* effettivo (nuovo mondo, nuova run) è responsabilità del task 045 (`start_world`), non di questo task.
- **Guardia anti-falso-positivo**: il tick immediatamente successivo alla creazione di `SimWorld`, prima che le specie di partenza siano piazzate, ha occupazione zero per costruzione — non deve essere interpretato come sconfitta.

---

## 🔗 Dependencies

- **Depends on**: 040 (tipo `WorldOutcome`/`Objective` condiviso).
- **Blocks**: 045 (la transizione di mondo consuma `GameState::Defeat`).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/041-failure-conditions.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
