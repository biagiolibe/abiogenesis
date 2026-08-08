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

## Non in scope per questa iterazione

- Redesign del pannello Action/Objective (solo nota: riusare la stessa palette specie, nessun altro cambiamento proposto).
- Qualsiasi asset grafico/illustrativo — resta fuori target per pilastro 3.
- Layout force-directed del grafo (proposto solo come estensione futura per mondi con 7-8 tag attive).
