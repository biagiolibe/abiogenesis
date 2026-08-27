# Processed design documents — do not read by default

**Directive: nothing in this directory is to be read or re-analysed as part of
normal work.** These documents have all been consumed: their proposals were
turned into the executable backlog (tasks 134-169) during the 2026-08-27
redesign adoption pass, and the decisions that survived that pass now live in
the canonical documents instead:

| Where the decisions live now | |
|---|---|
| [`abiogenesis-gdd.md`](../../abiogenesis-gdd.md) | The GDD (v0.7). Source of truth for mechanics and decisions. |
| [`tasks/QUEUE.md`](../../tasks/QUEUE.md) | What to do now, in order, with dependencies. |
| [`PROJECT_PLAN.md`](../../PROJECT_PLAN.md) | The same backlog with its rationale. |
| [`abiogenesis-INDEX.md`](../abiogenesis-INDEX.md) | The map of this corpus, annotated with the Phase 0 findings and the three corrections applied to it. |

**The single exception**: when a task file explicitly names one of these
documents as its design source (`Design source: ...`), open *that* document, read
*that* section, and stop there. Do not read around it, do not pull in the
neighbouring documents, and do not reopen decisions the backlog already records.

Reading this corpus in full costs a large fraction of a session's context and
produces nothing the canonical documents don't already say. If something here
contradicts the code, **the code wins** — that rule predates this directory and
still holds.
