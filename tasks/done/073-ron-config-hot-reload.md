# Task 073 — Migrate `SimConfig` to a hot-reloadable RON asset

> **ID**: `073`
> **Category**: Architecture
> **Priority**: 🟢 P3
> **Estimate**: ~2-3h
> **Assigned to**: unassigned
> **Session**: 2026-08-09, requested directly by the user to unblock fast iteration for the final-tuning backlog (074 and beyond)

---

## 🎯 Objective

GDD §5.6 asks for simulation coefficients to be "ideally hot-reloadable."
TECH_DESIGN.md's Asset Pipeline section already anticipated this: *"In Phase
0, `SimConfig` is built from constants in `config.rs`; during the tuning
phase it migrates to a RON asset with hot-reload. The structure should be
set up now (a single `Resource`, read and never duplicated), the
implementation not."* That phase has arrived — task 074 (final grid-size
tuning) and the rest of the "Final tuning" backlog in `PROJECT_PLAN.md`
depend on being able to change coefficients without a full recompile.

Migrate `SimConfig` (and every nested config struct in `src/config.rs`) from
hardcoded `impl Default` blocks to a RON file loaded via `bevy_asset`, with
hot-reload: editing the RON file on disk while `cargo run` is active updates
the live `SimConfig` resource without restarting the app.

---

## 📋 Acceptance Criteria

- [x] The code compiles without errors; `cargo clippy -- -D warnings` is clean.
- [x] Every config struct in `src/config.rs` (`SimConfig`, `GridConfig`,
      `EnvironmentConfig`, `TimeConfig`, `ActionCosts`, `EnergyConfig`,
      `TagConfig`, `NotebookConfig`, `DifficultyConfig`, `WorldgenConfig`,
      `ObjectiveConfig`, `TerrainConfig`) derives `serde::Serialize` +
      `serde::Deserialize` in addition to its existing derives.
- [x] `assets/config/sim_config.ron` holds the same numeric values as
      `impl Default`'s blocks — transcribed by hand from the source, not
      guessed.
- [x] `ConfigPlugin` loads this RON file via `bevy_common_assets`'s
      `RonAssetPlugin<SimConfig>` (`SimConfig` itself derives `Asset` +
      `TypePath`, no separate wrapper type) instead of
      `init_resource::<SimConfig>()`'s bare `Default` call. A
      `SimConfigHandle` resource tracks the load; `sync_sim_config_on_reload`
      reads `AssetEvent<SimConfig>` (`Added`/`Modified`) and clones the
      loaded asset into the live `SimConfig` resource.
- [x] Editing `assets/config/sim_config.ron` while `cargo run` is active and
      saving updates the running simulation without restarting — verified
      visually on the user's machine (`cliclick`/`screencapture`): placed an
      organism (energy 5.0, `repro_threshold` 10.0 → half-filled indicator),
      edited `repro_threshold` to `5.0` and saved, and the dot's fill color
      changed live (pixel-sampled: RGB lightness rose from the fill=0.5 shade
      to the fill=1.0 shade) with no restart — `bevy_asset`'s log confirmed
      `Reloaded config/sim_config.ron`. Reverted the test edit afterward.
- [x] `sim`/`world`/`config` still don't depend on `bevy::render` or
      `bevy_egui` (TECH_DESIGN.md invariant 2) — only `bevy_asset` (via
      `bevy_common_assets`) and `serde` were added.
- [x] The simulation stays deterministic and headless-testable: `impl Default`
      for every config struct is kept hand-written (mirroring the RON file,
      documented in `SimConfig`'s doc comment as the two-source-of-truth
      tradeoff), so all existing tests build `SimConfig` exactly as before,
      with no dependency on asset loading or Bevy's `App` machinery.
- [x] `cargo test` is clean (91 lib tests + integration tests).

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/config.rs` | Every config struct; `ConfigPlugin`; today's `impl Default` blocks become the RON file's baseline values. |
| `assets/` (new `config/sim_config.ron` or similar) | The new hot-reloadable asset file. |
| `Cargo.toml` | New dependency: `serde` (with `derive`), and likely `ron` directly if writing a custom `AssetLoader` rather than pulling in a third-party RON-asset crate. |
| `TECH_DESIGN.md` | Asset Pipeline section (§ already quoted above) — this task fulfills that stated plan; update it if the actual shape differs from what was anticipated. |

---

## 🧩 Technical Context

- **Current behavior**: `ConfigPlugin::build` calls
  `app.init_resource::<SimConfig>()`, which uses `SimConfig`'s `#[derive(Default)]`
  chain through every nested `impl Default for XConfig` block. All values are
  compile-time constants; changing tuning requires editing Rust source and
  recompiling.
