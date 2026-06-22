//! Continuous motor decoder selector over `feagi-sensorimotor` coders.
//!
//! Reads per-channel activations from an OPU area via FEAGI's native count/percentage output
//! coder and maps them to a **normalized** continuous action (each component in `[-1, 1]`) for
//! embodied/control tasks (plan Phase 1d). Like [`ClassDecoder`](crate::binding::ClassDecoder)
//! it implements no spike-coding math beyond configuring/driving the native coder and mapping
//! its output into the typed contract.
//!
//! Separation of concerns: the decoder is **environment-agnostic** — it emits a normalized
//! action. Scaling/clamping that action to the environment's actuator `ctrlrange` is the
//! closed-loop executor's job (using `Environment::action_bounds`), so the same decoder serves
//! any actuator range.

use std::time::Instant;

use feagi_sensorimotor::ConnectorCache;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalUnitIndex, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

use crate::binding::decoder::DecoderPlugin;
use crate::binding::profile::DecoderBindingProfile;
use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::prediction_record::TypedPrediction;
use crate::error::TrainerError;

/// Motor cortical unit index this decoder reads from (matches the pinned genome's OPU unit 0).
const MOTOR_UNIT: u8 = 0;

/// How OPU channel activations encode a signed continuous action component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuousDecodeScheme {
    /// One channel per action dimension; activation `a` in `[0, 1]` maps to `2a - 1` in
    /// `[-1, 1]`.
    BipolarSingleChannel,
    /// Two channels per dimension `(negative, positive)`; the component is `positive - negative`
    /// clamped to `[-1, 1]`. Useful when push-left / push-right are distinct populations.
    BipolarOpposingPair,
}

/// Decodes a normalized continuous action vector from an OPU area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContinuousMotorDecoder {
    scheme: ContinuousDecodeScheme,
    output_dims: u32,
}

fn map_err<E: std::fmt::Display>(e: E) -> TrainerError {
    TrainerError::Config(e.to_string())
}

impl ContinuousMotorDecoder {
    /// Stable plugin id for this decoder selector.
    pub const PLUGIN_ID: &'static str = "decoder.continuous_motor";

    /// Creates a decoder for `output_dims` action dimensions using `scheme`.
    pub fn new(scheme: ContinuousDecodeScheme, output_dims: u32) -> Result<Self, TrainerError> {
        if output_dims == 0 {
            return Err(TrainerError::Config(
                "continuous motor decoder requires output_dims > 0".to_string(),
            ));
        }
        Ok(Self {
            scheme,
            output_dims,
        })
    }

    /// Number of OPU channels the area must expose for this scheme/dimensionality.
    pub fn channel_count(&self) -> u32 {
        match self.scheme {
            ContinuousDecodeScheme::BipolarSingleChannel => self.output_dims,
            ContinuousDecodeScheme::BipolarOpposingPair => self.output_dims * 2,
        }
    }

    /// Pure mapping from per-channel activations (`[0, 1]`) to a normalized action (`[-1, 1]`).
    ///
    /// Channel layout is dimension-major: dimension `d` occupies channel `d`
    /// (`BipolarSingleChannel`) or channels `2d` (negative) and `2d + 1` (positive)
    /// (`BipolarOpposingPair`).
    fn normalize(&self, activations: &[f64]) -> Result<Vec<f64>, TrainerError> {
        let expected = self.channel_count() as usize;
        if activations.len() != expected {
            return Err(TrainerError::Evaluation(format!(
                "continuous decoder expected {expected} channel activations, got {}",
                activations.len()
            )));
        }
        let action = match self.scheme {
            ContinuousDecodeScheme::BipolarSingleChannel => activations
                .iter()
                .map(|a| (2.0 * a - 1.0).clamp(-1.0, 1.0))
                .collect(),
            ContinuousDecodeScheme::BipolarOpposingPair => (0..self.output_dims as usize)
                .map(|d| {
                    let negative = activations[2 * d];
                    let positive = activations[2 * d + 1];
                    (positive - negative).clamp(-1.0, 1.0)
                })
                .collect(),
        };
        Ok(action)
    }
}

