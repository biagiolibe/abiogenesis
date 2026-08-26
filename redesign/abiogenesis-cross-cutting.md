# Abiogenesis — voci trasversali: salvataggio, audio, accessibilità, performance, lingua, impostazioni

Documento autonomo. Raccoglie sei temi mai affrontati nei documenti di design né nel GDD, ciascuno con una raccomandazione operativa. Non sono meccaniche di gioco: sono le cose che, se mancano, rendono il gioco inutilizzabile o escludente indipendentemente da quanto è buono il design.

---

## 1. Salvataggio e ripresa

### Snapshot, non journal

Due strade possibili:

- **Journal** (seed + sequenza di azioni, si rigioca da capo al caricamento): file minuscoli, e darebbe replay condivisibili gratis. **Ma dipende dal fatto che la simulazione non cambi mai** — e il GDD è pieno di coefficienti dichiarati da validare in playtest, che stiamo per ritarare tutti. Ogni modifica a un numero invaliderebbe ogni salvataggio esistente. In più il caricamento richiederebbe di ri-simulare migliaia di pulse.
- **Snapshot** (serializzazione dello stato): robusto ai cambi di codice se versionato, caricamento istantaneo. Le dimensioni non sono un problema — 128×80 celle con pochi byte ciascuna, più organismi, specie e notebook, restano nell'ordine delle centinaia di KB.

**Raccomandazione: snapshot.** Il journal va però **conservato come metadato accanto allo snapshot**, non per caricare ma perché abilita replay condivisibili e soprattutto **debug riproducibile** — con una simulazione deterministica (§5.7 GDD), poter dire "questa è la sequenza esatta che ha prodotto il bug" vale molto.

### Sospendi-e-riprendi, non checkpoint — è una scelta di design, non tecnica

Se il giocatore può salvare e ricaricare liberamente, **può annullare un esperimento andato male** — e questo demolisce la tensione centrale del budget azioni: "non puoi fare tutto, scommetti sulla tua ipotesi migliore" non significa nulla se una scommessa sbagliata si annulla ricaricando.

**Raccomandazione: uno slot unico, consumato al caricamento.** Si salva per chiudere il gioco, non per assicurarsi contro le proprie decisioni. Il giocatore ottiene ciò di cui ha realmente bisogno (interrompere una run lunga) senza ottenere ciò che romperebbe il gioco.

### Due dettagli che è facile scoprire tardi

- **Va salvato lo stato del generatore casuale, non il seed.** Salvando solo il seed e riprendendo a metà, la sequenza pseudo-casuale riparte da capo e la simulazione diverge da quella che il giocatore stava giocando. È un bug che si manifesta solo in playtest.
- **Notebook e Cronaca vanno nello snapshot.** Sono la progressione reale dentro una run: perdere le conferme accumulate sarebbe peggio che perdere lo stato della griglia.

**Momento del salvataggio:** al confine di stagione o era, dove il tempo è già naturalmente fermo. Salvare a metà pulse richiederebbe di catturare stati parziali senza alcun guadagno.

---

## 2. Audio

**Il pilastro 3 non lo vieta.** "Il divertimento è nel sistema, non nella grafica" parla di *grafica* — e il gioco si descrive come ipnotico da guardare (§2 GDD). L'audio è probabilmente il moltiplicatore di atmosfera col miglior rapporto costo/resa disponibile, e l'unico che non contraddice nessun vincolo esistente.

**Raccomandazione: audio generativo, non tracce composte.** Coerente con lo spirito del progetto (nessuna pipeline di asset, tutto derivato dal sistema):

- **Drone ambientale modulato dagli scalari** della zona inquadrata — temperatura, luce, tossicità pilotano timbro e intensità. Il mondo *suona* diverso a seconda di dove guardi, senza una nota composta a mano.
- **Il pulse come battito sonoro sommesso** — rinforza direttamente la metafora già scelta per il nome dell'unità di tempo, e rende percepibile l'avanzamento continuo senza guardare il contatore.
- **Eventi sonorizzati con parsimonia**: nascite/morti come micro-eventi appena percettibili, i reveal con uno stacco netto e differenziato per tier (minore/notevole/epocale). Il silenzio nelle ere di quiete è parte del design, esattamente come nella Cronaca.

**Beneficio secondario non banale:** l'audio è un canale informativo indipendente dal colore, quindi contribuisce direttamente al punto 3.

---

## 3. Accessibilità cromatica

