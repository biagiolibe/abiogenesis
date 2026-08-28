# Task 147 — Splice restricted to confirmed traits + growing genome bank + "synthesised" origin

> **ID**: `147`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~2h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

Splice today lets the player swap in or add **any active tag** in the world
(`ui.rs::splice_panel` iterates `world.active_tags` unfiltered) and never
distinguishes a player-synthesised species from an original one anywhere in
the HUD or Catalog. Per the design doc's "laboratory" reading of Splice (you
synthesise a strain from biochemistry you've *actually decoded*, not from
the full unseen pool), this task:

1. Restricts the tags offered by `SwapTag`/`AddTag` to ones the player has
   already **confirmed** in that world's matrix — not the whole active pool.
2. Keeps the xenotrait pool permanently excluded from Splice, at the code
   level (currently moot — see Phase-0-style finding below — but must stay
   true once xenotraits exist).
3. Gives the Catalog an explicit **origin** label per species — seeded /
   indigenous / **synthesised** — the synthesised value being new.
4. Visually distinguishes synthesised species from original ones wherever
   the roster is listed (Seed Palette, Catalog).

Design source: `redesign/processed/abiogenesis-actions.md`, "Splice —
sintesi di una nuova specie in laboratorio" (confirmed-traits constraint,
xenotrait exclusion) and its "Cosa serve per l'integrazione" bullets;
`redesign/processed/abiogenesis-hud-notebook.md` line 117 (dynamic genome
bank, visual distinction) and line 115 (origin era — already implemented,
see below).

---

## Findings against the current build (verified this session, don't re-derive)

- **Splice already creates a genuinely new species**, not an edit of a live
  one (`input.rs::apply_splice`, `world.push_species`) — this part of the
  doc is already implemented (also noted in QUEUE.md's Phase 0 findings).
  `apply_splice` clones a `source` species, applies one edit
  (`SwapTag`/`AddTag`/`ShiftTempOptimum`), and pushes the clone as a new
  `world.species` entry.
- **The genome bank already grows at runtime, functionally**: the Seed
  Palette (`ui.rs:611-629`) iterates `0..world.species.len()`, skipping only
  `world.is_wild(..)` — so a species `apply_splice` just pushed is already
  seedable next season without any additional plumbing. What's missing is
  **visual distinction**, not growth.
- **No xenotrait pool exists yet** in code (`grep -rn xenotrait src/` is
  empty; that pool is task 168, Phase 5) — point 2 above is a code-level
  guard against a pool that doesn't exist yet. Add the exclusion as a
  structural gate (e.g. filtering by an explicit "assignable" set rather
  than "all defined tags"), not a runtime check against something absent,
  so task 168 can't accidentally make xenotraits Splice-reachable by
  omission later.
- **Confirmation is stored per *pair*, not per tag.** `MatrixKnowledge`
  (`knowledge.rs`) only answers `is_confirmed(exerter: TagSlot, receiver:
  TagSlot) -> bool` — there is no existing notion of "this one tag, in
  isolation, is confirmed." **Open design call, not resolved by the source
  doc**: define a tag as confirmed if it appears (as exerter or receiver) in
  at least one confirmed pair against any other active tag. Simplest
  reading consistent with "you've decoded something about this trait," and
  symmetric with how the notebook's hypothesis grid already reports
  per-tag rows. Flag this reading explicitly in the PR/commit rather than
  silently assuming it — it's a judgment call, not a spec.
- **No `origin` field or seeded/indigenous label exists anywhere today** —
  the design doc's phrasing ("origine acquisisce un terzo valore oltre a
  *seminata* e *indigena*") assumes a two-value field that isn't actually
  built. `world.is_wild(SpeciesId)` (`world.rs:2249`, backed by
  `wild_species: Vec<SpeciesId>`) is the only origin-adjacent signal that
  exists, and the Catalog doesn't surface it as text today. This task
  builds the origin concept from scratch as three values, not two-plus-one.
- **Origin era already exists**: `world.species_seeded_era` +
  `text::species_population_line` already show when a species was first
  seeded (task 103) — the doc's line 115 open question ("verificare se
  questo dato esiste già") is answered: yes, no work needed here.

---

## 📋 Acceptance Criteria

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] `splice_panel`'s `SwapTag`/`AddTag` tag lists (`ui.rs:1192-1227`) only
      offer tags confirmed per the definition above — computed from
      `MatrixKnowledge`, passed into `splice_panel` (currently reads only
      `world`/`draft`, no knowledge parameter).
