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
| [`abiogenesis-gdd.md`](abiogenesis-gdd.md) | **Design — source of truth.** Mechanics, tick formulas (§5.6), numeric baseline (§5.9). |
| [`TECH_DESIGN.md`](TECH_DESIGN.md) | Architecture: plugins, states, `SystemSets`, invariants. |
| [`tasks/QUEUE.md`](tasks/QUEUE.md) | **What to do now.** |
| [`player_guide.md`](player_guide.md) | **Player-facing manual.** What the game is, controls, loop, mechanics — also surfaced in-game via the main menu's "How to play" panel. |

## Conventions

- **Talk to the user in Italian in chat.** Everything written to disk — code, comments, documents, commit messages — stays in English regardless of the chat language.
- **Code, comments, and documents in English.**
- **One module = one Bevy `Plugin`.**
- **The grid is a `Resource`, not ECS entities.** Bevy entities exist only for rendering.
- **The simulation is deterministic and runs headless**: RNG in world state, no `rand::rng()`, no iteration over `HashMap`, no parallel queries in tick logic. `sim`/`world`/`config` don't depend on `bevy::render` or `bevy_egui`.
- **No magic numbers**: all coefficients live in `SimConfig` (`src/config.rs`).

The rationale for these rules is in `TECH_DESIGN.md` §5. Do not work around them: if a task seems to require it, the task is wrong.

## Workflow (Meridian)

One task at a time. On task completion:

1. verify the acceptance criteria in the task file;
2. move the file from `tasks/` to `tasks/done/`;
3. update the status to `[x]` in `tasks/QUEUE.md` and in `PROJECT_PLAN.md`.

## Approach
- Read existing files before writing. Don't re-read unless changed.
- Thorough in reasoning, concise in output.
- Skip files over 100KB unless required.
- No sycophantic openers or closing fluff.
- Tool calls (Read/Edit) consume context/tokens like any other message. Don't re-read a file just edited to "confirm" it — the tool already errors if the edit failed. Read narrow ranges, not whole files, when only a section is needed. Keep prose between tool calls minimal: state results and decisions, not a running commentary.