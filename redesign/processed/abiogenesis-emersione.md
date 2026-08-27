# Abiogenesis — Emersione: da microscopico a macroscopico

Documento autonomo per un task di design/integrazione. Propone un'ipotesi concreta e dettagliata di come innescare la transizione di una specie da popolazione microscopica a organismo macroscopico singolo — sia come tipo di finale-mondo, sia come base per un'eventuale estensione futura. Costruisce sopra tre documenti già esistenti (archetipi biochimici/famiglie di tratti, scala temporale a tre livelli e reveal, generazione dinamica del testo narrativo) e li riassume qui dove necessario per restare autosufficiente.

## Perché questa meccanica, e perché ora

Il gioco si chiama Abiogenesi: la storia di come la vita comincia semplice e diventa complessa. La transizione a macroscopico è la conclusione narrativa naturale di quella storia, non una feature scollegata — e risolve senza doverla decidere subito la tensione tra un'esperienza a run brevi (tipo Plague Inc) e una più simile a un colony builder di lungo periodo: le run restano contenute, ma l'Emersione diventa **la porta verso un'eventuale fase due**, costruita solo come aggancio, non come sistema da sviluppare ora.

## Principio di innesco: integrazione, non accumulo

Coerente con tutto il resto del design (il mistero non deve mai ridursi a una singola barra numerica che si riempie), l'innesco **non è un singolo contatore che supera una soglia**:

1. **Diversità di famiglia nei tratti evoluti lungo una lineage** — condizione necessaria, sufficiente da sola.
2. **Sopravvivenza a un evento catastrofico** — acceleratore opzionale quando quel sistema esisterà, non un requisito bloccante.

Nessuna delle due va mostrata come barra di progresso esplicita al giocatore — si scopre che sta per succedere solo dagli indizi indiretti già previsti dal sistema di reveal (comportamenti insoliti prima della conferma), non da un contatore visibile.

### Condizione 1 — Diversità di famiglia

Ogni tratto (cfr. documento sugli archetipi biochimici) appartiene a una delle 5 famiglie: strutturale, metabolico, segnalazione, genetico, riserva. Una specie parte con un genoma iniziale (i tratti con cui è stata seminata o generata) e può **accumulare tratti aggiuntivi tramite evoluzione** nel tempo.

**Precisazione sul meccanismo (allineata al sistema di evoluzione per speciazione, GDD §5.11):** l'evoluzione non aggiunge tratti alla stessa specie — crea una **nuova specie** a ogni evento di soglia superata, con un genoma modificato dallo stimolo dominante rispetto alla specie genitore. Di conseguenza, "tratto evoluto" per una specie in punta di una lineage significa: **un tratto presente nel suo genoma che non era nel genoma della specie originale da cui quella lineage è partita** (seminata dal giocatore, o wild). La copertura di famiglie si traccia quindi **lungo la catena di discendenza**, sommando i tratti comparsi a ogni salto di speciazione — non come accumulo diretto sulla stessa specie.

**Proposta di soglia:** una specie (in punta di lineage) diventa candidata quando i tratti comparsi lungo la sua discendenza (rispetto al genoma della specie originale) coprono **almeno 3 delle 5 famiglie**. Non conta quanti tratti totali sono comparsi, conta la copertura tra famiglie diverse — una lineage con 4 tratti evoluti tutti "metabolico" non qualifica, una con 3 tratti da 3 famiglie diverse sì. In pratica, questo richiede quasi sempre più speciazioni successive (una lineage con un solo salto difficilmente copre 3 famiglie), quindi la "profondità" non è un numero imposto a parte — emerge naturalmente dalla condizione di copertura.

**Percorso alternativo più raro:** copertura di **2 famiglie + 1 xenotratto** (cfr. documento sugli archetipi biochimici, sezione "estensione futura") qualifica allo stesso modo. **Origine meccanica proposta per gli xenotratti**, più precisa di quella nel documento originale: a ogni evento di soglia superata (§5.11), l'edit al genoma è normalmente una modifica standard (cambio/aggiunta di un tratto reale, spostamento dell'optimum termico) — ma con una probabilità bassa e separata, l'edit produce invece uno xenotratto. Questo dà agli xenotratti un aggancio meccanico preciso al sistema di speciazione esistente, invece di restare un evento scollegato. Un xenotratto, essendo già raro ed emerso da evoluzione, pesa come segnale di complessità più di un tratto reale aggiuntivo — questo percorso resta naturalmente più raro del primo.

### Condizione 2 — Sopravvivenza a un evento catastrofico (acceleratore opzionale, non requisito bloccante)

