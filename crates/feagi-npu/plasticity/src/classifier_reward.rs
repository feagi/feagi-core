// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Per-scanning-instance pain and pleasure for a classifier.
//!
//! The associative mapping keeps one set of R-STDP settings. Each scanning
//! instance applies those settings only to the class channels of its own
//! decision.

/// How one class channel on a decision is corrected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelAffect {
    /// Strengthen the associative weight of this channel.
    Pleasure,
    /// Weaken the associative weight of this channel.
    Pain,
}

/// One class-channel correction for a single scanning instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelUpdate {
    pub channel: u32,
    pub affect: ChannelAffect,
    /// Pleasure for a channel the decision did not already light.
    pub bind: bool,
}

/// What the answer-feedback area did on the burst being graded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerObservation {
    /// No answer area is connected. Ambiguous decisions still take pain.
    Absent,
    /// The answer area is connected and fired no class channel. Do not score.
    Silent,
    /// The answer area fired these class channels.
    Present(Vec<u32>),
}

/// One scanning instance remembered so a later answer can grade it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedDecision {
    pub burst: u64,
    pub area_idx: u32,
    pub neuron_idx: usize,
    pub origin: (u32, u32, u32),
    pub spatial_hash: u64,
    pub channels: Vec<u32>,
}

/// Open decisions and which pattern was already corrected since the field was last quiet.
#[derive(Debug, Clone, Default)]
pub struct PresentationLedger {
    decisions: Vec<RecordedDecision>,
    corrected: std::collections::HashSet<(u32, u64)>,
}

impl PresentationLedger {
    /// Remember the class channels a scanning instance recalled on `burst`.
    pub fn record(&mut self, decision: RecordedDecision) {
        self.decisions.retain(|existing| {
            !(existing.burst == decision.burst
                && existing.area_idx == decision.area_idx
                && existing.neuron_idx == decision.neuron_idx
                && existing.origin == decision.origin)
        });
        self.decisions.push(decision);
    }

    /// Class channels recalled `latency` bursts before `burst` for this instance.
    pub fn channels_at(
        &self,
        burst: u64,
        latency: u32,
        area_idx: u32,
        neuron_idx: usize,
        origin: (u32, u32, u32),
    ) -> Option<Vec<u32>> {
        let graded_burst = burst.checked_sub(u64::from(latency))?;
        self.decisions.iter().rev().find_map(|decision| {
            if decision.burst == graded_burst
                && decision.area_idx == area_idx
                && decision.neuron_idx == neuron_idx
                && decision.origin == origin
            {
                Some(decision.channels.clone())
            } else {
                None
            }
        })
    }

    pub fn already_corrected(&self, area_idx: u32, spatial_hash: u64) -> bool {
        self.corrected.contains(&(area_idx, spatial_hash))
    }

    pub fn mark_corrected(&mut self, area_idx: u32, spatial_hash: u64) {
        self.corrected.insert((area_idx, spatial_hash));
    }

    /// Drop open decisions when the field goes quiet so the next image cannot
    /// be trained with the previous label.
    pub fn expire_area(&mut self, area_idx: u32) {
        self.decisions
            .retain(|decision| decision.area_idx != area_idx);
        self.corrected.retain(|(area, _)| *area != area_idx);
    }

    /// Drop decisions older than the latency the grader can still use.
    pub fn prune(&mut self, burst: u64, latency: u32) {
        let keep_from = burst.saturating_sub(u64::from(latency));
        self.decisions
            .retain(|decision| decision.burst >= keep_from);
    }
}

/// Pain and pleasure for one decision.
///
/// `feedback == None` means no correct-answer area is connected.
/// A connected but silent answer is not passed here. The caller skips scoring.
pub fn scanning_instance_affect(decision: &[u32], feedback: Option<&[u32]>) -> Vec<ChannelUpdate> {
    let mut decision_channels = decision.to_vec();
    decision_channels.sort_unstable();
    decision_channels.dedup();

    let Some(feedback) = feedback else {
        if decision_channels.len() <= 1 {
            return Vec::new();
        }
        return decision_channels
            .into_iter()
            .map(|channel| ChannelUpdate {
                channel,
                affect: ChannelAffect::Pain,
                bind: false,
            })
            .collect();
    };

    let mut feedback_channels = feedback.to_vec();
    feedback_channels.sort_unstable();
    feedback_channels.dedup();

    let mut updates = Vec::new();
    for channel in &decision_channels {
        let affect = if feedback_channels.contains(channel) {
            ChannelAffect::Pleasure
        } else {
            ChannelAffect::Pain
        };
        updates.push(ChannelUpdate {
            channel: *channel,
            affect,
            bind: false,
        });
    }
    for channel in feedback_channels {
        if decision_channels.contains(&channel) {
            continue;
        }
        updates.push(ChannelUpdate {
            channel,
            affect: ChannelAffect::Pleasure,
            bind: true,
        });
    }
    updates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_channel_without_feedback_is_unchanged() {
        assert!(scanning_instance_affect(&[2], None).is_empty());
    }

    #[test]
    fn ambiguous_decision_without_feedback_is_pain_on_each_channel() {
        let updates = scanning_instance_affect(&[4, 1, 4], None);
        assert_eq!(updates.len(), 2);
        assert!(updates
            .iter()
            .all(|update| update.affect == ChannelAffect::Pain));
        assert_eq!(updates[0].channel, 1);
        assert_eq!(updates[1].channel, 4);
    }

    #[test]
    fn wrong_channel_is_pain_and_confirmed_channel_is_pleasure() {
        let updates = scanning_instance_affect(&[1, 2], Some(&[2]));
        assert_eq!(
            updates,
            vec![
                ChannelUpdate {
                    channel: 1,
                    affect: ChannelAffect::Pain,
                    bind: false,
                },
                ChannelUpdate {
                    channel: 2,
                    affect: ChannelAffect::Pleasure,
                    bind: false,
                },
            ]
        );
    }

    #[test]
    fn latency_grades_the_earlier_decision_and_a_quiet_field_closes_it() {
        let mut ledger = PresentationLedger::default();
        ledger.record(RecordedDecision {
            burst: 4,
            area_idx: 1,
            neuron_idx: 2,
            origin: (0, 0, 0),
            spatial_hash: 9,
            channels: vec![1, 3],
        });
        assert_eq!(ledger.channels_at(5, 1, 1, 2, (0, 0, 0)), Some(vec![1, 3]));
        assert!(ledger.channels_at(5, 2, 1, 2, (0, 0, 0)).is_none());
        ledger.mark_corrected(1, 9);
        assert!(ledger.already_corrected(1, 9));
        ledger.expire_area(1);
        assert!(!ledger.already_corrected(1, 9));
        assert!(ledger.channels_at(5, 1, 1, 2, (0, 0, 0)).is_none());
    }

    #[test]
    fn missed_feedback_channel_is_bound_under_pleasure() {
        let updates = scanning_instance_affect(&[1], Some(&[3]));
        assert_eq!(
            updates,
            vec![
                ChannelUpdate {
                    channel: 1,
                    affect: ChannelAffect::Pain,
                    bind: false,
                },
                ChannelUpdate {
                    channel: 3,
                    affect: ChannelAffect::Pleasure,
                    bind: true,
                },
            ]
        );
    }
}
