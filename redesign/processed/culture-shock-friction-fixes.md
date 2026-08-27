# Culture Shock — interventi anti-attrito (specifica e priorità)

Documento autonomo. Trasforma le quattro correzioni proposte in `culture-shock-naive-player-example.md` da idee a specifiche implementabili, e le colloca nel piano di lavorazione (`abiogenesis-INDEX.md`).

## Principio comune a tutti e quattro

Nessuno dei quattro dice al giocatore **cosa fare** — tutti indicano solo **dove guardare**, coerente col vincolo "niente tutorial guidato" (`abiogenesis-menu-onboarding.md`). Tutti e quattro **riusano dati già calcolati** dalla pipeline del tick o dal sistema di generazione narrativa — nessuno richiede un nuovo calcolo, solo una nuova via di esposizione.

---

## Intervento 1 — secondo hint contestuale sullo stallo apparente

**Problema (punto di attrito 1):** un organismo isolato a energia quasi-pareggio non mostra alcun cambiamento visibile per diversi tick. Il giocatore non ha modo di distinguere "sto assistendo al comportamento previsto" da "ho sbagliato qualcosa".

**Trigger:** energia netta pro-capite di una popolazione entro una soglia stretta attorno a zero (proposta: `|netto| < 0.1`) per **N tick consecutivi** (proposta: N=15) senza che sia già comparso l'hint dell'onboarding esistente (avanza-una-volta-e-osserva).

**Presentazione:** una riga discreta, non bloccante, non modale — stesso stile del primo hint già previsto (`abiogenesis-menu-onboarding.md`). Testo indicativo: *"nessun cambiamento non è un errore — vale la pena guardare cosa succede quando due specie si toccano."*

**Frequenza:** una sola volta per run (non per popolazione), come il primo hint. Non ricompare se il giocatore lo ha già visto.

**Dipendenze:** richiede accesso al netto energetico per-capita già prodotto dalla fase 5 della pipeline del tick (`abiogenesis-tick-pipeline.md`) e un contatore di tick consecutivi entro soglia per popolazione.

---

## Intervento 2 — indicatore di saturazione visibile sulla mappa

**Problema (punto di attrito 4, il più grave):** una cella satura senza sbocco per lo sfondamento accumula pressione selettiva locale, ma oggi questa condizione è visibile **solo** aprendo la card di ispezione (click esplicito) — nessun segnale ambientale la comunica. È l'unico dei cinque punti di attrito del giocatore ignaro classificato come "invisibile per progettazione attuale", non solo "poco chiaro".

**Trigger:** popolazione di una cella alla capacità massima, con tutte le celle adiacenti occupate da specie diverse (nessuno sbocco disponibile) — la stessa condizione già definita in `culture-shock-population-model-aesthetic.md` che alimenta la pressione locale.

