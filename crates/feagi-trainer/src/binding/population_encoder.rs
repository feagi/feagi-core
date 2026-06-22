//! Population (single-spike) encoder selector over `feagi-sensorimotor` coders.
//!
//! Selects FEAGI's percentage coder (`CountInput` template) and applies it: each scalar
//! feature in `[0,1]` is written as a `Percentage` and encoded into a single spike at the
//! matching bin of an N-bin cortical column (population/positional coding). This implements
//! no spike-coding math — it configures and drives the native coder (ADR-006, Appendix B).
//!
//! Convention for the IRIS slice: the features map to one `CountInput` area at unit index 0
//! with one channel per feature. The pinned genome's IPU area must match the `CorticalID`
//! derived from these parameters.

use std::time::Instant;

use feagi_sensorimotor::data_types::Percentage;
use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
use feagi_sensorimotor::ConnectorCache;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

use crate::binding::encoder::{EncoderPlugin, ObservationEncoder};
use crate::binding::encoding_scheme::BinSpacing;
use crate::binding::profile::EncoderBindingProfile;
use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::ir_sample::{IRSample, Payload};
use crate::error::TrainerError;

/// Sensory cortical unit index this encoder writes to for the IRIS slice.
const IRIS_SENSORY_UNIT: u8 = 0;

/// Stateless selector that encodes tabular features via FEAGI population coding.
#[derive(Debug, Clone, Copy, Default)]
pub struct PopulationEncoder;

impl PopulationEncoder {
    /// Creates a new selector.
    pub fn new() -> Self {
        Self
    }

    /// Shared core: encodes a slice of normalized `[0,1]` features into a sensory frame via the
    /// native population coder. Used by both the [`EncoderPlugin`] (dataset sample) and
    /// [`ObservationEncoder`](crate::binding::encoder::ObservationEncoder) (control observation)
    /// paths so the coder configuration lives in exactly one place.
    fn encode_features(
        &self,
        features: &[f64],
        profile: &EncoderBindingProfile,
    ) -> Result<CorticalMappedXYZPNeuronVoxels, TrainerError> {
        let resolved = profile.scheme.resolve()?;

        if features.len() != profile.channels as usize {
            return Err(TrainerError::Config(format!(
                "feature count {} does not match profile channels {}",
                features.len(),
                profile.channels
            )));
        }

        let cache = ConnectorCache::new();
        let positioning = to_positioning(resolved.spacing);
        let unit = CorticalUnitIndex::from(IRIS_SENSORY_UNIT);

        let mut sensor_cache = cache.get_sensor_cache();
        sensor_cache
            .count_input_register(
                unit,
                CorticalChannelCount::new(profile.channels).map_err(map_err)?,
                FrameChangeHandling::Absolute,
                NeuronDepth::new(resolved.bins).map_err(map_err)?,
                positioning,
            )
            .map_err(map_err)?;

        for (channel, &feature) in features.iter().enumerate() {
            // No silent clamping: a value outside [0,1] is a caller/normalization error.
            let percentage = Percentage::new_from_0_1(feature as f32).map_err(|e| {
                TrainerError::Config(format!(
                    "feature {channel} = {feature} is not a normalized percentage in [0,1]: {e}"
                ))
            })?;
            sensor_cache
                .count_input_write(
                    unit,
                    CorticalChannelIndex::from(channel as u32),
                    WrappedIOData::Percentage(percentage),
                )
                .map_err(map_err)?;
        }

        sensor_cache
            .encode_all_sensors_to_neurons(Instant::now())
            .map_err(map_err)?;
        Ok(sensor_cache.get_neurons().clone())
    }
}

fn to_positioning(spacing: BinSpacing) -> PercentageNeuronPositioning {
    match spacing {
        BinSpacing::Linear => PercentageNeuronPositioning::Linear,
        BinSpacing::Fractional => PercentageNeuronPositioning::Fractional,
    }
}

fn map_err<E: std::fmt::Display>(e: E) -> TrainerError {
    TrainerError::Config(e.to_string())
}

impl EncoderPlugin for PopulationEncoder {
    type Frame = CorticalMappedXYZPNeuronVoxels;

    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId("encoder.population_single_spike".to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn encode(
        &mut self,
        sample: &IRSample,
        profile: &EncoderBindingProfile,
    ) -> Result<Self::Frame, TrainerError> {
        let features = match &sample.payload {
            Payload::Tabular(values) => values,
            other => {
                return Err(TrainerError::Config(format!(
                    "population encoder requires a tabular payload, got {other:?}"
                )))
            }
        };
        self.encode_features(features, profile)
    }
}

impl ObservationEncoder for PopulationEncoder {
    type Frame = CorticalMappedXYZPNeuronVoxels;

    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId("encoder.population_single_spike".to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn encode_observation(
        &mut self,
        observation: &crate::binding::environment::Observation,
        profile: &EncoderBindingProfile,
    ) -> Result<Self::Frame, TrainerError> {
        self.encode_features(observation, profile)
    }
}
