# Abiogenesis — archetipi biochimici per i tratti

Documento autonomo per un task di integrazione. Sostituisce i glifi segnaposto (lettere greche) usati finora nei mockup del notebook con termini reali o quasi-reali di biochimica/microbiologia, organizzati in famiglie, con un sistema di codici a 3 lettere per il rendering.

## Principio di design (non negoziabile)

**Il nome di un tratto non deve mai suggerire se il suo effetto sarà positivo o negativo.** Ogni nome descrive cosa una cosa *è* o *fa strutturalmente* (una struttura, un processo, un comportamento), mai un giudizio sul suo impatto. La matrice di interazione resta generata in modo indipendente dal nome — un tratto chiamato "prione" non è più incline a effetti negativi di uno chiamato "pigmento".

**Perché i nomi reali non creano un pregiudizio problematico:** il giocatore può portarsi dietro associazioni dal mondo reale (es. "i prioni causano malattie"), ma il gioco stesso lo corregge strutturalmente — si gioca su mondi alieni, con chimiche diverse dalla Terra, e la matrice è casuale a ogni mondo. Un giocatore che assume che un tratto si comporti come nella realtà terrestre viene smentito dall'evidenza del mondo specifico, il che rinforza — non contraddice — il pilastro centrale del gioco: fidarsi dell'osservazione, non dell'assunzione. L'autenticità scientifica dei nomi resta un valore (coerente con l'ispirazione hard-SF), il "rischio" di pregiudizio diventa esso stesso materiale di gameplay.

## Sistema di codici (sostituisce i glifi greci)

Ogni tratto ha un **codice a 3 lettere maiuscole**, nello stile delle sigle reali di geni/proteine/composti (es. ATP, mRNA, p53) — coerente con l'estetica monospace/console già stabilita altrove, e più autenticamente "biochimico" di un glifo greco decorativo. Il codice è quello che compare nella matrice/hypothesis grid al posto di ε, β, κ ecc.

## Le famiglie

Le famiglie sono un **livello di lettura secondario e opzionale**, pensato per giocatori esperti dopo molti run (si aggancia alla proposta "grammatica nascosta tra i mondi" in un altro documento) — un'ipotesi che certi domini tendano statisticamente a comportarsi in un certo modo *nel proprio stile di generazione*, mai una regola dichiarata o affidabile su un singolo mondo. **La matrice resta indipendente dalla famiglia tanto quanto lo è dal nome**: non è un vincolo di design da implementare ora, è solo l'organizzazione con cui presentare la lista.

---

### Strutturale / membrana

L'interfaccia fisica dell'organismo con l'ambiente.

| Codice | Nome | Descrizione neutra | Stato |
|---|---|---|---|
| CHT | Parete chitinosa | Rivestimento rigido esterno | pool attivo |
| POR | Poro ionico | Canale selettivo nella membrana | pool attivo |
| LIP | Membrana lipidica | Doppio strato che delimita la cellula | pool attivo |
| CAP | Capsula mucosa | Rivestimento esterno gelatinoso protettivo | riserva futura |
| SPN | Spina proteica | Proiezione rigida sulla superficie cellulare | riserva futura |

### Metabolico / enzimatico

Come l'organismo processa energia e materia.

| Codice | Nome | Descrizione neutra | Stato |
|---|---|---|---|
| CHL | Chelasi | Enzima che lega ioni metallici | pool attivo |
| FRM | Fermentasi | Enzima per metabolismo in assenza di ossigeno | pool attivo |
| OSM | Osmoregolatore | Meccanismo di bilanciamento della pressione interna | pool attivo |
| CAT | Catalasi | Enzima che scompone composti reattivi | riserva futura |
| SNT | Sintetasi | Enzima che assembla molecole complesse | riserva futura |

### Segnalazione / comunicazione

Come l'organismo comunica con l'ambiente o con altri organismi.

| Codice | Nome | Descrizione neutra | Stato |
|---|---|---|---|
| QRM | Feromone di quorum | Segnale chimico legato alla densità di popolazione | pool attivo |
| FLG | Flagello chemiotattico | Appendice mobile sensibile a gradienti chimici | pool attivo |
| REC | Recettore di membrana | Struttura che rileva segnali esterni | pool attivo |
| BIO | Biofilm adesivo | Matrice che favorisce l'aggregazione tra organismi | riserva futura |
| PIG | Pigmento fotosensibile | Molecola che reagisce alla luce | riserva futura |

### Genetico / informazionale

La famiglia concettualmente più "misteriosa": informazione che si propaga o si trasforma.

| Codice | Nome | Descrizione neutra | Stato |
|---|---|---|---|
| PRN | Prione strutturale | Proteina che induce il proprio ripiegamento in altre proteine | pool attivo |
| PLM | Plasmide mobile | Frammento di materiale genetico trasferibile | pool attivo |
| RBZ | Ribozima catalitico | RNA capace di catalizzare reazioni chimiche | pool attivo |
| TRP | Trasposone | Sequenza genetica capace di spostarsi nel genoma | riserva futura |
| EPI | Epigenoma labile | Marcatore che regola l'espressione genica senza alterare la sequenza | riserva futura |

### Riserva / energia

Accumulo e conservazione di risorse.

| Codice | Nome | Descrizione neutra | Stato |
|---|---|---|---|
| CRS | Cristallo di riserva | Struttura minerale per l'accumulo di composti | pool attivo |
| SPO | Endospora dormiente | Forma di resistenza a metabolismo sospeso | pool attivo |
| VAC | Vacuolo lipidico | Compartimento per l'accumulo di lipidi | pool attivo |
| GRP | Granulo di polifosfato | Deposito interno di fosfato ad alta energia | riserva futura |
| FTS | Fotosoma * | Struttura ipotizzata per l'accumulo di energia luminosa | riserva futura |

\* Unico termine non strettamente reale della lista (le altre 24 voci sono tutte fenomeni o strutture biochimiche/microbiologiche realmente esistenti) — non a caso è stato mandato in riserva: se la priorità è autenticità biochimica al 100%, i 15 del pool attivo la rispettano già senza eccezioni.

---

## Copertura e scalabilità — pool a due livelli

**Correzione rispetto alla versione precedente di questo documento:** il GDD fissa il pool globale a **10 tratti** (§5.5), non 25. Con gli attivi-per-mondo fermi a 5→8 (leva di difficoltà del GDD, non toccata), un pool a 10 lascia margine troppo stretto: un mondo tardo (8 attivi) copre l'80% del pool, quasi tutto — la varietà tra mondi diversi si assottiglia proprio dove vorresti che il gioco si sentisse più ricco.

**Proposta:** pool a **15 tratti attivi di lancio** (3 per famiglia, marcati "pool attivo" nelle tabelle sopra), con i restanti **10 tenuti come riserva documentata per espansione futura** (marcati "riserva futura") — nessun lavoro curatoriale sprecato, solo non ancora in gioco. Con 15 nel pool, un mondo tardo (8 attivi) ne copre il 53% — resta impegnativo (8 tratti attivi restano 8 tratti attivi, la difficoltà della matrice T² non cambia) ma lascia margine reale perché due mondi tardi si sentano chimicamente distinti tra loro.

Nota importante: allargare il **pool** non tocca la difficoltà di una singola partita (che dipende solo dai tratti attivi, già bilanciati e non modificati qui) — tocca solo quanto due mondi diversi si assomigliano tra loro. Sono due assi indipendenti, e questa proposta interviene solo sul secondo.

## Cosa serve per l'integrazione

- **Sostituire i glifi greci** (ε, β, κ, α, ζ) con i codici a 3 lettere in tutti i punti dove compaiono: hypothesis grid/rete di relazioni, catalog delle specie, eventuali log/reveal.
- **Il nome/codice non deve mai comparire vicino a un'indicazione di segno** prima che sia stato osservato — coerente con le regole già stabilite per la hypothesis grid (nessun colore/segno finché non c'è evidenza).
- **Solo i 15 tratti marcati "pool attivo" entrano nella generazione dei mondi** al lancio; i 10 "riserva futura" restano documentati ma inattivi finché non si deciderà di espandere il pool.

