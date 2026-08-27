# Abiogenesis — sistema di biomi

Documento autonomo per un task di integrazione. Contiene tutti i biomi proposti, i loro valori ambientali di base, le regole per l'overlay degli alberi, e i collegamenti con le meccaniche di gioco esistenti o proposte altrove. Non richiede la lettura di altri documenti per essere capito, a parte il GDD per i riferimenti a formule/costanti citate.

## Contesto

Il gioco ha già un layer ambientale (`temperature`, `light`, `toxicity`, §5.2 GDD) e regole che legano l'ambiente alla sopravvivenza degli organismi (`env_fit`, §5.6). Oggi però questo layer è generato solo come due gradienti statici su due assi (Phase 0) o, nella build attuale, reso con un rendering troppo morbido/sfumato che ne appiattisce la leggibilità (vedi nota di stile più sotto). Il sistema di biomi qui proposto dà una **forma leggibile e diversificata** a quel layer, mantenendo tutti i vincoli già definiti (pilastro 3, quadratini colorati, nessun asset grafico).

## Stile di rendering (vincolo trasversale a tutti i biomi)

Per coerenza con l'estetica già stabilita nel resto del gioco (console/laboratorio, dati leggibili):

- **Colore piatto per cella, non gradiente.** Ogni cella appartiene a un bioma discreto e prende un colore piatto, non una sfumatura continua tra biomi.
- **Bordi netti allineati alla griglia.** I confini tra biomi sono linee sottili sul bordo condiviso tra celle, non forme vettoriali morbide o sfumate. I confini con l'acqua (linea di costa) sono più marcati dei confini interni tra biomi terrestri.
- **Dithering leggero, non blending.** Per dare consistenza materica a un bioma senza sfumare mai il confine con la cella accanto, si alternano due toni molto vicini a scacchiera (pattern fisso) all'interno dello stesso bioma — mai un gradiente continuo.
- **Overlay come tinta piatta a opacità fissa, non blend.** Quando un'informazione aggiuntiva (es. una zona calda) si sovrappone a un bioma, va resa come una tinta piatta uniforme sopra il colore base, non come una media/fusione dei due colori — altrimenti si ottengono colori "sporchi" e ambigui che non comunicano più un dato preciso.
- **Ogni colore deve rappresentare un dato reale**, non essere solo decorativo — coerente con la scelta già presa altrove di scartare biomi/regioni a colore arbitrario scollegato dalla simulazione.

## I biomi

16 biomi in totale: i 12 richiesti più 4 proposti per agganciarsi meglio alle meccaniche di gioco (motivazione di ciascuno più sotto).

| Bioma | `temperature` | `light` | `toxicity` | Alberi | Nota meccanica |
|---|---|---|---|---|---|
| Acqua profonda | 0.30 | 0.10 | 0.00 | no | nessun archetipo terrestre può occuparla oggi |
| Acqua bassa | 0.40 | 0.50 | 0.00 | no | nicchia costiera, luce moderata |
| Lago | 0.45 | 0.40 | 0.05 | no | acqua ferma, residuo si accumula → nicchia per decompositori |
| Palude | 0.50 | 0.30 | 0.45 | sparso | tossicità elevata di base — stessa logica della zona tossica già esistente, qui diffusa su un intero bioma invece che su un rettangolo isolato |
| Pianura | 0.50 | 0.60 | 0.00 | sparso | bioma di riferimento, baseline per gli altri |
| Foresta | 0.50 | 0.35 | 0.00 | denso | luce ridotta sotto la chioma (nuance meccanica: più "lussureggiante" a vista ma meno luce reale di quanto sembri), residuo alto |
| Collina | 0.45 | 0.55 | 0.00 | sparso | transizione, valori moderati ovunque |
| Montagna | 0.25 | 0.70 | 0.00 | sparso | freddo, molta luce |
| Vetta | 0.05 | 0.85 | 0.00 | no | estremo freddo, solo archetipi molto specializzati sopravvivono |
| Roccia nuda | 0.40 | 0.50 | 0.30 | no | scarsa risorsa organica → nicchia naturale per un futuro metabolismo chemiolitotrofo legato alla tossicità (già previsto come estensione futura, §5.4 GDD) |
| Cratere profondo | 0.60 | 0.20 | 0.60 | no | depressione in ombra, tossicità residua da impatto — vedi "Collegamenti" sotto |
| Deserto | 0.85 | 0.90 | 0.00 | no | luce altissima, escursione termica forte |
| Bocca vulcanica | 0.95 | 0.40 | 0.40 | no | fonte di calore puntiforme che diffonde nell'ambiente — vedi "Collegamenti" |
| Geyser | 0.80 | 0.40 | 0.10 | no | fonte di calore/umidità puntiforme e pulsante, impronta piccola (1-2 celle) — vedi "Collegamenti" |
| Tundra | 0.10 | 0.50 | 0.00 | no | estremo freddo alternativo alla vetta, ma pianeggiante (nicchia fredda più accessibile) |
| Distesa di cristalli | 0.40 | 0.60 | 0.20 | no | chimica anomala — candidato per attivare un tag genetico raro/unico, non pienamente descrivibile con i soli tre scalari ambientali esistenti (vedi "Collegamenti") |

