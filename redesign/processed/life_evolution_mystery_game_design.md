# Specifica di game design: simulazione di vita, evoluzione e mistero biochimico

## Scopo del documento

Questo documento descrive il concept di un gioco di simulazione di vita ed evoluzione ambientato in un mondo procedurale. Il giocatore deve esplorare ecosistemi, studiare specie con biochimiche differenti e ricostruire progressivamente le relazioni tra archetipi biochimici inizialmente nascosti in una matrice.

Il documento è scritto come contesto e specifica iniziale per un'altra AI incaricata di assistere nella progettazione o implementazione del gioco in Rust/Bevy.

Il principio di design principale è:

```text
Il giocatore non sblocca direttamente la verità.
Costruisce una teoria abbastanza buona da rischiare un esperimento.
```

Il gioco deve combinare:

1. esplorazione di un mondo procedurale;
2. osservazione di organismi e ambienti;
3. esperimenti controllati;
4. inferenza delle relazioni biochimiche;
5. gestione di popolazioni ed ecosistemi;
6. mutazione, adattamento e speciazione;
7. eventi emergenti e misteri di scala crescente.

---

## 1. Concetto generale

Il mondo è una mappa ecologica regionale. Ogni cella rappresenta un habitat locale, non necessariamente un singolo metro quadrato:

- tratto di costa;
- valle;
- versante montano;
- zona di foresta;
- bacino idrografico;
- tratto di fiume;
- pianura;
- palude;
- regione desertica.

Le specie possiedono una composizione biochimica parzialmente sconosciuta. Gli archetipi biochimici influenzano:

- produzione e consumo di energia;
- assorbimento dei nutrienti;
- resistenza alle tossine;
- crescita e riproduzione;
- compatibilità con altre specie;
- simbiosi;
- predazione;
- malattie;
- trasmissione di metaboliti;
- adattamento agli ambienti;
- probabilità di mutazione e speciazione.

La matrice biochimica completa è nota al motore ma inizialmente nascosta al giocatore. Il giocatore possiede invece una copia incompleta, costruita tramite osservazioni, ipotesi ed esperimenti.

---

## 2. Scala del mondo

### 2.1 Griglia principale

La mappa giocabile è una griglia di:

```text
128×80 celle = 10.240 celle
```

Questa dimensione è sufficiente per un mondo regionale completo e credibile, ma non per un continente realistico su scala globale con molti grandi biomi indipendenti.

La mappa deve essere trattata come una regione ecologica compatta, con una quantità limitata ma significativa di biomi.

### 2.2 Scala fisica suggerita

La scala della cella può variare in base al gameplay:

| Scala della cella | Estensione approssimativa | Interpretazione |
|---|---:|---|
| 10 m | 1,28×0,8 km | area molto piccola |
| 50 m | 6,4×4 km | valle o isola compatta |
| 100 m | 12,8×8 km | regione esplorabile |
| 500 m | 64×40 km | macro-regione |
| 1 km | 128×80 km | territorio molto vasto |

Per il gameplay consigliato, usare indicativamente una scala tra 50 e 200 metri per cella. La cella è un habitat aggregato e può contenere molte entità non simulate individualmente.

### 2.3 Risoluzione gerarchica

Usare livelli di simulazione distinti:

| Livello | Scala | Responsabilità |
|---|---|---|
| Mondo | 128×80 | esplorazione, biomi, risorse, fiumi |
| Clima | 32×20 o 64×40 | temperatura, pioggia, umidità, macro-regioni |
| Popolazione | una o più popolazioni per habitat | crescita, competizione, mutazione |
| Individuo | campioni o organismi selezionati | comportamento e osservazione dettagliata |
| Decorazione | sottocelle 4×4 o 8×8 opzionali | alberi, rocce, risorse, dettagli visivi |

Il clima deve essere più lento e più spaziale. Il dettaglio locale può essere più ricco senza modificare l'identità climatica della macro-regione.

### 2.4 Quantità ragionevole di biomi

Non forzare ogni partita a contenere tutti i biomi. Una singola generazione dovrebbe avere normalmente:

- 4–8 macro-regioni climatiche;
- 5–10 biomi effettivamente presenti;
- 1–3 fiumi principali;
- 1–5 laghi;
- 8–20 specie iniziali;
- 0–3 feature geologiche importanti.

Nel corso della campagna il giocatore può scoprire più biomi tramite esplorazione, migrazione, cambiamenti climatici o profili di mondo diversi.

### 2.5 Dimensioni minime utili

Questi valori sono linee guida per mantenere la leggibilità:

| Elemento | Dimensione pratica |
|---|---:|
| Bioma dominante | almeno 15–20 celle di larghezza |
| Regione climatica | 20–40 celle |
| Foresta o deserto riconoscibile | circa 15×15 celle o più |
| Palude leggibile | 8–12 celle |
| Catena montuosa | 20–50 celle di lunghezza |
| Montagna isolata | 5–10 celle di diametro |
| Lago importante | 6–15 celle di diametro |
| Cratere | 6–12 celle di diametro |
| Campo di cristalli | 5–15 celle di diametro |
| Fiume principale | 20–40 celle di percorso |

Un biome di 2–4 celle deve essere trattato come transizione o micro-habitat, non come macro-regione.

---

## 3. Distribuzione del mondo

### 3.1 Distribuzione iniziale suggerita

Usare i seguenti valori come punto di partenza, non come vincolo universale:

```text
Acqua                    35–45%
Pianura/prateria          15–25%
Foresta                   10–20%
Collina/alta montagna      10–20%
Deserto/steppa              5–12%
Tundra/neve/ghiaccio        3–8%
Palude                      2–6%
Feature speciali            <5%
```

I valori dipendono dal profilo del mondo. Un mondo vulcanico può avere più roccia e meno foresta. Un mondo temperato può avere poca tundra. Un mondo umido può non avere un deserto significativo.

### 3.2 Macro-regione prima del biome locale

Prima di classificare ogni cella, generare macro-regioni climatiche su una griglia bassa:

```text
climate grid 32×20 o 64×40
    -> 4–8 macro-regioni
    -> biome dominante per regione
    -> correzioni locali su griglia 128×80
```

Ogni macro-regione deve avere un’identità prevalente:

```text
regione umida temperata
    -> Forest dominante
    -> Plain ai margini
    -> Swamp nei bacini
    -> RiverForest lungo i fiumi
```

Non assegnare ogni biome con probabilità indipendente per cella.

### 3.3 Pattern climatici leggibili

La mappa dovrebbe poter produrre pattern come:

```text
oceano -> costa umida -> foresta -> montagne -> steppa -> deserto
```

Questo risultato deve derivare da:

- costa;
- vento prevalente;
- sollevamento orografico;
- rain shadow;
- temperatura;
- elevazione.

I biomi devono avere relazioni spaziali riconoscibili.

---

## 4. Profili di generazione

Non usare la stessa distribuzione per ogni partita. Definire profili:

```rust
#[derive(Clone, Copy, Debug)]
pub enum WorldProfile {
    TemperateIsland,
    VolcanicArchipelago,
    NorthernContinent,
    DryFrontier,
    ToxicWetlands,
}
```

Ogni profilo controlla:

- frazione d'acqua;
- temperatura media;
- intensità del gradiente latitudinale;
- quantità di montagne;
- prevalenza di foreste, deserti o tundra;
- quantità di paludi;
- probabilità di laghi;
- quantità di feature;
- biomi obbligatori e opzionali.

```rust
pub struct WorldGenerationProfile {
    pub required_biomes: Vec<Biome>,
    pub optional_biomes: Vec<Biome>,
    pub target_land_fraction: RangeF32,
    pub target_mountain_fraction: RangeF32,
    pub target_forest_fraction: RangeF32,
    pub target_desert_fraction: RangeF32,
    pub target_swamp_fraction: RangeF32,
    pub max_special_features: usize,
}
```

Esempio:

```text
TemperateIsland:
    richiesti: DeepWater, ShallowWater, Plain, Forest, Mountain
    opzionali: Swamp, Desert, Tundra, Crater

VolcanicArchipelago:
    richiesti: DeepWater, ShallowWater, Mountain, VolcanicVent
    opzionali: BareRock, CrystalField, Lake, Forest

DryFrontier:
    richiesti: Plain, Desert, Mountain
    opzionali: Forest, Tundra, Swamp, Lake
```

