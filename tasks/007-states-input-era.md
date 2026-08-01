# Task 007 — `GameState`/`EraState`, input, era animata

> **ID**: `007`
> **Categoria**: Feature
> **Priorità**: 🟡 P2
> **Stima**: ~2h
> **Assegnato a**: non assegnato
> **Sessione**: —

---

## 🎯 Obiettivo

Dare al giocatore il **controllo del tempo**: premere `space` fa avanzare un'era di `ERA_TICKS = 25` tick **animati uno per uno**, poi il gioco torna in attesa.

Questo task chiude il core loop della Fase 0. Il modello del tempo del GDD §4 non è un dettaglio di comodo: fa **coincidere il modello del tempo col loop mentale** del giocatore — pianifichi, esegui, osservi.

---

## 📋 Acceptance Criteria

- [ ] Esistono `GameState` (`Loading` → `MainMenu` → `Playing`) e il sotto-stato `EraState` (`Planning` / `Advancing` / `Observing`).
- [ ] `space` in `Observing` (o `Planning`) avvia un'era: transizione ad `Advancing`.
- [ ] L'era avanza di **esattamente 25 tick**, uno per frame fisso, **visibili come animazione**; poi torna a `Observing`.
- [ ] `s` avanza di **un solo tick**, senza passare da `Advancing`.
- [ ] `r` rigenera il mondo con un nuovo seed.
- [ ] `Esc` esce.
- [ ] Gli input di avanzamento sono **ignorati durante `Advancing`** (niente ere accodate per errore).
- [ ] Il sistema di simulazione **non gira** fuori da `Advancing`.
- [ ] Seminando un fotolitico in zona luminosa e premendo `space`, si vede una fioritura crescere tick per tick.
- [ ] `cargo clippy -- -D warnings` pulito.

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `src/input.rs` | `InputPlugin`, mappatura tasti |
| `src/sim.rs` | `SimPlugin`, run condition e `EraProgress` |
| `src/world.rs` | Reset/reseed |
| `src/main.rs` | Registrazione degli stati |

---

## 🧩 Contesto Tecnico

- **Comportamento attuale**: `step()` esiste ed è corretto (task 005), la griglia si vede (task 006), ma nulla lo invoca in modo controllato.
- **Comportamento desiderato**: il giocatore governa l'avanzamento del tempo.

### GDD §4 — Il modello delle ere

> Il tempo avanza a **ere**: il giocatore mette in coda una o più azioni, poi fa avanzare la simulazione di *N* tick in blocco e osserva il risultato.
>
> - **Animazione durante l'era:** l'avanzamento dei tick viene mostrato tick-per-tick (rapido), così si conserva la sensazione di "sistema che respira", ma il *controllo* resta a scatti deliberati.
> - **Lunghezza dell'era:** `ERA_TICKS = 25` come default, regolabile.

### Il ciclo (GDD §16.4)

```
PIANIFICA (budget azioni)  →  [SPACE]  →  AVANZA ERA (25 tick animati)  →  OSSERVA & REGISTRA
      ▲                                                                            │
      └────────────────────────────────────────────────────────────────────────────┘
```

`EraState` mappa 1:1 su questo ciclo (`TECH_DESIGN.md` §2). In Fase 0 `Planning` è di fatto vuoto — diventa significativo in Fase 2, con le azioni.

### Realizzazione (`TECH_DESIGN.md` §3.4)

Il sistema di avanzamento gira in **`FixedUpdate`** con timestep configurabile — che qui regola la **velocità dell'animazione**, non la velocità della logica. Una resource `EraProgress` conta i tick residui; a zero, transizione a `Observing`.

---

## 🔨 Implementazione Suggerita

1. **Stati**

   ```rust
   #[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
   pub enum GameState {
       #[default]
       Loading,
       MainMenu,
       Playing,
   }

   /// Mirrors the player-facing loop of GDD 16.4.
   #[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
   #[source(GameState = GameState::Playing)]
   pub enum EraState {
       Planning,
       #[default]
       Observing,
       Advancing,
   }
   ```

   In Fase 0 `Loading` può transitare direttamente a `Playing`; `MainMenu` resta uno stub (diventa reale in Fase 3).

2. **Contatore dell'era**

   ```rust
   /// Ticks left in the era currently being animated.
   #[derive(Resource, Default)]
   pub struct EraProgress {
       remaining: u32,
   }
   ```

3. **Avvio dell'era** — su `space`, solo se **non** si è già in `Advancing`:

   ```rust
   fn start_era(/* ... */) {
       progress.remaining = config.time.era_ticks;
       next_state.set(EraState::Advancing);
   }
   ```

4. **Avanzamento** — in `FixedUpdate`, con run condition sullo stato:

   ```rust
   app.add_systems(
       FixedUpdate,
       advance_tick
           .in_set(SimSet::Advance)
           .run_if(in_state(EraState::Advancing)),
   );
   ```

   `advance_tick` chiama `step`, decrementa `remaining` e, a zero, incrementa `world.era` e passa a `Observing`.

5. **Timestep.** Impostare `Time<Fixed>` a un ritmo che renda l'era percepibile ma rapida: **~20 tick/secondo** fa durare un'era circa 1.2 s, coerente con l'"avanzamento rapido" del GDD §4. Il valore va in `SimConfig` — è una manopola di *feel*, quindi da tarare.

6. **Singolo tick.** `s` chiama `step` una volta e basta, senza toccare gli stati: serve all'osservazione fine e al debug (GDD §11).

7. **Reset.** `r` ricostruisce `SimWorld` con un nuovo seed. Il seed successivo va derivato **dall'RNG del mondo corrente**, non dall'orologio di sistema, per non introdurre non-determinismo (invariante 1). In alternativa, un contatore incrementale sul seed iniziale.

8. **Semina di partenza.** Perché l'animazione mostri qualcosa, il mondo deve nascere con almeno un organismo: seminare un fotolitico al centro della fascia luminosa alla creazione del mondo. È un provvisorio della Fase 0 — l'azione *semina* vera arriva in Fase 1 — quindi va marcato con un commento.

---

## ⚠️ Vincoli e Attenzioni

- **Non accodare ere.** Se `space` viene premuto durante `Advancing`, va ignorato: la run condition sullo stato è la difesa più semplice.
- **Il timestep governa l'animazione, non la logica.** Cambiarlo deve alterare solo la velocità di riproduzione, mai il risultato della simulazione. Se cambiando il timestep cambia lo stato finale, l'invariante 1 è stata violata da qualche parte.
- **Esattamente 25 tick per era.** Contare sul tempo trascorso invece che sui tick porta a ere di lunghezza variabile e distrugge la riproducibilità.
- **Niente `Time` di Bevy dentro `step`** (invariante 1). Il tempo di Bevy decide *quando* chiamare `step`, mai *cosa* fa.
- `q` come tasto di uscita era previsto dal GDD v0.3 ma è stato rimosso in v0.4: serve libero per l'input testuale futuro. Resta `Esc`.

---

## 🔗 Dipendenze

- **Dipende da**: 005, 006
- **Blocca**: 008

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/007-states-input-era.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
