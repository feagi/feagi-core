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

/// Pain and pleasure for one decision.
///
/// `feedback == None` means no correct-answer area is connected.
/// An empty feedback slice means the correct answer lit no class channel.
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
