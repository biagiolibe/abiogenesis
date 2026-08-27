# Abiogenesis — azioni del giocatore: correzioni e discussione

Documento autonomo per un task di design/integrazione. Rivede le 3 azioni esistenti più delicate (Splice, Cull, Stress) alla luce della correzione di bilanciamento "la matrice deve essere necessaria" (cfr. documento dedicato), e valuta esplicitamente quali azioni nuove aggiungere o scartare.

## Principio di partenza

Le 4 azioni attuali (Seed/crea, Stress/perturba, Cull/distruggi, Splice/modifica) coprono concettualmente bene lo spazio "create/perturba/distruggi/modifica" — un set pulito. Il budget di 3 punti e il numero contenuto di leve dirette sono probabilmente la parte più preziosa del bilanciamento attuale ("un'unità di tempo = un esperimento deliberato"). **Ogni azione nuova deve superare una soglia alta di giustificazione**: deve abilitare una decisione che nessuna delle azioni esistenti permette, non essere una comodità o una variante di un'azione già presente.

## Ambito delle azioni: per cella, non per area

**Discrepanza rilevata e risolta.** Il GDD §6 (prosa risalente a v0.3) descrive Stress come "altera uno scalare ambientale **in un'area**" e Cull come "elimina un organismo **o una specie in un'area**". Ma due fonti più recenti dicono il contrario:
- GDD §11 (controlli): "**mouse cell selection**" per entrare in modalità azione.
- Il testo di onboarding della build attuale: *"Left click: perform the selected action **on a cell**."*

**La build è per-cella**, quindi §6 è prosa rimasta indietro rispetto all'implementazione, non una decisione viva in conflitto — caso previsto dal principio già stabilito nel documento di consolidamento ("vince lo stato reale del software, non il documento di design"). **Azione richiesta: correggere il GDD §6**, non i documenti di design.

**Sul merito, il per-cella è anche la scelta migliore per entrambe le azioni:**
- **Stress** per-cella, combinato con la diffusione ambientale (§5.2 Phase 1+), produce *naturalmente* un effetto ad area che si propaga nel tempo — più elegante di un'area istantanea decisa a priori, e conserva il doppio livello di conseguenza descritto sotto.
- **Cull** per-cella non è solo coerente, è **necessario per il design**: un Cull che elimina un'intera specie in un'area darebbe al giocatore una scorciatoia per rimuovere ciò che lo disturba invece di capirlo — bypasserebbe il loop centrale esattamente come faceva il problema di bilanciamento sulla riproduzione, e a 1 punto di costo sarebbe sproporzionato.

## Splice — sintesi di una nuova specie in laboratorio

**Chiarimento fondamentale sul funzionamento (come pensato e implementato):** Splice **non modifica una specie già presente sul mondo**. Crea una **nuova specie** con i tratti decisi dal giocatore, che va ad aggiungersi alla banca genomi (il roster seminabile) — e che poi va **piazzata sul mondo con una normale azione Seed**, come qualunque altra specie disponibile.

È concettualmente un **esperimento di laboratorio**: sintetizzi un ceppo a partire dalla biochimica che hai scoperto, poi lo impianti e osservi se regge. Non è un intervento a distanza su qualcosa di già vivo.

**Perché questa lettura è quella giusta, e cosa risolve:**

