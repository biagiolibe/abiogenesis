# Task 191 — "How to play" presentation: chrome, scannability, and height

Priority: 🟡 P2
Status: IN_PROGRESS
Review: REQUIRED
Dependencies: none (190 already landed the content this task reformats)
Reasoning: medium

## Amendment 3 — emphasis, contrast, width (2026-09-02, post-indent-fix screenshot)

Hanging indent (Amendment 2) works — the follow-up screenshot shows wrapped
lines correctly landing under the text, not the bullet. User's next
request: the guide is readable now but still visually flat — no color, no
isolation for the individual commands that deserve to stand out (Controls
section, mainly) — plus the content column should be wider. This amendment
proposes a concrete design for that, staying inside `VISUAL_STYLE_GUIDE.md`
§1 rule 3 ("color = state, never identity") — so the fix is **typographic
weight/tone, not a new color**, reusing tokens the palette already defines
rather than inventing one for this single panel.

**1. Wider content column.** Bump `HOW_TO_PLAY_CONTENT_WIDTH` (`ui.rs`,
currently `600.0`) up — target roughly 720-800px, enough that the longer
sentences (the Controls `P:` line, the Objectives intro sentence) wrap at
most once instead of twice, without approaching full-window sprawl (the
thing Amendment 1 fixed). Guard it against small windows: clamp to
`ui.available_width()` (or whatever the surrounding layout already
provides) so a narrow window can't get an overflowing fixed-width child —
check this didn't already need guarding for the current `600.0` value
before assuming it's new work.

