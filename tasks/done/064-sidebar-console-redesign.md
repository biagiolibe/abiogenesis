# Task 064 — Sidebar console redesign

> **ID**: `064`
> **Category**: UI
> **Priority**: 🟡 P2
> **Estimate**: ~5-6h (structural rewrite of `hud_panel` plus several new small components)
> **Assigned to**: unassigned
> **Session**: 2026-08-08 (from `redesign/abiogenesis-sidebar-redesign.md`)

---

## 🎯 Objective

Implement `redesign/abiogenesis-sidebar-redesign.md` in full (that document is self-contained design context — read it alongside this task, including its two SVG mockups `redesign/sidebar-full.svg` and `redesign/sidebar-censimento-scaled.svg`). Summary of what changes in `src/ui.rs::hud_panel` (`src/ui.rs:240-357`):

1. **One continuous panel** with thin hairline separators between sections, instead of today's four independently bordered `group_frame` boxes (`src/ui.rs:548-553`).
2. **Monospace font** across the whole panel ("lab console / field journal" register) — a deliberate, scoped exception, not a font change anywhere else in the game.
3. **Diegetic English labels**: `Action` → `Moves`, `Population` → `Biosphere`, `Seed palette` → `Species`, `Objective` → `This world wants` (translated from the source doc's Italian mockup text, then revised again directly with the user — the first English pass, `Intervene`/`Census`/`Gene bank`/`Directive`, read as too formal/managerial; this lighter set is the one to implement).
4. **Discrete tick/dot indicators** replacing continuous `egui::ProgressBar`s for small countable resources: the action-budget bar (`src/ui.rs:287-291`) and the objective's era-progress bar (`src/ui.rs:525`, for `Coexistence`/`SurviveIn` — see the open question below on `TriggerBloom` and large era counts).
5. **The objective as narrative text**: the active objective's description rendered as an italicized, quoted sentence in a distinct typeface from the rest of the (monospace) panel — the *only* place in the panel that breaks the monospace rule. Used nowhere else.
6. **Biosphere and Species scale to N species**: Biosphere becomes a fixed-max-height, single-line-per-species list with internal scrolling past ~4-5 visible rows (`redesign/sidebar-censimento-scaled.svg`, including its bottom fade-and-scroll-hint treatment); Species becomes a horizontally scrollable chip strip instead of a multi-row grid.
7. **Trend indicator** in Biosphere: consumes task 063's per-species `Rising`/`Falling`/`Stable` classification (▲ green / ▼ red / ▬ gray, per the mockup) — this task is blocked on 063 landing first.

---

## 📋 Acceptance Criteria

### Panel structure

- [x] `hud_panel` (`src/ui.rs:240-357`) restructured to a single `egui::Frame`/`Panel` with `egui::Separator`-style hairlines (thin, low-contrast) between sections instead of `group_frame`'s bordered boxes. `group_frame` (`src/ui.rs:548-553`) is removed once nothing calls it, or repurposed if the splice sub-panel (`src/ui.rs:604-691`) still needs visual grouping — use judgment, but the four *main* sections (Moves/Biosphere/Species/This world wants) must read as one continuous console, per the mockup.
- [x] A monospace `egui::TextStyle` (or explicit `FontId::monospace(..)`) is applied panel-wide. Check what monospace font `configure_fonts` (`src/ui.rs:173-188`) currently has available (egui ships a default monospace family, "Hack" — confirm it covers everything the panel needs, e.g. the `●`/`▲`/`▼`/`▬` glyphs used for species swatches and trend indicators; if any are missing, extend the existing `DejaVuSans` fallback pattern to the monospace family rather than inventing a second font-loading mechanism).

### Labels

- [x] `text::HEADING_ACTION` → `"Moves"`, `text::HEADING_POPULATION` → `"Biosphere"`, `text::HEADING_SEED_PALETTE` → `"Species"`, `text::HEADING_OBJECTIVE` → `"This world wants"` (`src/text.rs`, exact constant names may already differ — grep for the current `HEADING_*` constants and rename/reword in place, don't introduce parallel new ones).
- [x] `player_guide.md` and any other player-facing doc referencing the old panel names ("Population panel", "Seed palette") updated to match, so in-game and manual terminology stay consistent.

### Discrete indicators

- [x] Action budget (`src/ui.rs:281-291`): replace the `egui::ProgressBar` with `total` (`config.time.point_budget_per_era`, small integer, GDD baseline `3`) small filled/empty dots or tick marks, matching the mockup's rounded-rect ticks. Keep the existing hover tooltip (`text::BUDGET_HOVER`).
- [x] Objective era-progress (`src/ui.rs:525`, fed by `eras_progress`, `src/ui.rs:534-542`): for `Coexistence`/`SurviveIn`, replace the bar with discrete dots sized to `eras_required`. **Open question to resolve during implementation**: `eras_required` isn't bounded as tightly as the action budget (worlds can plausibly require several eras) — decide a reasonable cap past which dots degrade gracefully (e.g. switch back to a compact numeric "X / Y eras" once `eras_required` exceeds some small constant, or scale dot size down) rather than rendering an unbounded row. Document whatever threshold is chosen as a named constant, not a hardcoded literal in the render call. `TriggerBloom`'s binary triggered/not-yet-triggered state (`src/ui.rs:505-516`) doesn't need dots at all — a single state label is enough, unchanged from today's `BLOOM_TRIGGERED`/`BLOOM_NOT_TRIGGERED` text.
  - **Resolved**: added `ERA_PROGRESS_DOT_CAP: u32 = 8` and an `EraProgressDisplay { Dots, Numeric }` decision function (`era_progress_display`, covered by two unit tests) — at or under the cap, dots; past it, `sustained_progress_bar_text`'s existing numeric "X / Y eras" readout. `TriggerBloom` kept as a plain state label, no dots, as this note anticipated. `BLOOM_TRIGGERED` (the text constant) turned out to be genuinely dead — `objective_panel`'s `progress.satisfied` early-return always wins before it could be read — and was deleted rather than kept as a stub.

### Objective narrative styling

- [x] The active objective's description line (`text::coexistence_objective_line`/`survive_in_objective_line`/`trigger_bloom_objective_line`, assembled in `objective_panel`, `src/ui.rs:451-526`), under the `"This world wants"` heading, renders in quotes, italicized, visually distinct from the panel's monospace body text.
- [x] **Open decision, resolve during implementation**: the mockup uses a literal serif font (`Georgia, serif`) for this line, which isn't bundled with the game today (only `DejaVuSans` is, as a glyph-coverage fallback, `src/ui.rs:128-136`). Adding a genuine italic serif font means embedding a new font asset — weigh that against approximating the effect with `egui::RichText::italics()` on the existing monospace/proportional font (egui applies `italics` as a formatting flag; confirm during implementation whether it visibly slants the currently-loaded fonts or is a no-op without an italic font variant registered). Prefer the no-new-asset approximation unless it reads as clearly insufficient — this is deliberately a small narrative accent (redesign doc: "da usare con parsimonia"), not worth a font-licensing/embedding effort if the cheap version reads fine. If a font is added, license it compatibly with `assets/fonts/DejaVu-LICENSE.txt`'s precedent and document the license file the same way.
  - **Resolved**: went with the no-new-asset approximation — `egui::RichText::new(...).italics().family(egui::FontFamily::Proportional).color(OBJECTIVE_NARRATIVE_COLOR)`. Confirmed in playtest it reads clearly as visually distinct (a warm off-white, non-monospace, italic-slanted line) from the surrounding monospace body text, without embedding a new font or license file.

### Biosphere scaling

- [x] Biosphere list (`src/ui.rs:300-316`) becomes single-line-per-species rows inside an `egui::ScrollArea::vertical()` with a fixed `max_height` (~4-5 rows worth, per the mockup), not unbounded panel growth.
- [x] Past the visible row count, a bottom fade/gradient or a "scroll for more" hint communicates there's more content (mockup: gradient fade + `"scroll for more"` text) — approximate with whatever egui affordance is cheapest (a simple hint label under the scroll area is an acceptable simplification if a true CSS-style fade proves fiddly in egui's immediate-mode painter).
- [x] Verify with a world seeded to 6+ coexisting species (matches the second mockup) that the list scrolls correctly and the rest of the panel (Species, This world wants) stays fixed below it, not pushed off-screen.

### Species scaling

- [x] Seed palette / Species (`src/ui.rs:319-329`) becomes a horizontally scrollable strip of chips (`egui::ScrollArea::horizontal()`), one chip per species, replacing the vertical `ui.radio_value` list. Selecting a chip sets `SelectedSpecies` exactly as the old radio buttons did — no behavior change, only layout.
- [x] Verify with 6+ species that the strip scrolls horizontally rather than wrapping or growing the panel's height.

### General

- [x] `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt -- --check` all clean.
- [x] Manual playtest (`cargo run`) confirming: panel reads as one continuous console (no boxed sections), all four renamed labels appear correctly, action budget and era-progress render as discrete indicators, the objective line is visually distinct, Biosphere scrolls past 4-5 species, Species scrolls horizontally past however many chips fit `HUD_WIDTH`.
  - **Caveats found and fixed during verification**: (1) `HUD_WIDTH` raised `300.0 → 340.0` — monospace text is measurably wider than the proportional-font-tuned original value, which clipped the Biosphere row's trend glyph. (2) The horizontal `ScrollArea`'s scrollbar rendered directly over the chip row (it's exactly one row tall with no room reserved below), turning edge clicks into scroll-drags; fixed with `.scroll_bar_visibility(AlwaysHidden)`. (3) `egui::ScrollArea` silently floors its scrolled axis at `min_scrolled_size` (`64.0`pt, egui 0.35 `scroll_area.rs`) — confirmed by instrumenting `ui.clip_rect()` at runtime. The Biosphere row height is measured from the panel's own style (`biosphere_row_height`, ~18.1pt at production settings) rather than hardcoded, so `BIOSPHERE_VISIBLE_ROWS`'s `max_height` and the `SCROLL_FOR_MORE` threshold stay consistent with each other if font/spacing change. (4) Hiding the chip strip's scrollbar removed the only cue that more chips exist off-strip; added a static `›` (`text::MORE_SPECIES_GLYPH`) at the strip's right edge, matching `redesign/sidebar-full.svg`, with `MORE_SPECIES_GLYPH_WIDTH` reserved so it doesn't render outside the panel's clip rect.
- [x] `redesign/abiogenesis-sidebar-redesign.md` — no changes needed to the doc itself; it's the spec this task implements.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `hud_panel` (structural rewrite), `group_frame`, `objective_panel`, `eras_progress`, `species_stats`, `action_icon_row` — most of this task lands here. |
| `src/text.rs` | `HEADING_*` constants (relabeling), any new label/hint strings the redesign needs. |
| `src/config.rs` | If a dot-count cap constant is needed for era-progress, it belongs here (`ObjectiveConfig` or similar), not hardcoded. |
| `player_guide.md` | Panel names referenced in prose. |
| `redesign/abiogenesis-sidebar-redesign.md` | Source spec — read in full before starting. |
| `redesign/sidebar-full.svg`, `redesign/sidebar-censimento-scaled.svg` | Visual reference mockups. |
| `assets/fonts/DejaVuSans.ttf`, `assets/fonts/DejaVu-LICENSE.txt` | Precedent for how a font gets bundled, if the directive-styling decision above lands on adding one. |

---

## 🧩 Technical Context

- **Current behavior**: four independently bordered `group_frame` sections, continuous `egui::ProgressBar`s for the action budget and objective progress, vertical radio-button list for species selection, unbounded-height population list — all documented in `redesign/abiogenesis-sidebar-redesign.md`'s "Contesto" section.
- **Desired behavior**: see the Decisions section of that document (§1-§7) and the two SVG mockups — this task file's Acceptance Criteria is a translation of those decisions into concrete code changes, not a paraphrase; when in doubt, defer to the source document and mockups over this file's wording.
- Depends on task 063 landing first: the Biosphere row's trend indicator (▲/▼/▬) needs 063's `Rising`/`Falling`/`Stable` resource and `text::population_line` signature change to exist before this task's Biosphere rendering can consume them.

---

## ⚠️ Constraints and Caveats

- **Style**: all new/renamed player-facing strings go through `text.rs` (task 034's convention) — no inline `format!`/string literals in `ui.rs`.
- **Pillar 3 scoping**: the monospace-console aesthetic and the directive's typographic accent are explicitly scoped to this one panel per the redesign doc ("Fuori scope... Qualsiasi asset grafico o illustrativo") — don't let font/styling choices bleed into the grid rendering or other UI (notebook window, menu screens), which stay as they are.
- **No magic numbers**: any new tunable (dot-count cap, scroll max-height in rows) should be a named constant near its use, matching the existing `HUD_WIDTH`/`ISOLATION_HINT_DURATION_TICKS` pattern in `ui.rs`, even where it isn't a `SimConfig` simulation coefficient (presentation constants already live as local `const`s in this file, e.g. `NODE_RADIUS` in `notebook.rs`).
- **Don't** change any simulation behavior, action costs, or objective logic — this is a rendering-layer task end to end.

---

## 🔗 Dependencies

- **Depends on**: 063 (population trend indicator must exist before Biosphere can render it)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/064-sidebar-console-redesign.md)"$'\n\nExecute this task in the current project.'
```