- **Elimina la questione del retroattivo.** Non c'è alcun cambiamento da propagare agli organismi già vivi — la domanda "gli esemplari esistenti cambiano o solo i nuovi nati?" semplicemente non si pone.
- **Rende il costo onesto.** Splice non è "riscrivi la tua specie migliore e vinci": è un esperimento a due tempi — spendi 2 punti per sintetizzare, poi **spendi un altro punto per seminare**, e devi comunque farla sopravvivere. A 2 punti su un budget di 3 il prezzo è difendibile, senza bisogno di alzarlo a 3 (§5.9 GDD lo lascia aperto come possibilità, ma con questa lettura non è necessario).
- **È coerente con l'architettura, non un'eccezione.** Crea una nuova voce in `world.species` esattamente come fa la speciazione (§5.11 GDD) — stesso meccanismo, stesso limite `max_species = 40`. Splice e speciazione diventano le **due strade verso una specie nuova**: una deliberata e costosa (laboratorio), una emergente e gratuita (pressione sostenuta). Simmetria pulita invece di due sistemi scollegati.
- **Rafforza la fantasia del gioco** (§2 GDD): progettare un ceppo a partire da ciò che hai decifrato e impiantarlo è molto più aderente al ruolo dello xenobiologo che non modificare da remoto qualcosa già vivo sulla mappa.

**Vincolo sui tratti utilizzabili:** Splice può assegnare **solo tratti che il giocatore ha già confermato nella matrice di quel mondo specifico** — non l'intero pool attivo. Con la lettura "laboratorio" questa non è più solo una regola di bilanciamento, è una **conseguenza logica della fantasia**: non puoi sintetizzare qualcosa che non hai ancora capito. Effetto sui due livelli di capacità già previsti (cfr. documento HUD/Notebook):
- **Livello iniziale ("solo tag lieve"):** più limitato/impreciso — poca conoscenza confermata, poca precisione di sintesi.
- **Livello sbloccato ("manuale completa"):** preciso, ma solo sul terreno già conquistato con l'osservazione.

Questo trasforma Splice in un **premio per aver decifrato qualcosa**: più capisci, più puoi costruire deliberatamente — una ragione in più per continuare a sperimentare, non un modo per smettere di doverlo fare.

**Vincolo permanente:** Splice **non include mai il pool xenotratti**, a nessun livello di sblocco. Risolve la contraddizione con il documento sugli archetipi biochimici (xenotratti "mai piazzabili dal giocatore, mai da scelta diretta") — restano strutturalmente irraggiungibili da Splice.

**Conseguenze su altre parti del gioco, da riportare nei rispettivi documenti:**
- **La banca genomi non è più un roster fisso** deciso dal world-gen: cresce durante la partita man mano che il giocatore sintetizza specie. L'HUD deve prevederlo, e probabilmente distinguere visivamente le specie originali da quelle sintetizzate.
- **Nel Catalog del notebook**, il campo "origine" acquisisce un terzo valore oltre a *seminata* e *indigena*: **sintetizzata**.

## Cull — corretto: knockout mirato, non sterminio

**Ambito, coerente col controllo esistente:** ogni azione nel gioco opera su **una singola cella** (GDD §11, "click sinistro: esegui l'azione selezionata su una cella") — non su una specie intera, non su un'area. Cull rimuove **l'organismo presente su quella cella**, mai l'intera popolazione della sua specie.

**Perché questo limite è necessario, non solo coerente:** se Cull potesse eliminare un'intera specie in un colpo, diventerebbe uno strumento di sterminio di massa invece di un esperimento mirato — e romperebbe l'incentivo a decifrare la matrice: il giocatore potrebbe eliminare una specie scomoda invece di capire perché dà fastidio, bypassando il loop centrale (lo stesso tipo di scorciatoia già affrontato nella correzione di bilanciamento).

**Cornice concettuale proposta — esperimento di rimozione (knockout):** rimuovere un organismo e osservare cosa cambia nei vicini è l'equivalente concettuale di un esperimento di knockout genico in biologia reale (rimuovi un elemento, osserva l'effetto della sua assenza). Cull diventa così la **terza modalità sperimentale** del gioco, accanto a "piazza e osserva" (Seed) e "stressa e osserva" (Stress): "rimuovi e osserva".

**Integrazione richiesta:** Cull dovrebbe generare esplicitamente un'osservazione tracciata dal notebook quando rimuove un organismo vicino ad altri — es. "Y si comporta diversamente dopo la rimozione di X" — con lo stesso sistema di peso già esistente (§7 GDD). Oggi questo collegamento probabilmente non esiste; è la correzione più importante da applicare, non il costo (che resta invariato a 1 punto — il budget limitato è già il freno naturale contro l'abuso).

