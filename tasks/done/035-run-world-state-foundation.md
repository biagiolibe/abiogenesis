# Task 035 — Run/world state foundation

> **ID**: `035`
> **Category**: Architecture
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

Fase 3 ("The run", GDD §8-§10) introduce il concetto di run multi-mondo: successo → mondo successivo, fallimento → fine run. Questo task posa le fondamenta minime senza cui nessun altro task di Fase 3 può procedere:

1. Nuove varianti `GameState::{WorldCleared, Defeat}` (interstiziali, non un vero "Victory" — il GDD non descrive mai una vittoria di run, solo "successo → mondo successivo" ad libitum: la run è endless-until-failure).
2. Resource `RunProgress` che traccia in che punto della run ci troviamo (indice mondo, seed lineage, sblocchi).
3. Evento `EraCompleted`, già documentato in `TECH_DESIGN.md` §4 ("Emesso da `sim`, consumato da `ui`, run flow (Phase 3)") ma mai implementato — è il gancio con cui i task successivi (failure conditions, transizione di mondo) osserveranno la fine di un'era senza duplicare logica dentro `advance_tick`.

Questo è deliberatamente un task "silenzioso": introduce tipi e un evento, non li collega ancora a input/UI. `main.rs` NON va toccato — il bypass `Loading → Playing` resta com'è finché non esiste una UI di main menu (task futuro 044); cambiarlo ora farebbe bootare il gioco su uno stato senza UI, rompendo la verifica manuale per tutti i task intermedi di Fase 3.

---

## 📋 Acceptance Criteria

- [x] `GameState` (`src/state.rs`) ha due nuove varianti `WorldCleared` e `Defeat`, accanto a `Loading`, `MainMenu`, `Playing`.
- [x] Nuova resource `RunProgress` con almeno i campi: `run_seed: u64`, `world_index: u32`, `world_seed: u64`, `worlds_cleared: u32`, `unlocks: Unlocks` (dove `Unlocks` è una struct/enum minimale, vuota o quasi — la popolazione reale è task 046, qui serve solo il campo per non dover rifare lo shape della resource dopo). Implementata in nuovo modulo `src/run.rs` (esportato da `lib.rs`), non in `world.rs`.
- [x] `RunProgress` è inserita come resource all'avvio, tramite `RunPlugin::build` (`app.init_resource::<RunProgress>()`), registrato in `main.rs`.
- [x] Nuovo evento/`Message` `EraCompleted` (segue il pattern esistente di `OrganismDied`/`SpeciesExtinct` in `src/sim.rs`), emesso da `advance_tick` nello stesso punto in cui oggi incrementa `world.era` e richiama `next_state.set(EraState::Observing)`.
- [x] `EraCompleted` è registrato come `Message`/evento in `SimPlugin` (`app.add_message::<EraCompleted>()`), oltre che nel test manuale che costruisce un `App` a mano in `sim.rs`.
- [x] `main.rs` non introduce nuove state transition: il bypass `Loading → Playing` resta invariato — l'unica modifica è la registrazione di `RunPlugin` nella tupla dei plugin (necessaria per inserire la resource `RunProgress` all'avvio, come previsto dall'implementazione suggerita del task).
- [x] Nessun sistema esistente legge ancora `WorldCleared`/`Defeat`/`RunProgress`/`EraCompleted` per pilotare la UI o l'input — sono solo raggiungibili/emessi, non consumati.
- [x] `cargo clippy -- -D warnings` pulito.
- [x] `cargo test` verde (68 test totali, nessuna regressione).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/state.rs` | Aggiunta delle varianti `GameState::{WorldCleared, Defeat}`. |
| `src/sim.rs` | `advance_tick` (righe ~427-450): emissione di `EraCompleted` nello stesso punto in cui oggi avviene l'incremento di `world.era`; definizione del tipo evento accanto a `OrganismDied`/`SpeciesExtinct` (righe ~20-25); registrazione in `SimPlugin`. |
| `src/run.rs` (nuovo, da valutare) | Definizione di `RunProgress` e `Unlocks`, se si preferisce non sovraccaricare `world.rs`. |
| `src/main.rs` | **Solo lettura** — verificare che il bypass `OnEnter(GameState::Loading) → enter_playing` resti invariato; non aggiungere qui la registrazione delle nuove plugin/resource se non strettamente necessario (va bene farlo da un nuovo `RunPlugin`, ma senza toccare la logica di stato). |

---

## 🧩 Technical Context

**Stato attuale (`src/state.rs`, 32 righe, letto per intero):**
```rust
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
}

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Playing)]
pub enum EraState {
    Planning,
    #[default]
    Observing,
    Advancing,
}
```
`GameState::MainMenu` ed `EraState::Planning` sono oggi irraggiungibili (`#![allow(dead_code)]` in testa al file) — commentati esplicitamente come "diventano raggiungibili nelle fasi successive". Le nuove varianti `WorldCleared`/`Defeat` saranno nello stesso stato: dichiarate ma non ancora raggiunte da nessuna transizione, finché i task 041/045 non le collegano.

