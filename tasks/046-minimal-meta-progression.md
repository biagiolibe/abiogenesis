# Task 046 — Minimal meta-progression

> **ID**: `046`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-04, Phase 3 planning session

---

## 🎯 Objective

GDD §10 [DECIDED light / persistence deferred post-MVP]: progression between runs deliberately light — unlock more starting species or tools (e.g. an extra action, or a known tag). The matrix always remains to be deciphered from scratch: you don't unlock "answers", you unlock *capabilities*. Persistence explicitly deferred: the MVP is **without on-disk persistence** — everything lives within the current process session, "trivial to add later, doesn't constrain the architecture".

This task closes Phase 3: it populates `RunProgress.unlocks` (field introduced empty by task 035) with real unlocks that expand the starting species pool (task 039) and/or the available tools (e.g. extra action, known tag), persistent across different runs **only within the same process session**.

---

## 📋 Acceptance Criteria

- [ ] `Unlocks` (in `src/run.rs`, today a minimal/empty struct from task 035) populated with at least one real unlock category: extra species in the available pool (039) and/or an additional tool (e.g. an extra `ActionBudget` point per era, or a tag revealed in advance).
- [ ] Unlocks accumulate based on a simple criterion tied to the concluded run's progress (e.g. `worlds_cleared` reached before `Defeat`) — no need for an elaborate progression system, just a clear, testable rule.
- [ ] `Unlocks` is **not written to disk**: no new dependency on filesystem APIs (`std::fs`, file-serialization crates) introduced in `run.rs`/`worldgen.rs`/`menu.rs` for this purpose — verifiable via grep.
- [ ] Unlocks remain available if the player starts a new run **within the same process session** (after a `Defeat` that returns to the main menu, task 045) — `RunProgress` (or a resource dedicated to accumulated unlocks, surviving `RunProgress`'s reset for the new run) keeps state across different runs as long as the process keeps running.
- [ ] A possible unlock summary is shown to the player (e.g. in the main menu or the defeat screen) — new section in `text.rs`.
- [ ] No unlock provides information about the hidden biochemical matrix (consistent with GDD §10: "the matrix always remains to be deciphered from scratch... you unlock capabilities, not answers" — a "known tag" unlocks the knowledge that that tag *exists/is active*, not its value in the matrix).
- [ ] `cargo clippy -- -D warnings` clean, `cargo test` green.
- [ ] Manual verification: start two consecutive runs in the same session (`cargo run` once), conclude the first with at least one unlock earned, confirm it's visible/active in the second.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/run.rs` | `Unlocks` (populated), resource that survives `RunProgress`'s reset across different runs in the same session. |
| `src/worldgen.rs` | Consumption of unlocks in the available species pool (039). |
| `src/menu.rs` | Possible unlock summary screen. |
| `src/text.rs` | New unlock strings section. |

---

## 🧩 Technical Context

**GDD §10, full text**:
> Progression between runs, deliberately light: unlock more starting species or tools (e.g. an extra action, or a known tag). The matrix always remains to be deciphered from scratch: you don't unlock "answers", you unlock capabilities.
> Persistence: the MVP is built without persistence (everything within a single run). Whether to save unlocks is decided only after verifying the loop is fun. Trivial to add later, doesn't constrain the architecture.

Note: "everything within a single run" in the GDD is the phrase used to mean "without on-disk saving" — in planning this task it was interpreted that unlocks live for the **process session** (they survive multiple consecutive runs as long as the game stays open), consistent with "trivial to add later" for actual persistence: if they were *not* to survive even across runs within the same session, meta-progression would have no observable effect for the player, which would make the task purposeless. If a different interpretation more faithful to the text emerges during implementation, verify it against `PROJECT_PLAN.md`/GDD §14 context before proceeding.

- **Current behavior**: `RunProgress.unlocks: Unlocks` exists (task 035) as an empty/minimal struct, never populated nor consumed.
- **Desired behavior**: concluding successive runs within the same session gradually expands the options available to the player (species/tools), never revealing the hidden biochemistry, never touching disk.

---

## 🔨 Suggested Implementation

1. Define `Unlocks` with concrete fields, e.g. `extra_species: Vec<SpeciesTemplate>`, `extra_action_points: u32`, `known_tags: Vec<TagId>` (adapt to existing types).
2. Decide where the state that survives `RunProgress`'s reset lives: probably a separate resource (e.g. `MetaProgress`, initialized once at process startup, not reset by `start_world`/the main menu) that accumulates unlocks earned from each concluded run.
3. At the end of a run (transition to `Defeat`, task 045), compute the unlocks earned from `worlds_cleared` and add them to `MetaProgress`.
4. In `worldgen.rs` (task 039), the available-species-pool generator consults `MetaProgress` to expand the pool.
5. Add a minimal summary to the UI (main menu or `Defeat` screen).
6. Manual verification with two consecutive runs in the same process.

---

## ⚠️ Constraints and Caveats

- **No on-disk persistence**: explicitly out of scope for the MVP (GDD §10) — don't introduce `serde`/save files for this task.
- **Don't reveal the matrix**: an unlock can say "you know tag X exists" but never "tag X is worth +2 toward tag Y" — that's information the player must deduce with the notebook (GDD §7/§11).
- **Don't introduce bonus objectives**: out of MVP scope, as already noted in task 042.
- **Keep it simple**: a clear, testable unlock criterion (e.g. linear on `worlds_cleared`) is preferable to an elaborate progression system — GDD §10 explicitly asks for "deliberately light".

---

## 🔗 Dependencies

- **Depends on**: 039 (available species pool to expand), 045 (run transition/conclusion from which to compute unlocks).
- **Blocks**: none — last task of Phase 3.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/046-minimal-meta-progression.md)"$'\n\nExecute this task in the current project.'
```
