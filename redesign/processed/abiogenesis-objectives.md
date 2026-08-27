# Abiogenesis — obiettivi del mondo: generazione procedurale e nuovi tipi

Documento autonomo per un task di design/integrazione. Definisce le regole per rendere gli obiettivi (GDD §8) davvero procedurali — ancorati ai dati reali del mondo generato, mai completabili per coincidenza, mai dipendenti da meccaniche rare — e propone nuovi tipi di obiettivo che diano un ruolo a meccaniche oggi prive di un traguardo dedicato.

## Contesto

Il GDD prevede 2→3 obiettivi in sequenza (early→late) più un obiettivo finale sempre presente, Speciazione (§8). I tipi oggi esistenti (Coexistence, SurviveIn, TriggerBloom, Speciation) misurano cose diverse tra loro (diversità, resistenza, crescita, evento di speciazione) — buon segnale di partenza, nessuna sovrapposizione concettuale. Questo documento non tocca la quantità né la struttura a sequenza, interviene su: correttezza (un obiettivo non deve poter risultare già soddisfatto nell'istante in cui compare), generazione dei parametri (devono derivare dal mondo reale, non essere generici), e ampiezza del pool di tipi disponibili.

## Principio 1 — snapshot all'attivazione (regola generale per ogni obiettivo)

**Problema:** un obiettivo che misura uno *stato* invece che un *cambiamento a partire dalla sua attivazione* può risultare già soddisfatto nell'istante in cui diventa il traguardo corrente — il giocatore lo vede comparire già completato, senza aver fatto nulla per meritarselo in quel momento. Esempio concreto: la speciazione può avvenire per qualunque specie, in qualunque momento del mondo, anche prima che l'obiettivo Speciazione sia attivo — se l'obiettivo controllasse solo "è mai avvenuta una speciazione", potrebbe risultare vero all'istante.

**Regola:** ogni obiettivo, quando diventa attivo, registra uno **snapshot dello stato del mondo in quel momento**. La condizione di completamento richiede un cambiamento o una persistenza **successiva** a quello snapshot, mai uno stato preesistente. Si applica a tutti i tipi, esistenti e nuovi:

- `Coexistence`: non "N specie coesistono ora", ma "N specie coesistono per M ere consecutive **a partire da quando l'obiettivo è diventato attivo**" — se il giocatore le aveva già, il timer parte comunque da zero all'attivazione, non retroattivamente.
- `Speciation`: non "è mai avvenuta una speciazione", ma una **nuova** speciazione avvenuta dopo l'attivazione, la cui specie risultante sopravvive per almeno un'era intera (soglia più sostanziale di "un evento qualsiasi", per dare peso reale a un obiettivo finale).
- Qualunque nuovo tipo proposto in questo documento (tutti richiedono per costruzione una finestra di ere consecutive) eredita automaticamente la regola.

## Principio 2 — nessun obiettivo dipende da meccaniche rare o probabilistiche

Gli obiettivi sono **requisiti obbligatori per vincere** — devono restare sempre raggiungibili con azione deliberata entro un budget di ere finito. Emersione e xenotratti restano **permanentemente esclusi** dal pool di tipi di obiettivo, non solo "per ora": entrambi conservano una componente probabilistica anche a condizioni soddisfatte (10-80% crescente per l'Emersione, probabilità bassa e separata per gli xenotratti) — un giocatore potrebbe fare tutto correttamente e comunque non ottenerli per un tiro sfortunato, il che è accettabile per un bonus opzionale ma inaccettabile per un requisito di vittoria. Verifica di coerenza per ogni nuovo tipo proposto in futuro: deve superare il test di agency già scritto nel documento di consolidamento dei sistemi ("se tolgo questo sistema, il giocatore perde una decisione o solo uno spettacolo?").

## Caso speciale — Speciazione con bersaglio specifico

Applicazione concreta dei due principi sopra al caso più delicato:

- **Se non è mai avvenuta una speciazione** nel mondo prima dell'attivazione dell'obiettivo → resta la versione generica corretta dal Principio 1 (nuova speciazione dopo l'attivazione, sopravvivenza di un'era).
- **Se è già avvenuta almeno una volta** → l'obiettivo si aggiorna a un bersaglio specifico: *"induci una speciazione nella specie X"*, dove X è scelta tra le specie **attualmente vive che non hanno ancora speciato**. Il giocatore non può più completarlo aspettando che scatti da qualche parte sulla mappa — deve individuare dove vive X e applicarle pressione mirata (stress, esposizione a un'interazione dannosa, disallineamento ambientale deliberato).

**Regole di supporto necessarie:**

- **Selezione della specie bersaglio:** deterministica dal seed (non ricalcolata a caso a ogni controllo), tra le specie idonee con popolazione sopra una soglia minima di vitalità — esclude sia specie già vicine all'estinzione (obiettivo ingiusto) sia, se in futuro sarà disponibile un dato di pressione già accumulata per specie, un bias verso specie già quasi pronte a speciare (renderebbe l'obiettivo di nuovo troppo facile).
- **Fallback su estinzione del bersaglio:** se la specie X si estingue prima del completamento, il sistema ri-seleziona automaticamente una nuova specie bersaglio tra quelle rimaste idonee, con lo stesso criterio — l'obiettivo resta sempre risolvibile, cambia solo il bersaglio.
- **Nome visibile, non un ID astratto:** il testo dell'obiettivo usa il nome/icona della specie già stabiliti altrove (Catalog, HUD, banca genomi) — es. *"induci una speciazione in Halo"*, mai "specie #4".