## Bias di famiglia dominante per mondo (proposta, rivista)

Idea per dare a ogni mondo un'**identità chimica riconoscibile** senza rendere la selezione dei tratti né uniforme-anonima né deterministica.

**Correzione rispetto alla versione precedente:** un bias sulla *selezione* dei tratti attivi (peso maggiorato nell'estrazione) perde forza nei mondi tardi — con pool a 15 e 8 attivi, l'estrazione è già ampia, quindi il bias su *quali* tratti escono ha poco margine su cui agire quando conta di più. La soluzione è spostare il bias su un asse che resta significativo indipendentemente da quanti tratti sono attivi:

- Ogni mondo, alla generazione, riceve una **famiglia dominante** scelta pseudo-casualmente dal seed (una delle 5) — invariato.
- **Il bias non agisce più sulla probabilità di selezione dei tratti**, ma sulla **distribuzione delle intensità nella matrice** (§5.9 GDD: interi in `{−2,−1,0,+1,+2}`) per le relazioni che coinvolgono un tratto della famiglia dominante — pesate verso gli estremi (`±2` più probabile di `±1`) rispetto alla distribuzione normale usata per le altre famiglie.
- **Perché funziona meglio:** non dipende da quanti tratti della famiglia sono stati effettivamente pescati — agisce *dopo* la selezione, sulla matrice generata, quindi resta ugualmente percepibile a 5 tratti attivi come a 8. Un mondo a dominanza genetica "si sente" chimicamente diverso perché le sue relazioni genetiche tendono a essere più nette/violente, non perché il giocatore ha visto più o meno tratti di quella famiglia — coerente anche col flavor testuale già scelto per quella famiglia (instabilità, propagazione).
- **La famiglia dominante non viene dichiarata esplicitamente al giocatore.** Si scopre giocando, osservando che le relazioni di certi tratti tendono a essere più estreme in quel mondo — coerente con "il mistero deve trainare".
- **Il bias riguarda solo l'intensità della matrice, mai il segno** — un mondo a dominanza genetica non è "più incline al negativo", solo a effetti più netti in entrambe le direzioni. Il principio di design non negoziabile enunciato in apertura resta intatto.
- Idea per il futuro, non necessaria ora: la famiglia dominante potrebbe in seguito legarsi anche alla generazione ambientale (es. un mondo a dominanza "riserva/energia" tende a biomi più aridi/poveri di risorse) — solo un'ipotesi di direzione, non specificata qui.

## Tratti attivi per mondo — proposta di revisione (da validare, non equiparabile alla correzione del pool)

**Attenzione:** a differenza della correzione sul pool (che non tocca il bilanciamento di una singola run, già validato in playtest), questa sezione propone di **rivedere l'unica vera manopola di difficoltà tarata del gioco** (5→8 tratti attivi, GDD §5.5/§9). Va trattata con più cautela — è un'ipotesi da validare in playtest, non una correzione equivalente a quella del pool.

- **World 0 più morbido (proposta: 4 tratti attivi invece di 5).** Coerente con l'ammorbidimento già applicato all'obiettivo del primo mondo (§9, `Coexistence` con `min_species = 2`): con 4 tratti attivi la matrice del primissimo mondo ha ~12 relazioni possibili (T²) invece di ~20, un ingresso ancora più graduale nei primi minuti di gioco. Dal mondo 1 in poi si torna a 5 come oggi.
- **Tetto superiore leggermente più alto (proposta: 9 invece di 8) per i mondi più tardi**, ora che il pool a 15 lo rende sostenibile senza esaurirlo. Il costo cresce quadratico (T²): 8 attivi → 56 relazioni, 9 → 72 (+29%). Il salto a 9 resta ragionevole; un salto a 10 (+61% rispetto a 8) è stato scartato come troppo ripido per un beneficio percepito marginale, dato che il giocatore comunque decodifica solo la parte rilevante della matrice, non l'intera cosa.
- **Curva più graduale sui tier intermedi**, invece di un salto diretto da 5 a 8/9: proposta indicativa `4 (world 0) → 5 → 6 → 7 → 8 → 9` invece dei due soli gradini attuali. Coerente col principio, già scritto nel documento di consolidamento dei sistemi, che ogni parametro di difficoltà dovrebbe scalare in modo continuo insieme agli altri assi (tag condizionati dal terreno, ostilità ambientale, budget ere) invece di restare l'unico a due soli valori.

## Estensione futura: tratti che non esistono nella chimica reale, sbloccati solo dall'evoluzione

Idea da tenere in roadmap, non da implementare ora: oltre ai 15 tratti reali del pool attivo (più i 10 di riserva futura) censiti sopra — tutti basati su biochimica reale o quasi-reale — esiste la possibilità di un secondo insieme — i **xenotratti** — con una regola strutturale diversa:

- Un xenotratto **non esiste nella chimica che conosciamo**: non è un'estrapolazione plausibile come i 25 tratti sopra, è deliberatamente qualcosa che la chimica terrestre non può spiegare.
- Un xenotratto **non è mai piazzabile dal giocatore**: non compare mai su un organismo seminato. Compare **esclusivamente** come risultato di un'evoluzione maturata in un organismo già presente nel mondo (si aggancia al documento sulla scala temporale a tre livelli: l'evoluzione applicata al reveal di fine era potrebbe, in casi rari, introdurre un xenotratto invece di modificare un tratto esistente).
- Un xenotratto **fa comunque parte della stessa matrice tratto×tratto**: ha le sue relazioni nascoste con gli altri tratti (reali o xeno) da scoprire con lo stesso sistema di osservazione/evidenza già in uso — non è un sistema separato, è un'estensione dello stesso.
- Conseguenza naturale e voluta: in un mondo dove l'evoluzione non matura mai, il giocatore potrebbe non incontrare **nessun** xenotratto in tutta la partita — la loro rarità stessa diventa parte del mistero, non un difetto da correggere.

