# Culture Shock — eventi di mondo: firme di bioma e origine cosmica

Documento autonomo, terzo capitolo della serie sugli eventi di mondo (dopo `abiogenesis-world-events.md` ed `abiogenesis-world-events-catastrophes.md`) e diretta applicazione del pilastro 5 (`culture-shock-wonder.md`). Specifica trigger, meccanismo e dipendenze per ciascuna proposta — pensato per essere preso e implementato, non solo discusso.

## Come si armonizza col resto

- **Nessun sistema nuovo per principio:** ogni evento qui riusa la pipeline del tick (`abiogenesis-tick-pipeline.md`), il meccanismo di biomi dinamici (`abiogenesis-world-events-catastrophes.md`), le famiglie di tratti (`abiogenesis-tag-archetypes.md`) o il meccanismo del Precursore (`abiogenesis-world-events.md`) — mai un calcolo nuovo scollegato.
- **Frequenza agganciata al tier di mondo**, coerente col principio "manopola singola" (`abiogenesis-system-hierarchy.md`).
- **Nessun evento rivela mai la famiglia dominante del mondo o un tratto condizionato dal terreno** prima che il giocatore lo osservi da sé — stesso vincolo già applicato allo strumento di ispezione (`culture-shock-inspect-tool.md`).
- **Le famiglie di tratti citate qui presuppongono il pool a 15+10 già formalizzato** in `abiogenesis-tag-archetypes.md`, non i 25 originali.

---

## Parte 1 — Eventi di mondo generici (oltre il catalogo esistente)

### Bloom sincronizzato multi-specie

**Cosa:** più specie diverse si riproducono nello stesso tick su un'area ampia, per puro allineamento statistico dei rispettivi trigger di riproduzione — non un evento scriptato, un pattern notato quando emerge.

**Trigger:** N popolazioni distinte (proposta: N≥3) che superano la soglia di riproduzione entro la stessa finestra di pochi tick, in celle reciprocamente vicine.

**Presentazione:** candidato di reveal a tier notevole; visivamente, un fronte di densità che si espande in poche stagioni, osservabile dall'overview.

**Dipendenza:** nessuna nuova — è un pattern-match sopra eventi di riproduzione già emessi dalla fase 5 della pipeline del tick.

### Silenzio anomalo

**Cosa:** un'area smette di produrre eventi osservabili per molte ere pur restando viva — energia stabile, nessuna interazione, nessuna pressione. Non un difetto: un evento in sé.

**Trigger:** una regione con popolazione viva e stabile (nessuna nascita, morte, o soglia superata) per un numero di ere sopra soglia (proposta: 3 ere piene).

**Presentazione:** una singola riga in Cronaca, tono deliberatamente enigmatico — *"quest'area non produce eventi da 3 ere. Nessuna causa nota."* Coerente col "mistero mai risolto" (`culture-shock-wonder.md`) se persiste oltre un limite superiore, altrimenti si risolve da sé quando qualcosa accade.

**Dipendenza:** contatore di ere consecutive senza eventi candidato-evento per regione — nuovo, ma leggero (un contatore, non un sistema).

### Corrente di residuo

**Cosa:** il residuo organico non resta fermo, ma si accumula lentamente lungo un gradiente, creando una scia che attraversa più biomi e una nicchia da decompositori che si sposta nel tempo invece di restare fissa.

**Trigger:** nessun evento puntuale — è una modifica strutturale al modello di decadimento/diffusione del residuo, già menzionato come modulabile per bioma in `abiogenesis-tick-pipeline.md` (fase 4).

**Livello:** variazione strutturale, non payoff raro — è una regola ambientale continua, non un evento discreto.

---

## Parte 2 — Firme specifiche per bioma

Oggi i 16 biomi (`abiogenesis-biomes.md`) si distinguono solo per temperatura/luce/tossicità di base — nessun comportamento proprio. Questa sezione dà a un sottoinsieme un evento-firma riconoscibile, riusando meccanismi già esistenti.

### Palude — fermentazione tossica (ciclo auto-inflitto)

**Trigger:** popolazione di decompositori sopra una soglia di densità locale, sostenuta per più tick, in una cella o area di bioma Palude.

**Meccanismo:** la tossicità locale non resta un valore fisso del bioma ma **cresce** in presenza della popolazione stessa — un ciclo che può, se non controllato, soffocare la popolazione che l'ha causata. Diverso da qualunque altro bioma: è l'unico dove una specie può letteralmente avvelenare la propria nicchia.

