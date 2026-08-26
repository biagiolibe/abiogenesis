# Culture Shock — modello di popolazione per cella ed estetica

Documento autonomo. Copre due decisioni collegate: un cambio del modello di simulazione (popolazione per cella invece di un organismo per cella) e le sue conseguenze dirette sul rendering; più le decisioni estetiche generali già discusse (materiale del notebook, codifica colore/forma).

## Perché cambiare il modello, non solo il disegno

Il tentativo iniziale di risolvere "come rendere leggibile la mappa a due zoom" partiva dal disegno: overview a densità disegnata ad arte, dettaglio con soglie di collasso in blocco. Il problema è più a monte: **con un organismo per cella, l'overview a densità è un'illusione pittorica sopra un modello che non la sostiene** — la vista non legge uno stato reale della simulazione, lo approssima per essere disegnata meglio. Il modello va cambiato alla radice, non solo il modo in cui lo si disegna sopra.

## Il modello

**Ogni cella tiene un conteggio di individui di una sola specie** (mai condivisa tra specie diverse — resta dedicata), più un'energia aggregata di quella popolazione locale, fino a una **capacità portante** per cella.

- **Guadagno e mantenimento restano pro-capite**, calcolati con le formule già riviste (documento sul bilanciamento), applicate all'energia aggregata invece che a un singolo organismo. Nessun coefficiente cambia.
- **La riproduzione diventa crescita continua**: quando l'energia media pro-capite supera la soglia, il conteggio della cella cresce, l'energia aggregata si riduce del costo corrispondente.
- **Sfondamento:** quando la popolazione supera la capacità, l'eccesso emigra verso una cella adiacente **vuota o già della stessa specie** — mai verso una cella occupata da un'altra specie, perché la cella resta dedicata a una sola.

### Interazione della matrice: per presenza, non per quantità **[deciso]**

