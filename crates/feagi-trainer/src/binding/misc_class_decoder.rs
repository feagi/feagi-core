//! Misc OPU class decoder: argmax over Y at each parallel X slot.
//!
//! Reads FEAGI MiscData voxels. P at (x, y, 0) is the class-y activation for beat slot x.

use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::genomic::MotorCorticalUnit;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

use crate::binding::decoder::{DecoderPlugin, SlotDecoder};
use crate::binding::profile::DecoderBindingProfile;
use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::prediction_record::TypedPrediction;
use crate::error::TrainerError;

/// Motor unit 0 is the Misc OPU the operator wires from the detection circuit.
#[derive(Debug, Clone, Copy, Default)]
pub struct MiscClassDecoder;

impl MiscClassDecoder {
    /// Creates a new selector.
    pub fn new() -> Self {
        Self
    }

    fn opu_id() -> feagi_structures::genomic::cortical_area::CorticalID {
        MotorCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
            FrameChangeHandling::Absolute,
            CorticalUnitIndex::from(0u16),
        )[0]
    }

    fn scores_by_slot(
        motor: &CorticalMappedXYZPNeuronVoxels,
        class_count: u32,
        slot_count: u32,
    ) -> Result<Vec<Vec<f64>>, TrainerError> {
        if class_count == 0 || slot_count == 0 {
            return Err(TrainerError::Config(
                "misc class decoder requires class_count > 0 and slot_count > 0".to_string(),
            ));
        }
        let mut scores = vec![vec![0.0_f64; class_count as usize]; slot_count as usize];
        let Some(neurons) = motor.get_neurons_of(&Self::opu_id()) else {
            return Ok(scores);
        };
        for neuron in neurons.iter() {
            let x = neuron.neuron_voxel_coordinate.x;
            let y = neuron.neuron_voxel_coordinate.y;
            if x >= slot_count || y >= class_count {
                continue;
            }
            scores[x as usize][y as usize] += f64::from(neuron.potential);
        }
        Ok(scores)
    }

    fn prediction(scores: Vec<f64>) -> Result<TypedPrediction, TrainerError> {
        let class_id = scores
            .iter()
            .enumerate()
            .reduce(|best, current| if current.1 > best.1 { current } else { best })
            .map(|(index, _)| index as u32)
            .ok_or_else(|| {
                TrainerError::Evaluation(
                    "misc class decoder received an empty score vector".to_string(),
                )
            })?;
        Ok(TypedPrediction::Class { class_id, scores })
    }
}

impl DecoderPlugin for MiscClassDecoder {
    type Frame = CorticalMappedXYZPNeuronVoxels;

    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId("decoder.misc_data_class".to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn decode(
        &mut self,
        motor: Self::Frame,
        profile: &DecoderBindingProfile,
    ) -> Result<TypedPrediction, TrainerError> {
        let mut slots = self.decode_slots(motor, profile, 1)?;
        slots.pop().ok_or_else(|| {
            TrainerError::Evaluation("misc class decoder produced no slot".to_string())
        })
    }
}

impl SlotDecoder for MiscClassDecoder {
    type Frame = CorticalMappedXYZPNeuronVoxels;

    fn plugin_ref(&self) -> PluginRef {
        DecoderPlugin::plugin_ref(self)
    }

    fn decode_slots(
        &mut self,
        motor: Self::Frame,
        profile: &DecoderBindingProfile,
        slot_count: u32,
    ) -> Result<Vec<TypedPrediction>, TrainerError> {
        let rows = Self::scores_by_slot(&motor, profile.class_count, slot_count)?;
        rows.into_iter().map(Self::prediction).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;

    fn profile(classes: u32) -> DecoderBindingProfile {
        DecoderBindingProfile {
            cortical_area_id: "misc_opu".to_string(),
            class_count: classes,
            bins: 1,
            mask_width: None,
            mask_height: None,
            mask_depth: None,
            cortical_name: None,
        }
    }

    #[test]
    fn argmax_is_y_with_highest_p() {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push_raw(0, 1, 0, 0.2);
        arrays.push_raw(0, 3, 0, 0.9);
        let mut motor = CorticalMappedXYZPNeuronVoxels::new();
        motor.insert(MiscClassDecoder::opu_id(), arrays);
        let mut decoder = MiscClassDecoder::new();
        let pred = decoder.decode(motor, &profile(5)).expect("decode");
        match pred {
            TypedPrediction::Class { class_id, scores } => {
                assert_eq!(class_id, 3);
                assert!((scores[3] - 0.9).abs() < 1e-6);
            }
            other => panic!("{other:?}"),
        }
    }
}
