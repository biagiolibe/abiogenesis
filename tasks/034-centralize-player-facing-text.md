# Task 034 — Centralize player-facing text behind a single `text` module

> **ID**: `034`
> **Category**: Architecture
> **Priority**: 🟢 P3
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, follow-up to task 030's HUD reorganization

---

## 🎯 Objective

Right now every player-facing string — HUD labels, section headings, action tooltips, the observation log's event sentences, the hypothesis-grid/catalog labels — is an inline literal or `format!()` scattered across `ui.rs` and `notebook.rs`. This task centralizes all of it behind a single module (`src/text.rs`), so there is one place that owns "what the player reads," without building any actual localization/loader machinery yet — strings stay hardcoded in English, just no longer inline at the call site.

This is prep work, not a real i18n system: no language switching, no resource files, no `Locale` resource. The payoff is that a future localization pass touches one module instead of `ui.rs` + `notebook.rs` + wherever else text creeps in next.

---

## 📋 Acceptance Criteria

- [ ] A new `src/text.rs` module exists, exposing functions (not raw `const`s, wherever the string is parameterized — e.g. `era_tick_label(era: u32, tick: u32) -> String`) for every player-facing string currently inline in `ui.rs` and `notebook.rs`.
- [ ] `ui.rs`'s HUD strings are centralized: headings ("Action", "Population", "Seed palette", etc.), the action-icon tooltips (name + cost + description per `ActionMode`), the budget-bar tooltip, the Splice panel's labels ("Swap a tag", "Add a tag", "Shift temperature optimum", the "(source already has 3 tags)"/"(pick a source species first)" hints), the bottom keyboard-shortcut hint.
- [ ] `notebook.rs`'s player-facing strings are centralized: the observation-log heading/empty-state, event sentences (e.g. `"species {} went extinct"` and the salient-death message from task 026), the hypothesis-grid/catalog headings and cell labels.
- [ ] Strings that are actually **data**, not UI copy, stay where they are: `species_label`/`tag_glyph` (task 029) already generate per-entity display names/glyphs from `SimWorld` state — those aren't static player-facing copy and are out of scope here. Boundary judgment call: if it's a fixed sentence/label independent of which species/tag it's about, centralize it; if it's derived from world data, leave it.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt --check` clean.
- [ ] `cargo test` still passes (no behavior change — this is a pure refactor).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/text.rs` | **New** — the centralized module this task creates. |
| `src/ui.rs` | `hud_panel`, `splice_panel`, `action_icon_row` — ~23 inline string call sites to migrate. |
| `src/notebook.rs` | Observation log, hypothesis grid, catalog panels — ~16 inline string call sites to migrate, plus the two `format!` event-sentence builders. |

---

## 🧩 Technical Context

<!-- TODO: add relevant code snippets and file paths -->

- **Current behavior**: strings are literals or `format!()` calls directly inside `ui.rs`/`notebook.rs`'s egui-drawing functions.
- **Desired behavior**: those call sites call into `text::some_function(...)` instead, which returns the same `String`/`&'static str` it did before — output is byte-for-byte identical, this is purely where the text *lives*.

---

## 🔨 Suggested Implementation

1. Create `src/text.rs`, add `mod text;` to `lib.rs` (or wherever modules are declared).
2. Work through `ui.rs` top to bottom, moving each literal/`format!` into a named function in `text.rs`, replacing the call site.
3. Repeat for `notebook.rs`.
4. Group related strings under comment banners in `text.rs` (e.g. `// HUD — action icons`, `// Notebook — observation log`) for readability, since the module will be a flat list of otherwise-unrelated functions.
5. Run `cargo clippy -- -D warnings`, `cargo fmt`, `cargo test`.

---

## ⚠️ Constraints and Caveats

- **No localization infrastructure**: no `Locale` enum, no resource files, no `fluent`/`rust-i18n` dependency. That's explicitly deferred until there's a real second-language target.
- Don't touch `species_label`/`tag_glyph`/tag-color logic (task 029) — those are data-driven, not static copy.
- Don't touch code comments or internal identifiers (`SpeciesId`, `ActionMode`, log/warn messages meant for developers, not players).
- Pure refactor: no visible behavior change. If a screenshot taken before and after this task would differ, something went wrong.

---

## 🔗 Dependencies

- **Depends on**: 030 (HUD reorganization — the tooltip text this task centralizes)
- **Blocks**: none (any future localization work would build on this)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/034-centralize-player-facing-text.md)"$'\n\nExecute this task in the current project.'
```
