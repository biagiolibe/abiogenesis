# Technical Design Document — Abiogenesis

Questo documento descrive l'architettura tecnica e le scelte implementative del progetto.
Il **design di gioco** vive in [`abiogenesis-gdd.md`](abiogenesis-gdd.md) (v0.4) ed è la fonte di verità: qui non si duplica, si rimanda.

---

## 1. Stack Tecnologico

- **Linguaggio**: Rust (Edizione 2021)
- **Toolchain**: **1.97.1**, pinnata in `rust-toolchain.toml`
  *Vincolo:* Bevy 0.19 richiede Rust ≥ 1.95.0.
- **Engine**: **Bevy 0.19.0**
- **UI**: **`bevy_egui` 0.41.1** (egui 0.35.0)
- **RNG**: **`rand` 0.10.2**, con seed esplicito conservato nello stato del mondo
- **Fisica**: nessuna (simulazione a griglia, non continua)

> Versioni risolte da `cargo add` in task 001 (vedi `Cargo.lock`).
> Attenzione: l'API di `rand` 0.10 differisce da 0.8 (`thread_rng` → `rng`, `gen` → `random`, tratti rinominati) — da tenere a mente in task 003 quando si inizializza l'RNG in `SimWorld`.

---

## 2. Stati del Gioco (`GameState`)

```
Loading  →  MainMenu  →  Playing
                            │
                            └── EraState (sotto-stato)
                                Planning → Advancing → Observing ─┐
                                    ▲                             │
                                    └─────────────────────────────┘
```

- `Loading`: inizializzazione delle resource e generazione del mondo.
- `MainMenu`: scelta del seed e avvio della run. *(Stub in Fase 0, reale in Fase 3.)*
- `Playing`: ciclo di gioco principale.

Il sotto-stato **`EraState`** mappa 1:1 sul ciclo descritto in GDD §16.4 — ed è questa corrispondenza a renderlo la spina dorsale dello scheduling:

| Stato | GDD §16.4 | Cosa accade |
|---|---|---|
| `Planning` | PIANIFICA | Il giocatore mette in coda azioni entro il budget. La simulazione è ferma. |
| `Advancing` | AVANZA ERA | `ERA_TICKS` tick avanzano **animati uno per uno**. Input di gioco ignorati. |
| `Observing` | OSSERVA & REGISTRA | Il giocatore legge il risultato e il taccuino. La simulazione è ferma. |

In Fase 0 esistono `Advancing` e `Observing`; `Planning` diventa significativo in Fase 2, con le azioni.

---

## 3. Architettura ECS & Moduli

### 3.1 La decisione strutturale: griglia come `Resource`

**La simulazione non è modellata in ECS. Vive in una `Resource` `SimWorld` con array densi e doppio buffer.** Le entità Bevy esistono **solo per la resa**: uno sprite per cella, sincronizzato in sola lettura.

Il motivo è il **determinismo** imposto da GDD §5.7, che qui non è un lusso ma un requisito funzionale: serve a debuggare l'emergenza, a riprodurre i bug e a rendere ripetibile il tuning di §5.8. L'iterazione parallela delle query ECS è il modo più rapido per perderlo. Una griglia densa iterata in ordine di indice lo garantisce per costruzione, e in più:

- l'algoritmo del tick (GDD §5.6) è uno **sweep su lattice con vicinato di Moore**: accesso per indice, non per entità;
- la logica resta **Rust puro**, eseguibile **senza `App` Bevy** → i test di determinismo e bilanciamento girano headless e veloci;
- niente overhead di 1536 entità mutate a ogni tick.

Bevy fornisce ciò per cui è bravo: scheduling, stati, plugin, input, finestra, resa, UI.

### 3.2 Struttura dei Plugin

Ogni modulo ha il proprio `Plugin` per incapsulare i sistemi.

| Plugin | Modulo | Responsabilità |
|---|---|---|
| `ConfigPlugin` | `config` | Resource `SimConfig`: tutti i coefficienti di GDD §5.9 in un unico punto |
| `WorldPlugin` | `world` | Resource `SimWorld`: griglia, registro specie, RNG seedato, contatori tick/era; generazione del mondo |
| `SimPlugin` | `sim` | Avanzamento dei tick; invoca la logica pura di `sim::step` |
| `GridRenderPlugin` | `render` | Sprite della griglia, camera 2D, sincronizzazione stato → colori |
| `UiPlugin` | `ui` | Pannelli `bevy_egui`: HUD e (Fase 2) taccuino |
| `InputPlugin` | `input` | Tastiera/mouse → intenzioni di gioco |

Moduli previsti nelle fasi successive: `notebook` (Fase 2), `actions` (Fase 2), `worldgen` (Fase 3).

### 3.3 Ordinamento dei Sistemi (`SystemSets`)

L'ordine di esecuzione garantito è:

`SimSet::Advance` → `SimSet::Sync` → `SimSet::Ui`

- **`Advance`** — avanza la simulazione. Gira in `FixedUpdate`, con run condition su `EraState::Advancing`.
- **`Sync`** — legge `SimWorld` e aggiorna i colori degli sprite. **Sola lettura sullo stato di simulazione.**
- **`Ui`** — pannelli egui. Legge tutto, scrive solo intenzioni.

