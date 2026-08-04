# Task 037 — `WorldParams` and difficulty curve

> **ID**: `037`
> **Category**: Feature
> **Priority**: 🔴 P1
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

GDD §9 descrive una curva di difficoltà: i primi mondi hanno 5 tag attivi e ambiente mite, salendo gradualmente fino a ~8 tag attivi, matrici più cattive, ambienti più estremi e budget di ere più corti. Il playthrough d'esempio (§16) mostra World 2 (secondo mondo) con **6** tag attivi — non un salto diretto da 5 a 8, quindi la curva è una rampa su più mondi, non due soli livelli "early"/"late".

`SimConfig` ha già `active_tags_early/late` ed `era_budget_early/late`, ma nessun consumer li usa per interpolare — sono pensati come i due estremi di una curva mai scritta. Questo task introduce quella curva come funzione pura, testabile headless, che i task di worldgen (038, 039, 042) consumeranno.

---

## 📋 Acceptance Criteria

- [ ] Nuovo modulo `src/worldgen.rs` con una funzione pura `pub fn world_params(world_index: u32, config: &SimConfig) -> WorldParams`.
- [ ] `WorldParams` include almeno: `active_tag_count: u32`, `era_budget: u32`, `toxic_zone_width: u32`, `toxic_zone_height: u32`, `temperature_spread: f32` (o campo equivalente per l'ampiezza del gradiente termico), `matrix_density: f32`, `objective_severity: f32` (o tipo equivalente consumato dal task 042).
- [ ] Nuovi campi endpoint in `SimConfig` per ogni asse che oggi non ne ha (l'ambiente non ha endpoint early/late oggi — solo tempo e tag): `toxic_zone_width_late`, `toxic_zone_height_late`, `temperature_spread_late` (o nomi equivalenti coerenti con `EnvironmentConfig` esistente), più `difficulty_ramp_worlds: u32` (numero di mondi su cui la rampa satura). Nessun magic number fuori da `SimConfig` (invariante CLAUDE.md).
- [ ] **Criterio letterale dal GDD §16**: `world_params(1, &config).active_tag_count == 6` (World 2, indice 1, ha 6 tag attivi con i valori di default `active_tags_early=5`).
- [ ] `world_params(0, &config)` produce esattamente i valori "early" attuali (5 tag attivi, budget 40 ere, ambiente mite) — nessuna regressione sul comportamento di Fase 0-2 quando si arriverà a usarla (task 038).
- [ ] La rampa satura ai valori `*_late` dopo `difficulty_ramp_worlds` mondi (oltre quell'indice, `world_params` resta costante ai valori late — nessun overflow/valore assurdo per indici alti, coerente con una run "endless-until-failure").
- [ ] `world_params` è testabile senza costruire un `SimWorld` (nessuna dipendenza da `bevy::render`/`bevy_egui`, invariante 2).
- [ ] Test unitari: valore a `world_index=0`, valore a `world_index=1` (vincolo dei 6 tag), saturazione oltre `difficulty_ramp_worlds`.
- [ ] `cargo clippy -- -D warnings` pulito, `cargo test` verde.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/worldgen.rs` (nuovo) | `WorldParams`, `world_params()`, test unitari. |
| `src/config.rs` | Nuovi campi endpoint per la curva ambientale, `difficulty_ramp_worlds`. |
| `src/main.rs` | Registrazione del nuovo modulo (`mod worldgen;`) — nessun Plugin necessario per questo task, è solo una funzione pura. |

---

## 🧩 Technical Context

**`TimeConfig` attuale** (`src/config.rs`, righe ~84-111):
```rust
pub struct TimeConfig {
    pub era_ticks: u32,               // 25
    pub era_budget_early: u32,        // 40
    pub era_budget_late: u32,         // 25
    pub point_budget_per_era: u32,    // 3
    pub action_costs: ActionCosts,
    pub era_tick_hz: f32,             // 20.0
}
```

**`TagConfig` attuale** (`src/config.rs`, righe ~188-221):
```rust
pub struct TagConfig {
    pub global_tag_pool: u32,       // 10
    pub active_tags_early: u32,     // 5
    pub active_tags_late: u32,      // 8
    pub tags_per_species_min: u32,  // 1
    pub tags_per_species_max: u32,  // 3
    pub effect_intensity_min: i8,   // -2
    pub effect_intensity_max: i8,   // 2
    pub matrix_density: f32,        // 0.4
}
```
Entrambi hanno endpoint `early`/`late` ma **nessun consumer** li interpola oggi — `SimWorld::new` usa solo `active_tags_early`, `advance_tick` non legge mai `era_budget_*`.

`EnvironmentConfig` (non ancora letto in dettaglio in questo task — verificare in `config.rs`) ha oggi solo valori statici (dimensione/posizione zona tossica fissa, gradienti fissi), senza endpoint early/late: questo task li introduce.

- **Comportamento attuale**: nessuna funzione converte "in che mondo siamo" in parametri concreti — il concetto stesso di "indice del mondo corrente" non esiste ancora nel codice prima di questo task (arriva con `RunProgress.world_index` dal task 035).
- **Comportamento desiderato**: `world_params(world_index, config)` è la singola fonte di verità per "quanto è difficile il mondo N" — ogni asse di difficoltà (tag, budget, ambiente, densità matrice, severità obiettivo) passa da qui, invece di essere ricalcolato ad-hoc in punti diversi del worldgen.

---

## 🔨 Suggested Implementation

1. Leggere `EnvironmentConfig` in `config.rs` per capire i nomi esatti dei campi ambientali esistenti (zona tossica, gradiente termico) prima di aggiungere gli endpoint `*_late`.
2. Definire `WorldParams` in `src/worldgen.rs` con i campi elencati sopra.
3. Scrivere `world_params` come interpolazione lineare clampata: per ogni asse, `value(world_index) = early + (late - early) * min(world_index, ramp_worlds) / ramp_worlds`, arrotondata/troncata al tipo del campo (`u32` per conteggi/budget, `f32` per ampiezze).
4. Verificare il vincolo letterale: con `active_tags_early=5`, `active_tags_late=8`, quale `difficulty_ramp_worlds` produce `active_tag_count(1) == 6`? Con una rampa lineare su 3 mondi (`ramp_worlds=3`), `world_index=1` → `5 + 3*1/3 = 6`. Impostare `difficulty_ramp_worlds` di default a un valore che soddisfi questo vincolo per l'asse dei tag, e riusarlo per gli altri assi (una sola costante di rampa, non una per asse, salvo necessità emersa in fase di implementazione).
5. Aggiungere i test unitari elencati nei criteri di accettazione.

---

## ⚠️ Constraints and Caveats

- **Funzione pura**: `world_params` non deve leggere `SimWorld`, RNG, o stato mutabile — solo `world_index` e `config`. Questo la rende testabile senza bootstrap Bevy.
- **Nessun magic number**: ogni endpoint/costante di rampa vive in `SimConfig`, non come letterale in `worldgen.rs`.
- **Non generare ancora nulla di procedurale qui**: questo task produce solo *parametri*, non seleziona tag/ambiente/specie/obiettivi concreti — quello è compito dei task 038, 039, 042 che consumano `WorldParams`.
- **Coerenza col modello endless-until-failure**: la curva deve restare sensata anche per `world_index` arbitrariamente alto (saturazione, non estrapolazione illimitata).

---

## 🔗 Dependencies

- **Depends on**: nessuno (parallelo al task 036).
- **Blocks**: 038 (worldgen consuma `WorldParams` per tag/ambiente), 042 (consuma `objective_severity`).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/037-world-params-difficulty-curve.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
