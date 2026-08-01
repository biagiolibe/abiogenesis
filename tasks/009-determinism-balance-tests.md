# Task 009 — Test di determinismo e validazione carrying capacity

> **ID**: `009`
> **Categoria**: Test
> **Priorità**: 🟡 P2
> **Stima**: ~1.5h
> **Assegnato a**: non assegnato
> **Sessione**: —

---

## 🎯 Obiettivo

Dimostrare con test automatici che la simulazione **è deterministica** e che **non degenera**: una fioritura fotolitica cresce e poi si stabilizza, invece di esplodere o estinguersi.

Insieme al task 007, **questo task è il traguardo della Fase 0** (GDD §13): *"guardi una specie fotolitica fiorire e stabilizzarsi grazie alla carrying capacity"*. Qui quel traguardo smette di essere un'impressione visiva e diventa una proprietà verificata.

Questi test sono anche la **rete di sicurezza del tuning**: il GDD §13 e §14 indicano il bilanciamento dell'emergenza come il lavoro più delicato del progetto, e senza un modo automatico di sapere se un ritocco ha rotto qualcosa, quel lavoro procede alla cieca.

---

## 📋 Acceptance Criteria

- [ ] I test stanno in `tests/` e girano **headless, senza costruire una `App` Bevy**.
- [ ] **Determinismo**: due run di 200 tick con lo stesso seed producono stato finale identico (griglia cella per cella, non solo la popolazione).
- [ ] **Sensibilità al seed**: seed diversi producono stato finale diverso (esclude il caso banale in cui l'RNG non viene mai usato).
- [ ] **Carrying capacity**: da un singolo seme in zona luminosa, la popolazione cresce, poi si stabilizza entro una banda e vi resta.
- [ ] **Nicchia di luce**: dopo N tick, nessun organismo sopravvive nelle righe con `light < 0.25`.
- [ ] **Non-estinzione**: la popolazione non arriva mai a zero nello scenario nominale.
- [ ] `cargo test` passa; `cargo clippy -- -D warnings` pulito.

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `tests/determinism.rs` | Test di riproducibilità |
| `tests/balance.rs` | Carrying capacity, nicchie, non-estinzione |
| `src/lib.rs` | **Potrebbe servire crearlo** — vedi note |

---

## 🧩 Contesto Tecnico

- **Comportamento attuale**: la simulazione funziona e si vede girare, ma nulla ne verifica automaticamente le proprietà.
- **Comportamento desiderato**: `cargo test` conferma determinismo e stabilità.

### GDD §5.7 — Determinismo

> La simulazione è **deterministica** a parità di seed: RNG seedato conservato nello stato del mondo. Fondamentale per debug dell'emergenza, riproducibilità dei bug e (in prospettiva) condivisione di seed interessanti.

### GDD §5.8 — Anti-degenerazione

> Il rischio numero uno dell'emergenza è il collasso in due esiti noiosi: **"muore tutto"** oppure **"una specie domina"**.

In Fase 0 c'è una sola specie, quindi delle tre leve del GDD §5.8 solo due sono attive e verificabili: **carrying capacity** (penalità da affollamento) ed **eterogeneità ambientale** (le nicchie). Il vincolo di ciclicità sulla matrice è Fase 1.

### I numeri di riferimento (GDD §5.9)

| Scenario | Atteso |
|---|---|
| Fotolitico isolato, `light ≈ 0.7` | netto `≈ +0.9`/tick → cresce |
| Con 7 vicini occupati | netto `≈ −0.15`/tick → si ferma |
| A `light = 0.2` | `gain 0.4 < upkeep 0.5` → non sopravvive |

La soglia di sopravvivenza sta attorno a `light = 0.25`: `light · 2.0 · env_fit = 0.5`.

---

## 🔨 Implementazione Suggerita

1. **Rendere il dominio raggiungibile dai test.** I test di integrazione in `tests/` importano dal crate come utente esterno, quindi serve una `src/lib.rs` che esporti `config`, `world` e `sim`, con `main.rs` che ne diventa un consumatore. È il momento giusto per farlo: conferma sul campo che la simulazione **è indipendente dalla resa** (invariante 2).

   In alternativa, test unitari `#[cfg(test)]` dentro `src/sim.rs`. La `lib.rs` è preferibile: il confine diventa esplicito e verificato dal compilatore.

2. **Determinismo**

   ```rust
   #[test]
   fn same_seed_yields_identical_history() {
       let cfg = SimConfig::default();
       let mut a = SimWorld::new(42, &cfg);
       let mut b = SimWorld::new(42, &cfg);
       for _ in 0..200 {
           step(&mut a, &cfg);
           step(&mut b, &cfg);
       }
       assert_eq!(snapshot(&a), snapshot(&b));
   }
   ```

   `snapshot` va confrontato **cella per cella** — specie e energia, non solo la popolazione totale: due mondi possono avere la stessa popolazione e configurazioni diverse. Per l'energia, confrontare i bit (`f32::to_bits`) o con tolleranza stretta: la stessa sequenza di operazioni sulla stessa piattaforma produce gli stessi bit, quindi un'uguaglianza esatta è legittima qui e coglie derive che una tolleranza larga nasconderebbe.

3. **Sensibilità al seed** — stesso schema con seed `42` e `43`, `assert_ne!`. Senza questo test, un `step` che non usa mai l'RNG passerebbe il test di determinismo a pieni voti.

4. **Carrying capacity**

   ```rust
   #[test]
   fn bloom_stabilises_instead_of_exploding() {
       // Seed one photolithic organism in the lit band, run long enough to saturate.
       // Population must grow, then settle: sampled over the last 50 ticks it should
       // stay within a narrow band and never hit zero or fill the lit area entirely.
   }
   ```

   Campionare la popolazione ogni N tick e verificare che l'**ampiezza relativa** delle ultime misure resti sotto una soglia (es. 10%), invece di fissare un numero assoluto: la banda esatta dipende dai coefficienti e questi sono dichiarati tarabili (GDD §14). Un test che fissa un valore assoluto va rifatto a ogni ritocco di tuning; uno che verifica la *forma* della curva sopravvive.

   Concedere abbastanza tick (300–500) perché la saturazione avvenga davvero.

5. **Nicchia di luce** — dopo la stabilizzazione, verificare che nelle righe con `light < 0.25` non ci sia alcun organismo vivo. È la prova che l'eterogeneità ambientale sta creando nicchie reali.

6. **Non-estinzione** — nello scenario nominale la popolazione non tocca mai zero.

---

## ⚠️ Vincoli e Attenzioni

- **Nessuna `App` Bevy nei test.** Se per testare la simulazione servisse costruirne una, l'invariante 2 è stata violata e va corretta *quella*, non il test.
- **Nessun test dipendente dall'orologio o dal parallelismo**: `cargo test` esegue i test in thread concorrenti, e ogni residuo di stato globale emergerebbe come flakiness.
- **Non fissare valori assoluti di popolazione** salvo dove il GDD li stabilisce. I coefficienti sono dichiarati tarabili: i test devono verificare *proprietà* (cresce, si stabilizza, non muore, rispetta le nicchie), non numeri.
- Se un test di bilanciamento fallisce, **la prima ipotesi è che siano i coefficienti a essere sbagliati, non il test**: annotare l'esito in `PROJECT_PLAN.md` fra le questioni di tuning invece di aggiustare le soglie fino al verde.
- Con 200–500 tick su griglia 48×32 i test restano nell'ordine dei secondi in release; se in debug fossero lenti, il profilo del task 001 (`opt-level = 3` sulle dipendenze) aiuta, ma per i test conta `cargo test --release`.

---

## 🔗 Dipendenze

- **Dipende da**: 005
- **Blocca**: nessuno — ma è il **cancello di uscita della Fase 0**: non si passa alla Fase 1 con questi test rossi.

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/009-determinism-balance-tests.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
