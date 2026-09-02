# Task 190 — Sync onboarding content (in-game panel + player guide) to current mechanics

Priority: 🔴 P1
Status: IN_PROGRESS
Review: REQUIRED
Dependencies: none
Reasoning: medium

## Authority

- `player_guide.md` (player-facing manual, per `CLAUDE.md`'s doc table: "also surfaced in-game via the main menu's 'How to play' panel").
- `src/text.rs:28-90` (`HOW_TO_PLAY_SECTIONS`) — the in-game panel's own content, rendered by `screens.rs::intro_screen_ui` and `menu.rs::main_menu_ui`.
- Accepted tasks whose shipped mechanics are missing from both: 179 (objective types), 153 (Chronicle), 147 (Splice confirmed-trait restriction), 152/176/177 (controls).

## Goal

Both onboarding surfaces are stale relative to accepted Phase 2/3 work, found by
a direct comparison against `tasks/QUEUE.md`'s `[x]` rows:

- **Metabolism**: `HOW_TO_PLAY_SECTIONS` (`src/text.rs:61-68`) lists only
  Photolithic/Predator/Decomposer — missing **Chemolithotroph** (draws energy
  from local toxicity), which exists in code and is already documented in
  `player_guide.md:54`. The in-game panel and the guide disagree with each
  other here, not just with the code.
- **Objectives**: both documents describe only Coexistence / hostile-zone
  survival / bloom trigger (`player_guide.md` also lists forced Speciation;
  the in-game panel doesn't even have that). Task 179 added and wired into
  the generator four more kinds — **Homeostasis, Tolerance,
  WildCoexistence, Rootedness** — present in neither document.
- **Notebook**: neither document mentions the **Chronicle** section (task
  153, GDD §7's fourth notebook section).
- **Splice**: neither document says Splice is restricted to **confirmed
  traits only**, drawing from a growing genome bank (task 147) — both read
  as if any tag/thermal edit is selectable from the start.
- **Controls**: `player_guide.md` has the wheel-pan (task 177) but not the
  continuous-advance key; `HOW_TO_PLAY_SECTIONS` has neither. Continuous
  advance toggles on **`p`** (`src/input.rs:407-431`, `toggle_continuous_advance`).

This matters now specifically because the pending human-playtest gate (task
171, `tasks/171-causal-legibility-playtest-gate.md`) hands the playtester
*only* the in-game onboarding, no other explanation. If that onboarding
under-describes objectives and metabolisms, the playtest risks measuring
confusion caused by stale documentation rather than the game's actual
legibility — the thing 171 exists to check. This task must land, and its
live-check must pass, before 171's human-playtest half is run.

## Expected code surface

- Add or change: `src/text.rs:40-90` (`HOW_TO_PLAY_SECTIONS` entries —
  "Metabolism and temperature", "Actions and budget", "The notebook",
  "Objectives and failure", "Controls") and `player_guide.md` (Species and
  metabolism, The notebook, Objectives/victory/failure, Controls sections)
  to reflect the gaps listed above.
- Preserve: `HOW_TO_PLAY_SECTIONS`' existing tone/length per section (short,
  scannable — don't turn it into a copy of `player_guide.md`; keep each
  entry a tightened summary, per its own existing register). Do not
  restructure `screens.rs`/`menu.rs` rendering, only the content array.
  `sim.rs`/`world.rs`/`config.rs` untouched.
- Evidence needed: `cargo build`/`clippy -D warnings`/`fmt` clean; a live
  check (`cargo run`) confirming the intro screen and main menu's "How to
  play" panel render the updated sections without truncation/overflow at
  the panel's existing scroll height (`src/menu.rs:38`).

## Out of scope

- Unifying the two onboarding sources into one (`player_guide.md` generating
  `HOW_TO_PLAY_SECTIONS`, or vice versa) — worth naming as a follow-up (the
  duplication is exactly why they drifted), but a structural change, not a
  content fix; don't do it as part of this task.
- Any new mechanic, balance change, or `sim`/`world`/`config` edit.
- The Chronicle's own content/behavior — only documenting that it exists.

## Acceptance criteria

- `HOW_TO_PLAY_SECTIONS` and `player_guide.md` both mention all four
  metabolisms, all eight objective kinds (four original + Homeostasis/
  Tolerance/WildCoexistence/Rootedness) plus forced Speciation, the
  Chronicle section, Splice's confirmed-trait-only restriction, and the
  continuous-advance key (`p`) alongside the existing control list.
- No mechanic described in either document is contradicted by the other.
- Live check confirms both surfaces render correctly (no overflow at the
  panel's current max-height, `src/menu.rs:38`).
- `tasks/QUEUE.md`'s row for 190 and the note on task 171 updated once this
  lands, so 171's human-playtest half is unblocked.

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
claude "$(cat tasks/190-onboarding-content-sync.md)"$'\n\nExecute this task in the current project.'
```
