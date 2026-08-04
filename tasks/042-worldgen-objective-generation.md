# Task 042 — Worldgen: per-world objective generation

> **ID**: `042`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Fase 3 planning session

---

## 🎯 Objective

GDD §9: ogni mondo generato proceduralmente include anche il suo obiettivo. Questo task collega il worldgen (task 038, ambiente/tag) col tipo `Objective` (task 040) per scegliere, dato un `world_seed`, quale obiettivo assegnare a un mondo e con quale severità — crescente con `WorldParams.objective_severity` (task 037).

---

## 📋 Acceptance Criteria

- [ ] Funzione in `src/worldgen.rs` che, dato `world_seed`, `WorldParams`, ed eventualmente il mondo generato dal task 038/039 (tag attivi, ambiente, specie disponibili), sceglie una variante di `Objective` (task 040) e ne parametrizza i valori (es. `min_species`, `ticks`, soglie).
- [ ] La scelta è deterministica dal `world_seed` (stesso seed → stesso obiettivo).
- [ ] La severità cresce coerentemente con `world_index`/`WorldParams.objective_severity` (es. soglie più alte, tick richiesti più lunghi nei mondi tardivi) — non un obiettivo scelto a caso indipendentemente dalla difficoltà.
- [ ] Verifica minima di coerenza: l'obiettivo generato non è palesemente irraggiungibile dato l'ambiente/le specie generate per lo stesso mondo (es. non richiedere sopravvivenza nella zona tossica se il mondo non ha una zona tossica di dimensione non-zero; non richiedere ≥N specie coesistenti se il pool di specie disponibili per quel mondo è < N). Non è richiesto un solver completo di raggiungibilità — solo un controllo di buon senso sui vincoli più ovvi.
- [ ] `cargo clippy -- -D warnings` pulito, `cargo test` verde.
- [ ] Test: determinismo, crescita della severità con `world_index`, assenza di obiettivi palesemente incoerenti sui casi limite testati.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/worldgen.rs` | Funzione di generazione obiettivo, integrata nel flusso di generazione mondo del task 038. |

---

## 🧩 Technical Context

Vedi task 040 per il tipo `Objective` e le sue varianti (`Coexistence`, `SurviveIn`, `TriggerBloom`), e task 037 per `WorldParams.objective_severity`.

- **Comportamento attuale**: nessun obiettivo esiste per nessun mondo — il task 040 introduce il tipo e il motore di valutazione ma non la generazione procedurale.
- **Comportamento desiderato**: ogni mondo generato ha un obiettivo assegnato, coerente col proprio ambiente/tag/specie e con la severità attesa per il suo `world_index`.

---

## 🔨 Suggested Implementation

1. Decidere una strategia di selezione semplice: es. campionare una variante di `Objective` dall'RNG del mondo tra quelle definite nel task 040, poi parametrizzarla scalando i valori base per `objective_severity`.
2. Per la verifica di coerenza minima, controllare solo i vincoli più diretti (es. se `SurviveIn { zone: Toxic, .. }` è scelto, verificare che `WorldParams`/l'ambiente generato abbia effettivamente una zona tossica di area non-zero; se `Coexistence { min_species, .. }` è scelto, verificare che `min_species` non superi il numero di specie disponibili generate dal task 039).
3. Integrare la chiamata nel flusso di generazione mondo esistente (dove il task 038 orchestra tag/matrice/ambiente).

---

## ⚠️ Constraints and Caveats

- **Determinismo**: solo RNG interno del mondo.
- **Non un solver completo**: la verifica di raggiungibilità è un controllo di buon senso sui vincoli più ovvi, non una dimostrazione formale che l'obiettivo sia sempre risolvibile (la difficoltà/imprevedibilità dell'emergenza è parte del design, GDD §14).
- **Non introdurre bonus objectives**: esplicitamente fuori dall'MVP (GDD §8: "planned in principle... but after the clean primary-objective→advance core. Not in the minimal MVP").

---

## 🔗 Dependencies

- **Depends on**: 038 (ambiente/tag/matrice del mondo), 040 (tipo `Objective`).
- **Blocks**: 045 (la transizione di mondo applica l'obiettivo generato al nuovo mondo).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/042-worldgen-objective-generation.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
