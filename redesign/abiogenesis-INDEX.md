# Culture Shock — indice dei documenti e ordine di lavorazione

*(Titolo deciso: **Culture Shock**. "Abiogenesis" resta solo nei nomi dei file, non è urgente rinominarli.)*

**Da leggere per primo.** Questo documento non contiene decisioni di design: dice **quale documento consultare per cosa** e **in che ordine affrontare il lavoro**. Ogni documento di dettaglio è autosufficiente; questo li mette in sequenza.

## Regole generali

1. **In caso di conflitto tra un documento e il codice, vince il codice.** I documenti sono proposte di design; l'implementazione è lo stato reale. Dove è stata rilevata una divergenza, è annotata nel documento stesso.
2. **`[DECIDED]` vs `[PROPOSED]`.** Il GDD marca esplicitamente cosa è deciso e cosa è proposto. I documenti di dettaglio sono, salvo dove indicato, proposte da validare — non specifiche approvate.
3. **I numeri sono punti di partenza, non valori finali.** Ogni coefficiente in questi documenti va validato in playtest.
4. **Quando un documento dice "verificare in build", verificalo prima di implementare** — sono i punti in cui il design è stato costruito su una deduzione, non su una lettura del codice.

---

## Documenti già analizzati e implementati

Non fanno parte del piano di lavorazione, ma restano validi come riferimento sulle scelte già prese: `abiogenesis-biomes.md` (roster biomi e valori ambientali), `abiogenesis-terrain-map.md` (rendering del terreno a fasce di elevazione), `abiogenesis-engagement-design.md` (proposte per i primi minuti di gioco). Diversi punti negli altri documenti vi rimandano — in particolare l'onboarding di world 0, che `menu-onboarding` riepiloga come sistema già progettato altrove.

## Mappa dei documenti

### Fonte principale
| Documento | Cosa contiene |
|---|---|
| `abiogenesis-gdd-v0.7.md` | Il GDD. Decisioni, struttura, pilastri. Sintetizza tutto il resto e rimanda ai documenti di dettaglio |
| `abiogenesis-system-hierarchy.md` | Classificazione di ogni sistema in tre livelli (core / variazione strutturale / payoff raro), i due principi trasversali, registro delle discrepanze risolte |
| `abiogenesis-open-points.md` | Cosa resta non deciso |

### Loop e simulazione
| Documento | Cosa contiene |
|---|---|
| `abiogenesis-tick-pipeline.md` | Il tick come pipeline a fasi con tre output; dove entra il bioma; punti di aggancio |
| `abiogenesis-matrix-necessity-balance.md` | Perché la matrice oggi è ignorabile e come correggerlo; coefficienti ricalcolati |
| `abiogenesis-time-scale-reveal.md` | Pulse / stagione / era; la stagione come unità di decisione; il reveal di fine era |
| `abiogenesis-actions.md` | Splice, Cull, Stress rivisti; azioni proposte e scartate |

### Interfaccia
| Documento | Cosa contiene |
|---|---|
| `abiogenesis-hud-notebook.md` | HUD e notebook: struttura, sezioni, apertura |
| `abiogenesis-notebook-cronaca.md` | Quarta sezione (Cronaca) e arricchimenti alle altre |
| `abiogenesis-menu-onboarding.md` | Menu, setup nuova run, e perché non c'è un tutorial guidato |
| `abiogenesis-sidebar-redesign.md`, `abiogenesis-ui-redesign.md` | Redesign della sidebar e principi visivi generali |

### Contenuto e varietà
| Documento | Cosa contiene |
|---|---|
| `abiogenesis-tag-archetypes.md` | Roster dei tratti, famiglie, codici, bias di famiglia, xenotratti |
| `abiogenesis-objectives.md` | Generazione procedurale degli obiettivi, nuovi tipi, vittoria come flag |

### Narrazione ed eventi
| Documento | Cosa contiene |
|---|---|
| `abiogenesis-narrative-generation.md` | Come si genera il testo di reveal e Cronaca |
| `abiogenesis-world-events.md` | Catalogo eventi concreti e ambiziosi |
| `abiogenesis-world-events-catastrophes.md` | Catastrofi, eventi neutri, meccanismo dei biomi dinamici |
| `abiogenesis-emersione.md` | Emersione: trigger, eredità, integrazione |

