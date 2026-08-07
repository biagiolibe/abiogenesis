// Run-level state (task 035, GDD §8-§10): where the player is within a
// sequence of worlds. Kept separate from `world.rs` (which owns a *single*
// world's simulation state) so it survives across the `SimWorld` rebuilds
// that world transitions (task 045) perform.

use bevy::prelude::*;

/// Tracks progress through a run (a sequence of worlds, GDD §8: "success ->
/// next world... failure -> the run ends"). `run_seed` is chosen once, at
/// the main menu (task 044) — the one legitimate point outside the
/// simulation where a run's variety originates; every subsequent
/// `world_seed` derives from the run's own seeded RNG, never a fresh source,
/// to preserve determinism (TECH_DESIGN.md §5).
#[derive(Resource, Debug, Clone, Default)]
pub struct RunProgress {
    pub run_seed: u64,
    pub world_index: u32,
    pub world_seed: u64,
    pub worlds_cleared: u32,
    pub unlocks: Unlocks,
}

impl RunProgress {
    /// Starts a fresh run at `run_seed` (task 044, chosen or generated at the
    /// main menu). Seed scheme: `world_index == 0`'s `world_seed` is
    /// `run_seed` itself, with no derivation step — typing a shared seed at
    /// the menu reproduces exactly the world that seed names, matching the
    /// existing `SimWorld::new(seed, ..)` behavior a fixed seed already had
    /// pre-menu. Every later world's seed (task 045) instead comes from the
    /// *previous* world's own RNG (`SimWorld::next_seed`, the same source the
    /// `r`-key reseed already draws from) — `run_seed` is read once, here,
    /// and never again.
    ///
    /// `unlocks` is a snapshot of `meta` taken once, here — meta-progression
    /// earned *during* this run (there isn't any: `MetaProgress` only grows
    /// on `Defeat`, task 046) never retroactively changes a run already in
    /// progress.
    pub fn start(run_seed: u64, meta: &MetaProgress) -> Self {
        Self {
            run_seed,
            world_index: 0,
            world_seed: run_seed,
            worlds_cleared: 0,
            unlocks: Unlocks {
                bonus_available_species: meta.bonus_available_species,
            },
        }
    }
}

/// This run's meta-progression, captured once at `RunProgress::start` (task
/// 046, GDD §10: "unlock capabilities, not answers"). A snapshot, not a live
/// view — see `RunProgress::start`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Unlocks {
    /// Extra species added to the starting palette's available pool
    /// (`worldgen::add_bonus_species`) beyond `WorldgenConfig::
    /// extra_available_species_count` — more to *choose from* with `Seed`,
    /// never any information about the hidden matrix.
    pub bonus_available_species: u32,
}

/// Meta-progression accumulated across runs within the same process session
/// (GDD §10: "deliberately light... you unlock capabilities, not answers";
/// persistence explicitly deferred post-MVP — this resource is never
/// written to disk and resets when the process exits). Unlike `RunProgress`,
/// nothing resets this for the lifetime of the process: `RunPlugin` inserts
/// it once, and `absorb` only ever adds to it.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct MetaProgress {
    pub bonus_available_species: u32,
    /// Whether the one-time intro screen (task 052) has already played this
    /// process session. Same non-persistence caveat as the rest of
    /// `MetaProgress`: `false` again on every fresh launch, not just once
    /// ever — there's no save file yet to make "once per install" literal.
    pub seen_intro: bool,
    /// Whether the guided first-isolation hint (task 055) has already shown
    /// this process session — same one-shot, non-persisted pattern as
    /// `seen_intro`. Gated here rather than on world/era state so it never
    /// reappears on a later world within the same run, or a later run in
    /// the same session.
    pub seen_isolation_hint: bool,
}

impl MetaProgress {
    /// Unlock rule (GDD §10 "deliberately light"): one extra available
    /// species for every 2 worlds cleared before the run ended in `Defeat`
    /// (task 045). Linear and uncapped by design — a clear, testable rule
    /// beats an elaborate progression system for an MVP.
    pub fn absorb(&mut self, worlds_cleared: u32) {
        self.bonus_available_species += worlds_cleared / 2;
    }
}

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunProgress>()
            .init_resource::<MetaProgress>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absorb_grants_one_bonus_species_per_two_worlds_cleared() {
        let mut meta = MetaProgress::default();
        meta.absorb(5);
        assert_eq!(meta.bonus_available_species, 2);
    }

    #[test]
    fn absorb_accumulates_across_multiple_runs() {
        let mut meta = MetaProgress::default();
        meta.absorb(3);
        meta.absorb(4);
        assert_eq!(meta.bonus_available_species, 1 + 2);
    }

    #[test]
    fn start_snapshots_the_current_meta_progress_into_unlocks() {
        let mut meta = MetaProgress::default();
        meta.absorb(10);

        let run = RunProgress::start(42, &meta);
        assert_eq!(run.unlocks.bonus_available_species, 5);

        // Earning more afterwards must not retroactively change the
        // already-started run's snapshot.
        meta.absorb(10);
        assert_eq!(run.unlocks.bonus_available_species, 5);
    }
}
