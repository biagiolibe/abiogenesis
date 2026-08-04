# Task 044 — Main menu

> **ID**: `044`
> **Category**: UI / Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

`GameState::MainMenu` esiste già (task 007) ma è irraggiungibile: `main.rs` fa `Loading → Playing` direttamente, con un commento esplicito *"Phase 0: `Loading` transitions straight to `Playing`; `MainMenu` becomes real in Phase 3."* `TECH_DESIGN.md` §2 descrive già `MainMenu` come "seed selection and run start — (Stub in Phase 0, real in Phase 3.)"

Questo task rende il main menu reale: nuovo modulo `src/menu.rs`, `main.rs` aggiornato a `Loading → MainMenu`, e la UI del menu è il punto in cui nasce `RunProgress.run_seed` — l'**unico** punto legittimo fuori dalla simulazione dove si può introdurre varietà (non è tick logic, non deve rispettare l'invariante "niente orologio di sistema nella sim" perché non è parte della sim; da lì in poi tutto deriva dall'RNG della run, mai un secondo clock read, stesso pattern già usato da `next_seed()` per il tasto `r`).

---

## 📋 Acceptance Criteria

- [ ] Nuovo modulo `src/menu.rs` con la UI del main menu (egui, coerente con lo stack esistente).
- [ ] `main.rs`: `OnEnter(GameState::Loading)` transiziona a `GameState::MainMenu` invece che direttamente a `Playing`.
- [ ] Il main menu offre almeno un'azione "nuova run" (con possibilità di specificare un seed esplicito, o generarne uno — coerente con GDD §14 "Real main menu with seed selection and sharing" citato come idea approvata in `PROJECT_PLAN.md`).
- [ ] "Nuova run": genera/accetta `run_seed`, inizializza `RunProgress` (task 035) con `world_index=0`, deriva `world_seed` dall'RNG della run (non dal `run_seed` grezzo direttamente, se questo introduce ambiguità — decidere uno schema chiaro e documentarlo nel codice), transiziona a `GameState::Playing`.
- [ ] Nuova sezione in `src/text.rs` per le stringhe del main menu (titolo, pulsanti, eventuale campo seed) — coerente col task 034.
- [ ] **Criterio di non-regressione**: una "nuova run" con un seed fissato riproduce lo stesso comportamento di gioco osservabile che il gioco ha oggi con `spawn_world(42, ...)` (quando quel seed è usato).
- [ ] `cargo clippy -- -D warnings` pulito, `cargo test` verde.
- [ ] Verifica manuale: `cargo run` fa bootare il gioco sul main menu, non su un mondo; premendo "nuova run" si arriva a un mondo giocabile.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/menu.rs` (nuovo) | UI del main menu, gestione input "nuova run". |
| `src/main.rs` | Cambio della transizione `OnEnter(GameState::Loading)` da `Playing` a `MainMenu`; registrazione del nuovo modulo/plugin. |
| `src/text.rs` | Nuova sezione stringhe main menu. |
| `src/world.rs` | `spawn_world`/`WorldPlugin::build` (righe ~249-255) — oggi chiamato da `Startup`, va spostato/reso condizionale a "nuova run avviata dal menu" invece che sempre all'avvio dell'app. |

---

## 🧩 Technical Context

**`main.rs` attuale**:
```rust
App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin { title: "Abiogenesis".into(), .. }))
    .add_plugins(EguiPlugin::default())
    .add_plugins((
        ConfigPlugin, WorldPlugin, SimPlugin, GridRenderPlugin,
        UiPlugin, NotebookPlugin, InputPlugin,
    ))
    .init_state::<GameState>()
    .add_sub_state::<EraState>()
    .add_systems(OnEnter(GameState::Loading), enter_playing)
    .run();
```
`enter_playing` è l'unica system di transizione di stato in `main.rs` — questo task la sostituisce/aggiunge una transizione verso `MainMenu`.

**`WorldPlugin::build`** (`src/world.rs`, righe ~249-255):
```rust
fn spawn_world(mut commands: Commands, config: Res<SimConfig>) {
    let mut world = SimWorld::new(42, &config);
    seed_starting_palette(&mut world, &config);
    commands.insert_resource(world);
}
```
Oggi gira in `Startup`, incondizionatamente. Con un main menu reale, la creazione del mondo deve avvenire quando il giocatore preme "nuova run", non all'avvio del processo — questo task sposta questa logica dietro l'azione del menu (probabilmente riusando/estendendo la stessa funzione che il task 045 userà per `start_world`, ma qui limitatamente alla creazione del *primo* mondo di una run).

**`reseed_world`** (`src/input.rs`, righe ~107-147, tasto `r`): pattern di riferimento per "come si genera un nuovo seed senza rompere il determinismo" — usa `world.next_seed()`, mai l'orologio di sistema.

- **Comportamento attuale**: il gioco boota direttamente su un mondo con seed fisso `42`, nessuna UI di avvio.
- **Comportamento desiderato**: il gioco boota su un main menu; il giocatore avvia esplicitamente una run, che da quel momento genera la propria sequenza di mondi in modo deterministico dal seed scelto/generato in quel momento.

---

## 🔨 Suggested Implementation

1. Creare `src/menu.rs` con un `Plugin` che aggiunge la UI egui per `OnEnter(GameState::MainMenu)`/durante `GameState::MainMenu`, e un sistema che gestisce l'input "nuova run".
2. In `main.rs`, cambiare la destinazione di `OnEnter(GameState::Loading)` a `GameState::MainMenu`.
3. Spostare la logica di `spawn_world` da `Startup` incondizionato a un sistema triggerato da `OnEnter(GameState::Playing)` (o dall'azione esplicita del menu prima della transizione), che inizializza sia `SimWorld` sia `RunProgress`.
4. Decidere lo schema seed: opzione più semplice — `run_seed` scelto/generato al menu diventa direttamente il primo `world_seed` (`world_index=0`); i mondi successivi (task 045) derivano i loro seed dall'RNG interno della run, non da un nuovo input utente.
5. Aggiungere le stringhe in `text.rs`, implementare la UI.
6. Verifica manuale.

---

## ⚠️ Constraints and Caveats

- **Determinismo**: il `run_seed` nasce al menu (presentation layer, fuori dalla sim) — da quel momento in poi nessun secondo punto del codice deve leggere l'orologio di sistema o generare un seed "fresco" indipendentemente; tutto deriva da lì.
- **`sim`/`world`/`config` restano headless**: `menu.rs` può dipendere da `bevy_egui`, ma non deve introdurre dipendenze di `world.rs`/`sim.rs` verso `menu.rs`/rendering.
- **Non implementare ancora le schermate world-cleared/defeat**: quelle sono il task 045 — questo task copre solo l'ingresso nella run (main menu → primo mondo).
- **Non implementare ancora la meta-progressione**: nessuno sblocco da mostrare qui (task 046).

---

## 🔗 Dependencies

- **Depends on**: 035 (`RunProgress`, `GameState`).
- **Blocks**: 045 (la transizione di mondo riusa lo schema di creazione mondo introdotto qui).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/044-main-menu.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
