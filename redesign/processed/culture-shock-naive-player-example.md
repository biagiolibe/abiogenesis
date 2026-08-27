# Culture Shock — anatomia di una partita (giocatore ignaro, stress-test dell'onboarding)

Documento autonomo. Contropartita deliberata di `culture-shock-worked-example.md`: quello mostra la partita nel suo momento migliore, scritta da chi conosce ogni sistema. Questo mostra la **stessa prima sessione vissuta da un giocatore che non sa nulla** — nessuna conoscenza pregressa, solo ciò che l'interfaccia comunica davvero. Non è un esercizio di scrittura: è un modo per scoprire se la decisione "niente tutorial guidato" (`abiogenesis-menu-onboarding.md`) regge, prima che lo scopra un giocatore vero.

## Le regole di questo esempio

- Il giocatore vede **solo** ciò che compare a schermo o è già stato letto nel riepilogo "come si gioca" (facoltativo, alcuni lo saltano — qui si assume letto, per non partire già sconfitti).
- Nessuna deduzione "gratuita": se una catena causale richiede più di un passaggio logico non suggerito dall'interfaccia, il giocatore **non la fa**, si comporta come farebbe davvero qualcuno al primo contatto.
- Segnalato in corsivo ogni punto in cui la sessione **potrebbe rompersi** — frustrazione, confusione, abbandono.

---

## Minuto 0-2 — il menu

Legge il riepilogo "come si gioca" (non tutti lo fanno; qui sì). Vede seed, controlli, "in breve" — costi azioni, i quattro metabolismi, il peso delle osservazioni, gli obiettivi. Molta informazione, letta in fretta. Clicca "semina il tuo primo mondo".

## Minuto 2-4 — il mondo si apre

Mappa scura, texture leggera, drone di fondo appena percettibile. Nessun organismo. Prova a muovere la camera, vede i biomi (colori diversi, non sa ancora cosa significhino esattamente).

Apre la banca genomi: tre specie disponibili, ciascuna con un'icona di metabolismo e un sottotesto (`fotolitico · cold`). Non sa cosa vuol dire "cold" in pratica — presume "ambiente freddo".

## Minuto 4-6 — la prima semina, alla cieca

Semina **Halo** in un punto a caso, vicino al centro della mappa, senza guardare bene i colori del bioma sotto. Avanza un'era.

*Punto di attrito 1:* Halo non muore né cresce. Il conteggio resta 1. Il giocatore non ha motivo di aprire lo strumento di ispezione — non gli è stato detto esplicitamente che esiste, o l'ha dimenticato tra le tante informazioni del riepilogo. Guarda la mappa, non succede nulla di visibile. **Prima incertezza: "ho fatto qualcosa di sbagliato, o è normale?"**

## Minuto 6-9 — tentativo di correzione

Pensando di aver sbagliato posizione, semina **Rask** vicino a Halo, sperando che "vicino" sia meglio di "isolato" — non per una teoria, per intuito generico da altri giochi (di solito vicino = buono). Avanza.

*Punto di attrito 2:* stavolta qualcosa cambia — il conteggio di Halo sale, ma il giocatore non sa **perché**. Non ha collegato la vicinanza di Rask alla crescita, perché non ha guardato il notebook né l'ispezione. Ipotesi plausibile nella sua testa: *"forse basta aspettare, cresce da sola col tempo."* — **ipotesi sbagliata, ma coerente con quello che ha visto finora**, e nulla nell'interfaccia la contraddice attivamente in questo momento.

## Minuto 9-14 — esplorazione senza teoria

Prova ad aprire il notebook (tasto Tab, ricordato dal riepilogo). Vede l'Observation log: una riga con un pallino verde, testo tecnico (`CHL isolato accanto a QRM: peso 1.0`). **Non collega "CHL" e "QRM" alle specie che ha seminato** — sono codici a tre lettere, non nomi di specie. Il grafo delle relazioni mostra un nodo isolato e un arco tratteggiato: non è ovvio a un primo sguardo cosa significhi "tratteggiato" senza aver letto la legenda in basso, che nota solo di sfuggita.

*Punto di attrito 3:* il notebook, pensato per essere lo strumento centrale di scoperta, in questo momento comunica **dati**, non **un racconto**. Il giocatore lo chiude senza aver capito che quella riga *è* la spiegazione di ciò che ha visto sulla mappa.

## Minuto 14-20 — semina per accumulo, non per ipotesi

Senza una teoria chiara, il giocatore adotta la strategia più naturale per chi non ha capito il sistema: **semina altre specie sparse per vedere cosa succede**, invece di sperimentare con intenzione. Semina Muck vicino a Halo dall'altro lato.

*Punto di attrito 4 — il più serio:* la popolazione di Halo, ora incastrata tra Rask e Muck, smette di crescere. Nessun messaggio lo dice esplicitamente in questo momento (l'avviso di pressione è visibile solo aprendo l'ispezione sulla cella, cosa che il giocatore non ha ancora fatto). Il giocatore **non ha alcun modo di sapere che sta succedendo qualcosa di meccanicamente interessante** (accumulo di pressione selettiva) — vede solo una crescita che si è fermata, di nuovo senza causa apparente.

