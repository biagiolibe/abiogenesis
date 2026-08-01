# Task 002 — `SimConfig`: coefficienti centralizzati

> **ID**: `002`
> **Categoria**: Architettura
> **Priorità**: 🔴 P1
> **Stima**: ~1h
> **Assegnato a**: non assegnato
> **Sessione**: —

---

## 🎯 Obiettivo

Trascrivere l'intera baseline numerica del GDD §5.9 in una **singola resource `SimConfig`**, punto unico di verità per ogni coefficiente della simulazione.

È necessario perché il GDD §5.6 lo impone come decisione di design: *"tutti i coefficienti sono costanti nominate in un unico punto, così il tuning finale è rapido"*. Il tuning del bilanciamento è dichiarato come il lavoro più delicato del progetto (GDD §13, §14): se i numeri finiscono sparsi nel codice, quel lavoro diventa impraticabile.

Si inseriscono **anche i coefficienti delle fasi successive** (matrice, taccuino, azioni): sono già decisi nel GDD e centralizzarli ora costa zero.

---

## 📋 Acceptance Criteria

- [ ] Ogni valore numerico delle tabelle di GDD §5.9 ha una costante o un campo corrispondente in `SimConfig`.
- [ ] `SimConfig` è registrata come `Resource` da `ConfigPlugin`.
- [ ] Esiste `SimConfig::default()` che restituisce la baseline del GDD.
- [ ] Nessun valore duplicato: ogni numero compare una volta sola.
- [ ] Ogni campo ha un commento con l'unità di misura o il riferimento al GDD.
- [ ] `cargo clippy -- -D warnings` pulito.

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `src/config.rs` | `SimConfig` e `ConfigPlugin` |

---

## 🧩 Contesto Tecnico

- **Comportamento attuale**: `src/config.rs` è uno stub vuoto (task 001).
- **Comportamento desiderato**: `SimConfig` disponibile come resource, con l'intera baseline del GDD.

### La baseline da trascrivere (GDD §5.9)

**Ambiente** — tutte le scalari sono in `[0,1]`

| Costante | Valore |
|---|---|
| Diffusione ambientale (Fase 1+) | `0.05` / tick |
| Gradiente luce: alto → basso | `0.9` → `0.2` |
| Gradiente temperatura: sx → dx | `0.2` → `0.8` |
| Zona tossica | `0.7` (resto `0.0`) |

**Tempo e azioni**

| Costante | Valore |
|---|---|
| `ERA_TICKS` | `25` |
| Budget ere / mondo | `40` (iniziali) → `25` (tardi) |
| Budget punti / era | `3` |
| Costo azioni | seed `1`, stress `1`, cull `1`, splice `2` |

**Energia e metabolismo** (per organismo)

| Costante | Valore |
|---|---|
| Energia al seed | `5.0` |
| `upkeep` base | `0.5` / tick |
| `crowd_factor` | `0.15` / vicino occupato |
| `repro_threshold` | `10.0` |
| `repro_cost` (al figlio) | `5.0` |
| Fotolitico `metabolism_gain` | `2.0` |
| Predatore `drain_cap` / `upkeep` | `2.0` / tick · `0.7` |
| Decompositore `extract_rate` / `upkeep` | `1.5` / tick · `0.5` |
| Residuo alla morte / decadimento | `3.0` · `0.2` / tick |
| `temp_tolerance` (σ) default | `0.15` |

**Tag e matrice**

| Costante | Valore |
|---|---|
| Pool globale tag | `10` |
| Tag attivi / mondo | `5` (iniziali) → `8` (tardi) |
| Tag per specie | `1–3` |
| Intensità effetto / adiacenza | interi in `{−2,−1,0,+1,+2}` |
| Densità matrice | ~`40%` coppie non nulle |

**Taccuino**

| Costante | Valore |
|---|---|
| Soglia di conferma / cella | `3.0` di evidenza cumulata |
| Peso di un'osservazione | `1 / (1 + n_confonditori_adiacenti)` |

**Griglia**

| Costante | Valore |
|---|---|
| Dimensione | `48×32` |
| Vicinato | Moore (8) |

---

## 🔨 Implementazione Suggerita

1. Raggruppare in sotto-struct tematiche invece di un'unica struct piatta da 30 campi — si legge meglio e rispecchia le tabelle del GDD:

   ```rust
   #[derive(Resource, Debug, Clone)]
   pub struct SimConfig {
       pub grid: GridConfig,
       pub environment: EnvironmentConfig,
       pub time: TimeConfig,
       pub energy: EnergyConfig,
       pub tags: TagConfig,
       pub notebook: NotebookConfig,
   }
   ```

2. Ogni sotto-struct implementa `Default` con i valori del GDD:

   ```rust
   #[derive(Debug, Clone)]
   pub struct EnergyConfig {
       /// Energy an organism starts with when seeded.
       pub seed_energy: f32,
       /// Base maintenance cost per tick.
       pub upkeep: f32,
       /// Carrying capacity: penalty per occupied neighbour (GDD 5.9).
       pub crowd_factor: f32,
       // ...
   }
   ```

3. `ConfigPlugin` inserisce la resource:

   ```rust
   impl Plugin for ConfigPlugin {
       fn build(&self, app: &mut App) {
           app.init_resource::<SimConfig>();
       }
   }
   ```

4. Usare `f32` per le scalari continue e `u32`/`i8` per i conteggi e le intensità della matrice.

---

## ⚠️ Vincoli e Attenzioni

- **Nessun valore va "arrotondato" o reinterpretato.** Il GDD §14 è esplicito: i numeri di §5.9 *"vanno confermati o ritoccati col playtest, non reinventati"*.
- `src/config.rs` **non deve dipendere da `bevy::render` né da `bevy_egui`** (invariante 2, `TECH_DESIGN.md` §5). L'unico import Bevy ammesso è quello che serve per `derive(Resource)`.
- I budget che nel GDD sono espressi come intervallo (ere per mondo `40 → 25`, tag attivi `5 → 8`) vanno modellati come coppia inizio/fine, non come valore singolo: serviranno alla curva di difficoltà in Fase 3.
- Non implementare l'hot-reload: è previsto (`TECH_DESIGN.md` §4) ma appartiene alla fase di tuning. Qui basta che la config sia **una sola resource letta e mai duplicata**, così la migrazione sarà indolore.

---

## 🔗 Dipendenze

- **Dipende da**: 001
- **Blocca**: 003

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/002-sim-config.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
