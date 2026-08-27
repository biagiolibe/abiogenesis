# Abiogenesis — UI/UX redesign notes (v1)

Contesto: redesign di alcune schermate per dare più carattere al gioco, restando fedeli al pilastro 3 del GDD ("il divertimento sta nel sistema, non nella grafica" — quadratini colorati, zero asset artistici). Nessuna di queste modifiche stravolge la simulazione o il modello dati esistente: sono tutte a livello di presentazione.

---

## 1. Hypothesis grid → grafo a nodi (invece di matrice NxN)

**Problema attuale:** la matrice tag×tag, sia nella versione a griglia sia in quella con cerchi fluttuanti scollegati, non comunica in modo leggibile lo stato di conoscenza del giocatore.

**Decisione:** sostituire con un **grafo diretto**, nodi = tag attive nel mondo, archi = relazioni osservate.

- **Layout:** circolare per ≤6 tag attive; force-directed (nodi collegati si attraggono, isolati restano ai margini) per 7-8 tag, per evitare l'effetto "spaghetti" con molti archi.
- **Nodo:** cerchio con il glifo della tag al centro. Bordo tratteggiato = tag non ancora coinvolta in nessuna osservazione (comunica "mistero non ancora toccato" senza testo).
- **Arco:** presente **solo** se esiste almeno un'osservazione per quella coppia ordinata di tag (niente arco = `?`, sconosciuto — non va disegnato nulla, a differenza della cella `?` di una matrice).
  - Colore: rosso = effetto negativo, verde = effetto positivo, grigio = nessun effetto (`0`).
  - Tratto continuo spesso = confermato (evidenza cumulata ≥ soglia, cfr. GDD §7/§5.9). Tratteggiato = ipotesi non ancora confermata.
  - Spessore della linea proporzionale all'intensità dell'effetto (`-2`/`-1`/`+1`/`+2`).
  - Etichetta numerica opzionale solo sugli archi confermati forti, per non affollare il disegno.
- **Interazione consigliata (fase 2, opzionale):** click su un nodo → evidenzia solo i suoi archi in entrata/uscita, sfuma il resto. Utile quando le tag attive salgono a 7-8.

## 2. Observation log — indicatore "esperimento pulito vs confuso"

**Problema attuale:** il log elenca eventi testuali senza comunicare la qualità dell'osservazione, mentre il GDD (§7) rende centrale il concetto "un'osservazione isolata vale di più" (`weight = 1 / (1 + n_confondenti)`).

**Decisione:** ogni riga del log riceve un piccolo indicatore visivo (pallino colorato) prima del testo:

- verde = osservazione pulita (0 confondenti, peso 1.0 o vicino)
- ambra = osservazione confusa (1+ confondenti, peso ridotto)

Non serve mostrare il numero esatto di peso nel log — il colore basta a insegnare per rinforzo "isolare gli esperimenti paga di più", coerente col principio "discovery as progression" del pilastro 2.

## 3. Mappa di simulazione — cue ambientali leggeri

**Vincolo:** rispettare il pilastro 3. Niente texture, niente gradienti continui, niente pattern decorativi pesanti (la prima proposta esplorata andava scartata per questo motivo).

**Decisione:**

- **Tint di sfondo cella** per temperatura/luce, a **3-4 fasce discrete** di colore (non gradiente continuo), stessa logica dei "quadratini colorati" già in uso — è un'estensione della palette esistente, non un nuovo linguaggio visivo.
- **Zona tossica:** contorno tratteggiato sottile, stesso stile visivo usato per i nodi "non osservati" nel notebook (coerenza cross-screen).
- **Forme per specie** (cerchio/triangolo/rombo/...) invece di soli pallini colorati identici — utile fin da subito e pronto per codificare mutazioni in futuro (es. uno spigolo in più, un pattern interno) senza aggiungere asset grafici.
- **Log eventi in-mappa:** striscia testuale minimale in basso (non un pannello nuovo), stile monospace coerente con l'estetica "terminale scientifico" del gioco — riporta 1-2 eventi salienti per tick/era (nascite/morti/mutazioni rilevanti), non un log completo.

