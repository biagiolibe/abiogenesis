# Task 008 — HUD `bevy_egui`

> **ID**: `008`
> **Categoria**: UI
> **Priorità**: 🟡 P2
> **Stima**: ~1.5h
> **Assegnato a**: non assegnato
> **Sessione**: —

---

## 🎯 Obiettivo

Aggiungere un pannello che mostri lo stato della simulazione in numeri: tick, era, popolazione, energia media, seed, comandi disponibili.

Serve a due cose. Al giocatore, per **leggere** l'ecosistema oltre il colpo d'occhio della griglia — il GDD §14 indica la leggibilità come secondo rischio del progetto. A chi sviluppa, per vedere subito se il tuning sta andando dove deve.

---

## 📋 Acceptance Criteria

- [ ] `EguiPlugin` è registrato e un pannello è visibile all'avvio.
- [ ] Il pannello mostra: **tick**, **era**, **popolazione per specie**, **energia media**, **seed**, **stato corrente** (`EraState`).
- [ ] Mostra gli **hint dei comandi** (`space`, `s`, `r`, `Esc`).
- [ ] I valori coincidono con `SimWorld` e si aggiornano durante l'animazione dell'era.
- [ ] Il pannello **non copre la griglia**: la camera va adattata o il pannello posizionato di conseguenza.
- [ ] Gli input di gioco continuano a funzionare mentre il puntatore è sul pannello.
- [ ] **Nessuna scrittura su `SimWorld`** dai sistemi UI.
- [ ] `cargo clippy -- -D warnings` pulito.

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `src/ui.rs` | `UiPlugin`, pannello HUD |
| `src/main.rs` | Registrazione di `EguiPlugin` |

---

## 🧩 Contesto Tecnico

- **Comportamento attuale**: la griglia si vede e le ere avanzano, ma non c'è alcun dato numerico a schermo. `bevy_egui` è una dipendenza dichiarata (task 001) ma non ancora registrata.
- **Comportamento desiderato**: HUD sempre visibile.

### GDD §11

> **Pannelli UI:** tick corrente, numero d'era, popolazioni per specie, energia media, obiettivo corrente, budget azioni, hint dei comandi.

Obiettivo e budget azioni appartengono alla Fase 3: qui si predispone lo spazio, senza inventarne i valori.

### Perché egui

`TECH_DESIGN.md` §1 sceglie `bevy_egui` per la UI. La ragione vera arriva in Fase 2: il taccuino (GDD §7) è una **griglia di ipotesi `tag × tag` densa e interattiva**, il caso d'uso in cui una UI immediata è nettamente più adatta di una a widget persistenti. L'HUD è il primo banco di prova di quella scelta.

---

## 🔨 Implementazione Suggerita

1. **Registrare il plugin** in `main.rs`, prima di `UiPlugin`. Verificare la firma richiesta da `bevy_egui` 0.41 (nelle versioni recenti `EguiPlugin` ha campi di configurazione: `EguiPlugin::default()` è il punto di partenza).

2. **Pannello laterale**, così da non sovrapporsi alla griglia:

   ```rust
   fn hud_panel(
       mut contexts: EguiContexts,
       world: Res<SimWorld>,               // read-only
       state: Res<State<EraState>>,
   ) {
       egui::SidePanel::right("hud").show(contexts.ctx_mut(), |ui| {
           ui.heading("Abiogenesis");
           ui.label(format!("Era {}  ·  tick {}", world.era, world.tick));
           // ...
       });
   }
   ```

3. **Statistiche** — calcolate leggendo la griglia. A 1536 celle il costo è trascurabile a ogni frame; se in futuro pesasse, si sposta il calcolo in `SimSet::Advance` e si memorizza in una resource.

   ```rust
   /// Population and mean energy per species, computed from the grid.
   fn species_stats(world: &SimWorld) -> Vec<(SpeciesId, usize, f32)>
   ```

   L'**energia media** va calcolata solo sugli organismi vivi (denominatore = popolazione, non numero di celle) — altrimenti sembra sempre in caduta libera.

4. **Spazio riservato alle fasi future.** Sezioni presenti ma inattive, così l'evoluzione della UI non richiede di riprogettare il pannello:

   ```rust
   // Placeholder: objective and action budget arrive in Phase 3 (GDD 8, 6).
   ```

5. **Hint dei comandi** in fondo, in testo tenue: `space` era · `s` tick · `r` reseed · `Esc` esci.

6. **Convivenza degli input.** Egui cattura tastiera e mouse quando un suo widget ha il focus. Con soli `label` non succede, ma quando in Fase 2 arriveranno campi di testo servirà interrogare `ctx.wants_keyboard_input()` prima di leggere gli input di gioco. Se questo task introduce un widget interattivo, gestirlo subito.

---

## ⚠️ Vincoli e Attenzioni

- **La UI legge e basta** (`TECH_DESIGN.md` §3.3): `Res<SimWorld>`, mai `ResMut`.
- **Non duplicare lo stato in resource UI.** L'HUD calcola dalla griglia a ogni frame; una copia sarebbe una seconda fonte di verità da tenere sincronizzata.
- **Non inventare l'obiettivo del mondo**: è la Fase 3 (GDD §8). Qui solo un segnaposto.
- Attenzione all'accoppiamento di versioni: `bevy_egui` 0.41 ↔ bevy 0.19 ↔ egui 0.35. L'API di `EguiContexts` è cambiata più volte tra versioni vicine — usare la documentazione della 0.41, non i primi esempi che si trovano.
- Il pannello va tenuto **compatto**: la griglia è il protagonista, l'HUD è di supporto.

---

## 🔗 Dipendenze

- **Dipende da**: 007
- **Blocca**: nessuno

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/008-hud-egui.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