**Il problema è reale e strutturale:** il segno delle relazioni è codificato rosso/verde — la peggiore combinazione possibile per protanopia e deuteranopia, che insieme riguardano una quota significativa dei giocatori maschi.

**Regola da adottare, non un'opzione da aggiungere: il colore non deve mai essere l'unico canale.** Ovunque il colore porti informazione, deve esserci un secondo canale non cromatico:

- **Segno delle relazioni** → simbolo esplicito (`+` / `−`) accanto al colore, non solo la tinta.
- **Confermata vs ipotesi** → già risolto (tratto continuo vs tratteggiato).
- **Specie** → già risolto (forma dell'icona per metabolismo, colore per specie — quindi due canali).
- **Biomi** → il dithering a due toni e i bordi netti già forniscono struttura oltre alla tinta; verificare che le coppie di biomi adiacenti restino distinguibili in scala di grigi.
- **Indicatore pulito/confuso nel log** → oggi solo colore (verde/ambra): aggiungere una distinzione di forma.

**In più**, una palette alternativa selezionabile nelle impostazioni. Ma la regola sopra viene prima: una palette alternativa non salva un'interfaccia che affida il significato al solo colore.

---

## 4. Performance

**La simulazione non è il rischio.** 128×80 = 10.240 celle, vicinato di Moore = 8 lookup per organismo: anche a occupazione piena siamo nell'ordine di ~10⁵ operazioni per pulse, trascurabile per Rust nativo.

**I tre rischi reali sono altrove:**

- **Rendering di 10.240 celle per frame.** In immediate mode (egui) un widget per cella è insostenibile. Va disegnato come singola texture/mesh aggiornata, o con batching — è il punto da verificare per primo, perché è quello che degrada visibilmente.
- **Avanzamento continuo = molti pulse al secondo.** Il costo per pulse va misurato con il rendering attivo, non isolato.
- **Crescita illimitata dei dati di osservazione.** Ogni adiacenza tra organismi con tratti genera un'osservazione (§7 GDD): su una run lunga con griglia popolata, la mole cresce senza limite naturale. Serve **compattazione o limite** — le osservazioni servono in forma aggregata (evidenza cumulata per coppia di tratti), non come lista integrale conservata per sempre. Stesso principio già adottato per la compressione delle ere di quiete nella Cronaca.

**Raccomandazione:** profilare presto con griglia piena e avanzamento continuo attivo, non a fine sviluppo.

---

## 5. Lingua

**Situazione attuale:** GDD e codice in inglese, testo di onboarding della build in inglese, documenti di design e mockup in italiano.

**Raccomandazione: inglese come lingua del gioco**, italiano solo per la documentazione interna. Il pubblico naturale di un simulatore hard-SF di nicchia è internazionale, e la scelta è già di fatto operativa nel codice.

**Il vincolo che conta, e va deciso adesso:** il testo generato proceduralmente (frammenti soggetto/causa/effetto per reveal e Cronaca) è **molto più costoso da localizzare** di stringhe fisse — le lingue con genere e accordo grammaticale rompono la concatenazione di frammenti pensata per l'inglese. Se una localizzazione futura è anche solo possibile, i frammenti vanno tenuti come **dati strutturati** (con i loro attributi grammaticali) e non come stringhe già concatenate. È una decisione di architettura da prendere prima di costruire il pool, non dopo.

---

## 6. Impostazioni

Nessuna schermata di opzioni è mai stata progettata, e diventa necessaria nel momento in cui una qualunque delle voci sopra viene adottata. Contenuto minimo:

- **Audio** — volumi separati per drone ambientale, eventi, interfaccia.
- **Velocità dell'avanzamento continuo** — non un dettaglio: è il ritmo percepito dell'intero gioco.
- **Palette alternativa** per accessibilità cromatica.
- **Lingua**, se mai localizzato.
- **Riduzione del movimento** — utile per chi è sensibile ad animazioni continue; disattiva pulsazioni ambientali e transizioni.

Accessibile sia dal menu principale sia dal menu di pausa in-run (definito in `culture-shock-controls.md` — raggiunto premendo `Esc` quando nessun'altra interfaccia è aperta).

---

## Fuori scope

- Formato di serializzazione concreto e strategia di versionamento degli snapshot.
- Sintesi audio concreta (motore, tecniche) — qui solo la direzione generativa.
- Palette alternativa concreta per accessibilità — la regola dei due canali viene prima.
- Budget di performance numerici — da stabilire profilando, non a tavolino.
