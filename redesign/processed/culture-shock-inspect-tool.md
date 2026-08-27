# Culture Shock — strumento di ispezione

Documento autonomo. Colma un buco identificato in una revisione precedente del design e mai più affrontato: il giocatore deve poter osservare per dedurre, ma oggi l'unica lettura diretta è "luminosità/riempimento = energia". Serve un modo per vedere *perché* una cella sta come sta.

## Il problema, aggravato dal modello di popolazione

Con il modello a organismo singolo, "qualcosa muore" era già poco informativo: si vedeva l'effetto, non la causa. Con il **modello di popolazione per cella** (cfr. `culture-shock-population-model-aesthetic.md`) il problema si aggrava: un conteggio che scende non dice più nemmeno *quale* individuo è stato colpito, perché non esistono più individui distinti da osservare — solo un aggregato. Senza uno strumento dedicato, l'esperimento pulito resta cieco: si vede che una popolazione cala, non quanto pesa la matrice rispetto all'ambiente.

## Il meccanismo: due livelli

**Immagine allegata separatamente: `inspect-tool.svg`** — Tooltip da hover a sinistra, card completa da click a destra con il bilancio scomposto

**Passaggio del mouse — leggero, sempre attivo, nessuna azione richiesta.** Un'etichetta minima accanto al cursore.

- **Cella con popolazione:** nome specie, conteggio popolazione, freccia di trend — come sopra.
- **Qualunque cella, popolata o vuota — nome del bioma.** Correzione a un buco reale: oggi il bioma si riconosce solo da colore/texture, mai da un'etichetta testuale, il che viola lo stesso principio già applicato al segno delle relazioni ("il colore non deve mai essere l'unico canale", `abiogenesis-cross-cutting.md`). Diventa particolarmente importante con **Seed armato**: il giocatore deve sapere dove sta per seminare prima di clicconare, non scoprirlo dopo aprendo l'ispezione. Su una cella vuota, il tooltip mostra solo il nome del bioma; su una cella popolata, bioma e dati della popolazione insieme.

Informazione ambientale, non richiede alcun click né consuma budget.

**Click, quando nessuna azione è armata — esplicito, ancora una card completa alla cella.** *(Correzione: se un'azione dall'HUD è armata, quel click esegue l'azione invece di aprire l'ispezione — vedi `culture-shock-controls.md` per la risoluzione completa di questo conflitto tra documenti.)* Resta visibile finché non si seleziona un'altra cella o si preme Esc (chiude prima l'ispezione, non esce dal gioco — Esc è a strati, vedi documento controlli). Contiene:

- **Specie e origine** — nome, icona di metabolismo, se seminata / indigena / sintetizzata (cfr. documento Cronaca).
- **Popolazione** — conteggio, energia media pro-capite, indicatore a tacche verso la soglia di riproduzione (stesso linguaggio discreto già usato altrove nell'HUD, non una barra continua).
- **Tratti** presenti sulla specie.
- **Bilancio dell'ultimo pulse, scomposto riga per riga**: guadagno dalla risorsa (luce/vicini/residuo/tossicità a seconda del metabolismo), **ogni tratto vicino elencato singolarmente con il proprio contributo di segno** (non un "effetto matrice" aggregato — è il dettaglio che rende l'ispezione utile: si vede *quale* vicino sta aiutando o danneggiando), mantenimento, affollamento, netto finale.
- **Avviso di pressione**, se la cella è satura e priva di sbocco per lo sfondamento (cfr. documento sul modello di popolazione — quella condizione alimenta la pressione selettiva locale).

**Click su una cella vuota — card di caratteristiche del bioma.** Estensione dello stesso strumento: cliccando una cella senza popolazione, la card mostra le caratteristiche ambientali invece del bilancio energetico.

- **Nome del bioma** (già nel tooltip, ripetuto qui per coerenza).
- **Temperatura, luce, tossicità in fasce qualitative** — stesso linguaggio già usato per le specie (es. "cold/warm"), non i valori numerici esatti del GDD, che restano dati interni non esposti al giocatore.
- **Abitabilità**, se il bioma è inabitabile per organismi terrestri (es. acqua profonda) — dichiarato esplicitamente, perché è un'informazione di vincolo, non di scoperta: sapere in anticipo che una cella non può ospitare nulla evita al giocatore di sprecare un'azione Seed per scoprirlo per tentativi.

