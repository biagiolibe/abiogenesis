# Abiogenesis — Game Design Document

**Working title:** Abiogenesis *(provvisorio, da confermare)*
**Genere:** Roguelike di simulazione emergente / laboratorio di xenobiologia
**Piattaforma:** Desktop, finestra grafica 2D
**Tech:** Rust + Bevy (ECS)
**Modalità:** Single-player, a ere, con obiettivi e meta-progressione leggera
**Stato del documento:** v0.4 — design pre-implementazione (decisioni chiuse + baseline numerica + esempio di partita)

> Legenda dello stato di ogni scelta:
> **[DECISO]** concordato e stabile · **[PROPOSTA]** baseline che propongo, da approvare/correggere · **[APERTO]** da decidere insieme

### Changelog

- **v0.4** — **Cambio di stack: da terminale (`ratatui`) a finestra grafica 2D con Bevy e modello ECS.** Ne conseguono revisioni a §5.1 (dimensione della griglia), §11 (presentazione) e §12 (stack e architettura), più la correzione di §13 sullo stato dello scaffold. **Il design non cambia:** §§1–10 e §§14–16 — pilastri, core loop, modello di simulazione, formule del tick, baseline numerica §5.9, taccuino, obiettivi, esempio di partita — restano validi parola per parola. Il pilastro 3 ("il divertimento sta nel sistema, non nella grafica") resta pienamente in vigore: quadrati colorati, zero asset artistici.
- **v0.3** — Decisioni di design chiuse, baseline numerica (§5.9), esempio di partita (§16).

---

## 1. Visione

Sei uno xenobiologo che semina la vita su mondi alieni e deve scoprire, per esperimenti, quale biochimica ne emerge. Ogni mondo ha **regole di interazione biochimica nascoste e diverse**: il gioco consiste nel fare reverse-engineering di quelle regole seminando organismi, osservando l'ecosistema che vive di vita propria, formulando ipotesi e verificandole con interventi mirati — il tutto per raggiungere gli obiettivi che ogni mondo pone.

