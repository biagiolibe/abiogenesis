# Task 185 — Era-reveal card: remove species-identity color from the genome-diff swatches

> **ID**: `185`
> **Category**: UI / Bugfix
> **Priority**: 🟡 P2 (corrective — Phase 2 residual)
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-29 scoping

---

## 🎯 Objective

The era-reveal card (`src/screens.rs:237-358`) is the one surface outside
180/181 that task 151 actually touched (a `.corner_radius(0.0)` edit,
`screens.rs:262`) — but it still violates `VISUAL_STYLE_GUIDE.md` §1 rule 3
("color = state, never identity"): its evolution-entry rows paint a
14×14 `rect_filled` swatch with `species_color(entry.parent)`/
`species_color(entry.child)` (`screens.rs:299-311`) next to each species'
name. This is the exact pattern already corrected elsewhere by tasks
180/181 (Biosphere rows, Catalog icons) — a colored identity swatch where
the design calls for a neutral shape-coded icon and name-as-identity. This
gap was not in `VISUAL_STYLE_GUIDE.md`'s original §8 tracked-gaps list
(found by a direct code audit, 2026-08-29) — it's now recorded there and
closed by this task.

Read `VISUAL_STYLE_GUIDE.md` first — §1 rule 3, §4 (organism/metabolism
iconography — this task reuses the same shared icon-painter helper 180/181
introduce).

---

## 📋 Acceptance Criteria

- [x] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [x] **Replaced the species-colored swatch with a neutral metabolism
      icon**: the two `rect_filled(..., species_color(...))` calls now call
      `ui::paint_metabolism_icon` (the shared helper 180/181 already
      introduced), keyed by `world.species[entry.parent/child.0 as
      usize].metabolism`, painted in `ICON_INK_SELECTED` (the actual
      constant name in code for the guide's "ORGANISM_INK" token, `ui.
      rs:1506`). Species identity stays carried by the adjacent name
      labels, unchanged.
- [x] `species_color` import (`screens.rs:20`) — removed; it had no other
      use left in this file.
- [-] Live visual check — skipped per explicit user instruction for this
      task; `cargo build`/`clippy`/`fmt`/`test` all clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/screens.rs` | `era_reveal_screen_ui` (`237-358`), specifically the swatch block at `299-311`; `species_color` import at `20`. |
| `src/render.rs` | Shared icon-painter helper (introduced by 180/181) — reuse, don't duplicate. |

---

## 🧩 Technical Context

- **Current behavior**: each evolution-entry row shows a small
  species-hued square before the parent/child names.
- **Desired behavior**: the same row shows a neutral amber metabolism-shape
  icon instead — consistent with every other identity-bearing list in the
  game once 180/181/185 all land.

---

## 🔨 Suggested Implementation

1. Confirm 180 or 181 has landed the shared icon-painter helper; if
   neither has yet, coordinate — this task is small enough to either wait
   or extract the helper itself and let 180/181 reuse it, whichever lands
   first in practice.
2. Swap the two `rect_filled(..., species_color(...))` calls for icon-
   painter calls keyed by `world.species[entry.parent.0 as usize].metabolism`
   / the child's equivalent.
3. Live-check via a triggered speciation event.

---

## ⚠️ Constraints and Caveats

- Don't touch anything else on this card (chrome, font, button) — those
  are task 182's scope if they need it; this task is narrowly the
  species-color swatch only.
- Keep `sim`/`world`/`config` untouched.

---

## 🔗 Dependencies

- **Depends on**: 180 or 181 (whichever lands first) for the shared
  icon-painter helper — coordinate rather than writing a third copy.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/185-era-reveal-species-color.md)"$'\n\nExecute this task in the current project.'
```