**Presentazione:** un piccolo indicatore sulla cella stessa nella vista di dettaglio (non nell'overview, dove il singolo dato di cella non è comunque visibile) — coerente con lo stile pixel deciso: un piccolo simbolo ai margini della forma dell'organismo, stesso trattamento a blocchi già usato per il badge di conteggio. **Solo la prima volta che accade in una run**, poi resta visibile ma senza ulteriore enfasi (es. un lampeggio o un colore d'accento nella prima occorrenza, poi indicatore statico nelle successive) — per non trasformarlo in rumore permanente una volta che il giocatore ha imparato a riconoscerlo.

**Dipendenze:** richiede che la fase 6 della pipeline del tick (accumulo di pressione selettiva, `abiogenesis-tick-pipeline.md`) esponga la condizione "saturo senza sbocco" come flag leggibile dal livello di rendering, non solo come input al calcolo di pressione.

---

## Intervento 3 — causa nominata nel reveal di speciazione

**Problema (punto di attrito 5):** il testo generato per una speciazione comunica **cosa** è successo ma non **perché** — il giocatore non collega l'evento alle proprie scelte pregresse (es. aver seminato in un punto senza sbocco), anche se meccanicamente il collegamento esiste.

**Correzione:** il frammento di testo del reveal include lo **stimolo dominante** che ha causato il superamento soglia (danno da interazione / disallineamento ambientale / tossicità — già un dato noto in §5.11 GDD e già passato attraverso la pipeline del tick). Non spiega il meccanismo sottostante (niente numeri, niente formula) — nomina solo la causa in linguaggio naturale, nello stesso registro clinico già stabilito per la generazione narrativa (`abiogenesis-narrative-generation.md`).

**Esempio di frammento aggiuntivo:** la frase esistente *"CHL, sotto pressione sostenuta, ha superato la soglia di riorganizzazione"* guadagna una clausola: *"...per mancanza di spazio libero"* (se lo stimolo dominante è disallineamento da saturazione) o *"...per l'esposizione prolungata a KTL"* (se dominante è danno da interazione con un tratto specifico).

**Dipendenze:** il generatore di frammenti (`abiogenesis-narrative-generation.md`) deve già ricevere lo stimolo dominante come parte dei dati dell'evento candidato — verificare se la fase 7 della pipeline lo passa già insieme all'evento di soglia superata, o se va aggiunto come campo.

---

## Intervento 4 — traduzione temporanea dei codici tratto nel log

**Problema (punto di attrito 3):** le prime righe dell'Observation log usano codici a tre lettere (`CHL`, `QRM`) che il giocatore non ha ancora imparato a collegare alle specie seminate — il notebook comunica dati, non un racconto leggibile, proprio nel momento in cui dovrebbe insegnare a leggerlo.

**Correzione:** nelle **prime N osservazioni di una run** (proposta: N=5), ogni riga del log mostra il nome della specie coinvolta accanto al codice del tratto, non il codice da solo — es. *"CHL (Halo) isolato accanto a QRM (Rask): peso 1.0"* invece di *"CHL isolato accanto a QRM: peso 1.0"*. Dopo le prime N, il log torna alla forma standard solo-codice, assumendo che il giocatore abbia imparato l'associazione.

**Perché non permanente:** la forma standard (solo codice) è più compatta e coerente con l'estetica "referto di laboratorio" già decisa — l'aiuto va dato solo nella finestra in cui serve, poi tolto, non lasciato per sempre come ridondanza.

**Dipendenze:** nessuna nuova, solo un contatore di osservazioni totali della run e una condizione sulla formattazione della riga di log in `abiogenesis-notebook-cronaca.md`.

---

## Priorità e collocazione nel piano di lavorazione

**Nessuno dei quattro è core** (il gioco funziona senza), ma **tutti e quattro dipendono solo da sistemi già pianificati nella Fase 1** (pipeline del tick, modello di popolazione, scala temporale) e si agganciano a sistemi della Fase 2-3 (ispezione, notebook, generazione narrativa). La collocazione naturale è quindi **una nuova fase intermedia, subito dopo il checkpoint di playtest della Fase 1**, prima di proseguire con la Fase 2 — non a fine piano, perché ogni fase successiva (contenuto, struttura di sessione, rifinitura) costruisce sopra un imbuto di primi 20 minuti che, se dispersivo, riduce il valore di tutto il resto.

### Fase 1b — interventi anti-attrito (nuova, tra Fase 1 e Fase 2)

| Priorità | Intervento | Perché quel livello |
|---|---|---|
| **Alta** | Intervento 2 — indicatore di saturazione | Unico dei quattro classificato come "invisibile per progettazione", non solo poco chiaro — il punto di attrito più grave identificato |
| Media | Intervento 1 — secondo hint contestuale | Rischio reale (prima incertezza del giocatore) ma mitigabile anche solo col riepilogo "come si gioca" già esistente |
| Media | Intervento 3 — causa nel reveal | Costo quasi nullo (il dato esiste già), impatto diretto sulla leggibilità del momento più memorabile del gioco |
| Bassa-media | Intervento 4 — traduzione codici nel log | Attrito reale ma più facilmente superato con l'esperienza, rispetto agli altri tre |

**Verifica raccomandata prima di procedere oltre la Fase 1b:** rigiocare mentalmente (o, meglio, con un vero playtester) lo stesso scenario di `culture-shock-naive-player-example.md` con questi quattro interventi applicati, per vedere se i punti di attrito si attenuano davvero — coerente col principio già scritto in quel documento che solo un playtest vero può dirlo con certezza, non un ragionamento sulla carta.

## Cosa serve per l'integrazione

Riassunto delle dipendenze tecniche, già dettagliate sopra per ciascun intervento:
- Contatore di tick entro soglia per popolazione (intervento 1).
- Flag "saturo senza sbocco" esposto dalla pipeline al livello di rendering, non solo al calcolo di pressione (intervento 2).
- Stimolo dominante passato come campo dell'evento candidato al generatore narrativo (intervento 3).
- Contatore di osservazioni totali della run per la formattazione del log (intervento 4).

## Fuori scope

- Valori numerici esatti delle soglie (N tick, N osservazioni, `|netto| < 0.1`) — indicativi, da validare in playtest come tutto il resto del bilanciamento.
- Verifica se gli interventi bastano davvero — richiede il playtest raccomandato sopra, non deducibile da questo documento.
