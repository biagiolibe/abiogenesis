# Specifica tecnica v2: generazione procedurale di mappe e biomi in Rust

## Obiettivo

Implementare un generatore deterministico di mappe per una griglia giocabile di **128×80 celle**, capace di produrre un mondo regionale credibile con una quantità ragionevole di biomi, fiumi, laghi e feature geologiche.

La griglia contiene 10.240 celle. È sufficiente per un mondo completo di gioco se viene trattata come una regione compatta, non come un intero continente realistico su scala globale. Il generatore deve quindi privilegiare:

- poche macro-regioni leggibili;
- biomi abbastanza grandi da essere riconoscibili;
- transizioni coerenti;
- fiumi e montagne con continuità spaziale;
- feature rare ma significative;
- dettaglio locale separato dal clima macro.

La pipeline principale deve essere:

```text
macro-geografia
    -> heightfield
    -> geomorfologia
    -> clima
    -> idrologia
    -> quantità/posizione delle macro-regioni
    -> biomi
    -> feature geologiche
    -> modificatori di gameplay
    -> decorazione
```

Il principio da evitare è:

```text
noise indipendente + soglie locali + override
    -> biome casuali
```

Il principio da adottare è:

```text
geografia -> idrologia -> clima -> biomi -> feature
```

---

## 1. Diagnosi del sistema attuale

Il generatore attuale usa tre stadi:

1. `generate_terrain`: elevazione e `TerrainKind`.
2. `classify_biomes`: biomi da elevazione, luce, temperatura, tossicità e patch noise.
3. `place_feature_biomes`: feature piazzate che sovrascrivono il risultato precedente.

Gli stream RNG separati e il retry dell'intero heightfield sono buone decisioni da mantenere. I problemi principali sono i seguenti.

### 1.1 Classificazione troppo precoce

Convertire l'elevazione continua in:

```text
Sea / Plain / Hill / Mountain
```

fa perdere:

- quota precisa;
- pendenza;
- curvatura;
- distanza dalla costa;
- distanza dall'acqua;
- bacini e depressioni;
- esposizione alle montagne;
- direzione del drenaggio.

`TerrainKind` può restare utile per gameplay e rendering, ma non deve essere l'input principale dei biomi.

### 1.2 Luce e temperatura non sono la stessa cosa

Separare:

- `light`: illuminazione, crescita, rendering o gameplay;
- `temperature`: clima;
- `latitude`: gradiente globale o regionale;
- `altitude`: raffreddamento con la quota;
- `distance_to_coast`: moderazione termica.

### 1.3 Tossicità e paludi sono concetti diversi

Una palude deve dipendere soprattutto da:

- bassa pendenza;
- quota bassa o depressione;
- alta precipitazione;
- alta saturazione del terreno;
- drenaggio insufficiente;
- vicinanza a fiumi, laghi o costa.

La tossicità va applicata in seguito come modificatore.

### 1.4 Patch noise senza causalità

Le patch a bassa frequenza servono per evitare il rumore a scacchiera, ma non spiegano perché esista un biome. Usare il noise come variazione:

```text
biome_score = climate_score + local_noise * small_amplitude
```

non come causa primaria:

```text
biome = local_noise > threshold
```

### 1.5 Priorità rigida

Una priorità come:

```text
Swamp -> Desert -> Tundra -> Forest -> Plain
```

crea discontinuità e rende difficile il tuning. Applicare prima soltanto vincoli fisici assoluti e poi usare score climatici continui.

### 1.6 Mancanza di idrologia

Senza fiumi, drenaggio, vento e rain shadow, i biomi diventano macchie indipendenti. Il generatore deve poter produrre almeno una relazione come:

```text
oceano -> costa umida -> foresta -> montagne -> steppa/deserto
```

### 1.7 Feature rettangolari

`Crater`, `CrystalField` e `Lake` piazzati come rettangoli sono visivamente artificiali. Usare maschere organiche deformate oppure derivare le feature dalla geomorfologia.

---

## 2. Vincoli della griglia 128×80

### 2.1 Scala

La griglia ha 10.240 celle. La sua utilità dipende dalla scala fisica della cella:

| Scala cella | Estensione approssimativa | Interpretazione |
|---|---:|---|
| 10 m | 1,28×0,8 km | area piccola |
| 50 m | 6,4×4 km | valle o isola compatta |
| 100 m | 12,8×8 km | regione esplorabile |
| 500 m | 64×40 km | macro-regione |
| 1 km | 128×80 km | regione molto ampia |