Un biome opzionale deve essere aggiunto solo se la geografia lo supporta naturalmente.

---

## 5. Ciclo di gioco principale

Il ciclo centrale deve essere:

```text
osserva
    -> formula un'ipotesi
    -> raccogli un campione
    -> esegui un esperimento
    -> modifica l'ambiente o una popolazione
    -> osserva la conseguenza
    -> aggiorna la matrice
    -> pianifica il prossimo esperimento
```

Il giocatore non deve ricevere soltanto missioni esplicite come:

```text
"Scopri la relazione tra A e B"
```

Deve ricevere problemi osservabili:

```text
"Perché la popolazione blu scompare quando il lago diventa torbido?"
```

oppure:

```text
"Perché questa colonia cresce soltanto vicino alla specie gialla?"
```

Il giocatore deve trovare la risposta tramite osservazioni, campioni e interventi controllati.

---

## 6. Ruolo del giocatore

Il giocatore è un ricercatore-ecologo con capacità limitate di intervento.

### Azioni principali

- esplorare regioni sconosciute;
- osservare organismi e habitat;
- raccogliere campioni;
- isolare archetipi biochimici;
- confrontare specie;
- annotare ipotesi;
- introdurre o rimuovere organismi;
- modificare localmente temperatura, umidità o tossicità;
- creare micro-habitat;
- seguire la crescita delle popolazioni;
- proteggere specie dall’estinzione;
- effettuare esperimenti rischiosi;
- seguire anomalie e reperti;
- costruire un modello predittivo.

Il giocatore non deve controllare direttamente ogni individuo. Deve modificare condizioni e popolazioni, lasciando emergere il comportamento dell’ecosistema.

---

## 7. Matrice biochimica nascosta

### 7.1 Matrice reale

Il motore possiede una matrice completa:

```rust
pub struct BiochemicalMatrix {
    pub interactions:
        HashMap<(ArchetypeId, ArchetypeId), Interaction>,
}
```

```rust
pub struct Interaction {
    pub compatibility: f32,
    pub energy_transfer: f32,
    pub toxicity: f32,
    pub transmission: f32,
    pub context_modifiers: Vec<ContextModifier>,
}
```

Una relazione può essere:

- simbiosi;
- competizione;
- predazione;
- parassitismo;
- neutralità;
- dipendenza metabolica;
- conversione di tossine;
- cooperazione riproduttiva;
- facilitazione ambientale.

### 7.2 Matrice del giocatore

Il giocatore possiede una rappresentazione incompleta:

```rust
pub struct DiscoveredRelation {
    pub left: ArchetypeId,
    pub right: ArchetypeId,
    pub relation_type: RelationType,
    pub confidence: ConfidenceLevel,
    pub context: Option<Biome>,
    pub evidence: Vec<EvidenceId>,
}
```

La matrice dovrebbe distinguere:

1. relazione sconosciuta;
2. esistenza sospettata;
3. correlazione osservata;
4. ipotesi formulata;
5. relazione testata;
6. relazione confermata;
7. ipotesi confutata;
8. relazione confermata solo in un contesto specifico.

### 7.3 Relazioni condizionali

La stessa coppia di archetipi può avere relazioni diverse in ambienti diversi:

```text
A + B in foresta umida -> simbiosi
A + B in deserto caldo -> competizione
A + B in zona tossica -> neutralità
```

Questo impedisce al giocatore di completare una semplice tabella statica e rende importante il contesto ecologico.

---

## 8. Archetipi biochimici

Gli archetipi devono avere effetti funzionali nella simulazione.

```rust
pub struct BiochemicalArchetype {
    pub id: ArchetypeId,
    pub energy_profile: EnergyProfile,
    pub nutrient_profile: NutrientProfile,
    pub toxin_resistance: f32,
    pub temperature_range: RangeInclusive<f32>,
    pub moisture_range: RangeInclusive<f32>,
    pub interaction_tags: Vec<InteractionTag>,
}
```

