# Task 121 — Conditional-tag catalog badge never renders in a live playtest

> **ID**: `121`
> **Category**: Bugfix / UI
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-12 (reported live during a playtest of already-shipped
> tasks 097/103/105)

---

## 🎯 Objective

Task 097 added a catalog badge (`conditional_tag_badge`, `src/notebook.rs`
~1109) that's supposed to show a terrain glyph plus an `↑`/`↓` marker next
to a conditional tag once its `(TagSlot, TerrainKind)` gate is confirmed
(`TerrainKnowledge::is_confirmed`). In a live `cargo run` session the
Observation log clearly showed confirmations firing —
`"Confirmed: tag β turns on on plains"` and
`"Confirmed: tag α turns off on plains"` — for tags that ARE present on a
species' catalog row (Cass and Rook both carry β and α), but the badge
never appeared next to those tags. Verified with zoom: the marker is
entirely absent, not just small or low-contrast.

This task exists to find the actual root cause and fix it — not to
re-guess from static reading, which this session already did without
success (see Technical Context).

---

## 📋 Acceptance Criteria

- [x] Root cause identified — via targeted code tracing of the exact event
      payload/consumer path, not the earlier session's shallower static
      pass (see "Root cause" below).
- [x] A regression test added that reproduces the bug outside of egui's
      rendering (`notebook.rs::
      accumulate_terrain_evidence_confirms_the_trigger_terrain_even_when_observed_elsewhere`).
      It failed against the pre-fix code (evidence landing on the
      incidental observed terrain, never on the tag's own trigger terrain)
      and passes against the fix.
- [x] Fix applied: `accumulate_terrain_evidence` (`notebook.rs`) now
      records evidence at `conditional.terrain` (the tag's one designated
      trigger terrain) instead of `event.terrain` (whatever terrain the
      organism happened to be standing on when the gate was evaluated).
      `conditional_tag_badge` was already querying `conditional.terrain`
      exclusively, so this makes the write side agree with the read side —
      the badge now renders for a confirmed `(TagSlot, TerrainKind)` pair
      on every species row carrying that tag. The confirmation log message
      was also switched to `conditional.terrain` for the same reason (it
      previously could name whichever terrain the organism was standing on
      at the exact tick evidence crossed the threshold, not necessarily the
      terrain the tag is actually gated on).
- [x] `cargo test` and `cargo clippy -- -D warnings` clean.
- [ ] Live verification skipped for this pass at the user's explicit
      request — not run via `cargo run`. See "Root cause" below for why
      static tracing was sufficient this time despite the task's original
      framing assuming it wouldn't be.

### Root cause

`sim.rs::tag_gate_satisfied` pushes a `TerrainGateObserved { tag, terrain,
passed }` on **every** gate evaluation (pass or fail), where `terrain` is
`carrier_terrain` — the terrain the organism is standing on *right now*,
not the tag's own trigger terrain. For `Mode::Repressible`, the gate
*passes* precisely when `carrier_terrain != conditional.terrain` — i.e.
most passing evaluations happen on terrains other than the trigger one.
Symmetrically, `Mode::Inducible` *fails* off the trigger terrain just as
often. `accumulate_terrain_evidence` was recording evidence at
`(event.tag, event.terrain)` — the incidental terrain from the event —
while `conditional_tag_badge` only ever queries
`is_confirmed(slot, conditional.terrain)`, the tag's single fixed trigger
terrain. Evidence for a given tag was fragmenting across every
`TerrainKind` the species' organisms actually stood on instead of
concentrating on the one cell the badge reads, so the badge's threshold
was reachable only via the narrow subset of evaluations that happened to
occur exactly on `conditional.terrain` — for `Repressible` tags (and
`Inducible` failures) that subset is small or can go a long time without
being hit, matching the live report's "confirmed in the log, badge never
appears" symptom exactly (the log used `event.terrain` too, so it could
print a terrain name unrelated to the tag's actual gate).

This diverges from the earlier session's static pass because that pass
checked confirmation *transitions* (`record`'s return value, `is_confirmed`
staying true) and *resource identity* (single `TerrainKnowledge` instance,
matching `TagSlot`/`TagId` resolution) — all correct in isolation — without
comparing the *terrain argument* the write side (`accumulate_terrain_
evidence`) uses against the one the read side (`conditional_tag_badge`)
uses. The existing regression test for the confirm+log path
(`accumulate_terrain_evidence_confirms_and_logs_once`) only ever exercised
an `Inducible` tag with `passed: true` and `event.terrain == conditional.
terrain` — the one case where `event.terrain` and `conditional.terrain`
coincide by construction, so it could not have caught this.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/notebook.rs` | `TerrainKnowledge` (~190-236), `accumulate_terrain_evidence` (~318-353, confirms + logs), `catalog_panel` (~1035-1098), `conditional_tag_badge` (~1109-1130, the badge that fails to render). Multiple `TerrainKnowledge::new`/`insert_resource` call sites also live here. |
| `src/menu.rs` | Another `TerrainKnowledge::new` / `insert_resource` call site (~157) — worth checking whether this one, `run_flow.rs`'s reset, and `notebook.rs`'s `NotebookPlugin::build` insertion can end up producing more than one live copy, or reset one out from under the render path. |
| `src/run_flow.rs` | `TerrainKnowledge` reset on world transition (~92) — same "could this clobber accumulated evidence right when the player checks the catalog" question. |
| `src/sim.rs` | `tag_gate_satisfied` (emits `TerrainGateObserved` with the `TagSlot` that should match `species.tags` exactly). |
| `src/world.rs` | `conditional_gate` (~904), `ConditionalTag`. |

---

## 🧩 Technical Context

- **Current behavior**: the confirmation-and-log half of the feature works
  — the Observation log correctly announces a `(tag, terrain)` pair as
  confirmed. The badge that's supposed to reflect that same confirmed
  state in the Catalog panel never shows up.
- **Desired behavior**: once `"Confirmed: tag X turns on/off on terrain"`
  has logged for a `(TagSlot, TerrainKind)` pair, every catalog row for a
  species carrying that tag shows the terrain glyph + `↑`/`↓` marker next
  to it, permanently (evidence never un-confirms).
- **What this session's static review already checked and ruled out**
  (don't re-derive this from scratch — start from here):
  - `TerrainKnowledge::record` (`notebook.rs` ~211-216) flips
    `is_confirmed` permanently `true` for the exact `(TagSlot,
    TerrainKind)` index the moment `evidence >= threshold` — evidence only
    grows, never resets mid-world, so once confirmed it stays confirmed.
  - `accumulate_terrain_evidence` (`notebook.rs` ~318) calls `record` with
    `event.tag`/`event.terrain` straight from `TerrainGateObserved`, and
    only logs `terrain_gate_confirmed_message` when `record` returns
    `true` — i.e. the log line and the confirmed state are written
    atomically together, no way for one to fire without the other.
  - `conditional_tag_badge` (`notebook.rs` ~1109) is called from
    `catalog_panel` once per `slot` in `species.tags`, and checks
    `terrain_knowledge.is_confirmed(slot, conditional.terrain)` where
    `conditional` comes from `world.conditional_gate(tag)` — the same
    `TagId` the badge itself resolves via `world.active_tags[slot.0 as
    usize]`. This is the same `TagSlot` space `tag_gate_satisfied` uses
    when it emits `TerrainGateObserved` in the first place
    (`sim.rs`'s `their_tag`/`my_tag`, drawn from `neighbour_species.tags`/
    `species.tags`).
  - Existing tests (`notebook.rs::accumulate_terrain_evidence_confirms_and_logs_once`,
    `terrain_knowledge_record_reports_the_confirmation_transition_exactly_once`)
    cover the confirmation+log half end-to-end and pass. **No existing
    test exercises `conditional_tag_badge`/`catalog_panel` at all** — the
    render half of this feature has zero coverage, which is presumably how
    this shipped broken.
  - `terrain_glyph(TerrainKind::Plain)` renders `"·"` (U+00B7, a plain
    middle dot) — ruled unlikely to be a missing-glyph/tofu issue like task
    119's icon problem, since it's an extremely common character, but
    worth a second look given the marker is *entirely* absent, not just
    small.
- **Candidates worth checking live, in rough order of suspicion**:
  1. Multiple `TerrainKnowledge::new`/`insert_resource` call sites
     (`notebook.rs`, `menu.rs`, `run_flow.rs`, `input.rs` test-only) — is it
     possible a later insertion (e.g. on a screen transition) replaces the
     resource `catalog_panel` reads with a fresh, unconfirmed one *after*
     the confirmation already logged, so the log and the badge are reading
     two different instances by the time the player checks the notebook?
  2. Is `species.tags` at render time still the same list the confirming
     organism actually carried, or could a `Splice`/respawn have changed
     it in a way that decouples the two?
  3. An egui-specific rendering issue (color/contrast against
     `tag_color(tag)`, an unexpected clip rect, ordering with something
     else drawn on top) — check this only after ruling out 1 and 2, since
     it's the hardest to verify without visual reproduction.

---

## 🔨 Suggested Implementation

1. Reproduce live: `cargo run`, seed a species with a conditional tag,
   expose it repeatedly to its trigger terrain until the Observation log
   confirms it, then open the Catalog and check the row.
2. If reproducible, add debug instrumentation (a temporary `info!` in
   `conditional_tag_badge` printing `is_confirmed`'s result and the
   `TerrainKnowledge` resource's evidence for that pair) to see directly
   whether the badge's own read of `is_confirmed` disagrees with what the
   log just announced, or whether the badge's early-return conditions
   (`conditional_gate` returning `None`) are the actual culprit.
3. Once the actual mismatch is found, write a regression test that
   reproduces it without egui (e.g. an `App`-based test asserting the same
   `TerrainKnowledge` resource instance that `accumulate_terrain_evidence`
   wrote to is the one `catalog_panel`'s system reads, if that's the bug).
4. Fix the root cause.
5. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`.
6. Live re-verify per the acceptance criteria.

---

## ⚠️ Constraints and Caveats

- Don't guess-and-patch without confirming the actual root cause first —
  this session already tried static analysis alone and it wasn't enough;
  the acceptance criteria require a reproduction step before a fix is
  considered valid.
- Keep the fix scoped to making the existing badge work — this is not an
  invitation to redesign the badge's presentation (glyph choice, marker
  characters, etc.) unless the redesign *is* the actual root cause.

---

## 🔗 Dependencies

- **Depends on**: 096, 097 (the feature this fixes).
- **Blocks**: none.

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/121-terrain-badge-missing-in-catalog.md)"$'\n\nExecute this task in the current project.'
```
