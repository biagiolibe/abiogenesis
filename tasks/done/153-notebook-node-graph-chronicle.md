# Task 153 — Notebook: Chronicle section, "descends from", quiet-era compression

> **ID**: `153`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~2h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

Add a fourth notebook section, **Chronicle**: the narrated history of the
world, distinct from the Observation log's raw scientific data — an
archive of every era-reveal card (`EraReveal`) once the player dismisses
it, with consecutive quiet eras (no reveal-worthy event) compressed into a
single line instead of one row per era. Extend the Species catalog with a
**"descends from"** field (parent species name, one hop back) for any
species born by speciation, and link every speciation entry in the
Chronicle back to its parent the same way.

Design source: `redesign/processed/abiogenesis-notebook-cronaca.md` (read
alongside `abiogenesis-hud-notebook.md` §8 for the base notebook layout it
extends, per the doc's own note).

**Scope correction against the doc — read this before implementing.** The
title in `tasks/QUEUE.md` ("node graph...") echoes the GDD v0.7 changelog
line "relationship graph replaces the matrix grid" — but that replacement
is **already shipped history**, not open work: `notebook.rs::hypothesis_grid`
(tasks 021/028/031/101/102) already draws `world.active_tags` as a node
graph with directed edges, confirmed/partial styling, and reveal-on-first-
observation. There is no matrix grid left to replace. This task's actual
new graph-adjacent work from the doc — **nodes as 3-letter trait codes**
and **xenotrait double-ring styling** — both depend on systems that don't
exist yet in this codebase (task 155, trait archetypes, Phase 3; task 168,
xenotraits, Phase 5) and **are out of scope here**. Don't touch
`hypothesis_grid`'s node/edge rendering in this task; it stays exactly as
it is until 155/168 land. This task's real scope is the doc's **Chronicle
section**, **quiet-era compression**, and the Catalog's **"descends from"**
field — the **"origin: sintetizzata"** tag the doc also lists is task 147's
job (Splice-created species), not this one's; consume the flag 147 adds,
don't duplicate it.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] New `Chronicle`/`ChronicleLog` resource (`notebook.rs`, mirrors
      `ObservationLog`'s shape) persists one entry per dismissed
      `EraReveal`: era number, `RevealTier`, and the narrated text already
      produced for the reveal card (`text::` module — reuse the existing
      strings verbatim, no second text-generation pass per the doc's own
      "no dedicated generation" note). Archived when the player dismisses
      the reveal (`screens::era_reveal_screen_ui`'s transition out of
      `EraState::Reveal`), before `build_era_reveal` next overwrites
      `EraReveal`.
- [ ] Consecutive eras with no reveal-worthy event compress into one
      discrete row (e.g. "eras 6-9: quiet") instead of a row per era. An
      era counts as "quiet" the same way `build_era_reveal` already
      classifies `RevealTier::Minor` with nothing else notable — check its
      existing tier logic (`sim.rs:715`) rather than inventing a second
      definition of "quiet."
- [ ] Chronicle renders as a fourth section in `notebook_window`
      (`notebook.rs`), below Catalog, most-recent-first, each entry's
      visual weight (marker size or color intensity — reuse whatever
      `hypothesis_grid`/reveal-card styling already ties to `RevealTier`,
      no new weight system) scaled by its tier.
- [ ] A speciation entry in the Chronicle names the parent species — same
      data as the Catalog's "descends from" field below, implemented once
      and shared (the doc's own instruction).
- [ ] `Species` (`world.rs:110`) gains a persisted `parent: Option<SpeciesId>`
      (or equivalent), set at construction time for a speciated species
      (`sim.rs`'s `speciate`/`EraEvolutionReveal::parent` at `sim.rs:579`
      already carries this value transiently — persist it onto the new
      species itself, `None` for anything seeded/synthesised/wild).
      One hop back only, no full lineage tree — the doc defers deeper
      reconstruction to reading the Chronicle manually.
- [ ] Catalog card (`notebook.rs::catalog_panel`) shows "descends from:
      {parent name}" when `Species::parent` is `Some`, omitted otherwise.
- [ ] `WorldResetParams`/equivalent clears `ChronicleLog` on world
      (re)start, same lifecycle as `ObservationLog`.
- [ ] At least one test: dismissing N consecutive quiet eras produces one
      compressed Chronicle row, not N; a speciation reveal produces a
      Chronicle entry naming the correct parent.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `notebook_window` (section layout, `~677`), `ObservationLog` (pattern to mirror for `ChronicleLog`), `catalog_panel` (add "descends from"). |
| `src/sim.rs` | `EraReveal`/`RevealTier` (`~560`-`620`), `speciate` (`~479`), `EraEvolutionReveal::parent` (`~579`) — source of the tier/text/parent data to archive. |
| `src/screens.rs` | `era_reveal_screen_ui` (`~191`) — dismissal point where the reveal should be archived into `ChronicleLog` before it's lost. |
| `src/world.rs` | `Species` (`~110`) — add persisted `parent` field. |
| `abiogenesis-gdd.md` | §7 (notebook sections) — document the new Chronicle section once implemented. |

---

## 🧩 Technical Context

- **Current behavior**: `EraReveal` is a single `Resource` overwritten every
  era close (`sim.rs:723`, `*reveal = EraReveal { .. }`) and read only by
  the reveal-card screen (`screens::era_reveal_screen_ui`) — once dismissed
  and the next era's reveal is built, the old text is gone. `Species` has
  no parent reference; the only place `parent: SpeciesId` currently exists
  is transiently on `EraEvolutionReveal` (`sim.rs:579`), consumed by
  `build_era_reveal`'s own display logic and never persisted onto the new
  `Species` record it creates. `RevealTier` (`Minor`/`Notable`/`Epochal`,
  `sim.rs:563`) is already computed per era and used for nothing visual
  beyond the reveal card itself.
- **Desired behavior**: every dismissed reveal becomes a permanent,
  browsable Chronicle entry; quiet stretches read as one line, not visual
  noise; a species' Catalog card and its Chronicle speciation entry both
  say who it came from.

---

## ⚠️ Constraints and Caveats

- **Don't touch `hypothesis_grid`** (the existing node/relationship graph)
  — see the scope correction above. The doc's node-styling changes belong
  to tasks 155/168, not this one.
- **No new text generation** — Chronicle entries are archived copies of
  text the reveal system already produces.
- **No dedicated lineage tree view** — the doc explicitly defers that;
  "descends from" is a flat one-hop field, full ancestry is reconstructed
  by reading the Chronicle in order, a future refinement.
- Keep `sim`/`world` free of `bevy::render`/`bevy_egui` per `TECH_DESIGN.md`
  §5 — `ChronicleLog`'s egui rendering lives in `notebook.rs`, its data
  (era/tier/text/parent) can be plain structs reused from `sim`.

---

## 🔗 Dependencies

- **Depends on**: 140 (`EraReveal`/`RevealTier` already exist), 147 (the
  "sintetizzata" origin flag this task's Catalog rendering should read but
  not implement).
- **Blocks**: none. The node-graph styling changes the doc also describes
  (3-letter trait codes, xenotrait double ring) are separately blocked on
  155 (Phase 3) and 168 (Phase 5) — not on this task.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/153-notebook-node-graph-chronicle.md)"$'\n\nExecute this task in the current project.'
```
