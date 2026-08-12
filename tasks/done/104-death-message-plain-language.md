# Task 104 — Plain-language death message for player-placed organisms

> **ID**: `104`
> **Category**: UX / Legibility
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-11 (scoped from `redesign/abiogenesis-death-legibility.md`)

---

## 🎯 Objective

`text::player_organism_death_message` (`src/text.rs:389-408`) already tries
to "answer why instead of only what" (its own doc comment), but renders five
raw signed numbers — e.g. `"your Foo organism at (3, 4) died: gain +0.40,
matrix +0.00, upkeep -0.70, crowding -0.15, predation +0.00"`. Interpreting
that requires already understanding the tick formula (GDD §5.6);
`player_guide.md` tells the player to "check the death log's gain
breakdown," but the message isn't actually legible to someone who hasn't
read the source.

Replace it with one qualitative sentence per death, naming the single
dominant negative contributor in plain language. GDD §7 is explicit that
"metabolisms and environmental ranges always remain readable as anchors" —
only the hidden tag×tag matrix (§11) is meant to stay a mystery. So this
task can be fully direct about temperature/resource/predation/crowding
causes; only the matrix term must stay vague.

A real gap found while grounding this: `OrganismDied` (`sim.rs:16-31`) only
exposes the *final* `gain`, already multiplied by the organism's `env_fit`
(computed at `sim.rs:254`, applied identically for all three metabolisms at
`sim.rs:259-263`). A bad temperature fit and a genuinely absent resource (no
light for Photolithic, no prey for Predator, no residue for Decomposer)
today collapse into the same small `gain` number — there's no way, even
server-side, to tell them apart. `fit` is already computed once per
organism per tick at `sim.rs:254` regardless of metabolism (only actually
used in the `Photolithic` arm of the `gain` match, `sim.rs:259-263`) — so
exposing it on `OrganismDied` is a cheap addition: store the existing local
`fit` into the event pushed at `sim.rs:339-348`, no new computation.

---

## 📋 Acceptance Criteria

- [ ] `OrganismDied` (`src/sim.rs:16-31`) gains an `env_fit: f32` field,
      populated from the `fit` value already computed at `sim.rs:254` and
      currently discarded after the `gain` match — no new computation, no
      change to `env_fit`'s formula (`sim.rs:402-405`).
- [ ] `player_organism_death_message` (`src/text.rs:389-408`) is rewritten
      to take the full `OrganismDied` term set (including the new
      `env_fit`) and produce exactly one short qualitative sentence, picking
      the single dominant negative contributor among: poor temperature fit
      (low `gain` + low `env_fit`), resource absence (low `gain` + decent
      `env_fit` — phrased per metabolism: no light for `Photolithic`, no
      prey for `Predator`, no residue for `Decomposer`), `predation_loss`,
      `crowding_penalty`, and `interaction_delta`. First-pass dominance rule
      (tune if needed, document the final choice in this file or the
      commit): compare each term's magnitude as a fraction of the total
      negative energy delta; the largest wins, ties broken by the order
      listed above (temperature/resource, predation, crowding, matrix).
- [ ] When `interaction_delta` is the dominant cause, the message reads a
      deliberately vague *"harmed by a nearby species"* (or equivalent) —
      **no numeric value, no tag identity, no sign** is shown. This is an
      explicit privacy/mystery-preservation decision (see design doc): the
      matrix term is the one thing this task must not make more legible.
- [ ] All other dominant causes are rendered in plain, direct language with
      **no raw numbers** — e.g. "the temperature here didn't suit it",
      "nothing to feed on here", "eaten by a predator", "too crowded here".
      Exact phrasing is a first-pass placeholder, not locked — tune for
      tone, but keep the "no numbers" rule.
- [ ] `notebook.rs`'s existing player-placed-organism filter
      (`record_events`, `src/notebook.rs:328-369`, using `PlayerPlacedCells`
      as the marker) is reused unchanged — this task only changes what
      `player_organism_death_message` is called with and what it returns,
      not which deaths get logged.
- [ ] Unit tests in `src/text.rs` (or wherever `player_organism_death_message`
      is tested today) covering each dominant-cause branch: construct
      inputs with each term dominant in turn (temperature-fit, resource
      absence per metabolism, predation, crowding, matrix) and assert the
      resulting sentence matches the expected qualitative phrase — plus one
      test asserting the matrix-dominant branch's output contains no digit
      characters.
