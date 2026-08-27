# Culture Shock — l'incentivo a sperimentare: test e lezioni dai comparabili

Documento autonomo. Definisce un test verificabile per una domanda che nessun altro documento pone: **il gioco premia davvero chi sperimenta, o premia chi gioca sul sicuro?** Include le lezioni operative ricavate dai giochi più vicini nel panorama esistente.

## La domanda, formulata correttamente

Il costo nominale è già simmetrico: seminare per esperimento e seminare per un piano costano entrambi 1 punto. **La tassa sulla curiosità non è nel prezzo — è nell'opportunità mancata e nel rischio.** Un'azione pianificata produce progresso verso l'obiettivo; un esperimento produce informazione che potrebbe non servire a nulla, e in più può danneggiare (cella occupata inutilmente, interazione negativa scoperta a proprie spese, popolazione persa).

La domanda misurabile è quindi: **l'informazione si converte in progresso abbastanza in fretta da valere il punto speso?**

Se la risposta è no, il gioco premia sistematicamente chi *non* fa la cosa che il design vuole incoraggiare — ed è il tipo di errore che non si vede leggendo i documenti, solo misurandolo.

---

## Come misurarlo

### 1. Due bot contro lo stesso seed — il test principale

Il determinismo (§5.7 GDD) rende questo test quasi gratuito, e **non richiede interfaccia**: è headless, eseguibile già in Fase 1.

Due strategie automatiche, stesso mondo:
- **Sfruttatore** — agisce solo su relazioni già confermate, sperimenta il minimo indispensabile.
- **Esploratore** — sonda deliberatamente coppie di tratti sconosciute prima di impegnarsi.

Far girare entrambe su qualche centinaio di seed, confrontando **ere impiegate a completare gli obiettivi**.

**Criterio di fallimento:** se lo sfruttatore vince sistematicamente, gli incentivi sono sbagliati — il gioco sta premiando chi non gioca al gioco progettato.

