# Task 001 — Toolchain, scaffold Cargo e app Bevy a plugin

> **ID**: `001`
> **Categoria**: Architettura
> **Priorità**: 🔴 P1
> **Stima**: ~1.5h
> **Assegnato a**: non assegnato
> **Sessione**: —

---

## 🎯 Obiettivo

Portare il progetto da "cartella con soli documenti" a **applicazione Bevy che si avvia e apre una finestra**, con la struttura a plugin già in piedi (stub vuoti) su cui tutti i task successivi andranno a innestarsi.

È necessario perché il progetto **non è scaffoldato**: in root non esistono né `Cargo.toml` né `src/`. Ogni altro task ne dipende.

---

## 📋 Acceptance Criteria

- [ ] `rustc --version` riporta **1.97.1** e `rust-toolchain.toml` pinna quella versione.
- [ ] `cargo build` compila senza errori.
- [ ] `cargo run` apre una finestra con titolo `Abiogenesis` e sfondo uniforme (nessun contenuto: è corretto).
- [ ] `cargo clippy -- -D warnings` è pulito.
- [ ] I sei plugin esistono come stub e sono registrati in `main.rs`.
- [ ] Le versioni esatte risolte da `cargo add` sono riportate in `TECH_DESIGN.md` §1, sostituendo la nota "Da completare nel task 001".
- [ ] `.gitignore` copre `/target` (già presente: verificare).

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `rust-toolchain.toml` | Pinna la toolchain (nuovo) |
| `Cargo.toml` | Dipendenze e profili di build (nuovo) |
| `src/main.rs` | Entry point, registrazione dei plugin (nuovo) |
| `src/config.rs`, `src/world.rs`, `src/sim.rs`, `src/render.rs`, `src/ui.rs`, `src/input.rs` | Stub dei plugin (nuovi) |
| `TECH_DESIGN.md` | §1 da aggiornare con le versioni risolte |

---

## 🧩 Contesto Tecnico

- **Comportamento attuale**: la root contiene solo documenti Markdown e un repo git inizializzato senza commit. Nessun codice Rust.
- **Comportamento desiderato**: `cargo run` apre una finestra Bevy.

**Vincolo di versione verificato:**

| Componente | Versione | Nota |
|---|---|---|
| Toolchain locale attuale | 1.90.0 | **insufficiente** |
| Bevy 0.19 | richiede Rust **≥ 1.95.0** | |
| Stable disponibile | **1.97.1** | va installata |
| `bevy_egui` | **0.41** | l'accoppiamento con bevy 0.19 è verificato (egui 0.35) |

*Fallback se l'aggiornamento della toolchain non è praticabile:* bevy `0.18.1` + bevy_egui `0.39.x` girano su Rust 1.90. In quel caso aggiornare di conseguenza `TECH_DESIGN.md` §1 e questo task file.

---

## 🔨 Implementazione Suggerita

1. **Toolchain**

   ```bash
   rustup update stable
   rustc --version   # atteso: 1.97.1
   ```

   Creare `rust-toolchain.toml`:

   ```toml
   [toolchain]
   channel = "1.97.1"
   components = ["rustfmt", "clippy"]
   ```

2. **Scaffold** — **`init`, non `new`**: la directory contiene già file.

   ```bash
   cargo init --name abiogenesis
   cargo add bevy@0.19
   cargo add bevy_egui@0.41
   cargo add rand
   ```

3. **Profilo di build.** Senza questo, la simulazione in debug è inguardabile: le dipendenze vanno compilate ottimizzate anche in dev. In `Cargo.toml`:

   ```toml
   [profile.dev]
   opt-level = 1

   [profile.dev.package."*"]
   opt-level = 3
   ```

4. **Stub dei plugin.** Un file per modulo, ciascuno con un `Plugin` vuoto:

   ```rust
   use bevy::prelude::*;

   pub struct ConfigPlugin;

   impl Plugin for ConfigPlugin {
       fn build(&self, _app: &mut App) {}
   }
   ```

5. **`main.rs`** — `DefaultPlugins` con la finestra configurata, più i sei plugin del progetto:

   ```rust
   fn main() {
       App::new()
           .add_plugins(DefaultPlugins.set(WindowPlugin {
               primary_window: Some(Window {
                   title: "Abiogenesis".into(),
                   ..default()
               }),
               ..default()
           }))
           .add_plugins((
               ConfigPlugin,
               WorldPlugin,
               SimPlugin,
               GridRenderPlugin,
               UiPlugin,
               InputPlugin,
           ))
           .run();
   }
   ```

6. Riportare le versioni risolte (`cargo tree --depth 0` o `Cargo.lock`) in `TECH_DESIGN.md` §1.

---

## ⚠️ Vincoli e Attenzioni

- **`cargo init`, non `cargo new`** — la cartella non è vuota.
- **`EguiPlugin` non va ancora registrato**: è il task 008. Qui `bevy_egui` è solo una dipendenza dichiarata.
- **Nessuna logica di gioco in questo task.** Gli stub restano vuoti; riempirli è compito dei task 002+.
- L'API di `rand` ≥ 0.9 differisce da 0.8 (`thread_rng` → `rng`, `gen` → `random`): verificare la versione risolta prima di scrivere codice che la usa (task 003).
- La prima build di Bevy scarica e compila molto: mettere in conto diversi minuti.

---

## 🔗 Dipendenze

- **Dipende da**: nessuno
- **Blocca**: 002 (e a cascata tutti gli altri)

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/001-scaffold-bevy.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