### Struttura della sessione e trasversali
| Documento | Cosa contiene |
|---|---|
| `abiogenesis-transitions-metaprogression.md` | Lasciare/entrare in un mondo, fine run (MVP), meta-progressione (post-MVP) |
| `abiogenesis-cross-cutting.md` | Salvataggio, audio, accessibilità, performance, lingua, impostazioni |
| `abiogenesis-audio.md` | Design audio in dettaglio + prototipi in `audio-prototipi/` |
| `culture-shock-identity.md` | Titolo, identità del gioco, meccanismo ipotesi/smentita (post-MVP) |
| `culture-shock-population-model-aesthetic.md` | Modello di popolazione per cella (capacità + sfondamento), rendering conseguente, estetica generale — inclusa la decisione sul rendering a grana pixel |
| `culture-shock-inspect-tool.md` | Strumento di ispezione gratuito: hover + card con bilancio energetico scomposto |
| `culture-shock-worked-example.md` | Specchio italiano dell'esempio ora nel GDD §16 (canonico in inglese) |
| `culture-shock-naive-player-example.md` | Stesso inizio partita rigiocato da un giocatore ignaro — stress-test dell'onboarding, 5 punti di attrito, 4 correzioni minime |
| `culture-shock-friction-fixes.md` | Specifica implementabile dei 4 interventi anti-attrito, con priorità e collocazione nel piano (Fase 1b) |
| `culture-shock-controls.md` | Schema controlli completo, risolve i conflitti tra azioni/ispezione/notebook, introduce il menu di pausa |
| `culture-shock-wonder.md` | Pilastro 5 (meraviglia/scoperta): proposte piccole/medie/grandi/stranissime, con priorità |
| `culture-shock-biome-cosmic-events.md` | Firme di bioma ed eventi di origine cosmica — terzo capitolo sugli eventi di mondo, specifiche implementabili |

---

## Ordine di lavorazione

Il criterio non è solo tecnico: **prima si rende il loop necessario e leggibile, poi si aggiunge varietà, poi si struttura la sessione, poi si rifinisce.** Il rischio maggiore del progetto non è tecnico — è che il gioco non sia divertente — quindi l'ordine massimizza quanto presto si può scoprirlo.

### Fase 0 — Due verifiche, non implementazione

Sono letture del codice, non sviluppo, ma cambiano ciò che va fatto dopo:

- **Esiste già un coefficiente di scala sull'`interaction_delta`, o le intensità `{−2..+2}` entrano grezze nella somma di energia?** → `abiogenesis-matrix-necessity-balance.md`. Se entrano grezze, i coefficienti proposti vanno ricalcolati **prima** di applicarli.
- **Verificare l'ambito reale delle azioni** (per-cella, come da §11 e build) → `abiogenesis-actions.md`.

### Fase 1 — Il loop centrale

> Questi tre sono un blocco unico: i numeri del punto 2 non sono finali finché la durata dell'era (punto 3) non è fissata.

| # | Cosa | Documento | Perché ora |
|---|---|---|---|
| 1 | Pipeline del tick + modello di popolazione | `tick-pipeline`, `population-model-aesthetic` | Fondamento tecnico di quasi tutto: emette le osservazioni e gli eventi che serviranno a notebook, reveal, audio, Cronaca. **Il modello di popolazione per cella va deciso qui**, prima di scrivere le fasi 2-5 della pipeline — cambiarlo dopo significa riscriverle |
| 2 | Bilanciamento matrice-necessaria | `matrix-necessity-balance` | Finché si può vincere ignorando la matrice, ogni contenuto aggiunto decora un loop che non funziona |
| 3 | Scala temporale | `time-scale-reveal` | Fissa la durata dell'era, da cui dipendono i numeri del punto 2 e la `selection_pressure_threshold` |

**→ CHECKPOINT: giocare.** Con questi tre il loop è completo e onesto. È il momento di massimo valore informativo per il minimo lavoro speso: si scopre se il gioco è divertente **prima** di investire su tutto il resto. Non saltare questo passaggio.

### Fase 1b — Interventi anti-attrito (nuova)

Emersa rigiocando la prima sessione come giocatore ignaro (`culture-shock-naive-player-example.md`) subito dopo il checkpoint di playtest sopra. Nessuno dei quattro interventi è core, ma dipendono solo da sistemi già presenti a fine Fase 1 e si agganciano a Fase 2-3 — vanno affrontati qui, prima di costruire contenuto e struttura sopra un imbuto dei primi 20 minuti ancora dispersivo. Specifica completa in `culture-shock-friction-fixes.md`.

| Priorità | Intervento | Perché |
|---|---|---|
| **Alta** | Indicatore di saturazione visibile sulla mappa | Unico dei quattro invisibile per progettazione attuale, non solo poco chiaro |
| Media | Secondo hint contestuale sullo stallo apparente | Rischio reale, mitigabile in parte dal riepilogo "come si gioca" |
| Media | Causa nominata nel reveal di speciazione | Costo quasi nullo (dato già esistente), alto impatto sul momento più memorabile |
| Bassa-media | Traduzione temporanea dei codici tratto nel log | Attrito reale ma più facilmente superato con l'esperienza |

**→ Verifica raccomandata prima di proseguire:** rigiocare (idealmente con un vero playtester) lo stesso scenario del giocatore ignaro con questi quattro interventi applicati.

### Fase 2 — Rendere leggibile ciò che il loop produce

