# Abiogenesis — transizioni, fine run e meta-progressione

Documento autonomo. Copre tre momenti che finora non avevano forma: lasciare un mondo, entrare nel successivo, chiudere una run. **Le prime due e la fine run sono da implementare in MVP** e progettate per funzionare **senza alcuna meta-progressione**. La meta-progressione è trattata a parte come aggiunta post-MVP, coerentemente con §10 GDD che ne differisce la persistenza.

## Perché questi momenti ora esistono

Finché "obiettivi completati → mondo successivo" era automatico, non c'era nulla da progettare. Con la decisione **vittoria come flag** (cfr. documento obiettivi) il giocatore *sceglie* quando lasciare un mondo — e oggi gli si chiede quella scelta senza dargli nulla per compierla. Servono risposte a due domande: *cosa mi resta ancora da vedere qui?* e *cosa mi aspetta di là?*

---

# Parte 1 — MVP

## 1.1 Lasciare un mondo

Non una schermata di trionfo: un momento che rende **leggibile la scelta**. Tre elementi, tutti derivati da dati che il gioco già possiede.

- **Cosa hai capito** — relazioni della matrice confermate su quante erano attive (es. "5 delle 20 relazioni di questo mondo"). Non "hai vinto": una misura di comprensione, che è la valuta reale del gioco.
- **Cosa resta aperto** — ere rimaste nel budget, ed eventualmente un accenno se qualcosa è in corso (es. "una lineage mostra segni insoliti"). **Mai dire *cosa***: coerente col principio già stabilito per l'Emersione, che non deve avere barre di progresso né segnalazioni esplicite. L'accenno serve a creare il dubbio, non a informare.
- **La frase di chiusura** — quella già progettata come "momento della rivelazione", generata dal grafo delle relazioni confermate (cfr. documento sulla generazione narrativa). Questo è il punto giusto in cui farla comparire.

**Requisito di chiarezza:** la scelta non deve essere irreversibile in modo nascosto. Se lasciare il mondo lo chiude definitivamente, va detto **prima** della conferma, non scoperto dopo.

## 1.2 Entrare in un mondo nuovo

**Deliberatamente minimale.** La tentazione è una schermata informativa (tratti attivi, difficoltà, obiettivi, biomi presenti): va evitata, perché anticipare quanto è difficile un mondo significa dire al giocatore qualcosa che dovrebbe scoprire osservando — lo stesso motivo per cui la famiglia dominante non viene dichiarata e gli obiettivi non spiegano perché hanno quei parametri.

Si mostra solo:
- numero del mondo e seed (§5.7 rende i seed condivisibili, ha senso esporli);
- gli obiettivi, che sono pubblici per design (§8).

Tutto il resto lo dice la mappa. Un mondo che si apre in silenzio, col terreno già visibile e gli scalari che si muovono piano, comunica più di qualunque pannello riassuntivo — e si aggancia alla proposta già scritta di far "respirare" l'ambiente prima ancora della prima semina.

## 1.3 Fine della run

Il GDD (§8) stabilisce che la run termina solo per scelta del giocatore, ma non esiste alcun momento di chiusura progettato.

**Inquadratura corretta: non è un game over, è la chiusura di una sessione di ricerca.** Quindi non un punteggio, ma un **bilancio cumulativo** su tutti i mondi attraversati:

- mondi visitati, e come si sono conclusi (per obiettivo / per Emersione / abbandonati con budget residuo);
- relazioni della matrice confermate in totale;
- speciazioni innescate;
- specie sintetizzate via `Splice`;
- biomi e wild species incontrati;
- se un'Emersione è mai avvenuta — l'unico elemento che merita risalto visivo rispetto agli altri.

**Funziona interamente senza meta-progressione:** in MVP questo bilancio è puramente retrospettivo, un modo per dare forma a ciò che il giocatore ha costruito. Non promette sblocchi, non mostra barre di avanzamento verso qualcosa, non allude a contenuti bloccati — deve reggere come momento conclusivo per sé stesso.

**Un unico accorgimento di progettazione in vista del futuro:** lasciare spazio nel layout perché, quando la meta-progressione arriverà, possa innestarsi qui senza ridisegnare la schermata. Spazio, non segnaposto visibili.

---

# Parte 2 — Post-MVP: meta-progressione

Coerente con §10 GDD (persistenza differita). Qui solo la direzione, da riprendere quando la persistenza sarà sul tavolo.

