# Task 140 — End-of-era reveal as a dedicated beat, and evolution applied there

> **ID**: `140`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-27 redesign adoption planning

---

## 🎯 Objective

With eras now long and rare (task 135), the end of an era must carry weight: the
game stops on its own and shows a dedicated card, and any evolution that matured
during the era is **applied at that moment** rather than the instant its
condition was met.

Design source: `redesign/processed/abiogenesis-time-scale-reveal.md` §3, §4.

Turning the wait for the era's end into anticipation is the point: the player
spends the era building hypotheses about what is maturing, and the reveal
confirms or surprises. With 60 reveals per world this would be noise; with
12-15 it is a moment.

---

## 📋 Acceptance Criteria

- [ ] At era close the simulation halts by itself and presents a dedicated
      card/vignette — not a line scrolling past in a log among other events.
- [ ] The player must see the reveal before being able to act again.
- [ ] **Relevance tiers** (minor / notable / epochal) with presentation scaling
      accordingly: a minor event can be a discreet badge, an epochal one can take
      the whole screen.
- [ ] Where applicable the reveal shows a **before/after comparison** alongside
      the generated text — e.g. a species icon that changes when it evolved a
      trait — not only the sentence.
- [ ] Evolutionary pressure accumulates during the era; a matured evolution is
      applied **at the reveal**, not when the threshold is crossed.
- [ ] Maturing evolutions give **indirect hints during the era** (population or
      energy behaving unusually) so the reveal is a *confirmation*, not a
      surprise from nowhere — consistent with the game's core principle that
      discovery is driven by observable evidence. *(This resolves one of the
      document's two open questions; the proposed answer is adopted.)*
- [ ] A species going extinct before its evolution matures simply **loses** it.
      *(The document's other open question; the simple answer is adopted for the
      first implementation.)* Leave a note in the data design: an
      never-completed evolution is exactly what a future "stratigraphic record"
      would want to record — do not make that impossible to add later.
- [ ] Time is paused while the reveal is open, consistent with the notebook rule.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] **Live `cargo run` check** — this is a presentation beat, it has to be seen.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/state.rs`, `src/run_flow.rs` | Era lifecycle and the state where the game halts. |
| `src/screens.rs` | Existing full-screen presentation layer — the closest prior art for a dedicated card. |
| `src/sim.rs` | `SelectionThresholdCrossed` and speciation: the application point moves to era close. |
| `src/text.rs` | Generated text (the full fragment grammar is task 157; here reuse what exists). |
| `src/ui.rs` | HUD state while the reveal is open. |

---

## ⚠️ Constraints and Caveats

- The full narrative generation architecture (event ranking, composable
  fragments, clinical register) is **task 157**. This task builds the beat and
  the tier mechanism; wire the tier to the same ranking score later rather than
  inventing a second scoring system now.
- Naming the *cause* of a speciation in the reveal text is **task 142** — small,
  separate, and it depends on this card existing.
- Deliberately not realtime: time never advances while the player plans or acts.
  Real-time proper was discarded definitively.

---

## 🔗 Dependencies

- **Depends on**: 135
- **Blocks**: 142

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/140-end-of-era-reveal-beat.md)"$'\n\nExecute this task in the current project.'
```