**`advance_tick` attuale (`src/sim.rs`, righe ~427-450):**
```rust
fn advance_tick(
    mut world: ResMut<SimWorld>,
    config: Res<SimConfig>,
    mut progress: ResMut<EraProgress>,
    mut next_state: ResMut<NextState<EraState>>,
    mut budget: ResMut<ActionBudget>,
    mut died: MessageWriter<OrganismDied>,
    mut extinct: MessageWriter<SpeciesExtinct>,
    mut adjacencies: MessageWriter<AdjacencyObserved>,
) {
    if progress.remaining() == 0 { return; }
    let events = step(&mut world, &config);
    died.write_batch(events.deaths);
    extinct.write_batch(events.extinctions);
    adjacencies.write_batch(events.adjacencies);
    progress.remaining -= 1;
    if progress.remaining() == 0 {
        world.era += 1;
        budget.refill(config.time.point_budget_per_era);
        next_state.set(EraState::Observing);
    }
}
```
`EraCompleted` va scritto nello stesso branch `if progress.remaining() == 0` (dopo `world.era += 1`), come nuovo parametro `mut era_completed: MessageWriter<EraCompleted>`. Contenuto minimo dell'evento: `struct EraCompleted { pub era: u32 }` (l'era appena conclusa), sufficiente per i consumer futuri (failure conditions leggeranno `world.era`/`RunProgress` direttamente dalla resource, non serve altro payload qui).

**`SpeciesExtinct`/`OrganismDied` come pattern di riferimento** (`src/sim.rs`, righe ~20-25):
```rust
#[derive(Message, Debug, Clone, Copy)]
pub struct SpeciesExtinct { pub species: SpeciesId }
```
`EraCompleted` segue la stessa forma (`#[derive(Message, ...)]`).

**`TECH_DESIGN.md` §4** documenta già la tabella eventi con `EraCompleted` assegnato a "run flow (Phase 3)" — questo task la rende reale, non introduce un concetto nuovo rispetto all'architettura documentata.

- **Comportamento attuale**: non esiste alcun concetto di "run" o "mondo corrente in una sequenza" — il gioco genera un solo `SimWorld` all'avvio (seed fisso `42`) e lo si può solo riseedare manualmente col tasto `r`. Non c'è modo di sapere "quanti mondi ha superato il giocatore" o "qual è il seed della run in corso".
- **Comportamento desiderato dopo questo task**: esistono i tipi (`GameState::{WorldCleared, Defeat}`, `RunProgress`, `EraCompleted`) su cui i task successivi costruiranno failure conditions, worldgen procedurale e transizione di mondo — ma il gioco si comporta esattamente come oggi dal punto di vista del giocatore (nessuna UI, nessuna transizione nuova raggiungibile).

