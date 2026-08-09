# Abiogenesis — proposte per un onboarding e un'identità di gioco più forti

Documento autonomo: contiene diagnosi, meccaniche proposte e priorità. Non richiede la lettura di altri documenti di redesign, solo familiarità col GDD del gioco per i riferimenti a formule/costanti citate.

## Contesto e obiettivo

Il loop centrale (semina → avanza era → osserva → ipotizza → interveni) è solido a regime, ma **i primi minuti di gioco non generano ancora la sensazione voluta**: un mondo vivo che il giocatore osserva e guida, era dopo era, dentro un'atmosfera di mistero da scoprire. Il feedback riportato dal designer dopo aver giocato:

- Nessuna stabilità percepita all'inizio.
- Nessuna sensazione di "mondo che si avvia verso un'evoluzione che guiderò io".
- Il gameplay percepito come "solo piazzamento di seed con tag limitati che interagiscono".
- La riproduzione, una volta partita, sembra troppo veloce.

## Diagnosi (radicata nei numeri e nelle regole attuali del GDD)

1. **Il turno zero è un foglio bianco assoluto.** Nessuna specie è auto-piazzata (rimozione intenzionale, task 050): il mondo comincia completamente vuoto, senza alcun segnale che orienti il giocatore. Per un gioco il cui pilastro è "scoperta come progressione", i primi minuti dovrebbero già comunicare che c'è qualcosa da scoprire, non aspettare che il giocatore costruisca tutto da zero.

2. **La riproduzione è oggettivamente troppo veloce nei numeri esistenti.** Energia di partenza `5.0`, soglia di riproduzione `10.0`, guadagno fotolitico isolato ≈`+0.9`/tick (§5.9 GDD) → un organismo isolato si riproduce in ~6 tick. Un'era dura `25` tick di default (§5.9). Significa che **entro la prima era**, prima che il giocatore veda un risultato, la popolazione ha già attraversato più cicli riproduttivi: non si "guarda crescere" nulla, si preme spazio e ci si ritrova già davanti a uno sciame.

3. **Le interazioni della matrice sono invisibili finché non sono confermate.** L'`interaction_delta` (§5.6) si applica dal primo tick, ma il giocatore lo percepisce solo indirettamente (una specie che si indebolisce o muore), e lo capisce solo dopo aver accumulato evidenza sufficiente (§7). Nei primi minuti il mistero non si percepisce come mistero: si percepisce come rumore, perché manca un segnale che dica "qui è appena successo qualcosa di significativo".

In sintesi: il loop funziona a regime, ma l'onboarding non è tarato per farlo scattare nei primi minuti — che è il momento in cui un giocatore decide se il gioco gli piace.

---

## Parte 1 — Meccaniche di onboarding (priorità: fondamenta, da affrontare per prime)

### 1.A Feedback visivo istantaneo sull'interazione ("lo spark")

Ogni volta che l'`interaction_delta` della matrice si applica su una cella, un impulso visivo immediato (basta un lampo/pulsazione di un frame) segnala che tra due organismi è appena successo qualcosa — anche prima che il giocatore capisca cosa. Trasforma la matematica nascosta in un evento osservato in tempo reale, invece che dedotto a posteriori guardando popolazioni che cambiano.

- **Costo:** minimo — non tocca bilanciamento, solo presentazione di un dato già calcolato ad ogni tick.
- **Impatto:** probabilmente il più alto rispetto al costo, di tutta questa lista.

### 1.B "Prima luce" garantita nel world-gen del primo mondo

Non piazzare specie per il giocatore (resta vietato), ma garantire — solo per world 0, solo come vincolo aggiuntivo di generazione — che una relazione forte (`±2`) della matrice sia raggiungibile e visibile entro la prima era, in una zona ragionevolmente vicina a dove il giocatore semina per primo.

- **Non contraddice il pilastro dell'imprevedibilità:** non si decide *cosa* succede, solo che *qualcosa* di chiaro succeda presto.
- **Impatto:** trasforma "forse in 10 minuti vedo il primo aha" in "il primo aha arriva quasi sempre entro l'era 1-2".

