# Abiogenesis

Emergent-simulation roguelike in Rust + Bevy: seed life on alien worlds and reverse-engineer a hidden biochemical matrix.

## Getting Started

```bash
cargo run                      # runs the game
cargo test                     # unit tests + determinism/balance tests
cargo clippy -- -D warnings    # must be clean before closing a task
cargo fmt
```

`rust-toolchain.toml` pins the compiler (channel `1.97.1`, `rustfmt` + `clippy` components); `rustup` picks it up automatically, no manual setup needed.

## Repository & Claude Code Configuration

Notes for replicating the working setup on another machine.

**Toolchain**
- Rust channel: `1.97.1` (`rust-toolchain.toml`)
- Dependencies: `bevy 0.19`, `bevy_egui 0.41`, `rand 0.10.2` (`Cargo.toml`)

**Claude Code — repo-level files (tracked in git)**
- `CLAUDE.md` — project conventions, invariants, and the Meridian workflow (task lifecycle, `PROJECT_PLAN.md` / `tasks/QUEUE.md`).
- `.claudeignore` — keeps build artifacts, `Cargo.lock`, editor config, and `.git/` internals out of Claude's context.
- `.gitignore` — excludes `/target`, OS cruft, and `.vscode/`.

**Claude Code — session settings (not stored in the repo, set per machine/session)**
- Model: **Sonnet 5** (`claude-sonnet-5`)
- Advisor: **Opus 5**
- Effort level: **medium** (session default)

These are CLI/session preferences, not project files — set them again with `/model`, `/advisor`, and `/effort` after cloning on a new machine.

**Not tracked in git (machine-local)**
- `.claude/settings.local.json` — local permission allowlist; regenerate as needed, Claude Code will prompt for approvals on first use.
- `.vscode/` — editor-specific config.
