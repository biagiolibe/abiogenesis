# Abiogenesis — Visual Style Guide

Canonical, written-down visual specification for the game: palette, iconography,
chrome, typography. This document exists because the source of truth was
previously scattered across `redesign/processed/*.svg` mockups and prose in
`culture-shock-population-model-aesthetic.md` — every implementation task
(151, 180, 181) had to re-derive rules by hand-parsing SVG coordinates, which
is slow and error-prone (task 151 itself shipped a narrower interpretation
than its own cited references, caught only by a direct screenshot
comparison, 2026-08-29). This file is the single place to check "what should
this look like" before writing rendering/UI code. It **does not introduce
new decisions** — every rule below traces to a `[DECIDED]`/`[deciso]` design
doc or a directly-inspected reference SVG, cited inline. Where the sources
leave something open, this file says so explicitly rather than inventing an
answer.

**Status**: written 2026-08-29, cross-referenced against the current build.
Where the build diverges from this spec, the divergence is noted under
"Implementation status" at the end, with the task that owns closing it.

---

## 1. Governing principles

1. **Pillar 3 — the fun is in the system, not the graphics** (GDD, v0.4
   note). Zero hand-drawn assets, ever. Every visual element is generated
   procedurally from data already in the simulation. This is not an
   aesthetic preference — it's the condition that makes 15+10 traits, 16
   biomes, and future xenotraits affordable as data instead of art
   production.
2. **Color = environment, shape = life** (GDD §11, `[PROPOSED — see
   culture-shock-population-model-aesthetic.md]`). Biomes and scalars keep
   color. Organisms are encoded by metabolism shape, never by color.
3. **Color = state, never identity** (extracted 2026-08-29 from a direct
   reading of `pixel-full-scene.svg`/`pixel-notebook.svg` — not written as
   a standalone rule in any prose doc before now, but consistently followed
   by both reference images). Green/red/amber encode an *outcome* or
   *selection state* (confirmed positive, confirmed negative, active/armed,
   neutral organism ink). Species and tag identity is carried by **text**
   (name, 3-letter code) and by **shape** (the metabolism block-icon) —
   never by a per-entity hue. This supersedes `abiogenesis-ui-redesign.md`
   §4's earlier "reuse the same species-color triad across map/population
   panel/catalog" guidance, which predates the population-model-aesthetic
   doc's neutral-ink decision. The census/notebook still carry species
   *identity* — just via name text, not a color swatch.
4. **Pixel-grain rendering, not a tileset** (GDD §11, `[DECIDED]`). Two
   fully procedural techniques: shapes snapped to a coarse block grid
   instead of smooth vector curves, and a lightweight deterministic noise
   texture over each biome's flat fill instead of a fixed two-tone dither.
   No smooth curves, no gradients, no blending anywhere in this register —
   flat color, quantized shape, stepped/orthogonal lines.
5. **A name never signals sign** (`abiogenesis-tag-archetypes.md`,
   "non-negoziabile"). A trait's name/code describes a structure or process,
   never implies whether its matrix effect is positive or negative. Applies
   to color too: nothing about a tag's node/label color may leak its sign
   before the player has observed it.

---

## 2. Typography

- **Monospace panel-wide** in the HUD sidebar (task 064) and the notebook
  window — the "field journal / lab console" register, `abiogenesis-ui-
  redesign.md` §5.
- **One deliberate break, used sparingly**: the active objective's quoted
  text ("This world wants...") renders in an editorial serif, italic
  (`pixel-full-scene.svg:6833`, `font-family="Georgia,serif"`). This is the
  *only* place a non-monospace font belongs. Don't extend it to any other
  element.
- Section labels (`TIME`, `INTERVIENI`, `BIOSPHERE`, `SEED`, etc.) are
  small, **uppercase**, letter-spaced, muted gray — visually distinct from
  body rows (`pixel-full-scene.svg:6787,6796,6814`: `font-size="8"
  letter-spacing="1.5" fill="#7d848a"`). Body text and values use the
  brighter ink tone (§3).
- Text itself is never pixel-quantized — "non aveva bisogno di alcun
  trattamento pixel" (`population-model-aesthetic.md:101`). Crisp monospace
  throughout, full stop.

---

## 3. Color system

### 3.1 Chrome and ink

