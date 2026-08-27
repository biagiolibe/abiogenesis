# Abiogenesis — gerarchia dei sistemi e piano di consolidamento

Documento autonomo. Non introduce meccaniche nuove: classifica tutto quello che è già stato deciso nel GDD (v0.6) o proposto nei documenti precedenti in tre livelli, e per ciascuna voce indica **cosa va fatto concretamente** per garantire che il sistema regga quando convive con tutti gli altri.

## Il problema che questo documento risolve

Con tutte le meccaniche accumulate — core loop, biomi, evoluzione, tag condizionali, reveal, eventi epocali, xenotratti, Emersione — il rischio non è "manca qualcosa", è l'opposto: troppi sistemi che girano in parallelo senza gerarchia possono far scivolare il gioco da "qualcosa che si gioca" a "qualcosa che si osserva". La difesa è duplice:

1. **Gerarchia esplicita a tre livelli** (sotto), invece di un elenco piatto di feature.
2. **Tre principi trasversali**, applicati a ogni sistema, vecchio o nuovo:
   - **Agency preservata:** ogni sistema che gira autonomamente deve restare riconducibile a una scelta passata del giocatore (dove ha seminato, stressato, mutato) — mai puramente casuale e scollegato.
   - **Manopola di difficoltà singola:** ogni nuovo parametro di intensità/frequenza si aggancia al tier di ostilità del mondo già esistente (tag attivi, ambiente, budget ere), non introduce una propria scala indipendente.
   - **Meraviglia e scoperta (pilastro 5 del GDD):** ogni proposta, piccola o grande, va misurata anche su questo filtro — dà al giocatore un motivo in più per pensare "voglio vedere cos'altro c'è", o è solo funzionale? Formalizzato con proposte concrete in `culture-shock-wonder.md`.

---

## Aggiornamento — discrepanze risolte

Passaggio di revisione contro il GDD v0.6. Esiti:

- **Ambito delle azioni:** GDD §6 dice "in un'area", ma §11 e la build reale dicono per-cella. **Vince la build** — §6 è prosa rimasta indietro, da correggere nel GDD. I documenti di design erano corretti.
- **Splice:** non modifica una specie esistente — **crea una nuova specie** con i tratti scelti, che va poi seminata come qualunque altra (esperimento di laboratorio). Risolve la questione del retroattivo, rende onesto il costo, e crea simmetria con la speciazione (le due strade verso una specie nuova: deliberata e costosa, oppure emergente e gratuita).
- **Numeri di bilanciamento:** ricalcolati sulla formula reale (`metabolism_gain` è un moltiplicatore, non il guadagno finale). Restano provvisori finché non si fissa la durata dell'era.
- **Scala temporale:** la **stagione diventa l'unità di decisione** (budget di azioni per stagione), l'era l'unità di racconto. Il budget di ere per mondo va abbassato drasticamente per compensare ere più lunghe.
- **Realtime:** il realtime "vero" (tempo che scorre mentre si pianifica) è **scartato definitivamente**. L'avanzamento continuo fermabile è invece adottato come parte necessaria della struttura.
- **Vittoria come flag:** deciso — completare gli obiettivi non termina il mondo, il giocatore sceglie quando passare oltre. Sblocca l'Emersione, che altrimenti era irraggiungibile.
- **Tempo a notebook aperto:** in pausa, coerente con lo scarto del realtime vero.
- **`toxic_zone` fuori dall'enum biomi:** già pianificato come task di unificazione — risolto, nessuna azione di design necessaria.

I punti **non ancora affrontati** (audio, salvataggio, transizione tra mondi, fine run, meta-progressione concreta, accessibilità cromatica, performance, lingua, impostazioni) e la questione aperta sulla **revisione della formula del tick** sono tracciati in `abiogenesis-open-points.md`.

---

## Livello 1 — Core (sempre presente, ogni run)