impl DecoderPlugin for ContinuousMotorDecoder {
    type Frame = CorticalMappedXYZPNeuronVoxels;

    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn decode(
        &mut self,
        motor: Self::Frame,
        profile: &DecoderBindingProfile,
    ) -> Result<TypedPrediction, TrainerError> {
        let channels = self.channel_count();
        // The pinned binding profile must agree with the decoder's channel layout, so a genome
        // mismatch fails explicitly rather than silently decoding the wrong shape.
        if profile.class_count != channels {
            return Err(TrainerError::Config(format!(
                "decoder profile class_count {} does not match continuous channel_count {channels}",
                profile.class_count
            )));
        }

        let cache = ConnectorCache::new();
        let unit = CorticalUnitIndex::from(MOTOR_UNIT);
        let snapshot = {
            let mut motor_cache = cache.get_motor_cache();
            motor_cache
                .count_output_register(
                    unit,
                    CorticalChannelCount::new(channels).map_err(map_err)?,
                    FrameChangeHandling::Absolute,
                    NeuronDepth::new(profile.bins).map_err(map_err)?,
                    PercentageNeuronPositioning::Linear,
                )
                .map_err(map_err)?;
            motor_cache
                .ingest_neuron_data_and_run_callbacks(motor, Instant::now())
                .map_err(map_err)?;
            motor_cache.read_decoded_motor_snapshot()
        };

        let mut activations = vec![0.0_f64; channels as usize];
        for decoded in snapshot {
            if decoded.group != MOTOR_UNIT as u32 {
                continue;
            }
            if let Some(slot) = activations.get_mut(decoded.channel as usize) {
                *slot = decoded.value;
            }
        }

        Ok(TypedPrediction::Vector(self.normalize(&activations)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_dims() {
        assert!(
            ContinuousMotorDecoder::new(ContinuousDecodeScheme::BipolarSingleChannel, 0).is_err()
        );
    }

    #[test]
    fn channel_count_matches_scheme() {
        let single =
            ContinuousMotorDecoder::new(ContinuousDecodeScheme::BipolarSingleChannel, 3).unwrap();
        assert_eq!(single.channel_count(), 3);
        let pair =
            ContinuousMotorDecoder::new(ContinuousDecodeScheme::BipolarOpposingPair, 3).unwrap();
        assert_eq!(pair.channel_count(), 6);
    }

    #[test]
    fn single_channel_maps_to_bipolar_range() {
        let d =
            ContinuousMotorDecoder::new(ContinuousDecodeScheme::BipolarSingleChannel, 1).unwrap();
        assert_eq!(d.normalize(&[0.0]).unwrap(), vec![-1.0]);
        assert_eq!(d.normalize(&[1.0]).unwrap(), vec![1.0]);
        assert_eq!(d.normalize(&[0.5]).unwrap(), vec![0.0]);
    }

    #[test]
    fn single_channel_handles_multiple_dims() {
        let d =
            ContinuousMotorDecoder::new(ContinuousDecodeScheme::BipolarSingleChannel, 2).unwrap();
        assert_eq!(d.normalize(&[0.0, 1.0]).unwrap(), vec![-1.0, 1.0]);
    }

    #[test]
    fn opposing_pair_subtracts_negative_from_positive() {
        let d =
            ContinuousMotorDecoder::new(ContinuousDecodeScheme::BipolarOpposingPair, 1).unwrap();
        // (neg=0.2, pos=0.8) -> 0.6
        assert!((d.normalize(&[0.2, 0.8]).unwrap()[0] - 0.6).abs() < 1e-12);
        // (neg=1.0, pos=0.0) -> -1.0
        assert!((d.normalize(&[1.0, 0.0]).unwrap()[0] + 1.0).abs() < 1e-12);
    }

    #[test]
    fn wrong_activation_length_is_error() {
        let d =
            ContinuousMotorDecoder::new(ContinuousDecodeScheme::BipolarSingleChannel, 2).unwrap();
        assert!(d.normalize(&[0.5]).is_err());
    }
}