| # | Cosa | Documento | Dipende da |
|---|---|---|---|
| 4 | Azioni | `actions` | Fase 1 punto 1 (l'osservazione da Cull nasce nella fase 7 della pipeline) |
| 4b | Strumento di ispezione | `inspect-tool` | Stessa fase, legge gli stessi valori intermedi — conviene farlo insieme alle azioni |
| 4c | Controlli — schema completo | `controls` | Risolve conflitti tra azioni, ispezione e notebook (già in Fase 2) — va chiuso qui, non rimandato: include lo zoom (dipende dal modello di popolazione, Fase 1) e il menu di pausa (contenuto dipende da salvataggio/impostazioni, Fase 4/5 — ma la sua *esistenza* come schermata va costruita qui, perché `Esc` la richiede subito) |
| 5 | HUD e notebook | `hud-notebook`, `notebook-cronaca` | Fase 1 |
| 6 | Obiettivi | `objectives` | Fase 1 punto 3 (durate in stagioni) |

⚠️ **Applicare qui la regola di accessibilità cromatica** ("il colore non è mai l'unico canale", → `cross-cutting`), mentre si costruisce l'interfaccia. Retrofittarla dopo costa molto di più.

### Fase 3 — Contenuto e varietà

| # | Cosa | Documento | Note |
|---|---|---|---|
| 7 | Archetipi dei tratti | `tag-archetypes` | Puramente additivo, nessuna dipendenza forte |
| 8 | Generazione narrativa | `narrative-generation` | Richiede che gli eventi della fase 1 esistano |

⚠️ **Prima del punto 8**, decidere se i frammenti testuali vanno tenuti come **dati strutturati** anziché stringhe concatenate (→ `cross-cutting`, voce lingua). Deciderlo dopo significa riscrivere il pool.

### Fase 4 — Struttura della sessione

| # | Cosa | Documento | Note |
|---|---|---|---|
| 9 | Transizioni e fine run | `transitions-metaprogression` (Parte 1) | Progettata per funzionare **senza** meta-progressione |
| 10 | Menu e onboarding | `menu-onboarding` | Richiede che il salvataggio esista, per la voce "Riprendi" |
| 11 | Salvataggio | `cross-cutting` (§1) | **Volutamente non prima**: fare snapshot di uno stato che cambia ogni settimana è lavoro rifatto di continuo |

### Fase 5 — Rifinitura e sistemi rari

| # | Cosa | Documento |
|---|---|---|
| 12 | Audio | `audio` + prototipi |
| 13 | Accessibilità cromatica (palette alternativa) | `cross-cutting` |
| 14 | Eventi di mondo e biomi dinamici | `world-events`, `world-events-catastrophes` |
| 15 | Emersione e xenotratti | `emersione`, `tag-archetypes` |

L'Emersione è ultima non perché conti poco, ma perché dipende da quasi tutto il resto: speciazione, famiglie di tratti, vittoria come flag, tier dei reveal.

---

## Pilastro 5 — contenuti "meraviglia e scoperta"

`culture-shock-wonder.md` non ha una singola collocazione di fase — è un filtro trasversale, non un blocco di lavoro. Le tre voci prioritarie che propone (tasche di anomalia sparse, tracce fossili, estremofilo concretizzato) sono comunque classificabili nel piano: tutte e tre appartengono al **Livello 2 (variazione strutturale)** della gerarchia di `abiogenesis-system-hierarchy.md`, quindi si collocano naturalmente **dopo la Fase 3** (contenuto e varietà) — riusano meccanismi già esistenti a quel punto (Precursore, ispezione, biomi) invece di introdurne di nuovi. Le voci "grandi" e "stranissime" restano payoff raro, Fase 5 o oltre.

`culture-shock-biome-cosmic-events.md` segue la stessa logica, stessa collocazione (dopo Fase 3): priorità alta per la pioggia di micrometeoriti (stesso sistema delle tasche di anomalia, nessun lavoro aggiuntivo) e le firme di Palude/Vetta; priorità media per il resto delle firme di bioma e per bloom sincronizzato/silenzio anomalo; bassa-media o payoff raro per il resto.

## Distribuzione — non fa parte del piano tecnico

`culture-shock-distribution.md` copre piattaforma, canali (itch.io poi Steam Early Access), marketing (condivisione seed/riassunto di mondo) e prezzo. Non richiede una posizione nelle fasi sopra — è un binario parallelo, non un prerequisito per nessuna di esse — ma un'azione ne dipende: la **funzione di esportazione del riassunto di mondo** va progettata insieme alla Fase 4 (transizioni e fine run), da cui dipende, non rimandata a fine sviluppo.

## Post-MVP, esplicitamente rimandato

- **Ipotesi dichiarate e smentite** → `culture-shock-identity.md`. Non necessario perché il gioco funzioni, ma probabilmente la singola aggiunta con più impatto sull'identità percepita tra tutte quelle rimandate
- Meta-progressione concreta e Codex → `transitions-metaprogression` (Parte 2)
- Registro stratigrafico, testimonianze da verificare, grammatica nascosta tra i mondi → `system-hierarchy` (livello 3)
- Localizzazione → `cross-cutting`