| Token | Hex | Use |
|---|---|---|
| Map/world background | `#08090c` | Outermost canvas background |
| Panel background (HUD, notebook) | `#1c2229` | The one continuous "console" panel material — no nested boxed panels |
| Map viewport inset | `#0d0f14` | Slightly lighter than the outer canvas, frames the grid |
| Hairline divider (in-panel) | `#23262e` | Thin dividers *inside* an already-dark panel (`hairline()` in `ui.rs:1391`, `HAIRLINE_COLOR`, already shipped task 064) — matches `abiogenesis-ui-redesign.md`'s own suggestion ("hairline `#23262e` o equivalente tema"), a document that predates `pixel-full-scene.svg` |
| Outline/unselected button stroke | `#3a4048` | Stroke on outline-chrome buttons and section boxes against the panel background (`pixel-full-scene.svg:6789,6800` etc.) — a *different, lighter* token from the in-panel hairline above: it needs more contrast to read as a control boundary, not just a content divider. **Correction, 2026-08-29**: an earlier draft of this table conflated the two and claimed `#3a4048` was "already shipped" for `hairline()` — false, the code uses `#23262e`. The two values come from two different source documents and serve two different roles; neither is a drift to fix, both are intentional once distinguished this way.
| Body ink (primary) | `#c3c9cf` | Row text, values |
| Section-label ink (muted) | `#7d848a` | Uppercase section headers, secondary captions |
| Dim ink | `#5a5c64` | Least prominent text (e.g. era/season footer line) |
| Accent/meta ink | `#8a919a` | Seed readout, right-aligned counts |

### 3.2 State colors

| Token | Hex | Meaning |
|---|---|---|
| Positive / confirmed / active | `#7fae6a` | Rising trend, confirmed positive matrix edge, filled budget tick, selected action's border |
| Negative / confirmed-negative | `#c96a5c` | Falling trend, confirmed negative matrix edge |
| Neutral / no-effect | grey (hairline tone) | Confirmed `0` matrix edge |
| Organism ink (above energy threshold) | `#e0c99a` | Full amber — the *only* color an organism gets on the map, on any metabolism shape, regardless of species |
| Organism ink (below energy threshold) | dimmed/desaturated variant of `#e0c99a` | Same hue, reduced fill/lightness — "in affanno" reading, still never species-hued |
| Selected-action fill | `#16241a` (dark green fill) + `#7fae6a` stroke | Action-mode button, armed state |

**`species_color`/`species_hue` (`render.rs:42-56`, a full HSL hue wheel per
`SpeciesId`) is a state-*adjacent* helper, not a general identity color.**
Per rule 3 in §1, it must not be read anywhere the mockups show a neutral
icon + name text instead (map cells, Biosphere rows, Catalog icons,
relationship-graph nodes — see tasks 180/181). Its legitimate remaining
uses, if any survive the 180/181 pass, must be justified against an actual
mockup reference, not assumed.

### 3.3 Biome palette (16 biomes)

Extracted directly from `redesign/processed/biome-reference-sheet.svg`
(2026-08-29 — first time these values have been transcribed anywhere
outside that SVG). Each biome dithers between two adjacent tones per task
151's `BIOME_NOISE_LEVELS = 4` noise generator (`render.rs:1873`) — the
reference sheet only shows the two-tone dither pair the noise ramps
between, not all four discrete levels; treat the pair as the ramp's
endpoints.

| Biome | Tone A | Tone B | Tree overlay |
|---|---|---|---|
| Acqua profonda (DeepWater) | `#0d1b2e` | `#10213a` | none |
| Acqua bassa (ShallowWater) | `#1a3350` | `#1f3d5e` | none |
| Lago (Lake) | `#17322c` | `#1c3a33` | none |
| Palude (Swamp) | `#2a3320` | `#313b24` | sparse, `#14210f` |
| Pianura (Plain) | `#223a24` | `#274022` | sparse, `#14210f` |
| Foresta (Forest) | `#1c2f1e` | `#213524` | dense, `#0f1a10` |
| Collina (Hill) | `#34402a` | `#3a4630` | sparse, `#14210f` |
| Montagna (Mountain) | `#453f36` | `#4a453c` | sparse, `#14210f` |
| Vetta (Peak) | `#5a5850` | `#68655c` | none |
| Roccia nuda (BareRock) | `#423f42` | `#48454a` | none |
| Cratere profondo (Crater) | `#2a1410` | `#331a14` | none |
| Deserto (Desert) | `#4a3f22` | `#504428` | none |
| Bocca vulcanica (VolcanicVent) | `#5a2410` | `#62290f` | none |
| Geyser | `#2f3d48` | `#37454f` | none |
| Tundra | `#3a4448` | `#40494e` | none |
| Distesa di cristalli (CrystalField) | `#2a2f4a` | `#2f3450` | none |

Rendering rules (`abiogenesis-biomes.md` §"Stile di rendering"): flat color
per cell, never a gradient; sharp grid-aligned borders (coastlines heavier
than inland biome borders); light dithering for material texture, never
blending; any overlay (toxicity tint, saturation marker, etc.) composites as
a flat tint at fixed opacity, never a color average.

