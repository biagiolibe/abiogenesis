# Abiogenesis — Notebook: Cronaca e arricchimenti minori

Documento autonomo, da leggere insieme a `abiogenesis-hud-notebook.md` (che resta la fonte per la struttura base del notebook: Observation log, Relationships, Species catalog, apertura del pannello). Qui solo le decisioni più recenti: una quarta sezione, Cronaca, e alcuni arricchimenti minori alle tre sezioni esistenti emersi da documenti successivi (archetipi biochimici, Emersione, wild species, eventi di mondo).

## Arricchimenti minori alle sezioni esistenti

### Relationships (grafo delle relazioni)

- **Nodi = tratti** (non più tag generici), con codice a 3 lettere invece delle lettere greche — cfr. documento archetipi biochimici.
- **Famiglia del tratto: resta nascosta nel grafo.** Coerente col principio "non rivelare mai il perché" già stabilito per il bias di famiglia dominante — il nodo mostra solo il codice a 3 lettere, mai un'etichetta di famiglia esplicita.
- **Xenotratti (se comparsi in quel mondo):** distinti visivamente dai tratti reali — nodo con **doppio contorno concentrico** invece del singolo bordo tratteggiato/continuo usato per i tratti reali, codice con prefisso `X-` (cfr. documento archetipi biochimici). Verificare che resti leggibile insieme alla codifica tratteggiato/continuo già in uso per confermata/ipotesi.

### Species catalog

Due campi aggiuntivi per card, oltre a quelli già definiti (nome, icona, metabolismo, range temperatura, popolazione, tratti, era di origine):

- **Origine — seminata, indigena o sintetizzata:** un piccolo tag testuale/icona distingue una specie del roster iniziale piazzata dal giocatore, una wild species trovata nel mondo (GDD §9), e una specie **sintetizzata in laboratorio dal giocatore tramite Splice** (cfr. documento sulle azioni — Splice crea una nuova specie, non modifica una esistente). Richiede un flag persistito alla creazione della specie, non derivabile in un secondo momento.
- **Discende da** (se applicabile): se la specie è nata per speciazione, il nome della specie genitore — campo semplice, un solo salto indietro nella lineage, non un albero completo. Per ricostruire una catena più lunga (utile in particolare per l'Emersione, cfr. documento dedicato), il riferimento è la Cronaca (sotto), che registra ogni salto nell'ordine in cui è avvenuto. Richiede che ogni specie nata per speciazione conservi un riferimento alla specie genitore (probabilmente già presente per il modello di speciazione, §5.11 GDD — verificare).

## Cronaca — quarta sezione del notebook

Sezione distinta dall'Observation log: mentre il log contiene dati scientifici grezzi (adiacenze pulite/confuse, materiale per inferenza), la Cronaca contiene la **storia narrata del mondo** — l'archivio di ogni reveal di fine era una volta chiuso (cfr. documento sulla scala temporale e documenti sugli eventi di mondo): eventi di soglia positiva, cascate di estinzione, transizioni di bioma, speciazioni, catastrofi.

### Come si popola

**Nessuna generazione di testo dedicata:** ogni voce è semplicemente il testo già prodotto dal sistema di generazione narrativa (cfr. documento dedicato) per il reveal corrispondente, persistito così com'era quando il giocatore lo ha visto — nessun doppio lavoro di generazione.

### Ere di quiete accorpate

Più ere consecutive senza eventi si comprimono in una singola riga discreta (es. "ere 6-9: quiete") invece di una riga ripetuta per ogni era — il ritmo della lista comunica l'assenza di eventi senza riempirsi di rumore.

### Ordinamento e gerarchia visiva

- **Più recente in alto**, organizzata per etichette d'era leggibili — resta leggibile come cronologia anche scorrendo verso il basso.
- **Peso visivo agganciato al tier del reveal** (minore/notevole/epocale, cfr. documento sulla scala temporale) — stessa intensità crescente già stabilita altrove (dimensione del marker o intensità del colore), nessun sistema nuovo da inventare.

### Collegamento con la lineage

Un evento di speciazione in Cronaca menziona sempre la specie genitore — coerente col campo "Discende da" nel Catalog (sopra), e primo passo verso poter ricostruire una lineage completa scorrendo la sezione, senza bisogno di una vista ad albero dedicata nella prima versione.

## Cosa serve per l'integrazione

- **Persistenza della Cronaca:** ogni reveal chiuso va archiviato automaticamente — richiede un log persistente per mondo dei testi già mostrati, con il tier associato per il peso visivo.
- **Compressione delle ere di quiete:** richiede rilevare sequenze consecutive senza eventi e presentarle come intervallo unico, non riga per riga.
- **Campo "Discende da" e Cronaca-con-genitore:** stesso dato richiesto in entrambi i punti, va implementato una sola volta e riusato.
- **Flag origine (seminata/indigena/sintetizzata):** persistito alla creazione della specie.
- **Distinzione visiva xenotratti nel grafo:** doppio contorno concentrico.

## Fuori scope

- Filtri per tipo di evento in Cronaca — da aggiungere solo se il playtest mostra che la lista diventa davvero ingombrante, non preventivamente.
- Click su una voce di Cronaca per saltare alla cella corrispondente sulla mappa — rifinitura futura.
- Vista ad albero completa della lineage nel Catalog — per la prima versione ci si affida alla Cronaca scorsa manualmente, non a una visualizzazione dedicata.
- Aggiornamento delle immagini mockup esistenti (`hud-full.svg`, `notebook-full.svg`) — non riflettono queste decisioni, restano riferimento per la struttura base già coperta da `abiogenesis-hud-notebook.md`.
