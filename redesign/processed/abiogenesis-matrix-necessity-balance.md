# Abiogenesis — rendere la matrice necessaria, non opzionale

Documento autonomo per un task di bilanciamento/integrazione. Descrive un problema strutturale nel loop centrale individuato ragionando da giocatore, la correzione di principio proposta, e una prima ipotesi numerica — esplicitamente da validare contro le formule reali, non un valore definitivo.

## Il problema

Con i valori baseline noti (§5.9 GDD: `energy_start = 5.0`, `repro_threshold = 10.0`, guadagno fotolitico isolato ≈ +0.9/tick in condizioni di temperatura ottimale), un organismo piazzato leggendo solo i dati **visibili** (temperatura, luce, tossicità) raggiunge la soglia di riproduzione in circa 6 tick, ben dentro una singola era (25 tick baseline) — **senza che nessuna interazione della matrice nascosta debba mai entrare in gioco**.

Questo significa che il layer visibile (ambiente) è oggi **sufficiente** per il successo meccanico di base (un organismo che sopravvive e si riproduce), mentre il layer nascosto (i tratti, il vero mistero del gioco) resta **opzionale**. Da giocatore, il sintomo è diretto: si piazzano un paio di specie leggendo solo calore/luce/tossicità, e in pochissimo tempo o muoiono per un disallineamento ambientale evidente, o si espandono rapidamente — in nessuno dei due casi la matrice ha avuto un ruolo necessario nella decisione. Non c'è una ragione strutturale per fare esperimenti: il gioco chiede di decifrare qualcosa che, per la sopravvivenza di base, non serve decifrare affatto.

## Il principio della correzione

**L'adattamento ambientale da solo deve portare un organismo a malapena in pareggio energetico — sopravvive, non si riproduce.** Solo un'interazione **positiva** della matrice (un tratto vicino che dà un bonus) deve fornire il margine sufficiente per superare la soglia di riproduzione. Un'interazione **negativa**, viceversa, deve essere ciò che spinge dal pareggio verso il declino vero — oggi quel ruolo è già in parte giocato dall'ambiente sbagliato, ma dovrebbe essere primariamente la matrice a deciderlo.

**Perché questa correzione, e non altre:**

- **Rende la matrice obbligatoria per il successo, non solo per la comprensione teorica.** Con questa correzione, crescere una popolazione *è* prova che il giocatore ha capito qualcosa di reale sul mondo — il successo meccanico e la scoperta diventano la stessa cosa, invece di due percorsi paralleli dove il primo rende il secondo facoltativo.
- **Genera trial-and-error per costruzione, non per regola imposta.** Se piazzare bene per temperatura dà solo stabilità e non crescita, l'unico modo per scoprire cosa fa prosperare una specie diventa provare a metterla vicino a tratti diversi e osservare — il comportamento desiderato emerge dal numero, non da una regola esplicita che impone di sperimentare.
- **Risponde al problema "troppo facile" senza rendere il gioco più punitivo in astratto.** La cura non è rallentare la riproduzione o renderla più costosa in sé (leve già proposte altrove per motivi di ritmo, non di necessità) — è condizionarla a un'informazione che il giocatore deve ancora scoprire. La difficoltà percepita cresce, ma nasce da comprensione mancante, non da un ostacolo arbitrario.

## Ipotesi numerica, ricalcolata sulla formula reale

**Correzione rispetto a una versione precedente di questo documento:** i numeri erano stati proposti trattando `metabolism_gain` come se fosse il guadagno finale. Nella formula reale (§5.6 GDD) è un **moltiplicatore**: `gain = light × metabolism_gain × env_fit`. I valori sotto sono ricalcolati sulla formula corretta.

**Avvertenza sulla provvisorietà:** questi valori sono tarati assumendo l'attuale `ERA_TICKS = 25`. Con la revisione della scala temporale (era molto più lunga, stagione come unità di decisione — cfr. documento dedicato) **vanno ritarati insieme a tutto il resto**: sono la forma della correzione e l'ordine di grandezza del rapporto, non valori finali.

### Situazione attuale (GDD §5.9)

Fotolitico isolato, `light 0.7`, `env_fit ≈ 1`, nessun vicino:
- `gain = 0.7 × 2.0 × 1 = 1.4`
- `upkeep = 0.5`, crowding `0`
- **netto = +0.9/tick** → soglia di riproduzione (da 5.0 a 10.0) in ~6 tick, meno di un quarto d'era. È il problema.

### Correzione proposta

| Costante | Attuale | Proposto | Nota |
|---|---|---|---|
| Photolithic `metabolism_gain` | 2.0 | **0.8** | unica modifica strutturale sul fotolitico |
| `upkeep` base | 0.5 | **0.5** | invariato — basta il moltiplicatore |
| Chemolithotroph `metabolism_gain` | 2.0 | **0.8** | stessa formula del fotolitico |
| Predator `drain_cap` | 2.0 | **0.8** | scalato con lo stesso rapporto |
| Decomposer `extract_rate` | 1.5 | **0.6** | scalato con lo stesso rapporto |
| Coefficiente di scala `interaction_delta` | *(da verificare in build)* | **0.15** per unità di intensità | vedi nota sotto |
| `crowd_factor` | 0.15 | **0.15** | invariato |
| `energy_start`, `repro_threshold`, `repro_cost` | 5.0 / 10.0 / 5.0 | invariati | |

**Gli altri metabolismi vanno scalati con lo stesso rapporto (÷2.5)**, non lasciati invariati — altrimenti diventerebbero sproporzionatamente forti rispetto al fotolitico. Gli `upkeep` differenziati per metabolismo (Predator 0.7, altri 0.5) restano come sono.

### Risultati attesi