Il piacere centrale è il doppio mistero: il mistero di **cosa succederà** (l'emergenza dinamica di un ecosistema imprevedibile) sovrapposto al mistero delle **regole stesse** (la deduzione della matrice biochimica segreta). È Liu Cixin in forma di piastra di Petri.

### Pilastri di design **[DECISO]**

1. **Imprevedibilità che nasce dalle mie scelte.** L'evoluzione non è scriptata: emerge dalle decisioni del giocatore incrociate con regole nascoste. Nessuna partita è uguale.
2. **Scoperta come progressione.** Non si avanza accumulando numeri, ma *capendo*. Il taccuino che si riempie è la barra di avanzamento.
3. **Il divertimento sta nel sistema, non nella grafica.** Presentazione minimale (quadrati colorati, zero asset artistici), profondità nella simulazione.
4. **Rigiocabilità dalla generazione procedurale vincolata.** Le regole di ogni mondo sono generate, non scritte a mano: contenuto infinito senza mesi di authoring.

### Cosa NON è **[DECISO]**

- Non è un clicker/idle a numeri crescenti.
- Non è un sandbox puramente contemplativo: c'è pressione, obiettivi, vittoria e sconfitta. *(Abbiamo esplicitamente scelto la versione con obiettivi rispetto alla versione zen.)*
- Non è un gioco a forte impatto grafico.

---

## 2. Il giocatore e la fantasia

**Fantasia:** "Sono uno scienziato di fronte a una biochimica aliena che non capisco, e la decifro coltivandola." Curiosità, metodo scientifico, quel momento in cui un'ipotesi si conferma e un pezzo di sistema si illumina.

**Giocatore-tipo:** chi ama i puzzle di deduzione, la simulazione emergente (Dwarf Fortress in miniatura), la hard-SF, l'ottimizzazione di sistemi. Persone che trovano ipnotico osservare un sistema che si auto-organizza.

---

## 3. Core loop **[DECISO]**

Il ciclo fondamentale, ripetuto dentro ogni mondo:

1. **Semina** — collochi organismi sulla griglia (cosa, dove, quando).
2. **Avanza un'era** — la simulazione procede di *N* tick; osservi l'ecosistema muoversi ed evolvere.
3. **Registra** — annoti nel taccuino cosa hai osservato (chi è fiorito, chi è collassato, quali adiacenze sembrano avere effetto).
4. **Ipotizza & interviene** — formuli un'ipotesi sulle regole nascoste e la metti alla prova con un intervento mirato (stress ambientale, rimozione, mutazione, nuova semina).
5. **Obiettivo** — quando l'obiettivo del mondo è soddisfatto, passi al mondo successivo, più profondo e più ostile.

Il fulcro esperienziale è il sottociclo **ipotesi → esperimento → *aha***, incastonato nell'imprevedibilità di un ecosistema che vive da solo.

---

## 4. Modello del tempo: le ere **[DECISO]**

Il tempo avanza a **ere**: il giocatore mette in coda una o più azioni, poi fa avanzare la simulazione di *N* tick in blocco e osserva il risultato. Questo fa **coincidere il modello del tempo col loop mentale**: pianifichi (ipotesi), esegui (era), osservi (risultato).

- **Animazione durante l'era [DECISO]:** l'avanzamento dei tick viene mostrato tick-per-tick (rapido), così si conserva la sensazione di "sistema che respira", ma il *controllo* resta a scatti deliberati.
- **Lunghezza dell'era [DECISO struttura / coefficiente da tarare]:** `ERA_TICKS = 25` come default, regolabile; esponibile al giocatore come scelta ("avanza di 1 / 10 / 25 tick"). Il valore preciso è un coefficiente da validare in playtest.
- **Modalità real-time [APERTO / futuro]:** sopra questa architettura costa poco aggiungerla in seguito come opzione. Non è nell'MVP.

---

## 5. Modello di simulazione

### 5.1 Griglia e celle **[DECISO]**

Griglia 2D di celle. Ogni cella contiene:

- uno **strato ambientale** (poche scalari continue);
- al più **un organismo** (occupazione singola per cella), con un livello di energia/popolazione.

**Dimensione [DECISO / dimensione finale empirica]:** **48×32** fin da subito, come costante di configurazione. *(Revisione v0.4: la vecchia scaletta "40×20 in Fase 0 → rendering half-block per arrivare a ~48×32" era interamente un vincolo di larghezza del terminale — 80 colonne a 2 caratteri per cella. Con una finestra grafica quel vincolo decade e si parte direttamente dal target.)* Più spazio serve all'emergenza: i pattern spaziali (tipo Lotka–Volterra) hanno bisogno di respiro e le griglie troppo piccole muoiono per rumore stocastico. La dimensione finale resta in parte empirica.
**Vicinato [DECISO]:** Moore (8 vicini) per interazioni e riproduzione.

### 5.2 Strato ambientale **[DECISO struttura / parametri in §5.9]**

Poche scalari per cella, in `[0,1]`:

- `temperature`
- `light`
- `toxicity`

**Fase 0:** gradienti statici (es. luce alta in alto, temperatura su un asse diverso) per creare eterogeneità spaziale → nicchie.
**Fase 1+:** diffusione lenta delle scalari (media coi vicini a rate basso), così gli interventi ambientali si propagano nel tempo.

### 5.3 Genoma di specie **[DECISO]**

Ogni specie è definita da un piccolo genoma:

- **Metabolismo** (uno tra i tipi sotto) — come ricava energia.
- **Intervallo ambientale preferito** — es. `temp_optimum` + `temp_tolerance` (fitness gaussiana attorno all'ottimo).
- **Soglia di riproduzione** — energia oltre la quale si riproduce.
- **Da 1 a 3 tag biochimici** — *l'unica cosa che conta per le interazioni tra specie.*

I metabolismi e gli intervalli ambientali sono **leggibili** (ancore per il giocatore). I tag sono **opachi** (vedi §5.5).

### 5.4 Metabolismi **[DECISO]**

- **Fotolitico** (`Photolithic`) — ricava energia dalla `light` locale. È il produttore primario.
- **Predatore** (`Predator`) — ricava energia dagli organismi vicini (ne consuma l'energia).
- **Decompositore** (`Decomposer`) — ricava energia dalla materia morta / residui.

*(Set iniziale. Se ne possono aggiungere altri — es. chemiolitotrofo legato alla toxicity — come contenuto sbloccabile.)*

### 5.5 Tag e matrice nascosta **[DECISO]** — *il cuore del gioco*

Si distinguono due livelli, per non confondere varietà e difficoltà:

- **Pool globale di tag [DECISO]:** ~**10 glifi** biochimici totali nel gioco, per dare varietà visiva tra i mondi.
- **Tag attivi per mondo [DECISO]:** solo un sottoinsieme è effettivamente in gioco in un dato mondo. **La difficoltà cresce aumentando i tag attivi, non il pool.** Baseline: **5 tag attivi** nei primi mondi, fino a **~8** nei mondi tardi.

Ogni specie porta 1–3 tag (tra quelli attivi nel mondo).

All'inizio di ogni mondo si sorteggia una **matrice segreta `tag × tag`**: per ogni coppia ordinata di tag, un effetto (positivo/negativo, con intensità) che si applica quando due organismi che portano quei tag sono **adiacenti**.

- **Effetti direzionali [DECISO]:** la matrice è **asimmetrica** — "A avvelena B" non implica "B avvelena A". Quindi le relazioni fuori diagonale sono ~**T²**: con 5 tag attivi ≈ **20 relazioni** (decifrabili in una run); con 8 ≈ **56** (troppe da decodificare *tutte*, ma non serve — al giocatore basta la parte rilevante per l'obiettivo e per le specie in campo). È una generalizzazione direzionale di sasso-carta-forbici: chi catalizza chi, chi avvelena chi.
- I tag sono mostrati come **glifi/colori alieni senza nome**: il giocatore ne impara l'effetto solo empiricamente.
- La matrice è **la cosa che il giocatore decodifica** per esperimenti. È il mistero delle regole.
- **Grado di opacità [DECISO]:** la matrice parte nascosta, ma **si rivela progressivamente** man mano che il taccuino conferma le relazioni (vedi §7). Metabolismi e intervalli ambientali restano sempre leggibili come ancore. La rivelazione progressiva del taccuino *è* la soluzione all'opacità — non sono due meccaniche separate.

### 5.6 Algoritmo del tick **[DECISO struttura / coefficienti da tarare]**

La **struttura** è decisa; i **coefficienti numerici** sono una baseline da validare in playtest (il tuning è il lavoro vero — vedi §13). Due decisioni di struttura, che sono scelte di *design* e non di tuning:

- **Effetto matrice additivo e lineare [DECISO]:** l'effetto di adiacenza è **additivo e indipendente per ogni coppia adiacente** (ogni A-vicino-a-B = ±k fisso, lineare nel numero di vicini), **non moltiplicativo**. Il moltiplicativo sarebbe più "realistico" ma accoppia gli effetti e rende la deduzione quasi impossibile; l'additivo è *leggibile* — il giocatore può ragionare "ogni A accanto mi costa circa 2 di energia". Il design serve alla deduzione.
- **Coefficienti centralizzati [DECISO]:** tutti i coefficienti sono costanti nominate in un unico punto (o file di config, idealmente ricaricabile a caldo), così il tuning finale è rapido.


Per ogni cella occupata:

1. **Fitness ambientale:** `env_fit = gaussian(temperature, temp_optimum, temp_tolerance)` ∈ `[0,1]`.
2. **Guadagno metabolico** (dipende dal metabolismo):
   - *Fotolitico:* `gain = light * metabolism_gain * env_fit`.
   - *Predatore:* preleva energia dai vicini occupati (entro un cap), pesata da `env_fit`.
   - *Decompositore:* preleva dai residui/materia morta nella cella o nei vicini.
3. **Effetto della matrice nascosta:** per ogni vicino occupato, per ogni coppia di tag (mio × suo), somma l'effetto dalla matrice segreta → `interaction_delta` (può essere + o −).
4. **Costi:** `upkeep` (costo base per tick) + `crowding_penalty = crowd_factor * n_vicini_occupati` (carrying capacity).
5. **Aggiornamento energia:** `energy += gain + interaction_delta − upkeep − crowding_penalty`.
6. **Morte:** se `energy <= 0` → l'organismo muore (la cella si libera; opzionale: lascia residui per i decompositori).
7. **Riproduzione:** se `energy >= repro_threshold` ed esiste un vicino vuoto → genera un figlio in un vicino vuoto (scelta casuale seedata) con `repro_cost` energia, sottratta al genitore. *(Fase futura: possibilità di mutazione del genoma del figlio.)*

**Ordine di elaborazione [PROPOSTA]:** iterazione in ordine mescolato (seedato) con guardia "nato/agito in questo tick" perché i neonati non agiscano nello stesso tick; oppure doppio buffer (snapshot → next). Da scegliere in implementazione privilegiando correttezza e determinismo.

### 5.7 Determinismo **[DECISO]**

La simulazione è **deterministica** a parità di seed: RNG seedato conservato nello stato del mondo. Fondamentale per debug dell'emergenza, riproducibilità dei bug e (in prospettiva) condivisione di seed interessanti.

### 5.8 Anti-degenerazione **[DECISO]** — *la difesa contro il rischio principale*

Il rischio numero uno dell'emergenza è il collasso in due esiti noiosi: **"muore tutto"** oppure **"una specie domina"**. Leve sistemiche per evitarli:

- **Vincolo di ciclicità sulla matrice:** la generazione garantisce almeno una relazione **ciclica non transitiva** (A batte B, B batte C, C batte A). Matematicamente è ciò che sostiene la coesistenza.
- **Eterogeneità ambientale → nicchie:** gradienti/diffusione fanno sì che specie diverse prosperino in zone diverse.
- **Carrying capacity:** la penalità da affollamento impedisce la crescita illimitata di una singola specie.

Queste tre manopole sono il principale oggetto del tuning finale.

### 5.9 Costanti di partenza (baseline) **[baseline plausibile / da validare in playtest]**

Valori iniziali coerenti tra loro (verificati concettualmente perché una fioritura fotolitica cresca in spazio libero e si stabilizzi per affollamento, e perché un −2 di matrice sia visibile ma non insta-letale). In implementazione vivono tutti in un unico punto di config (§5.6), idealmente ricaricabile a caldo.

**Ambiente**

| Costante | Valore | Note |
|---|---|---|
| Scala scalari | `[0,1]` | temperature, light, toxicity |
| Diffusione ambientale (Fase 1+) | `0.05` / tick | blend lento con la media dei vicini |
| Gradiente luce (Fase 0) | `0.9` (alto) → `0.2` (basso) | crea nicchia verticale |
| Gradiente temperatura (Fase 0) | `0.2` (sx) → `0.8` (dx) | crea nicchia orizzontale |
| Zona tossica | `toxicity = 0.7` | resto `0.0` |

**Tempo e azioni**

| Costante | Valore | Note |
|---|---|---|
| `ERA_TICKS` | `25` | tick per era |
| Budget ere / mondo | `40` (iniziali) → `25` (tardi) | finito: dà tensione roguelike |
| Budget punti / era | `3` | |
| Costo azioni | seed `1`, stress `1`, cull `1`, splice `2` | splice tarabile fino a `3` |

**Energia e metabolismo** (per organismo)

| Costante | Valore | Note |
|---|---|---|
| Energia al seed | `5.0` | |
| `upkeep` base | `0.5` / tick | costo di mantenimento |
| `crowd_factor` | `0.15` / vicino occupato | carrying capacity |
| `repro_threshold` | `10.0` | energia per riprodursi |
| `repro_cost` (al figlio) | `5.0` | sottratta al genitore |
| Fotolitico `metabolism_gain` | `2.0` | `gain = light · gain · env_fit` |
| Predatore `drain_cap` | `2.0` / tick, `upkeep 0.7` | preleva dai vicini |
| Decompositore `extract_rate` | `1.5` / tick, `upkeep 0.5` | dai residui |
| Residuo alla morte | `3.0`, decade `0.2` / tick | nutre i decompositori |
| `env_fit` | `exp(−(temp−temp_opt)² / (2·temp_tol²))` | `temp_tol` (σ) default `0.15` |

*Verifica rapida:* fotolitico isolato con `light≈0.7`, `env_fit≈1` → `gain≈1.4`, netto `≈+0.9`/tick (cresce); con 6–8 vicini → netto `≈−0.15`/tick (si ferma → carrying capacity). In zona buia (`light 0.2`) → `gain 0.4 < upkeep 0.5` → non sopravvive (nicchia di luce). Predatore senza prede: `gain 0 − upkeep 0.7` → collassa in ~7 tick (dinamica preda-predatore).

**Tag e matrice**

| Costante | Valore | Note |
|---|---|---|
| Pool globale tag | `10` glifi | varietà tra mondi |
| Tag attivi / mondo | `5` (iniziali) → `8` (tardi) | leva di difficoltà |
| Tag per specie | `1–3` | |
| Intensità effetto / adiacenza | interi in `{−2,−1,0,+1,+2}` | additivo (§5.6) |
| Densità matrice | ~`40%` coppie non nulle | resto `0` |
| Vincolo di generazione | ≥1 ciclo RPS negativo garantito | coesistenza (§5.8) |

**Taccuino**

| Costante | Valore | Note |
|---|---|---|
| Soglia di conferma / cella | `3.0` di evidenza cumulata | |
| Peso di un'osservazione | `1 / (1 + n_confonditori_adiacenti)` | premia gli esperimenti puliti |
| Stati cella | `?` sconosciuto · `0` nessun effetto · `±?` ipotesi · `±!` confermato | |

**Griglia**

| Costante | Valore | Note |
|---|---|---|
| Dimensione | `48×32` | dalla Fase 0 (v0.4) |
| Vicinato | Moore (8) | |

---

## 6. Azioni del giocatore (interventi) **[DECISO]**

Le azioni sono ciò che il giocatore mette in coda prima di far avanzare un'era:

- **Semina** (`Seed`) — colloca un organismo di una specie disponibile in una cella.
- **Stress ambientale** (`Stress`) — altera una scalare ambientale in un'area (es. alza toxicity, abbassa temperature).
- **Rimozione / cull** (`Cull`) — elimina un organismo o una specie in un'area.
- **Mutazione / splice** (`Splice`) — modifica il genoma di una specie (es. cambia/aggiunge un tag, sposta l'ottimo termico). È lo strumento sperimentale più potente e più costoso.

**Budget di azioni per era [DECISO struttura / baseline in §5.9]:** un budget stretto a punti per era rafforza il modello mentale "un'era = un esperimento deliberato" — non puoi tappezzare la griglia, devi scommettere sull'ipotesi migliore. Baseline: **3 punti per era**, costi **seed 1, stress 1, cull 1, splice 2** (la mutazione è lo strumento più potente, quindi il più caro; tarabile fino a 3). Col budget 3 e splice 2 puoi combinare uno splice + un'azione economica, oppure tre azioni economiche: la scelta è interessante. I numeri esatti si affinano in Fase 3.

---

## 7. Lo strato di scoperta: il taccuino **[DECISO]**

È il cuore della progressione (§ pilastro 2) e trasforma l'osservazione in *gioco di deduzione*.

- **Log delle osservazioni:** eventi salienti registrati era per era (fioriture, collassi, estinzioni, adiacenze notevoli).
- **Griglia di ipotesi:** una vista della matrice `tag × tag` dove il giocatore annota le proprie congetture sugli effetti; il gioco segna quali celle sono **confermate** dall'evidenza raccolta.
- **Catalogo tag/specie:** glifi alieni incontrati e ciò che se ne sa finora.

**Modello di conferma [DECISO] — "B con sfumatura di C":** il gioco **accumula evidenza** dalle osservazioni e **conferma una cella** della matrice quando l'evidenza supera una soglia. La cella confermata si "illumina": è l'*aha* esplicito e la barra di progresso del pilastro 2. La sfumatura di C premia il **buon metodo sperimentale**: un'osservazione *pulita* (adiacenza isolata, tipicamente prodotta da un esperimento deliberato) **pesa molto più** di una confusa. Concretamente il peso dell'evidenza è **inversamente proporzionale al numero di altri tag adiacenti** che potrebbero confondere il segnale — `peso = 1 / (1 + n_confonditori)` — e una cella si conferma a **evidenza cumulata ≥ 3.0** (baseline, §5.9). Così non serve costruire un fragile rilevatore di "esperimento pulito": si pesa semplicemente per quanti confonditori erano presenti (un'osservazione isolata vale 1.0, una con tre tag confondenti vale 0.25). Questa meccanica **è** la rivelazione progressiva della matrice (§5.5): non c'è una seconda meccanica di opacità separata.

---

## 8. Obiettivi, vittoria e sconfitta **[DECISO]**

Ogni mondo pone una o più **richieste esplicite**. Esempi del tipo di obiettivo:

- "Ottieni una biosfera con **≥3 specie coesistenti** per **50 tick**."
- "Coltiva una specie che **sopravvive nella zona tossica**."
- "**Innesca una fioritura** di un tipo specifico."

- **Successo** → si passa al mondo successivo (più tag attivi, matrice più cattiva, ambiente più ostile).
- **Fallimento** → la run termina; nuova run = nuova biochimica.

**Condizioni di fallimento [DECISO]:**

- **Estinzione totale** → fallimento immediato (il pavimento ovvio).
- **Budget di ere per mondo** generoso ma **finito** (baseline: 40 ere nei mondi iniziali, che scende verso 25 nei mondi tardi): un giocatore bloccato alla fine fallisce invece di macinare all'infinito. È ciò che dà la tensione roguelike.

**Obiettivi bonus [DECISO come direzione / bassa priorità]:** previsti in linea di principio (danno currency di meta-progressione), ma **dopo** il core pulito "obiettivo primario → avanzi". Non nell'MVP minimo.

---

## 9. Generazione dei mondi e curva di difficoltà **[DECISO]**

Ogni mondo è generato proceduralmente:

- **Matrice biochimica** nuova (asimmetrica, con il vincolo di ciclicità di §5.8).
- **Ambiente** (gradienti, zone estreme) con ostilità crescente.
- **Tag attivi** (sottoinsieme del pool) e **specie di partenza** disponibili.
- **Obiettivo/i** del mondo.

**Curva [DECISO direzione]:** i primi mondi con **5 tag attivi** e ambiente mite; via via fino a **~8 tag attivi**, matrici con più relazioni "cattive", ambienti più estremi (zone tossiche ampie, gradienti termici severi), obiettivi più stringenti e budget di ere più corto.

La **biochimica è fresca a ogni run**: la rigiocabilità nasce da qui, non da contenuto scritto a mano.

---

## 10. Meta-progressione **[DECISO leggera / persistenza rimandata post-MVP]**

Progressione *tra* le run, deliberatamente **leggera**:

- Sblocco di **più specie di partenza** o **strumenti** (es. un'azione in più, o un tag noto).
- La **matrice resta sempre da decifrare** da capo: non si sbloccano "risposte", si sbloccano *capacità*.

**Persistenza [DECISO: rimandata]:** l'MVP si costruisce **senza persistenza** (tutto dentro una run). Si decide se salvare gli sblocchi (profilo/save) **solo dopo** aver verificato che il loop è divertente. È banale da aggiungere in seguito e non vincola l'architettura.

---

## 11. Presentazione e UX **[DECISO direzione]**

*(Revisione v0.4: la resa passa dal terminale a una finestra grafica 2D. Il pilastro 3 non cambia — niente asset artistici, solo quadrati colorati: la finestra dà più spazio e una UI leggibile, non "grafica".)*

- **Resa:** finestra 2D. Griglia di celle come quadrati colorati.
  - Celle occupate: colore = specie/tag; luminosità = energia.
  - Celle vuote: sfondo tenue che riflette l'ambiente (es. luminosità = `light`).
- **Tag alieni:** glifi/colori senza nome, imparati empiricamente.
- **Pannelli UI:** tick corrente, numero d'era, popolazioni per specie, energia media, obiettivo corrente, budget azioni, hint dei comandi.
- **Taccuino:** finestra dedicata (log + griglia ipotesi `tag × tag` + catalogo). La griglia di ipotesi è una tabella densa e interattiva: è il caso d'uso per cui la UI immediata (egui) è nettamente più adatta di una UI a widget persistenti.

### Controlli **[PROPOSTA]**

- `space` — avanza di un'era (*N* tick).
- `s` — avanza di un singolo tick (osservazione fine / debug).
- (Fase 2+) tasti per entrare in modalità azione: semina, stress, cull, splice; **selezione della cella col mouse** (frecce come alternativa da tastiera).
- `tab` — apri/chiudi taccuino.
- `r` — reset / reseed del mondo.
- `Esc` — esci.

---

## 12. Stack tecnico **[DECISO]**

*(Revisione v0.4: da `ratatui`/TUI a Bevy con modello ECS.)*

- **Linguaggio:** Rust, edizione 2021 (codice e commenti in inglese). Toolchain pinnata a **1.97.1**.
- **Engine:** **Bevy 0.19** — ECS, scheduling, stati, plugin, input, finestra, resa 2D.
- **UI:** **`bevy_egui` 0.41** (egui 0.35) per HUD e taccuino.
- **RNG:** `rand` con seed esplicito conservato nello stato del mondo.
- **Architettura [DECISO]:** un modulo = un `Plugin` Bevy — `ConfigPlugin`, `WorldPlugin`, `SimPlugin`, `GridRenderPlugin`, `UiPlugin`, `InputPlugin`. La simulazione è separata dalla resa e dagli input.
  - **La griglia è una `Resource`, non entità ECS.** Lo stato vive in `SimWorld` come array densi con doppio buffer; le entità Bevy esistono **solo per la resa** (uno sprite per cella, sincronizzato in sola lettura). Il motivo è il determinismo di §5.7: l'iterazione parallela delle query ECS è il modo più rapido per perderlo.
  - **La logica del tick è Rust puro**, invocabile senza `App` Bevy: è ciò che rende testabili headless il determinismo e il bilanciamento (§5.8) e che rende praticabile il tuning finale.

Il dettaglio architetturale — stati, `SystemSets`, eventi, invarianti — vive in `TECH_DESIGN.md`, non qui.

---

## 13. Piano di sviluppo (~2 settimane, a fasi) **[DECISO]**

L'implementazione vera avverrà in Claude Code, con questo GDD come riferimento.

### Fase 0 — Scheletro camminante *(~2–3 giorni)*
Griglia + ambiente (gradienti statici) + **un** metabolismo (fotolitico) + riproduzione + morte + resa a sprite colorati in finestra 2D + HUD + avanzamento per era / singolo tick.
**Traguardo:** guardi una specie fotolitica fiorire e stabilizzarsi grazie alla carrying capacity. *(Revisione v0.4: la v0.3 dava il progetto per già scaffoldato — non lo era. Lo scaffold Cargo + app Bevy è il primo task della fase, insieme all'aggiornamento della toolchain da 1.90 a 1.97.1, richiesto da Bevy 0.19.)*

### Fase 1 — Emergenza *(~3–4 giorni)*
Tag + **matrice nascosta** + specie multiple + predazione e decomposizione + azione **semina**.
**Traguardo:** appare l'emergenza vera; più specie interagiscono via matrice.

### Fase 2 — Deduzione *(~3–4 giorni)*
Taccuino + log osservazioni + griglia di ipotesi + azioni **stress / cull / splice**.
**Traguardo:** nasce il *gioco* di deduzione, non solo la simulazione.

### Fase 3 — La run *(~2–3 giorni)*
Sistema di **obiettivi** + generazione dei mondi + win/lose + flusso della run e meta-progressione minima.
**Traguardo:** un ciclo di gioco completo, mondo dopo mondo.

### Tuning finale *(tempo residuo)* — *l'arte vera*
Bilanciamento dell'emergenza: le manopole di §5.8 (ciclicità, eterogeneità, carrying capacity) + le formule del tick (§5.6). Obiettivo: emergenza *interessante e leggibile*, evitando "muore tutto" e "uno domina".

---

## 14. Rischi e questioni aperte

### Rischio principale **[DECISO come priorità]**
**Rendere l'emergenza interessante invece che noiosa o illeggibile.** È tuning di *sistema*, non arte grafica. Mitigazioni in §5.8. Parte da dinamiche note (predazione/riproduzione su lattice, tipo Lotka–Volterra spaziale) con le regole nascoste come "spezie".

### Rischio di leggibilità
Il massimo mistero (§5.5) può risultare troppo crudo. Mitigazione: rivelazione progressiva della matrice via taccuino; mantenere metabolismi e ambiente sempre leggibili.

### Questioni chiuse
- **Tag:** pool globale ~10, attivi 5→~8; matrice **asimmetrica/direzionale** (§5.5).
- **Griglia:** 48×32 dalla Fase 0 (§5.1).
- **Stack:** Rust + Bevy 0.19 (ECS), finestra 2D, UI `bevy_egui`; griglia come `Resource`, entità solo per la resa (§12).
- **Budget azioni:** punti per era (baseline 3), costi differenziati (§6).
- **Conferma ipotesi:** modello **B con sfumatura di C**, peso inverso ai confonditori (§7).
- **Fallimento:** estinzione totale + budget di ere finito per mondo (§8).
- **Struttura formule tick:** effetto matrice **additivo/lineare**, coefficienti centralizzati (§5.6).
- **Persistenza meta-progressione:** rimandata post-MVP (§10).

### Restano da validare in playtest (coefficienti, non struttura)
- I valori numerici hanno ora una **baseline plausibile in §5.9** (`ERA_TICKS`, budget, soglie di conferma, `metabolism_gain`, `upkeep`, `crowd_factor`, `repro_*`, intensità matrice, ecc.): vanno confermati o ritoccati col playtest, non reinventati.
- Dimensione griglia definitiva (parte empirica).
- **Titolo definitivo** (non urgente; "Abiogenesis" è il segnaposto).

---

## 15. Glossario

- **Tick:** unità atomica di simulazione.
- **Era:** blocco di *N* tick fatto avanzare in un colpo; l'unità di interazione del giocatore.
- **Tag:** marcatore biochimico astratto di una specie; unica cosa che conta per le interazioni tra specie.
- **Matrice nascosta:** tabella segreta `tag × tag` degli effetti di adiacenza, diversa per ogni mondo.
- **Metabolismo:** come una specie ricava energia (fotolitico / predatore / decompositore).
- **Carrying capacity:** tetto di popolazione imposto dalla penalità da affollamento.
- **env_fit:** idoneità ambientale di un organismo alla cella in cui si trova.

---

## 16. Anatomia di una partita (esempio illustrato)

Un **Mondo 1** di esempio, per mostrare come i sistemi si intrecciano nel gioco reale. Le griglie qui sono ridotte a **10×6** per leggibilità (in gioco 48×32). I glifi dei tag sono `◆ ○ ▲ ✦ ✚`; in gioco sono simboli alieni senza nome — qui etichettati per il lettore. Legenda delle griglie: `.` cella vuota · lettera = organismo di quella specie · `+` = residuo di un organismo morto.

### 16.1 Setup del mondo

**Ambiente** (Fase 0, gradienti statici): la luce cala dall'alto verso il basso, la temperatura cresce da sinistra (freddo) a destra (caldo). Ne risultano nicchie spaziali.

```
        freddo ───────────────▶ caldo
        col:  0  1  2  3  4  5  6  7  8  9
 luce▲  r0    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·   luce alta (0.9)  ┐
 alta│  r1    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·                    │ fascia
     │  r2    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·   luce media       │ vitale
     │  r3    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·                    ┘
 luce│  r4    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·   luce bassa (0.2) → troppo buio
 bassa  r5    ·  ·  ·  ·  ·  ·  ·  ·  ·  ·      per i fotolitici
```

**Palette di specie iniziali** (4 disponibili al giocatore):

| Specie | Metabolismo | Tag | Nota |
|---|---|---|---|
| **P** | Fotolitico | `◆ ○` | Produttore bilanciato, `temp_opt 0.5` |
| **Q** | Fotolitico | `▲` | Amante del caldo, `temp_opt 0.7` |
| **R** | Predatore | `✦` | Preda i vicini |
| **D** | Decompositore | `✚` | Vive sui residui |

**La matrice nascosta** — la "soluzione" del mondo, invisibile al giocatore (riga = tag che *esercita* l'effetto, colonna = tag che lo *subisce*; valore = delta di energia per adiacenza):

| esercita ↓ / subisce → | ◆ | ○ | ▲ | ✦ | ✚ |
|---|---|---|---|---|---|
| **◆** | · | · | **−2** | · | · |
| **○** | · | · | · | +1 | · |
| **▲** | · | · | · | **−2** | · |
| **✦** | **−2** | · | · | · | · |
| **✚** | **+2** | · | · | · | · |

Due strutture nascoste dentro questi numeri:

- **Un ciclo RPS** (i tre `−2` in grassetto): `◆` sopprime `▲`, `▲` sopprime `✦`, `✦` sopprime `◆`. Tradotto in specie: **P sopprime Q, Q sopprime R, R sopprime P**. È il vincolo di §5.8 che impedisce a chiunque di dominare → coesistenza.
- **Un anello mutualistico** (`✚→◆ = +2`): il decompositore **D fertilizza** il produttore **P**. Più `○→✦ = +1`, un effetto minore che fa da "rumore" per rendere la deduzione non banale.

Nota la **direzionalità**: `◆→▲ = −2` ma `▲→◆ = ·` (0). **P danneggia Q, ma Q non danneggia P.** Questa asimmetria è ciò che il giocatore scoprirà per prima.

### 16.2 La partita, era per era

**Era 0 — semina iniziale.** Il giocatore semina P nella fascia mite-luminosa e Q nell'angolo caldo (2 azioni, budget 3).

```
 r0  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r1  ·  ·  ·  ·  P  P  ·  ·  Q  ·
 r2  ·  ·  ·  ·  P  P  ·  ·  Q  ·
 r3  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

**Era 1 — fioritura nelle nicchie.** Entrambe crescono dove l'ambiente le favorisce; le righe buie (r4–r5) restano vuote (luce insufficiente). I due fronti si avvicinano attorno alla colonna 6–7.

```
 r0  ·  ·  ·  P  P  P  ·  ·  Q  ·
 r1  ·  ·  P  P  P  P  ·  Q  Q  Q
 r2  ·  ·  ·  P  P  P  ·  Q  Q  Q
 r3  ·  ·  ·  ·  P  P  ·  ·  Q  ·
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

Il giocatore vuole un **esperimento pulito** sull'interazione P–Q: spende 2 semine per mettere una coppia isolata P–Q in una tasca vuota e luminosa in alto a sinistra (r0, c0–c1), lontano da tutto il resto.

**Era 2 — la prima interazione si rivela.** Al confine principale (col 6–7) e nella coppia isolata, **Q appassisce dove tocca P** (`◆→▲ = −2`). Nella coppia isolata, Q muore (lascia `+`) mentre P resta illeso.

```
 r0  P  +  ·  P  P  P  P  Q  Q  ·     ← coppia isolata: P vivo, Q morto (+)
 r1  ·  ·  P  P  P  P  P  +  Q  Q     ← + a c7: Q morto al contatto col fronte P
 r2  ·  ·  ·  P  P  P  P  +  Q  Q
 r3  ·  ·  ·  ·  P  P  P  ·  Q  Q
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

L'osservazione isolata è **pulita** (nessun confonditore): peso `1.0`. Il taccuino registra due fatti — `◆→▲` negativo, e `▲→◆ = 0` (P è rimasto intatto: Q non danneggia P). Ecco lo stato del taccuino:

| ↓ / → | ◆ | ○ | ▲ | ✦ | ✚ |
|---|---|---|---|---|---|
| ◆ | ? | ? | **−!** | ? | ? |
| ○ | ? | ? | ? | ? | ? |
| ▲ | **0** | ? | ? | ? | ? |
| ✦ | ? | ? | ? | ? | ? |
| ✚ | ? | ? | ? | ? | ? |

*(`−!` = negativo confermato · `0` = nessun effetto confermato · `?` = ignoto)*

**Era 3 — introdurre un predatore.** P sta dilagando. Il giocatore semina **R** (predatore) nel territorio di P. R fa un doppio danno a P: lo **mangia** (metabolismo) e lo **sopprime chimicamente** (`✦→◆ = −2`). R esplode.

```
 r0  P  ·  ·  P  R  R  P  Q  Q  ·
 r1  ·  ·  R  R  R  R  +  +  Q  Q
 r2  ·  ·  ·  R  R  R  +  +  Q  Q
 r3  ·  ·  ·  R  R  P  +  ·  Q  Q
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

Intanto l'angolo caldo di Q resta **intoccato**: `▲→✦ = −2` significa che **Q sopprime R**, quindi il predatore non riesce a invadere la nicchia calda. (Il giocatore lo nota ma non capisce ancora perché — è un indizio.)

**Era 4 — crollo del predatore e boom del decompositore.** R ha divorato quasi tutto P; senza prede **muore di fame** (`gain 0 − upkeep 0.7`) e lascia un campo di residui. Il giocatore semina **D**, che fiorisce sui resti.

```
 r0  P  ·  ·  +  +  +  +  Q  Q  ·
 r1  ·  ·  +  D  D  +  +  ·  Q  Q
 r2  ·  ·  D  D  D  D  +  ·  Q  Q
 r3  ·  ·  ·  D  D  +  ·  ·  Q  Q
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

Ora l'anello si chiude: **D fertilizza P** (`✚→◆ = +2`), e P ricomincia a crescere dalle chiazze di D. Nel taccuino, `✦→◆` è **sospettato** ma *confuso* dalla predazione (R mangiava P *e* lo sopprimeva) → peso basso → resta ipotesi `−?`. `✚→◆` invece si vede pulito nelle chiazze D–P → sta per confermarsi.

| ↓ / → | ◆ | ○ | ▲ | ✦ | ✚ |
|---|---|---|---|---|---|
| ◆ | ? | ? | **−!** | ? | ? |
| ○ | ? | ? | ? | +? | ? |
| ▲ | **0** | ? | ? | **−?** | ? |
| ✦ | **−?** | ? | ? | ? | ? |
| ✚ | **+?** | ? | ? | ? | ? |

**Era 5–6 — equilibrio dinamico → obiettivo.** Il giocatore semina un po' di R per rimettere in moto la predazione e bilanciare. Il ciclo RPS (P⊣Q, Q⊣R, R⊣P) più il mutualismo D–P si assestano in **onde spaziali** che rotolano sulla griglia: nessuna specie domina, quattro coesistono.

```
 r0  P  P  D  R  R  P  Q  Q  Q  ·
 r1  P  D  D  R  P  P  ·  Q  Q  Q
 r2  D  D  P  P  R  R  ·  Q  Q  Q
 r3  ·  P  P  R  R  D  ·  ·  Q  Q
 r4  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
 r5  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

**Obiettivo del mondo: "≥3 specie coesistenti per 50 tick" → soddisfatto** (P, R, D in ciclo + Q nell'angolo). **Vittoria → Mondo 2**, che aggiunge un 6° tag attivo e una matrice più cattiva.

### 16.3 Gli schemi che il giocatore ha decodificato

Il **ciclo RPS** che sostiene la coesistenza:

```
        P (◆○) ──(◆→▲ : −2)──▶ Q (▲)
          ▲                       │
          │                  (▲→✦ : −2)
      (✦→◆ : −2)                  │
          │                       ▼
          └─────────────────── R (✦)

   P sopprime Q · Q sopprime R · R sopprime P → nessuno vince → coesistono.
```

L'**anello del decompositore** che ricicla la morte in crescita:

```
   morte di P/Q/R ──▶ residuo (+) ──▶ D (✚) fiorisce
                                          │
                                     (✚→◆ : +2)
                                          ▼
                              P (◆) rinvigorito ──▶ nuova biomassa ──▶ (nuove morti) ⟳
```

### 16.4 Anatomia di una singola era

```
 ┌─ PIANIFICA (budget: 3 punti) ─────────────────────────────┐
 │  es.  seed R (1)  +  stress: alza tossicità in un'area (1)  │
 │       +  cull P in una zona (1)      → budget esaurito       │
 └─────────────────────────────────────────────────────────────┘
                    │  premi  SPACE
                    ▼
 ┌─ AVANZA ERA (25 tick, animati) ───────────────────────────┐
 │  ogni tick, per ogni organismo:                             │
 │  env_fit → guadagno metabolico → effetto matrice →          │
 │  costi (upkeep + affollamento) → morte / riproduzione       │
 │  (deterministico a parità di seed)                          │
 └─────────────────────────────────────────────────────────────┘
                    │
                    ▼
 ┌─ OSSERVA & REGISTRA ──────────────────────────────────────┐
 │  il taccuino accumula evidenza; le celle si illuminano →    │
 │  formuli la prossima ipotesi e la prossima era              │
 └─────────────────────────────────────────────────────────────┘
```

### 16.5 Cosa dimostra l'esempio

In questa run il giocatore ha confermato solo **3–4 celle** delle ~20 della matrice — ed è bastato per vincere: non serve decodificare *tutto*, solo la parte rilevante per le specie in campo e per l'obiettivo. Ha esercitato ogni pilastro: **nicchie ambientali** (Era 1), **deduzione via esperimento pulito** (Era 2), **preda-predatore** (Era 3–4), **anello del decompositore** e **coesistenza RPS** (Era 5–6). E soprattutto: nel Mondo 2 la matrice è **rimescolata**, quindi i fatti imparati **non si trasferiscono** — si trasferisce solo il *metodo*. È qui che vive la rigiocabilità.

---

*Fine documento — v0.4. Tutte le decisioni di design sono chiuse e corredate da una baseline numerica (§5.9) e da un esempio giocato (§16). Prossimo passo: implementare la Fase 0 seguendo la coda operativa in `tasks/QUEUE.md`, con questo GDD come riferimento di design e `TECH_DESIGN.md` come riferimento di architettura.*
