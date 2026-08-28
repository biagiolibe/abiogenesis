// Cumulative evidence about the hidden tag matrix (GDD §7): the store the
// notebook fills from `sim`'s `AdjacencyObserved` stream, and the single
// definition of the observation-weight formula.
//
// Lives in the library rather than in the binary's `notebook.rs` (where it
// started) because it is simulation knowledge, not presentation: the
// headless two-bot survey (`examples/two_bot_survey.rs`, task 134) needs to
// ask "is this pair confirmed yet?" to model a player who only acts on what
// they've established, and `tests/` and `examples/` can only see this crate.
// Presentation — log lines, the confirmation badge, the hypothesis graph —
// stays in `notebook.rs`.

use bevy::prelude::Resource;

use crate::config::SimConfig;
use crate::sim::AdjacencyObserved;
use crate::world::{SimWorld, TagSlot};

/// Cumulative weighted evidence for every `(exerter_tag, receiver_tag)`
/// pair (GDD §7's "B with a hint of C" confirmation model) — the mechanism
/// that progressively reveals `world.matrix`, not a second opacity layer.
/// Sized and laid out exactly like `TagMatrix`
/// (`exerter.0 * size + receiver.0`) so the two stay trivially parallel for
/// task 021's UI.
///
/// Reads through to `world.matrix` for the revealed sign once a pair is
/// confirmed, rather than storing its own copy of the value — simpler, and
/// there's only ever one `SimWorld` to read from.
///
/// There is no "confirmed zero effect" state (GDD §5.9's `0` cell): task
/// 018 only emits `AdjacencyObserved` for pairs with a non-zero matrix
/// entry, so evidence never accumulates for a genuinely-zero pair — the
/// hypothesis grid (021) only distinguishes `?` (unconfirmed) from `±!`
/// (confirmed non-zero, sign shown).
#[derive(Resource)]
pub struct MatrixKnowledge {
    size: usize,
    threshold: f32,
    evidence: Vec<f32>,
}

impl MatrixKnowledge {
    pub fn new(active_tags: usize, threshold: f32) -> Self {
        Self {
            size: active_tags,
            threshold,
            evidence: vec![0.0; active_tags * active_tags],
        }
    }

    /// Adds `weight` to a pair's evidence, returning `true` if this call is
    /// the one that pushed it from below `threshold` to at/above it (GDD
    /// §7's "aha" moment) — `false` on every other call, including repeat
    /// evidence for an already-confirmed pair.
    pub fn record(&mut self, exerter: TagSlot, receiver: TagSlot, weight: f32) -> bool {
        let idx = exerter.0 as usize * self.size + receiver.0 as usize;
        let was_confirmed = self.evidence[idx] >= self.threshold;
        self.evidence[idx] += weight;
        !was_confirmed && self.evidence[idx] >= self.threshold
    }

    pub fn evidence(&self, exerter: TagSlot, receiver: TagSlot) -> f32 {
        self.evidence[exerter.0 as usize * self.size + receiver.0 as usize]
    }

    pub fn is_confirmed(&self, exerter: TagSlot, receiver: TagSlot) -> bool {
        self.evidence(exerter, receiver) >= self.threshold
    }

    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Whether `tag` participates in at least one confirmed pair, as either
    /// exerter or receiver, against any other active tag (task 147: the
    /// per-tag reading `Splice`'s trait filter needs). Confirmation is
    /// stored per-*pair*, not per-tag — this is a judgment call, not a
    /// spec: "confirmed" here means "you've decoded *something* about this
    /// trait," the simplest reading consistent with the design doc, not
    /// "confirmed against every other active tag."
    pub fn is_tag_confirmed(&self, tag: TagSlot) -> bool {
        (0..self.size as u8).any(|other| {
            let other = TagSlot(other);
            self.is_confirmed(tag, other) || self.is_confirmed(other, tag)
        })
    }

    /// The real matrix value for a confirmed pair, `None` if not yet
    /// confirmed. Reads through `world.matrix`, not a stored snapshot.
    pub fn revealed_value(
        &self,
        exerter: TagSlot,
        receiver: TagSlot,
        world: &SimWorld,
    ) -> Option<i8> {
        self.is_confirmed(exerter, receiver)
            .then(|| world.matrix.get(exerter, receiver))
    }
}

/// Folds one tick's `AdjacencyObserved` stream into `knowledge`, returning
/// the pairs this call pushed over the confirmation threshold (GDD §7's
/// "aha" moment) in the order they crossed.
///
/// The weight formula `numerator / (1 + n_confounders)` (GDD §7) lives here
/// and nowhere else. `notebook.rs`'s Bevy system calls this and then does the
/// presentation half — log entry, unseen-confirmation badge — with the pairs
/// it returns; the bot survey calls it and ignores the return value. A second
/// copy of the formula in either caller is exactly the notebook/narration
/// drift `redesign/processed/abiogenesis-tick-pipeline.md` argues against.
pub fn accumulate_adjacency_evidence(
    observations: impl IntoIterator<Item = AdjacencyObserved>,
    config: &SimConfig,
    knowledge: &mut MatrixKnowledge,
) -> Vec<(TagSlot, TagSlot)> {
    let mut newly_confirmed = Vec::new();
    for event in observations {
        let weight =
            config.notebook.observation_weight_numerator / (1.0 + event.n_confounders as f32);
        if knowledge.record(event.exerter_tag, event.receiver_tag, weight) {
            newly_confirmed.push((event.exerter_tag, event.receiver_tag));
        }
    }
    newly_confirmed
}

