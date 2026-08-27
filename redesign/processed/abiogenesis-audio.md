# Abiogenesis — design audio

Documento autonomo. Approfondisce la voce "audio" già introdotta in `abiogenesis-cross-cutting.md`, con la struttura a tre livelli, i punti di aggancio nella simulazione, e i prototipi sonori generati.

## Il vincolo che detta la struttura

Il gioco **non ha momenti di silenzio naturale**: la simulazione gira sempre, e con l'avanzamento continuo può girare per minuti. Un suono di fondo continuo e pieno per venti minuti diventa affaticante, e la reazione tipica del giocatore è disattivare l'audio — perdendo anche tutto il suo valore informativo.

La domanda progettuale non è quindi "che suoni facciamo" ma **"come si respira in questo mix"**. Da qui una struttura a tre livelli con **densità decrescente**, non tre categorie di suoni pari.

## Perché l'audio non è decorativo, in questo gioco specifico

Con l'avanzamento continuo, **il giocatore non può guardare 10.240 celle contemporaneamente**. Un suono discreto quando qualcosa di rilevante accade in una zona *fuori dalla vista corrente* è informazione che nessun canale visivo può fornire — è letteralmente l'unico modo per dire "guarda là". L'audio smette di essere atmosfera e diventa **uno strumento di attenzione**, il che giustifica il suo costo indipendentemente dal valore estetico.

Beneficio secondario: è un canale informativo indipendente dal colore, quindi contribuisce direttamente all'accessibilità cromatica (cfr. `abiogenesis-cross-cutting.md`).

---

## Livello 1 — Il fondo

**Deve essere progettato per essere dimenticato**, non per essere apprezzato. Non un drone pieno e continuo: qualcosa che il giocatore smette di sentire coscientemente dopo un minuto e nota solo **quando cambia**.

- **Modulato dagli scalari della zona inquadrata**, non dell'intera mappa — si aggiorna quando la camera si muove, non a ogni pulse.
- Mappatura proposta: temperatura → corpo/calore del timbro; luce → apertura (frequenza di taglio del filtro); tossicità → instabilità (ampiezza ondeggiante, componente rumorosa).
- **Interpolazione lenta** tra stati, per evitare salti percepibili durante il pan della camera.
- Inviluppo di "respiro" molto lento sovrapposto, così il fondo non è mai statico ma nemmeno mai in primo piano.

## Livello 2 — Il ritmo

**Non un battito su ogni pulse.** Con l'avanzamento continuo diventerebbe un ticchettio ossessivo — questa è una correzione rispetto a una prima ipotesi che legava il suono al pulse per rinforzare la metafora del nome.

**Marcare invece il confine di stagione**, l'unità di decisione (cfr. documento sulla scala temporale). Il suono segna i momenti in cui il giocatore **deve tornare a decidere**, non lo scorrere meccanico del tempo. Il pulse resta impercettibile o silenzioso.

Il confine d'era, chiudendosi con un reveal, ha già il proprio stacco (livello 3) e non richiede un marker separato.

## Livello 3 — Gli eventi

**Regola che tiene insieme tutto: il suono di un evento è proporzionale alla sua rilevanza — e la rilevanza è già calcolata.** È lo stesso punteggio di ranking definito per i reveal (minore / notevole / epocale, cfr. documento sulla generazione narrativa). Stesso numero, tre usi: quanto è cerimoniale il reveal, quanto pesa nella Cronaca, quanto è udibile l'evento.

Conseguenza importante: **la coerenza è garantita per costruzione** — non può accadere che qualcosa suoni importante e poi risulti marginale nel notebook, perché entrambi leggono lo stesso valore.

### Spazializzazione — il "guarda là"

Per gli eventi, la posizione rispetto al viewport determina:
- **Panning stereo** — sinistra/destra se l'evento è fuori campo lateralmente;
- **Distanza** → suono più sordo (filtro passa-basso più chiuso) e più basso di volume man mano che l'evento è lontano dalla vista corrente.

Senza questi due parametri l'audio segnala *che* è successo qualcosa ma non *dove*, perdendo gran parte della sua utilità.

---

## Cosa evitare

- **Musica composta.** Non per purismo: una traccia lineare combatte contro un sistema di durata imprevedibile — o si ripete fino alla noia, o taglia nei momenti sbagliati.
- **Feedback sonoro su ogni interazione di interfaccia.** Con un budget di 3 punti ogni click è già un momento pesato; sonorizzarli tutti li appiattisce. Semmai un suono solo sul **commit** dell'azione, non sulla selezione.
- **Suono su ogni pulse** (vedi livello 2).

## Idea accantonata

Dare a ciascuna **famiglia di tratti** una firma timbrica, così che un giocatore esperto riconosca a orecchio che tipo di chimica sta osservando. Interessante come secondo livello di lettura non visivo, ma **accantonata**: rischia di rivelare la famiglia dominante del mondo, che per design deve restare nascosta (cfr. documento archetipi biochimici). Riprendibile solo se limitata a eventi già osservati, dove non aggiungerebbe informazione non guadagnata.

---

## Punti di aggancio nella simulazione

| Livello | Aggancio | Frequenza di aggiornamento |
|---|---|---|
| Fondo | scalari medi del viewport | al movimento camera, con interpolazione lenta |
| Ritmo | confine di stagione | a ogni transizione di stagione |
| Eventi | **fase 7 della pipeline del tick** (emissione osservazioni ed eventi, cfr. `abiogenesis-tick-pipeline.md`) | per candidato-evento, con punteggio di rilevanza già disponibile |

Nessun sistema nuovo richiesto: tutti e tre leggono dati che la simulazione già produce.

---

## Prototipi generati

Cartella `audio-prototipi/`, tutti sintetizzati proceduralmente (nessun campione, nessun asset esterno). Gli script `synth.py`, `synth2.py` e `common.py` espongono i parametri di mappatura, mostrando concretamente come si legano agli scalari.

| File | Contenuto |
|---|---|
| `01-drone-cold-dark.wav` | fondo con temperatura bassa, luce bassa |
| `02-drone-warm-bright.wav` | fondo con temperatura alta, luce alta |
| `03-drone-toxic.wav` | fondo con tossicità alta — l'instabilità è udibile nell'ampiezza |
| `04-season-boundary.wav` | marker di confine di stagione |
| `05-event-tiers.wav` | i tre tier in sequenza: minore, notevole, epocale |
| `06-offscreen-cue.wav` | stessa nota da tre posizioni: sinistra lontana, centro vicina, destra media |
| `07-full-mix-demo.wav` | **il più utile** — 16 secondi con tutti i livelli insieme: fondo, due confini di stagione, due eventi minori fuori campo, uno notevole, uno epocale |

**Stato:** prototipi per valutare la *struttura*, non asset definitivi. I due criteri da verificare all'ascolto sono se il fondo è abbastanza "dimenticabile" e se i tre tier si distinguono senza guardare lo schermo.

## Fuori scope

- Motore audio e tecniche di sintesi in produzione (i prototipi sono in Python; l'implementazione reale userà quanto disponibile nello stack Rust).
- Timbri definitivi — i prototipi definiscono relazioni e gerarchia, non il suono finale.
- Firma timbrica per famiglia di tratti — accantonata, vedi sopra.
