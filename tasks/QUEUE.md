# Task Execution Queue

Questa è la coda di esecuzione operativa. I task sono ordinati per priorità.

## Come usare questa coda

- **Esecuzione**: Prendi il primo task `[ ]` disponibile.
- **Aggiornamento**: Cambia `[ ]` in `[/]` quando inizi e in `[x]` quando finisci.
- **Archiviazione**: A task completato, sposta il file in `tasks/done/`.

## Priorità

| Codice | Significato |
|--------|-------------|
| 🔴 P1  | Bloccante / Critico |
| 🟡 P2  | Feature importante |
| 🟢 P3  | Ottimizzazione / Polish |

---

## 🤖 Come delegare un task a Claude CLI

```bash
claude "$(cat tasks/NNN-nome.md)"$'\n\nEsegui questo task nel progetto corrente.'
```

---

## 🏃 Coda Attiva

**Fase 0 — Scheletro camminante.** Traguardo: una specie fotolitica fiorisce e si stabilizza grazie alla carrying capacity (GDD §13).

| Stato | ID | Titolo | Priorità | Dipende da | Agente | Task File |
|-------|----|--------|----------|------------|--------|-----------|
| `[ ]` | 002 | `SimConfig`: coefficienti centralizzati | 🔴 P1 | 001 | — | [002](002-sim-config.md) |
| `[ ]` | 003 | Tipi di dominio e resource `SimWorld` | 🔴 P1 | 002 | — | [003](003-domain-simworld.md) |
| `[ ]` | 004 | Ambiente: gradienti statici | 🔴 P1 | 003 | — | [004](004-environment-gradients.md) |
| `[ ]` | 005 | Algoritmo del tick (Fase 0), puro e headless | 🔴 P1 | 004 | — | [005](005-tick-algorithm.md) |
| `[ ]` | 006 | Resa della griglia a sprite + camera 2D | 🟡 P2 | 003 | — | [006](006-grid-rendering.md) |
| `[ ]` | 007 | `GameState`/`EraState`, input, era animata | 🟡 P2 | 005, 006 | — | [007](007-states-input-era.md) |
| `[ ]` | 008 | HUD `bevy_egui` | 🟡 P2 | 007 | — | [008](008-hud-egui.md) |
| `[ ]` | 009 | Test di determinismo e validazione carrying capacity | 🟡 P2 | 005 | — | [009](009-determinism-balance-tests.md) |

**Cancello di uscita della Fase 0:** non si passa alla Fase 1 con i test del 009 rossi.

Le fasi successive vivono come backlog in [`PROJECT_PLAN.md`](../PROJECT_PLAN.md) e si espandono in task file quando ci si arriva.

---

## 🧪 Task Rapidi (Senza File)

Task che richiedono < 15 min e non necessitano di briefing dettagliato.

| Stato | Descrizione | Priorità |
|-------|-------------|----------|
| `[ ]` | *(nessuno)* | — |

---

## ✅ Archiviati (Completati)

| Stato | ID | Titolo | Agente | File |
|-------|----|--------|--------|------|
| `[x]` | 001 | Toolchain, scaffold Cargo e app Bevy a plugin | Claude | [001](done/001-scaffold-bevy.md) |

---

*Ultimo aggiornamento: 2026-08-01*