**Perché questo caso specifico è permesso, a differenza di Emersione/xenotratti:** la pressione che porta a speciazione è **deterministica** (soglia numerica superata, non un tiro probabilistico) e il giocatore ha **agency diretta** per indurla di proposito. Supera il test di agency; Emersione e xenotratti no.

## Generazione procedurale dei parametri — leggere il mondo reale

Ogni tipo di obiettivo, invece di parametri generici o puramente casuali, dovrebbe scegliere i propri parametri leggendo dati **effettivamente presenti** nel mondo già generato:

- `SurviveIn` sceglie come bersaglio uno dei biomi/feature *effettivamente presenti* in quel mondo (Cratere se c'è, Palude se c'è) — non tutti i mondi avranno lo stesso ventaglio di biomi estremi, quindi non tutti potranno pescare lo stesso sapore di obiettivo. Introduce varietà reale tra mondi senza nuovi tipi.
- `TriggerBloom` sceglie come specie bersaglio una tra quelle effettivamente seminabili con la banca genomi di quel mondo specifico, non un riferimento generico.
- I nuovi tipi proposti sotto (Tolleranza, Radicamento, Convivenza selvatica) seguono lo stesso principio per costruzione — hanno bisogno di un bioma, un tratto condizionato dal terreno, o una wild species effettivamente presenti per poter essere generati.

**Vincolo non negoziabile:** il testo dell'obiettivo dichiara **cosa** misura, mai **perché** quel parametro specifico è stato scelto per quel mondo. Se in futuro sarà implementato il bias di famiglia dominante (cfr. documento archetipi biochimici), un obiettivo non deve mai rivelarlo (es. mai "sfrutta la famiglia genetica di questo mondo") — sarebbe un leak diretto di un'informazione che il gioco vuole far scoprire da solo. Stessa regola già applicata ai nomi dei tratti (mai un'indicazione di segno prima dell'osservazione).

## Nuovi tipi di obiettivo proposti

Aggiunte al pool dei tipi disponibili — non aumentano quanti obiettivi attivi per mondo (resta 2→3 + Speciazione, vedi sotto), aumentano solo la varietà tra cui il generatore può scegliere.

### Omeostasi (energia)