Possibili proprietà:

- produzione di energia;
- assorbimento di nutrienti;
- decomposizione;
- resistenza alla tossicità;
- produzione di segnali chimici;
- difesa;
- scambio metabolico;
- compatibilità riproduttiva;
- trasmissione;
- tolleranza a temperatura e umidità.

Una specie deve avere una composizione, non soltanto un singolo tipo:

```rust
pub struct SpeciesGenome {
    pub archetypes: Vec<ArchetypeWeight>,
    pub morphology: Morphology,
    pub behavior: BehaviorProfile,
    pub mutation_rate: f32,
}
```

Le mutazioni possono modificare:

- il peso di un archetipo;
- la soglia di tolleranza;
- l'efficienza energetica;
- il comportamento;
- la compatibilità riproduttiva;
- la probabilità di trasmettere un tratto.

---

## 9. Livelli di conoscenza

Ogni relazione deve avere una progressione di conoscenza.

```rust
pub struct KnowledgeEntry {
    pub observed: bool,
    pub hypothesized: bool,
    pub confirmed: bool,
    pub confidence: f32,
    pub evidence: Vec<EvidenceId>,
}
```

### Stato 1: ignoranza

Il giocatore non sa che la relazione esiste.

### Stato 2: indizio

Osserva una correlazione:

```text
"La specie Delta aumenta quando Beta è presente."
```

### Stato 3: ipotesi

Formula una spiegazione:

```text
"Beta nutre Delta."
```

### Stato 4: esperimento

Modifica una variabile e crea un gruppo di controllo.

### Stato 5: conferma o confutazione

Aggiorna la matrice e il livello di confidenza.

### Stato 6: comprensione

Usa la relazione per prevedere un risultato in un habitat nuovo.

Il gioco deve premiare anche le ipotesi confutate: una teoria sbagliata ma informativa è progresso.

---

## 10. Esperimenti controllati

Ogni esperimento dovrebbe cercare di isolare una variabile.

Esempio:

```text
Ipotesi:
La specie Delta dipende dall'archetipo Beta.

Esperimento:
- due habitat identici;
- stessa temperatura;
- stessa umidità;
- stesse risorse;
- Beta presente nel primo;
- Beta assente nel secondo.

Risultato:
Delta cresce solo nel primo habitat.
```

Struttura dati:

```rust
pub struct Experiment {
    pub subject_species: SpeciesId,
    pub variables_changed: Vec<VariableChange>,
    pub control_group: Option<HabitatId>,
    pub observations: Vec<Observation>,
    pub conclusion: ExperimentConclusion,
}
```

Gli esperimenti possono essere:

- osservazionali;
- comparativi;
- controllati;
- di introduzione;
- di rimozione;
- di isolamento;
- di stress ambientale;
- di co-coltura;
- di riproduzione;
- evolutivi.

Non tutti gli esperimenti devono essere perfettamente controllati. Il giocatore deve poter scoprire che una conclusione era errata perché erano cambiate contemporaneamente temperatura, predatori e risorse.

---

## 11. Feedback e informazioni incomplete

Il giocatore deve ricevere dati parziali ma utili:

```text
- "Crescita accelerata in presenza della specie K."
- "Reazione anomala al campione K."
- "Mortalità ridotta del 18%."
- "Effetto osservato solo sopra 0,6 di umidità."
- "La relazione sembra dipendere dall'età."
- "Il campione viene consumato durante l'interazione."
- "Il fenomeno non è stato osservato in ambiente sterile."
```

L’interfaccia può rappresentare la matrice come una rete:

- linea tratteggiata: osservazione;
- linea gialla: ipotesi;
- linea verde: relazione confermata;
- linea rossa: ipotesi confutata;
- linea grigia: relazione inattiva in quel contesto;
- linea animata: relazione attiva in tempo reale.

Il giocatore deve sapere abbastanza per formulare una domanda, ma non abbastanza per ottenere automaticamente la risposta.

---

## 12. Eventi emergenti

La simulazione deve produrre eventi interpretabili:

- fioritura improvvisa;
- estinzione locale;
- comparsa di una mutazione;
- invasione di una specie;
- collasso della rete alimentare;
- contaminazione di un lago;
- cambiamento climatico;
- comparsa di una simbiosi;
- separazione di una popolazione;
- speciazione;
- migrazione;
- epidemia;
- diffusione di un archetipo raro;
- proliferazione di una specie dopo la scomparsa di un concorrente.

Gli eventi devono lasciare tracce:

- residui chimici;
- variazione nei rapporti di popolazione;
- cambiamento della vegetazione;
- nuove strutture organiche;
- comportamento anomalo;
- cambiamento della composizione dell'acqua;
- mutazione osservabile;
- sedimentazione o contaminazione.

Un evento non deve essere solo una notifica. Deve essere un problema da interpretare.

---

## 13. Simulazione delle popolazioni

Non simulare individualmente migliaia di organismi nella prima versione. Usare popolazioni aggregate.

```rust
pub struct Population {
    pub species: SpeciesId,
    pub habitat: HabitatId,
    pub count: f32,
    pub health: f32,
    pub energy: f32,
    pub age_structure: AgeStructure,
    pub genetic_variance: f32,
    pub known_archetype_weights: Vec<ArchetypeWeight>,
}
```

Formula concettuale:

```text
growth =
    base_reproduction
    * habitat_fitness
    * resource_access
    * biochemical_compatibility
    * predation_pressure
    * disease_modifier
    * population_density_modifier
```

Il motore può conoscere la formula completa, ma l'interfaccia deve rivelare solo osservazioni e segnali parziali.

### Tick di simulazione

Separare le frequenze:

```rust
pub struct SimulationClock {
    pub short_tick: u64,
    pub ecological_tick: u64,
    pub evolutionary_tick: u64,
}
```

Esempio:

```text
short tick       -> comportamento e consumo
ecological tick  -> crescita, risorse, migrazione
long tick        -> mutazioni, selezione, speciazione
```

Non simulare ogni processo a ogni frame.

---

## 14. Speciazione e isolamento

La speciazione deve avere cause ricostruibili:

- separazione geografica;
- cambiamento climatico;
- pressione di predazione;
- nuova risorsa;
- relazione biochimica;
- isolamento riproduttivo;
- mutazioni accumulate.

Esempio:

```text
popolazione A
    -> migra in una valle fredda
    -> sviluppa tolleranza al gelo
    -> perde compatibilità riproduttiva con la popolazione originaria
    -> nasce specie A-2
```

Il giocatore deve poter ricostruire la storia tramite:

- albero filogenetico incompleto;
- reperti;
- differenze morfologiche;
- archetipi condivisi;
- tracce ambientali;
- stima della data di divergenza.

Non mostrare subito l’albero completo. La sua ricostruzione deve essere una ricompensa.

---

## 15. Progressione

La progressione deve basarsi sulla capacità di porre domande migliori, non solo sul livello del giocatore.

### Assi di progressione

| Asse | Sbloccabili | Effetto |
|---|---|---|
| Esplorazione | sensori, mappe, spedizioni | trova nuovi habitat |
| Analisi | microscopia, spettrometria, colture | produce dati migliori |
| Intervento | incubatori, isolatori, habitat artificiali | abilita esperimenti controllati |
| Teoria | modelli, simulazioni predittive, archivio | collega relazioni distanti |

Progressione informativa:

```text
osservazione:
"K cresce vicino a J."

analisi:
"K assorbe un metabolita prodotto da J."

modello:
"Se introduco K nel bacino freddo, J dovrebbe stabilizzarsi."
```

Il giocatore diventa più competente senza ottenere automaticamente tutta la verità.

---

## 16. Rischio, costi e irreversibilità

Gli esperimenti devono avere un costo, altrimenti il giocatore testerà tutte le combinazioni senza ragionare.

Possibili costi:

- tempo;
- energia;
- campioni limitati;
- contaminazione;
- perdita di popolazioni;
- alterazione permanente dell'habitat;
- mutazioni impreviste;
- aumento della tossicità;
- estinzione locale;
- impossibilità di ripetere lo stesso esperimento.

Non rendere tutto irreversibile:

```text
esperimenti comuni -> reversibili
esperimenti avanzati -> costosi
esperimenti proibiti -> conseguenze persistenti
```

Usare strumenti di recupero per evitare frustrazione:

- backup genetici;
- crioconservazione;
- popolazioni di riserva;
- habitat isolati;
- possibilità di riavvolgere una simulazione locale;
- salvataggi sperimentali separati.

---

## 17. Mistero a tre scale

### 17.1 Mistero locale

Risolto in pochi minuti:

```text
Perché la colonia cambia colore?
Perché il fungo appare dopo la pioggia?
Perché la specie B evita il lago?
```

### 17.2 Mistero regionale

Richiede più spedizioni e confronti:

```text
Perché tutte le specie della valle condividono un archetipo?
Perché il lato est della catena è sterile?
Perché le popolazioni collassano ogni 12 cicli?
```

### 17.3 Mistero globale

Richiede la ricostruzione della matrice e della storia del mondo:

```text
Chi ha creato la matrice?
La biochimica è evoluta o progettata?
Le specie sono realmente indipendenti?
Perché certi archetipi compaiono in biomi incompatibili?
```

Le risposte globali devono emergere da dati ed esperimenti, non soltanto da testi o filmati.

---

## 18. Struttura di campagna

### Fase 1: osservazione

Strumenti poveri:

- mappa incompleta;
- osservazione visiva;
- campioni grezzi;
- matrice quasi vuota;
- poche specie note.

Obiettivo: trovare correlazioni.

### Fase 2: laboratorio

Il giocatore può:

- isolare archetipi;
- coltivare specie;
- confrontare campioni;
- creare habitat controllati;
- testare compatibilità.

Obiettivo: distinguere correlazione e causalità.

### Fase 3: intervento

Il giocatore può:

- trasferire specie;
- modificare habitat;
- salvare popolazioni;
- provocare esperimenti evolutivi;
- gestire contaminazioni.

Obiettivo: usare le relazioni senza destabilizzare l'ecosistema.

### Fase 4: sintesi

Il giocatore costruisce un modello predittivo:

```text
Se habitat = umido e freddo
  e archetipo A > 0,4
  e B è presente
allora C dovrebbe evolvere verso C-2.
```

Obiettivo: prevedere eventi ancora non osservati e testare una teoria generale.

---

## 19. Meccaniche da evitare

Evitare:

- sblocco diretto delle relazioni tramite esperienza;
- scanner che rivela automaticamente tutta la biochimica;
- combinazioni arbitrarie senza feedback;
- morte permanente frequente delle specie iniziali;
- raccolta di risorse senza valore investigativo;
- simulazione individuale di migliaia di organismi fin dall'inizio;
- biomi puramente decorativi;
- missioni che rivelano già la risposta;
- matrice con una sola interpretazione possibile;
- progressione basata soltanto su nuovi strumenti;
- eventi casuali non interpretabili;
- punizioni che cancellano ore di scoperta senza possibilità di recupero.

La relazione deve essere scoperta perché il giocatore ha capito qualcosa, non perché ha raggiunto un livello.

---

## 20. Architettura Rust/Bevy

Separare il modello simulativo dal rendering e dall'interfaccia.

```rust
pub struct WorldSimulation {
    pub habitats: Vec<Habitat>,
    pub populations: Vec<Population>,
    pub species: Vec<Species>,
    pub matrix: BiochemicalMatrix,
    pub environment: EnvironmentFields,
    pub knowledge: PlayerKnowledge,
}
```

Sistemi ECS possibili:

```text
EnvironmentUpdateSystem
ClimateSystem
ResourceRegenerationSystem
PopulationGrowthSystem
InteractionResolutionSystem
MutationSystem
MigrationSystem
SpeciationSystem
ObservationSystem
ExperimentSystem
KnowledgeUpdateSystem
EventLogSystem
```

Possibili risorse Bevy:

```rust
#[derive(Resource)]
pub struct SimulationClock { ... }

#[derive(Resource)]
pub struct BiochemicalMatrix { ... }

#[derive(Resource)]
pub struct PlayerKnowledge { ... }

#[derive(Resource)]
pub struct ActiveExperiment { ... }
```