Per un gioco di esplorazione, una scala di 50–200 metri per cella è generalmente un buon compromesso. La stessa griglia può rappresentare una regione compatta oppure un territorio molto più vasto, ma la dimensione percepita dei biomi deve rimanere coerente con questa scelta.

### 2.2 Quantità ragionevole di biomi

Non obbligare ogni mappa a contenere tutti i biomi in proporzioni equivalenti. Una configurazione credibile può avere:

- 2–4 macro-regioni principali;
- 5–8 biomi dominanti;
- alcuni biomi di transizione;
- feature rare e localizzate.

La mappa può contenere quasi tutti i biomi indicati nell'insieme delle generazioni, senza obbligare ogni singolo seed a contenere una grande regione di ciascuno.

### 2.3 Larghezze minime consigliate

Questi valori sono criteri di design, non regole fisiche:

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
| Feature piccola | 3–5 celle di diametro |

Un bioma largo soltanto 2–4 celle non deve essere considerato una macro-regione: è meglio trattarlo come transizione, corridoio ecologico o decorazione.

### 2.4 Distribuzione iniziale consigliata

Usare come punto di partenza, non come vincolo rigido:

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

Le percentuali vanno adattate al profilo del mondo. Una mappa vulcanica può avere più roccia e meno foresta; una mappa temperata può avere poca tundra; una mappa arida può avere un deserto dominante ma non necessariamente una palude estesa.

### 2.5 Orientamento della mappa

Il rapporto 128/80 è 1,6 e funziona bene per un gradiente climatico verticale e una direzione del vento orizzontale.

Una configurazione leggibile è:

```text
ovest                                      est

oceano -> costa umida -> foresta -> montagne -> steppa -> deserto
```

Non rendere obbligatoria questa disposizione, ma usare vento, costa e rilievi per creare pattern climatici simili.

---

## 3. Profili di generazione

Non usare una sola distribuzione per tutti i mondi. Definire profili configurabili.

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

Ogni profilo deve controllare:

- frazione d'acqua;
- temperatura media;
- intensità del gradiente latitudinale;
- quantità di montagne;
- prevalenza di foresta, deserto o tundra;
- numero massimo di feature;
- probabilità di laghi e paludi;
- biomi obbligatori e opzionali.

Struttura suggerita:

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

Il generatore può rigenerare il mondo se manca un biome obbligatorio, ma deve evitare di imporre ogni biome opzionale.

---

## 4. Modello dati

Mantenere i valori continui e le classificazioni separate.

```rust
#[derive(Clone, Debug)]
pub struct Cell {
    // Geografia
    pub elevation: f32,
    pub slope: f32,
    pub curvature: f32,
    pub latitude: f32,
    pub distance_to_coast: f32,
    pub distance_to_water: f32,

    // Idrologia
    pub rainfall: f32,
    pub humidity: f32,
    pub soil_moisture: f32,
    pub water_saturation: f32,
    pub flow_accumulation: f32,
    pub flow_direction: Option<Direction>,
    pub river_width: f32,
    pub is_lake: bool,
    pub is_river: bool,

    // Clima
    pub temperature: f32,
    pub wind: Vec2,

    // Classificazioni
    pub terrain_kind: TerrainKind,
    pub biome: Biome,
    pub secondary_biome: Option<Biome>,
    pub biome_blend: f32,

    // Gameplay e feature
    pub toxicity: f32,
    pub fertility: f32,
    pub feature: Option<Feature>,
}
```

Se il progetto usa una struttura SoA, mantenere layer separati:

```rust
pub struct WorldFields {
    pub elevation: Grid<f32>,
    pub slope: Grid<f32>,
    pub curvature: Grid<f32>,
    pub rainfall: Grid<f32>,
    pub temperature: Grid<f32>,
    pub moisture: Grid<f32>,
    pub flow: Grid<f32>,
    pub biomes: Grid<Biome>,
}
```

---

## 5. Risoluzione gerarchica

La mappa giocabile può essere 128×80 senza obbligare tutti i processi a usare la stessa risoluzione.

### 5.1 Climate grid

Calcolare il clima su una griglia 32×20 o 64×40:

- temperatura;
- vento;
- precipitazione;
- umidità;
- macro-biome;
- rain shadow.

Interpolare o propagare il risultato alla griglia 128×80. Questo mantiene grandi regioni climatiche e impedisce cambi di biome cella per cella.

### 5.2 Terrain grid

Calcolare a 128×80:

- elevazione;
- pendenza;
- coste;
- valli;
- fiumi;
- laghi;
- zone sature;
- transizioni locali.

### 5.3 Dettaglio locale

Se servono alberi, rocce, risorse e ostacoli più fini, suddividere una cella macro in sottocelle:

