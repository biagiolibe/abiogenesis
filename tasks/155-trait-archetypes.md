# Task 155 — Trait archetypes: 3-letter codes replacing the Greek glyphs

> **ID**: `155`
> **Category**: Feature
> **Priority**: 🟡 P3 (Phase 3 — tag-archetypes)
> **Estimate**: ~2h
> **Assigned to**: Claude CLI
> **Session**: TBD

---

## 🎯 Objective

Tags currently render as single opaque Greek letters (`α`, `β`, `γ`, ...,
`notebook::TAG_LETTERS`, task 029) drawn from a **10-tag global pool**
(`TagConfig::global_tag_pool`). Replace that alphabet with **15 named
biochemical trait archetypes**, each rendered by a **3-letter uppercase
code** (`CHT`, `POR`, `LIP`, ...) in the style of real gene/protein
abbreviations, organized into **5 families** of 3 traits each (structural,
metabolic, signalling, genetic, storage). Raise `global_tag_pool` from `10`
to `15` to match.

Design source: `redesign/processed/abiogenesis-tag-archetypes.md`, sections
"Sistema di codici", "Le famiglie" (all five tables), and "Copertura e
scalabilità — pool a due livelli". GDD cross-reference: §5.5
(`abiogenesis-gdd.md:138,141`), both currently `[PROPOSED]`.

**Non-negotiable design rule (from the source doc, carried through as-is):**
a trait's name/code must never hint whether its matrix effect is positive or
negative — it describes what the thing *is or does structurally*, never a
value judgment. The matrix stays generated independently of the name, family,
or code, exactly as it's independent of the Greek letter today.

**Deliberately excluded from 155** (per `tasks/QUEUE.md`'s Phase 3 note and
the source doc's own "Fuori scope" section):
- The "active traits per world" revision (4 in world 0, ceiling 9, gradual
  4→5→6→7→8→9 ramp) — playtest-gated, touches the one tuned difficulty
  lever. `TagConfig::active_tags_early`/`active_tags_late` stay `5`/`8`.
- Dominant-family matrix-intensity bias — task 156.
- Xenotraits — task 168.
- The 10 "riserva futura" traits — documented in the source doc as a future
  expansion pool, not implemented now. `global_tag_pool` becomes `15`, not
  `25`.
- Reconsidering "Fotosoma" (the one non-strictly-real reserve name) — moot
  while the reserve stays unimplemented.

---

## 📋 Acceptance Criteria

- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] `TagConfig::global_tag_pool` default raised `10 → 15`
      (`src/config.rs:506`); `assets/config/sim_config.ron:72`'s
      `global_tag_pool: 10` updated to `15` in lockstep (`config_ron_sync.rs`
      drift test enforces this — it must still pass).
      `TagConfig::conditional_tag_count` (`src/config.rs:517`, currently `4`)
      and `active_tags_early`/`active_tags_late` (`5`/`8`) are **not**
      touched — out of scope, see above.
- [ ] A new archetype table (15 entries: code, name, family) replaces
      `notebook::TAG_LETTERS` (`src/notebook.rs:613-619`), indexed by
      `TagId` the same way `TAG_LETTERS` was (`TagId(0)` → first entry,
      etc.) — same modulo-indexed lookup pattern
      (`notebook::tag_glyph`, `src/notebook.rs:624-626`) so every existing
      call site (`notebook.rs`: lines 519, 648, 1046, 1250-1280, 1328, 1398;
      `ui.rs`: lines 1059, 1073, 1794, 1810, 1829; `text.rs`'s
      `tag_option_label`/`node_tag_line`, taking the rendered string as a
      parameter) keeps working unchanged — `tag_glyph`'s signature
      (`TagId -> &'static str`) and its role as the single choke point for
      tag display stay the same, only its output format changes from a bare
      Greek letter to a 3-letter code.
- [ ] The 15 active-pool entries from the source doc's five family tables
      are transcribed with their **English** names (see caveat below on
      translating the Italian source terms) and organized so the family
      grouping is queryable — needed by task 156's dominant-family bias and
      exposed somewhere a player-facing surface could eventually read it
      (e.g. the notebook), even if 155 itself doesn't add a family-display
      UI (see Suggested Implementation for the minimal viable shape).
      Reserve entries are **not** added to the table.
