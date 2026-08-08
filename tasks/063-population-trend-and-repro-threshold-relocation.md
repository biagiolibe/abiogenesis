# Task 063 — Population trend indicator, repro-threshold relocation, per-era birth log

> **ID**: `063`
> **Category**: Feature / UI data
> **Priority**: 🟡 P2
> **Estimate**: ~3-4h
> **Assigned to**: unassigned
> **Session**: 2026-08-08 (from `redesign/abiogenesis-sidebar-redesign.md`, point 7 — a data-correctness concern raised during sidebar design, prerequisite for task 064's visual redesign)

---

## 🎯 Objective

`redesign/abiogenesis-sidebar-redesign.md` (§7) identifies a real misleading-UI problem in the current HUD: the Population panel's `population_line` (`src/text.rs:187-194`, wired in `src/ui.rs:305-315`) shows **average energy across the whole species population** next to `repro_threshold` — a **per-individual** reproduction trait (GDD §5.3/§5.9). A population average of 7.44 against a threshold of 10.0 doesn't mean "nobody's reproduced yet": the population could contain individuals well above threshold and others well below, invisible in the average. This task fixes the data model before task 064 reskins the panel that displays it:

1. **`repro_threshold` moves to the notebook catalog** (`src/notebook.rs::catalog_panel`, `src/text.rs::species_catalog_line`) — a static per-species trait, alongside metabolism and temperature range, where it belongs. It's removed from the HUD Population line entirely.
2. **The HUD's average-energy figure becomes a qualitative trend indicator** (rising / falling / stable vs. the *previous era*), not a raw number compared against a threshold it was never meaningfully comparable to.
3. **Actual births become a log event**, the real signal for "individuals crossed the threshold" — not something to infer from an average.

---

## 📋 Acceptance Criteria

### Repro-threshold relocation

- [ ] `text::species_catalog_line` (`src/text.rs:376-386`) gains a `repro_threshold: f32` parameter and includes it in the formatted line (e.g. appended as `· repro ≥{repro_threshold:.1}`).
- [ ] `notebook.rs::catalog_panel` (`src/notebook.rs:592-617`) passes `config.energy.repro_threshold` through.
- [ ] `text::population_line` (`src/text.rs:187-194`) drops the `repro_threshold` parameter entirely — signature becomes `(species_label, population, avg_energy)` plus whatever the trend indicator needs (see below). Update `src/ui.rs:308-313`'s call site.
- [ ] `player_guide.md`'s species-legibility section (task 057's addition) updated: reproduction threshold is documented as living in the notebook catalog, not the Population panel.

### Per-era population trend

- [ ] New resource tracking each species' average energy **as of the last completed era** — e.g. `PopulationTrendHistory(Vec<f32>)` indexed like `TagMatrix`/`MatrixKnowledge` (by `SpeciesId`), or a small struct resource in `ui.rs` or a new `src/population_trend.rs` module (pick whichever keeps `ui.rs`'s existing read-only-against-`SimWorld` discipline, TECH_DESIGN.md §3.3).
- [ ] A system consuming `EraCompleted` (`src/sim.rs:41-48`, already fired by `advance_tick`/`single_tick`, see `src/sim.rs:475-482`) that: computes current per-species average energy (reuse or extract `ui.rs::species_stats`'s aggregation, `src/ui.rs:696-717`), compares it against the stored previous-era snapshot per species, classifies each as `Rising` / `Falling` / `Stable` using a **named threshold in `SimConfig`** (no magic numbers — e.g. `EnergyConfig::trend_epsilon` or a new small config struct), then overwrites the snapshot with the current value for next era's comparison.
- [ ] A species with no previous-era snapshot yet (first era it appears in, or first era of the world) reads as `Stable` — there's nothing to compare against, and `Stable` is the least presumptuous default.
- [ ] `text::population_line` (or its replacement) takes a `PopulationTrend` (or equivalent enum: `Rising`/`Falling`/`Stable`) instead of `repro_threshold`, and `ui.rs`'s HUD rendering shows a small directional indicator (▲/▼/▬, matching the redesign mockup's convention: green rising, red falling, gray stable) instead of the old `avg_energy/repro_threshold` fraction. Keep the raw average-energy number in the line too (mockup keeps `7.44`) — only the threshold comparison goes away, not the number itself.
- [ ] Unit tests for the trend classification function (pure, given `previous: f32, current: f32, epsilon: f32 -> Trend`) covering: clearly rising, clearly falling, within-epsilon (stable), and the "no previous snapshot" case.

