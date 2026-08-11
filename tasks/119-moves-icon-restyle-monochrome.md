# Task 119 — Moves icons: switch to monochrome glyphs that actually render (visual restyle only, no mutation-level badge)

> **ID**: `119`
> **Category**: UI / Visual polish
> **Priority**: 🟢 P3
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-12 (scoped from `redesign/abiogenesis-hud-notebook.md` §3, after a
> discrepancy-check pass against tasks 100-103/097 — descoped from the doc's
> full "two-tier mutation badge" proposal per this session's decision, since
> that badge would have nothing real to represent yet: see Constraints)

---

## 🎯 Objective

`redesign/abiogenesis-hud-notebook.md` §3 asks for the 4 Moves action icons
(Seed/Stress/Cull/Splice) to use small, simple, **monochrome** line-art
glyphs, plus a badge on the Splice/mutation icon showing which mutation
capability tier is unlocked. Per this session's decision, only the
**icon-set restyle** is in scope here — the mutation-tier badge is dropped
entirely for now, since no such tiered-unlock mechanic exists anywhere in
the game today (`Splice` already offers all three edit kinds —
`SwapTag`/`AddTag`/`ShiftTempOptimum` — unconditionally, no progression
gate). A badge with nothing real to represent would mislead rather than
inform; revisit if/when a real mutation-progression mechanic is designed and
scoped elsewhere.

**A real, pre-existing rendering bug makes this restyle more than
cosmetic.** `ui.rs`'s own doc comment on `DEJAVU_SANS` (line ~205-212)
already documents that egui has **no color-emoji rendering path at all**
(no COLR/bitmap glyph support) — so `ACTION_GLYPHS`' current 🌱 (Seed), 💀
(Cull), and 🔬 (Splice) are almost certainly rendering as **tofu boxes**
today, not as icons, regardless of font. Only ⚡ (Stress) happens to work,
because it has a genuine monochrome Unicode dingbat fallback glyph distinct
from its color-emoji form. The redesign doc's "monochrome, simple line
icons" direction is, independently of its own aesthetic reasoning, the
actual fix for this — verify this bug's presence at the start of this task
(`cargo run`, look at the Moves row) rather than assuming the comment is
stale.

---

## 📋 Acceptance Criteria

- [ ] Confirm live (`cargo run`) whether Seed/Cull/Splice currently render
      as tofu boxes as the code comment predicts, before making any change
      — note the actual observed state in this task's outcome notes.
- [ ] `ACTION_GLYPHS` (`src/ui.rs`, currently `[(Seed, "🌱"), (Stress, "⚡"),
      (Cull, "💀"), (Splice, "🔬")]`) is updated to use genuine monochrome
      Unicode dingbat glyphs — ones with a real non-emoji code point, not
      just "an emoji that might render," so this doesn't just swap one
      tofu-box set for another. Candidates worth checking against DejaVu
      Sans's actual coverage (verify visually, don't assume): `☠` (U+2620
      SKULL AND CROSSBONES — the classic dingbat skull, distinct from 💀's
      emoji code point) for Cull; `⚗` (U+2697 ALEMBIC — a long-standing
      chemistry-flask symbol, thematically close to the mockup's "fiala"/
      vial) for Splice; Stress keeps `⚡` (already confirmed working); Seed
      needs its own monochrome candidate (no strong pre-vetted suggestion
      here — try a few simple options and confirm what actually renders,
      e.g. a plant-adjacent dingbat or a simpler abstract mark if nothing
      plant-shaped is available in the monochrome range).
- [ ] Every one of the 4 Moves icons renders as an actual visible glyph
      (not a tofu box) after the change, confirmed via `cargo run` — this is
      the load-bearing criterion, more important than exactly matching the
      mockup's specific pictograms if a perfect monochrome match for one of
      them isn't available.
- [ ] No mutation-level badge is added — out of scope per this task's
      framing above.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean (no test asserts
      on the exact glyph strings today per a quick check, but re-verify).
- [ ] Verified live via `cargo run`: all 4 Moves icons render as legible
      glyphs, hover tooltips (name/cost/description) still work unchanged.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `ACTION_GLYPHS` (the 4 glyphs to change), `DEJAVU_SANS`'s doc comment (the documented tofu-box bug this task's restyle happens to fix), `action_icon_row` (renders the glyphs — unchanged otherwise). |

---

## 🧩 Technical Context

- **Current behavior**: `ACTION_GLYPHS` uses color emoji for 3 of 4 icons,
  which egui cannot render at all per the existing `DEJAVU_SANS` doc
  comment — those almost certainly show as tofu boxes today, not as
  pictograms, a pre-existing bug nobody had scoped a fix for until this
  redesign pass surfaced it as a side effect of the "monochrome icons"
  aesthetic ask.
- **Desired behavior**: 4 visible, monochrome, simple-line glyphs — visual
  match to the mockup's pianta/fulmine/teschio/fiala icons is a nice-to-have
  where a good monochrome candidate exists, but "actually renders as
  something, not a tofu box" is the hard requirement.
- DejaVu Sans (`assets/fonts/DejaVuSans.ttf`) is already loaded specifically
  for its non-default-egui-font glyph coverage (Greek, `●` bullet) — check
  its coverage for whatever candidate glyphs are tried here; egui/DejaVu
  won't render a code point the font simply doesn't have, same failure mode
  as the emoji problem, just for a different reason.

---

## 🔨 Suggested Implementation

1. `cargo run`, look at the current Moves row, confirm which icons are
   actually tofu boxes today.
2. Try candidate monochrome dingbat glyphs for each icon (see Acceptance
   Criteria for starting suggestions on Cull/Splice); Stress stays `⚡`.
3. Update `ACTION_GLYPHS` with whatever set actually renders and reads
   reasonably close to the mockup's intent.
4. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
5. `cargo run`: confirm all 4 render, tooltips still work.

---

## ⚠️ Constraints and Caveats

- **No mutation-tier badge, no progression logic** — this task is a glyph
  swap only. If a future task adds a real mutation-capability-tier mechanic,
  it can revisit the icon for a badge overlay then; don't build a badge
  against nonexistent state now.
- **Rendering correctness over pictogram fidelity**: a legible abstract
  monochrome glyph that isn't a perfect thematic match beats an emoji that
  matches the mockup's picture exactly but renders as an empty box.
- Don't touch `action_name`/`action_description`/`action_cost` (the
  tooltip text) — this task is glyphs only.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.
- **Related, not a dependency**: the source doc's mutation-tier badge idea
  is explicitly deferred, not built here — revisit once/if a real
  mutation-progression mechanic exists and is scoped.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/119-moves-icon-restyle-monochrome.md)"$'\n\nExecute this task in the current project.'
```