**Known mismatch, not yet reconciled**: the current build's `biome_color`
(`render.rs:1840-1859`) uses its own hand-picked HSL values and covers 17
`Biome` variants, three of which (`Peak`, `Glacier`, `AlpineMeadow` — task
130's Mountain sub-banding) don't appear in the 16-biome roster above, and
one of the 16 (Geyser) doesn't exist in code yet (task 114, blocked). The
two palettes were never unified — this guide records the *design* palette;
closing the gap (or deciding the code's HSL values supersede it, since
"in caso di conflitto tra un documento e il codice, vince il codice" per
`abiogenesis-INDEX.md` rule 1) is unscoped work, not silently resolved
here.

---

## 4. Organism / metabolism iconography

Four shapes, one per metabolism, unchanged in identity since their original
specification — this guide only fixes their *rendering register*, not the
shape-to-metabolism mapping:

| Metabolism | Shape |
|---|---|
| Photolithic | Asterisk |
| Predator | Triangle |
| Decomposer | Diamond |
| Chemolithotroph | Hexagon |

- **Single neutral ink**: `#e0c99a` full / dimmed variant, per §3.2 — never
  species-hued, on the map or anywhere else an organism icon appears
  (Biosphere rows, Catalog icons — task 180/181).
- **Fill state**: full/solid above the local population's energy-critical
  threshold, hollow/dim below it — the shape's fill-fraction is the health
  signal, not a separate widget.
- **Rasterization**: block-snapped to a coarse grid (`SHAPE_BLOCK_GRID = 6`
  blocks across a `SHAPE_TEXTURE_SIZE = 24`-texel mask, `render.rs:1471,
  1478`), not smooth vector curves. Baked into the texture, independent of
  GPU sampling — already shipped by task 151.
- **One shared pattern library**, reused verbatim across map, HUD, and
  notebook (`population-model-aesthetic.md` §"Cosa serve per
  l'integrazione": "un solo set riusato ovunque, non uno per contesto") —
  don't hand-author a second geometry definition for the same shape in a
  different file (task 180/181's shared-icon-painter requirement).

---

## 5. Trait / tag iconography

- **No per-tag icon exists or is planned.** A tag's visual identity is its
  **3-letter code** (task 155, `abiogenesis-tag-archetypes.md`), rendered as
  text inside/beside its node — nothing else. Don't invent a tag glyph
  system; the design corpus deliberately settled on codes-as-text over
  glyphs precisely because a glyph-per-tag doesn't scale past a handful of
  tags without becoming arbitrary (the same reasoning that killed the
  original Greek-letter scheme, see 155's own file).
- **Families (5: Structural/membrane, Metabolic/enzymatic, Signaling,
  Genetic/informational, Reserve/energy) are never shown to the player as a
  visual grouping.** `abiogenesis-tag-archetypes.md` §"Le famiglie" is
  explicit: family is "un livello di lettura secondario e opzionale" for
  experienced players inferring patterns across many runs, and the
  per-world dominant-family bias (task 156) "non viene dichiarata
  esplicitamente al giocatore" — it's discoverable only by noticing that a
  world's relationships tend to run more extreme, never labeled or
  color-coded anywhere in the UI. **Do not give families a color or icon**
  — that would contradict the doc's own design intent, not just be
  undocumented.
- **A tag's code/name never appears near a sign indicator before it's been
  observed** (§1 rule 5, restated from `abiogenesis-tag-archetypes.md`
  §"Cosa serve per l'integrazione").

### Relationship-graph grammar (`abiogenesis-ui-redesign.md` §1,
`pixel-notebook.svg`)

- **Node**: neutral dark-slate box (`#1c2229`, the panel material — not a
  new color) with an amber (`#e0c99a`) stroke — never tag-colored fill
  (task 181 corrects the current `tag_color(tag)`-filled circle). Solid
  border = the tag has any confirmed evidence; dashed border = hypothesis
  only, no confirmation yet.
- **Edge**: present only once at least one observation exists for that
  ordered tag pair (no edge = unknown, nothing drawn — never a `?`
  placeholder). Color: green = positive, red-rust = negative, gray =
  confirmed zero. Solid + thick = confirmed (thickness proportional to
  `{-2,-1,+1,+2}` magnitude); dashed = unconfirmed hypothesis. Routed as
  **stepped orthogonal paths** (horizontal-then-vertical), never a smooth
  or diagonal line — already shipped by task 151's `stepped_path`.
- **Layout**: circular for ≤6 active tags; force-directed for 7-8 (deferred,
  `abiogenesis-ui-redesign.md` explicitly scopes this as a future
  extension, not required now).

---

## 6. HUD/notebook chrome

- **One continuous panel**, not nested boxed sub-panels — divided by
  hairlines (`#3a4048`), already shipped (task 064's `hairline()`).
- **Squared corners everywhere** — no rounded chrome anywhere in this
  register (task 151's global `corner_radius = 0` pass).
- **Two button registers**, chosen by what the control represents:
  - **Outline box** (no fill, `#3a4048` stroke) for a plain action (pulse/
    era/notebook toggle) — filled/highlighted only when in an active state
    specific to that control (e.g. the Notebook button's unseen-observation
    badge).
  - **Filled state box** (dark-green fill `#16241a` + green stroke
    `#7fae6a` when active/selected, outline-only when not) for a
    *mode-select* control — the four action-mode buttons (Seed/Stress/
    Cull/Splice).
- **Discrete ticks, not continuous bars**, for small countable resources
  (action budget, era progress) — already shipped, `dot_row` (`ui.rs:1430`).
- **Icons are painted block patterns, never font glyphs** — emoji/Unicode
  glyphs don't render reliably in egui (`ui.rs:332-334`'s own doc comment)
  and were never the intended language; every icon in this game is the same
  procedural block-pattern technique as organism shapes (§4), just applied
  to action icons, not just metabolisms.

---

## 7. Reference mockups — what's canonical for what

| File | Covers | Status |
|---|---|---|
| `pixel-full-scene.svg` | Map + HUD, final combined pixel-grain rendering | **Canonical** for HUD layout/chrome/color |
| `pixel-notebook.svg` | Notebook's four sections in the same register | **Canonical** for notebook layout/chrome/color |
| `pixel-art-compare.svg` | The two pixel techniques (shape-snap, biome noise) isolated | Reference for the *techniques*, not final layout |
| `biome-reference-sheet.svg` | All 16 biome swatches, flat+dithered, tree overlay | **Canonical** for biome palette (§3.3) |
| `biome-example-map.svg` | A composed demonstration map | Illustrative only — "la disposizione specifica è solo dimostrativa," not a worldgen spec |
| `hud-full.svg`, `notebook-full.svg` | Earlier vector-era HUD/notebook layout (pre-pixel-grain) | **Structural reference only** (section order/presence) — superseded for rendering style by the two `pixel-*.svg` files above |
| `species-icons-color.svg` | Earlier per-species colored icon scheme | **Superseded** — contradicts §1 rule 3 (color=state, not identity); do not use |
| `inspect-tool.svg` | Inspection-tool card layout | Structural reference for that feature (task 149, already shipped) |

---

## 8. Implementation status (as of 2026-08-29)

Shipped correctly, matches this guide:
- Block-snapped organism shapes, biome noise texture, squared global chrome,
  stepped notebook edges, discrete-tick budget/era indicators, monospace
  sidebar font, hairline dividers (task 151, 064).

Open, tracked elsewhere — this guide doesn't duplicate their acceptance
criteria, only points at them:
- **Map organism ink still species-hued**, not neutral amber → task 180.
- **Action-mode icons are unrendered emoji, no block-icon library exists
  for HUD actions** → task 180.
- **Time-control buttons use filled egui defaults, not outline chrome** →
  task 180.
- **Biosphere rows use a species-colored dot, not a neutral metabolism
  icon** → task 180.
- **Relationship-graph nodes are tag-colored filled circles, not neutral
  boxes with amber stroke** → task 181.
- **Catalog icons are a flat colored bullet, not the shared block-pattern
  icon** → task 181.
- **Observation-log markers are species-colored, not outcome-colored** →
  task 181 (flagged as possibly needing new data, not just a style fix).
- **Greek-letter tag glyphs, not yet 3-letter codes** → task 155.
- **No dominant-family matrix-intensity bias yet** → task 156.
- **Biome palette: code vs. design-doc mismatch** (§3.3's "known mismatch")
  → unscoped.
- **Geyser biome doesn't exist in code** → task 114, blocked.
- **Interstitial screens and main menu use default egui chrome/buttons/
  font**, no exact-hex state-color constants exist yet (three independent
  hand-picked approximations instead) → task 182.
- **Pause menu and confirmation dialog use default `egui::Window` chrome,
  no state-color distinction on Confirm/Cancel** → task 183.
- **Inspect card's saturated-without-outlet warning is painted in the
  positive/active state color — a semantic inversion, not just an
  off-palette hex** (found 2026-08-29, code audit) → task 184, highest
  priority item in that task.
- **`hover_tooltip`'s trend indicator uses raw Unicode glyphs (`▲/▼/▬`)**,
  forbidden by §6 → task 184.
- **`Frame::popup`'s default blurred drop shadow appears on every floating
  overlay** (inspect card, hover tooltip, contextual hints, victory
  banner) — violates §1 rule 4 ("no gradients, no blending") → tasks 182
  (victory banner) and 184 (the rest).
- **Era-reveal card's genome-diff rows use `species_color` swatches**,
  violating §1 rule 3 (found 2026-08-29, code audit — not caught by the
  original 180/181 scoping pass since this card lives in `screens.rs`, not
  `ui.rs`/`notebook.rs`) → task 185.