### Per-era birth log

- [ ] New message type mirroring `OrganismDied`/`SpeciesExtinct`'s pattern (`src/sim.rs:14-39`) — e.g. `OrganismBorn { species: SpeciesId }` — emitted at the reproduction site in `advance_tick` (`src/sim.rs:341-355`, where a child organism is currently spawned with no corresponding event).
- [ ] A system (`notebook.rs`, alongside `record_events`/`accumulate_evidence`) tallies `OrganismBorn` events per species within the current era, and on `EraCompleted` pushes one `LogEntry` per species with at least one birth that era — e.g. "Kael: +3 births this era" via a new `text.rs` helper — then resets the tally for the next era. Zero-birth species get no line (keeps the log curated, matching `record_events`'s existing filtering philosophy, `src/notebook.rs:237-239` — this is the opposite case from task 061's per-observation logging, which deliberately went verbose; births stay a curated once-per-era summary).
- [ ] Test coverage: multiple births for the same species in one era tally to a single correct count; births reset to zero after the era-completion log entry fires (no carry-over into the next era's tally).

### General

- [ ] `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt -- --check` all clean.
- [ ] `abiogenesis-gdd.md` untouched (this is presentation/legibility, not a mechanics change) — verify no formulas moved.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/text.rs` | `population_line` (loses `repro_threshold`, gains trend), `species_catalog_line` (gains `repro_threshold`), new birth-log line helper. |
| `src/ui.rs` | `hud_panel`'s Population group (`~305-315`), `species_stats` (`696-717`, aggregation to reuse for the era-snapshot system). |
| `src/notebook.rs` | `catalog_panel` (`592-617`, add `repro_threshold`), new birth-tally system alongside `record_events`/`accumulate_evidence`. |
| `src/sim.rs` | `EraCompleted` (`41-48`), reproduction site (`341-355`, new `OrganismBorn` message), event registration (`app.add_message::<...>()`, `447`/`653`). |
| `src/config.rs` | `EnergyConfig` — new trend-epsilon constant. |
| `player_guide.md` | Species-legibility section (task 057). |
| `redesign/abiogenesis-sidebar-redesign.md` | Source design doc, §7. |

---

## 🧩 Technical Context

- **Current behavior**: `population_line` shows `avg_energy/repro_threshold` as if the threshold were a ceiling for the average — it isn't; it's an individual-level trigger. No signal exists anywhere for "how many individuals reproduced this era."
- **Desired behavior**: `repro_threshold` is a static per-species catalog fact; the HUD's per-era number is explicitly a *trend*, not a threshold comparison; actual reproductions surface as log events, matching how deaths/extinctions already work (`OrganismDied`/`SpeciesExtinct`).
- `species_stats` (`src/ui.rs:696-717`) already computes per-species population and average energy every frame from `SimWorld` — the new era-snapshot system needs the same aggregation, but only captured once per era (on `EraCompleted`), not every frame. Consider whether to call `species_stats` directly (if made `pub(crate)` or moved) or duplicate the small aggregation loop in the new system — prefer reuse if it doesn't tangle module boundaries.

---

## ⚠️ Constraints and Caveats

- **No magic numbers**: the trend-classification epsilon belongs in `SimConfig`.
- **Style**: all new player-facing strings go through `text.rs` (task 034's convention).
- **Module boundaries**: keep the `Ui` write-only-intents / read-only-`SimWorld` discipline (TECH_DESIGN.md §3.3) — the era-snapshot and birth-tally resources are presentation-side bookkeeping (same category as `PlayerPlacedCells`, `NotebookHasUnseenConfirmation`), not simulation state; don't put them in `sim.rs`/`world.rs`.
- **Determinism**: `OrganismBorn` must be emitted from the same deterministic tick path as `OrganismDied` — no new RNG surface, just a new message alongside an already-deterministic spawn decision.
- **Don't** touch `repro_threshold`'s actual value or the reproduction mechanic itself — this task is entirely about how existing data is *displayed and logged*, not simulation behavior.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: 064 (sidebar console redesign) — it renders the trend indicator and relies on `repro_threshold` already being out of the HUD's data path.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/063-population-trend-and-repro-threshold-relocation.md)"$'\n\nExecute this task in the current project.'
```