---

## 🔨 Suggested Implementation

1. In `src/state.rs`, aggiungere `WorldCleared` e `Defeat` a `GameState`. Aggiornare eventuali `match` esaustivi altrove nel codice che elencano le varianti di `GameState` (`cargo build` li segnalerà come errori di compilazione se esistono — è il modo più affidabile per trovarli tutti).
2. Decidere dove vive `RunProgress`: se il progetto preferisce un nuovo modulo dedicato (consigliato, dato che i task 044-046 lo estenderanno con logica di transizione/meta-progressione), creare `src/run.rs` con:
   ```rust
   #[derive(Resource, Debug, Clone, Default)]
   pub struct RunProgress {
       pub run_seed: u64,
       pub world_index: u32,
       pub world_seed: u64,
       pub worlds_cleared: u32,
       pub unlocks: Unlocks,
   }

   #[derive(Debug, Clone, Default)]
   pub struct Unlocks; // popolata dal task 046
   ```
   Non serve ancora un `Plugin` dedicato se non c'è logica di sistema da registrare — può bastare `app.init_resource::<RunProgress>()` chiamato da `WorldPlugin` o da un nuovo `RunPlugin` minimale in `main.rs` (in tal caso è l'unica riga che main.rs guadagna, senza toccare la logica `Loading → Playing`).
3. In `src/sim.rs`: aggiungere `EraCompleted` accanto a `SpeciesExtinct`, aggiungere il parametro `MessageWriter<EraCompleted>` ad `advance_tick`, emetterlo subito dopo `world.era += 1`. Registrare l'evento in `SimPlugin::build` (`app.add_message::<EraCompleted>()`, stessa API usata per gli eventi esistenti — verificare il nome esatto del metodo cercando come sono registrati `OrganismDied`/`SpeciesExtinct` nello stesso file).
4. Eseguire `cargo build`, `cargo clippy -- -D warnings`, `cargo test` e correggere ogni `match` non esaustivo o warning di variante inutilizzata (`#[allow(dead_code)]` già presente in `state.rs` copre le nuove varianti finché restano irraggiungibili).

```rust
// Esempio EraCompleted, accanto a SpeciesExtinct in src/sim.rs
#[derive(Message, Debug, Clone, Copy)]
pub struct EraCompleted {
    pub era: u32,
}
```

---

## ⚠️ Constraints and Caveats

- **Determinismo (TECH_DESIGN.md §5, invariante 1)**: `RunProgress::run_seed`/`world_seed` sono dati, non generati qui — questo task non introduce ancora un generatore di seed (arriva col main menu, task 044). Se serve un valore di default per `RunProgress::default()`, usare `0`, non un clock read.
- **Nessun magic number fuori da `SimConfig`**: questo task non introduce nuovi coefficienti numerici (l'era budget vive già in `SimConfig::time`), quindi non tocca `config.rs`.
- **`sim`/`world`/`config` restano headless** (invariante 2): `EraCompleted` e `RunProgress` non devono dipendere da `bevy::render`/`bevy_egui`.
- **Non anticipare i task successivi**: non implementare qui il check di estinzione totale, il consumo di `EraCompleted` per il game over, o la UI del main menu — sono task 040/041/044/045. Questo task si ferma alla definizione dei tipi e all'emissione dell'evento.
- **Stile**: seguire le convenzioni di `TECH_DESIGN.md` e il pattern esistente per eventi/resource nel resto del codebase.

---

## 🔗 Dependencies

- **Depends on**: nessuno (primo task di Fase 3).
- **Blocks**: 040 (objectives — legge `RunProgress`/può reagire a `EraCompleted`), 041 (failure conditions — consuma `GameState::Defeat`), 044 (main menu — inizializza `RunProgress.run_seed`), 045 (transizione di mondo — consuma `GameState::WorldCleared`/`Defeat` e aggiorna `RunProgress`).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/035-run-world-state-foundation.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