**Non ridondante con "Isola" (proposta, sotto):** Isola previene la contaminazione prima che accada, Cull la rimuove retroattivamente per osservarne l'effetto — complementari, non doppioni.

## Stress — corretto: tre assi selezionabili, locale, temporaneo salvo uso ripetuto

**Problema individuato:** Stress oggi copre solo l'asse termico, senza che sia mai stata una scelta deliberata — luce e tossicità restano fuori dal controllo diretto del giocatore, nonostante contino quanto la temperatura per fotolitico e chemiolitotrofo rispettivamente. Un'asimmetria arbitraria, non voluta.

**Correzione proposta:** trasformare Stress in un'**unica azione con tre assi selezionabili** (termico / luminoso / tossicologico) — stesso slot nel roster, stessa icona, stesso costo di budget. Non aumenta il numero di azioni (resta a 4), non tocca l'economia "poche leve pesanti", elimina l'asimmetria e sposta la scelta *dentro* la decisione invece di aggiungere un'altra decisione da gestire. Si aggancia direttamente al nuovo obiettivo Tolleranza (cfr. documento obiettivi): con uno stress tossicologico deliberato, il giocatore può costruire attivamente lo scenario richiesto invece di sperare che il bioma giusto capiti.

**Ambito e durata:**
- **Locale, sulla singola cella selezionata** — coerente col controllo per-cella esistente.
- **Non permanente in un singolo uso:** l'applicazione sposta temporaneamente il valore ambientale scelto, che poi decade gradualmente verso il baseline nel corso di alcuni tick. Se esiste già diffusione ambientale passiva tra celle vicine (Phase 1+, §5.2), la perturbazione si propaga naturalmente ai vicini tramite quel sistema — nessun meccanismo nuovo necessario per la diffusione.
- **Permanente se applicato ripetutamente sulla stessa cella per più ere:** questo aggancia Stress direttamente al trigger dei biomi dinamici proposto nel documento sugli eventi di mondo ("condizione sostenuta oltre soglia per più ere consecutive"). Un giocatore che investe ere intere di Stress ripetuto sulla stessa cella può **forzare deliberatamente** una trasformazione di bioma (es. indurre un risveglio vulcanico invece di aspettare che accada da sé).