**Cosa resta deliberatamente fuori — vincolo non negoziabile:** se il bioma attiva un tratto condizionato dal terreno (§5.5 GDD), **la card non lo rivela**. Quel legame è un'estensione della stessa opacità che protegge la matrice dei tratti — va scoperto osservando un comportamento anomalo sul campo, non letto in un pannello di consultazione. Mostrarlo trasformerebbe un pezzo di mistero in un dato consultabile a comando, contraddicendo il principio che rende necessaria la matrice in primo luogo (cfr. documento sul bilanciamento).

## Gratuita, fuori dal budget azioni

Coerente con la nota originale che ha sollevato il problema: l'ispezione è **osservazione**, non intervento. Non costa punti, non consuma budget di stagione — va tenuta nettamente distinta dalle quattro azioni dirette (Seed, Stress, Cull, Splice), che restano l'unico luogo con un costo. Confondere le due cose annacquerebbe il significato del budget azioni.

## Nessun calcolo nuovo

Ogni riga del bilancio scomposto è un valore che **la pipeline del tick già produce e passa tra le fasi** (cfr. `abiogenesis-tick-pipeline.md`): guadagno (fase 2), contributo per vicino (fase 3, dove l'interazione conta già per presenza — un elenco di tratti vicini con il loro segno è quindi già la forma naturale del dato, non richiede una nuova aggregazione), costi (fase 4), netto (fase 5). Lo strumento di ispezione è pura esposizione di dati già calcolati, non un sistema a parte da costruire e mantenere sincronizzato.

## Perché due livelli e non uno solo

Un solo livello fallisce in entrambe le direzioni possibili: mostrare sempre tutto (il breakdown completo) sommergerebbe lo schermo di testo a ogni passaggio del mouse; richiedere sempre un click anche per l'informazione minima (nome, trend) renderebbe l'osservazione passiva più laboriosa di quanto debba essere per qualcosa di gratuito. Il tooltip leggero copre il caso comune (scorrere la mappa con lo sguardo), la card completa copre il caso in cui il giocatore ha già deciso che quella cella specifica merita attenzione.

## Cosa serve per l'integrazione

- **Nome del bioma leggibile dal tooltip per qualunque cella**, popolata o vuota — richiede che il tipo di bioma sia accessibile dal livello di rendering per ogni cella sotto il cursore, non solo per quelle con popolazione.
- **Card caratteristiche bioma su click di cella vuota** — richiede la conversione di temperatura/luce/tossicità in fasce qualitative (stessa scala già usata per il range termico delle specie) e un flag di abitabilità per bioma. **Va garantito che questa card non includa mai alcun riferimento a tratti condizionati dal terreno**, nemmeno indirettamente (es. un'icona o un colore che lasci intuire "qui succede qualcosa di speciale") — verificare esplicitamente in fase di implementazione, è il tipo di leak facile da introdurre per errore.

- **Stato di hover e stato di selezione** distinti nell'interfaccia mappa, con la card che segue la selezione invece che il cursore.
- **Esposizione dei valori intermedi della pipeline** (già richiesta per altri motivi — notebook, reveal, audio) fino al livello di UI: qui non servono nuovi campi, serve che quelli esistenti siano leggibili dal livello di rendering.
- **Formattazione del breakdown come lista di contributi per tratto vicino**, non come singolo numero aggregato — verificare che la fase 3 della pipeline mantenga il dettaglio per-vicino invece di sommarlo prima di restituirlo.

## Fuori scope

- Layout esatto e posizionamento della card (ancoraggio alla cella, offset, comportamento ai bordi dello schermo) — il mockup mostra la struttura del contenuto, non una specifica pixel-perfect.
- Scorciatoia da tastiera per l'ispezione, se mai ne servirà una oltre a click/hover.
