# Task 005 — Algoritmo del tick (Fase 0), puro e headless

> **ID**: `005`
> **Categoria**: Feature
> **Priorità**: 🔴 P1
> **Stima**: ~2h
> **Assegnato a**: non assegnato
> **Sessione**: —

---

## 🎯 Obiettivo

Implementare l'unità atomica della simulazione: **un tick**, secondo i 7 passi di GDD §5.6, limitato a ciò che la Fase 0 richiede (solo metabolismo fotolitico, nessuna matrice di interazione).

È il cuore del progetto. Deve essere **Rust puro, invocabile senza `App` Bevy**: è questa proprietà a rendere possibili i test di determinismo e bilanciamento (task 009) e il tuning finale.

---

## 📋 Acceptance Criteria

- [ ] Esiste `pub fn step(world: &mut SimWorld, config: &SimConfig)`, chiamabile **senza `App` Bevy**.
- [ ] Implementa i 7 passi di GDD §5.6, con `interaction_delta = 0` (nessuna matrice in Fase 0).
- [ ] Usa il **doppio buffer** (snapshot → next), come deciso in `TECH_DESIGN.md` §6.
- [ ] Deterministico: stesso seed + stessa sequenza di `step` ⇒ stato identico.
- [ ] La contesa sulla cella di nascita è risolta in **ordine di indice deterministico**, mai dall'ordine di iterazione.
- [ ] `world.tick` si incrementa a ogni chiamata.
- [ ] **Verifica numerica** — test che riproducono i tre scenari di GDD §5.9 (dettagli sotto).
- [ ] `SimPlugin` espone un sistema che invoca `step`; l'attivazione per stato è il task 007.
- [ ] `cargo clippy -- -D warnings` pulito.

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `src/sim.rs` | `step()`, `SimPlugin` |
| `src/world.rs` | `SimWorld`, vicinato di Moore, doppio buffer (task 003) |
| `src/config.rs` | `SimConfig` (task 002) |

---

## 🧩 Contesto Tecnico

- **Comportamento attuale**: esistono mondo e ambiente, ma nulla si muove.
- **Comportamento desiderato**: chiamando `step` gli organismi guadagnano energia, muoiono e si riproducono.

### GDD §5.6 — i 7 passi, per ogni cella occupata

1. **Fitness ambientale:** `env_fit = gaussian(temperature, temp_optimum, temp_tolerance)` ∈ `[0,1]`.
2. **Guadagno metabolico** — in Fase 0 solo il fotolitico: `gain = light * metabolism_gain * env_fit`.
3. **Effetto della matrice nascosta** → `interaction_delta`. **In Fase 0 vale 0.**
4. **Costi:** `upkeep` + `crowding_penalty = crowd_factor * n_vicini_occupati`.
5. **Aggiornamento energia:** `energy += gain + interaction_delta − upkeep − crowding_penalty`.
6. **Morte:** se `energy <= 0` → l'organismo muore, la cella si libera, lascia un residuo.
7. **Riproduzione:** se `energy >= repro_threshold` ed esiste un vicino vuoto → genera un figlio in un vicino vuoto (scelta casuale seedata) con `repro_cost` energia sottratta al genitore.

La formula di `env_fit` (GDD §5.9): `exp(−(temp − temp_opt)² / (2 · temp_tol²))`, con `temp_tol` (σ) di default `0.15`.

### Verifica numerica attesa (GDD §5.9)

Sono i numeri che i test devono riprodurre — **e sono anche la definizione di "corretto"** per questo task:

| Scenario | Atteso |
|---|---|
| Fotolitico isolato, `light ≈ 0.7`, `env_fit ≈ 1` | `gain ≈ 1.4`, netto **`≈ +0.9`/tick** → cresce |
| Stesso, con **7** vicini occupati | `1.4 − 0.5 − 0.15·7` = netto **`≈ −0.15`/tick** → si ferma (carrying capacity) |
| Fotolitico in zona buia, `light = 0.2` | `gain = 0.4 < upkeep 0.5` → **non sopravvive** (nicchia di luce) |

*(Il GDD cita "6–8 vicini → ≈ −0.15": il valore esatto `−0.15` si ottiene con 7 vicini. Con 8 il netto è `−0.3`, con 6 è `0.0`.)*

### Perché il doppio buffer

