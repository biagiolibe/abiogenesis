# Abiogenesis — HUD e Notebook

Documento autonomo per un task di integrazione: contiene tutto il contesto, le decisioni e il riferimento visivo necessari per implementare l'HUD e il Notebook. Non richiede la lettura di altri documenti di redesign per essere capito, a parte il GDD del gioco per i riferimenti a meccaniche esistenti (moves, biosphere, matrice tag).

## Contesto

Le etichette "Moves", "Biosphere", "Species", "This world wants", "Notebook" sono quelle già in uso nel gioco — questo documento non le rinomina, le riorganizza e le arricchisce con le informazioni che mancano per un controllo reale della partita, mantenendo lo stile console/laboratorio (monospace, colore piatto, indicatori discreti, hairline invece di riquadri bordati) già stabilito in altri task di redesign.

## Decisioni — HUD

### 1. Rinominare "tick" in "pulse"

Proposta di terminologia, da confermare: **pulse** al posto di tick, per agganciarsi diegeticamente all'idea (discussa in un altro task) di un mondo che "respira" — ogni pulse è un passo del respiro della simulazione, un'era è un ciclo completo di pulse. Se il nome non convince, la struttura sotto resta valida sostituendo semplicemente l'etichetta.

### 2. Controllo del tempo: passo singolo + avanzamento continuo

Oggi i comandi sono solo passo-singolo (tick/era). Vanno aggiunti:

- **Avanza pulse** — un passo singolo, come oggi.
- **Avanza era** — un blocco di pulse, come oggi.
- **Toggle avanzamento automatico (play/pausa)** — un terzo controllo che avvia/ferma l'avanzamento continuo dei pulse senza dover premere ripetutamente "avanza pulse". Stato acceso/spento va reso visivamente distinto (es. bordo/riempimento colorato quando attivo), non solo dal simbolo dell'icona.
- Il readout mostra era, stagione corrente e progresso in pulse (es. `era 5 · stagione 2/4 · pulse 14/25`) — con la revisione della scala temporale la stagione è l'unità di decisione e va resa visibile, non solo era e pulse.

### 3. Moves: 4 azioni, con indicatore di livello sulla mutazione

Le 4 azioni sul mondo:

| Azione | Icona (mockup) | Nota |
|---|---|---|
| Seed | pianta | invariata |
| Stress termico | fulmine | nuova azione, shock ambientale localizzato |
| Rimozione forzata (cull) | teschio | invariata |
| Mutazione | fiala | **ha due livelli di capacità, non un solo stato** |

