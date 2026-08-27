# Culture Shock — meraviglia e scoperta (pilastro 5)

Documento autonomo. Formalizza il quinto pilastro di design aggiunto al GDD ("un senso costante di meraviglia e scoperta") e propone contenuti concreti — piccoli, medi, grandi, stranissimi — pensati per rinforzarlo. Tutto resta dentro il vincolo hard-SF già stabilito: nessuna fantasia, nessuna magia, solo biochimica ed estrapolazione plausibile.

## Il pilastro, e come va usato

**Non è una lista di feature una tantum — è un filtro da applicare a ogni proposta futura, piccola o grande:** questa cosa dà al giocatore un motivo in più per pensare *"voglio vedere cos'altro c'è"*, o è solo funzionale? Le correzioni di leggibilità/attrito (`culture-shock-friction-fixes.md`) restano necessarie ma rispondono a un obiettivo diverso e complementare — vanno tenuti vivi entrambi, non ottimizzato solo l'uno a scapito dell'altro.

Il riferimento esplicito, per calibrare il tono: la sensazione di Terraria e RimWorld — non un singolo colpo di scena, ma una superficie costante di piccole sorprese che fa venire voglia di continuare a giocare "solo un altro po'".

---

## Piccole — economiche, frequenti, quasi gratis

### Micro-descrizioni uniche per organismi notevoli
Il sistema di generazione narrativa (`abiogenesis-narrative-generation.md`) guadagna una coda di frammenti aggettivali rari, riservata a occorrenze eccezionali (popolazione insolitamente longeva, lineage con catena di speciazioni particolarmente lunga): *"una colonia inspiegabilmente resiliente"*. Nessun sistema nuovo, solo un pool di frammenti in più nello stesso generatore esistente.

**Livello:** variazione strutturale — riusa un sistema già core.

### Piste false
Un'osservazione anomala, isolata, che sembra significativa ma non lo è — un evento senza causa ricostruibile, mai ripetuto in quella run. Non un bug da correggere: insegna scetticismo scientifico per esperienza diretta, coerente con l'identità "il metodo scientifico come esperienza emotiva" (`culture-shock-identity.md`).

**Livello:** payoff raro. **Vincolo:** deve restare genuinamente raro — se troppo frequente, il giocatore impara a ignorare ogni segnale anomalo, il che danneggerebbe anche i segnali veri.

### Segno visivo minimo su un individuo eccezionale
Una variazione minuscola sul glifo pixel (cfr. `culture-shock-population-model-aesthetic.md`) per una popolazione con una storia particolare — lineage lunga, sopravvissuta a pressione estrema. Puro fiocco cosmetico, nessun effetto meccanico.

**Livello:** variazione strutturale — estensione della libreria di glifi pixel già decisa.

### Un suono raro, mai spiegato
Un cue sonoro distintivo (cfr. `abiogenesis-audio.md`) legato a una condizione rarissima, mai etichettato esplicitamente a schermo — pura curiosità uditiva per chi lo nota.

**Livello:** payoff raro.

---

## Medie — un po' di lavoro, effetto duraturo

### Tasche di anomalia sparse
Oltre al Precursore (singolo punto fisso per mondo, `abiogenesis-world-events.md`), alcune **tasche minori**, più piccole e più numerose, sparse sulla mappa — ciascuna con una minuscola deviazione locale della matrice (una singola coppia di tratti che lì si comporta diversamente dal resto del mondo). Trasforma l'esplorazione dell'intera griglia 128×80 in qualcosa che vale la pena fare, non solo il centro dove si è seminato.

**Livello:** variazione strutturale, vicino al payoff raro per densità (poche per mondo).
**Dipendenza:** stesso meccanismo del Precursore, generalizzato a N istanze più deboli invece di una sola forte.

### Tracce fossili
Aggancio concreto e piccolo al registro stratigrafico (menzionato come futuro in `abiogenesis-system-hierarchy.md`, livello 3): quando una lineage si estingue del tutto, la cella dove è vissuta più a lungo lascia un piccolo marker ispezionabile — non un intero sistema di memoria per cella, solo un'eco leggibile al click, riusando lo strumento di ispezione già costruito (`culture-shock-inspect-tool.md`).

**Livello:** variazione strutturale.
**Dipendenza:** un solo campo persistito per cella (lineage più longeva mai vissuta lì), non l'intero registro stratigrafico completo.

### Catalogo curiosità trasversale ai mondi
Estensione leggera del Codex minimale già proposto post-MVP (`abiogenesis-transitions-metaprogression.md`, Parte 2): non solo specie/biomi incontrati, ma eventi rari e anomalie viste almeno una volta. Motore dello stesso istinto collezionistico che rende Terraria compulsivo.

**Livello:** post-MVP, stesso motivo del Codex che estende.

