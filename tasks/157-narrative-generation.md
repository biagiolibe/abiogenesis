# Task 157 — Narrative generation: event ranking, fragment grammar, clinical register

> **ID**: `157`
> **Category**: Feature
> **Priority**: 🟢 P3 (Phase 3 — content and variety)
> **Estimate**: ~3h (includes the blocking pre-decision below; the
>   implementation itself is ~2h once that's settled)
> **Assigned to**: Claude CLI
> **Session**: unscheduled

---

## ⚠️ BLOCKING PRE-DECISION — read before writing any code

`tasks/QUEUE.md`'s Phase 3 row for this task carries an explicit warning:

> ⚠️ **Before 157**, decide whether text fragments are structured data rather
> than concatenated strings (`cross-cutting` §5). Deciding later means
> rewriting the pool.

This is not a stylistic nitpick — it's an architecture fork with a real cost
on the wrong branch, and it **must be resolved as the first acceptance-
criterion step**, deliberately, before a single fragment is written. Do not
silently default to one side.

### What `redesign/processed/abiogenesis-cross-cutting.md` §5 actually says

Full quote (§5, "Lingua"), translated:

> **The constraint that matters, to be decided now:** procedurally generated
> text (subject/cause/effect fragments for the reveal and Chronicle) is
> **much more expensive to localise** than fixed strings — languages with
> grammatical gender and agreement break the fragment concatenation designed
> around English. If a future localisation is even remotely possible, the
> fragments must be kept as **structured data** (with their grammatical
> attributes) rather than as already-concatenated strings. This is an
> architecture decision to make before building the pool, not after.

The document's own recommendation elsewhere in §5 is **English as the game's
language**, Italian only for internal documentation — i.e. no committed
localisation plan exists today. §5 does not resolve the fragment-format
question itself; it only states the tradeoff and insists it be decided
consciously.

### The two options, concretely

- **Option A — plain `String` fragments (today's pattern).** Every
  `text.rs` function in this codebase already works this way:
  `era_reveal_evolution_line` (`src/text.rs:142-155`), `genome_edit_line`
  (`src/text.rs:181-196`), `dominant_stimulus_clause`
  (`src/text.rs:162-172`) all build and return owned `String`s via `format!`,
  English-only, with grammatical agreement (`"trait"` vs `"traits"`,
  `"was"` vs `"were"`) hand-coded inline as ad-hoc `if` branches. Fast to
  write, consistent with every existing function in the module, zero new
  types. **Cost:** if the game is ever localised into a language with
  gender/case agreement, every fragment function in the new pool has to be
  rewritten from scratch — not extended, rewritten — because the
  concatenation points themselves are English-shaped.
- **Option B — structured fragment data carrying grammatical attributes,
  rendered to a string only at the final step.** Fragments become small
  structs/enums (e.g. carrying gender, number, case as data) assembled by a
  render function, with English as the only implemented renderer today.
  **Cost:** more upfront design and code for a codebase that currently has
  zero localised languages and no committed localisation task in
  `tasks/QUEUE.md` (checked — none of Phases 3-4 mentions localisation
  work); risks over-building structure nothing will read for a long time,
  against this project's own "no magic numbers, no speculative generality"
  bias elsewhere.

### Why this can't be deferred

Once the fragment pool exists and every reveal/Chronicle/objective-adjacent
line in the game reads from it (this task's own scope — see §"What needs to
be built" below), migrating from Option A to Option B means touching every
fragment and every call site a second time, not adding a new field. That's
exactly QUEUE.md's "deciding later means rewriting the pool."

### What the implementer must do

Before writing acceptance criterion 1 below, **pick A or B explicitly, write
one paragraph in this task file (or a follow-up commit note) stating which
and why**, weighing:
- Is a localisation pass realistically on this project's horizon? (As of
  this writing, `tasks/QUEUE.md` Phases 3-4 have no localisation task; the
  GDD's language stance is "English as the game's language" per
  `abiogenesis-cross-cutting.md` §5's own recommendation — leaning toward
  "not soon.")
- Is the near-term win (shipping faster, staying consistent with the
  existing `text.rs` module, which is 100% Option A today) worth the
  later rewrite risk if that assumption changes?

This document deliberately does not make that call. Whoever picks up this
task must, in the open, before touching the fragment pool.

---

## 🎯 Objective

Several points in the game already generate short prose from multiple
contributing causes by hand — the end-of-era reveal's cause clause
(`text::dominant_stimulus_clause`, `src/text.rs:162-172`, task 142) and
genome-diff line (`text::genome_edit_line`, `src/text.rs:181-196`, task 170)
are exactly this, done for a single event type with a fixed, small set of
hand-written variants (one string per enum variant, no lexical variety, no
ranking against other candidate events in the era). `RevealTier`
(`src/sim.rs:903-915`) is explicitly documented as "a first-pass,
deliberately simple heuristic — task 157 builds the real event-ranking
score everything (including this tier) should eventually read from instead;
do not grow a second, competing scoring system here." This task is that
real system: a shared **event-ranking score** (reusing the notebook's
existing confounder-weight formula, not a new causal model), a small
**composable fragment grammar** (subject/cause/effect fragments with 3-4
seeded-deterministic variants each) to replace today's single-string-per-
case pattern, and an explicit **ambiguity output** when no candidate
dominates. Scope is the era-reveal and Chronicle text paths; it does not
invent new event types beyond what `EraTally`/`EraEvolutionReveal` already
track.

---

## 📋 Acceptance Criteria

- [ ] **Step 0 (blocking):** the structured-data-vs-strings decision above
      is made explicitly and recorded (a paragraph in this file, or a doc
      comment on the new fragment module stating the choice and why) before
      any fragment pool code is written.
- [ ] A single **event score function** exists in `sim.rs` (not duplicated
      between narrative and notebook code), computed as *effect magnitude ×
      signal cleanliness*, reusing the **exact same confounder-weight
      formula** the notebook already uses — `weight = 1 / (1 +
      n_confounders)`, currently implemented inline where
      `AdjacencyObserved::n_confounders` (`src/sim.rs:141`) is consumed
      (`src/sim.rs:387-394`, and the GDD-cited formula at
      `abiogenesis-gdd.md:270` / `:367`). Do **not** build a second,
      independent causal-attribution system for narrative purposes — GDD
      §7's own text warns against exactly this drift.
- [ ] `RevealTier` (`src/sim.rs:909-915`) is computed from this shared score
      instead of the current heuristic at `src/sim.rs:1066-1073`
      (`Epochal` if any evolution / `Notable` if extinctions or lost
      evolutions / `Minor` otherwise) — the doc comment on `RevealTier`
      itself (`src/sim.rs:903-908`) names this as the intended replacement.
      Exact score-to-tier thresholds are explicitly **out of scope for GDD
      wording** (the design doc leaves them for playtest, see Constraints
      below) — pick defensible starting values in `SimConfig`, not
      hardcoded literals (project convention, `CLAUDE.md`: "no magic
      numbers").
- [ ] At least one **candidate-collection pass** per era-reveal build
      (`build_era_reveal`, `src/sim.rs:1032-1082`) gathers this era's
      candidate events — population deltas, confirmed relations, environmental
      shocks, matured evolutions (per the design doc's own list) — from data
      `EraTally`/`EraEvolutionReveal`/`SimWorld` already track; this task
      does **not** need to add new event *sources*, only rank the ones that
      already exist in these structs.
- [ ] The **top-ranked candidate** becomes the reveal's main clause; a
      second candidate becomes a subordinate clause only if its score is
      within a configurable closeness threshold of the top one (new
      `SimConfig` fields, not magic numbers); all other candidates are
      **not** folded into the generated sentence — GDD/design-doc
      instruction that the text must not try to be a complete era summary.
- [ ] **Ambiguity path**: when the top two (or more) candidate scores are
      within that same closeness threshold with no clear single dominant
      cause, the generated text explicitly says so (e.g. "several factors
      seem to have overlapped this era, none clearly isolable") instead of
      forcing an attribution — per the design doc §3, this is a legitimate,
      intentional output, not an edge case to hide, and it reinforces the
      game's existing epistemic framing ("check the log before suspecting
      the matrix," main-menu copy).
- [ ] A small **fragment grammar** replaces today's single-string-per-case
      functions for at least the cause-clause path currently served by
      `dominant_stimulus_clause` (`src/text.rs:162-172`): each of subject /
      cause / effect gets 3-4 lexical variants, chosen deterministically —
      seeded from **world seed + era number** (not a global/thread RNG;
      project rule, `CLAUDE.md`: "RNG in world state, no `rand::rng()`"),
      so regenerating the same era (e.g. via a debug replay) reproduces the
      exact same sentence. Reuse `SimWorld`'s existing deterministic RNG
      infrastructure rather than introducing a new one — check how
      `SimWorld`'s RNG field is seeded/consumed elsewhere in `sim.rs` before
      adding a second seeded-RNG pattern.
- [ ] Register check: every new fragment reads in the same clinical,
      no-raw-numbers voice as the existing lines it extends —
      `dominant_stimulus_clause`, `genome_edit_line`,
      `population_delta_label` (`src/text.rs:438-462`), `species_origin_label`
      (`src/text.rs:966-972`) are the established tone reference; no new
      literary register, no restating raw floats/enum indices (same rule
      task 170 already followed).
- [ ] **Data-coherence check**: every fragment used in the generated
      sentence must be traceable to a real value shown elsewhere in the UI
      at that moment (population count/delta, tag name, era number) — per
      the design doc §5, a mismatch between the generated sentence and the
      Biosphere/notebook readout is called out as the single most damaging
      failure mode, worse than limited lexical variety. Add or extend a
      test asserting the chosen fragment's referenced entity (species name,
      tag) matches the data structure it was ranked from.
- [ ] Chronicle (`text::chronicle_quiet_line`, `src/text.rs:988-995`,
      `abiogenesis-gdd.md:365`) continues to store exactly what the reveal
      already produced — "no separate text generation," per the GDD line —
      so this task's new ranking/fragment output flows into the Chronicle
      unchanged, not re-derived.
- [ ] `cargo build` / `cargo clippy -- -D warnings` clean, `cargo fmt`.
- [ ] Unit tests: score computation reuses the confounder-weight formula
      (assert a shared function, not two copies); ambiguity path fires when
      scores are close and produces the documented hedge text; seeded
      fragment choice is deterministic across two calls with the same
      seed/era and varies across different eras; existing
      `era_reveal_evolution_line_names_a_different_cause_per_dominant_stimulus`
      test (`src/text.rs:1040-1041` region) and other text-layer tests
      touching functions this task extends still pass.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/sim.rs` | `RevealTier` (`903-915`) — target of the tier-computation replacement. `EraEvolutionReveal`/`EraReveal` (`924-966`) — existing per-era data to rank. `build_era_reveal` (`1032-1082`) — where candidate collection and ranking hooks in. `AdjacencyObserved`/`n_confounders` (`~117-141`, weight computed `387-394`) — the confounder-weight formula to reuse, not reimplement. |
| `src/text.rs` | `era_reveal_evolution_line` (`142-155`), `dominant_stimulus_clause` (`162-172`), `genome_edit_line` (`181-196`) — the single-string-per-case pattern this task's fragment grammar extends/replaces for the cause-clause path. `population_delta_label` (`438-462`), `species_origin_label` (`966-972`) — clinical-register reference points. `chronicle_quiet_line` (`988-995`) — confirms Chronicle stores reveal output as-is. |
| `src/config.rs` | `SimConfig` — new closeness-threshold / tier-cutoff fields belong here, not as local `const`s or magic numbers. |
| `redesign/processed/abiogenesis-narrative-generation.md` | Design source for this task — read in full already; architecture section ("Architettura proposta") is the authoritative spec for ranking + fragment grammar + ambiguity + register. |
| `redesign/processed/abiogenesis-cross-cutting.md` §5 | The blocking pre-decision quoted in full above. |
| `abiogenesis-gdd.md` | §7 (confounder weight, lines ~270, ~367) — formula to reuse. §5.11-adjacent Chronicle description (line 365) — "no separate text generation." |

---

## 🧩 Technical Context

- **Current behavior**: `RevealTier` uses a fixed three-branch heuristic
  (any evolution → Epochal; extinctions/lost evolutions → Notable; else
  Minor) with no notion of relative event magnitude. Cause clauses
  (`dominant_stimulus_clause`, `genome_edit_line`) are one hardcoded string
  per enum variant — no lexical variety, no seeding, and only cover the
  single "why did this species evolve" event type. There is no candidate
  ranking across event *types* (population delta vs. confirmed relation vs.
  environmental shock vs. evolution) at all today — each type is surfaced
  independently wherever the UI happens to show it.
- **Desired behavior**: one shared score (magnitude × confounder-based
  cleanliness) ranks all of an era's candidate events; the top one or two
  drive the generated sentence; ties/no-clear-winner produce an explicit
  hedge sentence instead of a forced attribution; the sentence is built
  from small seeded fragments instead of one fixed string per case,
  extending — not replacing — the module's existing clinical register.
- **Explicitly not required by this task** (design doc's own "Fuori scope"
  section): the full lexical content of every fragment category (structure
  matters here, not exhaustive content — a small pool that's built to grow
  by addition is enough); exact numeric thresholds for "how close is a
  tie" or "how high a score counts as dominant" (playtest-tuned, same
  status as `MATURING_HINT_FRACTION`/`STALL_BAND` elsewhere in `sim.rs`);
  an LLM-based cosmetic layer (explicitly rejected as a foundation by the
  design doc, for reproducibility and offline-play reasons — optional
  future layer only, not this task).

---

## 🔨 Suggested Implementation

1. Resolve the blocking pre-decision (see top of this file) and record the
   choice.
2. Add a shared `event_score(magnitude: f32, n_confounders: u32) -> f32`
   (or similar) in `sim.rs`, next to or reusing the existing confounder
   weight computation at `src/sim.rs:387-394`. Both the notebook's
   observation weighting and this task's event ranking should call the
   same function.
3. In `build_era_reveal`, build a `Vec` of candidate events for the closing
   era from `EraTally` (population deltas, extinctions), confirmed
   relations logged this era, environmental-shock data if tracked, and
   `EraEvolutionReveal` entries already collected — score each, sort
   descending.
4. Replace the `RevealTier` heuristic (`src/sim.rs:1066-1073`) with a
   score-threshold read from new `SimConfig` fields.
5. Add the fragment-grammar module (new file or a section of `text.rs`,
   depending on the Step-0 decision) with a small seeded-choice helper
   keyed on `(world_seed, era)`, reusing `SimWorld`'s existing RNG
   infrastructure rather than a new `rand` source.
6. Wire the top 1-2 ranked candidates into the reveal's generated sentence
   via the fragment grammar; wire the ambiguity path when scores are
   close.
7. Extend tests per the acceptance criteria.

---

## ⚠️ Constraints and Caveats

- **The blocking pre-decision is not optional and not this document's call
  to make** — see the section at the top. Do not start fragment-pool code
  before it's resolved and recorded.
- **Do not build a second causal-attribution system.** The design doc is
  explicit: the notebook's confounder-weight formula (`1/(1+n_confounders)`)
  *is* the signal-cleanliness half of the event score — reuse it exactly,
  don't approximate or reimplement it for narrative purposes. Two systems
  that can silently drift apart and describe the same event differently is
  called out as a real risk, not a hypothetical one.
- **Exact numeric thresholds are playtest territory**, not something to
  invent confidently here — same status as other first-pass constants in
  `sim.rs` (`MATURING_HINT_FRACTION`, `STALL_BAND`). Put them in
  `SimConfig` with a clearly documented "indicative, needs playtest" doc
  comment rather than treating them as final.
- **No raw internal numbers in player-facing text** — same rule task 170
  already followed for `genome_edit_line`; translate to natural language,
  never print floats/indices directly.
- **Keep `sim`/`world`/`config` free of `bevy::render`/`bevy_egui`** — the
  ranking and scoring logic belongs in `sim.rs`/`config.rs`; only the
  fragment rendering and any UI wiring touch `text.rs`/`screens.rs`.
- **Determinism**: fragment selection must be seeded from world seed + era
  (or pulse), never a global/thread RNG — CLAUDE.md's determinism rule is
  non-negotiable here, and the design doc calls this out explicitly (§2:
  "seeded, not truly random").
- Task 155 (trait archetype rework: 3-letter codes replacing Greek glyphs,
  `tasks/QUEUE.md:247`) is queued before this one but not yet done as of
  this writing, and this task does not depend on it landing first — if 155
  ships first, any tag-name fragment in this task's pool should read from
  whatever naming lookup 155 leaves in place (currently
  `notebook::translated_tag_label`, task 144) rather than hardcoding glyphs.
- The optional LLM-based cosmetic layer mentioned in the design doc is
  explicitly out of scope — do not add a runtime API dependency as part of
  this task.

---

## 🔗 Dependencies

- **Depends on**: 142 (`dominant_stimulus_clause`, extended here), 170
  (`genome_edit_line`, same register this task's fragments must match).
  Not hard-blocked by 155/156 (trait archetype rework) — see caveat above.
- **Blocks**: none directly, but `RevealTier`'s doc comment
  (`src/sim.rs:903-908`) frames this task as the intended replacement for
  its current heuristic, so any future task that further tunes reveal
  presentation should read tier from this task's score, not reintroduce a
  third heuristic.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/157-narrative-generation.md)"$'\n\nExecute this task in the current project.'
```
