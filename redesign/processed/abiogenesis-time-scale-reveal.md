# Abiogenesis — scala temporale a tre livelli e reveal di fine era

Documento autonomo per un task di design/integrazione. Contiene contesto, decisioni, e alcune risposte **proposte** (non definitive) alle domande aperte lasciate esplicite — da decidere insieme prima di passare tutto a chi implementa.

## Contesto

Idea di partenza: introdurre la modalità realtime già in roadmap, allungare molto la durata delle ere, e aggiungere un livello temporale intermedio tra pulse ed era per poter avanzare rapidamente senza perdere la granularità del pulse. Alla chiusura di ogni era, il mondo mostra al giocatore cosa è successo in quell'era — un evento degno di essere raccontato — e le eventuali evoluzioni maturate durante l'era vengono applicate proprio in quel momento.

Questa proposta generalizza ed espande due idee già presenti in un documento precedente (proposte per rendere il gioco più coinvolgente nei primi minuti):

- **2.4 — Eventi epocali:** shock globali rari e procedurali sopra gli scalari ambientali esistenti. Qui gli eventi di fine era non sono più solo rari/eccezionali, diventano il battito regolare del gioco a ogni chiusura di era.
- **2.6 — Il momento della rivelazione:** una singola frase generata dai dati alla chiusura di un mondo. Qui lo stesso principio si applica a ogni era, non solo alla fine dell'intero mondo — e si arricchisce con l'applicazione dell'evoluzione maturata.

## Decisioni

### 1. Scala temporale a tre livelli: Pulse → Stagione → Era

- **Pulse** — invariato, l'unità più piccola di avanzamento (rename già completato, task 118 GDD).
- **Stagione** (nuovo) — livello intermedio, un blocco di pulse. **È la nuova unità di decisione del giocatore** (vedi punto 2).
- **Era** — **molto più lunga** rispetto al baseline attuale (`ERA_TICKS = 25`, §5.9), e diventa l'**unità di racconto**: è la scala a cui il mondo tira le somme e comunica cosa è successo (vedi punto 3).

Il nome "stagione" completa una gerarchia intuitiva senza spiegazioni: un pulse è troppo piccolo per raccontare qualcosa, una stagione è la scala giusta per notare un cambiamento percepibile e prendere una decisione, un'era è la scala dove la storia si sedimenta.

### 2. La stagione è l'unità di decisione, l'era l'unità di racconto

**Problema che risolve.** Le ere non sono solo tempo: sono **decisioni**. Il budget di 3 punti azione è per era (§5.9) e il GDD dice esplicitamente che il modello temporale coincide col loop mentale ("un'era = un esperimento deliberato"). Allungare l'era senza toccare il budget non renderebbe il gioco più lungo in modo interessante — ridurrebbe la densità di decisioni per unità di tempo, rendendolo più passivo. E il budget di 60→45 ere per mondo (§5.9), con ere molto più lunghe, farebbe esplodere la durata di un mondo.

**Decisione:**

- **Il budget di azioni si ricarica a ogni stagione**, non a ogni era. Il numero di decisioni per mondo resta nello stesso ordine di grandezza di oggi.
- **Il budget del mondo, contato in ere, va abbassato drasticamente** — se un'era vale ~4 stagioni, il baseline 60→45 ere diventa qualcosa come **15→11 ere**. La durata totale in pulse resta simile a oggi; cambia il ritmo, non la lunghezza.
- **L'era diventa rara e pesante** — condizione necessaria perché il reveal di fine era funzioni come beat cerimoniale. Con 60 reveal per mondo sarebbero rumore; con 12-15 diventano momenti.
- **Gli obiettivi a durata vanno espressi in stagioni**, non in ere (es. "≥3 specie coesistenti per N stagioni") — l'unità di interazione del giocatore è cambiata, e §8 del GDD tuna esplicitamente gli obiettivi *nell'unità di interazione del giocatore*. Alcuni obiettivi potrebbero restare più naturali in ere: va valutato caso per caso nel documento dedicato agli obiettivi.

### 3. Il reveal di fine era è un beat dedicato, non una riga di log

Con ere molto più lunghe e rare, questo momento deve avere peso:

- Il gioco **si ferma da solo** alla chiusura dell'era e mostra una card/vignetta dedicata (non testo che scorre in un log tra altri eventi).
- Il giocatore deve vedere il reveal prima di poter riprendere ad agire.
- **Livelli di rilevanza** (minore / notevole / epocale) — la presentazione scala di conseguenza: un evento minore può essere un badge discreto, uno epocale può occupare tutto lo schermo.
- Il reveal mostra, quando pertinente, un **confronto prima/dopo** (es. un'icona di specie che cambia se ha evoluto un tratto) accanto al testo generato — non solo la frase, anche l'evidenza visiva del cambiamento.

### 4. Evoluzione: matura durante l'era, si applica al reveal

Le pressioni evolutive si accumulano durante l'era; se maturano, l'evoluzione risultante viene **applicata nel momento del reveal**, non istantaneamente quando la condizione si verifica. Trasforma l'attesa della fine dell'era in anticipazione — il giocatore passa l'era a costruire ipotesi su cosa sta maturando, e il reveal conferma o sorprende.

### 5. Avanzamento continuo (non realtime), e controllo "avanza al prossimo evento notevole"

**Chiarimento su una conflazione precedente:** in versioni precedenti di questo e altri documenti la modalità "realtime" era trattata come acquisita perché già in roadmap. Il GDD la marca `[OPEN / future]`, non in MVP (§4). Vanno distinte due cose diverse:

- **Avanzamento continuo** — il tempo scorre finché non lo fermi o finché non raggiungi un checkpoint. Con ere lunghe **diventa un requisito ergonomico**, non un extra: senza, il giocatore dovrebbe premere ripetutamente un tasto per attraversare una stagione. **Adottato** come parte della struttura temporale.
- **Realtime "vero"** — il tempo scorre *anche mentre il giocatore pianifica e agisce*. **Scartato definitivamente**: contraddirebbe il modello "un'unità di tempo = un esperimento deliberato", renderebbe il budget per stagione difficile da leggere, e metterebbe in crisi i reveal che devono fermare l'azione. Non è più una questione aperta.

**Controllo "avanza al prossimo evento notevole":** oltre all'avanzamento per stagione/era, un comando che avanza automaticamente **fino al prossimo evento sopra una certa soglia di rilevanza**. Con stagioni come unità di decisione ed ere lunghe è probabilmente il controllo più utile del gioco — e non richiede realtime vero per funzionare.

### 6. Bilanciamento da rivedere, non solo presentazione

I numeri esistenti (soglia di riproduzione, guadagni energetici, `selection_pressure_threshold`, durate degli obiettivi) erano tarati su ere da 25 pulse. Allungare molto le ere **non è un cambio di presentazione** — richiede una ritaratura dell'intero set. In particolare:

- I coefficienti energetici proposti nel documento sul bilanciamento della matrice assumono `ERA_TICKS = 25`: vanno rivalutati insieme a questa revisione.
- La `selection_pressure_threshold` (`20.0`, §5.9) è tarata sull'era corta. Se l'era si allunga senza ritararla, le speciazioni scatterebbero molte volte per era, banalizzando sia l'obiettivo `Speciation` sia l'Emersione.
- Il budget di ere per mondo va abbassato (vedi punto 2), e il budget di punti azione spostato sulla stagione.

## Domande aperte — con una risposta proposta, da confermare insieme

### Le evoluzioni in maturazione danno indizi prima del reveal, o è una sorpresa totale?

**Proposta:** sì, danno indizi indiretti durante l'era (popolazione o energia che si comportano in modo insolito) prima che il reveal confermi cosa è successo. Il reveal diventa quindi una **conferma**, non una sorpresa dal nulla — coerente col principio già centrale nel gioco che la scoperta è guidata da evidenza osservabile, non da colpi di scena arbitrari. Da confermare: è la scelta che meglio si allinea al resto del design, ma va validata insieme prima di considerarla definitiva.

### Cosa succede se una specie si estingue prima che la sua evoluzione maturi?

**Proposta:** per una prima implementazione, l'evoluzione in maturazione va semplicemente persa con l'estinzione della specie — la soluzione più semplice, e ragionevole come punto di partenza. **Ma vale la pena segnalare fin da ora un collegamento naturale**: un'evoluzione mai completata è esattamente il tipo di informazione che il "registro stratigrafico" (proposta 2.1 in un altro documento — una cella che ricorda chi è morto lì e quando) potrebbe un giorno registrare. Non è in scope implementarlo ora, ma non andrebbe scartata l'idea in fase di design dei dati, per non doverla reintrodurre a fatica più avanti se si decide di farlo.

## Fuori scope

- Valori numerici esatti (durata di pulse/stagione/era, soglie di rilevanza degli eventi, quota di budget ricaricata a stagione) — tutti da bilanciare in fase di implementazione/playtest.
- Implementazione del registro stratigrafico (2.1) — solo menzionato come collegamento concettuale per l'evoluzione persa.
- Dettaglio della modalità realtime in sé (già in roadmap separatamente) — qui si assume solo che esista e che debba convivere con questa struttura temporale.
