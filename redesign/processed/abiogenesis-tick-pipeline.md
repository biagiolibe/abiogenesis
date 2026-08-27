# Abiogenesis — specifica della pipeline del tick

Documento autonomo per un task di implementazione. Riscrive §5.6 del GDD in forma di **pipeline a fasi con output espliciti**, invece che come singola formula per l'energia. Non cambia la matematica esistente: la riorganizza e indica dove agganciare i sistemi introdotti dopo (biomi, evoluzione, osservazioni, eventi).

## Perché non basta ritarare i coefficienti

§5.6 è scritta come se il tick producesse **un solo output**: l'energia aggiornata. Con tutto ciò che è stato aggiunto, un tick deve in realtà produrne **tre**, e tutti e tre si nutrono degli stessi valori intermedi:

1. **Energia aggiornata** — la formula attuale.
2. **Pressione selettiva accumulata** (§5.11) — oggi descritta come sistema separato, ma i suoi tre stimoli (danno da interazione, disallineamento ambientale, esposizione a tossicità) sono esattamente valori già calcolati per l'energia.
3. **Osservazioni ed eventi emessi** — oggi impliciti. Il notebook ha bisogno delle adiacenze osservate col loro peso; il sistema di reveal ha bisogno di candidati-evento da classificare; `Cull` come knockout deve generare un'osservazione.

**Il rischio concreto se restano sistemi separati:** gli stessi valori verrebbero ricalcolati in tre punti diversi del codice, con possibilità reale di divergere. È lo stesso rischio già segnalato nel documento sulla generazione narrativa — notebook e narrazione che raccontano versioni incoerenti dello stesso evento. Un'unica pipeline con tre uscite lo elimina per costruzione.

## La pipeline

Per ogni organismo, in ordine. I nomi tra parentesi sono i valori intermedi che vanno **conservati e passati alle fasi successive**, non ricalcolati.

### Fase 0 — Gate di abitabilità *(nuovo)*

Il bioma della cella può rendere la cella **inabitabile** per l'organismo, indipendentemente dagli scalari. Se il gate fallisce, l'organismo muore (causa: habitat) e la pipeline si ferma qui per lui.

Serve perché acqua profonda non è "poca luce e poco freddo": è un posto dove un organismo terrestre non può stare, punto. È un vincolo binario che oggi non ha alcuna espressione nella formula.

→ produce: `habitat_ok` (bool)

### Fase 1 — Adattamento ambientale

Invariata rispetto a §5.6: `env_fit` da temperatura (e in prospettiva dagli altri scalari) rispetto al genoma della specie.