```text
128×80 macro-cells
ogni macro-cell -> 4×4 o 8×8 subtiles
```

Il clima rimane stabile sulla macro-cella, mentre la decorazione usa noise e regole locali.

---

## 6. Pipeline completa

```text
1. create_generation_profile
2. generate_macro_terrain
3. generate_heightfield
4. derive_geomorphology
5. classify_water
6. fill_or_breach_depressions
7. estimate_initial_rainfall
8. compute_preliminary_hydrology
9. compute_climate
10. compute_final_hydrology
11. derive_macro_biome_regions
12. classify_biome_scores
13. apply_hydrology_overlays
14. validate_biome_distribution
15. place_geological_features
16. apply_gameplay_modifiers
17. generate_decoration
18. validate_world
```

Dipendenze:

- il clima legge elevazione, latitudine e costa;
- l'idrologia legge elevazione e precipitazione;
- le paludi leggono idrologia e clima;
- i biomi leggono temperatura, pioggia, umidità, elevazione, pendenza e acqua;
- le feature conservano i campi sottostanti;
- la decorazione viene per ultima.

---

## 7. Macro-geografia

Usare noise a bassa frequenza per continenti e regioni. Separare macro-struttura e dettaglio.

```rust
fn generate_macro_terrain(
    width: usize,
    height: usize,
    seed: u64,
) -> Grid<f32> {
    let continent = low_frequency_noise(seed ^ CONTINENT_SEED_OFFSET);
    let regional = low_frequency_noise(seed ^ REGION_SEED_OFFSET);
    let falloff = island_falloff_mask(width, height);

    map_grid(width, height, |x, y| {
        let base = continent.sample(x, y);
        let region = regional.sample(x, y) * 0.20;
        base + region - falloff[x, y]
    })
}
```

Se il mondo non deve essere sempre un'isola, rendere opzionale la falloff ai bordi.

La mappa climatica deve vedere le macro-regioni, non il dettaglio di ogni cella.

---

## 8. Heightfield e geomorfologia

### 8.1 Elevazione multi-scala

```text
final_elevation =
    continental_base
    + mountain_ranges
    + hills
    + local_detail
```

Il dettaglio locale deve avere ampiezza molto inferiore alla struttura continentale.

### 8.2 Catene montuose

Usare ridged noise, ridge lines, spline o maschere regionali.

```rust
let ridge = ridged_noise.sample(domain_warp(x, y));
let mask = mountain_regions.sample(x, y).smoothstep(0.35, 0.65);
let mountain_height = ridge.powf(2.5) * mask;
```

Su una mappa 128×80 è preferibile avere:

- una catena principale;
- zero, una o due catene secondarie;
- alcune cime isolate.

Molte montagne indipendenti renderebbero il terreno frammentato e lascerebbero troppo poco spazio ai biomi.

### 8.3 Gradienti

```rust
slope = gradient_magnitude(elevation);
curvature = laplacian(elevation);
distance_to_coast = distance_transform(!land_mask);
```

Usare la pendenza per:

- roccia esposta;
- paludi;
- costruibilità;
- velocità di drenaggio;
- foreste di montagna.

Usare la curvatura per identificare:

- creste;
- valli;
- depressioni;
- bacini potenziali.

### 8.4 Erosione

Applicare una simulazione leggera e deterministica:

1. hydraulic erosion;
2. thermal erosion sui pendii ripidi;
3. sediment deposition nelle zone basse;
4. ricalcolo di pendenza e curvatura.

Limitare le iterazioni per garantire performance e riproducibilità.

---

## 9. Clima

### 9.1 Temperatura

Usare valori normalizzati `0.0..=1.0`.

```rust
pub fn compute_temperature(
    latitude: f32,
    elevation: f32,
    distance_to_coast: f32,
    local_variation: f32,
    config: &ClimateConfig,
) -> f32 {
    let latitude_temperature =
        (std::f32::consts::PI * latitude).sin();
    let altitude_cooling =
        elevation * config.altitude_lapse_rate;
    let coastal_moderation =
        (1.0 - distance_to_coast).max(0.0)
        * config.coastal_moderation;

    saturate(
        latitude_temperature
            - altitude_cooling
            + coastal_moderation
            + local_variation
                * config.temperature_noise_amplitude,
    )
}
```

La temperatura deve essere più bassa con alta quota e nelle regioni fredde definite dal profilo. Il noise climatico deve essere debole.

### 9.2 Vento

Usare una direzione prevalente per macro-regione:

```rust
wind = prevailing_wind(latitude)
     + low_frequency_wind_noise * WIND_VARIATION;
```

Non usare vento completamente casuale per ogni cella.