- [ ] `abiogenesis-gdd.md` updated:
      - Line 138: `**Global tag pool [DECIDED: 10 / PROPOSED: 15]:**` →
        `[DECIDED: 15]`, prose updated to state 15 as shipped, not proposed.
      - Line 141: `**Named trait archetypes [PROPOSED]:**` → `[DECIDED]`,
        prose confirmed accurate against what's actually shipped (family
        names, code style).
      - Line 154: `Tags are shown as **nameless alien glyphs/colors**` no
        longer accurate once tags carry real names — reword to state tags
        are **named but still opaque as to effect** (the non-negotiable
        rule above), distinguishing "has a name" from "hints at meaning."
        Check `abiogenesis-gdd.md:453` (`§15`/glossary-style restatement)
        for the same wording and update it too if it repeats the same
        "nameless" claim.
      - Lines 143 (dominant family), 145 (xenotraits) stay `[PROPOSED]` —
        untouched, out of scope for 155.
      - `abiogenesis-gdd.md:258` (§5.9 numeric-baseline table, "Tags and
        matrix" row `Global tag pool | `10` glyphs | ... [PROPOSED] `15`
        launch pool ...`) — a fourth place stating the pool as 10/proposed;
        update to state `15` as shipped. Leave the "Active tags / world"
        row below it (`4`/`9`/gradual ramp) untouched — out of scope.
- [ ] `player_guide.md:59` ("shown only as **nameless glyphs and colors**")
      is player-facing and goes stale the same way GDD:154 does — reword
      consistently with whatever phrasing is chosen there (named, still
      opaque as to effect).
- [ ] Existing tests referencing `tag_glyph`/`TAG_LETTERS` output
      (e.g. `src/notebook.rs` tests using `tag_glyph(TagId(0))` for
      assertions, `src/text.rs:1059-1060`'s `genome_edit_line` test which
      passes a literal `"α (Halo)"` string) still pass — the literal-string
      test doesn't call `tag_glyph` so it's unaffected either way, but
      confirm no test hardcodes a bare Greek-letter assumption anywhere
      else (`grep -rn "α\|β\|γ" src/*.rs` after the change should show only
      the now-dead `TAG_LETTERS` removal diff, no live assertions).
- [ ] `cargo test` clean, including `config_ron_sync`, `worldgen`, and
      `notebook` test modules. None of the current test suites
      (`tests/determinism.rs`, `tests/run_reproducibility.rs`) pin literal
      seeded output — they compare same-seed-vs-same-seed and
      different-seed-divergence, so the pool-size RNG-stream shift (see
      Technical Context) should not require regenerating golden values. If
      any test *does* turn out to assert a literal `TagId`/glyph value tied
      to a specific seed, that's a discovery to flag, not silently patch
      around.
- [ ] Visual check (`cargo run`, this is one of the cases where live
      verification is actually warranted — a 1-character glyph becoming a
      3-character code is a layout change, not just a data change):
      hypothesis-grid node circles (`notebook.rs:942-1090`, edges anchored
      to "a node's *rim*, not its center" per the comment at `:1095` —
      node radius is likely sized around a short label) fit the 3-letter
      code without clipping or overlapping edges; `catalog_panel`
      (`:1328`, `:1398`) and `splice_panel` radio labels
      (`ui.rs:1794/1810/1829`) don't wrap or overflow `NOTEBOOK_WIDTH`
      (`480.0`, `notebook.rs:676`) worse than before.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `TAG_LETTERS`/`tag_glyph` (`613-626`) — replace the alphabet with the 3-letter code table; `translated_tag_label` (`647-663`) composes on top, unaffected. All in-module `tag_glyph` call sites (`519`, `1046`, `1250-1280`, `1328`, `1398`). |
| `src/config.rs` | `TagConfig` (`468-520`) — `global_tag_pool` default. |
| `assets/config/sim_config.ron` | `tags.global_tag_pool` (`~72`) — must match `config.rs`'s default, enforced by `tests/config_ron_sync.rs`. |
| `src/world.rs` | `TagId`/`TagSlot` (`36`, `46`); tag-pool draw (`2981-2990`, `pool: Vec<TagId> = (0..config.global_tag_pool as u8).map(TagId).collect()`) — pool size flows through automatically once `global_tag_pool` changes, no logic change expected here. |
| `src/ui.rs` | `splice_panel` (`1741-...`) tag radio lists (`1794`, `1810`, `1829`) — display-only consumers of `tag_glyph`. |
| `src/text.rs` | `tag_option_label` (`632`), `node_tag_line` (`903`) — take a glyph string, format-agnostic, no change needed. |
| `abiogenesis-gdd.md` | §5.5 (`134-156`) — flip the two `[PROPOSED]` markers this task resolves; reword the "nameless" line. |
| `redesign/processed/abiogenesis-tag-archetypes.md` | Design source — the 15-entry active pool (5 family tables), read fully for this task; the 10-entry reserve tables and everything under "Fuori scope" are reference-only, not implemented. |
| `tasks/QUEUE.md` | Line 254's exclusion note (active-tags-per-world) — already respected, don't reopen; lines 346-352 (task 147's `is_tag_confirmed`/`active_tags` note) — confirms 155 must keep new archetypes inside `active_tags`, not create a parallel pool (relevant mainly to task 168's xenotraits, not 155 itself). |

---

## 🧩 Technical Context

- **Current behavior**: `tag_glyph(TagId) -> &'static str` returns one of 24
  hardcoded Greek letters, indexed `tag.0 as usize % TAG_LETTERS.len()`.
  `global_tag_pool` defaults to `10`, so the modulo never actually wraps
  today — it's a safety margin, not load-bearing. Every UI surface that
  names a tag (hypothesis grid nodes/edges, catalog panel, splice panel
  radio lists, matrix-confirmation log lines) calls `tag_glyph` (directly or
  via `translated_tag_label`), never reads `TagId`/`TagSlot` directly for
  display.
- **Desired behavior**: same call sites, same function signature, but the
  string returned is a 3-letter code (`"CHT"` instead of `"α"`) drawn from a
  15-entry table that mirrors the source doc's 5 families × 3 traits.
  `tag_color` (`src/notebook.rs:601-604`, deterministic HSV from `TagId`)
  is untouched — color assignment is independent of naming.
- `tag_glyph` is confirmed (by the call-site grep above) as the single
  choke point for tag display — no other function independently renders a
  Greek letter or would need a parallel update.
- **Raising `global_tag_pool` is an RNG-stream change, not just a config
  edit.** `world.rs:2986` builds `pool: Vec<TagId> = (0..global_tag_pool as
  u8).map(TagId).collect()` and draws `active_tags` from it using the
  world's own RNG; a bigger pool changes how much randomness that draw
  consumes, which shifts every downstream roll for a given seed (matrix
  generation, wild-species placement, everything after in the same
  stream) — including task 084's world-0 "first light" guarantee. This is
  expected and not a bug: `tests/determinism.rs`/`tests/run_reproducibility.rs`
  only assert same-seed-reproduces-same-seed and
  different-seed-diverges, not literal pinned values, so they should
  survive unchanged. Still worth confirming nothing added since (e.g.
  task 084's own test) accidentally pins a literal seed-to-outcome
  assertion.
- The family grouping is currently **not used by any shipped mechanic** —
  it exists in 155 purely as data (needed by task 156's dominant-family
  bias downstream). Keep the family metadata simple (e.g. an enum
  `TraitFamily { Structural, Metabolic, Signalling, Genetic, Storage }`
  alongside a `const TAG_ARCHETYPES: [(&str, &str, TraitFamily); 15]` or
  equivalent) rather than building UI or query surfaces 156 hasn't asked
  for yet.

---

## 🔨 Suggested Implementation

1. In `src/notebook.rs`, replace `TAG_LETTERS: [&str; 24]` with a
   `TAG_ARCHETYPES: [TagArchetype; 15]`-shaped table (or a simpler
   `[(&str, &str, TraitFamily); 15]` if a full struct feels like
   over-engineering for what's still display-only data) — code first
   (what `tag_glyph` returns), name second (unused by 155's own scope but
   captured now so it doesn't have to be re-derived from the design doc
   later), family third.
2. `tag_glyph` keeps its exact signature and modulo-index pattern, just
   reads `.0` (the code) from the new table instead of indexing the old
   `&str` array directly.
3. `TagConfig::global_tag_pool` default: `10 → 15` (`src/config.rs:506`).
   Mirror in `assets/config/sim_config.ron:72`.
4. Translate the source doc's Italian names to English for the `name`
   field — see the caveat below, this needs a human-quality pass, not a
   literal dictionary swap, since some source terms (e.g. "Fermentasi")
   aren't standard English biochemistry vocabulary as written.
5. Run `cargo test` — `config_ron_sync`, `worldgen`
   (`world_index_one_has_six_active_tags` and friends, unaffected by pool
   size), and any `notebook.rs` test asserting on `tag_glyph` output.
6. Update `abiogenesis-gdd.md` §5.5 per the acceptance criteria above.

---

## ⚠️ Constraints and Caveats

- **Translation quality is an open question, not resolved by this task
  file.** The source doc is in Italian; six of its ~15 active-pool terms
  are already close to standard English biochemistry vocabulary (Chitinous
  wall, Ionic pore, Lipid membrane, Chelatase, Quorum pheromone/sensing,
  Membrane receptor, Structural prion, Mobile plasmid, Catalytic ribozyme,
  Storage crystal, Dormant endospore, Lipid vacuole), a few need a
  deliberate call rather than a literal gloss (e.g. "Fermentasi" isn't a
  real English word — "Fermentation enzyme" or a more specific real enzyme
  name would need picking; "Flagello chemiotattico" → "Chemotactic
  flagellum"; "Osmoregolatore" → "Osmoregulator"). Whoever implements this
  should pick real/plausible English biochemistry terms consistent with the
  doc's "100% authentic" intent (see its footnote on "Fotosoma" being the
  one deliberate exception, kept in reserve precisely because it isn't
  authentic) rather than transliterate. Not a design decision this task
  file can make in advance — flagged, not resolved.
- **Ordering of the 15 `TagId`s is arbitrary but not risk-free.**
  `TagConfig::conditional_tag_count` (`4`) relies on convention: "the first
  `conditional_tag_count` `TagId`s ... are always the conditional ones"
  (`src/config.rs:492-499`). This convention is about index position, not
  identity or family — any consistent ordering of the 15 archetypes into
  the table satisfies it equally, so 155 has no reason to disturb which
  `TagId`s are conditional, but implementers should not accidentally read
  family membership into the conditional-tag convention (they're
  orthogonal axes).
- **Don't invent family gameplay effects.** The family grouping is
  "a secondary, optional reading level ... never a declared or reliable
  rule on a single world" per the source doc — 155 only needs the data to
  exist and be well-formed, not surfaced or weighted anywhere yet (that's
  156).
- Keep `sim`/`world`/`config` free of `bevy::render`/`bevy_egui` — this
  table is display data consumed from `notebook.rs`/`ui.rs`/`text.rs`; if
  task 156 later needs `TraitFamily` from `sim.rs`/`world.rs` for matrix
  generation, that's 156's problem to place correctly, not 155's to
  pre-empt.
- The reserve 10 traits are **documentation only** in the source doc — do
  not add a `reserve` array to the codebase for them; if a future task
  wants to promote one, it edits the 15-entry table directly.
- **Two doc comments go stale and need fixing, not just the code:**
  `notebook.rs:628-634` currently reasons that `tag_glyph` "only uses single
  Greek letters ..., already less opaque than the design doc's illustrative
  three-letter codes" and that task 143/144's friction fix "would matter
  equally ... if the game ever named traits with opaque codes" — once 155
  lands, that hypothetical is the shipped reality, so the comment needs
  rewriting, not deleting outright (the friction-fix reasoning it's
  building on is still relevant). `ui.rs:331`'s comment pointing at
  `TAG_LETTERS`/`TAG_GLYPH` for font-coverage rationale needs its
  `TAG_LETTERS` reference updated to whatever the new table is called (the
  font is still needed for `●`/`★`, only the identifier changed).
- **Open question, not resolved here: does `FIRST_N_TRANSLATED_OBSERVATIONS`
  (`notebook.rs:635`, currently `5`) still hold?** It was tuned against
  single Greek letters; 3-letter codes are arguably *more* opaque to a new
  player (a code needs decoding as an abbreviation, not just as an unknown
  symbol), which could argue for translating more of the early
  observations, not fewer. Flagged for whoever implements/playtests this,
  not decided by this task file — no source document sizes it.
- **Conditional-tag density silently shifts, worth confirming rather than
  assuming.** `conditional_tag_count` stays `4`, keyed on the first 4
  `TagId`s pool-wide (`config.rs:492-499`), but the pool grows 10→15: the
  expected number of conditional tags landing in a given world's active set
  drops (5-active world: ~2.0 → ~1.33 expected; 8-active: ~3.2 → ~2.13).
  This moves *toward* GDD:149's stated "~1-2 per world" rather than away
  from it, so it's likely a non-issue or even a small fix, but it is a
  behavioral change this task causes as a side effect of the pool-size
  edit, not something decided on its own merits. `grep -rn "conditional"
  src/worldgen.rs src/world.rs tests/` to confirm nothing pins the old
  density before assuming it's fine.

---

## 🔗 Dependencies

- **Depends on**: none (029's `tag_glyph`/`TAG_LETTERS` and 038's tag-pool
  draw are the pre-existing surfaces this task extends, both already
  shipped).
- **Blocks**: 156 (dominant-family matrix-intensity bias needs the family
  metadata this task introduces). Does not block 168 (xenotraits) directly,
  but 168's own naming-convention idea ("X-QRN" prefix style) assumes this
  task's code format exists first.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/155-trait-archetypes.md)"$'\n\nExecute this task in the current project.'
```
