# Culture Shock — identità del gioco

Documento autonomo. Nomina l'identità emersa dalle decisioni prese, fissa il titolo, e definisce un meccanismo mancante (ipotesi dichiarate e smentite) rimandato esplicitamente a post-MVP.

## Il titolo: **Culture Shock** *(deciso)*

Sostituisce il titolo di lavoro *Abiogenesis*. Sottotitolo mantenuto: **Culture Shock — A sterile world**.

Le ragioni, in ordine di peso:

- **Doppio senso reale, non forzato:** *culture* nel senso di coltura microbiologica, *shock* che è letteralmente una delle quattro azioni del giocatore. Descrive la meccanica, non solo l'ambientazione.
- **Il tono è parte del messaggio.** Un titolo che fa una battuta dichiara che il gioco non si prende troppo sul serio pur essendo rigoroso — la posizione giusta per un gioco duro sull'epistemologia. *Abiogenesis* è austero e descrive solo l'ambientazione; *Petrichor* (l'altro candidato) è bello ma non dice nulla del gioco.
- **Il sottotitolo guadagna:** "A sterile world" accanto a *Culture Shock* funziona meglio che accanto ad *Abiogenesis* — uno dà il tono, l'altro l'atmosfera, e non si sovrappongono.

**Rischio da mettere in conto:** è un'espressione comune, quindi la ricerca online è affollata. Gestibile col sottotitolo e con attenzione nel naming del progetto; il guadagno di identità vale più della penalità di reperibilità.

**Nota operativa:** tutti i documenti di design esistenti usano ancora "Abiogenesis" nei titoli e nei nomi file. Non è urgente rinominarli — ma il nome nel gioco, nel menu e nella documentazione pubblica dovrebbe essere Culture Shock da qui in avanti.

---

## L'identità: un gioco sul metodo scientifico come esperienza emotiva

Il GDD dichiara "Liu Cixin in forma di piastra di Petri" — buono come **posizionamento** (dice a cosa somiglia) ma non come identità (non dice cosa *è*).

Guardando le decisioni effettivamente prese, non le intenzioni dichiarate, emerge qualcosa di più specifico: **non un gioco a tema scientifico, ma un gioco in cui la meccanica *è* l'epistemologia.** Le prove sono tutte scelte già fatte:

| Decisione | Cosa insegna davvero |
|---|---|
| L'ambiente da solo non basta a crescere; serve la matrice | La comprensione è l'unica strada al successo — crescere una popolazione *è* la prova che hai capito |
| L'osservazione isolata pesa più di quella confusa | Il controllo sperimentale è una risorsa |
| Matrice ri-generata a ogni mondo | Scetticismo attivo: generalizzare da run passate viene punito |
| L'ambiguità causale è un output legittimo del reveal | Il gioco è disposto a dire "non è chiaro" invece di inventare una causa |
| `Splice` limitato ai tratti confermati | Non puoi costruire ciò che non hai capito |

Nessuna di queste è "sapore scientifico" applicato sopra un gioco: sono meccaniche che *sono* il metodo scientifico.

**Forza attuale:** molto solida concettualmente, ancora debole come **esperienza percepita**. Il giocatore vive tutto quanto sopra ma non lo sente mai nominare. Un gioco ha identità quando il giocatore sa dire cos'è dopo dieci minuti, non quando lo capisce dopo dieci ore.

### Test di verifica proposto

Chiedere a un playtester, subito dopo la sessione: **"di cosa parla questo gioco?"**
- *"Devi far sopravvivere delle specie"* → l'identità non è passata.
- *"Devi capire come funziona un mondo che non ti dice niente"* → è arrivata.

### Antagonista concettuale (da esplicitare)

Ogni identità forte si definisce contro qualcosa. Questo gioco sta implicitamente dicendo: **contro i giochi che premiano l'ottimizzazione senza comprensione.** Non è mai stato scritto. Vale come bussola per i casi dubbi futuri, non come slogan.

### Estetica — da riprendere

L'estetica attuale (console di laboratorio, monospace, colore piatto, dithering) è nata **per sottrazione** (dai vincoli del pilastro 3) più che per scelta, e la differenza si percepisce: un'estetica scelta comunica un punto di vista, una nata da vincoli comunica economia. Direzione proposta da approfondire in una sessione dedicata: **strumento scientifico, non interfaccia di gioco** — oscilloscopio, sismografo, registratore da laboratorio, oggetti in cui ogni elemento visivo esiste perché *misura* qualcosa.

Ne discende un principio già applicato di fatto ma mai scritto: **ogni elemento visivo deve rappresentare un dato reale.** È stato usato per scartare i biomi arbitrari, per il colore delle specie, per il dithering. Merita di diventare un pilastro esplicito, perché è più utile del pilastro 3 attuale ("niente grafica"): dice cosa *fare*, non solo cosa evitare.

---

## Ipotesi dichiarate e smentite **[POST-MVP]**

### Il buco attuale

**Nel gioco non esiste un'ipotesi del giocatore.** Il notebook ha una "hypothesis grid" nel nome, ma ciò che fa è mostrare le conferme che il *sistema* accumula automaticamente al superamento della soglia di evidenza. Il giocatore le legge, non le formula.

Conseguenza: **non c'è nulla da smentire, perché il giocatore non ha mai dichiarato nulla.** Il suo ragionamento resta nella sua testa, e il gioco non lo vede — il che rende impossibile il momento identitario più forte che il gioco potrebbe avere.

### Il meccanismo

Il giocatore può **marcare una previsione** su una coppia di tratti nel grafo delle relazioni (segno, eventualmente forza), prima di sapere. La marcatura resta visibilmente **sua**, distinta dalle conferme del sistema. Poi l'evidenza si accumula e produce uno di tre esiti:

- **Confermata** — aveva ragione. Soddisfazione, nessuna cerimonia particolare.
- **Smentita** — l'evidenza contraddice la previsione. **È l'evento che merita peso**, e la prima volta in una run merita il tier di reveal più alto: è il momento in cui il giocatore capisce che tipo di gioco sta giocando.
- **Ambigua** — l'evidenza si accumula ma resta contraddittoria. Il gioco lo dice, coerentemente con la regola già stabilita che l'incertezza è un output legittimo.

### Le tre regole che lo rendono buono anziché fastidioso

1. **Facoltativo.** Obbligare a dichiarare un'ipotesi prima di sperimentare lo trasformerebbe in burocrazia. Disponibile, mai richiesto — chi vuole solo osservare gioca lo stesso.
2. **Nessun costo, nessuna penalità.** Nessun punteggio che scende quando sbagli, nessuna risorsa spesa per ipotizzare. Nel momento in cui sbagliare costa, il giocatore smette di ipotizzare o ipotizza solo quando è già sicuro — uccidendo esattamente il comportamento da incoraggiare.
3. **Deve dare qualcosa in cambio.** Non un bonus meccanico ma **attenzione**: le relazioni marcate vengono sorvegliate, e quando arriva evidenza su una di esse il gioco lo segnala invece di lasciarla annegare tra le altre. L'ipotesi diventa un filtro di attenzione — utile, e coerente col tema.

### Tono della smentita

**Non deve suonare come un rimprovero.** Non *"ti sbagliavi"* ma qualcosa come *"l'evidenza contraddice la tua previsione su questa coppia"* — asciutto, da referto, coerente col registro clinico già scelto per la generazione narrativa. È l'unico modo perché il giocatore la riceva come informazione preziosa invece che come punizione.

### Il momento identitario di fine run

Con questo meccanismo, il bilancio di fine run può fare ciò che oggi non può: **mettere sullo stesso piano ciò che il giocatore ha capito e ciò che ha sbagliato**, senza gerarchia tra le due. È la dichiarazione di identità più forte disponibile — *sbagliare ipotesi non è un errore, è il lavoro*.

Regola per quel momento: **il gioco nomina il metodo, non si complimenta.** Non *"Ottimo lavoro, 12 relazioni confermate!"* (premia il risultato e rende l'errore un fallimento), ma un accostamento asciutto di conferme e smentite.

### Cosa serve per l'integrazione

L'infrastruttura esiste quasi tutta: l'evidenza cumulata per coppia di tratti è già calcolata per le conferme automatiche, il ranking degli eventi esiste. Servono due cose nuove:

- **Memorizzare la previsione del giocatore** accanto all'evidenza, per coppia di tratti.
- **Confrontarle al superamento della soglia**, producendo uno dei tre esiti sopra.

Più due agganci: la marcatura nell'interfaccia del grafo, e il conteggio di conferme/smentite negli aggregati di fine run.

**Perché post-MVP:** non è necessario perché il gioco funzioni, ed è meglio introdurlo quando il notebook e il grafo sono stabili. Ma è probabilmente **la singola aggiunta con più impatto sull'identità percepita** tra tutte quelle rimandate.
