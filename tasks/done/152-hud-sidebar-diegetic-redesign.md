# Task 152 — HUD and sidebar: auto-advance toggle, mutation-level badge, species subtext

> **ID**: `152`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~1.5h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

**Scope correction against the QUEUE row title.** `tasks/QUEUE.md` names this
task "diegetic labels, notch indicators, narrative directive" from
`abiogenesis-sidebar-redesign.md`. That whole document — hairline-divided
single panel, diegetic English labels (`Moves`/`Biosphere`/`Species`/`This
world wants`), discrete dot/tick indicators replacing progress bars, and the
italic narrative-quote objective line — **already shipped**, task 064
(2026-08-08, archived `tasks/QUEUE_ARCHIVE.md`), corrected by task 065. There
is nothing left to redo from that document; verified directly against
`src/ui.rs`/`src/text.rs` (`hairline`, `dot_row`/`DotShape`, `HEADING_ACTION
= "Moves"`, `HEADING_POPULATION = "Biosphere"`, `HEADING_SEED_PALETTE =
"Species"`, `HEADING_OBJECTIVE = "This world wants"`,
`narrative_quote`/`OBJECTIVE_NARRATIVE_COLOR`).

The actual remaining Phase-2 HUD gap is in the **newer**
`abiogenesis-hud-notebook.md` document (its own "Cosa serve per
l'integrazione" checklist), cross-checked against the current build. Most of
that checklist is *also* already done (Biosphere numeric delta — task 120;
era-relative season/pulse readout — tasks 117/135; notebook
unread-observation dot — task 054's `NotebookHasUnseenConfirmation`; dynamic
genome bank with original/synthesised distinction — explicitly task 147's
own scope, not duplicated here). Three items are genuinely unbuilt:

1. **Continuous-advancement toggle (play/pause).** Today the HUD only offers
   single-pulse (`Tick` button) and single-era step; there is no loop that
   auto-advances pulses, and no visible on/off HUD state for it. GDD §11
   names a `p` keybind for this ("toggles continuous advancement") as part
   of the v0.7 design pass, but it isn't implemented — `input.rs` has no
   `KeyP`/continuous-advance handling today.
2. **Mutation-level badge on the Splice action icon.** The doc calls for a
   small badge on the Splice icon showing which capability tier is active
   (initial: tag-only, imprecise; unlocked: full manual mutation, precise)
   — the badge itself is this task's job, not the unlock condition (the
   design doc explicitly puts that logic out of its own scope, same as this
   task).
3. **Species-row abbreviated subtext.** The Seed-palette species row
   (`ui.rs::species_row`, task 065) shows a metabolism glyph + name today,
   but not the doc's abbreviated `metabolism · temp-preference` subtext
   (e.g. `photo · cold`) that lets the player judge a species' fit without
   opening the notebook.

