# Abiogenesis — generazione dinamica del testo narrativo

Documento autonomo per un task di integrazione. Descrive come generare in modo procedurale (non scriptato a mano, non tramite modello linguistico a runtime) il testo che comunica al giocatore eventi che dipendono da più fattori — il reveal di fine era, la frase di chiusura di un mondo, ed eventuali altri momenti narrativi dinamici del gioco.

## Contesto

Diversi punti del gioco richiedono di comunicare al giocatore, in prosa breve, un evento che nasce dalla combinazione di più cause (un delta di popolazione, una relazione della matrice confermata, uno shock ambientale, un'evoluzione maturata). Generare questo testo "a mano" per ogni combinazione non è praticabile (esplosione combinatoria); generarlo con logica ingenua (mad-libs con slot fissi, o un riassunto che prova a infilare tutte le cause in una frase) produce testo riconoscibile come artificiale o, peggio, un readout statistico travestito da narrazione.

## Principio guida: non costruire un sistema causale nuovo, riusare quello del notebook

Il gioco calcola già un peso per ogni osservazione (`1/(1+confondenti)`, §7 GDD) per distinguere un'osservazione "pulita" da una "confusa" nel sistema di ipotesi. **Questo è lo stesso segnale da usare per decidere quale causa, tra quelle di un'era, merita di diventare la frase generata.** Non va creato un sistema di attribuzione causale separato per la narrazione — il rischio, altrimenti, è che i due sistemi (notebook e narrazione) raccontino versioni incoerenti dello stesso evento.

## Architettura proposta

### 1. Ranking degli eventi candidati, non riassunto cumulativo

Per ogni finestra temporale che genera testo (fine era, chiusura di un mondo), si raccolgono gli eventi candidati (delta di popolazione significativi, relazioni confermate nell'era, shock ambientali, evoluzioni maturate — cfr. documento sulla scala temporale a tre livelli). Ogni candidato riceve un **punteggio di rilevanza**: grandezza dell'effetto moltiplicata per la pulizia del segnale causale (lo stesso peso già usato per le osservazioni del notebook).

- Il candidato con punteggio più alto diventa la clausola principale del testo.
- Un secondo candidato, se abbastanza rilevante, può diventare una clausola subordinata.
- Gli altri candidati **non entrano nel testo** — restano visibili altrove nell'interfaccia (Biosphere, log), non vanno ripetuti in prosa. Il testo generato non deve provare a essere un riassunto completo dell'era.

### 2. Grammatica a frammenti componibili

Invece di frasi intere pre-scritte per ogni combinazione possibile, il testo si costruisce da **frammenti piccoli e riusabili**, ciascuno con alcune varianti lessicali:

- **Frammento soggetto** — riferimento alla specie o al tratto coinvolto.
- **Frammento causa** — descrizione breve di cosa ha innescato l'evento (es. prossimità isolata a un tratto, esposizione a uno shock ambientale).
- **Frammento effetto** — cosa è cambiato (delta di popolazione, tratto confermato, evoluzione applicata).

Ogni frammento ha 3-4 varianti scelte in modo **pseudo-casuale ma seedato** (deterministico dato lo stesso seed di mondo/era, coerente con la riproducibilità già richiesta al resto del gioco, §5.7 GDD) — non generazione realmente casuale a ogni esecuzione. Con poche decine di varianti per categoria si ottengono centinaia di combinazioni percepite come diverse, senza dover scrivere ogni frase a mano né incorrere nell'esplosione combinatoria di frasi intere pre-scritte.

### 3. L'incertezza è un output legittimo

Quando il ranking non produce una causa chiaramente dominante (più fattori con punteggio simile), il sistema **non deve inventare una causa netta**. In quel caso il testo generato comunica esplicitamente l'ambiguità (es. "più fattori sembrano essersi sovrapposti in quest'era, nessuno isolabile con certezza") invece di forzare un'attribuzione causale che il notebook stesso non potrebbe confermare. Questo non è un caso limite da nascondere: è coerente con il tono epistemico che il gioco insegna fin dal main menu ("controlla il log prima di sospettare la matrice") — rinforza il tema invece di romperlo.

### 4. Registro clinico, non prosa letteraria

Frasi brevi, quasi da log di laboratorio (es. "κ si è indebolito dopo l'esposizione a ζ isolato"), non periodi elaborati. Un registro secco e osservazionale nasconde meglio le giunture della generazione procedurale rispetto a una prosa che prova a essere "scritta bene" — ed è anche coerente con il tono console/laboratorio già stabilito in tutta l'interfaccia (HUD, notebook, main menu), quindi non è solo un accorgimento tecnico, è coerenza stilistica.

### 5. Coerenza obbligatoria con i dati visibili altrove

Il testo generato non deve mai introdurre colore narrativo scollegato dai numeri reali mostrati nell'interfaccia (conteggio popolazione, delta, nome del tratto coinvolto). Un'incoerenza tra quello che dice la frase e quello che mostra la riga Biosphere o il notebook è l'errore più dannoso per la credibilità del sistema — più dannoso di qualunque limite nella varietà lessicale dei frammenti. Ogni frammento usato nel testo deve poter essere tracciato a un dato reale del gioco in quel momento.

## Collegamento con il reveal di fine era

Questa architettura è pensata per alimentare direttamente il beat di reveal descritto nel documento sulla scala temporale a tre livelli (Pulse → Stagione → Era):

- Il **ranking degli eventi candidati** di un'era determina anche il **livello di rilevanza** del reveal (minore / notevole / epocale) proposto in quel documento — sono lo stesso calcolo, non due sistemi paralleli da tenere sincronizzati manualmente.
- Se un'evoluzione matura durante l'era (cfr. stesso documento), il frammento-effetto della sua applicazione entra nel ranking come qualunque altro candidato, con lo stesso criterio di punteggio.
- La frase di chiusura di un intero mondo (proposta "il momento della rivelazione", in un altro documento) usa la stessa architettura su una finestra temporale più ampia (l'intero mondo invece della singola era) — stesso ranking, stessa grammatica a frammenti, stesso principio di coerenza con i dati.

## Alternativa scartata: generazione via modello linguistico a runtime

Una chiamata a un modello linguistico a runtime darebbe più varietà "gratis", ma è stata scartata come fondamenta per due motivi concreti: rischia di rompere la riproducibilità via seed già valorizzata dal gioco (§5.7 GDD), a meno di seedare anche quella generazione; e introduce una dipendenza esterna e latenza in un gioco altrimenti pensato per girare offline. Resta possibile, se mai, come **strato cosmetico opzionale sopra** l'architettura a ranking + grammatica a frammenti descritta qui — non come sostituto.

## Cosa serve per l'integrazione

- **Definire il calcolo di punteggio evento** (grandezza × pulizia del segnale) in un unico posto condiviso tra notebook e narrazione, non duplicato.
- **Costruire il pool di frammenti** (soggetto/causa/effetto, 3-4 varianti ciascuno) per le categorie di evento previste: delta di popolazione, relazione confermata, shock ambientale, evoluzione maturata. Il pool iniziale può essere piccolo — l'architettura è pensata per crescere per aggiunta di frammenti, non per riscrittura.
- **Verificare il seeding della scelta pseudo-casuale dei frammenti** — deve dipendere da seed di mondo + numero di era (o pulse), non da uno stato globale non riproducibile, altrimenti la stessa era rigenerata (es. per debug) produrrebbe testo diverso.
- **Collegare il livello di rilevanza del reveal** (minore/notevole/epocale, cfr. documento sulla scala temporale) allo stesso punteggio di ranking descritto qui.

## Fuori scope

- Il pool completo di frammenti testuali (soggetto/causa/effetto con le loro varianti) — qui è definita la struttura, non il contenuto testuale completo.
- Soglie numeriche esatte per "quanto un punteggio deve essere alto per diventare la clausola principale" o "quanto vicini due punteggi devono essere per attivare il caso di ambiguità" — da bilanciare in playtest.
- Il livello cosmetico opzionale basato su modello linguistico — solo menzionato come possibilità futura, non specificato.