### Loop centrale (seed → era → osserva → ipotizza)
**Stato:** deciso, GDD §3. **Criticità emersa, priorità alta:** con i valori baseline noti (§5.9), un organismo piazzato leggendo solo dati ambientali visibili raggiunge la soglia di riproduzione senza che la matrice nascosta debba mai intervenire — il layer di mistero è oggi opzionale per il successo meccanico di base, non necessario. Approfondito in documento dedicato (`abiogenesis-matrix-necessity-balance.md`), con una prima ipotesi numerica (da validare contro le formule reali) per rendere l'adattamento ambientale da solo un quasi-pareggio, lasciando alla matrice la decisione tra crescita e declino.
**Azione concreta:** nessuna nuova meccanica deve introdurre un modo di avanzare che scavalchi la fase di osservazione — in particolare, l'avanzamento automatico proposto nell'HUD deve restare **sempre pausabile** e non deve mai incatenare più ere senza dare al giocatore un punto di lettura, altrimenti il loop stesso si eroderebbe. **In aggiunta:** questa criticità sul bilanciamento andrebbe verificata e risolta prima di molte altre rifiniture proposte in questo documento — se il loop centrale non rende la matrice necessaria, buona parte del valore di sistemi come il bias di famiglia, i biomi, o gli obiettivi (che presuppongono tutti un giocatore che *deve* sperimentare) si riduce.

### Matrice tratto×tratto
**Stato:** deciso, GDD §5.5 — **il documento sugli archetipi biochimici è stato aggiornato**: pool a due livelli (15 tratti attivi di lancio, 3 per famiglia + 10 in riserva futura documentata, invece dei 25 originali), bias di famiglia dominante spostato dalla selezione dei tratti all'intensità delle relazioni nella matrice (più efficace a qualunque numero di tratti attivi), e una proposta — esplicitamente da validare, non equiparabile alla correzione del pool — di rivedere la curva di tratti attivi per mondo (4 in world 0, tetto a 9 nei mondi tardi, progressione più graduale sui tier intermedi).
**Azione concreta:** nessuna aggiuntiva qui — il documento dedicato è già coerente. Da tenere presente: la revisione dei tratti attivi per mondo tocca l'unica manopola di difficoltà già tarata in playtest (§9), va trattata con la cautela che il documento stesso raccomanda, non implementata come se fosse equivalente alla correzione del pool.

### Metabolismo e ambiente (4 metabolismi)
**Stato:** deciso, GDD §5.4 — Fotolitico, Predatore, Decompositore, Chemiolitotrofo tutti attivi.
**Azione concreta:** il set di icone specie già li copre tutti e 4 come "attivi" — nessuna azione lì. Il main menu onboarding va aggiornato per menzionare il chemiolitotrofo nel bullet dei metabolismi (già segnalato in altra sede, non ancora applicato su richiesta tua).

### Notebook (log, grafo relazioni, catalog)
**Stato:** deciso nella struttura (GDD §7), arricchito nei documenti di redesign (grafo a nodi, catalog dettagliato per card).
**Azione concreta:** verificare che il grafo a nodi resti leggibile fino a 8 tratti attivi (caso limite tardo gioco, GDD §5.5) — con molti archi il layout circolare fisso proposto rischia di affollarsi. Se satura, implementare la vista "focus su un tratto" (già menzionata come estensione non sviluppata) prima che diventi un problema di leggibilità reale, non dopo.

### Obiettivi in sequenza + Speciazione finale
**Stato:** deciso nella struttura (GDD §8) — **approfondito in documento dedicato** (`abiogenesis-objectives.md`): regola generale "snapshot all'attivazione" per evitare completamenti per coincidenza, caso speciale per Speciazione con bersaglio specifico quando una speciazione è già avvenuta, 5 nuovi tipi di obiettivo proposti (Omeostasi, Tolleranza, Convivenza selvatica, Radicamento, Prima conferma), generazione dei parametri ancorata a dati reali del mondo. Resta **aperta** la questione se la vittoria debba essere un flag non bloccante invece di terminare forzatamente il mondo — impatta direttamente la fattibilità dell'Emersione (sotto), da confermare con chi implementa.
**Azione concreta:** nessuna nuova meccanica di fine-mondo (in particolare Emersione, tier 3) deve sostituire questa sequenza — resta sempre disponibile come percorso standard, l'Emersione è un'alternativa rara che si affianca, non un rimpiazzo. Vedi documento dedicato per il resto.

