# Abiogenesis — main menu e onboarding

Documento autonomo. Ridefinisce la schermata iniziale alla luce delle decisioni successive (salvataggio a slot unico, impostazioni, vittoria come flag) e stabilisce l'approccio all'insegnamento delle basi. Sostituisce l'impostazione del mockup `main-menu-onboarding-simple.svg`, che resta valido solo come contenuto testuale di riferimento.

## Perché il menu precedente non regge più

Era una schermata unica che conteneva tutto: meccanismo, controlli, riepilogo, tip, seed e avvio. Funzionava quando c'era una sola cosa da fare — iniziare. Ora ci sono almeno tre destinazioni diverse (**riprendi**, **nuova run**, **impostazioni**), e metterle in coda a un muro di istruzioni le rende invisibili proprio a chi torna al gioco e non ha bisogno di rileggere nulla.

Il problema di fondo: **quel testo è interamente onboarding**, e un giocatore alla decima run se lo trova davanti ogni volta. Era il menu che faceva il lavoro del tutorial.

## Main menu — essenziale

Titolo, sottotitolo, e quattro voci:

- **Riprendi** — in evidenza quando uno slot di salvataggio esiste, con un riferimento minimo a *cosa* si riprende (era, seed). Assente se non c'è nulla da riprendere. È ciò che vuole chi torna, quindi sta per primo.
- **Nuova run** — porta alla schermata di setup, non avvia direttamente.
- **Impostazioni**
- **Esci**

**Nota sul salvataggio, visibile in schermata:** riprendere **consuma** lo slot (cfr. `abiogenesis-cross-cutting.md` — sospendi-e-riprendi, non checkpoint). Va detto qui, non scoperto dopo: una riga discreta sotto le voci.

**Immagine allegata separatamente: `menu-main.svg`** — Main menu

## Setup nuova run

Dove sopravvive ciò che serve davvero prima di iniziare:

- **Una frase sul mechanism** — due righe, non un capitolo.
- **Seed precompilato**, modificabile, con rigenera — §5.7 GDD rende i seed riproducibili e condivisibili, ha senso esporli.
- **Controlli come key cap** — compatti, scansionabili.
- **Link discreto "come si gioca"** — apre il riepilogo completo (costi azioni, metabolismi, peso delle osservazioni, obiettivi, grace period: il contenuto già scritto nel mockup precedente) per chi vuole leggerlo. **Opzionale, mai obbligatorio.**
- **Avvia.**

Quando la meta-progressione arriverà (post-MVP), le eventuali opzioni di partenza sbloccate vivranno qui.

**Immagine allegata separatamente: `menu-newrun.svg`** — Setup nuova run

---

## Tutorial: perché non farne uno

**Un tutorial guidato sarebbe attivamente dannoso in questo gioco specifico.** Non per scelta stilistica: il gioco insegna che si formulano ipotesi e si verificano osservando. Un tutorial che dice al giocatore *cosa fare* insegna l'esatto opposto — che le risposte arrivano da fuori invece che dall'osservazione. Rischia di rompere il patto epistemico nei primi cinque minuti, che è precisamente quando si stabilisce.

**Quello che serve non è un tutorial: è che il primo mondo sia leggibile.** E quasi tutto il necessario è già stato progettato altrove senza chiamarlo così:

| Elemento | Stato | Dove |
|---|---|---|
| Obiettivo ammorbidito in world 0 | già nel GDD | §9 |
| Meno tratti attivi in world 0 (4 invece di 5) | proposto | documento archetipi biochimici |
| "Prima luce" — un'interazione forte visibile presto | proposto | documento onboarding/engagement |
| Spark visivo sull'interazione | proposto | documento onboarding/engagement |
| Obiettivo "prima conferma" riservato a world 0 | proposto | documento obiettivi |

Insieme **fanno il lavoro di un tutorial senza esserlo**: il giocatore impara osservando un mondo tarato per essere leggibile, che è esattamente ciò che il gioco gli chiederà di fare per sempre.

## Le due sole aggiunte proposte

**1. Un suggerimento contestuale, una volta sola, non bloccante.** Quando il giocatore ha seminato e sta per avanzare per la prima volta, una riga discreta che indica **cosa guardare**, non cosa fare — es. *"osserva cosa succede dove due specie si toccano"*. Compare, non blocca nulla, non ricompare mai più.

La distinzione tra "cosa guardare" e "cosa fare" è il punto: la prima insegna un metodo di osservazione, la seconda sostituisce il ragionamento del giocatore.

**2. Il tip iniziale spostato nel notebook.** Il suggerimento già scritto — *semina 2-3 organismi di specie diverse vicini tra loro, avanza una sola unità di tempo, leggi il log prima di spendere altro budget* — resta ottimo, ma va messo **come prima voce dell'Observation log in world 0**, non in una schermata di menu. Il giocatore lo trova nel posto dove imparerà a guardare, invece che in una schermata che imparerà a saltare.

---

## Cosa serve per l'integrazione

- **Rilevamento slot di salvataggio** all'avvio, per mostrare o nascondere "Riprendi" e popolarne il sottotitolo.
- **Schermata setup separata** dal menu, con il campo seed già previsto.
- **Riepilogo "come si gioca"** come schermata secondaria richiamabile, non come passaggio obbligato.
- **Flag "primo avanzamento della prima run"** per il suggerimento contestuale una-tantum.
- **Voce precompilata nell'Observation log** solo per world 0 della prima run.

## Fuori scope

- Contenuto testuale completo del riepilogo "come si gioca" — già scritto, resta valido nel mockup precedente.
- Layout della schermata impostazioni — elenco delle voci in `abiogenesis-cross-cutting.md`.
- Opzioni di partenza sbloccabili nel setup — dipendono dalla meta-progressione, post-MVP.
