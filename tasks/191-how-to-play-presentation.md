# Task 191 — "How to play" presentation: chrome, scannability, and height

Priority: 🟡 P2
Status: READY_FOR_REVIEW
Review: REQUIRED
Dependencies: none (190 already landed the content this task reformats)
Reasoning: medium

## Authority

- `VISUAL_STYLE_GUIDE.md` §2 (typography — monospace, uppercase muted-gray
  section labels), §6 (HUD/notebook chrome — one continuous panel divided
  by hairlines, no nested boxed sub-panels).
- `abiogenesis-gdd.md` §"No guided tutorial" `[PROPOSED]` — a hand-holding
  wall of instruction before the player has touched anything works against
  the game's own teaching method; the fix named there is "one non-blocking
  contextual hint," not a mandatory read-through.
- `src/ui.rs::section_header` (1536), `src/ui.rs::hairline` (1473) — the
  chrome primitives this task reuses, already `pub(crate)` and already
  imported into `menu.rs` (`apply_monospace`, `outline_button_auto` from
  the same module).

## Goal

Task 190 fixed *what* the "How to play" content says; this task fixes *how*
it's shown. User-reported problems, both confirmed against the current code:

1. **Formatting doesn't help reading.** `menu.rs:119-124` and
   `screens.rs:70-74` both render each section as plain `ui.strong(heading)`
   + `ui.label(body)`, where `body` is one dense paragraph-length string
   (`src/text.rs:40-90`). No visual hierarchy, no scanning aid — it reads
   as a wall of prose, not a reference a player can skim mid-decision.
2. **The panel doesn't use the space it has.** `HOW_TO_PLAY_PANEL_HEIGHT`
   (`menu.rs:41`, `260.0`) and `INTRO_GUIDE_HEIGHT` (`screens.rs:47`,
   `340.0`) are fixed pixel constants, independent of the actual window
   size. On any window taller than a few hundred pixels, the guide is
   boxed into roughly half the available height and forces scrolling for
   content that would otherwise fit — while the rest of a full-screen
   interstitial (`interstitial`, `screens.rs:377-415`, already a
   `CentralPanel` over the whole viewport) sits unused.

Both screens share the same `text::HOW_TO_PLAY_SECTIONS` array and render
loop; fix both call sites, not one.

## Design decisions

- **Reuse the notebook's own chrome instead of inventing new chrome.**
  `section_header` (uppercase, small, muted `#7d848a`) replaces
  `ui.strong(*heading)`; `hairline()` replaces the plain `ui.add_space(8.0)`
  between sections — the same "one continuous panel, divided by hairlines"
  pattern §6 already specifies and the notebook already uses. No new colors,
  no new panel material.
- **Bullets, not paragraphs.** Change `HOW_TO_PLAY_SECTIONS`'s type from
  `&[(&str, &str)]` to `&[(&str, &[&str])]` — one short line per fact
  instead of one run-on sentence per section. This is a reformatting of
  190's existing content into shorter lines, not new facts: split each
  current paragraph at its natural clause boundaries. Render each line with
  a leading bullet glyph consistent with the rest of the game's block-icon
  language (reuse whatever the notebook/catalog already uses for a list
  marker — check before inventing a new one).
- **Responsive height, not a magic constant.** Replace
  `HOW_TO_PLAY_PANEL_HEIGHT`/`INTRO_GUIDE_HEIGHT` with a height derived from
  `ui.available_height()` (minus whatever's reserved below it — the
  "Begin"/toggle button and its spacing) so the guide fills the actual
  window on both screens instead of a fixed half-height box.
- **Trim the mandatory intro, keep the full reference optional.** The intro
  interstitial (`screens.rs::intro_screen_ui`) currently shows the *entire*
  7-section guide before a first-time player's first "Begin" — the exact
  mandatory-wall-of-text shape the GDD's own "No guided tutorial" principle
  warns against. Narrow the intro to a short primer: "The premise," "The
  loop," and the single most load-bearing control (`Tab` opens the
  notebook) — 3 sections, not 7 — plus one line pointing at the full guide
  ("Full guide any time from the main menu's 'How to play' button," which
  already exists and stays unchanged). The main menu's opt-in panel keeps
  the complete, newly-reformatted section list — it's consulted by choice,
  not forced reading, so its full depth is appropriate there.

## Expected code surface

- Add or change: `src/text.rs:40-90` (`HOW_TO_PLAY_SECTIONS` type and
  content, split into bullet lines); `src/menu.rs:41,113-125` (chrome +
  responsive height); `src/screens.rs:44-83` (chrome + responsive height +
  intro-specific short section subset).
- Preserve: the two screens' existing entry points and button behavior
  (`show_guide` toggle in the main menu, "Begin" in the intro); the
  monospace-everywhere rule; every fact task 190 added (metabolisms,
  objective kinds, Chronicle, Splice restriction, controls) — reformatted,
  not dropped, from whichever surface still carries the full list.
  `sim.rs`/`world.rs`/`config.rs` untouched.
- Evidence needed: `cargo build`/`clippy -D warnings`/`fmt` clean; a live
  check (`cargo run`) on at least two window sizes showing the guide fills
  available height without needless scrolling at normal sizes, sections
  read as scannable bullets with hairline-divided headers, and the intro
  screen's shortened primer still leads to the main menu's full guide
  correctly.
  - Implementer validation performed: `cargo build`, `cargo clippy --all-targets -- -D warnings`,
    `cargo fmt --check`, and `cargo test` (full suite, including
    `determinism`/`balance`) all clean. No GUI live check was performed —
    the sandbox has no Screen Recording permission (same constraint task
    190 recorded). Live verification on real window sizes is still owed to
    the user before `ACCEPTED`.

## Out of scope

- Any change to the *facts* task 190 established — this task only
  reformats and re-scopes where they're shown, doesn't add or remove
  mechanics coverage from the full guide.
- Tabs, accordions, or any navigation widget beyond what `section_header`/
  `hairline` already give a single continuous scroll — this game's
  established idiom (HUD, notebook) is one continuous panel, not a tabbed
  UI; don't introduce a new interaction pattern for this one surface.
- Merging `player_guide.md` and `HOW_TO_PLAY_SECTIONS` into one source —
  named as a follow-up by task 190, still out of scope here.

## Acceptance criteria

- Both `menu.rs`'s and `screens.rs`'s "How to play" rendering use
  `section_header`/`hairline` instead of `ui.strong`/flat `add_space`.
- `HOW_TO_PLAY_SECTIONS` bodies render as short bulleted lines, not dense
  paragraphs, on both surfaces.
- Neither screen's guide area is bounded by a fixed pixel constant anymore
  — height derives from available space at render time.
- The main-menu panel shows the complete section list (190's full content);
  the intro interstitial shows only the 3-section short primer plus the
  "full guide from the main menu" pointer line.
- Live check confirms readability and correct height behavior on at least
  one normal and one small window size, no clipped or overlapping text.

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
claude "$(cat tasks/191-how-to-play-presentation.md)"$'\n\nExecute this task in the current project.'
```