### Azioni core (Seed, Stress, Cull, Splice)
**Stato:** decise nella struttura (GDD §6/§11), **approfondite in documento dedicato** (`abiogenesis-actions.md`): Splice limitato ai tratti già confermati dal giocatore (mai xenotratti, vincolo permanente), Cull ridefinito come esperimento di rimozione su singolo organismo agganciato al sistema di osservazione, Stress esteso a tre assi selezionabili (termico/luminoso/tossicologico) con doppio effetto — temporaneo su uso singolo, permanente se ripetuto sulla stessa cella (aggancio diretto al trigger dei biomi dinamici). Isola/Quarantena raccomandata come possibile quinta azione, Sposta discussa ma non raccomandata senza costo alto, Analizza e Ibridazione diretta scartate per principio (violerebbero rispettivamente la deduzione indiretta e la regola "l'evoluzione non si sceglie a comando").
**Azione concreta:** verificare, in fase di implementazione, se Cull genera già un'osservazione tracciata quando rimuove un organismo vicino ad altri — è la correzione più importante del documento e la più facile da dimenticare perché non cambia il costo né l'interfaccia dell'azione, solo il suo effetto sui dati.

### HUD
**Stato:** documento di redesign proprio.
**Azione concreta:** ogni nuova informazione prodotta da un sistema di tier 2/3 (bias di famiglia, eventi epocali, evoluzione) **non deve** guadagnarsi un pannello sempre visibile nell'HUD. O passa dal notebook (dove il giocatore va a cercarla attivamente), o resta implicita/scopribile — altrimenti l'HUD torna ad affollarsi come nella prima versione che avevamo scartato per "troppo sparso".

### Main menu / onboarding
**Stato:** documento proprio, da aggiornare su due punti minori (chemiolitotrofo, Speciazione finale) — non urgente, rimandato su tua richiesta.
**Azione concreta:** nessuna finché non deciderai di aggiornarlo.

---

## Livello 2 — Variazione strutturale (scala con l'ostilità del mondo)

### Biomi
**Stato:** deciso, sia nel GDD (§5.10, fasce di elevazione + `Forest`/`Swamp`/`Crater`/`CrystalField`/`Lake`) sia nel documento biomi proposto (16 in totale) — **risolto**: esiste già un task che introduce la proposta a 16 biomi, quindi vale quella come soluzione applicata. Non è più un punto aperto.
**Azione concreta:** nessuna — in caso di qualunque futuro disallineamento tra documento e implementazione, vince lo stato reale del software/dei task, non il documento di design. Questo vale come principio generale anche per le altre voci di questo documento, non solo per i biomi.

### Tratti condizionati dal terreno
**Stato:** deciso, GDD §5.5 (task 096) — 1-2 per mondo.
**Azione concreta:** nessuna — è già ben vincolato in numero, non rischia di esplodere in complessità.

### Wild species (specie indigene)
**Stato:** deciso, GDD §9 (task 098).
**Azione concreta:** il GDD non specifica se la quantità/rarità di wild species scala col tier di mondo. Proposta da validare: più il mondo è avanzato, più wild species compaiono (segnale indiretto di un ecosistema preesistente più ricco) — aggancia questo sistema alla stessa manopola di difficoltà invece di lasciarlo a un valore fisso.

### Evoluzione per speciazione
**Stato:** deciso, GDD §5.11 — meccanismo a 3 stimoli pesati (danno da interazione, disallineamento ambientale, tossicità), soglia cumulativa crea una **nuova specie**, non modifica quella esistente.
**Azione concreta:** nessuna sul sistema in sé, è già ben progettato. Il documento Emersione è stato aggiornato per costruire il proprio trigger su questo meccanismo reale (catena di lineage, non accumulo su una singola specie) — vedi voce dedicata al livello 3.

### Scala temporale Pulse → Stagione → Era
**Stato:** proposta mia, non ancora nel GDD.
**Azione concreta:** se si procede, la durata di stagione/era dovrebbe anch'essa scalare col tier di mondo (mondi avanzati = ere più lunghe = più tempo perché l'evoluzione maturi), invece di restare un valore fisso globale — coerente col principio "un mondo tardo è più ostile *e* più profondo su più assi contemporaneamente", non solo su uno.

### Generazione narrativa dinamica
**Stato:** proposta mia — infrastruttura di supporto al reveal, non un contenuto in sé.
**Azione concreta:** il pool di frammenti testuali va costruito **prima** per le categorie di evento comuni (delta di popolazione, relazione confermata — tier 1/2) e solo dopo esteso a eventi rari (catastrofi, Emersione — tier 3). Costruire prima il caso raro lascerebbe vuoto il caso comune, che è quello che il giocatore vede più spesso.

