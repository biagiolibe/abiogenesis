# Task 054 — Celebrate the first confirmed hypothesis-grid cell

> **ID**: `054`
> **Category**: UI / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-07 (first-minutes engagement design session)

---

## 🎯 Objective

GDD §7 names the notebook's confirmation model ("a cell lights up when evidence crosses the threshold") as the game's core *aha* moment and its designed mitigation for the hidden matrix feeling too opaque (GDD line 376). Today that event is completely silent outside the notebook window itself: if the player isn't already looking at the hypothesis grid when a cell confirms, they never notice it happened. Add a visible feedback moment the first time this occurs in a run, so the game's central discovery beat actually registers.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors (`cargo build`).
- [ ] When `MatrixKnowledge` (src/notebook.rs) transitions a cell from unconfirmed/partial to confirmed, an entry is written to the observation log via the existing `ObservationLog`/`log_entry_line` mechanism, visually distinguished from ordinary log lines (e.g. a distinct prefix/marker consistent with existing log styling — see how death/extinction messages are already formatted in `text.rs`).
- [ ] This applies to every confirmation event, not just the first — but the acceptance bar for this task is that at minimum the *first* confirmation of a run is clearly visible; if extending it to all confirmations is free, do so for consistency (matches how extinction/death logging already covers every occurrence, not just the first).
- [ ] A visible indicator (badge/highlight) appears on whatever UI element opens the notebook window in the HUD, so a confirmation is noticeable even if the notebook isn't currently open. The badge clears once the player opens the notebook.
- [ ] All new copy lives in `src/text.rs`.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` are clean.
- [ ] `cargo test` passes; add/extend a test if `MatrixKnowledge`'s confirmation-transition logic gains new pure behavior worth covering (there are likely existing tests for the confirmation threshold — extend rather than duplicate).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `MatrixKnowledge` update logic (task 020) — where a cell's evidence crosses the confirmation threshold. Also where `ObservationLog` entries get pushed (task 018/026 pattern) and where the "notebook ever opened"/toggle system lives (also touched by task 053 — coordinate if both tasks land close together). |
| `src/text.rs` | New log-line formatting function for confirmation events, following the `extinction_message`/`player_organism_death_message` pattern. |
| `src/ui.rs` | Wherever the HUD exposes a way to open the notebook (or `src/notebook.rs` if the toggle affordance lives there) — add the badge/highlight state. |

---

## 🧩 Technical Context

- **Current behavior**: `MatrixKnowledge` (task 020, `src/notebook.rs`) accumulates weighted evidence per tag×tag cell and marks it confirmed at the threshold (`GDD §7`: cumulative evidence ≥ 3.0). This only changes what the hypothesis grid *renders* (task 031's graph view) — no log entry, no notification anywhere else.
- **Desired behavior**: the confirmation is loud enough to notice from anywhere in the game — a log line in the always-visible-ish observation log, plus a badge on the notebook affordance for players who don't have the log open either.
- Reference `text.rs`'s `extinction_message`/`player_organism_death_message` (`text.rs:188-216`) as the established pattern for turning a domain event into a log string — a `confirmation_message(from_glyph, to_glyph, positive)` function following `confirmed_relation_line`'s existing formatting (`text.rs:226-229`) is the natural fit, reused for the log instead of only the hypothesis grid's node labels.
- Check how `ObservationLog` entries currently get their `SpeciesId` for the color swatch (per the "Observation log legibility" quick task) — a matrix confirmation is about tags, not species, so this new entry type may not need that swatch, or may need its own visual treatment.

---

## 🔨 Suggested Implementation

1. In `src/notebook.rs`, find where `MatrixKnowledge`'s per-cell evidence update happens and detect the unconfirmed→confirmed transition (likely already computable from the before/after state during the update).
2. On transition, push a new `ObservationLog` entry using a new `text.rs` formatter (reuse `confirmed_relation_line`'s glyph/sign formatting, adapted into a full log sentence, e.g. via `log_entry_line`).
3. Add a simple `bool`/counter resource (e.g. `NotebookHasUnseenConfirmation`) set `true` on any confirmation, cleared when the notebook window is opened (same toggle system task 053 touches — check for that task's changes first to avoid duplicating the "notebook opened" hook, or coordinate ordering).
4. In the HUD (wherever the notebook is opened from — check if there's a button/heading, or if it's purely the `tab` keybinding with no HUD affordance yet; if there's no visible open-notebook control, a minimal one may be needed to hang the badge on), render a small badge/highlight when the flag is set.
5. Playtest via `cargo run`: play until a matrix cell confirms, verify the log entry appears and the badge lights up if the notebook is closed; open the notebook, verify the badge clears.

---

## ⚠️ Constraints and Caveats

- **Style**: no magic numbers; all copy through `text.rs`; UI/observation-log only — the confirmation *threshold* itself (GDD §5.9, ≥3.0 evidence) is not being changed by this task.
- **Coordination**: this task and task 053 both may touch a "notebook ever opened / notebook toggle" hook — whichever lands second should extend the other's flag/resource rather than introduce a duplicate.

---

## 🔗 Dependencies

- **Depends on**: none (builds on the existing `MatrixKnowledge` from task 020)
- **Blocks**: none (independent of tasks 052, 053 from the same design session; coordinate with 053 if implemented in parallel, see note above)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/054-celebrate-first-confirmed-hypothesis.md)"$'\n\nExecute this task in the current project.'
```
