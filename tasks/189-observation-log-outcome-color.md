# Task 189 — Observation-log markers: outcome color, not species color

Priority: 🟡 P2
Status: QUEUED
Review: REQUIRED
Dependencies: none
Reasoning: medium

## Authority

- `abiogenesis-gdd.md` §7 "Observation log" bullet, `[DECIDED, task 181 gap resolution]` clause (added alongside this task).
- `VISUAL_STYLE_GUIDE.md` §1 rule 3 ("color = state, never identity") and §8's task-181 implementation-status line.

## Goal

Task 181 (`tasks/done/181-notebook-chrome-fidelity.md`) shipped every AC except one:
Observation-log entry markers in `src/notebook.rs` (~797-844) draw
`species_color(species)` per entry instead of an outcome color. It was left
unfixed because `LogEntry` (`src/notebook.rs:39-43`, fields `era`/`species`/
`text` only) carries no valence signal, and the task correctly refused to
invent new `SimWorld` state to satisfy a UI-styling AC.

The gap is now resolved as a design decision (GDD §7), not by adding sim
state: the outcome is intrinsic to *which event kind* produced each entry,
already known statically at every push site in `src/notebook.rs`:

- Negative: `extinctions` and a player-placed organism's `deaths`
  (`record_events`, ~477-550).
- Positive: `species_evolved` (`record_events`, ~477-550) and the per-era
  birth-tally line (`tally_births`, ~559-594).
- Neutral: `objectives_advanced`, `terrain_revealed` (`record_events`,
  ~477-550), matrix confirmation (`accumulate_evidence`, ~321-348), and
  terrain-gate confirmation (`accumulate_terrain_evidence`, ~358-413).

This is `notebook.rs`'s own UI struct being extended with a UI-derived
classification fixed at push time — not new `sim`/`world`/`config` state.

## Expected code surface

- Add or change: `src/notebook.rs` — add an outcome/valence field to
  `LogEntry` (small enum, e.g. `Positive`/`Negative`/`Neutral`), set it
  explicitly at each of the ~6 push sites listed above, and use it in the
  Observation Log rendering (~797-844) to pick the marker color instead of
  `species_color(species)`. Reuse the existing positive/negative color pair
  already defined for the relationship-graph edges (`EDGE_POSITIVE_COLOR` /
  `EDGE_NEGATIVE_COLOR`, `src/notebook.rs:975-976`, rgb(96,200,120) /
  rgb(220,96,96)) — one color pair per semantic role, not a second pair
  picked from the mockup's raw `#7fae6a`/`#c96a5c`. Neutral entries keep the
  current neutral/no-color glyph treatment.
- Preserve: species identity stays carried by name text only — the marker
  color must never vary by species, only by outcome kind. `sim.rs`,
  `world.rs`, `config.rs` untouched.
- Evidence needed: `cargo build`/`clippy -D warnings`/`fmt` clean; a live
  check (`cargo run`) with at least one of each outcome kind present in the
  log confirming the color reads correctly.

## Out of scope

- The separate *clean vs confounded* evidence-quality indicator (GDD §7,
  still `[PROPOSED]`) — a different axis about evidence weight, not outcome
  sign. Do not implement it here.
- The map, HUD, or any surface outside `notebook.rs`'s Observation Log
  section.

## Acceptance criteria

- Each of the ~6 `LogEntry` push sites sets an explicit outcome consistent
  with the classification above — no default/inferred fallback.
- Observation Log entries render with a color keyed to outcome: positive/
  negative reuse `EDGE_POSITIVE_COLOR`/`EDGE_NEGATIVE_COLOR`; neutral
  entries keep the current neutral/no-color glyph treatment.
- No species-colored markers remain in the Observation Log.
- Live check (`cargo run`) with one of each outcome kind present confirms
  the color reads correctly.
- `VISUAL_STYLE_GUIDE.md` §8's task-181 line updated from open to resolved.
- `tasks/QUEUE.md`'s row for 181 updated to drop the "flagged as possibly
  needing new data" framing (superseded by this task).

## Validation

- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## Completion

- `Review: REQUIRED`: set this task's and `tasks/QUEUE.md`'s status to
  `READY_FOR_REVIEW` only after validation passes; a reviewer-integrator (a
  different identity) then applies `docs/CODE_REVIEW_PROMPT.md` and records
  `ACCEPTED`.

## Delegating this task

```bash
claude "$(cat tasks/189-observation-log-outcome-color.md)"$'\n\nExecute this task in the current project.'
```
