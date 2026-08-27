# Abiogenesis — eventi di mondo: catastrofi, non-catastrofi, biomi dinamici

Documento autonomo, secondo giro di brainstorm sugli eventi di mondo (il primo è in `abiogenesis-world-events.md`). Qui il tema è più ampio: cosa può succedere in un mondo alieno vivo, esplorato liberamente, con un filo conduttore emerso durante la discussione — alcuni eventi non dovrebbero limitarsi a cambiare numeri, dovrebbero **cambiare la mappa stessa**.

## Il filo conduttore: biomi dinamici

Finora i biomi (documento biomi, GDD §5.10) sono generati una volta e restano fissi per tutta la partita. Diversi eventi proposti sotto richiedono che un bioma possa **nascere o trasformarsi durante il gioco**. Prima di elencare gli eventi, la meccanica che li rende possibili:

### Trigger
Una condizione ambientale sostenuta **oltre soglia per più ere consecutive**, mai un singolo tick — coerente col principio "niente scatta all'istante" già applicato all'Emersione (cfr. documento dedicato) e alla scala temporale a tre livelli.

### Transizione visibile, non sfumata
Nel mezzo della trasformazione, la cella **non deve mai mostrare un colore "fuso"** tra i due biomi — coerente col principio già stabilito nel documento sullo stile grafico (colore piatto, mai blend). La cella in transizione porta invece un **bordo tratteggiato distintivo**, la stessa grammatica già usata per "non ancora osservato" nel notebook. Il colore resta netto fino al completamento, cambia di scatto solo quando la trasformazione si conclude.

### Reversibilità differenziata
- **Eventi istantanei** (impatto, sisma) → permanenti, nessuna marcia indietro.
- **Eventi a condizione sostenuta** (risveglio vulcanico, cristallizzazione, fronte glaciale) → **reversibili finché non si completano**: se la condizione che li alimenta si interrompe prima del completamento, la cella torna gradualmente al bioma originale invece di restare a metà per sempre. Dà al giocatore una leva reale (es. raffreddare artificialmente una zona per fermare un vulcano nascente), non solo uno spettacolo passivo.

### Integrazione obbligata con i tratti condizionati dal terreno
Se una cella cambia bioma, qualunque tratto attivato da quel bioma (GDD §5.5) deve disattivarsi/riattivarsi di conseguenza. È il punto di integrazione più delicato di questa proposta — va segnalato esplicitamente a chi implementa, non è ovvio finché non emerge come bug.

### Aggancio al reveal
Il completamento di una trasformazione di bioma è candidato naturale per il reveal di tier alto (cfr. documento sulla scala temporale) — riusa il sistema già scritto, nessun lavoro aggiuntivo.

---

## Eventi catastrofici

### Impatto cosmico
Un frammento cade sulla mappa — tossicità/luce alterate localmente, e la cella d'impatto diventa un nuovo Cratere dove prima non esisteva. Evento istantaneo, permanente.

### Risveglio vulcanico
Una zona (pianura o roccia) comincia a scaldarsi progressivamente nel corso di più ere, fino a diventare una Bocca vulcanica. Trasformazione a condizione sostenuta, reversibile finché non si completa — il giocatore vede arrivare la minaccia tramite indizi ambientali crescenti, non di sorpresa.

### Bloom tossico auto-inflitto
Non un evento esterno: conseguenza delle scelte del giocatore. Troppi decompositori concentrati producono un accumulo di residuo che supera soglia e degenera in tossicità diffusa. Unico tipo di catastrofe riconducibile a una scelta del giocatore, non al mondo — coerente col principio di agency già scritto nel documento di consolidamento.

### Tempesta magnetica
Colpisce specificamente i tratti della famiglia Segnalazione (QRM, FLG, REC — cfr. documento archetipi biochimici), sopprimendo temporaneamente la loro efficacia su tutta la mappa. Primo evento proposto che colpisce **una famiglia di tratti** invece dell'ambiente in generale — apre un genere di eventi "mirati a una famiglia", che si aggancerebbe bene al bias di famiglia dominante (un mondo a dominanza genetica potrebbe subire più spesso eventi mirati a quella famiglia, per coerenza tematica).

### Glaciazione in avanzata
Un fronte freddo che si propaga da un bordo della mappa verso il centro nel corso di più ere — non un valore che cambia ovunque insieme, un fronte che si sposta e che il giocatore può vedere avvicinarsi, con tempo per reagire.