Una cella vicina con un certo tratto conta come un contributo, **indipendentemente da quanti individui contiene** — concettualmente identico a oggi, dove "vicino" era un singolo organismo e ora è una cella-popolazione. **Il coefficiente 0.15 e l'intera ritaratura del documento sul bilanciamento restano validi senza modifiche.** Scartata l'alternativa "per quantità" (l'effetto scala con la numerosità): più realistica ma introduce un rischio di retroazione che si autoalimenta e avrebbe richiesto di rifare la ritaratura da capo.

### Cella dedicata a una specie, non condivisa **[deciso]**

Niente micro-ecosistema dentro la stessa cella. Effetto collaterale non scritto ma emergente dal modello: **una specie interamente circondata da altre specie diverse ha la crescita bloccata**, perché non ha dove sfondare — competizione spaziale reale, senza bisogno di scriverla come regola a parte.

### Popolazione bloccata → pressione selettiva locale **[deciso]**

Quando una cella è satura e non ha nessuna cella adiacente libera o della stessa specie verso cui sfondare, l'energia in eccesso **non va sprecata**: alimenta la pressione selettiva locale (§5.11 GDD), sotto lo stimolo già esistente "disallineamento ambientale". Non è un nuovo motivo di speciazione — è una causa reale data a uno stimolo che il sistema già prevedeva in modo generico. Una popolazione compressa che non può espandersi è, meccanicamente e narrativamente, precisamente il tipo di pressione che dovrebbe spingere verso una speciazione.

### Osservazione pulita — reinterpretazione minima

Con una cella = una specie, "isolato" torna quasi a coincidere con la definizione originale: quante celle vicine contengono un tratto diverso. Il peso (`1/(1+confondenti)`, §7 GDD) non cambia formula, cambia solo cosa si conta come vicino (celle-popolazione invece di organismi).

### `Cull`, più netto

Azzera l'intera popolazione locale della cella colpita. Con una cella = una popolazione, il knockout diventa un esperimento più leggibile di prima: rimuovi un intero avamposto, non un individuo statisticamente irrilevante su uno sciame numeroso — coerente con la cornice "esperimento di rimozione" già scelta (documento azioni).

## Cosa NON cambia

- La matrice dei tratti e le sue relazioni.
- `Splice` — continua a seminare una piccola popolazione iniziale, nessun cambio concettuale.
- I biomi.
- Il meccanismo di speciazione in sé (agisce già a livello di specie, non di singolo individuo — era già compatibile).
- Il coefficiente di interazione e gran parte della ritaratura del documento sul bilanciamento.

## Conseguenza sul rendering — nessun artificio necessario

**Immagine allegata separatamente: `aesthetic-model-b.svg`** — Overview a densità reale e dettaglio con zona di pressione da sfondamento bloccato

L'immagine mostra il modello finale: a sinistra l'overview con densità reale (popolazione ÷ capacità, non più disegnata ad arte), a destra il dettaglio con una forma per cella e badge di conteggio, con la cella satura e priva di sbocco evidenziata come zona di pressione selettiva.

Il problema di disegno del giro precedente (molte forme identiche affollate, soglie di collasso da tarare) **si risolve da solo con questo modello**, senza bisogno di alcuna soglia arbitraria: ogni cella-specie è già, nel dato, un singolo aggregato. Non si simulano N individui per poi fingerli come un blocco in fase di disegno — il dato stesso è uno solo.

- **Dettaglio:** una forma per cella (codificata per metabolismo, come già stabilito), con un badge numerico se il conteggio supera 1. Una cella satura e priva di sbocco per lo sfondamento porta un indicatore visivo di pressione locale.
- **Overview:** stessa mappa, densità come **dato reale** (popolazione ÷ capacità per cella) invece che come puntini disegnati ad arte — colore per bioma-specie, opacità per densità.
- **Due soli livelli di zoom**, nessuno stadio intermedio con una sua codifica propria: lo zoom stesso, ingrandendo le forme, fa la transizione percettiva.

## Costo e non-costo dell'operazione

**Tocca:** le fasi 2-5 della pipeline del tick (guadagno, interazione, costi, aggiornamento energia — riscritte per lavorare su aggregati per cella invece che per organismo), il pannello Biosphere dell'HUD (diventa più semplice: legge direttamente gli aggregati invece di doverli ricostruire).

**Non tocca:** Splice, la matrice dei tratti, i biomi, il meccanismo di speciazione in sé, il coefficiente di interazione.

**Nota di performance:** aggregare per (cella, specie) è probabilmente più leggero, a runtime, dell'iterare migliaia di organismi singoli in condizioni di alta densità — uno dei rari casi in cui la scelta concettualmente più pulita è anche la più economica.

---

## Decisioni estetiche generali

### Notebook: materiale "ardesia" **[deciso]**

Non carta (era stata scartata: troppo vicina al quaderno scolastico), non bianco (troppo clinico). Un grigio-blu scuro ma più chiaro della mappa — stessa famiglia cromatica, si legge come un pannello distinto appoggiato sopra, resta dentro il mondo "strumento" invece di uscirne verso il cartaceo.

**Immagine allegata separatamente: `aesthetic-final-slate.svg`** — Mappa scura e notebook ardesia affiancati, con la palette dei segni ritarata per funzionare su entrambi i fondi

**Nota su altre immagini prodotte durante l'esplorazione** (`aesthetic-dual-material.svg`, `aesthetic-notebook-variants.svg`, `aesthetic-notebook-dark-variants.svg`, `aesthetic-organism-encoding.svg`, `aesthetic-organism-more.svg`, `aesthetic-refined.svg`, `aesthetic-zoom-sequence.svg`): documentano alternative scartate o superate nel corso della discussione (carta, bianco puro, colore per specie, tre livelli di zoom, soglie di collasso in blocco) — utili come registro del ragionamento, ma **non riflettono le decisioni finali** di questo documento. Le due immagini sopra sono le uniche coerenti con lo stato deciso.

### Colore = ambiente, forma = vita **[deciso]**

Regola generale: **il colore appartiene all'ambiente** (biomi, scalari), **la forma appartiene alla vita** (metabolismo delle popolazioni). Le due informazioni non competono più sullo stesso canale.

- Colore dei biomi: invariato.
- Organismi: forma per metabolismo (asterisco/fotolitico, triangolo/predatore, rombo/decompositore, esagono/chemiolitotrofo — stesso set già definito per le icone specie), **un solo inchiostro neutro** per tutte le forme, mai colorate per specie.
- **Inchiostro: ambra spenta**, non bianco puro — meno clinico, resta neutro.
- Riempimento pieno/vuoto della forma per lo stato energetico della popolazione locale (pieno = sopra soglia critica, vuoto/contorno = in affanno).

**Cosa si perde, consapevolmente:** la mappa non distingue più a colpo d'occhio due specie con lo stesso metabolismo — quell'identità resta nel censimento e nel notebook, non sulla griglia. Scambio accettato: il colore torna a significare una cosa sola, funziona con qualunque numero di specie senza esaurire una palette, ed è accessibile per costruzione.

## Rendering a grana pixel **[deciso]**

Raffinamento di *come* si disegnano le decisioni sopra (colore-ambiente/forma-vita, notebook ardesia), non una loro revisione. Motivazione: l'interfaccia vettoriale liscia comunicava "strumento tecnico" ma non "calore" — la richiesta era di avvicinarsi alla tattilità di titoli come Stardew Valley o Dwarf Fortress **senza reintrodurre una pipeline di asset disegnati a mano**, che romperebbe la stessa condizione abilitante (contenuto quasi a costo zero) che ha reso possibile tutto il resto — 15+10 tratti, 16 biomi, xenotratti.

**Due tecniche, entrambe procedurali al 100%, combinate:**

1. **Forme organismo scattate su una griglia di pixel** invece di path vettoriali lisci — le stesse identità già codificate (asterisco/fotolitico, triangolo/predatore, rombo/decompositore, esagono/chemiolitotrofo), ricostruite come piccoli pattern di blocchi. Nessun asset nuovo: è la stessa geometria di prima, solo quantizzata su una griglia invece che disegnata a curve morbide.
2. **Texture procedurale a grana pixel sui biomi** — un rumore leggero generato algoritmicamente sopra il colore piatto, al posto del dithering a due toni fisso. Rompe la piattezza senza introdurre un pattern disegnato a mano.

**Estensione allo stesso registro su tutta l'interfaccia, non solo la mappa:** icone azione (Seed/Stress/Cull/Splice) ricostruite come blocchi pixel, indicatori a tacche (moves rimasti) come blocchi netti invece di barre arrotondate, bordi squadrati ovunque (via gli angoli arrotondati). Motivazione: un HUD vettoriale liscio accanto a una mappa a pixel avrebbe creato una frattura stilistica — l'intera interfaccia deve respirare lo stesso registro.

**Nel notebook, lo stesso trattamento si estende anche alle connessioni del grafo delle relazioni**, non solo ai nodi: gli archi diventano **percorsi a scalino** (segmenti orizzontali e verticali in pixel) invece di linee diagonali morbide — coerente col principio "niente linee lisce" applicato fino in fondo, non solo alle forme.

**Cosa resta invariato:** il testo, sempre monospace nitido — non aveva bisogno di alcun trattamento pixel, si comportava già bene. La regola colore-ambiente/forma-vita, il materiale ardesia del notebook, l'inchiostro neutro per gli organismi (qui una variante più calda: ambra piena `#e0c99a` per stato energetico sopra soglia, ambra spenta per sotto soglia, invece di pieno/vuoto puro).

**Immagini di riferimento**, tutte coerenti con lo stato deciso:
- `pixel-art-compare.svg` — le due tecniche isolate, a confronto, prima di essere combinate.
- `pixel-full-scene.svg` — mappa e HUD nella versione finale combinata.
- `pixel-notebook.svg` — il notebook completo nelle sue quattro sezioni (Observation log, Relationships con archi a scalino, Species catalog, Chronicle) nello stesso registro.

**Perché non l'opzione 3 (tileset pixel art vera), scartata a monte:** richiederebbe uno sprite per elemento (bioma, metabolismo, transizione), reintroducendo esattamente il costo di produzione che il progetto ha evitato fin dall'inizio — ogni tratto o bioma nuovo tornerebbe a essere un costo di contenuto invece che una riga di dati.

## Cosa serve per l'integrazione

- **Libreria di pattern a blocchi per le 4 forme di metabolismo**, coerente tra mappa, HUD e notebook — un solo set riusato ovunque, non uno per contesto.
- **Generatore di texture procedurale per bioma**, parametrizzato (seed, densità del rumore) invece di un pattern di dithering fisso.
- **Router di rendering per i percorsi a scalino** nel grafo delle relazioni, al posto di linee/curve dirette.
- **Aggiornamento dei componenti HUD esistenti** (icone azione, indicatori a tacche, bordi) allo stesso registro squadrato — non un sistema nuovo, un ripasso di stile su ciò che è già stato specificato in `abiogenesis-hud-notebook.md` e `abiogenesis-sidebar-redesign.md`.
- **Ridefinizione della struttura dati di cella:** da organismo opzionale a popolazione (specie, conteggio, energia aggregata) opzionale.
- **Capacità portante per cella:** valore da definire, eventualmente modulato per bioma (coerente con la proposta già scritta nel documento sulla pipeline del tick per `crowd_factor`).
- **Logica di sfondamento:** ricerca di cella adiacente idonea (vuota o stessa specie) quando la capacità è superata.
- **Rilevamento di saturazione senza sbocco**, per alimentare la pressione selettiva locale — nuovo aggancio nella fase 6 della pipeline del tick.
- **Rendering overview a densità reale** (popolazione ÷ capacità), sostituendo qualunque logica di densità disegnata.
- **Rendering dettaglio:** libreria delle 4 forme di metabolismo con badge numerico e riempimento pieno/vuoto, in ambra spenta su ardesia/mappa.

## Fuori scope

- Valore numerico della capacità portante per cella/bioma — da bilanciare in playtest.
- Se e come la capacità debba variare per bioma oltre al principio già proposto per `crowd_factor`.
- Dettaglio implementativo della ricerca di cella idonea per lo sfondamento (ordine di priorità tra celle adiacenti, comportamento con più direzioni ugualmente valide).
