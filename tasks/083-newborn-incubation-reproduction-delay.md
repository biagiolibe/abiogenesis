# Task 083 — Newborn incubation: reproduction delayed to the following era

> **ID**: `083`
> **Category**: Feature / Balance
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (scoped from `redesign/abiogenesis-engagement-design.md`, proposal 1.C)

---

## 🎯 Objective

An organism born this era can currently reproduce again the very same era
once its energy crosses `repro_threshold` (`10.0`) — with a ~`+0.9`/tick
isolated photolithic gain, that's roughly 6 ticks after birth. Combined with
a `25`-tick era, world 0's population goes through several reproductive
cycles before the player has seen anything settle, reading as an instant
swarm rather than a generation the player can watch grow.

**Decided as a permanent system rule** (not scoped to world 0/onboarding
like task 082's shorter eras): an organism cannot reproduce until a later
era than the one it was born in. This changes population dynamics for the
whole game, not just the opening swarm — task 009's balance/determinism
tests must be re-validated, not just world-0 playtested.

**Must be tuned jointly with task 082**: once onboarding eras are short, the
era-boundary wait this task imposes becomes the dominant pacing constraint
during onboarding, not the energy threshold task 082 modulates on its own.

---

## 📋 Acceptance Criteria

- [ ] `Organism` (`src/world.rs:82-86`) gains a `born_era: u32` field.
- [ ] Both spawn sites in `src/sim.rs` updated: the parent's post-cost copy
      (`sim.rs:350-353`, uses `..organism` — `born_era` is naturally
      preserved, no change needed there beyond the struct gaining the
      field) and the child's spawn (`sim.rs:370-373`) — the child must be
      given `born_era: world.era` explicitly (its birth era differs from
      the parent's, so `..organism` can't be relied on here).
- [ ] The reproduction check (`sim.rs:361`, `if new_energy >=
      species.repro_threshold`) gains an additional clause: the organism may
      only reproduce if `organism.born_era < world.era` (it has survived
      into a later era than the one it was born in).
- [ ] `EnergyConfig`/`repro_threshold`/`repro_cost` (`src/config.rs:227-271`)
      are untouched — this is a structural gate, not a new tunable
      coefficient, consistent with "no magic numbers" (it reads the
      already-existing `world.era`, introduces no new constant).
- [ ] `tests/balance.rs`'s existing suites (`population_rarely_reaches_total_extinction_across_seeds`,
      `population_never_saturates_the_grid_across_seeds`,
      `bloom_usually_grows_then_stabilises_across_seeds`,
      `dark_rows_stay_uninhabited_across_seeds`) re-run and re-validated —
      incubation directly shifts these population curves; if any fails,
      investigate whether it's a genuine balance regression or an assertion
      that needs updating for the new intended dynamics (do not silently
      loosen assertions without understanding why they moved).
- [ ] `src/sim.rs`'s inline reproduction unit tests (`mod tests`, from line
      544) extended with a case confirming a same-era-born organism cannot
      reproduce even with energy above `repro_threshold`, and can once
      `world.era` advances.
- [ ] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Verified live via `cargo run`, ideally in the same session as task 082:
      a freshly-seeded organism's first reproduction visibly waits for the
      next era boundary rather than firing mid-era.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | `Organism` struct (line 82-86) — new `born_era` field. |
| `src/sim.rs` | Reproduction check (line 361) and both spawn sites (line 350-353, 370-373); inline tests (`mod tests`, line 544+). |
| `tests/balance.rs` | Population-dynamics assertions to re-validate (lines 126, 171, 191, 212). |
| `tests/determinism.rs` | Confirm determinism still holds with the new field (lines 42, 60) — no RNG introduced, should be unaffected, but re-run. |

---

## 🧩 Technical Context

- **Current behavior**: reproduction is gated solely on
  `new_energy >= species.repro_threshold` (`sim.rs:361`), with no notion of
  how long the organism has existed.
- **Desired behavior**: reproduction additionally requires
  `organism.born_era < world.era` — i.e. the organism must have survived
  across at least one era boundary since its own birth.
- `world.era: u32` (`src/world.rs:159`) is plain sim state, incremented in
  `advance_tick` (`sim.rs:504`) — no Bevy dependency, satisfies the headless
  constraint (`CLAUDE.md`).

---

## 🔨 Suggested Implementation

1. `world.rs`: add `born_era: u32` to `Organism`.
2. `sim.rs`: set `born_era: world.era` explicitly at the child spawn site
   (`370-373`); add the `organism.born_era < world.era` clause to the
   reproduction check (`361`).
3. Extend inline reproduction tests in `sim.rs`.
4. Run `tests/balance.rs` and `tests/determinism.rs`; investigate and fix
   or knowingly re-tune any assertion that breaks as a direct, understood
   consequence of the new pacing — not by loosening thresholds blindly.
5. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
6. Live verification via `cargo run`, jointly with task 082 if both are
   ready, to feel the combined onboarding pacing.

---

## ⚠️ Constraints and Caveats

- This is a **permanent** rule — resist the temptation to scope it to world
  0 only; that was explicitly decided against (unlike task 082).
- Don't introduce a new `SimConfig` coefficient for this — it's a structural
  gate on existing state (`world.era`), not a magic number.
- If `tests/balance.rs` assertions need updating, document *why* in the test
  comment (expected shift from incubation), not just adjust the numbers
  silently.

---

## 🔗 Dependencies

- **Depends on**: 009 (the balance/determinism test suite this must satisfy).
- **Tune jointly with**: 082 (shorter onboarding eras) — do not finalize
  either task's numeric constants without playtesting both together.
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/083-newborn-incubation-reproduction-delay.md)"$'\n\nExecute this task in the current project.'
```
