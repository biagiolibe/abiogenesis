# Task 006 — Resa della griglia a sprite + camera 2D

> **ID**: `006`
> **Categoria**: Feature
> **Priorità**: 🟡 P2
> **Stima**: ~2h
> **Assegnato a**: non assegnato
> **Sessione**: —

---

## 🎯 Obiettivo

Rendere **visibile** la griglia: uno sprite quadrato per cella, colorato in base allo stato della simulazione, con una camera 2D che inquadra il tutto.

Senza questo task la simulazione gira alla cieca. Il traguardo della Fase 0 (GDD §13) è letteralmente *"guardi una specie fotolitica fiorire e stabilizzarsi"*: qui si costruisce il "guardi".

---

## 📋 Acceptance Criteria

- [ ] `cargo run` mostra una griglia `48×32` di quadrati colorati.
- [ ] Gli sprite sono spawnati **una volta sola** in `Startup`; i tick successivi aggiornano solo il colore.
- [ ] **Celle occupate**: colore = specie, luminosità = energia.
- [ ] **Celle vuote**: sfondo tenue proporzionale a `light`, così i gradienti ambientali sono visibili a occhio.
- [ ] I **residui** sono visivamente distinguibili sia dalle celle vuote sia da quelle occupate.
- [ ] La griglia resta centrata e interamente visibile ridimensionando la finestra.
- [ ] **Nessun sistema di resa scrive su `SimWorld`** (solo `Res`, mai `ResMut`).
- [ ] `cargo clippy -- -D warnings` pulito.

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `src/render.rs` | `GridRenderPlugin`, spawn degli sprite, camera, sincronizzazione |
| `src/world.rs` | `SimWorld` in sola lettura (task 003) |

---

## 🧩 Contesto Tecnico

- **Comportamento attuale**: la finestra si apre vuota; il mondo esiste in memoria ma non si vede.
- **Comportamento desiderato**: la griglia è visibile e riflette lo stato della simulazione.

### GDD §11 — Presentazione

> - **Resa:** finestra 2D. Griglia di celle come quadrati colorati.
>   - Celle occupate: colore = specie/tag; luminosità = energia.
>   - Celle vuote: sfondo tenue che riflette l'ambiente (es. luminosità = `light`).

### Architettura: entità solo per la resa

`TECH_DESIGN.md` §3.1 stabilisce che la griglia **non** è modellata in ECS: lo stato vive in `SimWorld`. Le entità sprite create qui sono una **vista**, non la fonte di verità. Portano un componente `GridCell { x, y }` che le lega alla cella corrispondente.

1536 sprite (48×32) sono un carico banale per Bevy.

---

## 🔨 Implementazione Suggerita

1. **Componente di legame**

   ```rust
   /// Links a rendered sprite back to its cell in SimWorld. The sprite is a view:
   /// simulation state lives in the resource, never here (TECH_DESIGN 3.1).
   #[derive(Component)]
   struct GridCell {
       x: usize,
       y: usize,
   }
   ```

2. **Spawn in `Startup`** — uno sprite per cella, `custom_size` pari al lato della cella, posizionati su una griglia centrata sull'origine. Ricordare che in Bevy la `y` cresce verso l'alto mentre l'indice di riga cresce verso il basso: la riga `0` (luce alta) deve apparire **in cima**, quindi la `y` del mondo va invertita.

3. **Camera** — `Camera2d`. Perché la griglia entri sempre nella finestra, usare una proiezione a dimensione fissa (`ScalingMode::AutoMin` o equivalente nella 0.19) dimensionata su `width * cell_size` × `height * cell_size`. Così il ridimensionamento non taglia la griglia.

4. **Sistema di sincronizzazione** — gira in `SimSet::Sync`, dopo `SimSet::Advance`:

   ```rust
   fn sync_grid_colors(
       world: Res<SimWorld>,              // read-only: never ResMut here
       mut cells: Query<(&GridCell, &mut Sprite)>,
   ) {
       for (cell, mut sprite) in &mut cells {
           sprite.color = cell_color(&world, cell.x, cell.y);
       }
   }
   ```

5. **Scelta dei colori** — regola unica in un solo posto:

   | Stato della cella | Colore |
   |---|---|
   | Occupata | tinta della specie, luminosità scalata sull'energia |
   | Con residuo, vuota | tinta neutra desaturata, intensità proporzionale al residuo |
   | Vuota | grigio molto scuro, luminosità proporzionale a `light` |

   Per l'energia, normalizzare su `repro_threshold` (`10.0`) e **clampare**: l'energia può superarla nel tick prima della riproduzione. Lavorare in HSL/HSV rende naturale "stessa tinta, luminosità variabile".

   Tenere basso il fondo delle celle vuote: deve suggerire il gradiente di luce senza competere con gli organismi.

6. **Test manuale**: avviare, seminare a mano un organismo in `SimWorld` (o attendere il task 007) e verificare che si veda. Il gradiente di luce deve essere percepibile come sfumatura verticale sulla griglia vuota.

---

## ⚠️ Vincoli e Attenzioni

- **Il rendering è in sola lettura.** Se un sistema di `render.rs` chiede `ResMut<SimWorld>`, l'architettura è stata violata.
- **Non spawnare/despawnare sprite a ogni tick**: si crea una volta, si aggiorna il colore. Il despawn per cella vuota costerebbe più della resa stessa.
- **Nessun asset artistico** (GDD, pilastro 3): sprite bianchi colorati via `Sprite::color`, nessuna texture da caricare.
- La resa dei **tag come glifi** (GDD §11) è Fase 1+: qui il colore identifica la specie e basta.
- Verificare l'API di Bevy 0.19 per `Sprite`/`Camera2d`: la 0.19 è recente e i nomi differiscono dalle 0.1x precedenti che compaiono nella maggior parte degli esempi online.

---

## 🔗 Dipendenze

- **Dipende da**: 003
- **Blocca**: 007

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/006-grid-rendering.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