Gli eventi catastrofici (cfr. proposta "eventi epocali" in altro documento) non sono ancora un sistema deciso nel GDD — per questo l'Emersione **non deve dipendere solo da essi**, altrimenti resterebbe bloccata in attesa di un sistema che potrebbe non arrivare mai. La condizione 1 da sola è già sufficiente per rendere una specie candidata (vedi innesco probabilistico sotto).

**Se gli eventi epocali verranno implementati**, la sopravvivenza a uno di essi mentre la condizione 1 è già vera diventa un **acceleratore**: riduce di 1 il numero di famiglie richieste (da 3 a 2, o da "2 famiglie + xenotratto" a "1 famiglia + xenotratto"), oppure accelera il tiro probabilistico sotto (proposta: raddoppia l'incremento di probabilità per era). Riflette comunque l'idea evolutivamente vera che la complessità emerge sotto pressione — ma senza rendere l'intera meccanica dipendente da un sistema non ancora costruito.

**Soglia di popolazione (proposta, resta valida se si usa l'acceleratore):** la popolazione residua immediatamente dopo l'evento deve restare sopra una soglia minima (proposta indicativa: 5 individui) — per evitare che un singolo superstite fortunato conti come "sopravvivenza" nel senso pieno del termine.

## Innesco probabilistico, non automatico

Per evitare che l'Emersione scatti nell'istante esatto in cui la condizione 1 è tecnicamente vera (troppo meccanico, prevedibile), l'innesco è **probabilistico e crescente**:

- Alla prima era in cui la condizione 1 risulta vera, viene tentato un tiro con probabilità base (proposta: 10% — o 20% se in quell'era è attivo anche l'acceleratore da evento catastrofico, condizione 2).
- Se non scatta, la probabilità **aumenta ad ogni era successiva** in cui la condizione resta vera (proposta: +10% per era, +20% per era se l'acceleratore è attivo, fino a un tetto — proposta: 80%) — mai certezza assoluta al 100%, per lasciare un margine di casualità anche nel caso limite.
- Se in una qualunque era la condizione smette di essere vera (es. la lineage perde diversità di famiglia perché la specie in punta si estingue), il contatore di probabilità si azzera e riparte da capo se la condizione torna vera in futuro (anche con una lineage diversa nello stesso mondo).

Questo dà al giocatore esperto un segnale indiretto legittimo (la specie "sembra sul punto di qualcosa" per più ere consecutive) senza mai dichiararlo esplicitamente — coerente col principio di indizi indiretti già stabilito nel documento sulla scala temporale.

## Cosa succede quando l'Emersione avviene

- **Collasso da popolazione a entità singola.** La specie smette di essere un conteggio (N individui, energia media) e diventa un **singolo organismo macroscopico** con un'identità propria — non più un numero, un individuo.
- **Tratti ereditati come caratteristiche definitorie.** Per ciascuna famiglia coperta lungo la lineage, l'organismo emerso eredita il tratto con il peso di evidenza più alto accumulato (il tratto "più confermato", non uno scelto a caso) come sua caratteristica distintiva visibile — indipendentemente da quale specie della catena di discendenza lo ha effettivamente introdotto. È la sintesi dell'intera storia evolutiva della lineage, non un dato nuovo generato ad hoc.
- **Marker visivo distinto.** Sulla griglia di gioco, l'organismo emerso non è più un pallino colorato tra tanti — richiede un marker unico, diverso da qualunque icona di metabolismo già definita (cfr. documento set di icone specie), proprio perché non è più un rappresentante di una popolazione ma un'entità unica. Non è in scope qui definire l'icona esatta.
- **Reveal al livello massimo di cerimonia.** L'Emersione è candidata naturale per il tier più alto del sistema di reveal (cfr. documento sulla scala temporale, livelli minore/notevole/epocale) — probabilmente l'unico tipo di evento che merita sempre quel livello, indipendentemente dal punteggio di ranking calcolato dal sistema di generazione narrativa (cfr. documento sulla generazione dinamica del testo): l'Emersione dovrebbe **scavalcare** il ranking ordinario e garantire sempre il tier massimo, non competere con altri eventi dell'era per meritarselo.

## Emersione come tipo di finale del mondo

Un secondo tipo di finale, distinto dagli obiettivi standard — non li sostituisce, li affianca come traguardo più raro e più prestigioso:

- Un mondo può chiudersi **per obiettivo** (sequenza completata) o **per Emersione**.
- **Dipendenza risolta:** l'Emersione richiede una catena di almeno 3 speciazioni imparentate, che arriva sempre dopo la prima speciazione — quella che chiude l'obiettivo finale `Speciation`. È stato quindi deciso (cfr. documento sugli obiettivi) che **completare gli obiettivi segna la vittoria come flag, senza terminare forzatamente il mondo**: il giocatore può continuare a giocare finché il budget di ere non si esaurisce. Senza questa decisione l'Emersione sarebbe irraggiungibile per costruzione.
- Il tipo di finale va conservato come metadato del mondo — ogni mondo concluso porta con sé *come* si è concluso, non solo che si è concluso.

## Base per un'estensione futura (fuori scope qui, solo aggancio)

L'Emersione come evento terminale è pensata per restare un seme legittimo di un'eventuale fase due del gioco, dove il giocatore non segue più una popolazione astratta ma l'organismo emerso, con le caratteristiche ereditate dai tratti accumulati durante la run. **Questo documento non specifica quella fase due** — costruisce solo il momento di transizione in modo che, se in futuro si deciderà di estenderlo, l'aggancio narrativo e meccanico esista già. **Riferimento guida assegnato per quando si riprenderà questo lavoro:** *Children of Time* di Adrian Tchaikovsky (`culture-shock-identity-visual-inspirations.md`) — non solo un'ispirazione tematica generica, ma la fonte diretta da cui è nata l'idea stessa dell'Emersione, e il romanzo da rileggere per orientare le decisioni sulla fase due.

## Cosa serve per l'integrazione

- **Tracciamento della lineage:** ogni specie nata per speciazione deve conservare un riferimento alla specie genitore (probabilmente già presente, dato che l'edit del genoma parte da una copia del genitore, §5.11) — la condizione 1 richiede di poter risalire dalla specie in punta fino alla specie originale (seminata o wild) attraverso l'intera catena.
- **Calcolo di copertura famiglia lungo la lineage:** dato l'insieme di tratti comparsi a ogni salto della catena (rispetto al genoma della specie originale), calcolare quante delle 5 famiglie sono coperte (o 2 famiglie + xenotratto per il percorso alternativo) — richiede che ogni tratto sappia a quale famiglia appartiene (cfr. documento archetipi biochimici).
- **Origine meccanica degli xenotratti:** il sistema di speciazione (§5.11) deve prevedere, con probabilità bassa e separata, che l'edit del genoma a un evento di soglia superata produca uno xenotratto invece della modifica standard (tag reale o optimum termico) — va aggiunto come possibile esito dell'evento `SelectionThresholdCrossed`.
- **Collegamento opzionale con gli eventi catastrofici:** se quel sistema verrà implementato, serve un modo per verificare, al momento della risoluzione di un evento, se una lineage era già "candidata" (condizione 1 vera) e se la sua popolazione residua supera la soglia minima proposta, per applicare l'acceleratore.
- **Contatore di probabilità crescente per lineage candidata:** stato persistente per lineage (non per singola specie, dato che la specie in punta può cambiare nel tempo restando la stessa lineage) che tiene il conteggio di ere consecutive con la condizione 1 vera, si azzera se la condizione smette di valere.
- **Logica di collasso popolazione → entità singola:** cosa succede esattamente allo stato interno della specie (energia, riproduzione, interazioni con la matrice) una volta collassata a organismo singolo **non è specificato in dettaglio in questo documento** — è una ridisegnazione della simulazione per un caso particolare, da trattare come task a parte quando si arriva all'implementazione.
- **Selezione del tratto ereditato per famiglia:** richiede accesso al peso di evidenza accumulato per tratto (lo stesso dato già usato dal notebook per determinare le conferme) per scegliere il tratto "più confermato" per famiglia, lungo l'intera lineage.
- **Override del tier di reveal:** il sistema di generazione narrativa (cfr. documento dedicato) deve prevedere un caso speciale in cui l'Emersione forza il tier "epocale" indipendentemente dal punteggio di ranking ordinario.
- **Metadato di tipo di finale sul mondo:** da aggiungere al modello dati del mondo, per distinguere "concluso per obiettivo" da "concluso per Emersione".

## Fuori scope

- Icona/marker visivo esatto per l'organismo emerso.
- Ridisegno della simulazione per un'entità macroscopica singola (energia, movimento, interazioni) dopo il collasso.
- La fase due del gioco costruita sull'organismo emerso — solo menzionata come possibilità futura.
- Valori numerici esatti (soglia di popolazione, probabilità base e incremento, tetto massimo di probabilità) — tutti proposti come indicativi, da bilanciare in playtest.