- [x] A tag with zero confirmed pairs is simply absent from both lists (not
      shown-disabled) — consistent with "you can't yet synthesise what you
      haven't decoded," and avoids a UI state (disabled-but-listed) nobody
      asked for.
- [x] Assignable-tag filtering is structurally exclusive of any future
      xenotrait pool (e.g. sourced from `world.active_tags` intersected with
      an explicit non-xeno set, not "all tags minus a xeno blocklist" that a
      later addition could bypass by omission). Add a doc comment recording
      this as the guard task 168 must respect.
- [x] New `SimWorld` origin tracking: a `spliced_species: Vec<SpeciesId>`
      alongside the existing `wild_species: Vec<SpeciesId>` (same shape,
      same rationale — see `world.rs:566-573`'s doc comment), populated in
      `apply_splice` at the same point it calls `push_species`.
- [x] A `species_origin`-style helper (mirrors `is_wild`) resolving
      seeded / indigenous / synthesised from `wild_species` /
      `spliced_species` / neither, single source of truth for both the
      Catalog and the Seed Palette.
- [x] Catalog (`notebook.rs::catalog_panel`) shows the origin label per
      species (new `text::` string(s), no magic literals inline).
- [x] Seed Palette (`ui.rs::species_row` / the loop at `611-629`) visually
      distinguishes a synthesised species from an original one — reuse the
      Catalog's origin label or a lighter marker (glyph/color), player's
      choice of exact treatment, but it must be visible without opening the
      notebook (doc's own framing: "the HUD must anticipate this").
- [x] Test coverage: a pair-confirmed tag appears in the Splice lists, an
      unconfirmed one doesn't; a spliced species reports `synthesised`
      origin and appears correctly marked in both Catalog and Seed Palette.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `splice_panel` (`~1145-1237`) — filter tag lists; Seed Palette loop (`~611-629`) — visual marker. |
| `src/input.rs` | `apply_splice` (`~502-594`) — record `spliced_species` alongside `push_species`. |
| `src/world.rs` | `wild_species`/`is_wild` (`~566-573`, `2249`) — add `spliced_species` + an origin-resolving helper next to it. |
| `src/knowledge.rs` | `MatrixKnowledge::is_confirmed` — source of the new per-tag-confirmed derivation. |
| `src/notebook.rs` | `catalog_panel` (`~1177+`) — origin label per species row. |
| `src/text.rs` | New origin-label strings. |

---

## ⚠️ Constraints and Caveats

- Don't build anything xenotrait-specific now (task 168, Phase 5 doesn't
  exist yet) — only the structural guard noted above.
- `sim`/`world`/`config` stay free of `bevy_egui` — the per-tag-confirmed
  derivation belongs as a plain function over `MatrixKnowledge`/
  `world.active_tags`, callable from both `ui.rs` and any future test.
- Keep the "confirmed if in at least one confirmed pair" definition
  explicit in a doc comment — it's this task's own judgment call, and a
  future design pass may want a stricter reading (e.g. confirmed against
  *every* other active tag).

---

## 🔗 Dependencies

- **Depends on**: none new — builds on already-shipped `Splice` (task 025),
  `MatrixKnowledge` (task 136b), `wild_species`/origin-era tracking
  (task 098/103).
- **Blocks**: none in Phase 2. Task 168 (Phase 5, xenotraits) must respect
  the structural exclusion guard this task adds.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/147-splice-confirmed-traits-genome-bank.md)"$'\n\nExecute this task in the current project.'
```
