# Abiogenesis — punti aperti da discutere

Documento autonomo di tracciamento. Raccoglie ciò che **non è ancora stato deciso** e merita una discussione dedicata. Non contiene decisioni: è la lista di cosa manca. Aggiornare o svuotare man mano che i punti vengono chiusi.

## 1. Revisione della formula del tick — **RISOLTO**

Chiuso. La formula non va sostituita ma **ristrutturata come pipeline a fasi con tre output** (energia, pressione selettiva, osservazioni/eventi), che oggi sono calcolati o assunti in punti separati a partire dagli stessi valori intermedi. Il bioma entra nella formula **come vincolo e come costo** (gate di abitabilità, `crowd_factor` per bioma, decadimento del residuo per bioma) ma **mai come guadagno**, per non duplicare informazione già portata dagli scalari. Energia e pressione selettiva restano accumulatori distinti: condividono gli input, non il valore.

Specifica applicabile in `abiogenesis-tick-pipeline.md`; sintesi recepita nel GDD §5.6.

## 2. Aree trasversali — **AFFRONTATE**

Tutte trattate in `abiogenesis-cross-cutting.md` (salvataggio, audio, accessibilità cromatica, performance, lingua, impostazioni) e in `abiogenesis-transitions-metaprogression.md` (transizione tra mondi, fine run, meta-progressione). Sintesi recepita nel GDD §14.

Restano aperti, dentro quei temi, solo i dettagli esecutivi esplicitamente rimandati:

- **Formato di serializzazione e versionamento** degli snapshot.
- **Motore e tecniche di sintesi audio** — definita solo la direzione generativa.
- **Palette alternativa concreta** per l'accessibilità cromatica — la regola dei due canali viene comunque prima.
- **Budget di performance numerici** — da stabilire profilando, non a tavolino.
- **Contenuto concreto della meta-progressione** (liste di sblocchi, soglie, curva) — categorie e criterio definiti, contenuto rimandato post-MVP per scelta.

## 2b. Estetica — **AFFRONTATA**

Direzione, materiale del notebook (ardesia), regola colore/forma e rendering a grana pixel (forme organismo su griglia + texture procedurale sui biomi + archi a scalino nel grafo) decisi in `culture-shock-population-model-aesthetic.md`, recepiti nel GDD §11. Il principio "ogni elemento visivo deve rappresentare un dato reale" resta candidato a pilastro esplicito — non formalizzato come tale, decisione aperta minore.

## 2d. Strumento di ispezione — **AFFRONTATO**

Buco identificato in una revisione precedente e mai più ripreso: nessuna lettura diretta del bilancio energetico di una cella. Risolto in `culture-shock-inspect-tool.md` — azione gratuita a due livelli (hover leggero + card completa), fuori dal budget, nessun calcolo nuovo (espone dati già prodotti dalla pipeline del tick).

## 2c. Modello di popolazione per cella — **AFFRONTATO**

Cambio di modello (popolazione con capacità e sfondamento invece di un organismo per cella) deciso in `culture-shock-population-model-aesthetic.md`, recepito nel GDD §5. Restano aperti solo i valori numerici (capacità portante per cella/bioma) e il dettaglio implementativo della ricerca di cella per lo sfondamento — segnalati come fuori scope in quel documento, da bilanciare in playtest.

## 3. Decisioni minori rimaste aperte nei documenti esistenti

- **Effetto al completamento di un obiettivo** — nessun effetto, o rinforzo narrativo ancorato al traguardo appena raggiunto (documento obiettivi, proposta non decisa).
- **Adozione di Isola e Sposta** come quinta/sesta azione (documento azioni, raccomandate/discusse ma non decise).
- **Conversione puntuale degli obiettivi a durata** da ere a stagioni, voce per voce (documento obiettivi, indicata come direzione).
- **Sostituzione di "Fotosoma"** con un termine strettamente reale, se si vuole il vincolo di autenticità biochimica al 100% (documento archetipi biochimici).
- **Tratti attivi per mondo** — la proposta di rivedere la curva (4 in world 0, tetto a 9, progressione graduale) è esplicitamente da validare in playtest, non equiparabile alle altre correzioni.


## Stress-test dell'onboarding con giocatore ignaro — **AFFRONTATO, priorità assegnata**

`culture-shock-naive-player-example.md` ha identificato 5 punti di attrito; `culture-shock-friction-fixes.md` formalizza le 4 correzioni come specifica implementabile e le colloca come nuova **Fase 1b** nel piano di lavorazione (`abiogenesis-INDEX.md`), subito dopo il checkpoint di playtest della Fase 1. Priorità: alta per l'indicatore di saturazione (l'unico invisibile per progettazione, non solo poco chiaro), media per gli altri due, bassa-media per la traduzione dei codici nel log. Resta aperto solo ciò che nessun documento può chiudere da solo: se queste correzioni bastino davvero, verificabile solo con un vero playtest — raccomandato esplicitamente come passo prima di proseguire oltre la Fase 1b.

## Pilastro 5 — meraviglia e scoperta

Aggiunto al GDD come quinto pilastro esplicito. `culture-shock-wonder.md` formalizza il brainstorm (piccole/medie/grandi/stranissime) con priorità sulle prime tre da spingere (tasche di anomalia sparse, tracce fossili, estremofilo). Nessuna decisione numerica presa — tutto da validare in playtest, con cautela particolare sulle voci esplicitamente segnalate come delicate da bilanciare in frequenza (piste false, mistero mai risolto per mondo).

## Firme di bioma ed eventi cosmici — nuovo, priorità assegnata

`culture-shock-biome-cosmic-events.md` specifica trigger/meccanismo/dipendenze per 11 eventi (3 generici, 5 firme di bioma, 4 di origine cosmica), tutti riusando sistemi già esistenti (pipeline del tick, bias di famiglia, xenotratti, Precursore). Priorità alta: pioggia di micrometeoriti (identica alle tasche di anomalia sparse di `culture-shock-wonder.md`, solo narrata diversamente) e le firme di Palude/Vetta. Nessun valore numerico deciso — tutto da validare in playtest.