## 4. Coerenza cromatica cross-screen

**Decisione:** la stessa terna colore-per-specie (rosso/verde/viola, o quella che si sceglierà) va riusata identica in:
- mappa (colore organismo)
- pannello popolazione (pallino accanto al nome specie)
- catalog nel notebook (pallino accanto al nome specie)

Attualmente ogni schermata sembra reinventare la codifica colore in modo indipendente; unificarla riduce il carico cognitivo senza toccare la logica di gioco.

---

## 5. Barra laterale — da dashboard gestionale a console di laboratorio

**Problema attuale:** 4 riquadri bordati separati (Action, Population, Seed palette, Objective), etichette generiche, progress bar continue in stile SaaS, icone stock.

**Decisione:**

- **Struttura:** un unico pannello continuo diviso da linee sottili (hairline `#23262e` o equivalente tema), niente più box annidati con bordo proprio.
- **Font:** monospace su tutta la sidebar, coerente con l'estetica "field journal / console scientifica" già presente altrove.
- **Etichette diegetiche** invece che da software gestionale:
  - "Action" → "Intervieni"
  - "Population" → "Censimento"
  - "Seed palette" → "Banca genomi"
  - "Objective" → "Direttiva"
- **Icone azione:** SVG semplici e outline, coerenti nello stile (confermato: vanno bene, non serve sostituirle con altro).
- **Contatori discreti al posto delle progress bar continue:** "azioni rimaste" e "ere trascorse" diventano tacche/pallini pieni-vuoti invece di barre percentuali — comunicano meglio che sono risorse contabili piccole, non metriche astratte.
- **Direttiva come frase, non specifica tecnica:** l'obiettivo attivo è presentato tra virgolette in corsivo (font editoriale), unica licenza narrativa della sidebar — da usare con parsimonia, non estendere ad altri elementi.

### 5.1 Scalabilità oltre 3 specie

Il roster di specie crescerà oltre 3, quindi sia il Censimento sia la Banca genomi vanno progettati per N specie fin da subito, non solo per il caso demo a 3:

- **Censimento:** righe compatte a riga singola (pallino colore specie + nome + barretta energia + conteggio popolazione), contenitore con altezza massima fissa e scroll interno oltre le ~4-5 righe visibili. Niente crescita illimitata della sidebar.
  - **Correzione importante sulla barra energia:** l'energia **non ha un tetto massimo** — il valore "10.0" visto nello screenshot originale è `repro_threshold` (§5.9 GDD), la soglia oltre la quale l'organismo si riproduce, non un cap. La barra va quindi calibrata come *progresso verso la riproduzione* (`energy / repro_threshold`, clampato al 100% visivo), non come percentuale di un massimo inesistente. Se l'energia supera la soglia, la barra satura a piena e mostra un piccolo indicatore ("pronta a riprodursi") invece di troncare silenziosamente il dato. Poiché `repro_threshold` è definito nel genoma per specie (§5.3), il calcolo va fatto per-specie, non con un valore fisso globale.
- **Banca genomi:** da griglia di chip a **striscia orizzontale scorrevole** — la griglia multi-riga usata nei primi mockup funziona solo fino a 3-4 specie, oltre diventa un blocco troppo alto.

### 5.2 Nota sui glifi delle tag

I glifi greci (ε, β, κ, α, ζ) usati nei mockup del notebook e del grafo erano segnaposto. È in programma sostituirli con simboli più vicini alla biochimica reale (es. notazione ispirata a gruppi funzionali, strutture semplificate) — il grafo a nodi e la matrice restano validi come struttura, cambia solo cosa viene disegnato dentro ai nodi/celle. Da definire in una sessione dedicata quando la direzione grafica delle tag sarà pronta.

---

## Non in scope per questa iterazione

- Redesign del pannello Action/Objective (solo nota: riusare la stessa palette specie, nessun altro cambiamento proposto).
- Qualsiasi asset grafico/illustrativo — resta fuori target per pilastro 3.
- Layout force-directed del grafo (proposto solo come estensione futura per mondi con 7-8 tag attive).
