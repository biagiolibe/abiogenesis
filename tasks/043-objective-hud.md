# Task 043 — Objective HUD

> **ID**: `043`
> **Category**: UI
> **Priority**: 🟡 P2
> **Estimate**: ~1-2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

GDD §11 elenca "current objective" tra i pannelli HUD sempre visibili. `ui.rs:243` ha già un commento placeholder letterale (`// Placeholder: objective arrives in Phase 3 (GDD §8).`) nel gruppo HUD popolazione/seed-palette, subito prima della riga dei tasti di aiuto — questo task lo riempie con un pannello obiettivo corrente + barra di progresso, leggendo `ObjectiveProgress` (task 040).

---

## 📋 Acceptance Criteria

- [ ] Il placeholder a `ui.rs:243` è sostituito da un pannello che mostra: il testo dell'obiettivo corrente, e una barra di progresso verso il suo soddisfacimento.
- [ ] La barra di progresso riusa il pattern visivo già stabilito dalla barra `ActionBudget` (`ui.rs:171, 202-212`, task 030) — coerenza visiva con l'HUD esistente, non un widget nuovo.
- [ ] Ogni stringa mostrata passa da `src/text.rs` (nuova sezione, es. `// --- Objective HUD (ui.rs::objective_panel) ---`), coerente col task 034 — nessuna stringa hardcoded in `ui.rs`.
- [ ] Il pannello è sempre visibile durante `GameState::Playing` (coerente con GDD §11).
- [ ] Il testo dell'obiettivo è generato a partire dai dati di `Objective`/`ObjectiveProgress` (task 040) — `text.rs` non accede a `SimWorld` direttamente (vincolo esistente del modulo, vedi header di `text.rs`), quindi la formattazione dei valori concreti (es. "3 specie coesistenti da 12 tick su 50") avviene in `ui.rs`, che poi chiama una funzione parametrizzata di `text.rs` per il template della frase.
- [ ] `cargo clippy -- -D warnings` pulito.
- [ ] Verifica manuale: avviare il gioco (con un obiettivo di test se il worldgen procedurale non è ancora integrato) e osservare che il pannello si aggiorna in tempo reale seguendo `ObjectiveProgress`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | Riga 243 (placeholder), pattern della barra `ActionBudget` (righe 171, 202-212) da imitare. |
| `src/text.rs` | Nuova sezione per le stringhe/template dell'obiettivo. |

---

## 🧩 Technical Context

**`text.rs`** (153 righe, task 034): modulo di sole costanti/funzioni pure per testo player-facing, organizzato in blocchi `// --- Section (source_file::fn) ---`. Non accede mai a `SimWorld` direttamente — i dati derivati dallo stato vivono altrove, il modulo fornisce solo stringhe/template parametrizzati (es. `extinction_message(species_id)`, pattern da imitare per un ipotetico `objective_progress_line(current, target)`).

- **Comportamento attuale**: nessun pannello obiettivo esiste — il commento a `ui.rs:243` è l'unico segnale che questo lavoro è previsto.
- **Comportamento desiderato**: il giocatore vede sempre, durante `Playing`, quale obiettivo deve soddisfare e quanto è vicino a soddisfarlo, con lo stesso linguaggio visivo delle altre barre HUD.

---

## 🔨 Suggested Implementation

1. Leggere `ui.rs` attorno alla riga 243 per capire esattamente il layout del gruppo HUD in cui va inserito il pannello.
2. Leggere il pattern della barra `ActionBudget` (righe 171, 202-212) per riusarne la struttura (widget egui, colori, layout).
3. Aggiungere in `text.rs` le funzioni/costanti necessarie per il testo dell'obiettivo (nome variante, descrizione parametrizzata, eventuale messaggio di "obiettivo soddisfatto").
4. Implementare il pannello in `ui.rs`, leggendo `ObjectiveProgress` come resource.
5. Verifica manuale con `cargo run`.

---

## ⚠️ Constraints and Caveats

- **`sim`/`world`/`config` restano headless**: questo task tocca solo `ui.rs`/`text.rs`, non deve introdurre dipendenze da `bevy_egui` in `objectives.rs`.
- **Coerenza con task 034**: nessuna stringa nuova hardcoded in `ui.rs` — tutto passa da `text.rs`.
- **Non anticipare le schermate di vittoria/sconfitta**: quelle sono task 045, questo task riguarda solo l'HUD durante il gioco attivo.

---

## 🔗 Dependencies

- **Depends on**: 040 (`ObjectiveProgress`).
- **Blocks**: nessuno (foglia del grafo, non blocca altri task).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/043-objective-hud.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
