# Abiogenesis

Roguelike di simulazione emergente in Rust + Bevy: si semina vita su mondi alieni e si fa reverse-engineering di una matrice biochimica nascosta.

## Comandi

```bash
cargo run                      # avvia il gioco
cargo test                     # test unitari + determinismo/bilanciamento
cargo clippy -- -D warnings    # deve essere pulito prima di chiudere un task
cargo fmt
```

## Documenti

| File | Cosa contiene |
|---|---|
| [`abiogenesis-gdd.md`](abiogenesis-gdd.md) | **Design — fonte di verità.** Meccaniche, formule del tick (§5.6), baseline numerica (§5.9). |
| [`TECH_DESIGN.md`](TECH_DESIGN.md) | Architettura: plugin, stati, `SystemSets`, invarianti. |
| [`tasks/QUEUE.md`](tasks/QUEUE.md) | **Cosa fare adesso.** |

## Convenzioni

- **Codice e commenti in inglese**; documenti in italiano.
- **Un modulo = un `Plugin` Bevy.**
- **La griglia è una `Resource`, non entità ECS.** Le entità Bevy esistono solo per la resa.
- **La simulazione è deterministica ed eseguibile headless**: RNG nello stato del mondo, niente `rand::rng()`, niente iterazione su `HashMap`, niente query parallele nella logica del tick. `sim`/`world`/`config` non dipendono da `bevy::render` né da `bevy_egui`.
- **Nessun numero magico**: tutti i coefficienti stanno in `SimConfig` (`src/config.rs`).

Il razionale di queste regole è in `TECH_DESIGN.md` §5. Non aggirarle: se un task sembra richiederlo, il task è sbagliato.

## Workflow (Meridian)

Un task alla volta. A task completato:

1. verifica gli acceptance criteria nel task file;
2. sposta il file da `tasks/` a `tasks/done/`;
3. aggiorna lo stato a `[x]` in `tasks/QUEUE.md` e in `PROJECT_PLAN.md`.
