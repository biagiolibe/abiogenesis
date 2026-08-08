# Task 065 — Species list vertical, metabolism glyph, seed relocated

> **ID**: `065`
> **Category**: UI
> **Priority**: 🟢 P3
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-08/09 (playtest feedback on task 064's sidebar redesign, raised directly by the user)

---

## 🎯 Objective

Three small corrections to task 064's sidebar console, raised directly by
the user after playing with it:

1. **Species list should scroll vertically, not horizontally.** Task 064
   implemented the redesign mockup's horizontal chip strip
   (`redesign/sidebar-full.svg`'s "Banca genomi" row) literally, but in
   practice the hidden scrollbar (needed to avoid overlapping the clickable
   chip row, see task 064's caveats) made overflow undiscoverable — it
   required a dedicated static `›` cue just to signal more content existed.
   The Biosphere section right above it already solves the same "N items,
   fixed height" problem with a vertical `ScrollArea` and a
   `SCROLL_FOR_MORE` hint; Species should use the same pattern instead of
   inventing a second one.
2. **Show each species' metabolism in the Species list.** Metabolism is a
   *readable* trait (GDD §5.3, "no guesswork needed"), but before this task
   it was only visible in the notebook's species catalog (`tab`) — which a
   fresh player hasn't opened yet at their very first placement (the
   notebook is explicitly something "you do after the first placement",
   per the user). A player picking a species to seed had no way to tell
   what it *was* without leaving the seed-selection panel entirely.
3. **Move the seed number out of the header, down to the keyboard-hints
   footer.** The redesign mockup's header (`sidebar-full.svg` lines 4-7)
   has no seed line at all — title, era/tick, status. The seed is
   debugging/reproducibility information, not something a player consults
   moment-to-moment; it reads better grouped with the other
   secondary/reference lines at the bottom (`r reseed`'s own hint is right
   there) than sitting in the primary at-a-glance header.

---

## 📋 Acceptance Criteria

- [x] Species section (`src/ui.rs::hud_panel`) uses
      `egui::ScrollArea::vertical()` with a fixed `max_height` and a new
      `SPECIES_VISIBLE_ROWS` constant (mirroring `BIOSPHERE_VISIBLE_ROWS`),
      past which `text::SCROLL_FOR_MORE` appears — same pattern as
      Biosphere, not a second bespoke affordance.
- [x] The horizontal chip strip (`species_chip`, `ScrollArea::horizontal`,
      the `MORE_SPECIES_GLYPH`/`MORE_SPECIES_GLYPH_WIDTH` overflow-cue
      machinery it required) is removed entirely, not kept as a dead
      alternate path.
- [x] A new `metabolism_glyph(Metabolism) -> &'static str` (`src/render.rs`,
      next to `species_label`/`species_color`) renders each row prefixed
      with an indicative (not opaque — metabolism isn't a design secret,
      unlike tags) glyph: ☀ Photolithic, ⚔ Predator, ♻ Decomposer. Verified
      in playtest against the notebook catalog that the glyph shown for
      each species matches its actual metabolism.
- [x] `text::seed_line` moves from the header block to the bottom
      `Layout::bottom_up` block alongside `KEYBOARD_HINT_PRIMARY`/
      `KEYBOARD_HINT_SECONDARY` — no change to `text::seed_line` itself,
      only where `hud_panel` calls it.
- [x] `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D
      warnings`, `cargo fmt -- --check` all clean.
- [x] Manual playtest (`cargo run`) confirming: Species renders as a
      vertical list matching Biosphere's visual pattern, each row shows the
      correct metabolism glyph (cross-checked against the notebook
      catalog), and the header no longer shows the seed number while the
      footer does.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `hud_panel` (header/footer reshuffle, Species section rewrite), `species_row` (replaces `species_chip`), `console_row_height` (renamed from `biosphere_row_height`, now shared by both lists), `SPECIES_VISIBLE_ROWS`. |
| `src/render.rs` | New `metabolism_glyph`, next to `species_label`/`species_color`. |
| `src/text.rs` | `MORE_SPECIES_GLYPH` constant removed (dead once the chip strip is gone). |

---

## 🧩 Technical Context

- **Depends on task 064**: this corrects two of that task's specific
  choices (horizontal strip, header seed placement) after seeing them in
  actual use, rather than reverting the redesign wholesale — the
  hairline-divided monospace panel, discrete dot indicators, and narrative
  objective styling all stay as they were.
- `console_row_height(ui)` (measuring `TextStyle::Body` height +
  `item_spacing.y` from the panel's own style, task 064's fix for egui's
  `ScrollArea` `min_scrolled_size` floor) is reused as-is for Species —
  both lists are the same kind of row, so one measurement function serves
  both `SPECIES_VISIBLE_ROWS` and `BIOSPHERE_VISIBLE_ROWS`.

---

## ⚠️ Constraints and Caveats

- **Style**: no inline string literals for player-facing text — reused
  `text::SCROLL_FOR_MORE` rather than adding a Species-specific variant,
  since the wording ("scroll for more") doesn't need to differ per list.
- **Don't** change `SelectedSpecies`-setting behavior — clicking a Species
  row still just sets `SelectedSpecies`, identical to the old chip's click
  handler.

---

## 🔗 Dependencies

- **Depends on**: 064 (sidebar console redesign)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/065-species-list-vertical-metabolism-seed-relocation.md)"$'\n\nExecute this task in the current project.'
```
