# Culture Shock — anatomia di una partita (esempio aggiornato)

Documento autonomo. Sostituisce l'esempio del GDD §16, esplicitamente marcato come superato (unità di decisione in ere non stagioni, vecchi coefficienti energetici, glifi greci invece di codici a tre lettere). Racconta una partita seguendo **tutte** le decisioni prese nella sessione di design che ha prodotto v0.7 e i documenti successivi, con numeri reali dove possibile — è anche un modo per verificare che i pezzi si incastrino davvero, non solo sulla carta.

Griglia ridotta a scopo illustrativo (in gioco 128×80). Specie: **Halo** (fotolitico), **Rask** (predatore), **Muck** (decompositore). Tratti mostrati con i codici reali: CHL, QRM, PRN, LIP.

---

## Setup del mondo

World 0, quindi già ammorbidito: **4 tratti attivi** invece di 5, obiettivo iniziale ammorbidito (`Coexistence`, min 2 specie), e un obiettivo aggiuntivo riservato a world 0, `Prima conferma` (conferma una relazione entro N stagioni). Sequenza completa: 2 obiettivi + `Speciation` finale.

Ambiente: gradiente di luce alto-basso, gradiente termico freddo-caldo, una piccola zona a tossicità elevata in un angolo (bioma Palude). Il mondo "respira" già prima che io semini: il drone ambientale è percettibile, sommesso, e uno sguardo alla mappa overview mostra densità zero ovunque — non c'è ancora vita, ma l'ambiente esiste.

## Turno per turno

### Stagione 1 — solo ambiente

Semino **Halo** in una cella a luce alta (bioma Pianura). Nessun vicino, quindi la matrice non ha ancora nulla su cui agire.

Avanzo un pulse alla volta all'inizio, per abituarmi al ritmo. Con lo **strumento di ispezione** (click sulla cella) leggo il bilancio del tick:

```
guadagno (luce)      +0.56
mantenimento         −0.50
affollamento          0.00
────────────────────────────
netto                +0.06
```

Quasi pareggio — coerente con la correzione di bilanciamento: l'ambiente da solo tiene in vita, non fa prosperare. Nessun allarme: è il segnale onesto previsto, non un bug.

### Stagione 2 — la matrice entra in gioco

Semino **Rask** nella cella adiacente a Halo. Il tooltip da hover mi dice solo nome e trend; clicco per la card completa e vedo che Rask porta il tratto **QRM**.

Avanzo la stagione. Al pulse successivo, l'ispezione su Halo mostra una riga nuova nel bilancio:

```
guadagno (luce)      +0.56
QRM (vicino)         +0.30
mantenimento         −0.50
affollamento          0.00
────────────────────────────
netto                +0.36
```

QRM ha un'interazione `+2` su Halo — non lo sapevo, l'ho scoperto guardando il segno del contributo. Il notebook registra l'osservazione: adiacenza isolata (nessun altro vicino), peso 1.0 — la più pulita possibile. Nel grafo delle relazioni compare un primo arco tratteggiato ambra (ipotesi, non ancora confermata): CHL → QRM, positivo.

Con questo, l'obiettivo `Prima conferma` di world 0 è quasi soddisfatto — manca solo che l'evidenza si accumuli oltre soglia.

### Stagione 3-4 — crescita, poi saturazione

Halo cresce: il conteggio della cella sale (popolazione per cella, non più un organismo singolo). Al superamento della soglia di riproduzione pro-capite, il conteggio aumenta e l'energia aggregata si riduce del costo — continuo a controllare col tooltip, che ora mostra "Halo · 4" con freccia in crescita.

Ho seminato **Muck** (decompositore) dall'altro lato di Halo, per curiosità. Errore tattico, lo scopro presto: quando la popolazione di Halo satura la capacità della cella, cerca uno sbocco — ma Rask e Muck occupano entrambe le celle adiacenti libere. Nessuno sfondamento possibile.

L'ispezione segnala l'avviso: *cella satura, nessuno sbocco → pressione in accumulo.* Sotto lo stimolo "disallineamento ambientale" (§5.11), non per il clima ma per lo spazio.

### Stagione 5 — la decisione: aspettare o forzare

Ho due strade, entrambe legittime:

**Aspetto.** Non tocco nulla, avanzo stagioni. La pressione da saturazione continua ad accumularsi.

**Forzo.** Uso **Stress** (asse termico) sulla cella di Halo, ripetutamente per un paio di stagioni — la spingo fuori dalla sua comfort zone di proposito, accelerando lo stimolo di disallineamento indipendentemente dalla saturazione.

Scelgo di forzare: costa budget (1 punto per applicazione, ricaricato per stagione), ma voglio vedere una speciazione prima che il budget di ere del mondo si esaurisca.

### Stagione 7 — reveal, tier notevole

Fine dell'era. Il gioco si ferma da solo e mostra il reveal, generato dal sistema di frammenti componibili:

> *"CHL, sotto pressione sostenuta, ha superato la soglia di riorganizzazione. Halo-B si distingue da Halo."*

Tier **notevole** (non epocale — è la prima speciazione del mondo, ma non tocca un tratto raro). Il beat mostra un confronto prima/dopo: l'icona di Halo-B ha una lieve variazione nella forma pixel, segnalando un tratto diverso.

La Cronaca archivia l'evento con la specie genitore: *"era 2 · Halo-B ⟵ speciazione da Halo (disallineamento ambientale)"*. Il Catalog di Halo-B mostra "discende da: Halo".

L'obiettivo `Speciation` finale, se attivo, si sarebbe aggiornato qui — ma essendo la prima speciazione del mondo, resta nella forma generica (nuova speciazione dopo l'attivazione, sopravvivenza di un'era), non richiede ancora un bersaglio specifico.

### Stagione 9 — la strada del laboratorio

Il notebook ha confermato CHL→QRM oltre soglia (arco solido verde nel grafo, non più tratteggiato). Decido di non aspettare un'altra speciazione naturale: uso **Splice**. Poiché QRM è confermato, posso sintetizzare una nuova specie che lo incorpora di serie — costo 2 punti. Nasce **Halo-C** nella banca genomi, non ancora sul mondo.

La semino (1 punto aggiuntivo) in una zona diversa, lontana da Rask. Nel Catalog, l'origine di Halo-C è marcata "sintetizzata" — distinta sia da "seminata" (Halo originale) sia da "discende da" (Halo-B).

### Stagioni successive — verso gli obiettivi

`Coexistence` (min 2 specie) è già soddisfatto da tempo. `Prima conferma` era già chiuso alla stagione 2. Restano da chiudere l'obiettivo `Speciation` finale (già in corso) e, se il mondo lo richiede, un secondo obiettivo della sequenza — supponiamo `Tolerance`: mantenere una specie viva nella zona a tossicità elevata. Semino **Muck** lì (i decompositori tollerano meglio il residuo, e la sua energia è meno legata alla luce) e lo mantengo per la durata richiesta.

### Vittoria come flag, non fine

Con tutti gli obiettivi soddisfatti, il mondo è **vinto** — ma non finisce. Resta budget di ere. So che Halo-B discende da Halo con una copertura di famiglia strutturale (il nuovo tratto è di famiglia diversa da CHL). Se un'altra speciazione imparentata aggiungesse una terza famiglia, la lineage diventerebbe candidata all'Emersione. Non ho garanzie — è probabilistico anche a condizioni soddisfatte — ma decido di restare a osservare qualche stagione in più prima di passare al mondo successivo, invece di uscire subito.

Non accade in questo mondo. Scelgo di lasciarlo. La schermata di uscita mostra: relazioni confermate (3 su 16 attive in quel mondo), ere residue nel budget, e la frase di chiusura generata dal grafo — qualcosa come *"in questa biochimica, QRM ha agito come catalizzatore di crescita ovunque incontrato."*

## I pattern decifrati in questa run

- **Nicchia ambientale** (stagione 1): l'ambiente da solo tiene in vita, non fa crescere.
- **Interazione della matrice scoperta per osservazione pulita** (stagione 2): un'adiacenza isolata, peso 1.0, prima ipotesi poi conferma.
- **Competizione spaziale emersa dal modello**, non scriptata (stagioni 3-4): una popolazione bloccata da vicini di specie diverse.
- **Pressione selettiva come scelta deliberata** (stagione 5): aspettare o forzare con Stress, entrambe legittime.
- **Le due strade all'evoluzione**, tenute distinte per design: speciazione naturale (gratuita, emergente, imprevedibile nel risultato) vs Splice (costosa, deliberata, limitata a ciò che è confermato).
- **Vittoria come traguardo, non fine**: la tensione di restare per l'Emersione anche dopo aver vinto.

Su un totale di 16 relazioni possibili in quel mondo (4 tratti attivi, T²), ne sono state confermate solo 3 — **e questo è bastato per vincere**. Coerente col principio che il gioco non richiede mai di decifrare l'intera matrice, solo la parte rilevante.

## Cosa questo esempio verifica

Ripercorrerlo per intero mostra che i pezzi decisi in sessioni diverse si parlano correttamente: il modello di popolazione alimenta la pressione selettiva, la pressione selettiva alimenta il reveal, il reveal alimenta la Cronaca, la Cronaca alimenta il Catalog (lineage), e le due strade di evoluzione (naturale/Splice) restano meccanicamente distinte come previsto senza sovrapporsi. Non ha rivelato incoerenze — ma è il tipo di verifica che va rifatta ogni volta che un pezzo importante cambia, non solo una volta.

## Fuori scope

- Valori numerici esatti di durata di stagione/era (qui impliciti, non specificati in tick reali) — restano da tarare come indicato nel documento sulla scala temporale.
- Rappresentazione a griglia ASCII come nel vecchio §16 — qui si descrive il flusso di gioco, non il rendering cella per cella.
