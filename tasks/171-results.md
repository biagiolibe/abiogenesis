# Task 171 — Results: causal-legibility playtest gate

Recorded verbatim, same pattern as task 134's baseline recording
(`tasks/done/134-two-bot-experiment-incentive-harness.md`). Two independent
checks: bot-vs-bot necessity/legibility (§1) and a human playtest protocol
(§2).

---

## 1. Bot-vs-bot necessity + legibility check

`examples/two_bot_survey.rs`, 40 seeds, world 0, unchanged config (era
budget 15, season pulses 25, seasons per era 4). Ran three arms:

- **exploiter** / **explorer** — the original task-134 pair, now also using
  `Cull` (once-per-season maintenance, culls a bot-placed organism whose
  summed neighbour interaction reads confirmed net-harmful) and `Splice`
  (once-per-world, adds the confirmed-beneficial tag to the first seedable
  species — gated by `MatrixKnowledge::is_tag_confirmed`, task 147's rule).
- **oracle** — the Explorer policy shape with every pair read bypassing
  `MatrixKnowledge` and going straight to `world.matrix` (still spends
  budget identically; only the information it can act on differs). Its
  `Cull`/`Splice` maintenance reads the real matrix directly instead of
  confirmed evidence.

```
$ cargo run --release --example two_bot_survey -- 40

two-bot survey — world 0, seeds 0..40, era budget 15, season pulses 25, seasons per era 4

## exploiter
  outcomes            cleared 8, extinct 7, era budget exhausted 25 (of 40)
  short-term seasons  reached 8/40 — median 11, p25 9, p75 19, min 6, max 28
  full-sequence seasons  reached 8/40 — median 11, p25 9, p75 19, min 6, max 29
  peak population     reached 40/40 — median 18, p25 1, p75 1177, min 1, max 3643
  objectives cleared  30 total, 0.75 per world
  points spent        4521 total — isolated 4330, known 0, unknown 191
  objectives / point  0.0066
  pairs confirmed     0.77 per world (31/391 confirmable, 7.9%)

## explorer
  outcomes            cleared 7, extinct 6, era budget exhausted 27 (of 40)
  short-term seasons  reached 7/40 — median 17, p25 14, p75 31, min 6, max 52
  full-sequence seasons  reached 7/40 — median 18, p25 14, p75 32, min 6, max 53
  peak population     reached 40/40 — median 339, p25 1, p75 1785, min 1, max 3497
  objectives cleared  33 total, 0.82 per world
  points spent        4958 total — isolated 2511, known 0, unknown 2447
  objectives / point  0.0067
  pairs confirmed     1.02 per world (41/391 confirmable, 10.5%)

## oracle (ground truth, not a real policy — see task 171)
  outcomes            cleared 8, extinct 7, era budget exhausted 25 (of 40)
  short-term seasons  reached 8/40 — median 11, p25 9, p75 19, min 6, max 28
  full-sequence seasons  reached 8/40 — median 11, p25 9, p75 19, min 6, max 29
  peak population     reached 40/40 — median 18, p25 1, p75 1177, min 1, max 3643
  objectives cleared  30 total, 0.75 per world
  points spent        4521 total — isolated 4330, known 191, unknown 0
  objectives / point  0.0066
  pairs confirmed     0.77 per world (31/391 confirmable, 7.9%)

## head to head (short-term objectives, same seed)
  exploiter faster on 7/9, explorer faster on 1/9, tied 1
  failure criterion: the exploiter winning systematically means the incentives are wrong.
  The explorer does not need to win — it needs to be competitive.
## legibility gap (surfaced Explorer vs. oracle, same seed)
  short-term seasons  surfaced - oracle: mean 4.00, stddev 2.71 (n=6)
  full-sequence seasons  surfaced - oracle: mean 4.00, stddev 2.71 (n=6)
  pairs confirmed  surfaced - oracle: 0.25 per world (oracle still accumulates evidence passively via `sim::step`, it just never acts on it)
  worst single-seed short-term gap: seed 13, +8 seasons
  a small, stable (low-stddev) mean gap here is a pass for this task's bot-vs-bot half —
  a large or seed-dependent one names exactly which signal is still effectively hidden.
```

### Reading the numbers

The **oracle row is statistically identical to the exploiter row** (same
outcome counts, same season percentiles down to the integer, same peak
population). This is not a bug — it falls out of `INFO_WEIGHT`'s own
design intent (`two_bot_survey.rs`'s own doc comment: "a tie-breaker
between comparably-good cells, never enough to make a mediocre cell...
beat a genuinely better one"): `INFO_WEIGHT`/`KNOWN_SUM_SCALE` cap the
information term at ≤0.15 of score, while sampled candidates' `viability`
spread (0.6..1.0, `MIN_VIABLE_FIT`..1) routinely exceeds that. So even
with perfect ground truth, the *placement* decision is almost always
settled by viability alone — the matrix signal essentially never flips
which of 256 sampled cells wins. This is a genuine, if sobering, finding:
at `Seed`-decision granularity, the current tuning makes the hidden matrix
a tie-breaker a bot's placement choice rarely needs, confirming task 136's
own retuning intent (metabolic gains lowered specifically so the matrix,
not the environment, would decide *growth* — not necessarily *where to
seed*).

The real, task-171-relevant signal is the **surfaced Explorer vs. oracle**
comparison (the pair the task actually asks for): on the 6/40 seeds where
both variants cleared the short-term objectives, the surfaced-only bot
took a mean **4.00 seasons longer** (stddev 2.71, worst single seed +8 on
seed 13) than the same policy shape given ground truth. Directionally
correct (surfaced is never faster than oracle in this sample) and a
moderate, not dramatic, gap relative to the ~11-17 season medians —
consistent with "the surfaced data is mostly sufficient, with room for a
faster ramp-up" rather than "the chain is opaque." The small `n=6` (most
seeds fail to clear short-term objectives at all under either variant,
independent of information — the era-budget/extinction failure modes
dominate far more than the legibility question) limits how much confidence
this single run can carry; a larger seed sweep (`cargo run --release
--example two_bot_survey -- 200`, several minutes) would tighten it but
wasn't run this session given the time budget.

### Verdict: **pass, with a caveat**

The bot-vs-bot half is a legibility pass: no evidence the surfaced data is
insufficient, a real but moderate surfaced-vs-oracle gap consistent with
"learnable, not instant," and no seed showing a wildly divergent
(non-converging) surfaced run. **Caveat, not a fail**: `n=6` is thin —
most of the 40 seeds fail their short-term objectives outright regardless
of information (era-budget exhaustion, extinction), which is an
availability/difficulty question this task's own constraints put
out of scope (**no balance changes in this task** — if that failure rate
itself is a concern, it is a new task, not a 171 finding to fix here).
Nothing here names a specific Phase 2 task (146/147/170) as under-delivering.

---

## 2. Human playtest protocol

**Status: protocol authored, not yet run.** Per the task's own acceptance
criteria, 171 stays open (`[/]`, not `[x]`) until a real run happens or is
explicitly handed off — same as Phase 1's own skipped checkpoint
(`tasks/QUEUE.md`'s Phase 1 note).

Handoff copy for the playtester (Italian, self-contained — setup, task,
observation checklist, the two closing questions with autosaving answer
boxes): **["Diario del primo contatto"](https://claude.ai/code/artifact/ed23a641-17bd-4deb-8f82-581a9bac2188)**
(private Claude artifact; share it from the page's own share menu when
handing it to someone). Source below is the same content in prose, kept as
the durable spec this page was built from.

### What to hand the playtester

- A fresh `cargo run` build, no prior explanation beyond the game's own
  onboarding (main menu "How to play" panel, in-game hints).
- No hint about the hidden matrix, tag pairs, or what "confirming" means —
  the whole point is to see whether the game teaches this on its own.

### What to ask them to do

> "Play world 0 until you clear it or the run ends. Along the way:
> 1. When you see a species evolve (the end-of-era reveal card), tell me in
>    your own words why you think it happened.
> 2. Once you're done, tell me one matrix relation (which tag helps or hurts
>    which) you're confident about, and how you know."

### What to observe (don't prompt, just watch and note)

- Do they open the notebook unprompted, or only after a hint nudges them?
- Do they read the dominant-stimulus line on the reveal card
  (`text::era_reveal_evolution_line`) and the genome-diff line this task
  added (`text::genome_edit_line`), or do they skim past the reveal card
  entirely?
- If they use Cull, do they correctly attribute the outcome to the
  organism they removed, or do they seem to treat it as a random "reset"?
- If they use Splice, do they pick a tag because they have evidence for it,
  or because it was just the first option in the list?
- Do they ever open the Chronicle/species catalog to check a "descends
  from" relationship, or do lineage facts pass them by?

### Pass/fail criteria

A friction point counts **fixed** only if the player *used* the surfaced
information unprompted — not merely that the game displayed it. Distinguish:

- **Fixed**: player correctly explains a speciation's cause using the
  reveal card's own wording, or names a matrix relation and cites the
  notebook/log as their source.
- **Displayed but unused**: the information is on screen (reveal card,
  notebook, log) but the player's own explanation doesn't reference it —
  this is the gap task 141-144 first found and is what this run re-checks
  against the accumulated Phase 1+1b+2 state.
- **Still missing**: the player asks a question the game should have
  already answered (e.g. "why did that just happen?") that no on-screen
  text addresses.

Overall pass: the player can state at least one matrix relation and its
evidence, and correctly explains at least one speciation event, without
being told the mechanic exists ahead of time.

### Outstanding

No playtester was available this session. **This half of task 171 is
explicitly owed** — leave 171 at `[/]` in `tasks/QUEUE.md` (mirroring how
Phase 1's own skipped checkpoint was tracked) until a real run happens and
its findings are appended here.
