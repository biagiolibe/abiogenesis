# Abiogenesis

Emergent-simulation roguelike in Rust + Bevy: seed life on alien worlds and reverse-engineer a hidden biochemical matrix.

## Commands

```bash
cargo run                      # runs the game
cargo test                     # unit tests + determinism/balance tests
cargo clippy -- -D warnings    # must be clean before closing a task
cargo fmt
```

## Documents

| File | Contents |
|---|---|
| [`abiogenesis-gdd.md`](abiogenesis-gdd.md) | **Design — source of truth** (v0.7). Mechanics, tick formulas (§5.6), numeric baseline (§5.9). Marks `[DECIDED]` vs `[PROPOSED]` explicitly — respect the distinction. Large (~106KB): read the section you need, not the whole file. |
| [`TECH_DESIGN.md`](TECH_DESIGN.md) | Architecture: plugins, states, `SystemSets`, invariants. |
| [`tasks/QUEUE.md`](tasks/QUEUE.md) | **What to do now.** |
| [`player_guide.md`](player_guide.md) | **Player-facing manual.** What the game is, controls, loop, mechanics — also surfaced in-game via the main menu's "How to play" panel. |
| [`VISION.md`](VISION.md) | **Long-term roadmap — aspirational, not committed.** Ecosystem depth, pacing, evolution, biochemistry flavor. Ideas here graduate into `PROJECT_PLAN.md` + a task file when prioritized. |
| [`VISUAL_STYLE_GUIDE.md`](VISUAL_STYLE_GUIDE.md) | **Visual spec — palette, iconography, chrome, typography.** Written 2026-08-29 after task 151 shipped a narrower interpretation of its own reference mockups than intended. Check here before writing any rendering/UI code — it's the transcribed, citable version of what used to live only in `redesign/processed/*.svg` coordinates. |

### `redesign/processed/` — do not read

**Never read anything under `redesign/processed/` as part of normal work**, and
never re-analyse that corpus. Those documents were consumed on 2026-08-27: their
proposals became the 134-169 backlog in `tasks/QUEUE.md`, and every decision that
survived is in the GDD. Reading them costs a large fraction of a session's
context and adds nothing the canonical documents don't already say.

The one exception: a task file that names a specific document as its
`Design source`. Open that document, read that section, stop. Don't read around
it, don't pull in its neighbours, don't reopen decisions the backlog records.

`redesign/abiogenesis-INDEX.md` stays outside `processed/` and is the map of the
corpus — it carries the Phase 0 code findings and the corrections applied to the
original plan. It is short; read it when you need the corpus's shape.

## Conventions

- **Talk to the user in Italian in chat.** Everything written to disk — code, comments, documents, commit messages — stays in English regardless of the chat language.
- **Code, comments, and documents in English.**
- **One module = one Bevy `Plugin`.**
- **The grid is a `Resource`, not ECS entities.** Bevy entities exist only for rendering.
- **The simulation is deterministic and runs headless**: RNG in world state, no `rand::rng()`, no iteration over `HashMap`, no parallel queries in tick logic. `sim`/`world`/`config` don't depend on `bevy::render` or `bevy_egui`.
- **No magic numbers**: all coefficients live in `SimConfig` (`src/config.rs`).

The rationale for these rules is in `TECH_DESIGN.md` §5. Do not work around them: if a task seems to require it, the task is wrong.

## Workflow

Read `PROJECT_WORKFLOW.md` before acting — it defines the task lifecycle, review policy, roles, and document precedence. Follow `docs/CONTEXT_BUDGET_POLICY.md` for context loading and reasoning selection.

Legacy tasks (this project's original single-operator flow, still governing everything already `ACCEPTED`/`[x]`): verify the acceptance criteria in the task file, move it from `tasks/` to `tasks/done/`, update the status to `[x]` in `tasks/QUEUE.md` and `PROJECT_PLAN.md`. New or materially revised tasks use `tasks/TASK_BLUEPRINT.md`'s `QUEUED`/`IN_PROGRESS`/`READY_FOR_REVIEW`/`ACCEPTED` lifecycle instead.

## Approach
See `docs/CONTEXT_BUDGET_POLICY.md` for context loading, reasoning profile, and planning/communication rules.
- No sycophantic openers or closing fluff.