---

## Grandi — costose, alta cerimonia

### Deriva morfologica visibile su scale lunghissime
Una lineage che sopravvive per moltissime ere accumula variazioni visive cumulative sul proprio glifo — non solo dati nel notebook, una forma che cambia lentamente e visibilmente sulla mappa. Premio per la pazienza estrema, indipendente dall'Emersione ma concettualmente imparentato.

**Livello:** payoff raro, costoso — richiede una libreria di varianti di glifo più ampia di quella base.

### Il mondo che si contraddice tra run
Già proposto (2.3/2.5 in `abiogenesis-system-hierarchy.md`) — il payoff più coerente con l'ispirazione hard-SF di tutto il lotto: scoprire che una regola creduta universale non lo è.

**Livello:** payoff raro, il più ambizioso del gruppo — resta futuro, non MVP.

---

## Stranissime — il salto immaginativo richiesto esplicitamente

### Un mistero che il gioco non risolve mai
Un'anomalia per mondo, rarissima, **senza alcuna spiegazione meccanica** — mai collegata a nessun sistema, mai etichettata. Non tutto deve avere una causa scopribile: è il punto più vicino allo spirito hard-SF di tutta questa lista.

**Livello:** payoff raro, il più delicato da bilanciare — deve restare eccezionale (un mondo su molti), altrimenti il giocatore la classifica come rumore e smette di darle peso, danneggiando anche i segnali reali.

### Metabolismo estremofilo, reso concreto
Aggancio meccanico finalmente vero per l'icona già riservata come futura (`abiogenesis-cross-cutting.md`/set icone specie): un organismo che non tollera ma **richiede** condizioni estreme (bocca vulcanica, vetta) per sopravvivere affatto — capovolge la logica di nicchia standard. Un capovolgimento concettuale forte a basso costo di sistema, dato che i biomi estremi esistono già.

**Livello:** variazione strutturale — riusa biomi e formula esistenti, aggiunge solo un metabolismo con requisito invertito.

### Superorganismo multi-cella, oltre l'Emersione
Estensione speculativa dell'Emersione (`culture-shock-emersione.md`): una lineage abbastanza evoluta non collassa in una singola cella ma in una struttura che occupa più celle contigue, un unico glifo esteso sulla griglia.

**Livello:** esplicitamente "fase due" — fuori scope anche per l'Emersione stessa, solo annotato come direzione.

### Diffrazione di segnale su scala di mondo
Estensione rara dell'"eco di segnale inspiegabile" (`abiogenesis-world-events-catastrophes.md`): per un breve periodo, l'**intera griglia** mostra una correlazione visiva coordinata sui tratti di segnalazione, come se qualcosa attraversasse il mondo intero simultaneamente. Nessun effetto meccanico, puro spettacolo raro — su 128×80 celle visivamente notevole dall'overview.

**Livello:** payoff raro, generalizzazione a scala di mondo di un evento già proposto puntuale.

---

## Priorità — le tre da spingere per prime

Criterio: massimo effetto sul pilastro 5, minimo costo di sistema, massima leva su cose già decise.

1. **Tasche di anomalia sparse** — rende l'intera mappa degna di esplorazione, non solo il centro seminato. Generalizza un meccanismo già esistente (Precursore).
2. **Tracce fossili** — aggancio economico e concreto a un sistema oggi solo menzionato come futuro, un solo campo dati.
3. **Metabolismo estremofilo concretizzato** — chiude un cerchio già aperto (icona riservata da tempo), costo minimo perché biomi e formula esistono già.

Le altre restano valide come direzione, non urgenti.

## Cosa serve per l'integrazione

- **Tasche di anomalia:** generalizzare il meccanismo del Precursore a N istanze deboli invece di una sola forte — stessa logica, parametro di densità/forza diverso.
- **Tracce fossili:** un campo persistito per cella (lineage più longeva mai vissuta lì) più un piccolo ramo nell'ispezione di cella vuota (`culture-shock-inspect-tool.md`) per mostrarlo.
- **Estremofilo:** requisito ambientale invertito nella formula del tick (§5.6 GDD) — energia negativa o nulla fuori dai biomi estremi, non solo ridotta.
- Tutto il resto: dipendenze già segnalate voce per voce sopra, nessuna nuova finché non si decide di procedere.

## Fuori scope

- Bilanciamento numerico di frequenza/rarità per ciascuna proposta — da validare in playtest, con particolare cautela sulle voci esplicitamente segnalate come delicate (piste false, mistero mai risolto).
- Le voci "grandi" e la maggior parte delle "stranissime" — direzione concettuale, non specifica pronta per l'implementazione.
- Superorganismo multi-cella — esplicitamente fase due, oltre l'orizzonte di questo documento.