- **Desired behavior**: the same values live in a RON file under `assets/`.
  `bevy_asset`'s file watcher (already part of Bevy's default asset plugin
  in dev builds) detects on-disk changes and fires `AssetEvent::Modified`;
  a system reads the updated asset and refreshes the `SimConfig` resource
  in place, so systems reading `Res<SimConfig>` see the new values on the
  next frame/tick without a restart.
- Check what Bevy 0.19 (this project's pinned version, see `Cargo.toml`)
  offers for asset hot-reload out of the box — recent Bevy versions enable
  the file-watcher behind a `bevy` feature flag (historically
  `file_watcher`) that may need turning on explicitly for dev builds.
- Because `SimConfig` today has no `Handle<T>` indirection (it's just a
  plain resource read everywhere via `Res<SimConfig>`), the cleanest shape
  is likely: a `SimConfigAsset` type implementing `bevy_asset::Asset` +
  `serde::Deserialize` mirroring `SimConfig`'s shape (or `SimConfig` itself
  gains the `Asset` derive, if its existing `Resource` derive doesn't
  conflict), loaded into a `Handle<SimConfigAsset>` resource, with a system
  syncing the asset's current value into the `SimConfig` resource whenever
  it changes (including on initial load, since asset loading is async).

---

## 🔨 Suggested Implementation

1. Add `serde` (with the `derive` feature) to `Cargo.toml`; add `ron` if a
   custom loader is the chosen path (check crates.io/bevy ecosystem for a
   RON-asset-loader crate compatible with Bevy 0.19 before writing one from
   scratch — only hand-roll the `AssetLoader` if nothing compatible exists).
2. Derive `serde::Serialize, serde::Deserialize` on every config struct in
   `src/config.rs`, alongside their existing derives.
3. Write `assets/config/sim_config.ron` with the exact values currently
   hardcoded in each `impl Default` block (transcribe carefully — this is
   the step most likely to silently drift the game's balance if done
   sloppily).
4. Implement the asset type + loader (or wire up the chosen crate), and a
   system that keeps the `SimConfig` resource in sync with the loaded/
   reloaded asset.
5. Update `ConfigPlugin::build` to register the asset type/loader and kick
   off the initial load, replacing `init_resource::<SimConfig>()`.
6. Decide what happens to the existing `impl Default for SimConfig` (and
   nested structs) used by tests — likely kept as-is (tests don't need
   asset loading), but make sure its values stay in sync with the RON file
   (a test asserting the two match, if feasible, would catch future drift).
7. Verify hot-reload end-to-end via `cargo run`: change a visible constant
   in the RON file while the game is running, confirm the change takes
   effect without restarting.
8. Run `cargo test` and `cargo clippy -- -D warnings`.

### 📝 Implementation notes (what actually happened)

- `bevy_common_assets` 0.17.0's `ron` feature was compatible with this
  project's Bevy 0.19 pin, so a hand-rolled `AssetLoader` wasn't needed —
  `RonAssetPlugin::<SimConfig>::new(&["ron"])` handles both loading and
  registration; `SimConfig` itself derives `Asset` + `TypePath` directly
  rather than going through a separate wrapper asset type.
- Enabling `bevy`'s `file_watcher` feature (required for hot-reload, not on
  by default) initially failed to resolve: `notify v8.2.0` (pulled in by
  `bevy_asset`) requires `kqueue ^1.1.1`, and this environment's locked
  `Cargo.lock` had pinned older, incompatible transitive versions. Deleting
  `Cargo.lock` and running `cargo generate-lockfile` picked a fresh,
  resolvable dependency set — no manual pinning needed.
- Bevy 0.19 renamed the event-reading API: `EventReader<T>` is now
  `MessageReader<T>` (`AssetEvent<A>` itself kept its name). The sync system
  uses `MessageReader<AssetEvent<SimConfig>>`.

---

## ⚠️ Constraints and Caveats

- **Don't let this leak rendering dependencies into `config`/`sim`/`world`**
  — `bevy_asset` is acceptable per TECH_DESIGN.md, `bevy::render`/`bevy_egui`
  are not.
- **No balance drift**: the RON file's values must match today's
  `impl Default` blocks exactly. If any value needs to change, that's a
  separate tuning task (074+), not something to fold into this migration.
- **No magic numbers introduced elsewhere** — this task moves *existing*
  centralized coefficients into a hot-reloadable asset; it must not
  reintroduce inline constants anywhere as a side effect.
- **Determinism**: hot-reloading config mid-run is a dev-time convenience,
  not a gameplay feature — don't let it interact with `SimWorld::rng` or
  any seeded/deterministic path. It only affects which coefficients future
  ticks read, same as restarting with different constants would.

---

## 🔗 Dependencies

- **Depends on**: none.
- **Blocks**: 074 (final grid-size tuning benefits directly from fast
  iteration; not a hard technical blocker, but doing 074 first would waste
  the recompiles this task is meant to eliminate).

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/073-ron-config-hot-reload.md)"$'\n\nExecute this task in the current project.'
```
