# Abiogenesis — redesign della barra laterale

Documento autonomo: contiene tutto il contesto necessario per implementare il redesign della barra laterale (HUD), senza bisogno di leggere altro materiale a parte il GDD del gioco.

## Contesto

La barra laterale attuale (Action / Population / Seed palette / Objective, ciascuna in un riquadro bordato separato, con progress bar continue in stile SaaS) comunica una sensazione da "software di gestione" invece che da gioco. Il gioco ha come pilastro dichiarato: "il divertimento sta nel sistema, non nella grafica" — quindi la correzione non introduce asset grafici o decorazioni, ma cambia struttura, terminologia e scelta dei tipi di indicatore.

## Decisioni

### 1. Struttura del pannello

Un unico pannello continuo, diviso da linee sottili (hairline), invece di 4 riquadri bordati indipendenti. Font monospace su tutta la barra, per un'estetica coerente da "console di laboratorio / field journal".

### 2. Etichette diegetiche

| Prima | Dopo |
|---|---|
| Action | Intervieni |
| Population | Censimento |
| Seed palette | Banca genomi |
| Objective | Direttiva |

### 3. Icone azione

Icone SVG semplici, outline, monocolore — confermate, nessun cambiamento richiesto rispetto ai mockup precedenti.

### 4. Indicatori discreti al posto delle progress bar continue

"Azioni rimaste" (0-3 per era) e "ere trascorse" (obiettivo) diventano tacche/pallini pieni-vuoti invece di barre percentuali: sono risorse piccole e contabili, non metriche continue — l'indicatore discreto comunica meglio la loro natura.

### 5. Direttiva come frase narrativa

L'obiettivo attivo è presentato tra virgolette, in corsivo, con un font editoriale diverso dal resto (unica eccezione monospace nel pannello) — dà un momento di respiro narrativo. Da usare con parsimonia, non estendere ad altri elementi della barra.

### 6. Scalabilità oltre 3 specie

Il roster di specie in gioco supererà 3 (fino a 8 tag attive nei mondi avanzati, cfr. GDD §5.5, e più specie coesistenti). Sia il Censimento sia la Banca genomi vanno progettati per N specie fin da subito:

- **Censimento:** righe compatte a riga singola, contenitore con altezza massima fissa (~4-5 righe visibili) e scroll interno oltre quel numero. Niente crescita illimitata della barra laterale.
- **Banca genomi:** striscia orizzontale scorrevole invece di griglia di chip multi-riga — la griglia funziona solo fino a 3-4 specie, oltre diventa troppo alta.

### 7. Censimento: l'energia mostrata è una media di popolazione, non un valore individuale

Punto rilevante emerso in fase di design, da rispettare nell'implementazione:

- Il valore "energia" nel Censimento è una **media su tutti gli individui** della specie, non l'energia di un singolo organismo.
- Nel gioco esiste una `repro_threshold` (soglia di riproduzione, cfr. GDD §5.3 e §5.9 — valore baseline `10.0`, definito per specie nel genoma) oltre la quale un singolo individuo si riproduce. **Non è un tetto massimo di energia**: un organismo può superarla.
- **Una barra che rapporta la media di popolazione a questa soglia individuale è fuorviante**: una media di 7.44 con soglia 10.0 non implica che nessun individuo si sia riprodotto — la popolazione potrebbe contenere sia individui già oltre soglia sia individui molto sotto, senza che la media lo comunichi.
- **Decisione:** le due informazioni vanno tenute separate.
  1. La **soglia di riproduzione** è un tratto statico per specie e va mostrata nel **Catalog del notebook** (dove già compaiono metabolismo, range di temperatura, tag), non nella barra laterale.
  2. L'**energia media** nel Censimento resta, ma solo come indicatore qualitativo di **vitalità/trend della popolazione** rispetto all'era precedente (in crescita / in calo / stabile — freccia o simbolo colorato), senza alcun riferimento visivo alla soglia di riproduzione.
  3. Le **nascite effettive** (il segnale diretto di quanti individui hanno superato la soglia) vanno comunicate dal log eventi, non inferite da una media — es. "Kael: +3 nascite in quest'era".

## Immagini

### Barra laterale completa

**Immagine allegata separatamente: `sidebar-full.svg`** — Mockup della barra laterale completa

Vista d'insieme: intestazione con era/tick/stato, sezione Intervieni con le 4 icone azione e il contatore a tacche, Censimento con tre specie (indicatore di trend al posto della barra soglia), Banca genomi a chip scorrevoli, Direttiva con testo narrativo e contatore a pallini per le ere.

### Censimento con più specie (scroll)

**Immagine allegata separatamente: `sidebar-censimento-scaled.svg`** — Mockup del censimento con sei specie e scroll interno

Stesso pannello con 6 specie invece di 3: righe compatte, contenitore con altezza fissa, sfumatura in basso che segnala contenuto scrollabile, indicatore di trend (▲/▼/▬) al posto del rapporto con la soglia di riproduzione.

## Fuori scope per questa iterazione

- Redesign del pannello Objective/Direttiva oltre a quanto descritto al punto 5.
- Qualsiasi asset grafico o illustrativo (fuori target rispetto al pilastro "il divertimento è nel sistema, non nella grafica").
- Layout force-directed o altre viste alternative del notebook — non fanno parte di questo documento, che riguarda solo la barra laterale.
