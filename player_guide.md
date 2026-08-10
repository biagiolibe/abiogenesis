# Abiogenesis — Player Guide

You are a xenobiologist. You've just landed on an alien world with life you don't understand yet — species that react to each other through a hidden biochemistry only you can uncover, one experiment at a time.

There are two mysteries running at once: **what will happen** — the ecosystem you seed lives its own unpredictable life — and **what the rules are** — a secret matrix of biochemical interactions, different every run, that you decode by watching what happens when species meet. Nobody hands you the answer. You earn it by experimenting cleanly.

---

## Controls

| Key / action | Effect |
|---|---|
| Left click | Perform the selected action (Seed / Stress / Cull) on the clicked cell |
| `space` | Advance one full era (a block of ticks, animated) |
| `n` | Advance a single tick (fine-grained observation) |
| `wasd` / arrow keys | Pan the camera |
| `tab` | Open / close your notebook |
| `r` | Reseed the current world (same difficulty, fresh random state) |
| `t` | Toggle a temperature heatmap over the grid (blue = cold, red = hot) |
| `l` | Toggle a light heatmap over the grid (blue = dark, red = bright) |
| `Esc` | Quit |

The HUD's right-hand console lets you pick which species to seed (**Species**), which action is active (**Moves**), and shows your biosphere and its trends (**Biosphere**) and current goal (**This world wants**) at a glance.

---

## The core loop

Each world plays out as a repeating cycle:

1. **Seed** — place organisms on an empty grid. Nothing starts pre-placed: the first move is always yours.
2. **Advance an era** — press `space` and watch the ecosystem live for a block of ticks. Organisms gain and lose energy, reproduce, starve, get eaten, decompose.
3. **Observe** — note what happened: who bloomed, who collapsed, which pairings of species seemed to hurt or help each other.
4. **Hypothesize and intervene** — spend your era's action budget on a deliberate experiment: seed more of something, stress the environment, cull a species, or splice a genome.
5. **Repeat** until the world's objective is met — then move on to a harder world.

The heart of it is the small loop inside the big one: **hypothesis → experiment → *aha*.** That's the moment a notebook cell lights up and you actually know something you didn't before.

---

## Species and metabolism

Every species has a readable side and a hidden side.

**Readable** (shown in the HUD, no guesswork needed):
- **Metabolism** — where its raw energy comes from:
  - *Photolithic* draws energy from local light.
  - *Predator* draws energy from neighboring organisms.
  - *Decomposer* draws energy from residue in its own cell or a neighboring one — it needs to actually be adjacent to leftover residue, not just anywhere.
- **Preferred temperature** — every metabolism's gain above is *multiplied* by how close the cell's temperature is to the species' comfort zone, not added to as a separate cost. This applies equally to all three: a Decomposer sitting right next to abundant residue can still starve to death if that cell's temperature is far from its optimum — the residue being there isn't enough on its own. If an organism keeps dying despite its metabolism's fuel being visibly present (residue, light, prey), suspect a bad temperature fit before suspecting a hidden matrix effect.
- **Reproduction threshold** — once an organism has enough energy, it reproduces into an empty neighboring cell. The threshold is the same for every species and shown in the notebook's species catalog (`tab`), alongside metabolism and temperature range. The HUD's Biosphere section instead shows each species' population trend since the last era (▲ rising / ▼ falling / ▬ stable) — actual births are logged as a per-era summary ("Kael: +3 births this era") when they happen.

**Hidden:**
- **1 to 3 biochemical tags** per species — shown only as nameless glyphs and colors. Tags are the *only* thing that determines how two adjacent species affect each other, and that effect is defined by a secret matrix generated fresh for every world. You never see the matrix directly — you infer it from what happens when tagged organisms sit next to each other.

The matrix is **directional**: species A carrying a tag that harms species B's tag doesn't mean B harms A back. Figuring out that asymmetry is usually the first real discovery of a run.

---

## Actions and your budget

Each era gives you a small budget of action points (3 by default) to spend before you advance time. This is deliberate: you can't blanket the grid, you have to bet on your best hypothesis.

| Action | Cost | What it does |
|---|---|---|
| **Seed** | 1 | Place an organism of the selected species on an empty cell |
| **Stress** | 1 | Shift an environmental scalar (e.g. temperature) at a cell |
| **Cull** | 1 | Remove the organism on a cell |
| **Splice** | 2 | Edit a species' genome — swap or add a tag, or shift its thermal optimum |