**Nota:** non serve che l'esploratore vinca sempre. Serve che sia **competitivo**: se le due strategie sono vicine, con l'esploratore avvantaggiato nei mondi più difficili (dove l'informazione conta di più), il bilanciamento è sano.

### 2. Strumentazione delle azioni

Loggare, per ogni azione eseguita, se avviene in un contesto **noto** (relazioni coinvolte già confermate) o **ignoto**, e misurare il progresso-obiettivo per punto speso nelle due categorie.

Dice non solo *se* c'è squilibrio, ma *quanto* — utile per calibrare l'entità della correzione invece di procedere per tentativi.

### 3. La domanda al playtester

Una sola, molto specifica: **"c'è stato un momento in cui volevi provare qualcosa e non l'hai fatto perché non te lo potevi permettere?"**

Se la risposta è sì, è la conferma qualitativa di ciò che i bot misurano quantitativamente.

---

## Le leve correttive, se il test fallisce

In ordine di quanto sono invasive.

**1. `Isola` gratuita o a costo ridotto.** Già proposta come possibile quinta azione (`abiogenesis-actions.md`). È la leva più elegante disponibile: **non regala nulla** — non rende l'esperimento gratuito, lo rende *pulito*. Il giocatore paga comunque per agire, ma ottiene un'osservazione di peso pieno invece che confusa. Se il test fallisce, questa è la prima da provare.

**2. Verificare che `Splice` converta conoscenza in capacità abbastanza presto.** La conversione esiste già (Splice limitato ai tratti confermati), ma se `Splice` diventa utile solo a metà partita, **nelle prime stagioni l'informazione non ha ancora alcun valore strumentale** — ed è lì che la curiosità viene tassata di più. Da misurare: a che punto della partita mediana il primo `Splice` diventa una mossa sensata?

**3. Prima semina di ogni specie in un mondo a costo ridotto.** Mirata esattamente alla fase in cui il giocatore non sa ancora nulla.

**4. Generalizzare gli obiettivi che premiano le conferme.** `Prima conferma` esiste già ma solo come tipo riservato a world 0 (`abiogenesis-objectives.md`). Potrebbe diventare un tipo ricorrente del pool invece che un'eccezione dell'onboarding.

**Leva da NON usare: rendere la sperimentazione priva di rischio.** Il rischio è ciò che rende l'osservazione pulita una *scelta* invece che un automatismo. Toglierlo risolverebbe il problema uccidendo la tensione che lo rende interessante.

---

## Lezioni dai comparabili

Ricerca sul panorama esistente: **nessun gioco fa esattamente la stessa cosa**, ma le due metà di Culture Shock sono entrambe affollate separatamente, e i parenti più stretti stanno fuori dai simulatori di ecosistemi.

### Dai roguelike (identificazione oggetti) — il parente meccanico più diretto

NetHack, DCSS: pozioni e pergamene con effetti rimescolati a ogni run, imparati sperimentando. È l'antenato diretto della matrice.

- **Il costo dell'esperimento è il problema centrale che hanno affrontato per decenni.** Se sperimentare è troppo pericoloso, il giocatore smette e gioca solo il sicuro — esattamente il fallimento che il test sopra cerca di intercettare.
- **Il test del taccuino esterno.** I giocatori seri di roguelike tengono appunti fuori dal gioco. Se succede anche qui, è il segnale che il notebook — lo strumento più identitario del gioco — non sta funzionando. **Da aggiungere come domanda diagnostica esplicita al playtest:** *qualcuno ha preso appunti fuori dal gioco?*

### Da Alchemists (gioco da tavolo) — il parente più vicino nello spirito

Usa un'app che randomizza le regole dell'alchimia a ogni partita; i giocatori deducono per esperimenti e pubblicano teorie, rischiando reputazione se sbagliano.

- **Il rischio della pubblicazione.** Il meccanismo di ipotesi dichiarate (`culture-shock-identity.md`, post-MVP) è stato scritto con "nessun costo per sbagliare", per non scoraggiare l'ipotesi. Alchemists dimostra che **un costo può funzionare**, se ciò che si rischia è reputazione simbolica e non progressione meccanica. **Da riconsiderare** quando si riprenderà quel meccanismo — non necessariamente da adottare, ma la scelta va rifatta consapevolmente invece che data per chiusa.
- **Il deficit informativo strutturale.** In Alchemists non ci sono mai abbastanza esperimenti per la certezza: devi scommettere su inferenze incomplete. **Da verificare esplicitamente:** un giocatore ottimale riesce a raggiungere la certezza sulla parte rilevante della matrice? Se sì, il budget è troppo generoso.

### Da Eleusis (gioco di carte) — l'archetipo

Un giocatore fissa una regola segreta, gli altri la deducono osservando quali carte sono accettate.

- **L'ipotesi si riformula, non si vince o si perde.** Il ciclo è "proponi → vieni smentito → affina", non "indovina o sbagli". **Applicazione:** una smentita dovrebbe **restringere il campo**, non chiudere il discorso — la differenza tra *"ti sbagliavi"* e *"l'evidenza esclude questa forma di relazione"*. Da tenere presente quando si scriverà il testo delle smentite.

### Da Understand (2021) — il campanello d'allarme

Puzzle sulla deduzione di regole nascoste; le versioni recenti hanno aggiunto generazione casuale delle regole. Osservazione riportata da chi l'ha giocato: **la generazione alza il pavimento dell'esperienza ma abbassa il soffitto.**

Tradotto: la matrice generata garantisce che nessuna run sia banale, ma nessuna raggiungerà l'eleganza di un puzzle progettato a mano. È un compromesso già accettato consapevolmente — ma **spiega perché il contorno (eventi, biomi con carattere, pilastro 5) non è decorazione: è ciò che compensa il soffitto più basso della generazione procedurale.** Conferma indipendente che il pilastro 5 non era un capriccio.

### Da Return of the Obra Dinn — la tentazione da evitare

Conferma le deduzioni a gruppi di tre, per impedire il brute-force senza dare feedback immediato. **Da non copiare:** il sistema di evidenza pesata già risolve lo stesso problema in modo più coerente col tema. Citato perché è la soluzione ovvia verso cui si scivola facilmente.

---

## Collocazione nel piano di lavorazione

**Il test dei due bot va eseguito in Fase 1**, insieme al checkpoint di playtest già previsto (`abiogenesis-INDEX.md`) — è headless, non richiede interfaccia, e misura una proprietà del bilanciamento che diventa molto più costosa da correggere una volta costruito tutto il resto sopra.

**Le due domande diagnostiche** (taccuino esterno, esperimento non fatto per mancanza di budget) vanno aggiunte al protocollo del playtest di Fase 1 e a quello di Fase 1b (`culture-shock-friction-fixes.md`).

## Fuori scope

- Implementazione concreta dei due bot (euristiche precise, criteri di scelta) — qui solo la struttura del test.
- Valori numerici di eventuali correzioni — dipendono dall'entità dello squilibrio misurato, non decidibili in anticipo.
- Il riesame del "nessun costo per sbagliare" nelle ipotesi dichiarate — segnalato come da riconsiderare, non deciso.