### 9.3 Umidità e pioggia

Stima iniziale:

```text
initial_rainfall = ocean_proximity * base_humidity
```

Modello iterativo semplificato:

```rust
for _ in 0..config.wind_steps {
    moisture = advect(moisture, wind);

    for cell in cells {
        let lift = uphill_component(
            wind[cell],
            elevation_gradient[cell],
        );

        let condensation = moisture[cell]
            * config.condensation_rate
            * lift.max(0.0)
            * temperature_condensation_factor(
                temperature[cell],
            );

        rainfall[cell] += condensation;
        moisture[cell] -= condensation;
    }
}
```

Lato sopravento:

```text
più umidità + sollevamento orografico -> più pioggia
```

Lato sottovento:

```text
meno umidità residua -> rain shadow -> regione più arida
```

La mappa dovrebbe poter produrre la sequenza:

```text
oceano -> foresta umida -> cresta -> steppa/deserto
```

### 9.4 Umidità del suolo

```text
soil_moisture =
    precipitation * rainfall_retention(slope)
    + river_proximity * river_moisture_bonus
    + lake_proximity * lake_moisture_bonus
    - evaporation(temperature)
    - drainage(slope, curvature)
```

La saturazione aumenta con:

- bassa pendenza;
- depressioni;
- flusso accumulato;
- vicinanza a fiumi o laghi;
- precipitazioni alte.

---

## 10. Idrologia

### 10.1 Depressioni

Prima dei fiumi:

- riempire piccole depressioni;
- aprire un'uscita tramite breaching;
- conservare grandi depressioni come laghi potenziali.

### 10.2 Flow accumulation

```rust
pub fn compute_flow_accumulation(
    elevation: &Grid<f32>,
    rainfall: &Grid<f32>,
) -> (Grid<f32>, Grid<Option<Direction>>) {
    let mut flow = rainfall.clone();
    let mut directions = Grid::filled(None);
    let mut cells = all_cells(elevation);

    cells.sort_by(|a, b| {
        elevation[*b].total_cmp(&elevation[*a])
    });

    for cell in cells {
        let next = lowest_downhill_neighbor(cell, elevation);
        directions[cell] = next.map(|n| direction_between(cell, n));

        if let Some(next_cell) = next {
            flow[next_cell] += flow[cell];
        }
    }

    (flow, directions)
}
```

Gestire plateau e depressioni in modo esplicito.

### 10.3 Fiumi

```text
is_river = flow_accumulation > river_threshold
river_width = river_scale * sqrt(flow_accumulation)
```

Su una griglia 128×80 limitare inizialmente a:

- 1–3 fiumi principali;
- alcuni affluenti;
- pochi delta;
- percorsi di 20–40 celle per i fiumi principali.

### 10.4 Laghi

Derivare i laghi da depressioni con sufficiente accumulo d'acqua. I laghi di design devono essere maschere deformate, non rettangoli.

```rust
radius(angle) = base_radius
    * (1.0 + noise.sample(angle * frequency) * distortion);
```

Dopo un lago, aggiornare distanza dall'acqua, umidità, saturazione e fiumi locali.

---

## 11. Quantità e posizionamento dei biomi

### 11.1 Macro-regioni prima dei biomi locali

Prima di classificare ogni cella, costruire macro-regioni climatiche a bassa frequenza.

Possibili sorgenti:

- climate grid 32×20 o 64×40;
- Voronoi regionale con 4–8 regioni;
- watershed principali;
- combinazione di temperatura e precipitazione filtrate.

Ogni macro-regione dovrebbe avere un'identità climatica prevalente:

```rust
pub struct MacroRegion {
    pub id: usize,
    pub cells: Vec<CellCoord>,
    pub dominant_temperature: f32,
    pub dominant_rainfall: f32,
    pub biome: Biome,
    pub area: usize,
}
```

Una regione può poi contenere biomi secondari lungo costa, fiumi, montagne e bordi.

### 11.2 Budget di biomi

Definire un budget target per ogni generazione.

```rust
pub struct BiomeBudget {
    pub min_fraction: f32,
    pub max_fraction: f32,
    pub min_component_area: usize,
    pub max_components: usize,
    pub required: bool,
}
```

Esempio di configurazione temperata:

```text
DeepWater: 15–35%, richiesto
ShallowWater: 5–15%, richiesto
Plain: 15–25%, richiesto
Forest: 10–20%, richiesto
Mountain: 8–18%, richiesto
Desert: 0–10%, opzionale
Swamp: 2–6%, opzionale
Tundra: 0–8%, opzionale
BareRock: 1–8%, opzionale
```