- [ ] No F2 debug overlay currently surfaces `gain`/`interaction_delta`/
      `upkeep`/`crowding_penalty`/`predation_loss` (`render.rs`'s only F2
      overlay, `EnergyOverlay`/`draw_energy_overlay` around
      `render.rs:334-380`, shows only live per-cell energy) — this task does
      not add one. Note in the commit/this file that raw death-term numbers
      are simply gone from player-facing output after this change, not
      relocated anywhere.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`: seed an organism, let it die (e.g. off
      its temperature optimum, or isolated with no light/prey/residue), and
      confirm the notebook log shows a plain-language sentence with no raw
      numbers.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `OrganismDied` (16-31) — add `env_fit` field; `advance_tick`'s death push (339-348) — populate it from the local `fit` computed at 254; `env_fit` fn (402-405) — read-only reference, unchanged. |
| `src/text.rs` | `player_organism_death_message` (389-408) — full rewrite to qualitative output; add/extend unit tests. |
| `src/notebook.rs` | `record_events` (328-369) — update the call site to pass the new field; `PlayerPlacedCells` filter — reused unchanged. |
| `player_guide.md` | Line 126 currently tells the player to "check the death log's gain breakdown" before suspecting the matrix — update this line to match the new plain-language message. |

---

## 🧩 Technical Context

- **Current behavior**: `player_organism_death_message` takes five `f32`
  terms and formats them as signed numbers with labels ("gain +0.40, matrix
  +0.00, upkeep -0.70, crowding -0.15, predation +0.00"). `OrganismDied`
  does not carry `env_fit` separately from `gain`, so a poor temperature fit
  and a genuinely absent resource are indistinguishable from the event
  alone.
- **Desired behavior**: one plain-language sentence, dominant cause only,
  numbers removed entirely from the player-facing message. The matrix cause
  additionally hides identity/sign, not just the number — see acceptance
  criteria.
- `fit` at `sim.rs:254` is computed unconditionally for every organism each
  tick (Photolithic, Predator, and Decomposer alike), even though only the
  `Photolithic` gain arm (`sim.rs:260`) uses it directly — Predator/
  Decomposer gain is already fit-multiplied earlier, inside
  `predation_gain`/`decomposition_gain` (`sim.rs:186`/`226`). The `fit` at
  254 is real and current for the organism's own cell regardless of
  metabolism, so it's the right value to expose.

---

## 🔨 Suggested Implementation

1. Add `env_fit: f32` to `OrganismDied` (`sim.rs:16-31`), doc-comment it
   analogous to the existing fields. This is a breaking struct-literal
   change: grep `OrganismDied {` across the whole crate (not just
   `sim.rs:339`) before compiling — `notebook.rs:949`, `notebook.rs:992`,
   and `notebook.rs:1033` also construct `OrganismDied` literals in tests
   and need the new field added.
2. At the death push (`sim.rs:339-348`), pass the local `fit` (already in
   scope from line 254) into the new field.
3. Rewrite `player_organism_death_message` in `text.rs`: take the species
   label, position, and all `OrganismDied` terms (including `env_fit` and
   the organism's `Metabolism` — thread it through if not already
   available at the call site, needed to pick the right resource-absence
   phrase); compute the dominant term; match on it to produce one sentence.
4. Update the call site in `notebook.rs::record_events` to pass the new
   arguments.
5. Update `player_guide.md`'s reference to the old numeric breakdown.
6. Add unit tests covering each dominant-cause branch.
7. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
8. Live-verify via `cargo run` per the acceptance criteria.

---

## ⚠️ Constraints and Caveats

- Do not expose `interaction_delta`'s value, sign, or the exerting
  species'/tag's identity anywhere in the rewritten message — this is the
  one deliberate mystery-preservation boundary in this task, everything
  else is meant to become fully legible.
- Do not touch `notebook.rs`'s player-placed-organism filter logic
  (`PlayerPlacedCells` marker, `src/notebook.rs:319-327`'s documented
  reasoning) — reuse it as-is, this task changes message content, not which
  deaths get logged.
- Do not invent a new debug overlay for the removed raw numbers — if one
  doesn't already show them (it doesn't, see acceptance criteria), just
  note they're gone.
- Dominance rule and exact phrasing are first-pass choices; tune visually
  but keep the "no raw numbers, matrix cause stays vague" constraints fixed.

---

## 🔗 Dependencies

- **Depends on**: none (touches `sim.rs`/`text.rs`/`notebook.rs` directly,
  no other in-flight task blocks it).
- **Related, shares logic with**: task 105 (Biosphere trend diagnosis) —
  both need a "which term dominates this death" classifier. If 105 lands
  after this task, its dominant-cause taxonomy should reuse/extract the
  same classification logic introduced here rather than duplicating it (see
  105's Dependencies for the reverse case).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/104-death-message-plain-language.md)"$'\n\nExecute this task in the current project.'
```