## 2.1 Il vincolo che rende difficile progettarla

§10 stabilisce: **si sbloccano capacità, non risposte.** È corretto, ma il problema è più profondo di come suona — **in un gioco sulla scoperta, qualunque sblocco che riduca il lavoro di scoperta erode il gioco stesso.** Uno sblocco tipo "il notebook conferma con metà delle prove" renderebbe il gioco più facile togliendo esattamente ciò che lo rende interessante.

La domanda giusta non è "cosa do al giocatore" ma: **cosa può crescere run dopo run senza rendere la scoperta più economica?**

## 2.2 Tre categorie che funzionano

**Ampiezza — più cose da scoprire, non scoperte più facili.** Nuove specie nel roster iniziale, nuovi biomi che possono comparire, xenotratti che entrano nel pool possibile, nuovi tipi di obiettivo. Il veterano non decifra più in fretta: incontra più varietà. La categoria più sicura, e probabilmente quella su cui costruire la maggior parte della progressione.

**Strumenti — nuovi modi di sperimentare, non di sapere.** Vi rientrano naturalmente `Isola` (se adottata) e la **mutazione manuale completa** — il livello 2 di `Splice`, che il GDD già prevede esplicitamente come sblocco. Danno più controllo sull'esperimento, non più informazioni sul risultato. `Sposta` starebbe qui, se mai adottata.

**Sfida — accesso al difficile, non scorciatoie.** Mondi di partenza più ostili, o run con vincoli scelti dal giocatore. Si sblocca il diritto di giocare più duro.

## 2.3 Cosa scartare, e un'eccezione istruttiva

**Da scartare:** qualunque cosa acceleri il notebook — soglie di conferma più basse, ipotesi pre-compilate, indizi diretti. Contraddice frontalmente §10 e riduce il gioco a un grind verso il momento in cui non serve più dedurre.

**Eccezione legittima:** le "testimonianze da verificare" (celle pre-compilate di cui **alcune sbagliate**, proposta già discussa altrove) **non sono risposte, sono ipotesi da falsificare** — aggiungono lavoro epistemico invece di toglierlo. Ammissibili anche sotto il principio di §10.

## 2.4 Criterio di sblocco — sull'aver capito, non sull'aver vinto

Se gli sblocchi derivassero dal completare obiettivi, premierebbero chi tira dritto. Derivandoli dai **totali cumulativi di comprensione** (relazioni confermate, speciazioni innescate, biomi e wild species incontrati) si premia chi esplora — il comportamento che il gioco vuole incoraggiare. Sono anche esattamente i dati che il bilancio di fine run già mostra: i due sistemi si parlano senza lavoro aggiuntivo.

**Curva:** i primi sblocchi molto ravvicinati (una o due run), poi progressivamente più distanti. Altrimenti i primi giocatori vedono un gioco più povero di quello reale.

## 2.5 Codex, in forma minimale

La "galleria dei mondi / Codex" era stata rimandata perché toccava la questione irrisolta colony-builder vs run brevi. In forma minimale — **un elenco di ciò che si è incontrato almeno una volta** (specie, biomi, xenotratti, tipi di evento) — quel problema non si pone: non è un mondo persistente da mantenere, è una lista che cresce. Serve anche a rendere *visibile* la categoria "ampiezza", che altrimenti crescerebbe senza che il giocatore se ne accorga.

---

## Cosa serve per l'integrazione (MVP)

- **Conteggio relazioni confermate su attive** per mondo — dato già disponibile dal notebook.
- **Rilevazione "qualcosa è in corso"** per l'accenno in uscita dal mondo, senza esporre di cosa si tratti — richiede accesso allo stato di candidatura dell'Emersione, in sola lettura e senza dettaglio.
- **Conferma esplicita prima di lasciare un mondo**, con l'irreversibilità dichiarata.
- **Aggregati cumulativi di run** (mondi, relazioni, speciazioni, sintesi, biomi, wild species, emersioni) — da tracciare durante la run, non ricostruibili a posteriori.
- **Riuso del generatore narrativo** per la frase di chiusura mondo — nessun sistema nuovo.

## Fuori scope

- Qualunque persistenza tra run — differita post-MVP con la meta-progressione.
- Liste concrete di sblocchi, soglie e curva — Parte 2 dà categorie e criteri, non contenuto.
- Il Codex come schermata — solo indicato in forma minimale, non progettato.