### 3.4 Modello del tempo

GDD §4 richiede che l'era sia un blocco di `ERA_TICKS = 25` tick **animati uno per uno**, con il controllo a scatti deliberati.

Realizzazione: il sistema di avanzamento gira in **`FixedUpdate`**, con timestep configurabile (= velocità dell'animazione, non velocità della logica), attivo solo sotto `EraState::Advancing`. Una resource `EraProgress` conta i tick residui; a zero, transizione a `Observing`.

Il tasto di singolo tick invoca `sim::step` esattamente una volta, senza passare da `Advancing`.

---

## 4. Convenzioni di Sviluppo

### Gestione degli Eventi

Usare gli eventi Bevy per disaccoppiare i moduli. Definiti fin dalla Fase 0 come punto d'innesto, anche se il consumatore arriva dopo:

| Evento | Emesso da | Consumato da |
|---|---|---|
| `TickCompleted` | `sim` | `ui` (HUD), `notebook` (Fase 2) |
| `EraCompleted` | `sim` | `ui`, flusso della run (Fase 3) |
| `OrganismDied` | `sim` | `notebook` (Fase 2) |
| `SpeciesExtinct` | `sim` | `notebook`, condizioni di fallimento (Fase 3) |

Sono le fondamenta del **log delle osservazioni** di GDD §7: il taccuino si costruisce consumando eventi, non ispezionando la griglia.

### FixedUpdate vs Update

- **`FixedUpdate`**: avanzamento della simulazione (`SimSet::Advance`).
- **`Update`**: resa, UI, input.

### Inserimento Asset

Nessun asset artistico: le celle sono sprite quadrati colorati (GDD, pilastro 3). Non serve un `GameAssets` centralizzato in Fase 0.

**Config ricaricabile a caldo:** GDD §5.6 chiede coefficienti *"idealmente ricaricabili a caldo"* — con `bevy_asset` è quasi gratis. In Fase 0 `SimConfig` è costruita da costanti in `config.rs`; nella fase di tuning migra a un asset RON con hot-reload. La struttura va predisposta ora (una `Resource` unica, letta e mai duplicata), l'implementazione no.

### Stile

- **Codice e commenti in inglese** (GDD §12); documenti Meridian e GDD in italiano.
- `cargo fmt` e `cargo clippy -- -D warnings` puliti prima di chiudere un task.
- Test unitari accanto al codice; test di determinismo e bilanciamento in `tests/`.

---

## 5. Invarianti Architetturali

Regole che nessun task può violare. Se un task sembra richiederlo, il task è sbagliato.

1. **Determinismo** (GDD §5.7) — stesso seed ⇒ stessa storia, sempre.
   - L'RNG vive in `SimWorld`. Niente `rand::rng()` / `thread_rng` nella logica.
   - Niente iterazione su `HashMap`/`HashSet` in punti che influenzano la simulazione.
   - Niente orologio di sistema, niente `Time` di Bevy dentro la logica del tick.
   - **Niente query parallele dentro la simulazione.**
2. **`sim` non dipende dalla resa** — nessun `use bevy::render` / `bevy_egui` nei moduli `sim`, `world`, `config`. La simulazione deve girare headless: è il presupposto dei test e del tuning.
3. **Coefficienti centralizzati** (GDD §5.6) — nessun numero magico sparso nel codice. Tutto in `SimConfig`.
4. **Effetto matrice additivo e lineare** (GDD §5.6) — è una scelta di *design*, non di tuning: l'additivo è ciò che rende la matrice **deducibile** dal giocatore. Non va "migliorato" in moltiplicativo.

---

## 6. Meccaniche Core (Dettagli)

Il grosso è già specificato nel GDD e **non va duplicato qui**:

| Meccanica | Riferimento |
|---|---|
| Algoritmo del tick (7 passi) | GDD §5.6 |
| Costanti numeriche di baseline | GDD §5.9 |
| Anti-degenerazione (ciclicità, nicchie, carrying capacity) | GDD §5.8 |
| Tag e matrice nascosta | GDD §5.5 |
| Modello di conferma del taccuino | GDD §7 |

Qui vive solo ciò che il GDD lascia aperto all'implementazione.

### Ordine di elaborazione del tick

GDD §5.6 marca questo punto come `[PROPOSTA]`, lasciando la scelta all'implementazione tra iterazione mescolata con guardia "nato/agito in questo tick" e doppio buffer.

**Deciso: doppio buffer** (snapshot → next). Si legge dallo snapshot immutabile del tick precedente e si scrive nel buffer successivo; a fine tick i due si scambiano.

Motivo: è la via più semplice al determinismo (invariante 1). Nessuna guardia da mantenere, nessuna dipendenza dall'ordine di visita, i neonati non possono agire nello stesso tick per costruzione. Costa un secondo buffer di griglia — irrilevante a 48×32.

Attenzione al caso della **riproduzione**: due genitori possono voler occupare la stessa cella vuota nello stesso tick. La risoluzione va fatta in ordine di indice deterministico (primo arrivato in ordine di scansione), mai lasciata all'ordine di iterazione.

---

*Ultima revisione: 2026-08-01*
