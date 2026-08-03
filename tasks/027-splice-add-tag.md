# Task 027 — Splice: add a real "Add tag" option, not just "Swap"

> **ID**: `027`
> **Category**: Bugfix / Feature
> **Priority**: 🟡 P2
> **Estimate**: ~45min
> **Assigned to**: unassigned
> **Session**: 2026-08-03 playtest

---

## 🎯 Objective

Task 025's `Splice` action only implements `SpliceEditChoice::SwapTag { old, new }`: replacing one existing tag with another. Task 025's own acceptance criteria said "swap/**add** one tag", and GDD §5.3 caps species at 1–3 tags — but a species with fewer than 3 tags currently has no way to grow toward that cap without sacrificing an existing tag first. A 2026-08-03 playtest hit this directly: spliced species 1 (2 tags) to pick up tag 4, and had to remove tag 3 to do it, even though the species had room for a third tag.

Add a genuine "Add tag" edit that appends a tag without removing one, available only when the source species has fewer than 3 tags (GDD §5.3's cap) — alongside the existing "Swap tag", not replacing it (removing a tag entirely, e.g. going from 2 tags to 1, is arguably also a real gap, but is explicitly out of scope for this task; note it as a follow-up if it comes up again in playtesting).

---

## 📋 Acceptance Criteria

- [ ] `SpliceEditChoice` gains a third variant, e.g. `AddTag { tag: Option<TagId> }`, alongside the existing `SwapTag` and `ShiftTempOptimum`.
- [ ] The HUD's Splice editor (`ui.rs::splice_panel`) offers "Add a tag" as a third radio choice, but only meaningfully selectable/usable when the selected source species has `tags.len() < 3` — when the species already has 3 tags, either hide this option or show it disabled with a short explanation (whichever is less code; must not panic either way).
- [ ] `apply_splice` (`input.rs`), on a complete `AddTag` selection, clones the source species and pushes the new tag onto its `tags` vec (not replacing anything), then applies the existing new-species-append + budget-spend logic identically to `SwapTag`.
- [ ] Applying `AddTag` to a species that already has 3 tags (e.g. a stale draft selection from before switching source species) does nothing, same "silently no-op on an invalid/incomplete draft" pattern the other `Splice` edits already follow — don't let `tags.len()` exceed 3.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `SpliceEditChoice` enum, `splice_panel` — gains the "Add a tag" UI branch |
| `src/input.rs` | `apply_splice` — gains the `AddTag` match arm |

---

## 🧩 Technical Context

This is a small, additive change to task 025's existing scaffolding — no new resources, no new systems, same `SpliceDraft`/`apply_splice` flow. The only real design point is the `tags.len() < 3` gating: check it in the UI (so the player isn't offered an edit that can't apply) *and* in `apply_splice` (so a stale/manipulated draft can't bypass the cap) — same defense-in-depth the other actions already use (e.g. `Cull`'s occupancy check happens before the budget check, not instead of a UI-side affordance check).

---

## 🔨 Suggested Implementation

1. `ui.rs`: add `SpliceEditChoice::AddTag { tag: Option<TagId> }`.
2. `splice_panel`: add the third radio option; when selected and a source species with `< 3` tags is picked, show radio buttons for `world.active_tags` filtered to tags the species doesn't already carry (mirrors the existing "Add tag" list already rendered inside `SwapTag`'s UI — likely reusable almost as-is).
3. `apply_splice`: add the `AddTag { tag: Some(tag) }` arm — `new_species.tags.push(tag)`, guarded by `new_species.tags.len() < 3` before pushing.
4. Unit tests (mirror task 025's existing `input.rs` test style): adding a tag to a 2-tag species appends without removing anything; attempting to add to a 3-tag species does nothing.
5. Manual verification: splice a 1- or 2-tag species with "Add a tag", confirm the new species has one more tag than the source with nothing removed; confirm the option is unavailable/inert on a 3-tag source.

---

## ⚠️ Constraints and Caveats

- Don't add a "remove tag without adding" edit here — out of scope, note as a follow-up if it comes up again.
- Keep `SwapTag` exactly as it is; this task adds a sibling option, not a replacement.

---

## 🔗 Dependencies

- **Depends on**: 025 (Splice action)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/027-splice-add-tag.md)"$'\n\nExecute this task in the current project.'
```
