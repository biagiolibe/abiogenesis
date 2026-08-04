# Task 046 — Minimal meta-progression

> **ID**: `046`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

GDD §10 [DECIDED light / persistenza rimandata post-MVP]: progressione tra le run deliberatamente leggera — sbloccare più specie di partenza o strumenti (es. un'azione extra, o un tag noto). La matrice rimane sempre da decifrare da zero: non si sbloccano "risposte", si sbloccano *capacità*. Persistenza esplicitamente rimandata: l'MVP è **senza persistenza su disco** — tutto vive nella sessione di processo corrente, "banale da aggiungere dopo, non vincola l'architettura".

Questo task chiude la Fase 3: popola `RunProgress.unlocks` (campo introdotto vuoto dal task 035) con sblocchi reali che ampliano il pool di specie di partenza (task 039) e/o gli strumenti disponibili (es. azione extra, tag noto), persistenti tra run diverse **solo all'interno della stessa sessione di processo**.

---

## 📋 Acceptance Criteria

- [ ] `Unlocks` (in `src/run.rs`, oggi struct minimale/vuota dal task 035) popolata con almeno una categoria di sblocco reale: specie extra nel pool disponibile (039) e/o uno strumento aggiuntivo (es. un punto extra di `ActionBudget` per era, o un tag rivelato in anticipo).
- [ ] Gli sblocchi si accumulano in base a un criterio semplice legato al progresso della run conclusa (es. `worlds_cleared` raggiunti prima del `Defeat`) — non serve un sistema di progressione elaborato, solo una regola chiara e testabile.
- [ ] `Unlocks` **non è scritto su disco**: nessuna nuova dipendenza da API di filesystem (`std::fs`, crate di serializzazione verso file) introdotta in `run.rs`/`worldgen.rs`/`menu.rs` per questo scopo — verificabile a grep.
- [ ] Gli sblocchi restano disponibili se il giocatore avvia una nuova run **nella stessa sessione di processo** (dopo un `Defeat` che riporta al main menu, task 045) — `RunProgress` (o una resource dedicata agli sblocchi accumulati, sopravvissuta al reset di `RunProgress` per la nuova run) mantiene lo stato tra run diverse finché il processo resta in esecuzione.
- [ ] Un eventuale riepilogo sblocchi è mostrato al giocatore (es. nel main menu o nella schermata di sconfitta) — nuova sezione in `text.rs`.
- [ ] Nessuno sblocco fornisce informazione sulla matrice biochimica nascosta (coerente con GDD §10: "the matrix always remains to be deciphered from scratch... you unlock capabilities, not answers" — un "tag noto" sblocca la conoscenza che quel tag *esiste/è attivo*, non il suo valore nella matrice).
- [ ] `cargo clippy -- -D warnings` pulito, `cargo test` verde.
- [ ] Verifica manuale: avviare due run consecutive nella stessa sessione (`cargo run` una sola volta), concludere la prima con almeno uno sblocco maturato, confermare che è visibile/attivo nella seconda.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/run.rs` | `Unlocks` (popolata), resource che sopravvive al reset di `RunProgress` tra run diverse nella stessa sessione. |
| `src/worldgen.rs` | Consumo degli sblocchi nel pool di specie disponibili (039). |
| `src/menu.rs` | Eventuale schermata/riepilogo sblocchi. |
| `src/text.rs` | Nuova sezione stringhe sblocchi. |

---

## 🧩 Technical Context

**GDD §10, testo completo**:
> Progressione tra run, deliberatamente leggera: sbloccare più specie di partenza o strumenti (es. un'azione extra, o un tag noto). La matrice rimane sempre da decifrare da zero: non si sbloccano "risposte", si sbloccano capacità.
> Persistenza: l'MVP è costruito senza persistenza (tutto dentro una singola run). Se salvare gli sblocchi si decide solo dopo aver verificato che il loop sia divertente. Banale da aggiungere dopo, non vincola l'architettura.

Nota: "tutto dentro una singola run" nel GDD è la frase usata per dire "senza salvataggio su disco" — nella pianificazione di questo task si è interpretato che gli sblocchi vivano per la **sessione di processo** (sopravvivono a più run consecutive finché il gioco resta aperto), coerente con "trivial to add later" per la persistenza vera e propria: se dovessero *non* sopravvivere nemmeno tra run nella stessa sessione, la meta-progressione sarebbe priva di effetto osservabile per il giocatore, il che renderebbe il task privo di scopo. Se in fase di implementazione emerge un'interpretazione diversa più aderente al testo, verificarla col contesto di `PROJECT_PLAN.md`/GDD §14 prima di procedere.

- **Comportamento attuale**: `RunProgress.unlocks: Unlocks` esiste (task 035) come struct vuota/minimale, mai popolata né consumata.
- **Comportamento desiderato**: concludere run successive nella stessa sessione amplia gradualmente le opzioni disponibili al giocatore (specie/strumenti), senza mai rivelare la biochimica nascosta, senza toccare il disco.

---

## 🔨 Suggested Implementation

1. Definire `Unlocks` con campi concreti, es. `extra_species: Vec<SpeciesTemplate>`, `extra_action_points: u32`, `known_tags: Vec<TagId>` (adattare ai tipi esistenti).
2. Decidere dove vive lo stato che sopravvive al reset di `RunProgress`: probabilmente una resource separata (es. `MetaProgress`, inizializzata una sola volta all'avvio del processo, non resettata da `start_world`/dal main menu) che accumula gli sblocchi maturati da ogni run conclusa.
3. Al termine di una run (transizione a `Defeat`, task 045), calcolare gli sblocchi maturati da `worlds_cleared` e aggiungerli a `MetaProgress`.
4. In `worldgen.rs` (task 039), il generatore del pool di specie disponibili consulta `MetaProgress` per ampliare il pool.
5. Aggiungere un riepilogo minimo nella UI (main menu o schermata `Defeat`).
6. Verifica manuale con due run consecutive nello stesso processo.

---

## ⚠️ Constraints and Caveats

- **Nessuna persistenza su disco**: esplicitamente fuori scope per l'MVP (GDD §10) — non introdurre `serde`/file di salvataggio per questo task.
- **Non rivelare la matrice**: uno sblocco può dire "conosci l'esistenza del tag X" ma mai "il tag X vale +2 verso il tag Y" — quella è informazione che il giocatore deve dedurre col notebook (GDD §7/§11).
- **Non introdurre bonus objectives**: fuori scope MVP, come già notato nel task 042.
- **Mantenere la semplicità**: un criterio di sblocco chiaro e testabile (es. lineare su `worlds_cleared`) è preferibile a un sistema di progressione elaborato — GDD §10 chiede esplicitamente "deliberately light".

---

## 🔗 Dependencies

- **Depends on**: 039 (pool di specie disponibili da ampliare), 045 (transizione di run/conclusione da cui calcolare gli sblocchi).
- **Blocks**: nessuno — ultimo task della Fase 3.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/046-minimal-meta-progression.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