### Alcuni esempi illustrativi (solo brainstorm, non formalizzati come la lista principale)

A differenza dei tratti reali sopra, questi vanno pensati come concetti impossibili — nessun ancoraggio alla biochimica reale, per marcare chiaramente "questo non poteva esistere prima che il mondo lo facesse evolvere":

| Nome provvisorio | Idea |
|---|---|
| Sincizio radiante | Organismi che condividono energia fondendosi temporaneamente a livello cellulare |
| Cronofago | Struttura che altera la percezione locale del proprio metabolismo nel tempo |
| Eco genomica | Capacità di conservare la memoria di una pressione ambientale passata, anche cessata |
| Simmetria negativa | Una struttura osservabile solo per l'effetto che produce, mai direttamente |
| Nucleo condiviso | Più organismi che fondono temporaneamente il proprio materiale genetico |

Se in futuro si formalizza questa lista, andrebbe mantenuta una distinzione visiva netta rispetto ai tratti reali — ad esempio un codice con prefisso dedicato (es. `X-QRN`) o uno stile di bordo diverso nella hypothesis grid — così il giocatore riconosce a colpo d'occhio "questo non l'ho seminato io, è emerso".

## Fuori scope

- Bias di generazione per dare identità chimica ai mondi — qui rivisto per agire sull'intensità della matrice invece che sulla selezione, ma il peso esatto della distribuzione verso gli estremi resta da validare in playtest.
- Xenotratti — solo direzione concettuale e brainstorm illustrativo, non una lista formalizzata né una specifica pronta per l'implementazione.
- Tratti attivi per mondo (4/9/curva graduale) — proposta esplicitamente da validare in playtest, non una correzione equivalente a quella del pool: tocca l'unica manopola di difficoltà già tarata del gioco.
- Eventuale promozione dei 10 tratti in riserva futura al pool attivo, o ulteriore espansione oltre quei 25 totali.
- Verifica finale se "Fotosoma" va sostituito con un termine più strettamente reale, per chi vuole mantenere il vincolo "100% autentico" senza eccezioni tra i tratti reali.
