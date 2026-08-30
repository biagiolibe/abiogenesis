# Task 182 — Interstitial screens + main menu chrome, and shared state-color constants

> **ID**: `182`
> **Category**: UI / Bugfix
> **Priority**: 🟡 P2 (corrective — Phase 2 residual)
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-29 scoping

---

## 🎯 Objective

Task 151 restyled the HUD sidebar and got a global corner-radius=0 pass;
tasks 180/181 close the remaining HUD/notebook gaps. This task covers
everything else that was never touched by any restyle pass: the five
interstitial screens in `src/screens.rs` (intro/how-to-play, victory
banner, world-cleared, world-failed, defeat) and the main menu
(`src/menu.rs`). All six share the same mechanical gap — default egui
`CentralPanel`/`Frame::popup` chrome and default filled buttons — so this is
one coherent pass, not six one-offs.

**This task also introduces the first exact-hex `Color32` constants**
matching `VISUAL_STYLE_GUIDE.md`'s tokens. Today three call sites
(`ALERT_COLOR`, `DOT_FILLED_COLOR`, `trend_color`'s green/red) each
hardcode their own close-but-not-exact approximation of the state colors —
tasks 183/184 need a real shared constant to fix their own call sites
against, not three more independent guesses.

Read `VISUAL_STYLE_GUIDE.md` first — §3 (color tokens, both tables), §6
(HUD chrome — same two-register button pattern applies here even though
these aren't HUD widgets).

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] **Introduce exact-hex style constants** (new `const`s, likely
      alongside the existing `ALERT_COLOR`/`DOT_FILLED_COLOR` in `ui.rs`,
      or a small shared module if that reads cleaner — project judgment
      call, not prescribed): `PANEL_BG = #1c2229`, `STATE_POSITIVE =
      #7fae6a`, `STATE_NEGATIVE = #c96a5c`, `ORGANISM_INK = #e0c99a`,
      `OUTLINE_STROKE = #3a4048` (see §3.1's corrected table — this is the
      *outline/button-stroke* token, not the in-panel hairline). Replace
      `ALERT_COLOR`'s `Color32::from_rgb(210, 90, 90)` (`ui.rs:1140`) with
      `STATE_NEGATIVE`; replace `DOT_FILLED_COLOR`'s `(96, 200, 120)`
      (`ui.rs:1419`) and `trend_color`'s matching green/red pair
      (`ui.rs:2018-2024`) with `STATE_POSITIVE`/`STATE_NEGATIVE`. This
      alone fixes three independent hex-drift bugs task 183/184 would
      otherwise inherit.
- [ ] **Panel background**: every `CentralPanel`/`Frame::popup` in scope
      gets an explicit `.fill(PANEL_BG)` instead of egui's default gray —
      `screens.rs`'s `interstitial()` helper (used by intro `55-81`,
      world-cleared `132-159`, world-failed `166-193`, defeat `197-216`),
      the victory banner's `Frame::popup` (`screens.rs:108`), and
      `menu.rs::main_menu_ui`'s `CentralPanel::default()` (`menu.rs:75`).
      Fixing the shared `interstitial()` helper covers four of the six
      surfaces in one edit.
- [ ] **No blurred drop shadow**: `Frame::popup`'s default `popup_shadow` is
      a soft blur — a direct violation of §1 rule 4 ("no gradients, no
      blending"). Wherever this task touches a `Frame::popup` (victory
      banner), set `shadow: egui::Shadow::NONE` or an equivalent flat
      alpha-composited dim instead (the era-reveal card's own
      `Color32::from_black_alpha(140)` backdrop, `screens.rs:254`, is the
      in-repo precedent for "flat alpha, not blur").
- [ ] **Buttons get the two-register chrome from `VISUAL_STYLE_GUIDE.md`
      §6**: every plain-action button in scope (Continue/Retry/Return-to-
      menu across the five screens, New-run/how-to-play-toggle in the main
      menu) becomes an outline box (no fill, `OUTLINE_STROKE`) instead of
      egui's default filled button — same custom-paint approach 180
      introduces for the HUD's time-control buttons (reuse that helper if
      180 lands first; otherwise extract here and let 180/183 reuse it).
- [ ] Font: these screens don't inherit `hud_panel`'s monospace override
      (it's `Ui`-scoped). Apply the same monospace-family override to each
      of these six surfaces' own `Ui` (same technique as `ui.rs:666-668`),
      consistent with §2's "monospace panel-wide" — the guide doesn't
      carve out an exception for menus/interstitials.
- [ ] Live visual check (`cargo run`, screenshot or interactive): confirm
      each of the six screens (intro, victory, world-cleared, world-failed,
      defeat, main menu) shows the dark panel background, outline buttons,
      and monospace text — not egui defaults.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/screens.rs` | `interstitial()` helper + its five callers (`55-216`), victory banner (`91-125`). |
| `src/menu.rs` | `main_menu_ui` (`58-125`). |
| `src/ui.rs` | `ALERT_COLOR` (`1140`), `DOT_FILLED_COLOR` (`1419`), `trend_color` (`2018-2024`) — constants to replace; wherever the new shared constants land. |

---

## 🧩 Technical Context

- **Current behavior**: all six surfaces use egui's default panel/window
  fill, default filled buttons, default proportional font. Three separate
  call sites hardcode their own approximate state-color hex instead of a
  shared exact constant.
- **Desired behavior**: dark panel background (`#1c2229`), outline-chrome
  buttons, monospace text, flat (non-blurred) dim backdrops, and one shared
  set of exact-hex state constants other tasks (183, 184) build on.

---

## 🔨 Suggested Implementation

1. Add the shared `Color32` constants (wherever they best live — a new
   small `mod style` in `ui.rs`, or alongside existing similar constants;
   don't over-engineer a new file for six constants).
2. Fix `interstitial()`'s `CentralPanel` fill + font — covers 4 of 6
   screens at once.
3. Fix the victory banner's `Frame::popup` fill/shadow separately (it
   doesn't go through `interstitial()`).
4. Fix `main_menu_ui`'s `CentralPanel` fill + font.
5. Extract (or reuse, if 180 already did) an outline-button helper; apply
   it to every button in scope.
6. Replace `ALERT_COLOR`/`DOT_FILLED_COLOR`/`trend_color`'s hardcoded hex
   with the new constants.
7. Live-check all six screens.

---

## ⚠️ Constraints and Caveats

- Don't touch the era-reveal card (`screens.rs:237-358`) — already
  restyled for corner-radius by 151; its remaining gap (species-color
  swatches) is task 185's job, not this one.
- **No hand-drawn assets** — button chrome is paint calls, not new images.
- Keep `sim`/`world`/`config` untouched — presentation layer only.
- If 180 lands first and already extracted an outline-button helper, reuse
  it rather than writing a second one — check before implementing.

---

## 🔗 Dependencies

- **Depends on**: 151 (extends it to surfaces 151 never reached).
- **Blocks**: 183 and 184 both reuse this task's shared state-color
  constants — land this one first where practical, though it's not a hard
  compile-time dependency (183/184 could add their own constants and
  converge later if scheduling forces the order).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/182-interstitial-screens-menu-chrome.md)"$'\n\nExecute this task in the current project.'
```