Mantieni l'energia media di una specie bersaglio dentro una fascia stabile (né troppo bassa da rischiare estinzione, né troppo alta da suggerire sovrappopolazione incontrollata) per N ere consecutive. Nessun tipo esistente misura l'energia direttamente — colma un vuoto reale. Qualitativamente diverso dagli altri: premia il **bilanciamento attivo** (uso deliberato di stress/splice come correzione preventiva) invece di diversità, resistenza passiva o crescita. Unico tipo che dà uno scopo strategico esplicito allo stress termico come strumento di cura, non solo come innesco di evoluzione.

### Tolleranza (chemiolitotrofo / tossicità)

Mantieni una specie viva in una zona ad alta tossicità (Palude, o area adiacente al Cratere) per N ere. Dà un traguardo esplicito al metabolismo chemiolitotrofo (GDD §5.4), oggi attivo ma privo di un obiettivo che lo valorizzi specificamente.

### Convivenza selvatica (wild species)

Mantieni viva una wild species (GDD §9) insieme ad almeno una specie seminata dal giocatore, per N ere. Dà un ruolo meccanico attivo alle wild species oltre il flavor iniziale — oggi, una volta osservate, non hanno più alcuna funzione nel resto della partita.

### Radicamento (tratti condizionati dal terreno)

Mantieni una specie viva specificamente in un bioma legato a un tratto condizionato dal terreno (GDD §5.5) attivo in quel mondo. Dà un ruolo esplicito nel layer obiettivi a quella meccanica, oggi solo un modificatore ambientale di sfondo.

### Prima conferma (world 0 / onboarding)

Conferma almeno una relazione della matrice entro le prime N ere. Pensato principalmente come tipo riservato al primo mondo (coerente con l'ammorbidimento già previsto per `Coexistence` in world 0, GDD §9) — rinforza il loop core proprio nei primi minuti, prima di introdurre obiettivi più esotici.

## Quantità — invariata

Nessun quarto obiettivo simultaneo, nemmeno nei mondi più tardi. Coerente col principio "manopola singola" già scritto nel documento di consolidamento: ogni obiettivo in più è un bersaglio aggiuntivo da tenere a mente in parallelo — carico cognitivo, non necessariamente difficoltà interessante. La difficoltà deve continuare a venire dall'ambiente e dalla matrice (che già scalano), non dall'accumulo di traguardi paralleli. Il pool di tipi si allarga (5 nuovi tipi proposti), la sequenza resta 2→3 + Speciazione.

## Genere di obiettivo scartato — decodifica percentuale della matrice

Esplicitamente evitato: un tipo come *"decodifica il 60% della matrice"* contraddice un principio già scritto nel GDD — il gioco è pensato perché basti decifrare 3-4 celle rilevanti su ~20 per vincere (§16.5), non l'intera matrice. Un obiettivo che richiede una percentuale di decodifica romperebbe quell'economia e trasformerebbe il notebook da strumento usato quanto serve a compito da completare per intero.

## Effetto al completamento di un obiettivo — proposta aperta, non decisa

Due opzioni considerate, nessuna scelta come definitiva in questo documento:

- **Nessun effetto collaterale** (opzione più sicura): l'unico premio è lo sblocco del successivo. Resta pulito, zero rischio di introdurre un premio percepito come ingiustificato.
- **Piccolo rinforzo narrativo ancorato a dati reali**: al completamento, il notebook riceve una promozione di una relazione osservata a *ipotesi* (non a conferma piena, servirebbe evidenza vera) — ma **solo se quella relazione è direttamente legata all'obiettivo appena completato** (es. dopo `SurviveIn` nel Cratere, la promozione riguarda un tratto osservato lì, non uno scelto a caso). Se non si riesce a garantire questo ancoraggio, non va implementata — un premio scollegato dal traguardo appena raggiunto sembrerebbe arbitrario, contraddirebbe il principio "ogni colore/dato deve rappresentare qualcosa di reale" già stabilito altrove.

## Vittoria come flag, non fine forzata del mondo **[deciso]**

**Problema:** se l'obiettivo `Speciation` (l'ultimo della sequenza) si chiude alla prima speciazione, e il completamento di tutti gli obiettivi termina forzatamente il mondo, **l'Emersione è irraggiungibile per costruzione** — richiede una catena di almeno 3 speciazioni imparentate (cfr. documento Emersione), che arriva sempre dopo la prima.

