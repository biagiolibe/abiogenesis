# Task 055 — Guided first-isolation hint

> **ID**: `055`
> **Category**: UI / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-07 (first-minutes engagement design session, follow-up)

---

## 🎯 Objective

The evidence-confidence system already rewards isolated experiments: `weight = 1/(1+confounders)` (`src/sim.rs:~272`), so a species placed with no occupied Moore neighbours gets full-weight evidence and confirms its matrix relations fastest. Nothing in the game currently tells the player this — a fresh player is just as likely to cluster their first placements together, diluting evidence and slowing their first "aha" moment.

Add a one-time, non-blocking hint that nudges the player's very first placement of their very first run toward isolation, without ever gating or requiring it — task 050 deliberately removed placement constraints so the player controls seeding freely, and this must not reintroduce a constraint in hint's clothing.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors (`cargo build`).
- [ ] The check runs only for the player's first-ever placement of their first-ever run (gated by a `MetaProgress` flag, same one-shot pattern as `seen_intro` from task 052 — do not gate on world index or era count alone, since those reset every world).
- [ ] After that first placement, check via `SimWorld::moore_neighbours` (`src/world.rs:298-312`) whether the placed cell's neighbours are all unoccupied.
  - [ ] If isolated: show a distinct hint (e.g. "You isolated this species — watch its energy over the next few ticks for a clean first reading").
  - [ ] If not isolated: show a softer hint suggesting isolation as a technique for a future era (e.g. "Tip: an isolated species gives cleaner readings — try it in a future era").
- [ ] The hint is purely informational: it never blocks, delays, or vetoes the placement or any subsequent action.
- [ ] The hint is self-dismissing (disappears after being shown once / after a few ticks) — no manual dismiss button, matching task 053's hint pattern.
- [ ] All new copy lives in `src/text.rs`.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` are clean.
- [ ] `cargo test` passes; the isolation check itself (given a placed cell + grid state) should be extracted as a pure function and unit-tested.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | Add the hint-drawing system, following the same viewport-overlay pattern introduced by task 053. |
| `src/notebook.rs` | `PlayerPlacedCells` (task 050) is the existing hook for "a placement just happened" — reuse it, don't duplicate. |
| `src/world.rs` | `SimWorld::moore_neighbours` (`:298-312`) — reuse as-is for the isolation check. |
| `src/run.rs` | `MetaProgress` — add a one-shot flag (e.g. `seen_isolation_hint: bool`), same pattern as task 052's `seen_intro`. |
| `src/text.rs` | New hint strings for the isolated / not-isolated cases. |

---

## 🧩 Technical Context

- **Current behavior**: the confounder-weight formula already exists and rewards isolation, but nothing surfaces this to the player. `Objective` (`src/objectives.rs:31-51`) evaluates sustained world conditions, not one-off placement behaviour, so this does not belong there — it needs a bespoke one-shot check, not a new `Objective` variant.
- **Desired behavior**: the very first time the player places a species in their very first run, the game observes whether that placement was isolated and tells them what that means for evidence quality — teaching by pointing at what just happened, not by lecturing beforehand.
- Coordinate with task 053: both hook the "first placement" moment via `PlayerPlacedCells`. If task 053 lands first, reuse its detection point instead of adding a second listener for the same event.

---

## 🔨 Suggested Implementation

1. Add `seen_isolation_hint: bool` to `MetaProgress` (`src/run.rs`), default `false`.
2. Extract a pure function, e.g. `fn is_isolated_placement(world: &SimWorld, x: usize, y: usize) -> bool`, checking `moore_neighbours(x, y)` all have `organism == None`. Unit-test it directly.
3. In `src/text.rs`, add the two hint strings (isolated / not-isolated cases).
4. In `src/ui.rs`, on the tick `PlayerPlacedCells` transitions from empty to non-empty (first-ever placement) and `!meta.seen_isolation_hint`: run the isolation check on the just-placed cell, show the matching hint, set `meta.seen_isolation_hint = true`.
5. Self-dismiss the hint after a short duration or after N ticks pass (reuse whatever timing/dismiss approach task 053 establishes for consistency).
6. Playtest via `cargo run`: verify the hint appears exactly once on a fresh install's first placement (both isolated and clustered scenarios), never blocks input, and doesn't reappear on subsequent runs.

---

## ⚠️ Constraints and Caveats

- **Style**: no magic numbers beyond what egui requires; all copy through `text.rs`; `sim`/`world`/`config` untouched beyond reading `moore_neighbours` (no new coefficients, no changes to tick logic).
- **Never a gate**: this must not block, cost, or discourage non-isolated placement — informational only. Task 050 removed placement constraints specifically to preserve player agency; do not walk that back.
- **One-shot, not per-run**: gate on `MetaProgress`, not on world/era state, so returning players don't see it again.

---

## 🔗 Dependencies

- **Depends on**: 053 (shares the "first placement" detection point and hint-overlay pattern; land after or alongside it to avoid duplicate resources)
- **Blocks**: none

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/055-guided-first-isolation-hint.md)"$'\n\nExecute this task in the current project.'
```