### 1.C Incubazione: i neonati non si riproducono nella stessa era in cui nascono

Regola aggiuntiva, non tocca `repro_threshold` né i guadagni esistenti (che restano bilanciati per il mid-game): un organismo appena nato deve sopravvivere almeno un'era intera prima di poter a sua volta riprodursi.

- **Effetto:** rallenta lo sciame iniziale senza alterare l'economia energetica del resto del gioco — dà la sensazione di "vedere crescere una generazione alla volta" invece di un'esplosione istantanea.

### 1.D Ere più corte solo all'inizio (onboarding, non baseline)

Le prime 2-3 ere di world 0 avanzano a blocchi ridotti (es. `8` tick invece di `25`) — stesso spirito della grace period già prevista (task 079): un'eccezione mirata all'apprendimento, non un cambio del bilanciamento globale. Più checkpoint proprio nei minuti in cui il giocatore impara a leggere il sistema; poi si torna al ritmo standard una volta che il loop ha "scattato" almeno una volta.

### 1.E Il mondo respira anche prima che il giocatore semini

La diffusione ambientale lenta è già prevista (Phase 1+, §5.2). Vale la pena mostrarla da subito, anche a griglia vuota — un gradiente che si muove impercettibilmente, una zona tossica che pulsa piano. Costo di design pressoché nullo (il dato esiste già), ma comunica fin dai primi secondi che il mondo ha una sua chimica indipendente dal giocatore — coerente con il tono "hard-SF in una piastra di Petri" della vision del gioco.

---

## Parte 2 — Meccaniche "epiche" (esplorazione ambiziosa, oltre l'onboarding)

Elenco completo per contesto, con priorità di sviluppo assegnata in coda a ciascuna.

### 2.1 Il registro stratigrafico — leggere la storia nel terreno

Ogni cella che ha visto morte accumula, era dopo era, uno strato persistente comprimibile: chi è morto lì, quando, in che concentrazione — non solo il residuo che nutre i decompositori (già esistente), ma una traccia leggibile nel tempo. Il giocatore può "carotare" una cella e leggerne la sequenza, come una carota di ghiaccio: un punto della mappa diventa un libro di storia geologica scritto dal giocatore stesso senza che se ne accorgesse. Nessun asset grafico nuovo, nessuna nuova simulazione — solo un log persistente per cella invece che globale.

**Priorità: futuro.** Cambia davvero la sensazione del gioco ma richiede più lavoro di modellazione dati (log per cella, non solo globale) rispetto alle altre proposte.

### 2.2 Il Precursore — un'anomalia che il giocatore non ha seminato

Una singola cella fissa per mondo, non una specie auto-piazzata (resta vietato), con una chimica anomala e costante: emette un effetto sulla matrice indipendentemente da quale tag il giocatore le porti vicino, come se qualcosa fosse già stato lì prima del suo arrivo. Diventa un polo di attrazione narrativo — il giocatore vuole scoprire cosa succede vicino a quel punto — senza una riga di narrativa scritta a mano: è letteralmente una cella che si comporta diversamente dalle altre, scopribile solo osservando.

**Priorità: subito, da approfondire e aggiungere.** Costo di sviluppo molto basso, ritorno "epico" molto alto.

### 2.3 Dati di un'esplorazione precedente — testimonianze da verificare, non risposte

In alcuni mondi, il notebook parte con alcune celle della matrice **pre-compilate**, etichettate come "rilevazioni precedenti, non verificate" — alcune corrette, alcune sbagliate. Non sono risposte sbloccate (resta coerente con la meta-progressione: "si sbloccano capacità, non risposte" — §10 GDD): sono testimonianze da verificare. La scoperta diventa anche un esercizio di scetticismo scientifico — fidarsi o rimettere tutto in discussione.

**Priorità: futuro.** Vero moltiplicatore di mistero, ma richiede progettazione attenta di come generare testimonianze "credibili ma talvolta false" senza rompere la fiducia del giocatore nel notebook come strumento.

