# Task 141 — Saturated-with-no-outlet indicator on the detail map

> **ID**: `141`
> **Category**: UI / Feature
> **Priority**: 🟡 Alta (Phase 1b)
> **Estimate**: ~1h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-28

---

## 🎯 Objective

The "saturated with no outlet" condition (a cell's population at carrying
capacity with breakout blocked because no neighbouring cell can take the
excess) is currently visible only by opening the inspection card. It's the
only one of the four Phase 1b friction points classified as *invisible by
design*, not merely unclear. Surface it directly on the detail map so the
player sees local selective pressure building without needing to click.

Design source: `redesign/processed/culture-shock-friction-fixes.md`,
Intervento 2.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean.
- [ ] `Population::blocked` (already set at `sim.rs:1192-1287`, see Technical
      Context) is rendered as a small pixel-style marker at the edge of the
      organism's cell shape, in the **detail view only** (not Overview).
- [ ] First time `blocked` becomes true for a given cell in the current run,
      the marker gets extra emphasis (accent color or blink); subsequent
      occurrences render as a plain static indicator.
- [ ] `cargo test` passes; add a render-layer test asserting the marker
      appears iff `Population::blocked` is true.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `Population::blocked` already computed and stored per-cell (lines ~1192, ~1242, ~1287). No sim changes expected. |
| `src/render.rs` | Detail-view cell rendering (`cell_color` and friends) — add the marker draw call here, gated on detail view. |
| `src/world.rs` | `Population` struct definition (confirm field visibility/access from render.rs). |

---

## 🧩 Technical Context

- **Current behavior**: `sim.rs`'s growth/breakout step (comment at
  `sim.rs:1188`) already computes `blocked: bool` and stores it on every
  `Population` written into `world.scratch[idx]`/`world.cells`. Nothing
  reads it — `grep -n '\.blocked\b' src/*.rs` matches only sim.rs's own
  writes.
- **Desired behavior**: `render.rs`'s detail-view organism drawing reads
  `Population::blocked` and, when true, draws a small marker at the edge of
  the cell's organism glyph — same blocky/pixel treatment as the existing
  count badge. Needs a "first occurrence this run" flag to decide
  emphasis vs. static — there is no existing per-cell run-scoped state for
  this; likely a new `HashSet`/`Vec<bool>`-backed field on whatever holds
  UI/run state (check `run_flow.rs` for the existing pattern used by the
  one-time onboarding hint, `run_flow.rs:109` area, to reuse the same
  idiom rather than inventing a new one).

---

## 🔨 Suggested Implementation

1. In `render.rs`, find where the detail view draws the organism glyph/count
   badge per cell and add a branch reading `cell.population.blocked`.
2. Add a small run-scoped tracker (mirrors the existing one-time-hint
   pattern in `run_flow.rs`) recording which cell indices have already
   shown the emphasized marker once.
3. Draw the marker: emphasized (accent color/blink) on first occurrence per
   cell, static otherwise. Reuse existing color-accessibility conventions
   (colour is never the only channel) — pick a distinct glyph/shape, not
   just a colour swap.
4. Add a unit/render test asserting marker presence tracks `blocked`.

---

## ⚠️ Constraints and Caveats

- **Style**: Follow `TECH_DESIGN.md` — sim/world/config stay
  render-independent; only `render.rs` (and its own run-scoped tracker)
  should depend on `blocked`.
- **Scope**: exact colours/glyph are an implementation choice, not
  specified numerically by the design doc — thresholds for `blocked` itself
  are already decided by task 137's population model, not to be retuned
  here.
- **No Overview change**: Overview doesn't expose per-cell data; this is
  detail-view only.

---

## 🔗 Dependencies

- **Depends on**: 137 (per-cell population model, `blocked` field), 138
  (tick pipeline).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/141-saturated-no-outlet-indicator.md)"$'\n\nExecute this task in the current project.'
```