`Splice` is the most powerful and most expensive tool: it lets you test a hypothesis directly by changing what a species *is*, not just where it sits.

---

## The notebook (`tab`)

This is where deduction actually happens. Open it any time — it doesn't pause anything or cost an action.

- **Observation log** — a curated feed of what mattered: extinctions, deaths of organisms you personally placed, and every matrix relationship the moment it's confirmed (marked with a `★`).
- **Hypothesis grid** — a graph of every active tag, with confirmed relationships drawn as colored directed edges (green = helps, red = harms). Unconfirmed pairs stay blank until you've gathered enough evidence.
- **Catalog** — every tag and species you've encountered, with what's known about each so far.

### How confirmation works

Every time two tagged organisms sit adjacent to each other, that's a data point. But not all data points are equal: an observation where the pair was **isolated** (no other tags confounding the signal) counts far more than one buried in a crowd. Concretely, each observation's weight is `1 / (1 + number of other adjacent tags)` — a clean pairing is worth 1.0, one with three confounders is worth only 0.25.

Once the accumulated evidence for a tag pair crosses a threshold, that relationship **confirms** — it lights up in the hypothesis grid and logs a `★` line. This is the real progress bar of the game: you're not accumulating score, you're accumulating *understanding*.

**Practical upshot:** if you want fast answers, engineer clean experiments. Seed a lone pair far from everything else and watch what happens between just the two of them, rather than reading the tea leaves of a crowded front.

---

## Objectives, victory, and failure

Every world sets a **sequence of goals** — 2 in the early worlds, 3 once the difficulty ramps up — shown in the HUD one at a time ("Objective i / N"). Each is one of:

- **Coexistence** — sustain N species at once for a number of eras.
- **Survive in a hostile zone** — get a species to survive in the world's toxic zone.
- **Trigger a bloom** — grow a specific species past a population threshold.

Clearing one objective moves you straight to the next in the same world — the world itself only clears once every objective in the sequence has. Meet the last one, and you move to the next world: more active tags, a meaner matrix, a harsher environment.

**Two ways a world goes wrong:**
- **Total extinction** — every organism dies. This retries the *same* world (same difficulty, fresh random draw), not your whole run — one bad world doesn't end everything you've built.
- **Running out of eras** — each world gives you a generous but finite number of eras. Exhaust it without meeting the objective, and the *run* ends: you're returned to the main menu, keeping whatever meta-progression you earned.

Every world also opens with a **grace period**: total extinction can't end it until you've kept a population alive for a full era at least once. If your first placement dies before then, nothing is lost — just reseed and keep watching. The HUD shows a "Grace period" line while it's active; it disappears the moment you've earned your first foothold, and doesn't come back for the rest of that world.

You don't need to decode the whole matrix to win a world — only the part relevant to the species you're actually using and the objective in front of you.

---

## Between runs

Progress between runs is deliberately light: clearing worlds unlocks a few extra species available at the start of your next run. The matrix itself is never something you unlock — every run starts from a completely fresh, undeciphered biochemistry. What carries over is *you getting better at the method*, not answers.

---

## Tips for your first run

- Your very first world always opens with the gentlest possible goal: get any 2 species coexisting. Later worlds can ask for more.
- Your very first placement of the run gets a one-time hint telling you whether it was isolated — pay attention to it, it's teaching you the core deduction trick.
- Don't crowd your first few placements. A lone organism (or an isolated pair) gives you the cleanest possible read on what's actually happening.
- Watch for deaths in the observation log, not just population totals — a death tells you *something* interacted badly, even before you know what.
- If a species is thriving in a spot with the wrong metabolism story (e.g. a photolithic organism doing fine in the dark), suspect a matrix effect, not the environment.
- The opposite case: if a species keeps *dying* despite its fuel visibly being there (a Decomposer next to residue, a Photolithic in bright light, a Predator with prey nearby), check the death log's gain breakdown before suspecting the matrix — a poor temperature fit silently shrinks that gain toward zero.
- Budget is tight on purpose. One well-chosen experiment beats three scattered ones.

---

## A note on balance

The core loop, mechanics, and systems described here are final — but the numeric tuning (how fast populations grow, how quickly the notebook confirms, how hostile late worlds get) is still under active adjustment based on actual play. If something feels broken rather than just *hard* — a species snowballs into filling the entire grid, or the whole ecosystem dies before you can learn anything — that's exactly the kind of thing worth flagging. The game is being balanced by watching real playthroughs, not by theory alone.
