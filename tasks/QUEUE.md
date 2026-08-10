# Task Execution Queue

This is the operational execution queue. Tasks are ordered by priority.

Closed phases (everything fully `[x]`) live in
[`QUEUE_ARCHIVE.md`](QUEUE_ARCHIVE.md), not here — this file only tracks
work with something still open, to keep the per-session read cost down.
Check the archive when you need the history/rationale behind a past phase,
not by default.

## How to use this queue

- **Execution**: Take the first available `[ ]` task.
- **Update**: Change `[ ]` to `[/]` when starting, and to `[x]` when finishing.
- **Archiving**: Once completed, move the file to `tasks/done/`. Once an
  entire phase/section below is fully `[x]`, move its rows to
  `QUEUE_ARCHIVE.md`.

## Priority

| Code | Meaning |
|--------|-------------|
| 🔴 P1  | Blocking / Critical |
| 🟡 P2  | Important feature |
| 🟢 P3  | Optimization / Polish |

---

## 🤖 How to delegate a task to Claude CLI

```bash
claude "$(cat tasks/NNN-name.md)"$'\n\nExecute this task in the current project.'
```

---

## 🏃 Active Queue

**Two-tier map view** (2026-08-09, design discussion held right after task
074's visual check surfaced an organism-legibility gap at 128×80 — full
decision record in `redesign/abiogenesis-two-tier-view.md`): a
continuous-zoom camera with a hard-threshold switch between the current
per-cell rendering (Detail) and an aggregated per-species cluster heatmap
(Overview), plus gating Stress/Cull to Detail while Seed/Splice stay
available in both. 075 and 076 are done (archive); 078 is a same-day
playtest correction to 076 (blobs currently trace the real occupied-cell
footprint 1:1, including gaps — should render smaller and uniformly filled).
**078 is on hold as of 2026-08-10 (⏸ do not pick up until unheld)** — no
blocking dependency, just a deliberate pause; still `[ ]` since it's not
cancelled and not blocked by another task.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[ ]` ⏸ | 078 | ON HOLD — Overview heatmap blob shape correction (playtest correction to 076: blobs must render smaller/abstracted and uniformly filled, not a 1:1 trace of the real occupied-cell footprint with its gaps) | 076 | [078](078-overview-heatmap-blob-shape-correction.md) |

**Onboarding & engagement rollout** (2026-08-09, from `redesign/abiogenesis-engagement-design.md`, full rationale in `PROJECT_PLAN.md`'s "Onboarding & engagement rollout"): 5 onboarding-foundation proposals scoped after a multi-round discussion. 080 first (diagnostic value for playtesting the rest); 082/083 are numerically coupled — tuned together (2026-08-10), both now done. Live playtest of the combined pacing still pending (082/083 verification steps skipped this session, see below).

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 080 | Interaction spark: instant visual feedback on first-seen relations | 018, 075, 076 | [080](done/080-interaction-spark-visual-feedback.md) |
| `[ ]` ⏸ | 081 | ON HOLD (2026-08-10) — The world breathes: toxic zone pulse + diffusion drift check (rescoped down after discussion) | 033, 072 | [081](081-ambient-diffusion-visible-on-empty-grid.md) |
| `[x]` | 082 | Shorter eras during world 0's opening | 079 | [082](done/082-shorter-onboarding-eras.md) |
| `[x]` | 083 | Newborn incubation: reproduction delayed to the following era | 009 | [083](done/083-newborn-incubation-reproduction-delay.md) |

084 (guaranteed "first light" relation in world 0's matrix) is scoped
(`tasks/084-first-light-guaranteed-relation-world0.md`) but deliberately
**excluded from this queue** — blocked on the "Meta-progression persistence"
proposal (`PROJECT_PLAN.md` §1), not available to pick up yet.

**Bugfixing & UX follow-ups** (2026-08-10, user-reported live during a
debugging/planning discussion held after tasks 085-090): three bugs plus
three improvements, investigated and scoped in one session. 091 merges the
two input bugs that share a root cause (nothing in the codebase checks
`bevy_egui`'s own input-capture state before the map's own input systems
run). 092 is a task-082 side effect (a fixed-tick hint duration no longer
tracks the shortened onboarding eras). 093 reverses a stale task-068
decision now that Sea carries real mechanical meaning (task 085). 094 and
095 are real feature additions, not fixes — no dependency between any of
the five, pick up in any order.

| Status | ID | Title | Depends on | File |
|-------|----|--------|------------|------|
| `[x]` | 091 | Gate map input (zoom/click/Tab) behind egui's own input capture | none | [091](done/091-egui-input-capture-gating.md) |
| `[x]` | 092 | Isolation hint duration should scale with era length | 082, 055 | [092](done/092-isolation-hint-duration-scales-with-era.md) |
| `[x]` | 093 | Sea should read as water, not "end of the world" | 068, 085 | [093](done/093-sea-color-reads-as-water-not-void.md) |
| `[x]` | 094 | On-screen buttons for tick/era/notebook controls | 030 | [094](done/094-hud-buttons-for-tick-era-notebook-controls.md) |
| `[ ]` | 095 | Procedural per-world species names + readable descriptions | 029 | [095](095-procedural-species-names-and-descriptions.md) |

Final tuning phase still lives as backlog in [`PROJECT_PLAN.md`](../PROJECT_PLAN.md) beyond what's already expanded into task files here.

---

*Last updated: 2026-08-10 (088-089, self-interaction balance bug fix, completed and archived to `QUEUE_ARCHIVE.md`. 082 and 083, tuned jointly, completed and archived to `tasks/done/`. 084 stays intentionally out of the queue as blocked. 078 and 081 on hold. 085-086, "Environment as sources," fully closed and archived to `QUEUE_ARCHIVE.md`. 090, terrain island-band retune, completed and archived. 091, egui input-capture gating, completed and moved to `tasks/done/` (live-verified by the user). 092-095, the rest of the bugfixing/UX batch, are open work.)*
