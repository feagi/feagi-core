//! Single-label class decoder selector over `feagi-sensorimotor` coders.
//!
//! Selects FEAGI's percentage coder (`CountOutput` template) with one channel per class,
//! reads each channel's decoded activation, and returns the argmax as the predicted class.
//! Implements no decoding math beyond configuring/driving the native coder and mapping its
//! output into the typed contract.
//!
//! Convention for the IRIS slice: the classes map to one `CountOutput` area at unit index 0
//! with one channel per class. The pinned genome's OPU area must match the `CorticalID`
//! derived from these parameters.

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

/// Motor cortical unit index this decoder reads from for the IRIS slice.
const IRIS_MOTOR_UNIT: u8 = 0;

/// Stateless selector that decodes a class index from per-class channel activations.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClassDecoder;

impl ClassDecoder {
    /// Creates a new selector.
    pub fn new() -> Self {
        Self
    }
}

fn map_err<E: std::fmt::Display>(e: E) -> TrainerError {
    TrainerError::Config(e.to_string())
}

impl DecoderPlugin for ClassDecoder {
    type Frame = CorticalMappedXYZPNeuronVoxels;

    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId("decoder.class_argmax".to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn decode(
        &mut self,
        motor: Self::Frame,
        profile: &DecoderBindingProfile,
    ) -> Result<TypedPrediction, TrainerError> {
        if profile.class_count == 0 {
            return Err(TrainerError::Config(
                "class decoder requires class_count > 0".to_string(),
            ));
        }

        let cache = ConnectorCache::new();
        let unit = CorticalUnitIndex::from(IRIS_MOTOR_UNIT);
        let snapshot = {
            let mut motor_cache = cache.get_motor_cache();
            motor_cache
                .count_output_register(
                    unit,
                    CorticalChannelCount::new(profile.class_count).map_err(map_err)?,
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

        let mut scores = vec![0.0_f64; profile.class_count as usize];
        for decoded in snapshot {
            if decoded.group != IRIS_MOTOR_UNIT as u32 {
                continue;
            }
            if let Some(slot) = scores.get_mut(decoded.channel as usize) {
                *slot = decoded.value;
            }
        }

        let class_id = argmax(&scores).ok_or_else(|| {
            TrainerError::Evaluation("no class channels decoded from motor frame".to_string())
        })?;

        Ok(TypedPrediction::Class {
            class_id: class_id as u32,
            scores,
        })
    }
}

/// Returns the index of the maximum score, ties broken by lowest index. `None` if empty.
fn argmax(scores: &[f64]) -> Option<usize> {
    scores
        .iter()
        .enumerate()
        .reduce(|best, current| if current.1 > best.1 { current } else { best })
        .map(|(index, _)| index)
}