**2. Bold the lead term on "Term: description"-shaped bullets.** Most
Controls bullets and four of the Objectives bullets already have this
shape ("`Left click`: perform the selected action...", "`Homeostasis`:
hold a species' energy..."). Split each such line at its *first* colon at
render time (`src/menu.rs`, `src/screens.rs`, wherever the bullet loop
lives after Amendment 2's `horizontal_top` change) and render the part
before the colon with `egui::RichText::new(term).strong()` in the same
body-ink color, the part after in the existing weight — reusing the
already-established `.strong()` idiom (`section_header` already leans on
bold-like emphasis for headers) rather than adding a new color token. A
line with no colon renders exactly as it does today, unstyled — this only
touches lines that already have the shape. Optionally render the
description half in the palette's existing **dim ink** token (`#5a5c64`,
already defined as "least prominent text" in `VISUAL_STYLE_GUIDE.md` §3.1)
instead of full body ink, so the bold term reads as the scannable anchor
and the explanation recedes slightly — try it, keep it only if it doesn't
hurt legibility of the longer descriptions.

**3. Split remaining multi-item bullets for the same one-fact-per-line
consistency Amendment 2 already established for the metabolism lines:**

- Controls' `"Space: advance one season. Shift+Space: a full era at
  once."` is one bullet carrying two key bindings — split into two lines
  ("Space: advance one season." / "Shift+Space: a full era at once.") so
  both get the same bold-lead-term treatment as every other control.
- Objectives' `"Coexistence, surviving a hostile zone, triggering a
  bloom."` is the odd one out in that section — the four newer objective
  kinds below it (Homeostasis/Tolerance/Wild coexistence/Rootedness) each
  get their own named "Term: description" line, but these original three
  are crammed into one unnamed list. Reformat into three "Term:
  description" lines matching the others' shape and depth, e.g.
  `"Coexistence: sustain multiple species at once."`, `"Hostile zone:
  survive on the world's toxic biome."`, `"Bloom: grow a species past a
  population threshold."` — same facts `player_guide.md`'s "Objectives,
  victory, and failure" section already states, just reformatted to match
  its siblings and become eligible for the same emphasis. Content
  reformatting only, not new facts — cross-check wording against
  `player_guide.md:102-105` and `HOW_TO_PLAY_SECTIONS`'s own existing
  four-kind lines for register consistency.

**Additional acceptance criteria** (append, don't replace):

- `HOW_TO_PLAY_CONTENT_WIDTH` (or equivalent) is wider than `600.0`,
  clamped so it can't overflow a narrow window.
- Every "Term: description"-shaped bullet (Controls, all seven Objectives
  kinds) renders its lead term visually distinct (bold, optionally dimmed
  description) from the rest of the line; lines without that shape are
  unaffected.
- No bullet states two unrelated commands/objective-kinds in one line
  where the rest of its section keeps one per line.
- User live-check confirms the Controls section in particular now reads as
  scannable key/action pairs rather than a flat block of sentences.

---

## Amendment 2 — readability review (2026-09-02, post-containment screenshot)

The width-containment fix (Amendment 1 below) worked — the guide now reads
as one centered, consistently-widthed block, not a full-window sprawl next
to a narrow menu column. User's follow-up: "ancora migliorabile come
leggibilità" (still improvable for readability), with a live screenshot of
the main menu. Two concrete defects visible in that screenshot, both
independent of Amendment 1's fix:

1. **Wrapped bullet lines have no hanging indent.** Every `ui.label(format!
   ("{bullet} {line}"))` call (`menu.rs`, `screens.rs`) wraps a long line at
   the label's own width, but the continuation lands flush at the panel's
   left edge — the same column the bullet glyph itself sits in — not
   indented under where the text started. Visible repeatedly in the
   screenshot: "...biochemistry is / hidden." and "...wild species /
   met." both read as if "hidden."/"met." were their own new bullet item,
   because nothing distinguishes a wrapped continuation from a fresh line.
   This is the dominant readability problem — worse than density or
   spacing. Fix: give each bullet a real hanging indent so continuation
   lines align under the first word of text, not under the bullet glyph.
   `egui::Grid` with two columns (a narrow fixed-width bullet column, a
   wrapped-text column) is the standard egui pattern for this — each row's
   second-column label wraps within that column's own width, keeping every
   line of a multi-line bullet aligned under the text start. Verify with an
   actual long line (e.g. the Controls section's `P:` entry) that the
   wrapped line lands under "toggle", not under "▪".
2. **One bullet, two unrelated facts.** `text::HOW_TO_PLAY_SECTIONS`'s
   "Metabolism and temperature" section has two lines each stating two
   different metabolisms' fuel source in one sentence ("Photolithic draws
   energy from light, Predator from neighboring organisms." and
   "Decomposer draws from residue in its own or a neighboring cell,
   Chemolithotroph from local toxicity.") — every other section keeps one
   fact per bullet; these two don't, which is part of why they read
   denser/harder to scan than the rest. Split each into two separate
   bullets (four metabolism lines total, one per metabolism), matching the
   one-fact-per-line rule the rest of the content already follows. Content
   only, not new facts — same information, one clause per line.
3. **Modest breathing room between bullets.** Once the indent fix (item 1)
   lands, add a small `add_space` (roughly 2-4px, don't overdo it — this is
   a density fix, not a redesign) between consecutive bullet lines within a
   section, so individual facts read as distinct items rather than a
   run-on block. Keep `hairline()`'s existing spacing between *sections* as
   is — that part reads fine in the screenshot.

**Additional acceptance criteria** (append, don't replace):

- A wrapped bullet line's continuation visibly aligns under the bullet's
  text, not under the bullet glyph — check against at least one bullet
  known to wrap at the current content width (the Controls `P:` line, or
  similar).
- No bullet line states two unrelated facts joined by a comma where the
  rest of the section uses one fact per line.
- User live-check confirms the wrapped-line/orphan-line impression from the
  2026-09-02 screenshot is resolved.

---

## Amendment 1 — review feedback from live check (2026-09-02)

The implementer's changes (bullets, `section_header`/`hairline`, responsive
height) landed and validated clean, but the user's own live check (the AC
this task was still owed) found the result "still a bit too full and mixed
together." Root cause, confirmed by reading the current code:

- `menu.rs::main_menu_ui` wraps the title/seed field/New Run button/unlocks
  line/guide-toggle button in one `ui.vertical_centered(...)` — a narrow,
  centered column. The "How to play" `ScrollArea` (lines ~114-131) sits
  *outside* that closure, in the panel's default left-aligned, full-width
  layout. Two different alignments/widths stacked in the same screen — a
  narrow centered block giving way to a full-window-wide left-aligned
  bullet list — reads as visually inconsistent, which is almost certainly
  what "pieno e mischiato" is naming, not the bullets/hairlines themselves.
- `screens.rs::intro_screen_ui` has the same shape: heading/pointer-line/
  Begin button sit in the interstitial's outer `vertical_centered`, while
  the bullet loop is deliberately re-wrapped in
  `Layout::top_down(Align::LEFT)` (already commented as intentional, to
  avoid each bullet line centering individually) — correct on its own
  terms, but produces the same centered-frame/full-width-content mismatch
  as the main menu.

**Fix**: give the guide content on both surfaces a single, consistent
contained width instead of letting it sprawl full-window — e.g. a new
presentation constant (`HOW_TO_PLAY_CONTENT_WIDTH`, same rationale
`ui.rs::HUD_WIDTH`/notebook's own width constant already establish) applied
via `ui.allocate_ui_with_layout` (or an equivalent max-width child `Ui`) so
the bulleted block reads as one contained column — text left-aligned
*within* that fixed width for readability, but the block itself sitting
under the narrower menu controls rather than stretching past them. Pick a
width that comfortably fits the longest existing bullet line without
excessive wrapping; don't undersize it just to match the seed field's own
(much narrower) width.

Additionally, add a touch more separation between the "start a run"
cluster (title/seed/New Run) and the "meta/reference" cluster (unlocks
line, guide toggle) — e.g. a `hairline()` or extra spacing before the
unlocks line — so the two read as distinct groups rather than one
undifferentiated stack. Minimal change: grouping, not new chrome.

**Additional acceptance criteria** (append to the section below, don't
replace it):

- Guide content on both surfaces renders at one consistent contained width
  — no full-window-wide text sitting directly under a narrow centered menu
  column.
- A visible separation exists between the run-starting controls and the
  meta/reference cluster below them on the main menu.
- User live-check (not the sandboxed build/clippy/test pass) confirms the
  "crowded and mixed" impression is resolved.

Out of scope for this amendment: the unlocks line's *wording*/*mechanic*
("clear worlds to earn more starting species") is a separate, real finding
from the same review — tracked as task 192 (depends on task 158), not
fixed here. Don't change `text::NO_UNLOCKS_YET`'s content in this task,
only whatever container it renders inside.

---

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
  - **Amendment pass (2026-09-02)**: added `ui::HOW_TO_PLAY_CONTENT_WIDTH`
    (600.0, doc comment justifies it against `HOW_TO_PLAY_SECTIONS`'s
    longest bullet lines) and wrapped both surfaces' guide content in
    `ui.allocate_ui_with_layout(egui::vec2(HOW_TO_PLAY_CONTENT_WIDTH, ..),
    Layout::top_down(Align::LEFT), ..)` so it reads as one contained,
    left-aligned block instead of full-panel-width text. `menu.rs` also
    wraps this in its own `vertical_centered` (its call site sits outside
    the menu's title/seed/button cluster); `screens.rs`'s call site was
    already inside `interstitial`'s outer `vertical_centered`, so only the
    width constraint needed adding there. Main-menu grouping: replaced the
    `add_space(16.0)` before the unlocks line with `add_space(32.0)` —
    deliberately plain spacing, not `hairline()`, since that helper paints
    edge-to-edge across the enclosing `vertical_centered` block's full
    panel width (the same narrow-column/full-width mismatch this amendment
    removes elsewhere), and the amendment names extra spacing as an
    acceptable alternative. Same `cargo build`/`clippy -D warnings`/`fmt
    --check`/`test` validation re-run clean after these changes. Still no
    GUI live check possible in this sandbox — `HOW_TO_PLAY_CONTENT_WIDTH`
    (600.0) and the divider spacing (32.0) are both unverified numeric
    guesses; status stays `IN_PROGRESS` pending the user's own live check.
  - **Amendment 2 pass (2026-09-02)**: fixed the hanging-indent defect by
    rendering each bullet as `ui.horizontal_top(|ui| { ui.label(BULLET);
    ui.add(egui::Label::new(line).wrap()); })` instead of a single
    `ui.label(format!("{bullet} {line}"))`, at both call sites (`menu.rs`,
    `screens.rs`). `egui::Grid` was investigated per the task's suggestion
    (egui 0.35 per `Cargo.lock`; no existing `egui::Grid` usage anywhere in
    `src/` to match) but wasn't needed: `notebook.rs` already has a working
    precedent for this exact defect (task 187's comment on line ~944,
    "plain label inside `ui.horizontal` doesn't wrap") — a two-widget row
    where the text widget carries `.wrap()` gets a real hanging indent for
    free, because egui lays out a wrapped label's continuation lines from
    that widget's own x-origin (right of the bullet glyph), not the row's.
    Used `ui.horizontal_top` (`Layout::left_to_right(Align::TOP)`), not
    plain `ui.horizontal` (`Align::Center`): a `horizontal` row grows tall
    once the text wraps to multiple lines, and center-alignment would float
    the bullet glyph beside the wrapped block's middle line instead of
    pinning it to the first line — this only shows up once a line actually
    wraps, so it's easy to miss testing against short lines alone. Reasoned
    through the wrap width without a GUI: both call sites nest this inside
    the existing `HOW_TO_PLAY_CONTENT_WIDTH` (600.0)-bounded child `Ui`
    (`allocate_ui_with_layout`/`ScrollArea`), so `ui.available_width()` at
    the text label correctly reflects 600 minus the bullet glyph's width
    minus one `item_spacing.x` — the Controls "P: toggle continuous
    advancement..." line (`src/text.rs`, ~123 chars) is well past that
    budget and should wrap, with its continuation aligned under "toggle",
    not under "▪"; not visually confirmed. Added `ui::
    HOW_TO_PLAY_BULLET_SPACING` (3.0, doc-commented) as `ui.add_space(...)`
    between consecutive bullet lines within a section only — gated so it
    never fires after a section's last bullet, leaving `hairline()`'s
    section-to-section spacing untouched as instructed. Split the
    "Metabolism and temperature" section's two combined-fact lines into
    four single-fact lines in `src/text.rs`; note for review: the split
    restores the elided verb for the second half of each original sentence
    ("Predator draws energy from neighboring organisms.", "Chemolithotroph
    draws energy from local toxicity.") so each line reads as a complete
    sentence — same facts, not a byte-for-byte clause split. Validation
    (`cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt
    --check`, `cargo test` full suite) all clean after these changes. No
    GUI live check performed — same sandbox constraint as every prior pass;
    the indent fix is verified by code mechanism and the task-187 precedent,
    not visually. Amendment 2's own acceptance criteria require a user
    live-check ("confirms the wrapped-line/orphan-line impression... is
    resolved"), which this sandbox cannot satisfy — status stays
    `IN_PROGRESS` pending that check.

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