**Dipendenza:** modifica del valore di tossicità per cella in funzione della densità locale di decompositori — aggancio nella fase 1/2 della pipeline (dove si legge lo scalare ambientale), con scrittura di ritorno verso lo scalare stesso.

### Distesa di cristalli — risonanza

**Trigger:** popolazione di chemiolitotrofi che vive per un numero di ere sopra soglia in un bioma Distesa di cristalli.

**Meccanismo:** aumenta leggermente la probabilità che un evento di soglia superata (§5.11 GDD) per quella popolazione produca uno **xenotratto** invece di un edit standard del genoma — riusa direttamente l'origine meccanica degli xenotratti già definita (`abiogenesis-tag-archetypes.md`: probabilità bassa e separata su ogni `SelectionThresholdCrossed`), qui semplicemente modulata al rialzo dal bioma invece di restare costante ovunque.

**Effetto:** dà al bioma più esotico del roster un ruolo meccanico specifico, non solo una tinta diversa.

### Vetta — letargo

**Trigger:** temperatura locale sotto soglia critica, sostenuta.

**Meccanismo:** la popolazione non muore né cresce — entra in uno stato di sospensione: energia congelata, nessun guadagno né perdita, fino a che le condizioni non migliorano, momento in cui riprende automaticamente. Introduce un terzo stato oltre a "cresce"/"declina": **pausa reversibile**.

**Presentazione:** il glifo pixel della popolazione si mostra visivamente sbiadito/statico mentre è in letargo — coerente con lo stile pieno/vuoto già stabilito, una terza variante intermedia.

**Dipendenza:** un flag di stato per popolazione ("in letargo"), che la fase 5 della pipeline controlla prima di applicare guadagni/perdite.

### Bocca vulcanica / Geyser — imprinting termico

**Trigger:** una speciazione (§5.11 GDD) che avviene in una cella di sorgente di calore.

**Meccanismo:** probabilità aumentata che la nuova specie erediti un tratto della famiglia **Riserva/energia** — coerente col fatto che questi biomi sono già pensati come sorgenti puntiformi di energia (`abiogenesis-biomes.md`).

**Dipendenza:** un bias di famiglia condizionato dal bioma della cella di origine, nella fase di generazione del genoma per speciazione — variante locale dello stesso bias di famiglia dominante già esistente a livello di mondo (`abiogenesis-tag-archetypes.md`), qui applicato punto per punto invece che globalmente.

### Cratere — cicatrice (puro flavor)

**Trigger:** una specie vive nel Cratere per un tempo prolungato.

**Meccanismo:** nessuno — solo un campo testuale nel Catalog ("cresciuta nell'ombra del Precursore"). Zero meccanica, rinforza che il luogo significa qualcosa senza costare nulla in bilanciamento.

**Dipendenza:** un flag persistito per specie, stesso pattern già usato per "discende da" (`culture-shock-notebook-cronaca.md`).

---

## Parte 3 — Origine cosmica

L'impatto da bolide (già in `abiogenesis-world-events-catastrophes.md`) resta l'evento cosmico distruttivo di base. Qui la varietà sotto la stessa origine.

### Pioggia di micrometeoriti — **stesso sistema delle tasche di anomalia sparse**

**Nota di armonizzazione, non duplicare lavoro:** questo è **esattamente** il meccanismo "tasche di anomalia sparse" già proposto in `culture-shock-wonder.md` (priorità alta), qui semplicemente giustificato con un'origine narrativa cosmica invece che geologica generica. Un solo sistema da costruire, due modi di raccontarlo nel testo generato — la scelta di quale narrazione usare può anche essere casuale per istanza, non richiede una decisione a monte.

### Raggio cosmico

**Trigger:** evento raro a probabilità bassa, scalata col tier di mondo.

**Meccanismo:** per una stagione, aumenta la probabilità che un evento di soglia superata tocchi specificamente la famiglia **Genetico** (prioni, plasmidi, ribozimi, trasposoni, epigenoma — già evocano instabilità nel nome). Prima volta che un evento cosmico ha un bersaglio tematicamente coerente con la sua causa fisica reale (le radiazioni cosmiche danneggiano il materiale genetico).

**Dipendenza:** stesso bias di famiglia condizionato già usato per l'imprinting termico (Parte 2), qui applicato a livello di mondo e a tempo invece che per cella.

### Grande oscuramento

