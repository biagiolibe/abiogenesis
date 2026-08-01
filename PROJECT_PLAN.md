# Project Plan — Abiogenesis

Questo documento traccia l'evoluzione del progetto dalle idee alla realizzazione.

**Visione.** Sei uno xenobiologo che semina la vita su mondi alieni con **regole biochimiche nascoste e diverse a ogni run**. Il gioco è farne reverse-engineering: semini, osservi un ecosistema che vive di vita propria, formuli ipotesi, le verifichi con esperimenti mirati. Il piacere è il doppio mistero — cosa succederà, e quali sono le regole. Design completo in [`abiogenesis-gdd.md`](abiogenesis-gdd.md); architettura in [`TECH_DESIGN.md`](TECH_DESIGN.md).

## Ciclo di Vita dei Task

```
PROPOSTE  →  (revisione)  →  BACKLOG  →  (sviluppo)  →  COMPLETATI
```

| Simbolo | Significato |
|---------|-------------|
| `[ ]`   | Task approvato nel backlog |
| `[/]`   | Task in lavorazione |
| `[x]`   | Task completato |
| `[-]`   | Task annullato / scartato |
| `[?]`   | Proposta (in attesa di valutazione) |

---

## 🗂️ SEZIONE 1 — PROPOSTE

> Idee da discutere prima di essere spostate nel backlog operativo.

### Questioni aperte dal GDD (§14)

- `[?]` **Obiettivi bonus** che danno currency di meta-progressione — previsti in linea di principio, ma **dopo** il core pulito "obiettivo primario → avanzi" (GDD §8).
- `[?]` **Persistenza della meta-progressione** (profilo/save degli sblocchi) — deliberatamente rimandata: si decide solo dopo aver verificato che il loop è divertente (GDD §10).
- `[?]` **Metabolismi aggiuntivi** oltre ai tre di base, es. chemiolitotrofo legato alla `toxicity`, come contenuto sbloccabile (GDD §5.4).
- `[?]` **Titolo definitivo** — "Abiogenesis" è un segnaposto (GDD §14).

### Nate dal passaggio a Bevy (v0.4)

- `[?]` **Config in RON con hot-reload** via `bevy_asset` — il GDD §5.6 chiede coefficienti "idealmente ricaricabili a caldo"; con Bevy costa poco. Da fare in fase di tuning, quando serve davvero.
- `[?]` **Zoom e pan della camera** — utile su griglie più grandi di 48×32.
- `[?]` **Modalità real-time** come opzione — il GDD §4 la dava per "costa poco aggiungerla in seguito"; con Bevy è quasi gratis (basta non fermarsi a fine era).
- `[?]` **Main menu reale** con scelta e condivisione del seed — il determinismo (GDD §5.7) rende i seed interessanti condivisibili.

---

## 🔵 SEZIONE 2 — BACKLOG (Operativo)

> Task approvati. La Fase 0 è già espansa in task file; le fasi successive si espandono quando ci si arriva.

### 🏗️ Fase 0 — Scheletro camminante

**Traguardo:** guardi una specie fotolitica fiorire e stabilizzarsi grazie alla carrying capacity (GDD §13).

- `[ ]` 001 — Toolchain, scaffold Cargo e app Bevy a plugin → [001](tasks/001-scaffold-bevy.md)
- `[ ]` 002 — `SimConfig`: coefficienti centralizzati → [002](tasks/002-sim-config.md)
- `[ ]` 003 — Tipi di dominio e resource `SimWorld` → [003](tasks/003-domain-simworld.md)
- `[ ]` 004 — Ambiente: gradienti statici → [004](tasks/004-environment-gradients.md)
- `[ ]` 005 — Algoritmo del tick (Fase 0), puro e headless → [005](tasks/005-tick-algorithm.md)
- `[ ]` 006 — Resa della griglia a sprite + camera 2D → [006](tasks/006-grid-rendering.md)
- `[ ]` 007 — `GameState`/`EraState`, input, era animata → [007](tasks/007-states-input-era.md)
- `[ ]` 008 — HUD `bevy_egui` → [008](tasks/008-hud-egui.md)
- `[ ]` 009 — Test di determinismo e validazione carrying capacity → [009](tasks/009-determinism-balance-tests.md)

