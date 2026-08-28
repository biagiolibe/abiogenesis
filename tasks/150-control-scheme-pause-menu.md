# Task 150 — Full control scheme + `Esc` cascade + pause menu + protected `R`

> **ID**: `150`
> **Category**: Feature
> **Priority**: 🟡 P2 (Phase 2 — legibility)
> **Estimate**: ~2h
> **Assigned to**: Claude CLI
> **Session**: 2026-08-29

---

## 🎯 Objective

`redesign/processed/culture-shock-controls.md` unifies a control scheme that
today is only partially built and, on one key, actively contradicts the
GDD's own already-`[DECIDED]` prose (§11 already lists `Esc` as closing the
topmost open UI layer and `R` as confirmation-gated — the build does
neither). Bring the real keyboard/mouse scheme up to what GDD §11 already
declares, add the one genuinely new piece (a pause menu, never defined
before this doc), and correct the drift.

Design source: `redesign/processed/culture-shock-controls.md` (full doc —
it explicitly supersedes the scattered control mentions in GDD §11,
`abiogenesis-actions.md`, `culture-shock-inspect-tool.md` and
`abiogenesis-hud-notebook.md`).

**Current state, verified against the build (`src/input.rs`,
`src/notebook.rs`):**
- `Esc` currently calls `AppExit` directly — `input.rs::quit` (`~line 596`).
  This is the exact "accidental instant exit" risk the whole doc exists to
  remove, and directly contradicts GDD §11's own control list. **Highest-
  priority fix in this task.**
- `R` reseeds with **no confirmation of any kind** — `input.rs::reseed_world`
  (`~line 229`). Its own doc comment records a *deliberate, reasoned*
  decision against a confirmation dialog (task 094: "this codebase has no
  'are you sure?' affordance to add a safety net with"). That reasoning no
  longer holds once this task adds a pause menu (which needs exactly that
  affordance for its own "Abbandona senza salvare" item) — the two pieces
  should share one confirmation-dialog primitive.
- Mouse wheel zoom (Overview↔Detail) already works (task 075's continuous
  camera) — nothing to do there.
- `Tab` (notebook toggle) already implemented (`notebook.rs::toggle_notebook`).
- `Space` already advances a **season**, not an era (`input.rs::start_era`,
  despite the function's pre-135 name) — already correct, doc's "fix" is
  already shipped.
- **Not implemented at all, anywhere in the codebase**: `Shift+Space`
  (full-era advance), right-click "disarm current action", a per-world
  "has this world been touched by a player action" flag, any pause-menu
  state/screen.
- **`P` (continuous advancement) and `G` (jump to next notable event) do
  not exist as systems, not just as keybinds** — there is no auto-play loop
  and no relevance-threshold event detection anywhere in the codebase (only
  a stray comment in `config.rs:521` mentions "notable event"). The doc
  itself frames these as "già proposti altrove, mai assegnati a un tasto"
  (already proposed elsewhere, never bound) — but no elsewhere actually
  built them. **Out of scope for this task**: binding a key to a feature
  that doesn't exist isn't a controls fix, it's a new feature (an auto-
  advance loop, and separately an event-relevance scorer). Flag as a gap;
  don't invent the underlying systems here. **`P`/continuous-advance is now
  covered**: task 152 (scoped the same session) picked it up as one of its
  three genuine gaps, including the `input.rs` keybind — this task should
  not also bind `p`, to avoid two tasks racing to add the same system.
  `G`/jump-to-notable-event remains unclaimed by any task.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] `Esc` no longer exits the game directly. New priority cascade (one
      handler, not three independent ones, per the doc's own integration
      note): (1) notebook open → close it; (2) else an inspect card
      docked/pinned → close it (if task 149's inspect tool has landed by
      then — if not yet, skip this tier, don't block on it); (3) else an
      action armed (`SelectedAction != ` some "none" state, or existing
      default-`Seed` semantics — decide and document) → disarm to
      observation-only; (4) else → open the pause menu.
- [ ] Right-click disarms the current action (returns to observation-only,
      executes nothing) — new input system, mirrors `Esc`'s tier 3.
      Requires `SelectedAction`/`ActionMode` to have a real "no action
      armed" state if it doesn't already (check `ui.rs::ActionMode` — today
      it defaults to `Seed` with no "none" variant; decide whether to add
      one or treat "observation-only" as a separate bool alongside it).
- [ ] `Shift+Space` advances a full era (reuses `start_era`'s existing
      season-advance machinery at era granularity, or a parallel path —
      whichever avoids duplicating `tick_and_complete_season` bookkeeping).
