# Task 036 — `TagSlot` newtype: compiler-driven matrix indexing

> **ID**: `036`
> **Category**: Refactor / Architecture
> **Priority**: 🔴 P1
> **Estimate**: ~3-4h (più corposo dello standard ~2h — confermato con l'utente in fase di pianificazione: non spezzabile senza lasciare la build rotta a metà)
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

`TagMatrix` e `MatrixKnowledge` sono oggi indicizzate direttamente per `TagId.0 as usize`, assumendo che i tag attivi di un mondo siano il range contiguo `TagId(0..n)` — comportamento del task 010, mai messo in discussione finora perché finora lo è sempre stato. Un commento esplicito in `world.rs:34-38` segnala il rischio: *"If Phase 3 world generation ever picks a non-contiguous subset of the global pool, this indexing needs to go through a tag→matrix-index lookup instead."*

Perché il worldgen (GDD §9) possa scegliere un sottoinsieme realmente vario dal pool globale di 10 tag (non solo i primi N), serve rompere questa assunzione. La soluzione scelta è introdurre `TagSlot(u8)` — la posizione di un tag nel sottoinsieme attivo del *mondo corrente* — come tipo distinto da `TagId(u8)` — l'identità del tag nel pool *globale*. `TagMatrix` e `MatrixKnowledge` vengono indicizzate per `TagSlot`; `SimWorld.active_tags: Vec<TagId>` resta l'unica mappa slot→identità globale, consultata solo da `text.rs`/`ui.rs` per risolvere nomi/colori/glifi.

Questo è un refactor puro: nessun comportamento di gioco cambia. Il worldgen che sfrutta effettivamente `TagSlot` per selezionare sottoinsiemi non contigui arriva nel task 038.

---

## 📋 Acceptance Criteria

- [x] Nuovo tipo `TagSlot(pub u8)` in `src/world.rs`, accanto a `TagId`, con derive coerenti (`Debug, Clone, Copy, PartialEq, Eq`).
- [x] `TagMatrix::get` prende `TagSlot` invece di `TagId`: `fn get(&self, exerter: TagSlot, receiver: TagSlot) -> i8`.
- [x] `Species.tags: Vec<TagSlot>` (era `Vec<TagId>`).
- [x] `MatrixKnowledge` in `src/notebook.rs` (record/evidence/is_confirmed/revealed_value e l'indicizzazione interna) migrata alle stesse chiavi `TagSlot`.
- [x] `SimWorld.active_tags: Vec<TagId>` resta come unica mappa slot→identità globale (l'indice nel `Vec` *è* lo slot); nessun altro punto del codice mantiene una mappatura parallela.
- [x] Tutti i siti che oggi convertono un tag in indice di matrice/evidenza con `tag.0 as usize` sono stati aggiornati per usare `TagSlot` direttamente — nessuno rimasto (verificato a grep: le uniche occorrenze di `TagId` fuori da `world.rs` sono `tag_color`/`tag_glyph`/`node_tooltip_text` in `notebook.rs`, identità globale per la UI, e `active_tags` di test in `input.rs`).
- [x] `text.rs`/`ui.rs`: i punti che devono mostrare nome/colore/glifo di un tag risolvono `TagSlot → TagId` tramite `world.active_tags[slot.0 as usize]` prima di chiamare le funzioni di `text.rs`/`tag_glyph`/`tag_color`.
- [x] `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` tutti verdi a fine task.
- [x] Nessuna regressione: 68 test passano, invariati nei valori attesi (solo i tipi delle fixture sono cambiati da `TagId` a `TagSlot` dove operano su matrice/evidenza).

## Riepilogo implementazione

- `src/world.rs`: `TagSlot(pub u8)` introdotto accanto a `TagId`; `TagMatrix::get` e `Species.tags` migrati a `TagSlot`; `generate_matrix` non richiede più `active_tags: &[TagId]`, solo `slot_count: usize` (la selezione del ciclo negativo campiona slot `0..n`, non identità); `draw_species_tags` ritorna `Vec<TagSlot>`.
- `src/sim.rs`: `AdjacencyObserved::{exerter_tag, receiver_tag}` e `neighbour_tags` migrati a `TagSlot` (l'intero file operava già in "slot space" tramite `species.tags`, quindi il cambio è stata una sostituzione diretta `TagId` → `TagSlot`).
- `src/notebook.rs`: `MatrixKnowledge` migrata a chiavi `TagSlot`; `hypothesis_grid`/`node_tooltip_text` ora enumerano `world.active_tags` per ottenere coppie `(TagSlot, TagId)` — slot per le query a `MatrixKnowledge`, `TagId` per glyph/colore; `catalog_panel` risolve `TagSlot → TagId` via `world.active_tags[slot.0 as usize]` prima di colorare/glifare.
- `src/ui.rs`: `SpliceEditChoice::{SwapTag, AddTag}` migrati a `TagSlot`; `splice_panel` enumera `world.active_tags` per costruire slot selezionabili, risolvendo l'identità solo per l'etichetta visuale.
- `src/input.rs`: `apply_splice` invariata nel corpo (operava già su `species.tags` senza nominare il tipo); solo i test aggiornati a `TagSlot` dove costruiscono tag di specie/`SpliceEditChoice`, mantenendo `TagId` per `world.active_tags`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | Definizione `TagSlot`, `TagMatrix::get`, `Species.tags`, `SimWorld.active_tags` come unica sorgente slot→identità. |
| `src/sim.rs` | `AdjacencyObserved` e ogni chiamata a `matrix.get(...)` nella tick logic; test hand-built che costruiscono matrici/specie. |
| `src/notebook.rs` | `MatrixKnowledge` (record/evidence/is_confirmed/revealed_value) e la griglia/grafo di rendering che iterano `world.active_tags`. |
| `src/ui.rs` | Ogni sito che itera i tag per nome/colore/glifo — deve risolvere `TagSlot → TagId` prima di chiamare `text.rs`. |
| `src/input.rs` | Chiamata di reset `MatrixKnowledge::new` (tasto `r`) — aggiornare alla nuova firma se cambia. |

---

## 🧩 Technical Context

**`TagMatrix` attuale** (`src/world.rs`, righe ~30-51):
```rust
pub struct TagId(pub u8);

pub struct TagMatrix {
    pub(crate) size: usize,
    pub(crate) values: Vec<i8>,
}

impl TagMatrix {
    pub fn get(&self, exerter: TagId, receiver: TagId) -> i8 {
        self.values[exerter.0 as usize * self.size + receiver.0 as usize]
    }
}
```
Il commento alle righe 34-38 documenta esattamente il rischio che questo task risolve.

`MatrixKnowledge` in `notebook.rs` duplica lo stesso pattern di indicizzazione (`TagId.0 as usize * size + ...`) per tracciare evidenza pesata (`1/(1+n_confounders)`, GDD §7) — va migrata in sincrono, non in un task separato, perché cambiare la firma di `TagMatrix::get` rompe la build di `notebook.rs` nello stesso momento.

- **Comportamento attuale**: `TagId` funge sia da identità globale nel pool sia da indice diretto in matrice/evidenza, perché finora i tag attivi sono sempre stati `TagId(0..active_tags_early)` — coincidenza che il worldgen (task 038) romperà.
- **Comportamento desiderato**: `TagId` = identità nel pool globale di 10 tag (usata solo per nome/colore/glifo tramite `SimWorld.active_tags`); `TagSlot` = posizione nel sottoinsieme attivo del mondo corrente (usata per ogni indicizzazione di matrice/evidenza). La conversione slot→identità passa sempre da `SimWorld.active_tags[slot.0 as usize]`, mai da una mappa duplicata.

---

## 🔨 Suggested Implementation

1. Aggiungere `TagSlot(pub u8)` in `src/world.rs`, vicino a `TagId`.
2. Cambiare `TagMatrix::get` per accettare `TagSlot`. Lasciare `cargo build` fallire e usare gli errori del compilatore come checklist per propagare il cambio a `Species.tags`, `sim.rs`, `notebook.rs`, `ui.rs` — è il modo più affidabile per non perdere un call site (da qui la scelta di farne un task unico).
3. In `sim.rs`, verificare `AdjacencyObserved` e ogni punto della tick logic che chiama `matrix.get`: deve ricevere `TagSlot`, non `TagId`. Se `AdjacencyObserved` (evento consumato dal notebook) porta oggi `TagId`, valutare se convertirlo a `TagSlot` a monte (nella tick logic, dove si ha già accesso a `SimWorld`) o lasciarlo `TagId` e convertire solo al consumo — preferire la prima opzione se semplifica `notebook.rs`, dato che `MatrixKnowledge` lavora già in termini di slot.
4. In `notebook.rs`, aggiornare `MatrixKnowledge` (struct interna, `record`, `evidence`, `is_confirmed`, `revealed_value`) alla nuova chiave `TagSlot`. Aggiornare le funzioni di rendering griglia/grafo che oggi iterano `world.active_tags` per costruire nodi/etichette: continueranno a iterare `world.active_tags` (che resta `Vec<TagId>`), ma l'indice di iterazione *è* il `TagSlot` da passare a `MatrixKnowledge`.
5. In `ui.rs`, per ogni sito che mostra nome/colore/glifo di un tag: assicurarsi che risolva `TagSlot → TagId` (`world.active_tags[slot.0 as usize]`) prima di chiamare le funzioni di `text.rs` — `text.rs` stesso non cambia (non conosce `SimWorld`, task 034).
6. In `input.rs`, aggiornare la costruzione di `MatrixKnowledge::new` nel reset del tasto `r` se la firma cambia.
7. Aggiornare i test hand-built in `world.rs`/`sim.rs`/`notebook.rs` che costruiscono `TagMatrix`/`Species` a mano: cambiano solo i tipi usati per costruire i dati di test, non la logica verificata.
8. `cargo build && cargo clippy -- -D warnings && cargo test` — iterare finché tutto è verde.

---

## ⚠️ Constraints and Caveats

- **Non introdurre un layer di lookup separato** (es. una `HashMap<TagId, TagSlot>`): l'indice nel `Vec<TagId>` di `SimWorld.active_tags` è già la mappa slot→identità; una mappa parallela sarebbe uno stato duplicato da tenere sincronizzato, contro il principio "niente astrazioni premature".
- **Determinismo (invariante 1)**: nessuna `HashMap`/`HashSet` iterata nella tick logic — se serve una struttura di lookup, usare `Vec` indicizzato.
- **Non toccare la logica di selezione dei tag attivi**: questo task non seleziona ancora sottoinsiemi non contigui (arriva nel task 038) — qui `active_tags` resta popolato esattamente come oggi (`0..active_tags_early`), cambia solo *come* viene indicizzata la matrice.
- **Nessun comportamento di gioco deve cambiare**: è un refactor a comportamento invariato, verificabile con i test di determinismo/bilanciamento esistenti (task 009) che devono continuare a passare senza modifiche ai valori attesi.

---

## 🔗 Dependencies

- **Depends on**: nessuno.
- **Blocks**: 038 (worldgen non può selezionare sottoinsiemi non contigui finché `TagSlot` non esiste).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/036-tag-slot-newtype.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
