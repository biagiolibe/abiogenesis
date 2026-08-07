# Task 052 — Intro screen for the first run

> **ID**: `052`
> **Category**: UI / Onboarding
> **Priority**: 🟡 P2
> **Estimate**: ~1.5h
> **Assigned to**: unassigned
> **Session**: 2026-08-07 (first-minutes engagement design session)

---

## 🎯 Objective

Today `Menu → "New run"` transitions straight into `GameState::Playing` on an empty grid (`src/menu.rs:103-127`, `start_run`), with zero framing of what the game is about. Add a one-time interstitial that plays before the very first `Playing` state of a fresh install, framing the GDD's "double mystery" (an emergent ecosystem to grow + a hidden biochemical matrix to deduce) and the era-as-experiment loop, so the player has *some* hype and context before facing a blank grid.

This must show **once per install**, not once per run — a returning player who already knows the game shouldn't see it again on their second, third, etc. run.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors (`cargo build`).
- [ ] A new `GameState::Intro` variant exists in `src/state.rs`, reachable only between `MainMenu`'s "New run" and `Playing`.
- [ ] `menu.rs::start_run` transitions to `GameState::Intro` instead of `GameState::Playing` when `MetaProgress.seen_intro == false`; if already `true`, it goes straight to `Playing` as today.
- [ ] The intro screen reuses `screens.rs::interstitial()` (same visual pattern as `world_cleared_screen_ui`/`world_failed_screen_ui`/`defeat_screen_ui`): heading + 2-3 sentence body + a single continue button.
- [ ] Copy: frames (a) the ecosystem is alien/emergent and the player seeds it, (b) its biochemistry (the tag×tag matrix) is hidden and must be deduced from observation, (c) each era is one deliberate experiment (seed/stress/cull/splice within a budget). Tone matches existing copy in `text.rs` (short, plain, no marketing fluff).
- [ ] Clicking the continue button sets `MetaProgress.seen_intro = true` and transitions to `GameState::Playing`.
- [ ] All new copy lives in `src/text.rs`, in a new section following the existing per-screen grouping convention (see `WORLD_CLEARED_TITLE` etc.).
- [ ] `cargo clippy -- -D warnings` and `cargo fmt` are clean.
- [ ] `cargo test` still passes (add a test if `MetaProgress`/`seen_intro` gains any pure logic worth covering — not required if it's a plain bool flip).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/state.rs` | Add `GameState::Intro` variant next to `MainMenu`/`Playing`/etc. |
| `src/screens.rs` | Add `intro_screen_ui` system (or similar name) using the existing `interstitial()` helper; register it in `ScreensPlugin::build` gated on `in_state(GameState::Intro)`. |
| `src/menu.rs` | `start_run` (menu.rs:103-127): branch the final state transition on `MetaProgress.seen_intro`. |
| `src/run.rs` | `MetaProgress` struct: add `seen_intro: bool` field (defaults `false`), same struct that already tracks `bonus_available_species`. |
| `src/text.rs` | New constants for the intro screen's title/body/button label, following the `WORLD_CLEARED_TITLE`/`world_cleared_body` pattern. |

---

## 🧩 Technical Context

- **Current behavior**: `start_run` (menu.rs) builds world 0 via `worldgen::build_world`, inserts Playing-state resources, and sets `GameState::Playing` directly. No screen, no copy, no explanation in between.
- **Desired behavior**: the first time ever (per `MetaProgress`, which already persists meta-progression like unlocked species across runs — check how it currently persists, e.g. in-memory-for-process vs. actual save file, since that determines whether "once per install" is literally true or just "once per process launch"), the player sees a short framing screen before the grid. Every subsequent run skips straight to `Playing` as today.
- Reference the `interstitial()` helper in `src/screens.rs:166-178` — it's the exact layout primitive to reuse (centered egui panel, no HUD).
- Reference `world_cleared_screen_ui` (`src/screens.rs:44-88`) as the closest existing example of a similar heading+body+button system reading/writing `NextState<GameState>`.

---

## 🔨 Suggested Implementation

1. Add `Intro` to `GameState` in `src/state.rs`, update its doc comment to describe when it's reachable (between `MainMenu` and `Playing`, first run only).
2. Add `seen_intro: bool` to `MetaProgress` in `src/run.rs` (default `false`).
3. Write the 2-3 sentence copy in `src/text.rs` as new `INTRO_*` constants (title, body, continue button label — reuse `CONTINUE_BUTTON` if the wording fits, otherwise a dedicated constant).
4. In `src/screens.rs`, add `intro_screen_ui`, modeled on `defeat_screen_ui`'s simplicity (no world rebuild needed — the world was already built by `start_run` before transitioning to `Intro`). On button click: set `meta.seen_intro = true`, `next_state.set(GameState::Playing)`.
5. Register the new system in `ScreensPlugin::build`, gated `run_if(in_state(GameState::Intro))`.
6. In `src/menu.rs::start_run`, after building the world and inserting resources, check `meta.seen_intro`: if `false`, transition to `GameState::Intro`; if `true`, transition to `GameState::Playing` (today's behavior).
7. Run `cargo run`, start a fresh run, verify the intro shows once; start a second run in the same session, verify it doesn't show again.

```
// Illustrative only — exact signatures depend on what's already imported.
fn intro_screen_ui(
    mut contexts: EguiContexts,
    mut meta: ResMut<MetaProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    interstitial(ctx, "intro-viewport", |ui| {
        ui.heading(text::INTRO_TITLE);
        ui.label(text::INTRO_BODY);
        if ui.button(text::INTRO_CONTINUE_BUTTON).clicked() {
            meta.seen_intro = true;
            next_state.set(GameState::Playing);
        }
    });
    Ok(())
}
```

---

## ⚠️ Constraints and Caveats

- **Style**: follow `TECH_DESIGN.md` §5 conventions — no magic numbers, all copy through `text.rs`, `sim`/`world`/`config` untouched (this is a pure `bevy_egui`/state-machine change).
- **Scope**: no toggle/settings option to re-show the intro is needed for this task — out of scope.
- **Do not** touch `sim`/`world`/`config` modules; this is UI/state-flow only.

---

## 🔗 Dependencies

- **Depends on**: none
- **Blocks**: none (independent of tasks 053, 054 from the same design session)

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/052-intro-screen-first-run.md)"$'\n\nExecute this task in the current project.'
```
