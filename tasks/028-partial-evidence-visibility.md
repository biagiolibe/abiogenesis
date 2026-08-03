# Task 028 — Distinguish "no evidence" from "unconfirmed evidence" in the hypothesis grid

> **ID**: `028`
> **Category**: UX / Feature
> **Priority**: 🟢 P3
> **Estimate**: ~1h
> **Assigned to**: unassigned
> **Session**: 2026-08-03 playtest

---

## 🎯 Objective

The hypothesis grid (task 021) only distinguishes two states per cell: `?` (unconfirmed) and `+!`/`-!` (confirmed, sign shown). `MatrixKnowledge` (task 020) actually accumulates a continuous evidence value per pair, but the UI never surfaces it below the confirmation threshold — so a pair with a real, non-zero matrix interaction that's only been observed once or twice (evidence `1.0`–`2.0` out of a `3.0` threshold) looks *identical* to a pair with a genuinely zero interaction (which never accumulates anything at all, since task 018 only emits `AdjacencyObserved` for non-zero matrix entries).

A 2026-08-03 playtest ran into exactly this: two organisms with different tags were adjacent briefly, one died, and the grid still showed `?` — with no way to tell whether that meant "no interaction exists" or "an interaction exists but wasn't observed enough yet." This task adds a visible middle state so the player can tell the difference.

**Priority note**: filed at 🟢 P3 deliberately — revisit and re-prioritize once more of Phase 2 has been played and it's clearer how much this actually confuses players in practice, versus how much of Phase 3's difficulty curve (more tags, more hostile environments) might make it moot or worse.

---

## 📋 Acceptance Criteria

- [ ] The hypothesis grid renders a third visual state for cells with `evidence > 0.0` but `< confirmation_threshold` — distinct from both `?` (zero evidence) and `+!`/`-!` (confirmed) — without revealing the sign (that would defeat the confirmation mechanic; only *that observations exist* is shown, not what they imply).
- [ ] The exact evidence value (e.g. `1.0` vs `2.9`) is **not** shown as a raw number — GDD §11's "nameless glyphs/colors, learned empirically" ethos extends here; a coarse indicator (e.g. a partially-filled glyph, a distinct weak color) is enough, not a progress bar with digits.
- [ ] `MatrixKnowledge` needs no logic changes — `evidence()` already exposes what's needed; this is a rendering-only change in `notebook.rs`'s `hypothesis_grid`.
- [ ] `cargo clippy -- -D warnings` clean.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `hypothesis_grid` — the only place this needs to change |

---

## 🧩 Technical Context

`MatrixKnowledge::evidence(exerter, receiver) -> f32` (task 020) already returns the raw accumulated value regardless of confirmation state — `hypothesis_grid` currently only calls `revealed_value` (which internally gates on `is_confirmed`). Add a call to `evidence()` for the `None` branch of `revealed_value` to decide between "true zero" and "some, not enough yet."

---

## 🔨 Suggested Implementation

1. In `hypothesis_grid`'s per-cell branch, when `revealed_value` returns `None`: check `knowledge.evidence(exerter, receiver) > 0.0`. If true, render a distinct "partial" glyph (e.g. `·?` or a dimmer `?`); if false (true zero), keep the plain `?`.
2. Manual verification: seed two organisms with a known non-zero-but-low-weight adjacency (few ticks, or several confounders so weight stays low), confirm the cell shows the new partial state instead of plain `?`; confirm a genuinely-zero pair still shows plain `?`.

---

## ⚠️ Constraints and Caveats

- Do not leak the sign or magnitude of unconfirmed evidence — only "something vs. nothing" is fair to show pre-confirmation.
- This is presentation-only; don't touch `MatrixKnowledge`'s accumulation logic or the confirmation threshold.

---

## 🔗 Dependencies

- **Depends on**: 020 (confirmation engine), 021 (hypothesis grid UI)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/028-partial-evidence-visibility.md)"$'\n\nExecute this task in the current project.'
```