### 2.4 Eventi epocali — il mondo che mette alla prova, non solo il tempo che scade

Shock globali rari e procedurali, costruiti sopra gli scalari ambientali già esistenti (`temperature`, `light`, `toxicity`): un picco improvviso di tossicità globale, un flare che raddoppia la luce ovunque per 2 ere, un collasso termico. Non serve un nuovo sistema — è scripting sopra dati che già esistono. Dà a ogni run un arco narrativo naturale (crescita → shock → adattamento o collasso) senza scrivere nessuna storia a mano: la genera la simulazione colpendo le scelte del giocatore.

**Priorità: subito, da approfondire e aggiungere.** Nessun nuovo sistema di dati richiesto, solo eventi che manipolano scalari esistenti — alto impatto narrativo per costo di sviluppo contenuto.

### 2.5 La forma nascosta dietro le forme — una grammatica tra i mondi

Ogni mondo rimescola la matrice (deve restare così), ma sotto il rumore potrebbe esistere una regolarità strutturale più profonda, percepibile solo da un giocatore esperto dopo molti run — es. certi pattern di glifi tendono statisticamente a comportarsi da catalizzatori, altri da veleni, non per un singolo mondo ma come tendenza del generatore stesso. Nessun mondo diventa prevedibile singolarmente, ma i veterani guadagnano un secondo livello di mistero, meta, sopra il primo — coerente con lo spirito "leggi universali sotto il caos apparente" della vision del gioco.

**Priorità: futuro, da definire bene in un secondo momento.** La più ambiziosa e la più a lungo termine: è quella che può dare al gioco "un'anima", ma anche la più delicata da bilanciare — rischia di introdurre pattern reali sfruttabili se non progettata con cura statistica.

### 2.6 Il momento della rivelazione — un mondo che risponde quando il giocatore lo capisce

Alla chiusura di un mondo, invece di passare semplicemente al successivo, il gioco genera **una singola frase**, sintetizzata algoritmicamente dalla struttura del grafo di relazioni confermate (non scritta a mano — derivata dai dati: es. "in questa biochimica, la decomposizione genera vitalità" se è stato confermato un ciclo D→P positivo). Zero arte, zero scrittura manuale, un algoritmo su dati che il gioco già possiede — ma un payoff emotivo alto al lavoro di decodifica: il momento in cui il giocatore sente di aver davvero capito qualcosa di vero su quel mondo.

**Priorità: subito, da approfondire e aggiungere.** Puramente derivato da dati esistenti (il grafo di relazioni confermate), nessun nuovo sistema di simulazione richiesto.

---

## Riepilogo priorità

| Voce | Priorità |
|---|---|
| 1.A – Feedback visivo istantaneo ("spark") | Fondamenta onboarding |
| 1.B – "Prima luce" garantita in world 0 | Fondamenta onboarding |
| 1.C – Incubazione neonati | Fondamenta onboarding |
| 1.D – Ere più corte a inizio partita | Fondamenta onboarding |
| 1.E – Il mondo respira prima della semina | Fondamenta onboarding |
| 2.2 – Il Precursore | **Subito, da approfondire e aggiungere** |
| 2.4 – Eventi epocali | **Subito, da approfondire e aggiungere** |
| 2.6 – Il momento della rivelazione | **Subito, da approfondire e aggiungere** |
| 2.1 – Registro stratigrafico | Futuro |
| 2.3 – Testimonianze da verificare | Futuro |
| 2.5 – Grammatica nascosta tra i mondi | Futuro, da definire bene in un secondo momento |

## Fuori scope per questa iterazione

- Implementazione concreta di 2.1, 2.3, 2.5 (solo direzione concettuale per ora).
- Qualsiasi asset grafico o illustrativo — tutte le proposte sono compatibili col pilastro "il divertimento è nel sistema, non nella grafica".
- Bilanciamento numerico fine delle proposte di Parte 1 (valori come "8 tick" o soglie di incubazione sono indicativi, da validare in playtest).