### ⚙️ Fase 1 — Emergenza

**Traguardo:** appare l'emergenza vera; più specie interagiscono via matrice (GDD §13).

- `[ ]` Tag biochimici: pool globale di 10 glifi, sottoinsieme attivo per mondo (GDD §5.5)
- `[ ]` Generazione della **matrice nascosta** `tag × tag`, asimmetrica, densità ~40% (GDD §5.5)
- `[ ]` **Vincolo di ciclicità**: garantire ≥1 ciclo RPS negativo in generazione (GDD §5.8) — è la leva anti-degenerazione principale
- `[ ]` Effetto di adiacenza nel tick: additivo e lineare (GDD §5.6, passo 3)
- `[ ]` Specie multiple e palette di partenza per mondo
- `[ ]` Metabolismo **predatore**: preleva energia dai vicini entro `drain_cap` (GDD §5.4)
- `[ ]` Metabolismo **decompositore** e ciclo dei residui (GDD §5.4)
- `[ ]` Diffusione lenta delle scalari ambientali (GDD §5.2, Fase 1+)
- `[ ]` Azione **semina** con selezione della cella col mouse (GDD §6)

### 🎨 Fase 2 — Deduzione

**Traguardo:** nasce il *gioco* di deduzione, non solo la simulazione (GDD §13).

- `[ ]` Taccuino: finestra egui con log, griglia di ipotesi `tag × tag`, catalogo (GDD §7)
- `[ ]` Log delle osservazioni costruito consumando gli eventi di simulazione (`TECH_DESIGN.md` §4)
- `[ ]` **Modello di conferma** "B con sfumatura di C": evidenza pesata `1/(1+n_confonditori)`, soglia `3.0` (GDD §7)
- `[ ]` Azioni **stress / cull / splice** (GDD §6)
- `[ ]` Budget azioni per era: 3 punti, costi differenziati; stato `EraState::Planning` diventa reale (GDD §6)

### 🏁 Fase 3 — La run

**Traguardo:** un ciclo di gioco completo, mondo dopo mondo (GDD §13).

- `[ ]` Sistema di **obiettivi** per mondo e verifica del soddisfacimento (GDD §8)
- `[ ]` **Generazione procedurale dei mondi**: matrice, ambiente, tag attivi, specie, obiettivi (GDD §9)
- `[ ]` Condizioni di fallimento: estinzione totale + budget di ere finito (GDD §8)
- `[ ]` **Curva di difficoltà**: 5 → ~8 tag attivi, ambienti più ostili, budget più corto (GDD §9)
- `[ ]` Flusso della run: main menu, vittoria, sconfitta, passaggio al mondo successivo
- `[ ]` Meta-progressione minima, senza persistenza (GDD §10)

### 🎚️ Tuning finale — *l'arte vera*

**Obiettivo:** emergenza *interessante e leggibile*, evitando "muore tutto" e "uno domina" (GDD §13, §14).

- `[ ]` Taratura delle tre manopole anti-degenerazione: ciclicità, eterogeneità ambientale, carrying capacity (GDD §5.8)
- `[ ]` Taratura dei coefficienti del tick e della soglia di conferma del taccuino (GDD §5.6, §5.9, §7)
- `[ ]` Dimensione definitiva della griglia (resta empirica, GDD §5.1)
- `[ ]` Migrazione della config a RON con hot-reload, per accorciare il ciclo di taratura

---

## 🟡 SEZIONE 3 — IN CORSO

> Task attualmente assegnati ad agenti o in sviluppo manuale.

- *(nessuno al momento)*

---

## ✅ SEZIONE 4 — COMPLETATI

### Milestones

- `[x]` Definizione del concept iniziale — GDD v0.3, decisioni di design chiuse con baseline numerica ed esempio di partita
- `[x]` Scelta dello stack: Rust + Bevy (ECS), finestra 2D, UI egui — GDD v0.4
- `[x]` Bootstrap Meridian dal GDD: `TECH_DESIGN.md`, backlog, coda operativa, task file della Fase 0

---

*Ultimo aggiornamento: 2026-08-01*