### Sconvolgimento sismico
Rialza o abbassa una regione — una pianura può diventare collina, una collina montagna. Come il risveglio vulcanico, ridisegna le fasce di elevazione stesse, non solo l'ambiente sopra di esse. Evento istantaneo, permanente.

## Eventi non-catastrofici, neutri o positivi

### Era di quiete
Non tutte le ere devono avere un evento — e il "niente è successo" va narrato esplicitamente dal sistema di generazione narrativa (cfr. documento dedicato), non lasciato come silenzio. Il contrasto tra ere di quiete ed eventi veri è ciò che dà peso a questi ultimi — se ogni era ha sempre qualcosa da raccontare, nulla pesa più delle altre cose.

### Fioritura stagionale
Un picco di luce globale temporaneo, positivo per i fotolitici — l'equivalente "buono" di un bloom tossico, stesso meccanismo di picco ambientale con segno opposto.

### Migrazione
Nuove wild species compaiono a metà partita in un bioma specifico, non solo alla generazione del mondo — estende il concetto di specie indigene (GDD §9) oltre il solo world-gen iniziale, dando alla scoperta un secondo momento nel corso della partita.

### Cristallizzazione
In una zona ad alta tossicità sostenuta nel tempo, la tossicità comincia lentamente a "solidificarsi" in depositi minerali — la cella evolve verso una Distesa di cristalli. Trasformazione a condizione sostenuta, reversibile finché non si completa.

### Eco di segnale inspiegabile
Evento puramente atmosferico, **senza alcun effetto meccanico**: per un'era, i tratti di Segnalazione in tutta la mappa mostrano un comportamento correlato che nessuna interazione locale spiega — nessuna causa nella matrice, nessun bioma coinvolto. Puro mistero senza soluzione meccanica, indizio che il Precursore (o qualcosa di simile) è presente nel mondo anche dove non è visibile direttamente. **Rischioso se abusato** (il giocatore potrebbe cercare invano una spiegazione inesistente) — va limitato a **al massimo una volta per mondo**, se mai implementato.

### Quiescenza genetica
Un periodo in cui la pressione evolutiva accumulata si raffredda più velocemente del normale — l'opposto di un acceleratore di speciazione, dà respiro invece di spingere.

---

## Perché i biomi dinamici sono la proposta più significativa di questo elenco

Impatto, risveglio vulcanico, sconvolgimento sismico e cristallizzazione condividono tutti lo stesso meccanismo di fondo (sopra). Insieme trasformano i biomi da "sfondo generato una volta" a **storia che si scrive mentre si gioca** — si agganciano naturalmente a quasi tutto il resto già proposto: il registro stratigrafico (2.1, una cella che ricorda), il momento della rivelazione, il reveal di fine era. Se dovesse esserci una sola cosa da approfondire per prima tra tutte quelle in questo documento, è questa: non tanto per la sua complessità di implementazione (contenuta, è un solo meccanismo condiviso da 4 eventi), quanto per l'impatto che ha su come si sente l'intero gioco nel tempo.

## Cosa serve per l'integrazione

- **Meccanismo di transizione bioma condiviso**, non uno per evento — impatto/sisma/vulcano/cristallizzazione riusano la stessa logica (trigger a soglia sostenuta o istantanea, bordo tratteggiato in transizione, reversibilità differenziata).
- **Verifica di disattivazione/riattivazione dei tratti condizionati dal terreno** quando una cella cambia bioma — punto di integrazione esplicitamente segnalato come delicato.
- **Limite di frequenza per "Eco di segnale inspiegabile"**: al massimo una volta per mondo, altrimenti perde l'effetto di mistero e diventa rumore.
- **Eventi mirati a una famiglia di tratti** (Tempesta magnetica): richiede che gli effetti di un evento possano essere filtrati per famiglia, non solo per bioma o per specie — nuovo tipo di targeting rispetto a quanto già previsto.

## Fuori scope

- Bilanciamento numerico di ciascun evento (soglie, durate, frequenze) — da validare in playtest.
- Implementazione del registro stratigrafico completo — qui solo richiamato come collegamento concettuale.
- Elenco esaustivo: questo documento raccoglie un brainstorm ampio, non pretende di essere l'elenco finale di tutti gli eventi possibili.