*I valori sono un baseline di partenza, coerenti nell'ordine di grandezza con gli altri valori già presenti in §5.9 GDD — da validare in playtesting come tutto il resto del bilanciamento numerico del gioco.*

### Perché questi 4 biomi in più

- **Bocca vulcanica** e **Geyser** sono la versione "bioma" di un'idea discussa in parallelo: sorgenti di calore/luce puntiformi che diffondono nell'ambiente, al posto degli attuali due gradienti fissi su assi indipendenti. Se quell'idea viene implementata, questi due biomi sono i luoghi naturali dove le sorgenti si trovano fisicamente sulla mappa — la bocca vulcanica come sorgente forte e costante, il geyser come sorgente più piccola e pulsante nel tempo (si aggancia anche alla proposta, discussa altrove, di far "respirare" l'ambiente con micro-animazioni anche a griglia vuota).
- **Tundra** dà un estremo freddo accessibile e pianeggiante, in coppia con il deserto (estremo caldo): senza, l'unico modo per sperimentare freddo estremo sarebbe la vetta, molto più piccola e marginale come superficie disponibile.
- **Distesa di cristalli** è la proposta più speculativa: un bioma dalla tonalità visivamente "aliena" (fuori dalla palette naturale terrosa/verde degli altri), pensato come possibile innesco per un tag genetico raro o un comportamento unico — un modo per rendere *scoperta* anche la semplice esplorazione geografica della mappa, non solo la decodifica della matrice biochimica.

### Collegamenti con altre proposte di design