I biomi opzionali possono essere esclusi se la geografia non li supporta. Non forzare un deserto in un mondo completamente umido o una tundra in un mondo troppo caldo.

### 11.3 Regole di presenza

Un biome deve essere considerato presente se ha una componente con area minima, non se contiene una sola cella.

```rust
fn biome_is_present(
    grid: &Grid<Biome>,
    biome: Biome,
    min_component_area: usize,
) -> bool {
    largest_connected_component(grid, biome)
        >= min_component_area
}
```

Se un biome opzionale ha solo 1–3 celle, trattarlo come transizione o non considerarlo presente ai fini del profilo.

### 11.4 Distribuzione non uniforme

Il generatore deve evitare di assegnare ogni biome con probabilità indipendente. Preferire:

```text
macro-region dominant biome
    + hydrological overlays
    + mountain/elevation corrections
    + small boundary noise
```

Esempio:

```text
regione umida temperata
    -> Forest dominante
    -> Plain ai margini
    -> Swamp nei bacini
    -> RiverForest lungo i fiumi
```

### 11.5 Transizioni

Usare biomi secondari e blending:

```rust
pub struct BiomeAssignment {
    pub primary: Biome,
    pub secondary: Option<Biome>,
    pub primary_score: f32,
    pub secondary_score: f32,
    pub blend: f32,
}
```

Un confine forest/desert deve avere eventualmente:

```text
Forest -> Woodland -> Grassland -> Steppe -> Desert
```

Non è necessario definire tutti questi nomi come biomi gameplay: possono essere stati di transizione visivi o ecologici.

### 11.6 Validazione e retry

Dopo la classificazione, calcolare:

- frazione di ogni biome;
- area della componente maggiore;
- numero di componenti;
- numero di cambi di biome tra celle adiacenti;
- numero di biomi richiesti mancanti.

Rigenerare l'intero mondo solo per violazioni macroscopiche:

```text
acqua troppo poca
nessuna regione terrestre grande
biome richiesto mancante
montagne eccessive
nessun percorso tra interno e costa
```

Non rigenerare una mappa solo perché una singola palude è piccola.

---

## 12. Classificazione finale dei biomi

### 12.1 Vincoli fisici assoluti

Applicare prima:

```rust
if is_deep_water(cell) {
    return Biome::DeepWater;
}

if is_shallow_water(cell) {
    return Biome::ShallowWater;
}

if is_glacier(cell) {
    return Biome::Glacier;
}

if is_exposed_rock(cell) {
    return Biome::BareRock;
}
```

### 12.2 Tabella temperatura/precipitazione

| Temperatura | Pioggia bassa | Pioggia media | Pioggia alta |
|---|---|---|---|
| Alta | Deserto caldo | Savana/steppa | Foresta calda |
| Media | Steppa/deserto temperato | Prateria | Foresta temperata |
| Bassa | Deserto freddo | Tundra | Taiga |
| Molto bassa | Ghiaccio | Tundra | Taiga fredda |

Ridurre la tabella se il gioco ha pochi biomi.

### 12.3 Score continui

```rust
fn biome_score(cell: &ClimateCell, biome: Biome) -> f32 {
    match biome {
        Biome::Forest => {
            moisture_fit(cell.soil_moisture, 0.70, 0.25)
                * temperature_fit(cell.temperature, 0.55, 0.30)
                * slope_fit(cell.slope, 0.0, 0.45)
        }
        Biome::Desert => {
            dryness(cell.rainfall)
                * temperature_fit(cell.temperature, 0.75, 0.35)
                * slope_fit(cell.slope, 0.0, 0.65)
        }
        Biome::Tundra => {
            coldness(cell.temperature)
                * low_to_medium_moisture(cell.rainfall)
        }
        Biome::Plain => {
            moderate_temperature(cell.temperature)
                * moderate_moisture(cell.soil_moisture)
                * low_slope(cell.slope)
        }
        _ => 0.0,
    }
}
```

Le funzioni di compatibilità devono restituire valori `0.0..=1.0` e usare curve morbide, non confronti binari.

### 12.4 Paludi

```text
slope < swamp_max_slope
&& water_saturation > swamp_min_saturation
&& rainfall > swamp_min_rainfall
&& (
    elevation bassa
    || flow_accumulation alta
    || vicinanza all'acqua
)
```

Poi applicare tossicità:

```rust
if biome == Biome::Swamp && cell.toxicity >= TOXIC_THRESHOLD {
    cell.feature = Some(Feature::ToxicZone { intensity });
}
```

### 12.5 Montagne

Distinguere:

```text
alta quota + temperatura bassa -> Glacier/Snow
alta pendenza + temperatura media -> BareRock
quota alta + temperatura moderata -> AlpineMeadow
pendenza moderata + umidità -> MountainForest
```

Non assegnare un singolo biome identico a tutta la catena montuosa.

---

## 13. Feature geologiche

### 13.1 Overlay separati

```rust
#[derive(Clone, Debug)]
pub enum Feature {
    VolcanicVent { intensity: f32, radius: f32 },
    Crater { radius: f32, depth: f32 },
    CrystalField { density: f32 },
    Lake { depth: f32 },
    ToxicZone { intensity: f32 },
}
```

Una cella può essere:

```text
Mountain + VolcanicAsh + Toxic
```

senza perdere il biome climatico.

### 13.2 Maschere organiche

```rust
fn organic_disk(
    cell: Vec2,
    center: Vec2,
    base_radius: f32,
    noise: &Noise,
) -> f32 {
    let delta = cell - center;
    let distance = delta.length();
    let angle = delta.y.atan2(delta.x);
    let distortion =
        noise.sample(angle * FEATURE_FREQUENCY)
        * FEATURE_DISTORTION;
    let radius = base_radius * (1.0 + distortion);

    smoothstep(
        radius,
        radius - FEATURE_EDGE_SOFTNESS,
        distance,
    )
}
```

### 13.3 Budget feature per 128×80

Come punto di partenza:

```text
Crater: 0–2
CrystalField: 0–3
Lake: 1–5
VolcanicVent: 0–3
ToxicZone: 0–3
```

La quantità deve dipendere dal profilo. Un mondo con 12 feature grandi può diventare più artificiale di un mondo con 3 feature ben posizionate.

### 13.4 Vulcani

Modificare:

- elevazione o pendenza locale;
- temperatura locale;
- sterilità;
- tossicità;
- roccia e cenere;
- possibile lava.

Non cancellare la temperatura climatica. Aggiungere un modificatore:

```rust
cell.temperature += volcanic_heat;
cell.toxicity += volcanic_toxicity;
```

### 13.5 Crateri

Usare una depressione con bordo rialzato:

```text
crater_delta = rim_profile(distance) - bowl_profile(distance)
```

Aggiornare localmente pendenza, drenaggio e possibilità di lago.

---

## 14. Seed e determinismo

```rust
const TERRAIN_SEED_OFFSET: u64 = 0x01;
const TERRAIN_DETAIL_SEED_OFFSET: u64 = 0x02;
const CLIMATE_SEED_OFFSET: u64 = 0x03;
const HYDROLOGY_SEED_OFFSET: u64 = 0x04;
const REGION_SEED_OFFSET: u64 = 0x05;
const FEATURE_SEED_OFFSET: u64 = 0x06;
const DECORATION_SEED_OFFSET: u64 = 0x07;
```

Regole:

- stessa coppia `(seed, config)` -> stesso mondo;
- cambiare il seed della decorazione non cambia biomi o elevazione;
- cambiare il seed delle feature non cambia il clima;
- evitare `self.rng` globale negli stage deterministici;
- campionare il noise per coordinate;
- loggare il seed di ogni stage.

---

## 15. Configurazione

```rust
pub struct TerrainConfig {
    pub sea_level: f32,
    pub mountain_threshold: f32,
    pub detail_amplitude: f32,
    pub erosion_iterations: usize,
}

pub struct ClimateConfig {
    pub altitude_lapse_rate: f32,
    pub coastal_moderation: f32,
    pub temperature_noise_amplitude: f32,
    pub prevailing_wind: Vec2,
    pub wind_steps: usize,
    pub condensation_rate: f32,
    pub rain_shadow_strength: f32,
}

pub struct HydrologyConfig {
    pub river_threshold: f32,
    pub lake_threshold: f32,
    pub depression_fill_limit: usize,
    pub swamp_max_slope: f32,
    pub swamp_min_saturation: f32,
    pub swamp_min_rainfall: f32,
}

pub struct BiomeConfig {
    pub snowline_temperature: f32,
    pub bare_rock_slope: f32,
    pub blend_threshold: f32,
    pub biome_noise_amplitude: f32,
    pub budgets: HashMap<Biome, BiomeBudget>,
}
```

Non lasciare soglie sparse in `world.rs`. Il tuning deve avvenire tramite config e profili.

---

## 16. Pseudocodice della generazione