Design source: `redesign/processed/abiogenesis-hud-notebook.md`, "Decisioni
— HUD" points 2 and 3, and the "Cosa serve per l'integrazione" checklist
items 1 and 2.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] A continuous-advance toggle exists: a `p` keybind (`input.rs`) and a
      HUD button (`ui.rs::time_control_row`) both flip the same state; while
      active, pulses advance automatically at a configurable rate
      (`TimeConfig::era_tick_hz` already exists for era-advance playback —
      decide whether continuous-advance reuses it or needs its own rate; no
      magic number either way). Toggling off mid-advance stops cleanly at
      pulse granularity, not mid-tick.
  - [ ] Visible on/off state distinct from the icon symbol alone (per the
        design doc's own explicit requirement) — e.g. a filled/highlighted
        button background while active, not just a glyph swap.
  - [ ] Pressing `Tick`/`Era` while continuous-advance is active is
        disabled the same way they already grey out during
        `EraState::Advancing` (`time_control_row`'s existing
        `add_enabled_ui(!advancing, …)` pattern) — one in-flight advance
        mechanism at a time.
- [ ] Mutation-level badge: a small marker on the Splice action icon in
      `action_icon_row` reflecting the current capability tier. If task 147
      (Splice trait-pool restriction) hasn't landed a concrete "which tier
      is active" signal yet, coordinate with it rather than inventing a
      placeholder tier here — see Dependencies.
- [ ] `species_row` gains the abbreviated subtext (metabolism + temperature
      preference, e.g. `photo · cold`) below or beside the existing
      glyph+name line, reusing whatever cold/warm labeling
      `species_catalog_line` (`text.rs:~777`) already computes for the
      notebook Catalog rather than duplicating the threshold logic.
- [ ] No regression to any already-shipped sidebar element (hairline
      structure, dot indicators, narrative objective line, headings).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `time_control_row` (`~1024`) — add the toggle button; `action_icon_row` — add the mutation-level badge; `species_row` (`~1000`) — add subtext. |
| `src/input.rs` | New `p`-keybind system for continuous advance, alongside the existing tick/era advance systems. |
| `src/text.rs` | New label/tooltip constants; reuse `species_catalog_line`'s temp-label logic (`~777`) for the species-row subtext instead of re-deriving it. |
| `src/config.rs` | `TimeConfig` — decide/add the continuous-advance rate if it isn't `era_tick_hz`. |

---

## 🧩 Technical Context

- **Current behavior**: `time_control_row` offers `Tick`/`Era` buttons plus
  the notebook toggle; both grey out while `EraState::Advancing`. No loop
  auto-advances pulses. `action_icon_row` renders the four action icons with
  no per-icon badge. `species_row` shows `{metabolism_glyph} {name}` only.
- **Desired behavior**: a third HUD control starts/stops continuous pulse
  advancement, with the same mutual-exclusion the manual controls already
  have; the Splice icon carries a live capability-tier badge; the species
  list row previews metabolism + temperature fit inline.

---

## 🔨 Suggested Implementation

1. Add a `ContinuousAdvance(bool)`-shaped resource (or reuse/extend
   `HudControlIntents`) toggled by both the `p` key and the new button.
2. Drive the actual pulse loop from wherever `EraState::Advancing`'s
   existing single-era stepping lives (`sim.rs`/`input.rs`) — continuous
   mode should step one pulse at a time at the configured rate rather than
   reusing era-block advancement, so toggling off lands cleanly between
   pulses.
3. Style the toggle button per the doc's explicit requirement: distinct
   fill/stroke while active (egui `Button::fill`/`selected` styling, same
   idiom `time_control_row`'s `selectable_label` already uses for the
   notebook button).
4. Coordinate with task 147 (or land after it) for the mutation-tier
   signal; render it as a small corner badge on the Splice icon in
   `action_icon_row`.
5. Extract or reuse the temperature-label helper `species_catalog_line`
   already has, call it from `species_row` too.

---

## ⚠️ Constraints and Caveats

- **Determinism**: continuous-advance is a presentation-layer control (like
  `era_tick_hz` playback speed) — it must not change simulation outcomes,
  only how often `sim::step` gets invoked in wall-clock time. Simulation
  state must not depend on real time elapsed, only on pulse count.
- **No magic numbers**: any new rate/threshold into `SimConfig`.
- Don't touch anything already covered by task 064/065 — verify against
  `src/ui.rs`/`src/text.rs` before assuming a gap, per this task's own
  scope-correction above.

---

## 🔗 Dependencies

- **Depends on**: 147 (Splice trait-pool restriction) for the mutation-tier
  signal the badge reads — implement the toggle and species-subtext items
  independently if 147 isn't done yet, and land the badge once it is, or
  coordinate scheduling so 152 runs after 147.
- **Owns the `p` keybind**: task 150 (control scheme, scoped the same
  session) explicitly excluded `P`/continuous-advancement from its own
  scope — no auto-play loop existed anywhere to bind a key to — and named
  it as a gap for a separate task. This task is that task; don't let 150
  also add a `p` handler.
- **Blocks**: 151 (pixel-grain visual register) — 151 restyles whatever HUD
  elements exist at that point; per this project's explicit phase-2
  sequencing (`abiogenesis-actions.md`/`culture-shock-population-model-aesthetic.md`
  split rationale — "so the HUD isn't restyled and then rebuilt"), 151 must
  land after 152 and 153, not before.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/152-hud-sidebar-diegetic-redesign.md)"$'\n\nExecute this task in the current project.'
```
