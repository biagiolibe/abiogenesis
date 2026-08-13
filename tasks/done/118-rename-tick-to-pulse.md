# Task 118 — Rename player-facing "tick" to "pulse"

> **ID**: `118`
> **Category**: UI / Terminology
> **Priority**: 🟢 P3
> **Estimate**: ~1h
> **Assigned to**: done
> **Session**: 2026-08-12 (scoped from `redesign/abiogenesis-hud-notebook.md` §1, after a
> discrepancy-check pass against tasks 100-103/097), implemented 2026-08-13.

---

## ✅ Implementation notes (2026-08-13)

- Landed after task 117 (era-relative readout), as recommended — no
  rebase needed.
- Every player-facing "tick" renamed to "pulse" in `src/text.rs`
  (`era_tick_line`'s output, `TICK_BUTTON_LABEL`, `TICK_BUTTON_TOOLTIP`,
  `KEYBOARD_HINT_PRIMARY`, the isolation-hint copy, the "Controls"
  how-to-play entry) and `player_guide.md` (controls table, loop
  description). Constant *names* (`TICK_BUTTON_LABEL`, `era_tick_line`)
  stay as-is, per scope — only string content changed.
- Internal identifiers confirmed unchanged: `SimWorld::tick`, `sim::step`,
  `EraProgress`, `single_tick`, doc comments referring to internal
  mechanics (e.g. `era_tick_line`'s own doc comment, `KEYBOARD_HINT_PRIMARY`'s
  surrounding "tick/era/notebook shortcuts" comment) all still say "tick" —
  correctly out of scope.
- No test asserted on the old exact string content, so no test changes
  were needed. `cargo build`/`clippy -D warnings`/`fmt`/`test` all clean.
- **Not verified live** (same environment limitation as task 117 — no
  Screen Recording permission for an automated screenshot); this is a
  pure string-literal change with no logic path, so risk is low, but a
  quick `cargo run` glance at the HUD/how-to-play panel is still worth
  doing before considering this fully closed.

## 🎯 Objective

`redesign/abiogenesis-hud-notebook.md` §1 proposes renaming the player-facing
term "tick" to **"pulse"** — a diegetic tie-in to another redesign track's
"the world breathes" framing (a pulse is one breath-step of the simulation,
an era is one full breath cycle). The doc itself flags this as "a proposal
to confirm, not binding" (§"Fuori scope": *"Naming definitivo 'pulse' —
proposta da confermare, non vincolante"*) — but per this session's decision,
proceed with the rename now rather than waiting for further confirmation.

This is **purely a player-facing terminology change** — internal
identifiers (`SimWorld::tick`, `sim::step`, `TickEvents`, `EraProgress`,
variable/type names throughout `src/`) stay exactly as they are. Only
strings a player actually reads change. Do **not** rename internal
Rust symbols — that would be a large, purely-cosmetic diff across
`sim.rs`/`world.rs`/tests for zero player-visible benefit, and directly
contradicts `CLAUDE.md`'s "don't refactor beyond what the task requires."

---

## 📋 Acceptance Criteria

- [ ] Every player-facing occurrence of "tick" in `src/text.rs` becomes
      "pulse" (case-matched to context): `TICK_BUTTON_LABEL` ("⏵ Tick" →
      "⏵ Pulse"), `TICK_BUTTON_TOOLTIP` ("Advance one tick (N)" → "Advance
      one pulse (N)"), `era_tick_line`'s output (`"Era {era} · tick
      {tick}"` → `"Era {era} · pulse {tick}"` — or whatever this task lands
      on, coordinate with task 117 if both are in flight, see Dependencies),
      `KEYBOARD_HINT_PRIMARY` ("space era · n tick · r reseed · wasd pan" →
      "... · n pulse · ..."), the isolation-hint copy ("watch its energy
      over the next few ticks..." → "... pulses..."), and the "Controls"
      how-to-play section ("N: advance a single tick." → "... a single
      pulse.").
- [ ] `player_guide.md`'s player-facing table/prose ("Advance a single tick
      (fine-grained observation)", "watch the ecosystem live for a block of
      ticks") updated to "pulse" the same way.
- [ ] Grep both `src/text.rs` and `player_guide.md` for any remaining
      player-facing "tick"/"ticks" occurrence after the pass above — this
      criterion isn't satisfied by editing the sites listed here if grep
      turns up more; the list above is this session's best-effort scan, not
      guaranteed exhaustive.
- [ ] Internal Rust identifiers are **unchanged**: `SimWorld::tick`,
      `sim::step`, `TickEvents`, `EraProgress`, `single_tick`
      (`input.rs`), test names, doc comments referring to internal
      mechanics — none of these rename. Only player-visible string
      *content* changes.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean (tests asserting
      on exact string content, e.g. any that check `TICK_BUTTON_LABEL`'s
      text or `era_tick_line`'s output, get their expected strings updated
      to match).
- [ ] Verified live via `cargo run`: check the HUD's tick button, tooltip,
      time readout, keyboard hint line, and the "How to play" panel all read
      "pulse," not "tick," anywhere a player would see it.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/text.rs` | `TICK_BUTTON_LABEL`, `TICK_BUTTON_TOOLTIP`, `era_tick_line`, `KEYBOARD_HINT_PRIMARY`, isolation-hint copy, `HOW_TO_PLAY_SECTIONS`' "Controls" entry — every player-facing "tick" occurrence. |
| `player_guide.md` | The full player manual's own "tick" occurrences (controls table, loop description). |
| `redesign/abiogenesis-hud-notebook.md` | §1, the source of this rename proposal; explicitly flagged there as non-binding, proceeding per this session's decision anyway. |

---

## 🧩 Technical Context

- **Current behavior**: "tick" is the player-facing term throughout the HUD,
  keyboard hints, how-to-play panel, and `player_guide.md`.
- **Desired behavior**: every one of those surfaces reads "pulse" instead,
  with zero change to internal simulation code/identifiers.
- Grep is the right tool here, not a mental list — `src/text.rs` and
  `player_guide.md` are both small enough to grep exhaustively for
  case-insensitive "tick" and review every hit individually (some may be
  internal doc-comment references that should stay as "tick" since they're
  not player-facing — judgment call per hit, not a blind find-replace).

---

## 🔨 Suggested Implementation

1. `grep -in tick src/text.rs player_guide.md` and classify every hit:
   player-facing string content (rename) vs. internal doc comment / Rust
   identifier reference (leave alone).
2. Edit each player-facing hit.
3. Update any test asserting on the old exact string content.
4. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
5. `cargo run`: verify per the acceptance criteria's live-verification line.

---

## ⚠️ Constraints and Caveats

- **No internal renames.** `SimWorld::tick`, `TickEvents`, `sim::step`,
  `single_tick`, test function names, etc. all stay as "tick" — this task's
  entire diff should be confined to string *literals* a player reads, not
  Rust symbol names. If tempted to rename `TickEvents` "for consistency,"
  don't — that's explicitly out of scope and would balloon this into an
  unrelated refactor.
- This rename is a **judgment call this session made**, not a user-confirmed
  final decision per the source doc's own hedge — if the user later wants
  "tick" back, or a different term entirely, that's a one-file `text.rs`
  string-content revert, not a structural rollback, precisely because
  internal code never changed.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: none.
- **Related, not a dependency**: task 117 (era-relative pulse-progress
  readout) touches `era_tick_line`'s same call site for a different
  reason (math, not wording) — no strict ordering requirement, but expect a
  small merge/rebase if both land in the same session without coordinating.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/118-rename-tick-to-pulse.md)"$'\n\nExecute this task in the current project.'
```