- **Isolato, condizioni ottimali, nessuna interazione:** `gain = 0.7 × 0.8 × 1 = 0.56`, netto **≈ +0.05/tick**. Servirebbero ~100 tick per riprodursi — di fatto, senza interazione la riproduzione non avviene in tempi utili.
- **Un vicino con interazione `+2`:** `0.05 + (2 × 0.15) − 0.15 (crowding) = **+0.20/tick**` → soglia in ~25 tick, esattamente un'era ai valori attuali.
- **Un vicino con interazione `−2`:** `0.05 − 0.30 − 0.15 = **−0.40/tick**` → morte in ~12 tick.

### Una proprietà emergente da preservare

Con `crowd_factor = 0.15` per vicino e un coefficiente di interazione di 0.15 per unità, **il costo di avere un vicino è dello stesso ordine di grandezza dell'effetto di un `+1`**. Significa che la vicinanza non è mai gratis: solo un'interazione **positiva forte** (`+2`) ripaga davvero la presenza di un vicino. Non è pianificato a parte — emerge dai numeri — e rinforza esattamente l'obiettivo della correzione: non basta stare vicino a qualcuno, deve essere *il vicino giusto*.

### Verifica di sanità: l'anti-degenerazione regge

Con `metabolism_gain 0.8`, un fotolitico in zona a luce bassa (0.2) ha `gain = 0.16` contro `upkeep 0.5` → muore comunque. La nicchia di luce (§5.9) continua a funzionare, la correzione non rompe le difese di §5.8.

### Da verificare in build prima di applicare

**Se esista già un coefficiente di scala sull'`interaction_delta`** o se le intensità `{−2..+2}` entrino grezze nella somma di energia. Se entrassero grezze, un `+2` varrebbe oggi +2.0/tick — enormemente più della correzione proposta, il che cambierebbe la diagnosi: la matrice sarebbe già dominante, ma solo *dopo* che la riproduzione facile ha già fatto il suo danno. È la prima cosa da guardare nel codice.

## Collegamenti con altre proposte già scritte

- **`crowd_factor` va verificato indipendentemente.** Se una specie sola, ben piazzata, può espandersi riempiendo la griglia senza mai incontrare un tratto diverso dal proprio, il problema si ripresenta in un'altra forma. L'affollamento dovrebbe rendere "una specie sola che si espande" un vicolo cieco energetico — non solo esteticamente ripetitivo — spingendo il giocatore a introdurre varietà per necessità.
- **Il peso dell'osservazione isolata (già nel notebook, §7 GDD) si rinforza con questa correzione invece di restare scollegato.** Piazzare con cura, uno alla volta, isolato, dà sia il miglior segnale scientifico (osservazione pulita, peso vicino a 1.0) sia, con questa correzione, il momento in cui l'effetto di un'interazione deliberata diventa visibile per la prima volta — i due sistemi si allineano invece di essere paralleli.
- **L'incubazione dei neonati** (proposta nel documento sull'onboarding) resta utile per il *ritmo*, ma cambia ruolo: prima serviva a rallentare uno sciame che cresceva "gratis" da solo, ora diventa un rinforzo secondario — lo sciame semplicemente non cresce senza una ragione da scoprire, l'incubazione modula il ritmo di chi quella ragione l'ha già trovata.
- **L'obiettivo Omeostasi** (proposto nel documento sugli obiettivi) guadagna significato reale con questa correzione: "mantenere l'energia stabile" smette di essere banale — un organismo isolato *tende naturalmente* a stabilità con questi numeri, quindi l'obiettivo diventa un vero esercizio di lettura della matrice applicata a un bersaglio preciso, non un traguardo quasi automatico.

## Cosa serve per l'integrazione

- **Verificare il coefficiente di scala sull'`interaction_delta`** (vedi sopra) — è il dato che determina se la correzione proposta è sufficiente o se serve un intervento più ampio.
- **Ritarare insieme alla nuova scala temporale:** questi numeri assumono `ERA_TICKS = 25`. Con l'era molto più lunga e la stagione come unità di decisione (cfr. documento sulla scala temporale), l'intero set va rivalutato — in particolare la `selection_pressure_threshold` (20.0, §5.9), tarata sull'era corta: se l'era si allunga senza ritararla, le speciazioni scatterebbero molte volte per era, banalizzando sia l'obiettivo Speciazione sia l'Emersione.
- **Verificare `crowd_factor` per il caso "specie singola che si espande"**: se una specie ben piazzata può riempire la griglia senza mai incontrare un tratto diverso, il problema si ripresenta in altra forma. Non risolto da questa correzione da sola.
- **Testare il caso limite delle prime ere** (world 0, giocatore senza informazioni sulla matrice): con questi numeri osserverà stabilità diffusa e nessuna crescita esplosiva nei primi minuti — verificare che si legga come "segnale onesto da interpretare" e non come "il gioco non sta facendo nulla", eventualmente in combinazione con le proposte di onboarding già scritte (spark visivo sull'interazione, prima luce garantita in world 0).
- **Nota su una revisione più ampia della formula:** con tutto ciò che è stato introdotto dopo la stesura di §5.6 (biomi, tratti condizionati dal terreno, evoluzione per speciazione, biomi dinamici), è aperta la questione se la formula del tick vada estesa o ripensata, non solo ritarata. Discussione separata, non affrontata in questo documento.

## Fuori scope

- I valori numerici esatti definitivi — questo documento propone una forma e un ordine di grandezza, non un bilanciamento finale.
- La revisione di `crowd_factor` in sé — solo segnalata come verifica necessaria collegata, non specificata qui.
- Bilanciamento dei nuovi tipi di obiettivo che si agganciano a questa correzione (Omeostasi) — rimane nel documento dedicato agli obiettivi.