#[cfg(test)]
mod tests {
    use super::*;

    const THRESHOLD: f32 = 3.0;

    fn observation(exerter: u8, receiver: u8, n_confounders: u32) -> AdjacencyObserved {
        AdjacencyObserved {
            receiver_species: crate::world::SpeciesId(0),
            exerter_tag: TagSlot(exerter),
            receiver_tag: TagSlot(receiver),
            contribution: 0.0,
            n_confounders,
            cell: 0,
        }
    }

    #[test]
    fn three_isolated_observations_reach_the_threshold_exactly() {
        // GDD §7: n_confounders = 0 -> weight 1.0 each; 3 * 1.0 == threshold.
        let mut knowledge = MatrixKnowledge::new(2, THRESHOLD);
        for _ in 0..3 {
            knowledge.record(TagSlot(0), TagSlot(1), 1.0);
        }
        assert!((knowledge.evidence(TagSlot(0), TagSlot(1)) - 3.0).abs() < 1e-6);
        assert!(knowledge.is_confirmed(TagSlot(0), TagSlot(1)));
    }

    #[test]
    fn record_reports_the_confirmation_transition_exactly_once() {
        let mut knowledge = MatrixKnowledge::new(2, THRESHOLD);
        assert!(!knowledge.record(TagSlot(0), TagSlot(1), 1.0));
        assert!(!knowledge.record(TagSlot(0), TagSlot(1), 1.0));
        assert!(knowledge.record(TagSlot(0), TagSlot(1), 1.0));
        // Already confirmed: further evidence keeps accumulating but no
        // longer reports a fresh transition.
        assert!(!knowledge.record(TagSlot(0), TagSlot(1), 1.0));
    }

    #[test]
    fn confounded_observations_need_four_times_as_many() {
        // n_confounders = 3 -> weight 0.25 each; 12 * 0.25 == threshold.
        let mut knowledge = MatrixKnowledge::new(2, THRESHOLD);
        for _ in 0..11 {
            knowledge.record(TagSlot(0), TagSlot(1), 0.25);
        }
        assert!(
            !knowledge.is_confirmed(TagSlot(0), TagSlot(1)),
            "11 * 0.25 = 2.75, just under threshold"
        );

        knowledge.record(TagSlot(0), TagSlot(1), 0.25);
        assert!(
            knowledge.is_confirmed(TagSlot(0), TagSlot(1)),
            "the 12th observation crosses the threshold"
        );
    }

    #[test]
    fn a_clean_observation_is_worth_more_than_a_confounded_one() {
        let config = SimConfig::default();
        let mut knowledge = MatrixKnowledge::new(2, THRESHOLD);
        accumulate_adjacency_evidence([observation(0, 1, 0)], &config, &mut knowledge);
        let clean = knowledge.evidence(TagSlot(0), TagSlot(1));
        accumulate_adjacency_evidence([observation(1, 0, 3)], &config, &mut knowledge);
        let confounded = knowledge.evidence(TagSlot(1), TagSlot(0));
        assert!(
            clean > confounded,
            "clean {clean} should outweigh confounded {confounded}"
        );
    }

    /// Task 147: `Splice`'s trait filter reads `is_tag_confirmed`, not
    /// `is_confirmed` directly — a tag counts as confirmed if it appears in
    /// at least one confirmed pair, as either exerter or receiver.
    #[test]
    fn is_tag_confirmed_true_for_either_side_of_a_confirmed_pair() {
        let mut knowledge = MatrixKnowledge::new(3, THRESHOLD);
        knowledge.record(TagSlot(0), TagSlot(1), THRESHOLD);

        assert!(knowledge.is_tag_confirmed(TagSlot(0)), "exerter side");
        assert!(knowledge.is_tag_confirmed(TagSlot(1)), "receiver side");
        assert!(
            !knowledge.is_tag_confirmed(TagSlot(2)),
            "tag 2 has no evidence at all"
        );
    }

    #[test]
    fn accumulate_returns_every_pair_it_confirms() {
        let config = SimConfig::default();
        // `observation_weight_numerator` is 1.0 by default, so one clean
        // observation is exactly enough at a threshold of 1.0.
        let mut knowledge = MatrixKnowledge::new(2, 1.0);
        let confirmed = accumulate_adjacency_evidence(
            [
                observation(0, 1, 0),
                observation(1, 0, 0),
                observation(0, 1, 0),
            ],
            &config,
            &mut knowledge,
        );
        assert_eq!(
            confirmed,
            vec![(TagSlot(0), TagSlot(1)), (TagSlot(1), TagSlot(0))]
        );
    }
}