```rust
pub fn generate_world(
    seed: u64,
    config: &WorldConfig,
) -> World {
    let mut world = World::new(
        config.width,
        config.height,
    );

    let profile = create_generation_profile(
        config.profile,
        &config.profiles,
    );

    generate_macro_terrain(
        &mut world,
        seed ^ TERRAIN_SEED_OFFSET,
        &config.terrain,
    );

    generate_heightfield(
        &mut world,
        seed ^ TERRAIN_DETAIL_SEED_OFFSET,
        &config.terrain,
    );

    derive_geomorphology(&mut world, &config.terrain);
    classify_water(&mut world, &config.terrain);
    fill_depressions(&mut world, &config.hydrology);

    let initial_rainfall = estimate_oceanic_rainfall(
        &world,
        &config.climate,
    );

    compute_preliminary_hydrology(
        &mut world,
        &initial_rainfall,
        &config.hydrology,
    );

    compute_climate(
        &mut world,
        seed ^ CLIMATE_SEED_OFFSET,
        &config.climate,
    );

    compute_final_hydrology(
        &mut world,
        &config.hydrology,
    );

    derive_macro_biome_regions(
        &mut world,
        seed ^ REGION_SEED_OFFSET,
        &profile,
    );

    classify_climate_biomes(
        &mut world,
        &config.biomes,
    );

    apply_hydrology_overlays(
        &mut world,
        &config.biomes,
    );

    if !validate_biome_distribution(
        &world,
        &profile,
    ) {
        return retry_or_fail(seed, config);
    }

    place_geological_features(
        &mut world,
        seed ^ FEATURE_SEED_OFFSET,
        &config.features,
    );

    apply_gameplay_modifiers(
        &mut world,
        &config.gameplay,
    );

    generate_decoration(
        &mut world,
        seed ^ DECORATION_SEED_OFFSET,
        &config.decoration,
    );

    validate_world(&world, config);
    world
}
```

---

## 17. Migrazione dal codice attuale

### Fase A: conservare Stage A

Mantenere:

- retry dell'intero heightfield;
- stream RNG dedicato;
- normalizzazione;
- `is_peak` dove utile.

Aggiungere:

- pendenza;
- curvatura;
- distanza dalla costa;
- elevazione continua disponibile agli stage successivi.

### Fase B: correggere la temperatura

Sostituire il ruolo climatico di `light` con:

```text
latitudine + altitudine + costa + noise debole
```

### Fase C: introdurre la precipitazione

Prima versione:

```text
rainfall = ocean_proximity
         * wind_exposure
         * orographic_lift
         * rain_shadow
```

### Fase D: introdurre l'idrologia

Implementare:

- flow direction;
- flow accumulation;
- fiumi;
- laghi da depressioni;
- saturazione del terreno.

### Fase E: introdurre climate regions e budget

Prima creare macro-regioni climatiche; poi assegnare biomi alle celle. Validare area minima, frazione e connettività.

### Fase F: classificazione a score

Passare da:

```text
TerrainKind -> Biome
```

a:

```text
TerrainFields + ClimateFields + HydrologyFields
    -> BiomeScores
```

### Fase G: feature

Sostituire rettangoli con maschere deformate. Conservare biome base, clima e feature in campi distinti.

### Fase H: debug

Aggiungere viste separate per:

- elevazione;
- pendenza;
- curvatura;
- temperatura;
- precipitazione;
- umidità;
- flow accumulation;
- fiumi;
- saturazione;
- macro-regioni;
- biome score;
- maschere feature.

---

## 18. Test automatici

### 18.1 Determinismo

```rust
#[test]
fn same_seed_produces_same_world() {
    let a = generate_world(TEST_SEED, config());
    let b = generate_world(TEST_SEED, config());
    assert_eq!(a.digest(), b.digest());
}
```

### 18.2 Separazione degli stream

Cambiare il seed di decorazione non deve modificare:

- elevazione;
- temperatura;
- precipitazione;
- idrologia;
- biomi.

### 18.3 Vincoli locali

Verificare che:

- l'acqua abbia elevazione sotto il livello del mare;
- i ghiacciai abbiano temperatura bassa;
- le paludi abbiano pendenza bassa e saturazione alta;
- i deserti non siano su celle molto piovose;
- la roccia esposta abbia pendenza o quota coerente;
- i fiumi scorrano verso quote non crescenti, salvo eccezioni esplicite.

### 18.4 Distribuzione globale

Per ogni profilo verificare:

- frazione d'acqua nel range;
- presenza dei biomi richiesti;
- area minima delle componenti principali;
- numero massimo di componenti frammentate;
- nessun biome dominante sotto la dimensione minima;
- massimo numero di feature rispettato.

### 18.5 Relazioni climatiche

Usare test statistici:

- temperatura media più bassa ad alta quota;
- temperatura coerente con il gradiente latitudinale;
- precipitazione maggiore sul lato sopravento;
- precipitazione minore oltre le montagne;
- saturazione maggiore vicino a fiumi e laghi;
- paludi concentrate in zone basse e pianeggianti.