- **Cratere profondo come sito del "Precursore".** In un documento di design separato (proposte per rendere il gioco più coinvolgente nei primi minuti) è stata proposta una singola cella fissa per mondo con una chimica anomala costante, che agisce come polo di attrazione narrativo. Il cratere profondo è il luogo naturale per farla coincidere fisicamente: invece di due elementi scollegati sulla mappa (un'anomalia invisibile e un bioma), diventano un unico luogo con più strati di mistero da scoprire insieme.
- **Bocca vulcanica / Geyser e le sorgenti di calore puntiformi.** Se l'ambiente passa da gradienti fissi su assi a sorgenti puntiformi che diffondono (proposta discussa in parallelo a questo documento), questi due biomi sono l'implementazione visiva di quell'idea — non sono biomi "a sé stanti" scollegati dal resto, sono il modo in cui il giocatore vede e localizza le sorgenti sulla mappa.
- **Roccia nuda e il futuro metabolismo chemiolitotrofo.** Il GDD (§5.4) menziona come estensione futura un metabolismo legato alla tossicità. La roccia nuda, con `toxicity 0.30` e bassa risorsa organica altrove, è la nicchia naturale per quel metabolismo quando verrà implementato — non richiede altro lavoro ora, ma va tenuto presente nella scelta dei valori.

## Overlay: alberi

Gli alberi **non sono un bioma**, sono un livello di decorazione indipendente sopra il bioma:

- **Densità sparsa** su: Pianura, Collina, Montagna, Palude.
- **Densità densa** su: Foresta (è la caratteristica distintiva di quel bioma).
- **Assenti** su tutti gli altri biomi (acqua in ogni sua forma, vetta, roccia nuda, cratere, deserto, bocca vulcanica, geyser, tundra, distesa di cristalli) — coerenza con le condizioni ambientali di ciascuno (troppo freddo, troppo arido, nessun suolo).

## Immagini

### Tavola di riferimento

**Immagine allegata separatamente: `biome-reference-sheet.svg`** — Tavola di riferimento con tutti i 16 biomi, colore piatto con dithering, overlay alberi mostrato dove applicabile

Ogni bioma mostrato come swatch isolato: colore piatto a due toni (dithering), nome, e overlay alberi dove previsto (sparso su pianura/collina/montagna/palude, denso su foresta).

### Mappa d'esempio composta

**Immagine allegata separatamente: `biome-example-map.svg`** — Mappa d'esempio 128x80 (dimensione griglia reale in uso) con tutti i biomi composti insieme in una configurazione plausibile

Mostra come i biomi si comporrebbero insieme su una mappa reale: costa a ovest (acqua profonda/bassa), una catena montuosa diagonale con vette e colline di transizione, un lago interno circondato da palude, foreste in due punti separati, deserto e tundra agli angoli opposti (caldo/freddo), roccia nuda vicino ai rilievi, un cratere profondo con una bocca vulcanica al centro, un geyser ai piedi della montagna, e una piccola distesa di cristalli isolata. **La disposizione specifica è solo dimostrativa** (generata con funzioni geometriche di prova per mostrare come i biomi si affiancano in modo plausibile) — la generazione reale dipenderà dall'algoritmo di world-gen del gioco.

**Nota — discrepanza con il GDD:** il documento di design (§5.1) dichiara una griglia **48×32** come dimensione di partenza. La mappa d'esempio qui usa **128×80**, la dimensione effettivamente in uso. Vale la pena verificare se il GDD è semplicemente non aggiornato rispetto a una modifica successiva, o se le due fonti sono scollegate per un altro motivo — non è qualcosa che questo documento può risolvere da solo, ma va segnalato per evitare che altri task si basino sul valore sbagliato.

## Cosa serve per l'integrazione (per chi implementa)

- **Dato sorgente per l'assegnazione del bioma:** ogni cella deve determinare il proprio bioma a partire da una combinazione di elevazione (se/quando esisterà un campo dedicato, vedi il documento sulla mappa a fasce di elevazione) e dai tre scalari ambientali già esistenti (`temperature`, `light`, `toxicity`). La tabella sopra fornisce i valori target per bioma; il verso opposto (dedurre il bioma dai valori di una cella generata) va progettato in coerenza con quella tabella.
- **Rendering:** riusare esattamente le stesse tecniche già stabilite per la mappa a fasce di elevazione (bordi netti sul confine tra celle di bioma diverso, coste più marcate dei confini interni, dithering a due toni, overlay a tinta piatta invece di blend) — nessuna tecnica nuova, solo applicata a più categorie.
- **Overlay alberi:** livello di rendering separato dal colore del bioma, con probabilità/densità di occorrenza per cella secondo la regola sparso/denso/assente sopra.
- **Geyser come elemento animato:** a differenza degli altri biomi (statici), il geyser è l'unico pensato esplicitamente per pulsare nel tempo (collegato alla proposta di far "respirare" l'ambiente anche a griglia vuota) — se quella proposta viene implementata, il geyser è il primo candidato a cui applicarla.

## Fuori scope

- Algoritmo di generazione procedurale reale dei biomi (qui solo un esempio dimostrativo, non un algoritmo di produzione).
- Implementazione delle sorgenti di calore puntiformi come sistema (bocca vulcanica/geyser qui sono solo la rappresentazione visiva del bioma; il sistema di diffusione da sorgenti è materia di un altro task).
- Il metabolismo chemiolitotrofo legato alla roccia nuda (resta un'estensione futura come già indicato nel GDD, §5.4).
- Bilanciamento numerico fine dei valori ambientali per bioma — sono un baseline di partenza, da validare in playtesting.