- [ ] New per-world "touched" flag, true from the first `Seed`/`Stress`/
      `Cull`/`Splice` executed against the current world, reset via
      `WorldResetParams` (`run_flow.rs`) alongside its ~20 existing sibling
      resources. `R` requires an explicit confirmation once this flag is
      set; reseeds instantly, as today, while it's false.
- [ ] Pause menu: new screen/state reachable via `Esc`'s tier 4, with the
      four items from the doc — Riprendi (close), Impostazioni (reuse the
      existing settings panel if one exists — check `ui.rs`/main-menu code;
      if none exists yet, that's a separate gap, don't build settings here),
      Salva ed esci, Abbandona senza salvare (visually distinct, e.g. an
      alert color). Time is paused while it's open (same rule as the
      notebook).
- [ ] **"Salva ed esci" has no save system to call** — task 161 (Phase 4,
      snapshot save) doesn't exist yet. Build the pause menu item now
      (design source explicitly wants the full four-item menu shipped as a
      unit) but wire it to the same "abandon" path as "Abbandona senza
      salvare" for now, with a `// TODO(task 161)` comment — do **not**
      silently drop the item or block this task on 161. Confirm this
      approach is acceptable before or during implementation; alternative
      is deferring the whole pause-menu item to a stub "not yet available"
      state — pick whichever reads less broken to a player.
- [ ] Reused visual style: pause menu reuses the main menu's existing
      layout/theme (find it — likely a `menu.rs` or similar) rather than
      inventing a new one.
- [ ] `player_guide.md` / in-game "how to play" panel keybind list updated
      to match. GDD §11 already states the target scheme — no GDD change
      needed unless this task's implementation diverges from it (document
      any deliberate divergence, e.g. the `P`/`G` exclusion above).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/input.rs` | `quit` (→ becomes the `Esc` cascade), `reseed_world` (→ gains the confirmation gate), new right-click-disarm system. |
| `src/ui.rs` | `ActionMode`/`SelectedAction` (may need a "none armed" state), new pause-menu screen/state, confirmation dialog widget shared between `R` and "Abbandona". |
| `src/run_flow.rs` | `WorldResetParams` — add the new per-world "touched" resource to the reset list. |
| `src/notebook.rs` | `toggle_notebook` — reference pattern for a keybind + `HudControlIntents` dual trigger, and the tier-1 `Esc` check. |
| `player_guide.md` | Keybind list sync. |

---

## 🔨 Suggested Implementation

1. Add a `WorldTouched` (or similarly named) resource, set on the first
   successful Seed/Stress/Cull/Splice each world, reset in
   `WorldResetParams`.
2. Add an app-level UI-layer state (e.g. `enum TopUiLayer { None, ActionArmed,
   InspectCard, Notebook, Pause }` or reuse/derive from existing resources —
   `NotebookWindowOpen`, `SelectedAction`, a future inspect-card state) so
   the `Esc` cascade is one `match` over one clearly-ordered signal, not
   three independent `if`s racing each other.
3. Replace `input.rs::quit` with the cascade system; add the right-click
   disarm system next to `stress_on_click`/`seed_organism_on_click`.
4. Build the pause-menu egui screen (new module or folded into `ui.rs`),
   gated on the new state, pausing whatever currently pauses for the
   notebook (check how `NotebookWindowOpen` blocks `EraState::Advancing`
   today and mirror it).
5. Gate `reseed_world` behind the touched flag + a shared confirmation-
   dialog primitive (also used by "Abbandona senza salvare").
6. `Shift+Space`: extend `start_era` or add a sibling system for the
   era-level advance, being careful not to duplicate
   `tick_and_complete_season`'s bookkeeping (era completion, budget refill,
   objective evaluation) — likely loop calling the season-advance path
   `seasons_per_era` times, or a dedicated "advance until era boundary"
   helper.

---

## ⚠️ Constraints and Caveats

- **No magic numbers**: any new threshold/duration into `SimConfig`.
- Don't build `P` (continuous advancement) or `G` (jump to notable event) —
  no underlying systems exist; scoping those is a separate task.
- Don't build settings-panel content if none exists — reuse only.
- `sim`/`world`/`config` stay free of `bevy::render`/`bevy_egui` — all new
  state here is UI-owned (`ui.rs`/`input.rs`/`run_flow.rs`), consistent
  with `TECH_DESIGN.md` §5.

---

## 🔗 Dependencies

- **Depends on**: none functionally, but tier 2 of the `Esc` cascade
  (closing a docked inspect card) only applies once task 149 (inspection
  tool) exists — implement the cascade so adding that tier later is a
  one-line insertion, don't block 150 on 149 landing first.
- **Blocks**: none in Phase 2. Task 161 (Phase 4, snapshot save) will need
  to replace the "Salva ed esci" stub wired here.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/150-control-scheme-pause-menu.md)"$'\n\nExecute this task in the current project.'
```