### 18.6 Continuità spaziale

Calcolare:

- numero di cambi di biome tra celle adiacenti;
- dimensione media delle regioni;
- numero di isole di una singola cella;
- connettività delle foreste e dei deserti;
- lunghezza dei fiumi;
- componenti disconnesse non desiderate.

---

## 19. Metriche di debug

Registrare per ogni generazione:

```text
land_fraction
water_fraction
mean_elevation
mountain_fraction
mean_temperature
mean_rainfall
river_fraction
lake_fraction
swamp_fraction
forest_fraction
desert_fraction
tundra_fraction
largest_land_region
largest_biome_region
number_of_biome_components
number_of_required_biomes_present
number_of_biome_transitions
number_of_special_features
```

Controlli macroscopici possibili:

```rust
assert!(land_fraction > profile.min_land_fraction);
assert!(largest_land_region > profile.min_mainland_fraction);
assert!(river_fraction < profile.max_river_fraction);
assert!(swamp_fraction < profile.max_swamp_fraction);
```

I retry devono essere usati per problemi globali, non per correggere ogni cella individuale.

---

## 20. Criteri di accettazione visivi

La generazione è accettabile quando:

- i continenti sono continui e non semplici macchie;
- le montagne formano catene leggibili;
- esiste almeno una valle o un bacino riconoscibile;
- i fiumi partono da zone alte e raggiungono mare o laghi;
- le paludi sono in zone umide e pianeggianti;
- i deserti sono in regioni con bassa precipitazione;
- il lato sottovento delle montagne può essere più secco;
- le foreste seguono temperatura e umidità;
- i biomi principali sono larghi abbastanza da essere riconosciuti;
- i confini sono irregolari ma non rumorosi;
- i laghi non sono rettangolari;
- le feature speciali non dominano la mappa;
- la distribuzione varia tra profili ma resta credibile;
- la stessa configurazione produce sempre lo stesso mondo.

---

## 21. Performance

Per 128×80 il numero di celle è contenuto, ma gli stage devono comunque essere separati in passaggi prevedibili:

```text
pass 1: elevation
pass 2: gradients
pass 3: climate macro-grid
pass 4: hydrology
pass 5: biome regions
pass 6: biome scores
pass 7: features
pass 8: decoration
```

Ottimizzazioni:

- `f32` per i campi;
- noise climatico su griglia ridotta;
- interpolazione verso 128×80;
- buffer contigui;
- parallelizzazione di noise, gradienti e biome score;
- modalità `fast_preview` con meno iterazioni;
- modalità `quality` per la generazione finale.

Flow accumulation, depression filling e trasporto dell'umidità hanno dipendenze tra celle e vanno parallelizzati solo quando l'algoritmo lo consente.

---

## 22. Decisione progettuale finale

La griglia 128×80 deve essere trattata come una **mappa regionale compatta**.

Una singola generazione dovrebbe normalmente contenere:

```text
1 costa o oceano
1 catena montuosa principale
1 grande foresta o regione umida
1 regione di pianura/prateria
0–1 regione arida significativa
0–1 regione fredda o alpina
1–3 fiumi principali
1–5 laghi
0–3 feature speciali principali
```

Gli altri biomi possono apparire come:

- transizioni;
- sottoregioni;
- zone lungo fiumi e coste;
- feature ecologiche;
- risultati di profili diversi.

Non obbligare ogni seed ad avere un grande deserto, una grande tundra e una grande palude. Prima generare una geografia che supporti naturalmente questi biomi; poi validare soltanto i biomi richiesti dal profilo.

La qualità percepita dipenderà più dalla coerenza delle relazioni:

```text
montagna -> pendenza -> drenaggio
costa -> umidità -> pioggia
vento -> rain shadow
bassa quota + saturazione -> palude
quota + temperatura -> tundra/neve
```

che dal numero assoluto di biomi presenti.

---

## Riferimenti

- [Red Blob Games — Making maps with noise functions](https://www.redblobgames.com/maps/terrain-from-noise/)
- [Red Blob Games — Wind patterns](https://www.redblobgames.com/x/1731-wind-patterns/)
- [Red Blob Games — Procedural map generation on a sphere](https://www.redblobgames.com/x/1843-planet-generation/)
- [AutoBiomes: procedural generation of multi-biome landscapes](https://cgvr.cs.uni-bremen.de/papers/cgi20/AutoBiomes.pdf)
- [Mapgen4 — Procedural wilderness map generator](https://www.redblobgames.com/maps/mapgen4/)