**Trigger:** evento raro, durata estesa (proposta: 3-5 ere).

**Meccanismo:** la luce cala **gradualmente** su tutta la mappa nel corso dell'evento, non di colpo — a differenza di ogni altro evento ambientale proposto finora, che sono shock istantanei o condizioni sostenute su una cella. Qui la minaccia è globale e lenta, visibile arrivare da lontano se il giocatore osserva il trend nel tempo, mettendo sotto pressione ogni popolazione fotolitica contemporaneamente.

**Dipendenza:** modifica temporanea e globale dello scalare luce, con rampa di transizione (non un valore che cambia istantaneamente) — verificare la compatibilità con la diffusione ambientale già prevista (§5.2 GDD, Phase 1+).

### Relitto

**Trigger:** eventualmente unico per l'intera vita del gioco per mondo (proposta: 1 mondo su molti, più raro del Precursore stesso).

**Meccanismo:** un oggetto di origine non identificata atterra intatto, diventando una **seconda** anomalia fissa nel mondo — stessa implementazione del Precursore (una cella con chimica anomala costante, `abiogenesis-world-events.md`), ma distinta e senza alcuna spiegazione mai data. Puro seme di "mistero mai risolto" (`culture-shock-wonder.md`).

**Dipendenza:** nessuna nuova — riusa l'implementazione del Precursore, solo con probabilità di comparsa molto più bassa e senza il testo esplicativo che il Precursore potrebbe avere.

---

## Priorità

| Livello | Voce | Perché |
|---|---|---|
| **Alta** | Pioggia di micrometeoriti | Stesso sistema di "tasche di anomalia sparse" già priorità alta in `culture-shock-wonder.md` — zero lavoro aggiuntivo |
| **Alta** | Firme di bioma (Palude, Vetta) | Rapporto costo/effetto migliore: danno personalità meccanica a un roster di 16 biomi che oggi si distingue solo per tre numeri |
| Media | Distesa di cristalli — risonanza, Bocca vulcanica/Geyser — imprinting | Riusano meccanismi già esistenti (xenotratti, bias di famiglia) applicati punto per punto — costo contenuto, effetto più di nicchia |
| Media | Bloom sincronizzato, Silenzio anomalo | Pattern-match su eventi già emessi, nessun nuovo calcolo, ma effetto più discreto |
| Bassa-media | Cratere — cicatrice | Puro flavor, costo minimo ma impatto anch'esso minimo |
| Bassa-media | Raggio cosmico, Grande oscuramento | Buone idee, ma richiedono modifiche più ampie (scalari globali, rampe temporali) |
| Payoff raro, non prioritario ora | Relitto, Corrente di residuo | Il primo per rarità intenzionale, il secondo per essere una modifica strutturale non urgente |

## Cosa serve per l'integrazione

- **Contatore "ere senza eventi" per regione** (Silenzio anomalo) — nuovo, leggero.
- **Scrittura di ritorno sullo scalare tossicità** in funzione della densità locale (Palude) — la fase 1/2 della pipeline oggi legge gli scalari, qui deve anche poterli modificare.
- **Bias di famiglia locale**, generalizzazione per-cella del bias già globale (Distesa di cristalli, Bocca vulcanica/Geyser, Raggio cosmico) — stesso meccanismo, tre punti di applicazione diversi (per bioma alla speciazione, per bioma alla soglia xenotratto, per mondo a tempo).
- **Flag di stato "in letargo" per popolazione** (Vetta), controllato prima delle fasi 2-5 della pipeline.
- **Campo testuale persistito "cicatrice"** per specie (Cratere) — stesso pattern di "discende da".
- **Rampa temporale per scalari globali** (Grande oscuramento) — non esiste oggi un precedente di modifica graduale-nel-tempo di uno scalare su tutta la mappa, va verificato contro la diffusione ambientale esistente.
- **Seconda istanza del meccanismo Precursore** (Relitto) — riuso diretto, solo una nuova soglia di rarità.

## Fuori scope

- Valori numerici esatti di tutte le soglie indicate (densità, ere, probabilità) — indicativi, da validare in playtest come il resto del bilanciamento.
- Testo generato per ciascun evento — qui solo trigger e meccanismo, il testo effettivo passa dal sistema di generazione narrativa (`abiogenesis-narrative-generation.md`).
- Se e come combinare più firme di bioma nello stesso mondo — non specificato, presumibilmente indipendenti tra loro.
