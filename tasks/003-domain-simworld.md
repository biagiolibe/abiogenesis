# Task 003 — Tipi di dominio e resource `SimWorld`

> **ID**: `003`
> **Categoria**: Architettura
> **Priorità**: 🔴 P1
> **Stima**: ~2h
> **Assegnato a**: non assegnato
> **Sessione**: —

---

## 🎯 Obiettivo

Definire i tipi che descrivono il mondo simulato — griglia, celle, organismi, specie, metabolismi — e la resource **`SimWorld`** che li contiene, **incluso l'RNG seedato**.

È il fondamento su cui poggiano tutti i task successivi. La decisione strutturale (`TECH_DESIGN.md` §3.1) è che **la griglia è una `Resource` con array densi, non entità ECS**: le entità Bevy serviranno solo alla resa (task 006).

---

## 📋 Acceptance Criteria

- [ ] `SimWorld::new(seed, &SimConfig)` costruisce un mondo `48×32`.
- [ ] **Determinismo**: due `SimWorld::new(42, &cfg)` producono stato identico; seed diversi producono stato diverso. Coperto da test.
- [ ] L'RNG è **conservato dentro `SimWorld`**, non creato al volo.
- [ ] Esiste un helper riusabile per il **vicinato di Moore (8)** che gestisce correttamente i bordi; test unitario che verifica 3 vicini in un angolo, 5 su un lato, 8 al centro.
- [ ] Doppio buffer predisposto: `SimWorld` può produrre uno snapshot e scrivere su un buffer successivo (task 005 lo userà).
- [ ] **Nessun `use bevy::render` e nessun `bevy_egui`** in `src/world.rs`.
- [ ] `SimWorld` è costruibile e usabile **senza `App` Bevy** (verificato dal fatto che i test non ne creano una).
- [ ] `cargo clippy -- -D warnings` pulito.

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `src/world.rs` | Tipi di dominio, `SimWorld`, `WorldPlugin` |
| `src/config.rs` | `SimConfig`, già disponibile (task 002) |

---

## 🧩 Contesto Tecnico

- **Comportamento attuale**: `src/world.rs` è uno stub vuoto; `SimConfig` esiste.
- **Comportamento desiderato**: `SimWorld` disponibile come resource, con griglia allocata e RNG seedato.

### Il genoma di specie (GDD §5.3)

Ogni specie è definita da:
- **Metabolismo** — come ricava energia;
- **Intervallo ambientale preferito** — `temp_optimum` + `temp_tolerance`, con fitness gaussiana attorno all'ottimo;
- **Soglia di riproduzione**;
- **Da 1 a 3 tag biochimici** — *l'unica cosa che conta per le interazioni tra specie* (GDD §5.5).

Metabolismi e intervalli ambientali sono **leggibili** dal giocatore; i tag sono **opachi**.

### I metabolismi (GDD §5.4)

- `Photolithic` — ricava energia dalla `light` locale. Produttore primario.
- `Predator` — ricava energia dagli organismi vicini.
- `Decomposer` — ricava energia dai residui.

Vanno definiti **tutti e tre come tipo** anche se in Fase 0 solo il fotolitico è attivo: evita un refactor in Fase 1.

### Determinismo (GDD §5.7)

> La simulazione è **deterministica** a parità di seed: RNG seedato conservato nello stato del mondo. Fondamentale per debug dell'emergenza, riproducibilità dei bug e condivisione di seed interessanti.

---

## 🔨 Implementazione Suggerita

1. **Tipi base**

   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum Metabolism {
       Photolithic,
       Predator,
       Decomposer,
   }

   /// Index into SimWorld::species. Kept small: species are few and never removed.
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub struct SpeciesId(pub u8);

   #[derive(Debug, Clone)]
   pub struct Species {
       pub metabolism: Metabolism,
       pub temp_optimum: f32,
       pub temp_tolerance: f32,
       pub repro_threshold: f32,
       pub tags: Vec<TagId>,   // 1..=3 (GDD 5.3)
   }

   #[derive(Debug, Clone, Copy)]
   pub struct Organism {
       pub species: SpeciesId,
       pub energy: f32,
   }

   #[derive(Debug, Clone, Copy, Default)]
   pub struct Cell {
       pub temperature: f32,
       pub light: f32,
       pub toxicity: f32,
       pub organism: Option<Organism>,
       /// Dead matter left behind, feeds decomposers (GDD 5.6 step 6).
       pub residue: f32,
   }
   ```

2. **`SimWorld`** — griglia densa indicizzata `y * width + x`:

   ```rust
   #[derive(Resource)]
   pub struct SimWorld {
       pub width: usize,
       pub height: usize,
       pub cells: Vec<Cell>,
       pub species: Vec<Species>,
       pub tick: u64,
       pub era: u32,
       pub seed: u64,
       rng: StdRng,          // seeded, lives in world state (GDD 5.7)
       scratch: Vec<Cell>,   // double buffer for the tick (TECH_DESIGN 6)
   }
   ```

   Usare `StdRng::seed_from_u64` di `rand`: è riproducibile tra esecuzioni, a differenza di `SmallRng` su piattaforme diverse.

3. **Accessori** — `get(x, y)`, `get_mut(x, y)`, `index(x, y)`, e l'RNG esposto solo tramite `&mut self` così che nessuno possa clonarlo.

4. **Vicinato di Moore** — helper riusabile, il punto più facile da sbagliare ai bordi:

   ```rust
   /// Moore neighbourhood (8 cells), clipped at the grid borders (GDD 5.1).
   pub fn moore_neighbours(&self, x: usize, y: usize) -> impl Iterator<Item = usize> + '_
   ```

   Nessun wrap-around: la griglia ha bordi veri (una cella d'angolo ha 3 vicini). Il GDD non prevede topologia toroidale.

5. **`WorldPlugin`** inserisce `SimWorld` in `Startup`, leggendo `SimConfig`. Il seed di default può venire da un valore fisso in Fase 0 (il reseed interattivo è il task 007).

6. **Test** in `src/world.rs`:
   - due mondi con lo stesso seed sono identici; con seed diversi, no;
   - conteggio dei vicini di Moore: `(0,0)` → 3, bordo → 5, centro → 8.

---

## ⚠️ Vincoli e Attenzioni

- **Invariante 1 (`TECH_DESIGN.md` §5)**: l'RNG vive in `SimWorld`. Niente `rand::rng()` / `thread_rng` da nessuna parte.
- **Invariante 2**: `src/world.rs` non dipende dalla resa. L'unico import Bevy ammesso è quello per `derive(Resource)`.
- **Niente `HashMap` per la griglia**: array densi. L'ordine di iterazione di una mappa è una delle vie più comuni alla perdita di determinismo.
- Verificare l'API di `rand` risolta dal task 001: da `rand` 0.9 alcuni nomi sono cambiati (`thread_rng` → `rng`, `gen` → `random`).
- **Una sola occupazione per cella** (GDD §5.1): `Option<Organism>`, mai una collezione.
- In Fase 0 le specie disponibili si riducono a una sola, fotolitica. Il registro `species` è comunque un `Vec` fin da ora.

---

## 🔗 Dipendenze

- **Dipende da**: 002
- **Blocca**: 004, 006

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/003-domain-simworld.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
