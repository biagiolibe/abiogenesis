# Task 004 — Ambiente: gradienti statici

> **ID**: `004`
> **Categoria**: Feature
> **Priorità**: 🔴 P1
> **Stima**: ~1h
> **Assegnato a**: non assegnato
> **Sessione**: —

---

## 🎯 Obiettivo

Popolare le scalari ambientali di ogni cella con i **gradienti statici** della Fase 0, così da creare **eterogeneità spaziale → nicchie**.

Non è decorazione: è una delle tre leve anti-degenerazione del GDD §5.8. Senza eterogeneità ambientale il sistema collassa nei due esiti noiosi ("muore tutto" / "una specie domina"), che il GDD indica come rischio numero uno del progetto.

---

## 📋 Acceptance Criteria

- [ ] Ogni cella ha `temperature`, `light`, `toxicity` in `[0,1]`.
- [ ] **Luce**: `0.9` sulla riga più alta → `0.2` sulla più bassa, interpolata linearmente.
- [ ] **Temperatura**: `0.2` sulla colonna più a sinistra → `0.8` sulla più a destra, interpolata linearmente.
- [ ] **Tossicità**: `0.7` in una zona definita, `0.0` altrove.
- [ ] I valori agli estremi della griglia corrispondono esattamente alla tabella di GDD §5.9 (coperto da test).
- [ ] La generazione è deterministica: stesso seed ⇒ stesso ambiente.
- [ ] Esiste il punto d'innesto per la diffusione (Fase 1+), non implementata.
- [ ] `cargo clippy -- -D warnings` pulito.

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `src/world.rs` | Generazione dell'ambiente, dentro o accanto a `SimWorld::new` |
| `src/config.rs` | Valori dei gradienti (task 002) |

---

## 🧩 Contesto Tecnico

- **Comportamento attuale**: `SimWorld` alloca la griglia (task 003), ma le scalari ambientali sono tutte a zero.
- **Comportamento desiderato**: griglia con gradienti che creano nicchie spaziali.

### GDD §5.2 — Strato ambientale

> **Fase 0:** gradienti statici (es. luce alta in alto, temperatura su un asse diverso) per creare eterogeneità spaziale → nicchie.
> **Fase 1+:** diffusione lenta delle scalari (media coi vicini a rate basso), così gli interventi ambientali si propagano nel tempo.

I due gradienti sono **su assi diversi di proposito**: luce verticale, temperatura orizzontale. È questo incrocio a generare nicchie bidimensionali distinte — una specie fotolitica amante del freddo prospera in alto a sinistra, una amante del caldo in alto a destra.

### Perché la fascia buia conta

Con `metabolism_gain = 2.0` e `upkeep = 0.5` (GDD §5.9), un fotolitico con `env_fit ≈ 1`:
- a `light = 0.7` guadagna `1.4` → netto `+0.9`/tick, **cresce**;
- a `light = 0.2` guadagna `0.4` < `0.5` di upkeep → **non sopravvive**.

La soglia di sopravvivenza sta attorno a `light = 0.25`. Le righe basse devono quindi restare sotto quel valore: sono la **nicchia di luce**, verificata dal task 009.

---

## 🔨 Implementazione Suggerita

1. Interpolazione lineare sull'asse corrispondente. Attenzione al caso `height == 1` (divisione per zero) anche se non si presenta a `48×32`:

   ```rust
   /// Static Phase 0 gradients: light falls top→bottom, temperature rises left→right.
   /// The two axes differ on purpose: their crossing is what creates 2D niches (GDD 5.2).
   fn apply_gradients(&mut self, config: &SimConfig) {
       let env = &config.environment;
       for y in 0..self.height {
           let ty = y as f32 / (self.height - 1).max(1) as f32;
           for x in 0..self.width {
               let tx = x as f32 / (self.width - 1).max(1) as f32;
               let cell = &mut self.cells[y * self.width + x];
               cell.light = lerp(env.light_top, env.light_bottom, ty);
               cell.temperature = lerp(env.temp_left, env.temp_right, tx);
               cell.toxicity = 0.0;
           }
       }
   }
   ```

2. **Zona tossica.** In Fase 0 basta una zona fissa e leggibile — un rettangolo in un angolo, dimensione da `SimConfig`. La generazione procedurale delle zone estreme è Fase 3 (GDD §9). Metterla lontano dalla fascia luminosa più fertile, così da non falsare i test del task 009.

3. **Punto d'innesto per la diffusione** — dichiararlo senza implementarlo, così la Fase 1 sa dove innestarsi:

   ```rust
   /// Phase 1+: blend each scalar toward its neighbours' mean at `diffusion_rate`
   /// per tick (GDD 5.2). Not active in Phase 0: gradients are static.
   fn diffuse_environment(&mut self, _config: &SimConfig) {
       // Intentionally empty in Phase 0.
   }
   ```

4. **Test**: `light` sulla riga 0 vale `0.9` e sull'ultima `0.2`; `temperature` sulla colonna 0 vale `0.2` e sull'ultima `0.8`; tutte le scalari restano in `[0,1]`; le celle della zona tossica valgono `0.7` e le altre `0.0`.

---

## ⚠️ Vincoli e Attenzioni

- **Tutte le scalari devono restare in `[0,1]`** (GDD §5.2): è l'assunzione su cui poggiano le formule del tick.
- I valori vengono da `SimConfig`, **non scritti a mano qui** (invariante 3).
- **Nessuna diffusione in Fase 0**: i gradienti sono statici. Implementarla ora renderebbe l'ambiente un bersaglio mobile proprio mentre si tara il tick.
- La zona tossica non ha ancora alcun effetto sulla simulazione: nessun metabolismo la legge in Fase 0. È corretto — serve a rendere visibile la struttura dell'ambiente e sarà usata dalla Fase 1 in poi.

---

## 🔗 Dipendenze

- **Dipende da**: 003
- **Blocca**: 005

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/004-environment-gradients.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