**Livelli di mutazione:** il gioco parte con una mutazione **lieve, solo sui tag** (l'azione altera i tag di un organismo in modo limitato); più avanti (condizione di sblocco da definire in un altro task, es. legata alla progressione) si acquisisce la **capacità di mutazione manuale completa**. L'icona dell'azione mutazione porta un piccolo badge (nel mockup: cerchio con "T") che comunica **quale livello è attivo in questo momento** — non sparisce quando si sblocca il livello completo, cambia forma/colore, così il giocatore sa sempre con che tipo di mutazione sta lavorando, senza dover ricordare a memoria lo stato di progressione.

**Moves rimasti:** indicatore discreto a tacche (es. `●●○` = 2 di 3 rimaste), non una barra continua — stesso principio già stabilito per gli altri contatori piccoli del gioco (ere, azioni).

### 4. Biosphere: trend **e** delta numerico, non solo trend

Ogni riga della lista popolazioni mostra:

- icona di metabolismo (vedi punto 6) + nome specie (colore identificativo)
- **freccia di trend** (▲ crescita / ▼ calo / ▬ stabile) — invariato rispetto a quanto già stabilito
- **delta numerico dall'era precedente** (`+4`, `-2`, `±0`) — **nuovo**, è l'informazione mancante per avere davvero sotto controllo la crescita: il trend da solo dice la direzione, il delta dice quanto e permette di accorgersi in tempo di un'esplosione o un crollo demografico prima che diventi irreversibile
- conteggio popolazione assoluto, allineato a destra

Lista compatta, contenitore ad altezza fissa con scroll interno oltre 3-4 righe visibili (stesso pattern già stabilito per liste con più di 3 elementi).

### 5. Species (banca genomi): icona di metabolismo + info sintetica

Ogni chip specie mostra, oltre al nome:

- **icona di metabolismo** (vedi punto 6 per il set di icone)
- sottotesto sintetico: metabolismo abbreviato + preferenza di temperatura (es. `photo · cold`, `pred · warm`)

Così il giocatore sa cosa sta per seminare senza dover aprire il notebook — l'informazione minima per una decisione informata resta nell'HUD, il dettaglio completo (range numerico, tag associati, popolazione, origine) resta nel Catalog del notebook.

### 6. Set di icone per metabolismo (coerente HUD + Notebook)

Icone piccole, monocolore, semplici a tratto — stesso linguaggio delle altre icone già in uso:

- **Fotolitico** → piccolo asterisco/sole (4 raggi)
- **Predatore** → piccolo trifoglio (3 cerchi sovrapposti)
- *(spazio riservato per future estensioni, es. decompositore, chemiolitotrofo — GDD §5.4)*

Vanno usate identiche in tutti i punti dove compare il metabolismo di una specie: chip Species nell'HUD, righe Biosphere, card nel Catalog del notebook.

### 7. Notebook: pulsante con notifica

Il pulsante che apre il notebook porta un piccolo indicatore (punto colorato) quando ci sono osservazioni nuove non ancora lette dal giocatore — un invito implicito a controllare, non un'interruzione forzata. Nessun testo aggiuntivo, solo il punto.

## Decisioni — Notebook

### 8. Contenuto: tre sezioni, invariate nella struttura già validata

- **Observation log** — invariato rispetto a quanto già definito in altri task: ogni riga ha un indicatore "pulito vs confuso" (pallino verde/ambra) legato al peso dell'osservazione.
- **Relationships** (grafo delle relazioni) — invariato rispetto a quanto già definito: nodi = tag, archi = relazioni osservate, colore = segno dell'effetto, tratto continuo/tratteggiato = confermata/ipotesi, nodo con bordo tratteggiato = tag non ancora coinvolta in nessuna osservazione.
- **Species catalog** — **arricchito**: non più una riga per specie, ma una card con più informazioni:
  - nome + icona di metabolismo (stesso set del punto 6)
  - metabolismo esteso + range di temperatura preferito (es. `fotolitico · temp 0.20±0.15 (cold)`)
  - popolazione corrente
  - tag associati alla specie
  - **era di origine** (quando è stata seminata) — nuovo dato, utile per capire da quanto tempo una specie è nel mondo rispetto alle altre

Lista specie con altezza massima fissa e scroll interno oltre 2-3 card visibili, stesso pattern già stabilito altrove.

### 9. Come si apre il notebook

- **Trigger:** tasto `Tab` oppure clic sul pulsante Notebook nell'HUD (vedi punto 7).
- **Presentazione:** pannello a comparsa dal **lato sinistro** dello schermo, che copre la porzione sinistra della mappa.
- **La mappa di gioco non sparisce**: resta visibile sullo sfondo, **attenuata** (overlay scuro semi-trasparente sopra). **Il tempo è in pausa mentre il notebook è aperto** — deciso, coerente col principio stabilito nella revisione della scala temporale: il tempo non scorre mai mentre il giocatore pianifica (il realtime "vero" è stato scartato). La mappa attenuata ma visibile comunica "il mondo è lì, ti aspetta", non "il mondo continua senza di te".
- **La sidebar HUD resta a destra, visibile e interagibile** — il giocatore può continuare a leggere Biosphere/Species/Time mentre consulta il notebook, senza dover chiudere l'uno per vedere l'altro.
- **Chiusura:** tasto `Tab` di nuovo, `Esc` (chiude solo il notebook, è il livello più alto della cascata — vedi `culture-shock-controls.md`), oppure clic fuori dal pannello (sulla mappa attenuata).

## Immagini

### HUD completo

**Immagine allegata separatamente: `hud-full.svg`** — Mockup dell'HUD con Time, Moves, Biosphere, Species, pulsante Notebook, This world wants, info seed e comandi

Vista d'insieme della sidebar: intestazione con seed e stato, sezione Time con readout e controlli (pulse/era/auto), Moves con le 4 azioni e il badge di livello mutazione, Biosphere con trend+delta per specie, Species con icona e info di metabolismo, pulsante Notebook con notifica, This world wants con il progresso a pallini, footer con seed e scorciatoie.

### Notebook completo, con vista d'apertura

**Immagine allegata separatamente: `notebook-full.svg`** — Mockup del notebook aperto: pannello a sinistra, mappa attenuata dietro, sidebar HUD ancora visibile a destra

Mostra il notebook nel suo stato aperto: Observation log, grafo Relationships, Species catalog con le card dettagliate — **e, sulla destra dell'immagine, la composizione con mappa attenuata e sidebar HUD ancora visibile**, per chiarire visivamente il comportamento descritto al punto 9.

## Cosa serve per l'integrazione (per chi implementa)

- **Toggle auto-avanzamento:** verificare la velocità di avanzamento automatico (pulse/secondo) — non specificata in questo documento, da decidere in fase di implementazione o esporre come parametro.
- **Badge livello mutazione:** il badge deve leggere lo stato di sblocco reale della capacità di mutazione (qualunque sia la condizione di sblocco, definita altrove) e cambiare aspetto di conseguenza — non è un'icona statica.
- **Delta Biosphere:** richiede di conservare il conteggio di popolazione dell'era precedente per calcolare la differenza — verificare se questo dato è già tracciato o va aggiunto.
- **Era di origine nel Catalog:** richiede di registrare, per ogni specie, l'era in cui è stata seminata la prima volta — verificare se questo dato esiste già nel modello o va aggiunto.
- **Notifica sul pulsante Notebook:** richiede un flag "osservazioni non lette dal giocatore", da resettare quando il notebook viene aperto.
- **Banca genomi dinamica:** il roster seminabile cresce durante la partita, perché l'azione Splice sintetizza nuove specie che vi si aggiungono (cfr. documento sulle azioni). L'HUD è oggi progettato assumendo un roster fisso deciso dal world-gen — va previsto l'inserimento di nuove voci in corso di partita, con distinzione visiva tra specie originali e specie sintetizzate dal giocatore.
- **Time: unità e budget aggiornati.** Con la revisione della scala temporale, la stagione diventa l'unità di decisione e il budget di azioni si ricarica a ogni stagione, non a ogni era — il readout e il contatore "moves rimaste" nell'HUD vanno riferiti alla stagione. L'avanzamento continuo (toggle play/pausa) è parte necessaria della struttura, non un extra opzionale.

## Fuori scope

- Icone per metabolismi futuri (decompositore, chemiolitotrofo) — solo spazio riservato nel set, non progettate qui.
- Condizione di sblocco della mutazione manuale completa — solo il badge che la comunica è in scope, non la logica di progressione.
- Bilanciamento numerico (velocità auto-avanzamento, soglie) — da definire in implementazione/playtest.
- Naming definitivo "pulse" — proposta da confermare, non vincolante.