→ produce: `env_fit`, e **`env_mismatch`** (quanto l'organismo è fuori dal proprio optimum) — quest'ultimo è già uno dei tre stimoli di pressione selettiva, va conservato invece di essere ricalcolato in §5.11.

### Fase 2 — Guadagno metabolico

Invariata: `gain = risorsa × metabolism_gain × env_fit`, con la risorsa che dipende dal metabolismo (luce / vicini / residuo / tossicità).

**Il bioma non entra qui.** Un bioma *è già* una combinazione di scalari — una palude è alta tossicità, un deserto è luce e temperatura alte. Aggiungere un moltiplicatore di bioma sopra gli scalari sarebbe un doppio conteggio della stessa informazione, e renderebbe impossibile tarare l'uno senza rompere l'altro.

→ produce: `gain`

### Fase 3 — Interazione della matrice

Invariata nella sostanza: somma degli `interaction_delta` dai vicini, moltiplicata per il coefficiente di scala (vedi documento sul bilanciamento — da verificare se esista già in build).

**Punto di aggancio per `Isola`:** se la cella è in stato protetto, questa fase viene saltata (contributo zero). Oggi non esiste un punto dove agganciarlo — va previsto qui, non altrove.

→ produce: `interaction_delta`, e separatamente **`interaction_harm`** (la sola componente negativa) — secondo stimolo di pressione selettiva, da conservare.

### Fase 4 — Costi

`upkeep` (per metabolismo) + affollamento.

**Il bioma entra qui, non nella fase 2.** Due agganci proposti:
- **`crowd_factor` modulato dal bioma** — oggi è una costante globale; ha senso che una foresta regga più densità di una vetta. Questo è un *costo*, non un guadagno: nessun doppio conteggio con gli scalari.
- *(collegato, fuori dalla pipeline dell'organismo)* **tasso di decadimento del residuo modulato dal bioma** — un lago trattiene, un pendio disperde. Dà finalmente un motivo meccanico per cui certi biomi sono nicchie da decompositori, invece di esserlo solo per gli scalari.

→ produce: `upkeep_total`, `crowd_cost`

### Fase 5 — Aggiornamento energia

`energy += gain + interaction_delta − upkeep_total − crowd_cost`, poi valutazione di morte per fame e riproduzione oltre `repro_threshold`.

→ produce: `energy'`, eventi di nascita/morte **con causa** (fame / interazione / habitat / temperatura) — la causa è già richiesta dal GDD per il death log, e serve al ranking degli eventi (fase 7).

### Fase 6 — Accumulo di pressione selettiva

Usa i valori **già calcolati sopra**: `interaction_harm` (fase 3), `env_mismatch` (fase 1), esposizione a tossicità (fase 1/2 a seconda del metabolismo). Nessun ricalcolo.

→ produce: `pressure'`, ed eventuale `SelectionThresholdCrossed`

**Da tenere separata dall'energia, deliberatamente.** Sarebbe tecnicamente possibile fondere energia e pressione in un unico accumulatore, ma sono due orologi con significati diversi: l'energia è sopravvivenza a breve termine, la pressione è adattamento a lungo termine. Tenerli distinti è ciò che permette a un organismo di sopravvivere benissimo mentre accumula pressione (o viceversa) — che è precisamente la tensione interessante. **Condividono gli input, non devono condividere il valore.**

### Fase 7 — Emissione di osservazioni ed eventi

Unico punto dove il tick "parla" ai sistemi di superficie:

- **Osservazioni per il notebook:** ogni adiacenza tra organismi con tratti, col peso `1/(1+confondenti)` già definito (§7 GDD). Il conteggio dei confondenti è disponibile dalla fase 3, non va ricostruito.
- **Candidati-evento per il reveal:** nascite/morti con causa, soglie superate, estinzioni. Il ranking (grandezza × pulizia del segnale) resta nel sistema di generazione narrativa, ma **i candidati e i loro pesi nascono qui**.
- **Osservazione da `Cull`:** quando un'azione di rimozione avviene vicino ad altri organismi, l'osservazione risultante entra da questa fase come tutte le altre — non da un percorso separato.

→ produce: lista di osservazioni, lista di candidati-evento

## Cosa NON cambia

- La matematica di `env_fit`, `gain`, `interaction_delta`, `upkeep`, affollamento, riproduzione: identica a §5.6.
- L'ordine di processamento degli organismi (shuffle seedato o double buffering) e le garanzie di determinismo (§5.7).
- I tre stimoli di pressione selettiva e la loro soglia (§5.11) — cambia solo *da dove* prendono i valori.

## Cosa serve per l'integrazione

- **Struttura dati per i valori intermedi:** le fasi devono poter passare `env_mismatch`, `interaction_harm`, conteggio confondenti alle fasi successive senza ricalcolo. È la modifica strutturale principale.
- **Gate di abitabilità per bioma:** tabella bioma → abitabile/non abitabile (eventualmente per metabolismo, se in futuro un metabolismo acquatico avrà senso).
- **`crowd_factor` per bioma:** da costante globale a lookup per bioma.
- **Tasso di decadimento residuo per bioma:** stessa forma.
- **Stato "cella protetta"** per `Isola`, valutato in fase 3 — solo se quell'azione viene adottata.
- **Causa di morte propagata** fino alla fase 7 — probabilmente già presente per il death log, da verificare che arrivi fino agli eventi.

## Fuori scope

- Valori numerici dei nuovi parametri per bioma (abitabilità, `crowd_factor`, decadimento residuo) — da definire insieme al roster biomi e da validare in playtest.
- Il coefficiente di scala sull'`interaction_delta` — trattato nel documento sul bilanciamento, qui solo indicato come punto della pipeline.
- Decisione sull'adozione di `Isola` — qui solo previsto il punto di aggancio nel caso.
