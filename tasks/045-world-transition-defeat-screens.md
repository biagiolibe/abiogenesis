# Task 045 — World-cleared/defeat screens + world transition

> **ID**: `045`
> **Category**: Feature / UI
> **Priority**: 🔴 P1
> **Estimate**: ~3h (task di convergenza — integra quasi tutti i task precedenti di Fase 3)
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

Questo è il task di convergenza della Fase 3: collega worldgen (038/039/042), obiettivi/fallimento (040/041), stato/run (035) e main menu (044) in un ciclo completo. Introduce le schermate interstiziali `GameState::WorldCleared` (successo → mondo successivo, più tag, matrice più cattiva, ambiente più ostile — GDD §8) e `GameState::Defeat` (fine run, ritorno al main menu).

Il punto tecnico centrale è estrarre `reseed_world` (`src/input.rs`, righe ~107-147, il codice dietro il tasto `r`) in una funzione condivisa `start_world(&mut World, world_index, seed)` che resetta **tutto** ciò che il tasto `r` resetta oggi — `MatrixKnowledge`, `ObservationLog`, `ActionBudget`, `SelectedSpecies`, `SpliceDraft`, `PlayerPlacedCells` — **più** `ObjectiveProgress` (task 040) e `world.era = 0`, applicando `WorldParams`/l'obiettivo del nuovo mondo (038/042). Sia il tasto `r` sia la transizione `WorldCleared`/nuova run la chiamano: **un solo punto di verità per il reset**, niente logica duplicata tra i due percorsi.

---

## 📋 Acceptance Criteria

- [ ] Funzione condivisa `start_world(...)` (nome indicativo, adattare allo stile del codice) che effettua il reset completo elencato sopra, riusata sia dal tasto `r` (`input.rs`) sia dalla transizione di mondo di questo task.
- [ ] Interstiziale `GameState::WorldCleared`: mostrato quando `evaluate` (task 040) produce `Cleared`; incrementa `RunProgress.world_index` e `RunProgress.worlds_cleared`; genera il mondo successivo tramite `start_world` con i parametri di `WorldParams(world_index + 1)` (037/038); un'azione esplicita (o un timer/tasto) porta a `GameState::Playing` sul nuovo mondo.
- [ ] Interstiziale `GameState::Defeat`: mostrato quando `evaluate`/le failure conditions (task 041) producono `Failed`; riporta a `GameState::MainMenu` (non a `Playing`) — la run è finita, una nuova run richiede di passare dal menu.
- [ ] Entrambe le schermate hanno testo in `src/text.rs` (nuova sezione), coerente col task 034.
- [ ] Nessuna duplicazione: il tasto `r` e la transizione di mondo chiamano la stessa funzione di reset — verificabile leggendo il diff, non due implementazioni parallele.
- [ ] **Criterio end-to-end**: una run avviata con `run_seed` pinnato (task 044) riproduce la stessa sequenza di mondi (tag attivi, matrice, ambiente, obiettivo) su almeno 2 transizioni consecutive — verificabile con un test che avanza la run programmaticamente per due cicli mondo-superato.
- [ ] `cargo clippy -- -D warnings` pulito, `cargo test` verde.
- [ ] Verifica manuale: `cargo run`, avviare una run, superare un mondo (o forzarlo per il test manuale), osservare l'interstiziale, osservare il mondo successivo con parametri di difficoltà coerenti con `world_index=1`; far fallire un mondo, osservare la schermata di sconfitta, tornare al menu.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `reseed_world` (righe ~107-147) — estratta nella funzione condivisa `start_world`, il tasto `r` aggiornato per chiamarla. |
| `src/state.rs` | Uso effettivo delle varianti `WorldCleared`/`Defeat` (già dichiarate dal task 035). |
| `src/run.rs` / `src/menu.rs` (o nuovo modulo dedicato) | Logica di transizione mondo, UI delle due schermate. |
| `src/text.rs` | Nuove sezioni per i testi delle due schermate. |

---

## 🧩 Technical Context

**`reseed_world`** (`src/input.rs`, righe ~107-147, tasto `r`): ricostruisce `SimWorld` da un nuovo seed (preso da `world.next_seed()`, mai dall'orologio di sistema), poi resetta esplicitamente: `MatrixKnowledge`, `ObservationLog` (notebook), `ActionBudget`, `SelectedSpecies`, `SpliceDraft`, `PlayerPlacedCells`. Questo è il precedente concreto più vicino a "cosa deve fare `start_world`" — la funzione di questo task è una generalizzazione di questo codice, non una riscrittura da zero.

- **Comportamento attuale**: l'unico modo di ottenere un "nuovo mondo" è il tasto `r`, che riparte sempre dagli stessi parametri di difficoltà (non esiste un concetto di "mondo successivo in una run" prima di questo task).
- **Comportamento desiderato**: superare l'obiettivo di un mondo porta automaticamente (dietro un'interazione esplicita del giocatore) al mondo successivo, più difficile; fallire riporta al main menu, chiudendo la run. Il tasto `r` continua a funzionare come "reseed manuale" ma tramite lo stesso meccanismo di reset, non una copia.

---

## 🔨 Suggested Implementation

1. Leggere per intero `reseed_world` in `input.rs` prima di estrarla.
2. Definire `start_world(world: &mut SimWorld, ..., world_index: u32, seed: u64, config: &SimConfig)` (firma indicativa) che fa: ricostruzione `SimWorld` dal seed e da `WorldParams(world_index)` (038), reset delle resource elencate, applicazione dell'obiettivo generato (042), reset `world.era = 0`.
3. Aggiornare il tasto `r` in `input.rs` per chiamare `start_world` con lo stesso `world_index` corrente (reseed dello stesso mondo, non avanzamento) — verificare che il comportamento visibile del tasto `r` resti quello atteso (rigenera il mondo corrente, non avanza la run).
4. Implementare la transizione `WorldCleared`: sistema che osserva l'esito `Cleared` (da 040/041), incrementa `RunProgress`, chiama `start_world` con `world_index + 1`, transiziona lo stato.
5. Implementare la transizione `Defeat`: sistema che osserva l'esito `Failed`, transiziona a `GameState::Defeat`, e da lì (su interazione) a `GameState::MainMenu`.
6. Scrivere il test end-to-end di riproducibilità su 2 transizioni.
7. Verifica manuale completa.

---

## ⚠️ Constraints and Caveats

- **Nessuna duplicazione di reset**: è il vincolo centrale di questo task — se il tasto `r` e la transizione di mondo finiscono per avere due implementazioni simili ma distinte, il task non è completo.
- **Determinismo end-to-end**: il criterio di accettazione sulla riproducibilità su 2 transizioni è il test più importante di tutta la Fase 3 — se fallisce, probabilmente c'è una fonte di non-determinismo introdotta in uno dei task precedenti (RNG esterno, iterazione `HashMap`, ecc.), da investigare prima di considerare questo task chiuso.
- **`Defeat` non torna a `Playing`**: torna a `MainMenu` — la run è conclusa, non è un game over che si può "continuare".

---

## 🔗 Dependencies

- **Depends on**: 035, 038, 039, 040, 041, 042, 044.
- **Blocks**: 046 (meta-progressione si aggancia alla transizione di run qui introdotta).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/045-world-transition-defeat-screens.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
