# Abiogenesis — why did it die: anchor causes vs. matrix mystery

Standalone design doc, raised directly by the user (2026-08-10/11) as a
companion topic to the notebook redesign discussion
(`abiogenesis-notebook-redesign.md`): concrete examples given were a species
dying from poor temperature fit and a predator dying from no nearby prey —
neither is communicated in any legible way today, even though the game
never intended either to be part of the hidden mystery. **Independent
track**, its own task(s) when scoped — touches `src/sim.rs`'s `OrganismDied`
event and the HUD's Biosphere panel, not `notebook.rs`'s window content.

## Current state

`text::player_organism_death_message` (`text.rs:389-408`) already exists
and already tries to "answer why instead of only what" (its own doc
comment) — but only for organisms the player personally placed (a marker-
based filter in `notebook.rs:319-360` deliberately avoids turning this into
an unfiltered per-tick feed, the same noise problem just fixed in the log
redesign). It renders five raw signed numbers: `gain`, `interaction_delta`
(the matrix term), `upkeep`, `crowding_penalty`, `predation_loss` — e.g.
"gain +0.40, matrix +0.00, upkeep -0.70, crowding -0.15, predation +0.00".
`player_guide.md` tells the player to "check the death log's gain
breakdown" for a poor temperature fit, but interpreting that requires
already understanding the tick formula (§5.6) — the data is present, the
message isn't actually legible.

**A real gap found while grounding this**: `OrganismDied` (`sim.rs:16-31`)
only exposes the *final* `gain`, already multiplied by `env_fit`
(`sim.rs:177/220/254/402`) — a bad temperature fit and a genuinely absent
resource (no light, no prey, no residue) both collapse into the same small
`gain` number today. There is currently no way, even server-side, to tell
"gain was low because of temperature" from "gain was low because there was
nothing to eat" — building the diagnosis proposed below needs `OrganismDied`
to carry one more signal (e.g. the `env_fit` value itself, or the
pre-multiplier raw resource magnitude), not just a `text.rs` rewrite.

## Why this doesn't fight the deduction pillar

GDD §7 is explicit: "**Metabolisms and environmental ranges always remain
readable as anchors**." Temperature fit and resource availability were
never meant to be part of the hidden mystery — only the tag×tag matrix is
(§11). So being fully direct about anchor causes loses no discovery value;
it just fixes a UX gap between "the game intends this to be legible" and
"the game actually shows it legibly."

## Decided in discussion

- **Split by cause type, not by uniform detail level**:
  - **Anchor causes** (temperature fit, missing resource for the
    organism's metabolism, overcrowding) → **plain, direct language**. No
    discovery cost, since these were never hidden by design.
  - **Matrix cause** (`interaction_delta` dominant) → **stays
    deliberately vague**: "harmed by a nearby species," no tag identity,
    no sign, no magnitude named. This is the one term that must protect
    the game's actual mystery.
- **Remove the exact `interaction_delta` number from the message** when the
  matrix term dominates — today's raw number is a small but real
  information leak to an attentive player, and it's inconsistent with the
  hypothesis-grid decision (task/notebook redesign) to stop showing raw
  numbers there too. Anchor-cause numbers can also drop in favor of their
  plain-language equivalent — nothing here needs to stay numeric for a
  player-facing message (raw values can still exist in a debug/F2 overlay
  if useful for the developer, unaffected by this change).
- **Scope: both** the existing player-placed death message (rewritten in
  plain language) **and** a new aggregate diagnosis on the HUD's Biosphere
  population-trend indicator (▲/▼/▬, already shown per `player_guide.md`)
  — covers both the single-organism case and the "my predators are
  quietly starving somewhere on the map" case the user's example named,
  without reintroducing per-organism log spam for organisms the player
  never placed.

## Proposed model

### Player-placed organism death (rewrite of existing message)

Pick the dominant negative contributor among `gain` shortfall (further
split into temperature-fit vs. resource-scarcity once `OrganismDied`
exposes that distinction), `upkeep`, `crowding_penalty`, `predation_loss`,
and `interaction_delta`, and render exactly one short qualitative sentence
per death instead of a five-term dump:

- Low gain, poor `env_fit` → *"died: the temperature here didn't suit it"*
- Low gain, resource absent (metabolism-specific: no light for
  Photolithic, no residue for Decomposer) → *"died: nothing to feed on
  here"*
- `predation_loss` dominant → *"died: eaten by a predator"* (already
  fairly directly observable today, not a new reveal)
- `crowding_penalty` dominant → *"died: too crowded here"*
- `interaction_delta` dominant → *"died: harmed by a nearby species"* (no
  number, no tag identity — the mystery stays intact)

### Aggregate Biosphere trend diagnosis (new)

- For a species with a falling trend (▼) in the HUD's Biosphere section,
  attach a short qualitative cause derived from the dominant term across
  that species' recent deaths in the current era (population-wide, not
  per-organism) — same taxonomy as above, one label next to the arrow
  rather than a log line per death.
- Reuses population data the tick loop is already computing (deaths are
  already tracked per species for the trend arrow itself); the new part is
  tallying dominant cause across those deaths, not new simulation state.

## Open questions for task scoping

- Exact `OrganismDied` schema change needed to distinguish temperature-fit
  from resource-scarcity within a low `gain` value (expose `env_fit`
  directly, or the pre-fit raw resource magnitude — pick whichever is
  cheaper to thread through `sim.rs:177/220/254`).
- Aggregation window/threshold for the Biosphere trend diagnosis (every
  death this era? a rolling window across eras?) and tie-breaking when two
  causes are similarly dominant across a population.
- Exact phrasing pass — the sentences above are placeholders for tone, not
  final copy.
- Whether the now-removed raw numbers should remain available anywhere
  (e.g. the existing F2 debug overlay) for development/tuning purposes —
  likely yes, since that overlay already exists for a different audience
  than the player-facing log.

## Scope note

Not scoped into task files yet. Two natural sub-tasks: (1) the
`OrganismDied`/message rewrite for player-placed deaths, (2) the Biosphere
trend aggregate diagnosis — (1) can land independently and doesn't block on
(2), since they touch different data paths (`text.rs` vs. HUD trend
computation in `render.rs`).
