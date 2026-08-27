# Culture Shock — identità visiva e ispirazioni letterarie

Documento autonomo. Copre due temi distinti: dare al gioco un tratto originale e sorridente senza contraddire l'estetica "strumento scientifico" già decisa (`culture-shock-population-model-aesthetic.md`), e dichiarare le ispirazioni fantascientifiche in modo legalmente pulito.

## Perché non nella mappa/HUD

L'estetica di gioco (ardesia, pixel netti, inchiostro ambra) è fredda e seria per scelta — coerente con l'identità "il metodo scientifico come esperienza emotiva" (`culture-shock-identity.md`). "Strano e sorridente" vive meglio in **due livelli separati**, non nell'interfaccia di gioco vera e propria: la voce del narratore (testo) e il wordmark (l'unico investimento artistico reale del progetto).

---

## Tono narrativo **[deciso]**

Il registro clinico resta identico per **ciò che conta**: segno, peso, dati — nulla di quello cambia. Cambia solo il colore delle parole intorno, nei punti dove il testo è già generato proceduralmente.

**Esempio, stesso evento:**

- *Oggi:* "CHL, sotto pressione sostenuta, ha superato la soglia di riorganizzazione. Halo-B è distinta da Halo."
- *Con tono:* "CHL non ce la faceva più a stare stretta. Ha ceduto, e da quel cedimento è uscita Halo-B — leggermente diversa, decisamente meno paziente."

Il dato è identico. La seconda versione suona come qualcuno che ha visto la cosa succedere ed è rimasto un po' colpito, invece di un log che stampa un fatto — lo stesso effetto che Dwarf Fortress ottiene con i suoi messaggi di morte assurdi: serietà del sistema sotto, voce leggermente stralunata sopra.

### Dove sì, dove mai

- **Sì:** reveal di fine era, Cronaca (`abiogenesis-notebook-cronaca.md`), frase di chiusura mondo, nomi dei tratti (già biochimicamente autentici — margine per un aggettivo scelto bene, non per inventare).
- **Mai:** numeri, pesi, segno delle relazioni, qualunque cosa che il giocatore debba leggere come dato affidabile per decidere. **Il tono va solo nel colore della frase, mai nel contenuto che serve a giocare.**

### Chi parla

Non un personaggio nominato — una **voce implicita coerente**: un registratore di bordo di una missione scientifica solitaria, abbastanza intelligente da notare l'assurdo, mai abbastanza libero dal protocollo da abbandonare il rigore. Più vicino a Kelvin di *Solaris* (osserva l'incomprensibile restando scientifico) che a un tono da commedia pura, che romperebbe il registro.

**Dipendenza:** estende il pool di frammenti già previsto in `abiogenesis-narrative-generation.md` — nessun sistema nuovo, solo varianti lessicali aggiuntive con questo colore, agganciate agli stessi punti già esistenti.

---

## Wordmark **[proposta, tre varianti]**

L'unico elemento del gioco dove vale la pena un vero investimento di cura grafica — coerente col principio "zero asset diffusi, un solo elemento curato" già applicato altrove.

**Tre varianti esplorate** (`wordmark-variants.svg`):

- **A — il testo poggia su un battito.** Una linea a scalini sotto il titolo, letteralmente un pulse — l'unità di tempo del gioco resa visibile nel logo stesso.
- **B — le O diventano piastre di Petri.** La seconda O di "SHOCK" si apre in un cerchio con organismi sparsi dentro, una colonia al posto di una lettera piatta.
- **C — readout da terminale.** Tipografia monospace su un riquadro netto con un indicatore di stato, massima coerenza con l'estetica "strumento" ma la meno sorridente delle tre.

**Raccomandazione: B.** È l'unica delle tre con una vera battuta visiva incorporata — coerente con "originale, strano ma bello, fa sorridere" — mentre A e C restano più sobrie e concettualmente valide come alternative.

---

## Ispirazioni letterarie — nominate, mai citate **[deciso]**

### Il vincolo, non negoziabile

Riprodurre citazioni testuali da opere protette da copyright in un prodotto commerciale richiede il permesso esplicito di autore/editore — non è una scelta di design, è una questione di licenza con un costo e un processo reali. **Scartato.**

### La strada scelta

- **Una pagina "ispirazioni" nei crediti/menu**, che nomina titolo e autore delle opere senza riprodurne il testo — prassi comune, legalmente pulita perché dichiara l'influenza invece di riprodurla.
- **Epigrafi originali**, scritte da zero nello stesso spirito delle opere citate, mai come citazione vera — stesso principio già usato per la Direttiva del mondo (corsivo, tono editoriale, §8 GDD), esteso a un momento diverso del gioco.

### Opere da nominare, scelte per coerenza tematica con ciò che il gioco fa davvero

| Opera | Autore | Perché è pertinente, non solo "fantascienza a caso" |
|---|---|---|
| *Il problema dei tre corpi* | Liu Cixin | Già il riferimento dichiarato nel GDD (§2) |
| *Solaris* | Stanisław Lem | Un'intelligenza aliena che resiste alla comprensione umana — il tema centrale del gioco in forma pura |
| *Blindsight* | Peter Watts | Epistemologia aliena, comunicazione che fallisce per motivi strutturali, non per cattiva volontà |
| *Picnic sul ciglio della strada* | Fratelli Strugatsky | Una zona con fisica anomala mai del tutto spiegata — quasi letteralmente il Precursore |
| *Semiosis* | Sue Burke | Biochimica vegetale/aliena come protagonista, affine tematico preciso |
| **Children of Time** | **Adrian Tchaikovsky** | **La maggiore ispirazione personale per il gioco, e la fonte diretta dell'idea del salto da micro a macro** — vedi sotto |

### Children of Time — non solo un'ispirazione tra le altre

A differenza delle altre opere in lista, *Children of Time* non è solo un affine tematico: è **la fonte diretta** da cui è nata l'idea dell'Emersione (`culture-shock-emersione.md`) — popolazioni che, sotto pressione ed evoluzione, attraversano il salto da collettivo microscopico a civiltà/organismo macroscopico, con lo stesso rigore biologico che il gioco cerca di mantenere.

**Ruolo assegnato:** guida di riferimento per un'eventuale **fase due** del gioco, oltre l'Emersione — l'estensione già menzionata come "fuori scope" in `culture-shock-emersione.md` (cosa succede meccanicamente a un organismo emerso). Quando quella fase verrà ripresa, è il romanzo da rileggere per orientare le decisioni, non solo da nominare nei crediti.

---

## Cosa serve per l'integrazione

- **Pool di frammenti a tono aggiuntivo** nel sistema di generazione narrativa, applicato solo ai punti elencati sopra (mai a dati numerici).
- **Wordmark definitivo** da produrre a partire dalla variante scelta (B raccomandata) — qui solo un mockup esplorativo, non un asset finale.
- **Pagina crediti/ispirazioni** nel menu — nuovo schermo minimale, coerente con lo stile già stabilito per menu e impostazioni (`abiogenesis-menu-onboarding.md`, `abiogenesis-cross-cutting.md`).

## Fuori scope

- Il pool di frammenti a tono nel dettaglio testuale — qui solo un esempio e la regola, non il pool completo.
- L'asset finale del wordmark — qui tre direzioni esplorative, non un file pronto per la produzione.
- La fase due del gioco (macro-mondo post-Emersione) — Children of Time è assegnato come guida, ma la fase stessa resta esplicitamente non progettata, come già indicato in `culture-shock-emersione.md`.