GDD §5.6 lascia aperta la scelta dell'ordine di elaborazione. `TECH_DESIGN.md` §6 la chiude sul **doppio buffer**: si legge dallo snapshot immutabile del tick precedente e si scrive nel buffer successivo. Nessuna guardia "nato/agito in questo tick" da mantenere, nessuna dipendenza dall'ordine di visita, i neonati non agiscono nello stesso tick **per costruzione**.

---

## 🔨 Implementazione Suggerita

1. **Struttura**

   ```rust
   /// Advances the simulation by one tick (GDD 5.6).
   /// Pure: no Bevy App required, so determinism and balance can be tested headless.
   pub fn step(world: &mut SimWorld, config: &SimConfig) {
       // 1. snapshot = read side, next = write side
       // 2. for each occupied cell in index order: energy update, death, reproduction
       // 3. decay residues
       // 4. swap buffers, world.tick += 1
   }
   ```

2. **Fitness ambientale**

   ```rust
   /// Gaussian environmental fitness around the species' thermal optimum (GDD 5.9).
   fn env_fit(temperature: f32, optimum: f32, tolerance: f32) -> f32 {
       let d = temperature - optimum;
       (-(d * d) / (2.0 * tolerance * tolerance)).exp()
   }
   ```

3. **Vicini occupati** — leggerli **dallo snapshot**, non dal buffer in scrittura: è ciò che rende il tick indipendente dall'ordine.

4. **Riproduzione.** Raccogliere i vicini vuoti *nello snapshot*, sceglierne uno con l'RNG del mondo, poi **verificare che sia ancora libero nel buffer in scrittura**. Se un altro genitore l'ha già occupato in questo tick, la nascita fallisce (il genitore conserva l'energia). Scandendo le celle in ordine di indice crescente, l'esito è deterministico.

   ```rust
   // Collect empty Moore neighbours from the snapshot, in index order.
   // Order matters: it is what makes the RNG draw reproducible.
   ```

5. **Morte.** `energy <= 0.0` → svuotare la cella e aggiungere `residue_on_death` (`3.0`) al residuo della cella. I residui decadono di `residue_decay` (`0.2`) per tick e vanno tenuti `>= 0`. Non nutrono nessuno in Fase 0 (i decompositori sono Fase 1), ma vanno già accumulati: il task 006 li rende visibili.

6. **Ordine di scansione.** Iterare `for idx in 0..cells.len()`. Il GDD §5.6 menziona un'iterazione mescolata come alternativa: **non usarla**, il doppio buffer la rende superflua e reintrodurrebbe una dipendenza dall'ordine.

7. **`SimPlugin`** — un sistema che chiama `step`:

   ```rust
   fn advance_tick(mut world: ResMut<SimWorld>, config: Res<SimConfig>) {
       step(&mut world, &config);
   }
   ```

   Registrarlo in `FixedUpdate` dentro `SimSet::Advance`. **Le run condition per stato sono il task 007**: qui va bene che giri sempre, o che sia registrato ma inattivo.

8. **Test** in `src/sim.rs` — costruire un mondo, forzare `light`/`temperature` sulle celle interessate, piazzare un organismo e verificare il delta di energia dopo un `step` sui tre scenari della tabella sopra. Usare una tolleranza (`(a - b).abs() < 1e-4`), non `==`, sui `f32`.

---

## ⚠️ Vincoli e Attenzioni

- **Invariante 1**: nessun RNG all'infuori di quello di `SimWorld`. La scelta della cella di nascita è l'unico punto casuale della Fase 0 — è lì che il determinismo si rompe più facilmente.
- **Invariante 2**: `src/sim.rs` non importa `bevy::render` né `bevy_egui`. La funzione `step` non deve toccare alcun tipo Bevy.
- **Invariante 3**: nessun numero magico. Ogni coefficiente da `SimConfig`.
- **Invariante 4**: l'effetto di adiacenza sarà **additivo e lineare** (Fase 1). Anche se qui vale 0, lasciare il punto d'innesto come somma, non come prodotto.
- **Non implementare** predazione, decomposizione, matrice, mutazione: sono la Fase 1. Il `match` sul metabolismo può avere i rami non fotolitici con `todo!()` o guadagno nullo — ma documentato.
- Attenzione ai **residui a energia negativa**: se `energy` scende sotto zero prima della morte, il residuo resta `residue_on_death` fisso, non `residue + energy`.

---

## 🔗 Dipendenze

- **Dipende da**: 004
- **Blocca**: 007, 009

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/005-tick-algorithm.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