**Perché i due livelli di conseguenza sono il punto di forza, non un effetto collaterale da correggere:** un uso singolo resta tattico ed economico (spinge una cella fuori dalla comfort zone per innescare pressione evolutiva mirata, l'effetto svanisce da solo); un uso ripetuto e deliberato diventa strategico e costoso in budget nel tempo. La stessa leva serve sia per un esperimento di un'era sia, con investimento sostenuto, per riscrivere la geografia del mondo — va preservato in implementazione, non semplificato via.

## Azioni nuove valutate

### Isola / Quarantena — raccomandata

**Problema che risolve:** con la correzione di bilanciamento che rende le osservazioni pulite necessarie per decifrare la matrice, il giocatore non ha oggi alcuno strumento per **garantirsi** un'osservazione isolata — può solo sperare che un organismo resti isolato abbastanza a lungo prima che la riproduzione affolli la cella.

**Proposta:** protegge una cella dall'influenza di organismi vicini per un'era. Dà al giocatore controllo diretto sulla qualità dell'esperimento stesso, non solo sui suoi ingredienti — nessuna delle 4 azioni esistenti fa questo. Si aggancia direttamente al sistema di peso dell'osservazione già scritto nel notebook (un'osservazione isolata vale di più — ora il giocatore avrebbe uno strumento per renderla isolata, non solo sperare che lo sia).

**Stato:** raccomandata con convinzione, ma resta una proposta — introduce una quinta azione diretta, va soppesata contro il principio "poche leve pesanti" prima di essere adottata definitivamente.

### Sposta — da discutere ulteriormente, non raccomandata senza vincoli

Riposizionare un organismo già seminato invece di doverne piazzare uno nuovo. Utile per costruire un esperimento mirato senza sprecare un altro Seed, ma rischia di rendere troppo comodo "correggere" un piazzamento sbagliato, diluendo il peso della decisione iniziale. **Se introdotta, va vincolata a un costo alto** (proposta: 2 punti, come Splice) apposta per non renderla la scelta di default.

### Analizza / Campiona — scartata

Un'azione che rivelerebbe direttamente un dato nascosto (es. la pressione accumulata di un organismo). Scartata perché va contro il pilastro centrale del gioco: la deduzione dall'osservazione indiretta, non la lettura diretta dei numeri interni a comando. Sarebbe la cosa più distruttiva che si potrebbe aggiungere all'identità del gioco, anche se superficialmente sembra solo una comodità.

### Ibridazione diretta — scartata

Forzare manualmente un incrocio/speciazione tra due specie invece di aspettare la pressione naturale. Scartata perché contraddice lo stesso vincolo già scritto per gli xenotratti: l'evoluzione deve emergere dalla pressione accumulata, mai essere scelta direttamente dal giocatore. Permetterla romperebbe la regola che rende speciale anche la speciazione "normale", non solo gli xenotratti.

## Domanda esplicitamente lasciata aperta

Se la scarsità di Stress a un solo asse ambientale (prima della correzione qui proposta) fosse stata una scelta deliberata di design (poche leve = scelte più nette) o solo un buco non notato, non è determinabile da questo documento — la correzione a tre assi selezionabili risolve comunque l'asimmetria arbitraria in entrambi i casi, senza necessità di saperlo in anticipo.

## Cosa serve per l'integrazione

- **Correzione al GDD §6:** allineare la prosa ("in un'area") all'ambito reale per-cella già implementato e documentato in §11.
- **Splice:** crea una **nuova voce in `world.species`** (stesso meccanismo della speciazione, §5.11, stesso limite `max_species`) e la aggiunge alla banca genomi seminabile — non modifica specie esistenti. Filtrare il pool di tratti assegnabili in base a quali sono confermati per il giocatore in quel mondo — richiede accesso allo stesso dato di conferma già usato dal notebook. Escludere sempre il pool xenotratti a livello di codice, non solo di intenzione di design.
- **Banca genomi dinamica:** il roster seminabile deve poter crescere in corso di partita (specie sintetizzate via Splice), con distinzione visiva rispetto alle specie originali del world-gen — impatta l'HUD, oggi progettato assumendo un roster fisso.
- **Campo "origine" nel Catalog:** terzo valore *sintetizzata*, oltre a *seminata* e *indigena* (cfr. documento sulla Cronaca del notebook).
- **Cull:** collegare l'evento di rimozione al sistema di generazione di osservazioni (§7 GDD) — oggi probabilmente non collegato.
- **Stress:** implementare selezione del tipo di asse (termico/luminoso/tossicologico) nella UI dell'azione; implementare decadimento graduale post-applicazione; implementare accumulo verso la soglia di trasformazione bioma per applicazioni ripetute sulla stessa cella — questo terzo punto dipende dal meccanismo di biomi dinamici (cfr. documento eventi di mondo, non ancora implementato).
- **Isola (se adottata):** nuovo stato per cella ("protetta per N pulse/stagione") che sospende l'`interaction_delta` dei vicini su quella cella specifica.

## Fuori scope

- Valori numerici esatti (velocità di decadimento dello Stress, numero di applicazioni ripetute necessarie per superare la soglia di trasformazione bioma, costo esatto di un'eventuale azione Sposta) — da validare in playtest.
- Decisione finale sull'adozione di Isola e Sposta come quinta/sesta azione — qui solo raccomandate/discusse, non decise.
