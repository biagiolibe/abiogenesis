# Task 043 — Objective HUD

> **ID**: `043`
> **Category**: UI
> **Priority**: 🟡 P2
> **Estimate**: ~1-2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Phase 3 planning session

---

## 🎯 Objective

GDD §11 lists "current objective" among the always-visible HUD panels. `ui.rs:243` already has a literal placeholder comment (`// Placeholder: objective arrives in Phase 3 (GDD §8).`) in the population/seed-palette HUD group, right before the help-keys line — this task fills it with a current-objective panel + progress bar, reading `ObjectiveProgress` (task 040).

---

## 📋 Acceptance Criteria

- [ ] The placeholder at `ui.rs:243` is replaced by a panel showing: the current objective's text, and a progress bar toward its satisfaction.
- [ ] The progress bar reuses the visual pattern already established by the `ActionBudget` bar (`ui.rs:171, 202-212`, task 030) — visual consistency with the existing HUD, not a new widget.
- [ ] Every displayed string goes through `src/text.rs` (new section, e.g. `// --- Objective HUD (ui.rs::objective_panel) ---`), consistent with task 034 — no hardcoded string in `ui.rs`.
- [ ] The panel is always visible during `GameState::Playing` (consistent with GDD §11).
- [ ] The objective's text is generated from `Objective`/`ObjectiveProgress` data (task 040) — `text.rs` doesn't access `SimWorld` directly (existing module constraint, see `text.rs`'s header), so formatting the concrete values (e.g. "3 coexisting species for 12 of 50 ticks") happens in `ui.rs`, which then calls a parametrized `text.rs` function for the sentence template.
- [ ] `cargo clippy -- -D warnings` clean.
- [ ] Manual verification: start the game (with a test objective if procedural worldgen isn't integrated yet) and observe that the panel updates in real time following `ObjectiveProgress`.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | Line 243 (placeholder), the `ActionBudget` bar pattern (lines 171, 202-212) to imitate. |
| `src/text.rs` | New section for the objective's strings/templates. |

---

## 🧩 Technical Context

**`text.rs`** (153 lines, task 034): module of pure constants/functions only for player-facing text, organized into `// --- Section (source_file::fn) ---` blocks. It never accesses `SimWorld` directly — data derived from state lives elsewhere, the module only provides parametrized strings/templates (e.g. `extinction_message(species_id)`, a pattern to imitate for a hypothetical `objective_progress_line(current, target)`).

- **Current behavior**: no objective panel exists — the comment at `ui.rs:243` is the only signal that this work is planned.
- **Desired behavior**: the player always sees, during `Playing`, which objective must be satisfied and how close they are to satisfying it, with the same visual language as the other HUD bars.

---

## 🔨 Suggested Implementation

1. Read `ui.rs` around line 243 to understand exactly the layout of the HUD group where the panel goes.
2. Read the `ActionBudget` bar pattern (lines 171, 202-212) to reuse its structure (egui widget, colors, layout).
3. Add to `text.rs` the functions/constants needed for the objective's text (variant name, parametrized description, possible "objective satisfied" message).
4. Implement the panel in `ui.rs`, reading `ObjectiveProgress` as a resource.
5. Manual verification with `cargo run`.

---

## ⚠️ Constraints and Caveats

- **`sim`/`world`/`config` stay headless**: this task only touches `ui.rs`/`text.rs`, it must not introduce `bevy_egui` dependencies in `objectives.rs`.
- **Consistency with task 034**: no new hardcoded string in `ui.rs` — everything goes through `text.rs`.
- **Don't anticipate the victory/defeat screens**: those are task 045, this task only concerns the HUD during active play.

---

## 🔗 Dependencies

- **Depends on**: 040 (`ObjectiveProgress`).
- **Blocks**: none (leaf of the graph, doesn't block other tasks).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/043-objective-hud.md)"$'\n\nExecute this task in the current project.'
```
