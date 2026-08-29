# Task 180 — HUD chrome fidelity: match the reference mockup's controls, icons, and panel style

> **ID**: `180`
> **Category**: UI / Bugfix
> **Priority**: 🟡 P2 (corrective — Phase 2 residual)
> **Estimate**: ~3h
> **Assigned to**: unassigned
> **Session**: 2026-08-29 scoping, revised same day after a live screenshot
>   comparison showed the gap was wider than the first draft.

---

## 🎯 Objective

Task 151 (`tasks/done/151-pixel-grain-visual-register.md`) closed against
four narrow acceptance criteria and its own commit admits the live
verification pass was cut short. A direct screenshot-vs-mockup comparison
(2026-08-29, user-initiated) found the gap is systemic across the whole HUD,
not limited to the handful of items first scoped here — this revision
widens the task accordingly. **User instruction (2026-08-29): the redesign
must cover the sidebar/HUD in full — backgrounds, control positions, text
and button style, icon shapes, all of it** — matching
`redesign/processed/pixel-full-scene.svg`'s right-hand HUD region (the
authoritative pixel reference, `x≈590-880` in that file). This task covers
the **HUD/sidebar** (`ui.rs`); the notebook window's own chrome is task
**181** (`tasks/181-notebook-chrome-fidelity.md`), split out because the two
surfaces are different `egui` windows built by different functions and
together exceed one atomic task.

Design source: `redesign/processed/culture-shock-population-model-aesthetic.md`
§88-121 (same as 151), cross-checked pixel-by-pixel against
`redesign/processed/pixel-full-scene.svg`'s HUD region. GDD §11 "Visual
language" (`~481`) carries the same rule, marked `[PROPOSED]`.

**Read `VISUAL_STYLE_GUIDE.md` first** (written the same session as this
task, after this comparison) — it's the transcribed palette/chrome/icon spec
so this task doesn't need to re-derive hex values and button states from
raw SVG coordinates. §3 (color), §4 (organism icons), §6 (HUD chrome) are
the sections this task implements.

**Governing rule extracted from the mockup, not previously written down
anywhere in a task file**: **color in this register encodes state, never
identity.** Green/red/amber encode outcome or selection state (confirmed
positive, confirmed negative, neutral/active); species and tag identity is
carried by **text** (name, 3-letter code) and by **shape** (the metabolism
block-icon), never by a per-entity hue. The current build inverts this in
several places — see AC 3 below — because task 151 (and everything before
it) treated `species_color`/`species_hue` as the map's *and* the HUD's
identity channel, matching pre-137 conventions the aesthetic doc actually
superseded.

**User-confirmed exclusion, unchanged from before: the map's tree-glyph
terrain overlay (task 112) stays untouched.**

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.

- [ ] **Action-mode buttons (Seed/Stress/Cull/Splice) stop using emoji.**
      `ACTION_GLYPHS` (`ui.rs:1615-1620`) uses `🌱⚡💀🔬`; the surrounding
      doc comment (`ui.rs:332-334`) already documents that **egui has no
      COLR/bitmap glyph path, so these never actually render as intended**
      regardless of font. Replace with the same block-pattern icon language
      `MetabolismShapes`/task 151 built for organism shapes on the map (a
      plus-cluster for Seed, a jagged bolt for Stress, an X/skull cluster for
      Cull, a flask cluster for Splice — see `pixel-full-scene.svg:6798,
      6801, 6804, 6807` for the four exact block patterns) painted directly
      via `egui::Painter`, not a font glyph. `action_icon_row`
      (`ui.rs:1628-1696`) is the call site; its `selectable_label` +
      `RichText::new(glyph).size(20.0)` (`1649-1652`) becomes a custom
      painted button (allocate a fixed square rect, paint background +
      border + block icon, no font glyph anywhere in the path).
- [ ] **Action buttons get the mockup's box chrome**: `52×52` fixed square,
      no corner rounding (already global per 151), unselected = no fill +
      `#3a4048`-equivalent gray stroke, selected = filled dark-green
      (`#16241a`-equivalent) with a green (`#7fae6a`-equivalent) stroke —
      `pixel-full-scene.svg:6797` (selected, Seed) vs. `6800/6803/6806`
      (unselected). Today `selectable_label` uses egui's default
      pill-button visuals (a filled background at all times, distinguished
      only by a lighter fill when selected) — replace with an explicit
      painted rect + stroke pair per button, same custom-paint path as the
      icon above.