Gli eventi ECS possono rappresentare:

```rust
pub struct PopulationCollapseEvent;
pub struct MutationObservedEvent;
pub struct NewInteractionEvidenceEvent;
pub struct SpeciationEvent;
pub struct HabitatContaminatedEvent;
```

Il modello dovrebbe essere testabile senza avviare il rendering Bevy.

---

## 21. Prototipo minimo raccomandato

Per la prima verticale implementare:

```text
Mappa:
- 128×80 celle.
- 32×20 climate grid.
- 4–6 macro-regioni.
- 5–7 biomi dominanti.
- 1–3 fiumi principali.
- 1–4 laghi.

Specie:
- 8–12 specie iniziali.
- 6–10 archetipi biochimici.
- 20–40 relazioni reali.
- relazioni dipendenti da bioma e temperatura.
- matrice completa nascosta.
- matrice del giocatore incompleta.

Simulazione:
- popolazioni aggregate.
- migrazione tra celle adiacenti.
- crescita, competizione e simbiosi.
- mutazioni rare.
- speciazione dopo isolamento o pressione significativa.

Scoperta:
- osservazione.
- raccolta campioni.
- esperimenti con controllo.
- ipotesi annotabili.
- evidenze con livelli di confidenza.
- eventi e anomalie.
```

La prima campagna dovrebbe completare un ciclo:

```text
scoperta di una specie
    -> osservazione di una correlazione
    -> esperimento
    -> conferma di una simbiosi
    -> applicazione nell'habitat
    -> mutazione inattesa
    -> nuova domanda
```

Ogni risposta deve risolvere una domanda e aprirne una più profonda.

---

## 22. Criteri di successo del design

Il gioco funziona se il giocatore:

- osserva un fenomeno e formula spontaneamente un'ipotesi;
- comprende la differenza tra correlazione e causalità;
- vuole ripetere o migliorare un esperimento;
- ricorda una relazione perché l'ha scoperta, non perché l'ha letta;
- riconosce che una relazione dipende dal contesto;
- teme ma desidera gli esperimenti rischiosi;
- vede le proprie decisioni cambiare l'ecosistema;
- riconosce le conseguenze di una specie introdotta molte fasi dopo;
- completa la matrice come strumento di comprensione, non come checklist;
- continua a giocare perché una risposta ha creato una domanda più interessante.

Il gioco non deve misurare soltanto quante specie sono state scoperte. Deve misurare anche quanto bene il giocatore sa prevedere l'ecosistema.

---

## Principio conclusivo

La mappa 128×80 deve essere un laboratorio ecologico vivo, non una simulazione planetaria. I biomi forniscono il contesto degli esperimenti; le specie rendono visibili le conseguenze; la matrice trasforma osservazioni in ipotesi; gli esperimenti trasformano ipotesi in conoscenza; l'evoluzione genera nuove anomalie.

Il ciclo desiderato è:

```text
osservazione
    -> ipotesi
    -> esperimento
    -> conseguenza
    -> nuova conoscenza
    -> nuova anomalia
    -> nuova ipotesi
```

La ricompensa principale non è ottenere una risposta dal gioco, ma arrivare a formulare una previsione corretta su qualcosa che il giocatore non ha ancora visto.

---

## Riferimenti concettuali

- [Red Blob Games — Procedural map generation on a sphere](https://www.redblobgames.com/x/1843-planet-generation/)
- [Red Blob Games — Mapgen4 procedural wilderness map generator](https://www.redblobgames.com/maps/mapgen4/)
- [AutoBiomes: procedural generation of multi-biome landscapes](https://cgvr.cs.uni-bremen.de/papers/cgi20/AutoBiomes.pdf)
- [The Design and Implementation of Biological Evolution as a Video Game Mechanic](https://dl.acm.org/doi/10.1007/978-3-031-49065-1_7)
- [Thrive — What is Thrive](https://wiki.revolutionarygamesstudio.com/wiki/What_is_Thrive)
- [TUNIC — mystery and meaningful secrets](https://www.gamedeveloper.com/design/video-how-tunic-was-built-on-mystery)
