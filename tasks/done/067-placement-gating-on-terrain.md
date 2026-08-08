# Task 067 — Placement gating on terrain

> **ID**: `067`
> **Category**: Feature
> **Priority**: 🟡 P2
> **Estimate**: ~2h
> **Assigned to**: unassigned
> **Session**: 2026-08-09 (from `redesign/abiogenesis-terrain-map.md`, follow-up to task 066)

---

## 🎯 Objective

Task 066 adds a per-cell terrain classification (Sea/Plain/Hill/Mountain +
peak flag) to `SimWorld`, generated deterministically per world. Nothing in
the simulation or input layer knows about it yet — organisms can be placed
and can reproduce onto any cell regardless of terrain.

This task makes terrain matter for placement, while keeping the door open
for a possible future aquatic species (discussed directly with the user):
the gating rule must live in one centralized place, not be duplicated as
`if terrain == Sea { block }` scattered across call sites.

---

## 📋 Acceptance Criteria

- [ ] The code compiles without errors; `cargo clippy -- -D warnings` is clean.
- [ ] A single centralized query point exists on `SimWorld` (e.g. `is_placeable(x, y) -> bool`) implementing today's rule purely from terrain: Sea and peak cells unplaceable for every current species, Plain/Hill/non-peak Mountain placeable. No species/tag-affinity infrastructure is built now — just make this the one place a future affinity check would extend.
- [ ] `seed_organism_on_click` (`src/input.rs:193`) refuses placement on an unplaceable cell, using the same feedback pattern the game already uses for a rejected Seed attempt (check `text.rs` for the existing message style rather than inventing a new one).
- [ ] Reproduction's neighbour filter (`sim.rs:361`, currently `world.cells[n].organism.is_none()`) also requires `is_placeable`, so offspring never spawn onto Sea/peak cells.
- [ ] New test: Seed on an unplaceable cell is rejected — no organism placed, action budget not spent (mirror how existing occupied-cell/out-of-budget Seed rejection is tested).
- [ ] New test: reproduction never spawns onto an unplaceable neighbour, even when it's the only Moore neighbour that's "empty" by occupancy alone.
- [ ] Existing tests continue to pass.

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/world.rs` | Home for the new centralized `is_placeable` query, alongside the terrain data from 066. |
| `src/input.rs` | `seed_organism_on_click` (line ~193) — player Seed action, needs the gate. |
| `src/sim.rs` | Reproduction neighbour filter (line ~361) — needs the gate. |
| `src/text.rs` | Existing player-facing message patterns for rejected actions — extend, don't invent a new style. |

---

## 🧩 Technical Context

- **Current behavior**: `Seed` and in-tick reproduction only check whether a cell already holds an organism (`organism.is_none()`); terrain is irrelevant to both today.
- **Desired behavior**: both also require the cell to be placeable per terrain — Sea and Mountain peaks are off-limits to every species that exists right now.

---

## 🔨 Suggested Implementation

1. Add `SimWorld::is_placeable(x, y)` (or equivalent), reading the terrain classification from task 066.
2. Update `seed_organism_on_click` to check it before placing, with matching player feedback.
3. Update the reproduction neighbour filter in `sim.rs` to check it too.
4. Add the two new tests described above.

---

## ⚠️ Constraints and Caveats

- **Single source of truth**: do not duplicate the Sea/peak check inline in both `input.rs` and `sim.rs` — both must call the same `SimWorld` method.
- **No speculative affinity system**: don't add species/tag-terrain compatibility now — that's a future task if/when an aquatic species is actually designed.

---

## 🔗 Dependencies

- **Depends on**: 066.
- **Blocks**: 068 (rendering benefits from placement being real, for visual consistency, though it doesn't strictly require 067's code).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/067-placement-gating-on-terrain.md)"$'\n\nExecute this task in the current project.'
```