## Minuto 20-25 — il primo vero comportamento emergente, non capito

Qualche era dopo, una speciazione avviene. Il reveal si attiva, mostra la frase generata: *"CHL, sotto pressione sostenuta, ha superato la soglia di riorganizzazione. Halo-B è distinta da Halo."*

*Punto di attrito 5:* è un bel momento in teoria — è esattamente il tipo di evento che il gioco vuole rendere memorabile — ma il giocatore **non ha idea di cosa lo abbia causato**. Non ha fatto nulla di deliberato per ottenerlo. Lo legge come un evento casuale del gioco, non come conseguenza delle proprie scelte (aver seminato in un punto senza sbocco). L'obiettivo del gioco — "le tue scelte causano quello che vedi" — non è stato comunicato in questo momento, anche se meccanicamente è vero.

## Minuto 25+ — o si accende la lampadina, o si abbandona

Da qui la sessione biforca, e non c'è modo di sapere quale ramo prevale senza un vero playtest:

- **Ramo A (si accende):** il giocatore, incuriosito da Halo-B, apre il notebook di nuovo, nota il nuovo nodo nel grafo, comincia a collegare i puntini. Da qui in poi la sessione converge verso quella dell'esempio "esperto" — ma ci è arrivato per curiosità/fortuna, non perché il gioco lo ha guidato lì.
- **Ramo B (si abbandona o continua senza capire):** il giocatore continua a seminare per tentativi, senza mai formulare un'ipotesi vera, tratta il gioco come "pianta cose e guarda che succede" invece che come un esperimento scientifico. Il mistero centrale del gioco resta invisibile per l'intera sessione.

## Cosa questo esempio rivela

Cinque punti di attrito, concentrati tutti nei primi 20 minuti — esattamente la finestra in cui, per tua stessa richiesta iniziale, il gioco deve generare voglia di continuare. Il problema comune a tutti e cinque: **il gioco sa sempre cosa sta succedendo (i dati esistono), ma non lo comunica mai proattivamente nel momento in cui servirebbe** — bisogna che sia il giocatore ad andare a cercarlo (aprire l'ispezione, aprire il notebook, leggere la legenda), e un giocatore ignaro non sa ancora che vale la pena cercarlo.

Questo non contraddice la decisione "niente tutorial guidato" — quella resta corretta per i motivi già scritti (un tutorial che dice cosa fare romperebbe il patto epistemico). Ma rivela una **lacuna diversa**: non manca l'insegnamento, manca la **spinta a guardare nel posto giusto al momento giusto**. Sono cose diverse.

## Correzioni minime suggerite, coerenti con "niente tutorial"

Nessuna di queste dice al giocatore *cosa fare* — indicano solo *dove guardare*, che è la distinzione già stabilita nel documento sul menu:

1. **Il suggerimento contestuale una-tantum va rafforzato**, non rimosso: oggi è previsto un solo hint prima del primo avanzamento. Punto di attrito 1 suggerisce che serva un secondo hint, agganciato al primo momento di stallo apparente (energia vicina a zero netto per N tick) — qualcosa come *"nessun cambiamento non è un errore — ma vale la pena guardare cosa succede quando due specie si toccano"*. Non dice "usa l'ispezione", ma orienta.
2. **L'avviso di pressione da saturazione dovrebbe comparire anche senza aprire l'ispezione**, almeno la prima volta che accade in una run — un piccolo indicatore visibile sulla mappa stessa (icona minore sulla cella), non solo nella card di dettaglio. Punto di attrito 4 è il più grave perché è **invisibile per design attuale**, non solo poco chiaro.
3. **Il reveal di speciazione potrebbe accennare alla causa**, non solo all'effetto — il testo generato oggi dice *cosa* è successo, potrebbe includere *perché* (già nei dati: lo stimolo dominante è noto) senza spiegare il meccanismo sottostante. Risolverebbe punto di attrito 5 quasi gratis, dato che il dato "stimolo dominante" esiste già nella pipeline.
4. **La prima riga del notebook dovrebbe tradurre i codici**, non solo elencarli — mostrare temporaneamente il nome della specie coinvolta accanto al codice del tratto nella prima manciata di osservazioni di una run, finché il giocatore non ha imparato a leggerli da solo. Risolverebbe punto di attrito 3.

Nessuna di queste è un tutorial. Sono tutte "il gioco dice dove guardare", mai "il gioco dice cosa fare" — coerenti col vincolo già stabilito.

## Fuori scope

- Se implementare o meno le quattro correzioni suggerite — proposte, non decise.
- Un vero playtest, che resta l'unico modo per sapere quale dei due rami (A/B) prevale davvero. Questo documento è un'ipotesi ragionata, non una misura.