### Sorgenti di calore puntiformi (bocca vulcanica/geyser)
**Stato:** proposta mia, non decisa nel GDD — ma i biomi Bocca vulcanica/Geyser sono ora risolti come adottati (sopra), quindi la dipendenza che bloccava questa voce è sciolta.
**Azione concreta:** può essere ripresa quando si vorrà specificarla nel dettaglio — non più bloccata da un'ambiguità sui biomi.

### Principio "manopola singola" — audit esplicito
**Azione concreta, trasversale a tutto questo livello:** fare (in un secondo momento, quando si passa all'implementazione) una tabella esplicita di quali parametri scalano già col tier di mondo (tratti attivi, ostilità ambientale, budget ere — tutti decisi) e quali nuovi parametri proposti (frequenza eventi epocali, soglia di speciazione, rarità wild species, densità di biomi estremi) andrebbero agganciati alla stessa variabile invece di introdurne una propria scollegata. Non farlo è la causa più probabile di un gioco che diventa "facile all'inizio, impossibile a caso più avanti" invece che gradualmente più difficile.

---

## Livello 3 — Payoff rari (alta cerimonia, non garantiti ogni run)

### Precursore
**Stato:** proposta mia.
**Azione concreta:** dato che il Cratere è già un bioma deciso, l'implementazione più economica è una proprietà speciale su **una singola cella Crater per mondo** (non tutte) — un flag booleano sul bioma esistente, non un nuovo sistema da costruire da zero.

### Eventi epocali
**Stato:** proposta mia, non decisa.
**Azione concreta:** la frequenza deve scalare col tier di mondo (principio manopola singola) — proposta indicativa: probabilità di un evento epocale per era ≈ proporzionale al tier del mondo rispetto al tier massimo, moltiplicata per una costante piccola. Mai una probabilità fissa uguale in ogni mondo.

### Momento della rivelazione (frase di chiusura mondo)
**Stato:** proposta mia, si appoggia al sistema di generazione narrativa (già in tier 2 come infrastruttura).
**Azione concreta:** nessuna aggiuntiva — riusa l'infrastruttura già pianificata al livello 2, non richiede un sistema separato.

### Xenotratti
**Stato:** proposta mia, non decisa.
**Azione concreta:** la loro comparsa deve restare condizionata **esclusivamente** dalla maturazione di un'evoluzione — mai piazzabili dal giocatore, mai generati come evento indipendente. Questo vincolo è già scritto nel documento originale: l'azione concreta qui è **preservarlo in implementazione**, resistendo alla tentazione di "sbloccarli di più" per dare più contenuto percepito — la loro rarità è il punto, non un limite da smussare.

### Emersione
**Stato:** proposta mia — trigger risolto (copertura di famiglie lungo la lineage, xenotratti con origine meccanica, catastrofe come acceleratore opzionale). **Dipendenza nuova, non ancora risolta:** l'Emersione richiede una catena di almeno 3 speciazioni imparentate, che per costruzione arriva sempre dopo la singola speciazione che soddisferebbe l'obiettivo finale — se completare tutti gli obiettivi termina forzatamente il mondo (comportamento attuale presunto), l'Emersione non avrebbe mai la possibilità di maturare. Vedi `abiogenesis-objectives.md`, nota "vittoria come flag, non fine forzata".
**Azione concreta:** la stessa decisione aperta segnalata nel documento obiettivi va presa prima di considerare l'Emersione implementabile così com'è — non risolvibile qui, dipende dallo stato reale del modello di fine-mondo nell'implementazione.

### Galleria dei mondi / Codex
**Stato:** proposta mia, esplicitamente rimandata — tocca la decisione ancora aperta tra scope "colony builder" e scope "run brevi tipo Plague Inc".
**Azione concreta:** nessuna implementazione finché quella decisione di scope non è presa. Costruirla prima rischia lo scope enorme già discusso in altra sede.

### Eventi di mondo (catastrofici, non-catastrofici, biomi dinamici)
**Stato:** proposta mia, non decisa — due documenti dedicati (`abiogenesis-world-events.md`, eventi concreti/ambiziosi con priorità; `abiogenesis-world-events-catastrophes.md`, catastrofi/non-catastrofi e meccanismo dei biomi dinamici).
**Azione concreta:** gli eventi "concreti" (soglia positiva, cascata di estinzione, finestra di colonizzazione, deriva genetica, risveglio wild species) sono livello 2 — riusano dati già calcolati, frequenza da agganciare al tier di mondo come da principio "manopola singola". Gli eventi "ambiziosi" e le catastrofi/biomi dinamici sono più vicini al livello 3 per costo e per l'effetto di rarità che richiedono (in particolare "Eco di segnale inspiegabile", esplicitamente limitato a una volta per mondo). I **biomi dinamici** (impatto, risveglio vulcanico, sisma, cristallizzazione) meritano priorità di approfondimento più alta delle altre proposte ambiziose per l'impatto trasversale che hanno su registro stratigrafico, reveal, e tratti condizionati dal terreno — segnalato esplicitamente nel documento dedicato.

### Registro stratigrafico, testimonianze da verificare, grammatica nascosta tra i mondi
**Stato:** tutte esplicitamente "futuro" nei documenti originali. Il documento sugli eventi di mondo propone ora un primo aggancio concreto per il registro stratigrafico ("Eco del passato": una cella con molta morte accumulata produce un bloom anomalo se rivisitata da un decompositore) — non un'implementazione completa, solo un punto di partenza più concreto di quanto ci fosse prima.
**Azione concreta:** nessuna ora — restano solo direzione concettuale, da riprendere quando (e se) il gioco avrà raggiunto una base stabile ai livelli 1 e 2.

---

## Checklist trasversale: "è da giocare o solo da osservare?"

Da applicare a **ogni** sistema, esistente o futuro, prima di considerarlo pronto per l'implementazione:

1. **Riconducibilità:** questo sistema, nel suo comportamento in una run specifica, può essere ricondotto a una scelta passata del giocatore (dove ha seminato, stressato, mutato)? Se la risposta è no, va ridisegnato o spostato a un livello più raro (tier 3), dove l'assenza di controllo diretto è accettabile perché l'evento è eccezionale, non ricorrente.
2. **Budget di azione:** le uniche leve dirette restano le 4 azioni esistenti (seed, stress, cull, splice) con budget 3 punti/era (GDD §6/§5.9). Nessun sistema futuro dovrebbe introdurre una quinta leva diretta senza una giustificazione forte — diluire quel budget indebolisce la tensione "un'era = un esperimento deliberato" che il GDD identifica esplicitamente come pilastro del bilanciamento.
3. **Test dello spettatore:** per ogni nuova meccanica, chiedersi "se la tolgo, il giocatore perde una decisione o solo uno spettacolo?" Se la risposta è "solo uno spettacolo", il sistema appartiene al livello 3 (raro, cerimoniale) o va ripensato per agganciarlo a una scelta reale — non va mai promosso a livello 1 o 2 così com'è.

**Rinforzo esplicito, in tensione diretta col pilastro 5:** più contenuto "da guardare" si aggiunge (meraviglia e scoperta), più cresce il rischio di scivolare verso l'osservazione passiva a scapito del gioco vero — è una tensione permanente da tenere sempre visibile, non un rischio risolto una volta per tutte. **Audit applicato a `culture-shock-wonder.md` e `culture-shock-biome-cosmic-events.md`:** la maggior parte delle proposte passa il test perché riconducibile a scelte passate del giocatore (fermentazione tossica in Palude, letargo in Vetta — reversibile con Stress, imprinting termico, risonanza nei cristalli) o perché crea una nuova decisione invece di sostituirla (raggio cosmico e grande oscuramento sono minacce a cui reagire, non solo eventi da guardare). Un piccolo gruppo resta **puro spettacolo deliberato** (silenzio anomalo, relitto, cicatrice testuale) — non fallisce il test per errore, il pilastro 5 richiede anche momenti di meraviglia senza leva — ma proprio per questo **deve restare eccezionale**: se la frequenza di questi elementi sale, un accento raro diventa un pattern, e un pattern puramente osservativo comincia a scalzare il gioco vero. Ogni futura proposta di questo tipo va sottoposta allo stesso audit prima di essere adottata.