**Decisione:** completare tutti gli obiettivi segna la vittoria come **traguardo raggiunto** (flag), non termina forzatamente il mondo. Il giocatore resta libero di continuare a giocare lo stesso mondo finché il budget di ere non si esaurisce, e passa al mondo successivo quando decide lui.

**Perché è coerente col resto del design, non solo un fix per l'Emersione:**
- Esiste già un precedente interno: il GDD (§8) stabilisce che **la run finisce solo per scelta del giocatore, mai automaticamente**. Applicare la stessa filosofia al singolo mondo è coerenza, non un'eccezione.
- È aderente alla vision (§2): sei uno scienziato che coltiva, non un giocatore che completa livelli.
- Crea una tensione desiderabile: *hai già vinto, ma vuoi vedere se succede qualcosa di più raro?* — l'Emersione diventa il premio per chi sceglie di restare invece di passare oltre.
- Non toglie nulla a chi vuole solo avanzare: il passaggio al mondo successivo resta disponibile in qualunque momento dopo la vittoria.

## Cosa serve per l'integrazione

- **Snapshot all'attivazione:** ogni obiettivo, alla transizione a "attivo", deve registrare lo stato di riferimento del mondo in quel momento (popolazioni presenti, specie senza speciazione, ecc.) da cui misurare il progresso — non una condizione valutata solo sullo stato attuale.
- **Selezione del bersaglio per il caso speciale Speciazione:** logica di selezione deterministica (seedata) tra specie idonee, più il fallback di ri-selezione su estinzione.
- **Lettura di dati reali del mondo per i parametri:** ogni tipo di obiettivo deve poter interrogare quali biomi/wild species/specie della banca genomi sono effettivamente presenti in quel mondo specifico, prima di scegliere i propri parametri.
- **5 nuovi tipi di obiettivo da implementare:** Omeostasi, Tolleranza, Convivenza selvatica, Radicamento, Prima conferma — ciascuno richiede accesso ai dati già elencati sopra per generarsi correttamente.
- **Vittoria come flag:** il modello di fine-mondo va cambiato da "completati gli obiettivi → passa al mondo successivo" a "completati gli obiettivi → mondo vinto, il giocatore decide quando passare oltre". Impatta anche il documento Emersione, che presuppone la possibilità di continuare a giocare un mondo già vinto.
- **Durate espresse in stagioni:** con la revisione della scala temporale (stagione = unità di decisione, cfr. documento dedicato), gli obiettivi a durata ("per N ere") vanno riespressi **in stagioni** — l'unità di interazione del giocatore è cambiata, e §8 GDD tuna esplicitamente gli obiettivi in quell'unità. Alcuni obiettivi potrebbero restare più naturali in ere (es. quelli legati a eventi che maturano su scala d'era): valutazione caso per caso, non una conversione meccanica.
- **Decisione da prendere separatamente:** se implementare l'effetto di completamento ancorato (proposta aperta sopra) o nessun effetto.

## Fuori scope

- Bilanciamento numerico esatto (soglie di ere per i nuovi tipi, soglia di vitalità minima per la selezione del bersaglio Speciazione) — da validare in playtest.
- Implementazione del bias di famiglia dominante sull'intensità della matrice (cfr. documento archetipi biochimici) — qui solo richiamato come vincolo per il testo degli obiettivi, non specificato di nuovo.
- La decisione sull'effetto di completamento di un obiettivo — segnalata come aperta, non decisa da questo documento.
- La conversione puntuale di ciascun tipo di obiettivo da ere a stagioni — indicata come direzione, non risolta voce per voce.
