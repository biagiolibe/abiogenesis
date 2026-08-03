# Task 029 — Stable tag identifiers and readable species names

> **ID**: `029`
> **Category**: UX
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-03 playtest

---

## 🎯 Objective

A 2026-08-03 playtest found the current presentation confusing to track: tags are rendered as identical colored dots (`●`) with no stable label a player can refer back to ("the greenish one" isn't a durable mental handle), and species are only ever "species 0", "species 1", etc.

**Design constraint, non-negotiable**: GDD §11 requires tags to stay "nameless glyphs/colors, learned empirically" — the whole deduction pillar depends on tags carrying no semantic hint. This task does **not** give tags descriptive names (e.g. "Toxin", "Symbiote") — that would leak what a tag does before the player deduces it. Instead:

- **Tags** get a stable, opaque, more-distinguishable identifier alongside the existing color — e.g. a Greek letter or symbol keyed deterministically off `TagId`, purely so a player can say "tag α" instead of "that dot" without the label meaning anything.
- **Species** get a genuinely more legible display identity — species aren't secret (metabolism and temperature range are already shown in the HUD/catalog), so there's no design constraint against making them easier to refer to.

---

## 📋 Acceptance Criteria

- [ ] Every place a tag is currently rendered as just a colored `●` (`notebook.rs`'s hypothesis grid, catalog panel; anywhere else tags appear) also shows a stable opaque identifier — e.g. a Greek letter cycling deterministically with `TagId.0` (`α, β, γ, δ, ε, ...`), not a random or semantic label.
- [ ] The identifier is presentation-only — no changes to `TagId`, `TagMatrix`, or any simulation-affecting type; purely a `notebook.rs`/`render.rs`-side lookup from `TagId` to a glyph string.
- [ ] Every place a species is currently rendered as "species N" (HUD population list, seed selector, notebook catalog) gets a more legible display alongside or instead of the raw number — decide and document whether that's a short generated name (e.g. from a small fixed word list, deterministic per `SpeciesId` so it never changes across frames) or just a cleaner visual treatment of the existing number; either is acceptable as long as it's an improvement over bare "species N" and doesn't require the player to memorize numeric IDs.
- [ ] Newly spliced species (task 025) get a legible identity too, not just the starting-palette ones — verify this doesn't require touching `apply_splice`'s simulation logic, only how the resulting species is displayed.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `hypothesis_grid`, `catalog_panel` — where tags currently render as bare colored dots |
| `src/ui.rs` | `hud_panel`, `splice_panel` — where species currently render as "species N" |
| `src/render.rs` | Precedent for deterministic per-id derivation (`SPECIES_HUE_STEP` golden-angle hue) — the tag-letter mapping should follow the same "derive from id, no stored state" approach |

---

## 🧩 Technical Context

Both are pure display-layer lookups, `TagId.0`/`SpeciesId.0` → some string, computed on the fly wherever needed — no new resources, no persisted naming state, consistent with how `tag_color`/species hue already work (deterministic functions of the id, not stored config). Keep it that simple; don't build a naming *system* (no player-editable names, no persistence) unless a later playtest specifically asks for one.

---

## 🔨 Suggested Implementation

1. `notebook.rs`: a small `tag_glyph(tag: TagId) -> &'static str` (or similar) returning a Greek letter from a fixed array, indexed `tag.0 as usize % ARRAY_LEN`. Use it everywhere `TAG_GLYPH` (`"●"`) is currently rendered, alongside the existing `tag_color`.
2. Pick a species display approach (generated short name vs. cleaner number treatment) and apply it consistently in `hud_panel`'s population list, seed selector, `splice_panel`'s source-species picker, and `catalog_panel`.
3. Manual verification: open the notebook, confirm tags show a stable letter+color pair that doesn't change between frames or reseeds' tag reassignment (a reseed still gets a **new** letter mapping if `TagId`s reshuffle — that's fine and expected, only the *within-a-run* stability matters); confirm species show something more legible than "species N" everywhere they appear, including a freshly-spliced one.

---

## ⚠️ Constraints and Caveats

- Do not give tags any label that hints at their effect — a Greek letter, a number, an abstract symbol are fine; a word implying "hot", "aggressive", "helpful", etc. is not.
- Don't add player-editable naming (a text-input rename feature) in this task — that's a larger UX surface (and `input.rs` notes `q` is kept free for "future text input", suggesting that's intentionally deferred); keep this task to deterministic, non-editable display identifiers.

---

## 🔗 Dependencies

- **Depends on**: 021 (hypothesis grid + catalog), 025 (Splice, for the new-species display case)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/029-tag-identifiers-species-names.md)"$'\n\nExecute this task in the current project.'
```