- [ ] **Time-control buttons (pulse/era/auto-advance/Notebook) get outline
      chrome, not filled egui buttons.** `time_control_row`
      (`ui.rs:1534-1609`) uses `ui.button`/`ui.selectable_label` for
      Tick/Era/Continuous/Notebook — egui's default filled-background
      style. The mockup (`pixel-full-scene.svg:6789,6791,6793`) draws these
      as `fill="none" stroke="#3a4048"` boxes, text-only until an active
      state applies a distinct fill (compare the Notebook button's small
      unread-observation badge, `6831`, already partially mirrored by this
      codebase's own `NotebookHasUnseenConfirmation` badge — keep that
      badge, only the button's own chrome needs the outline treatment).
- [ ] **Species identity goes text+shape, not hue, in the Biosphere list.**
      `ui.rs:740`: `ui.colored_label(species_color(*species), SPECIES_GLYPH)`
      colors a bullet per species. Per the governing rule above and
      `pixel-full-scene.svg:6815-6826` (Biosphere rows use a neutral amber
      `#e0c99a` pixel-cluster icon **keyed by metabolism**, not species hue
      — species is disambiguated by the name text alone, "Halo"/"Rask"/
      "Muck"), replace `SPECIES_GLYPH` here with the same metabolism
      block-icon used elsewhere (reuse whatever shared icon-painting
      function AC 2 introduces, keyed by `world.species[species.0 as
      usize].metabolism` instead of a fixed glyph). `species_color` itself
      (`render.rs:42-56`) is not deleted — task 181 needs it for whatever
      *does* legitimately stay state-colored (trend arrows, positive/
      negative edges) — just stop using it as a per-species identity tint
      in this row.
- [ ] **Section labels get the mockup's label treatment.** The mockup's
      section headers (`TIME`, `INTERVIENI`/moves, `BIOSPHERE`) render
      small, uppercase, letter-spaced, and muted-gray
      (`pixel-full-scene.svg:6787,6796,6814`, `font-size="8"
      letter-spacing="1.5" fill="#7d848a"`), distinct from body text.
      Today's headers (`ui.strong(text::HEADING_...)`, e.g. `ui.rs:699,730`)
      use plain bold body-size text. Give section headers their own
      consistent style (uppercase transform of the existing label text,
      wider letter spacing, the muted-gray tone already used for
      `ui.weak`) — a style constant/helper, not per-call-site duplication.
- [ ] **Organism ink on the map goes neutral amber, not species hue** —
      carried over from this task's first draft, still in scope here since
      it's the same governing rule applied to the grid itself.
      `render.rs::cell_color`'s `occupant` branch (`~1744-1767`) currently
      colors both `Detail` and `Overview` cells via
      `Color::hsl(species_hue(species), 0.75, lightness)` (`~1771`). Per
      the design doc (never revised, only refined by 151): a single neutral
      ink for every organism, full amber (`#e0c99a`) above the
      energy-critical threshold, dim/desaturated amber below — no
      species-hue variation on the grid. Replace the HSL-by-species formula
      with an amber ramp driven by the existing `fill` computation (don't
      change the `fill` math, only the color it drives). Apply to both
      `Detail` and `Overview` branches. `species_color`/`species_hue`
      (`render.rs:42-56`) stay untouched as *functions* — they're still
      legitimately read elsewhere (state colors, whatever 181 keeps) — this
      AC only stops `cell_color` from calling them.
- [ ] Live visual check (`cargo run`, screenshot or interactive): action
      buttons show real block icons (no tofu boxes), selected action has
      the green fill+stroke, time-control buttons read as outline boxes,
      Biosphere rows show a neutral icon per metabolism (not a colored
      dot), section headers are visually distinct from body rows, map
      organisms read as neutral amber regardless of species at both zoom
      levels.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/ui.rs` | `ACTION_GLYPHS`/`action_icon_row` (`1615-1696`) — icon+chrome rebuild. `time_control_row` (`1534-1609`) — outline chrome. Biosphere row (`~738-767`) — icon swap. Section header call sites (`ui.strong(text::HEADING_...)`, multiple). |
| `src/render.rs` | `MetabolismShapes`/mask generators (`~1408-1520`) — reuse pattern, don't reinvent; `species_color`/`species_hue` (`42-56`) — kept for legitimate state uses, not identity tinting. |
| `abiogenesis-gdd.md` | §11 "Visual language" (`~481`) — flip to `[DECIDED]` once this and 181 both land (not this task alone — split across two files, sync once both are done). |

---

## 🧩 Technical Context

- **Current behavior**: action buttons show unrendered emoji glyphs inside
  default-filled egui pill buttons; time-control buttons are default filled
  egui buttons; the Biosphere list identifies species by a species-hued
  bullet; section headers are plain bold text with no distinct treatment
  from body rows.
- **Desired behavior**: action buttons are custom-painted square boxes with
  block-pattern icons and state-only fill/stroke; time-control buttons are
  outline-only boxes; Biosphere rows use a neutral metabolism icon, name
  text carries identity; section headers read as a distinct label style.
- The emoji-rendering gap (`ui.rs:332-334`'s own doc comment) predates this
  task and predates 151 — it was never actually fixed, only documented as a
  known limitation to design around, which nothing has done yet.

---

## 🔨 Suggested Implementation

1. Extract a small shared `fn paint_metabolism_icon(painter: &egui::Painter, rect: egui::Rect, metabolism: Metabolism, color: egui::Color32)` — block-coordinate list per metabolism (mirroring whatever `MetabolismShapes`' mask generator already encodes, or a hand-transcribed version of the mockup's four icon patterns if extracting from the GPU-texture path is awkward, same latitude task 180's predecessor draft allowed for the notebook side).
2. `action_icon_row`: replace the `selectable_label` per action with an
   `ui.allocate_exact_size` 52×52 rect, paint fill+stroke by selected state,
   then call the shared icon painter with `#9aa0a6`-equivalent gray ink
   (unselected) or `#e0c99a` amber (selected, matching
   `pixel-full-scene.svg:6798`'s icon color inside the selected Seed box).
3. `time_control_row`: swap `ui.button`/first `selectable_label` calls for
   a small outline-box helper (allocate rect, `painter.rect_stroke`, center
   text) — keep the Notebook button's existing unseen-badge logic layered
   on top.
4. Biosphere row: swap `SPECIES_GLYPH`/`species_color` for the shared icon
   painter, keyed by `world.species[species.0 as usize].metabolism`, amber
   ink.
5. Section-header helper: a small `fn section_header(ui: &mut egui::Ui, label: &str)` wrapping the uppercase/letter-spacing/muted-gray style, called everywhere `ui.strong(text::HEADING_...)` is today.
6. Live-check the sidebar at both zoom levels and mid-run (some rows only
   populate once species/objectives exist).

---

## ⚠️ Constraints and Caveats

- **Do not touch the tree-glyph terrain overlay** (task 112) — direct user
  instruction.
- **No hand-drawn assets** — icons stay procedural block patterns, same
  constraint 151 operated under.
- **Don't delete `species_color`/`species_hue`** — task 181 and legitimate
  state-color uses elsewhere still need them; this task only stops using
  them as an *identity* tint in the two call sites named above.
- Exact letter-spacing/font-size values are a judgment call — egui doesn't
  expose CSS-style letter-spacing directly; approximate via spaced
  characters or a slightly wider font if the direct API isn't available,
  and don't burn time chasing sub-pixel fidelity the mockup itself doesn't
  demand (it's a design reference, not a spec sheet — same latitude every
  other pixel-grain task in this queue has taken).
- Keep `sim`/`world`/`config` untouched — presentation layer only.

---

## 🔗 Dependencies

- **Depends on**: 151 (corrects/extends it, doesn't replace it).
- **Related**: 181 (notebook-side equivalent of this task, same governing
  rule, split for atomicity — not a hard dependency, can land in either
  order, but the GDD §11 marker flip waits for both).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/180-pixel-grain-corrections.md)"$'\n\nExecute this task in the current project.'
```
