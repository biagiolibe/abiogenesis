# Piano — Icone vettoriali custom per HUD/Notebook (metabolismi + Moves + Notebook)

> **Come riprendere**: questo file è un piano completo e autosufficiente,
> pronto per essere trasformato in due task Meridian (`tasks/132-*.md` e
> `tasks/133-*.md`) in una sessione futura, anche su un'altra macchina.
> Non richiede di rileggere la conversazione che l'ha prodotto — tutte le
> decisioni, i numeri, e i riferimenti a file:linea sono qui dentro. Le
> uniche letture da fare all'atto dell'implementazione: i tre file SVG
> citati (`redesign/hud-full.svg`, `redesign/notebook-full.svg`,
> `redesign/species-icons-color.svg`) e i file sorgente Rust elencati nelle
> sezioni "Call site". Se nel frattempo il codice è cambiato in modo
> sostanziale rispetto a quanto descritto qui (numeri di riga, nomi di
> funzione), ri-verificare con `grep`/`Read` prima di fidarsi ciecamente dei
> riferimenti sotto — sono accurati al momento della stesura
> (2026-08-13), non garantiti per sempre.

## Contesto

`redesign/abiogenesis-hud-notebook.md` propone un linguaggio visivo a icone
vettoriali disegnate a mano (line-art, coerente con la pillar "la grafica è
minimale, il divertimento è nel sistema") per: le 4 icone metabolismo
(HUD Species/Biosphere + Notebook catalog/legend), le 4 icone Moves
(Seed/Stress/Cull/Splice), e l'icona del pulsante Notebook. Una prima
sessione di scoping (2026-08-12, vedi `PROJECT_PLAN.md` "HUD & Notebook
redesign follow-up") aveva scelto di **non** portare 1:1 il disegno del
mockup, tenendo i glifi Unicode/emoji esistenti (☀⚔♻☣ per i metabolismi,
🌱⚡💀🔬 per le Moves) e scoping solo `task 119` (glifi dingbat monocolore
alternativi, per risolvere un bug reale — 3 delle 4 icone Moves quasi
certamente renderizzano come tofu box, egui non supporta emoji a colori).

Questa sessione (2026-08-13) **riapre quella decisione**: l'utente vuole le
primitive vettoriali del mockup disegnate letteralmente con `egui::Painter`
(linee, cerchi, poligoni, curve di Bézier), lo stesso linguaggio già usato
per il grafo delle relazioni nel notebook (`notebook.rs::draw_edge`/
`draw_dashed_line`), non un secondo giro di ricerca di glifi Unicode.
L'utente ha fornito anche un **secondo file di riferimento**,
`redesign/species-icons-color.svg`, che è la fonte canonica per tutti e 4 i
metabolismi reali (Fotolitico, Predatore, Decompositore, Chemiolitotrofo —
quest'ultimi due non erano nel mockup originale) più 3 proposte future
(Parassita, Simbionte, Estremofilo) esplicitamente fuori scope oggi: non
esiste ancora una variante `Metabolism` per nessuna delle tre.

Quattro decisioni già prese con l'utente in questa sessione (non da
rimettere in discussione senza un motivo nuovo):

1. **Badge livello mutazione**: scartato. Nessuna meccanica di capacità a
   livelli esiste oggi (`Splice` offre sempre tutte e tre le modifiche);
   esiste solo uno sconto di costo (`RunProgress::splice_cost`,
   `src/run.rs:65-76`), un concetto diverso. Non disegnare il badge.
2. **Riga Biosphere**: inclusa nello stesso lavoro — oggi mostra un
   pallino generico (`SPECIES_GLYPH = "●"`, `src/ui.rs:154`), non l'icona
   di metabolismo; va allineata come nel mockup.
3. **Task 119**: superato, non emendato — le sue acceptance criteria
   riguardano "quale code point Unicode renderizza", nulla sopravvive
   all'inversione verso un approccio vettoriale disegnato. Va marcato
   `[-]` (cancellato) in `tasks/QUEUE.md` con puntatore al nuovo task 133,
   non implementato.
4. **Tutti e 4 i metabolismi reali** ottengono l'icona vettoriale (non
   solo Fotolitico/Predatore) — l'utente ha fornito
   `redesign/species-icons-color.svg` apposta per coprire anche
   Decompositore e Chemiolitotrofo, eliminando il mix di pesi ottici che
   sarebbe rimasto altrimenti nella riga Biosphere.

## Design tecnico

### Dove vive il codice

Nuovo file `src/icons.rs` (crate binario, accanto a `ui.rs`/`notebook.rs`/
`render.rs`/`text.rs` — non fa parte della libreria `abiogenesis` che
`sim.rs`/`world.rs`/`config.rs` popolano, per lo stesso motivo per cui
`render.rs` oggi dipende da `bevy::render`/`bevy_egui`). Non diventa un
`Plugin` Bevy — è un modulo di puro disegno, senza stato, stesso status di
`text.rs`. `render.rs::metabolism_glyph` **resta invariata** (serve ancora
come riferimento/fallback testuale se in futuro arriva un quinto
metabolismo prima della sua icona).

### API (`src/icons.rs`)

```rust
pub enum Icon {
    Photolithic, Predator, Decomposer, Chemolithotroph,
    Seed, Stress, Cull, Splice, Notebook,
}

/// Disegna `kind` dentro `rect` (già posizionato/dimensionato dal chiamante).
pub fn paint(painter: &egui::Painter, kind: Icon, rect: egui::Rect, color: egui::Color32, stroke_width: f32);

/// Comodo per icone inline-con-testo: alloca un rect quadrato `size`x`size`
/// (stesso pattern di `ui.rs::dot_row`, `ui.allocate_exact_size` +
/// `ui.painter()`) e ci disegna dentro.
pub fn inline(ui: &mut egui::Ui, kind: Icon, size: f32, color: egui::Color32) -> egui::Response;

/// Converte `Metabolism` in `Icon` — l'unico punto che conosce la
/// corrispondenza 1:1 (oggi tutti e 4 i metabolismi hanno un'icona
/// vettoriale; se in futuro un quinto metabolismo arriva prima della sua
/// icona, questo è il punto in cui aggiungere un fallback testuale, es.
/// tramite `render::metabolism_glyph`).
pub fn metabolism_inline(ui: &mut egui::Ui, m: Metabolism, size: f32, color: egui::Color32) -> egui::Response;
```

- Coordinate normalizzate per icona in uno spazio locale centrato
  sull'origine (stessa scala delle sorgenti SVG, vedi sotto), scalate al
  `rect` effettivo al momento del disegno — porting diretto, nessun flip
  d'asse (SVG e `egui` sono entrambi y-down).
- `stroke_width` passato esplicitamente dal chiamante, non derivato dalla
  scala (il mockup usa ~1.4pt su icone da 8-16px e ~1.6pt su icone da
  22-38px — una scala lineare renderebbe l'icona piccola sfocata o quella
  grande troppo sottile). Due costanti modulo, `ICON_STROKE_INLINE` /
  `ICON_STROKE_BUTTON`.
- Colore sempre passato dal chiamante (mai fisso per metabolismo) — nota
  esplicita in `species-icons-color.svg` riga 4: *"nel gioco ogni icona
  prende il colore della specie, non un colore fisso per metabolismo"*.
  Per Seed/Stress/Cull/Splice/Notebook il colore segue lo stato del
  bottone (vedi sotto).

### Verificato: API `epaint` 0.35.0 (pinnata via `bevy_egui`)

Confermato leggendo il sorgente in
`~/.cargo/registry/src/index.crates.io-*/epaint-0.35.0/src/shapes/`:
`egui::Shape::line(Vec<Pos2>, stroke)` (polilinea aperta),
`Shape::closed_line(...)` (poligono chiuso, solo contorno),
`egui::epaint::CubicBezierShape::from_points_stroke([Pos2;4], closed, fill,
stroke)` con `.flatten(Option<f32>) -> Vec<Pos2>` — stessa forma di
`QuadraticBezierShape` già usata in `notebook.rs::draw_edge`. Nessuna
primitiva d'arco nativa: l'arco del seme (vedi sotto) va campionato a mano
in punti e unito alla polilinea complessiva.

### Traduzione primitiva-per-primitiva

**Metabolismi — fonte canonica `redesign/species-icons-color.svg`**
(sostituisce le proporzioni leggermente diverse di `hud-full.svg`, che
restano lo sketch originale ma non l'origine dei numeri finali):

- **Fotolitico** (righe 8-13): 4 segmenti dal centro — orizzontale
  `(-6,0)-(6,0)`, verticale `(0,-6)-(0,6)`, diagonale 1
  `(-4.2,-4.2)-(4.2,4.2)`, diagonale 2 `(-4.2,4.2)-(4.2,-4.2)`. 4
  `line_segment`, nessun riempimento.
- **Predatore** (righe 20-24): 3 cerchi pieni, raggio 4 —
  `(0,-3.5)`, `(-3,2)`, `(3,2)`. 3 `circle_filled`.
- **Decompositore** (riga 31): rombo contornato, 4 punti —
  `(0,-8),(8,0),(0,8),(-8,0)` — `Shape::closed_line`, solo stroke.
- **Chemiolitotrofo** (riga 38): esagono contornato, 6 punti —
  `(8,0),(4,6.9),(-4,6.9),(-8,0),(-4,-6.9),(4,-6.9)` —
  `Shape::closed_line`, solo stroke.
- **Fuori scope oggi** (righe 46-72 dello stesso file): Parassita,
  Simbionte, Estremofilo — nessuna variante `Metabolism` esiste per loro;
  disegno di riferimento tenuto nel file per quando (se) verranno
  implementati come meccanica, non tradotto in codice ora.

**Moves + Notebook — fonte `redesign/hud-full.svg`** (coordinate
ricalcolate relative all'origine di ciascun bottone 38×38, così l'icona è
indipendente dalla posizione assoluta nel mockup):

- **Seed** (bottone origine `(20,160)`): contorno chiuso —
  stelo-taper `(19,10)→(13,16),(10,23)→(10,29)` (cubica), poi un
  semicerchio campionato (centro `(19,29)`, r=9, dal punto `(10,29)`
  all'angolo `(28,29)` passando per il basso `(19,38)` — il bulbo
  arrotondato del seme), poi stelo-taper di ritorno
  `(28,29)→(28,23),(25,16)→(19,10)` (cubica, speculare alla prima). Le tre
  parti (cubica, arco campionato, cubica) vanno emesse come **un'unica**
  polilinea chiusa (`Shape::closed_line` dopo aver appiattito le due
  cubiche con `.flatten()` e concatenato i punti dell'arco), non tre shape
  separate — altrimenti si vedono le giunture.
- **Stress** (bottone origine `(66,160)`): contorno **non convesso** a
  zigzag (fulmine) — punti `(24,7),(12,23),(20,23),(16,32),(30,15),(22,15)`
  — `Shape::closed_line`, solo stroke, **non** `convex_polygon` (il
  fulmine si autointerseca in silhouette).
- **Cull** (bottone origine `(112,160)`): cerchio contornato (teschio,
  centro `(19,15)` r7, `circle_stroke`) + 2 puntini pieni (occhi, `(16,14)`
  e `(22,14)`, r1.2, `circle_filled`) + una curva a mandibola —
  cubica 1 `(12,29)→(12,25),(15,23)→(19,23)`, cubica 2 (riflessione della
  `s` SVG, **non** il valore letterale) `(19,23)→(23,23),(26,25)→(26,29)`.
  Attenzione: leggere l'operando `s` come riflessione del control-point
  precedente attorno al punto condiviso, non come punto di controllo
  letterale — è il bug classico che produce una giuntura visibile.
- **Splice/Mutate** (bottone origine `(158,160)`): **solo linee rette**,
  nessuna curva — polilinea aperta `(13,7),(13,15),(7,29),(23,29),(17,15),
  (17,7)` (`Shape::line`) più una linea separata per il collo/tappo
  `(9,7)-(23,7)`. **Nessun badge** di livello mutazione (vedi decisione 1
  sopra).
- **Notebook** (icona dentro il bottone/etichetta esistente in
  `time_control_row`, origine locale `(46,516)`): rettangolo contornato
  `(0,0)-(12,16)` (`rect_stroke`, "copertina") + 2 linee interne
  `(3,5)-(9,5)` e `(3,10)-(9,10)` (righe di testo).

### Punti da gestire esplicitamente (non presenti nell'SVG, emersi
dall'analisi tecnica)

- **Tinta selezionato/disabilitato**: `action_icon_row` disabilita
  Stress/Cull fuori da Detail (`add_enabled_ui`) e usa `selectable_label`
  per lo stato selezionato — un'icona disegnata a mano non eredita
  automaticamente questi stati. Il colore va risolto esplicitamente al
  call site da `ui.visuals()` (colore debole/disabilitato) e dallo stato
  di selezione (mockup: Seed selezionato `#8FD666`, non selezionato
  `#888780`) — criterio di accettazione esplicito nel task.
- **Budget di altezza riga**: `BIOSPHERE_VISIBLE_ROWS`/`SPECIES_VISIBLE_ROWS`
  limitano lo scroll a `VISIBLE_ROWS * console_row_height(ui)`, e
  `console_row_height` misura lo *stile testo*, non il contenuto reale —
  un'icona inline più alta dello stile testo desincronizzerebbe quel
  limite silenziosamente (stessa classe di problema già documentata per lo
  scroll-floor 64.0 in `ui.rs:164-175`). Vincolo: icona inline ≤ altezza
  dello stile testo `Body`.
- **Perdita del target di click**: in `species_row` (`ui.rs:908`) oggi il
  glifo è *dentro* il testo del `selectable_label`, quindi parte del target
  cliccabile. Separare in `ui.horizontal` + icona disegnata a parte
  restringe leggermente il target di click alla sola porzione testo — da
  accettare esplicitamente, non costruire un widget custom cliccabile solo
  per questo (sproporzionato).

## Task da creare

Due task Meridian, prossimi ID liberi al momento della stesura: **132** e
**133** (verificare che siano ancora liberi al momento dell'implementazione
— `ls tasks/*.md tasks/done/*.md | grep -oE '[0-9]+' | sort -n | tail -5`).
Sequenziali: 133 dipende da 132 (stabilisce il modulo/convenzione che 133
riusa).

### Task 132 — Icone vettoriali per i 4 metabolismi

- Nuovo `src/icons.rs`: modulo, enum `Icon` (solo le 4 varianti
  metabolismo per questo task), `paint`/`inline`/`metabolism_inline`.
- Implementa Photolithic/Predator/Decomposer/Chemolithotroph con le
  coordinate sopra.
- Call site: `src/ui.rs:908` (`species_row`, chip Species HUD),
  `src/ui.rs:526` (riga Biosphere, sostituisce `SPECIES_GLYPH`),
  `src/notebook.rs:1198` (legenda metabolismo una tantum) e `:1215`
  (riga card Species catalog).
- Criteri: tinta = colore specie (mai fisso), icona ≤ altezza stile testo
  `Body`, nessuna regressione ai test esistenti su queste righe,
  `cargo clippy -- -D warnings`/`fmt`/`test` puliti, verifica live (icone
  leggibili nei 4 punti, coerenti tra loro).
- Dipende da: nessuno. Blocca: 133 (riusa modulo/convenzione).

### Task 133 — Icone vettoriali Moves + Notebook (supersede task 119)

- Estende `Icon` con `Seed`/`Stress`/`Cull`/`Splice`/`Notebook`.
- Implementa le 5 icone con le coordinate sopra (incluso il caso
  arco-campionato per Seed e la riflessione `s` per Cull).
- Call site: `src/ui.rs:990-1031` (`action_icon_row`, dipinge dentro
  `response.rect` dopo il `selectable_label`), `src/ui.rs:927-972`
  (`time_control_row`, icona Notebook accanto al testo esistente, layout
  invariato).
- Gestisce esplicitamente tinta selezionato/disabilitato (vedi sopra).
- **Nessun badge mutazione.**
- `tasks/119-moves-icon-restyle-monochrome.md` marcato `[-]`
  (cancellato/superato) in `tasks/QUEUE.md`, con nota che punta al 133 —
  non implementato, non cancellato silenziosamente.
- Criteri: le 4 icone Moves + l'icona Notebook renderizzano (fix del bug
  tofu-box, criterio ereditato da 119), tooltip invariati, `clippy`/`fmt`/
  `test` puliti, verifica live.
- Dipende da: 132. Blocca: nessuno.

## Fuori scope (esplicito, non da riaprire in questi due task)

- Parassita/Simbionte/Estremofilo (nessuna meccanica `Metabolism` a cui
  agganciarli).
- Badge livello mutazione (nessuna meccanica di capacità a livelli).
- Layout del pulsante Notebook come sezione dedicata a piena larghezza
  dopo Species (oggi è dentro `time_control_row`) — questione di
  posizionamento, non di disegno icona; resta un gap noto per una futura
  passata di "visual polish" insieme a sottotesto chip Species (mockup
  punto 5, `photo · cold`) e palette esatta, non affrontato qui.

## Verifica

- `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test`
  dopo ciascun task.
- Verifica live via `cargo run` per entrambi i task (le icone sono
  puramente visive — i test automatici non possono confermare la resa
  reale).

## Prossimo passo suggerito quando si riprende

1. Verificare che i riferimenti a file:linea sopra siano ancora accurati
   (il codice potrebbe essere cambiato nel frattempo).
2. Scrivere `tasks/132-*.md` e `tasks/133-*.md` seguendo il formato di
   `tasks/TASK_BLUEPRINT.md`, usando le sezioni sopra come contenuto.
3. Aggiungere le due righe in `tasks/QUEUE.md` e marcare 119 `[-]`.
4. Implementare 132, poi 133.
