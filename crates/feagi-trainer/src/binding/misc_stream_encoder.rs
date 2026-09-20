//! Graded-P Misc encoder for streamed amplitude (A) and class teacher (B).
//!
//! Uses FEAGI's MiscData coder: each non-zero cell is a voxel at (x, y, z) with P = value.
//! Amplitude area is `N×1×1`. Teacher area is `N×C×1` with one class neuron held at P = 1.0.

use feagi_sensorimotor::data_types::descriptors::MiscDataDimensions;
use feagi_sensorimotor::data_types::MiscData;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::genomic::SensoryCorticalUnit;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};

use crate::binding::encoder::TickEncoder;
use crate::binding::encoding_scheme::EncodingScheme;
use crate::binding::profile::EncoderBindingProfile;
use crate::contracts::common::{PluginId, PluginRef};
use crate::error::TrainerError;

/// Unit index 0 is amplitude; unit 1 is the class teacher (profile may override).
#[derive(Debug, Clone, Copy, Default)]
pub struct MiscStreamEncoder;

impl MiscStreamEncoder {
    /// Creates a new selector.
    pub fn new() -> Self {
        Self
    }

    fn misc_id(unit: u16) -> feagi_structures::genomic::cortical_area::CorticalID {
        SensoryCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
            FrameChangeHandling::Absolute,
            CorticalUnitIndex::from(unit),
        )[0]
    }

    fn write_area(
        width: u32,
        height: u32,
        cells: impl Iterator<Item = (u32, u32, f32)>,
    ) -> Result<NeuronVoxelXYZPArrays, TrainerError> {
        let dims = MiscDataDimensions::new(width, height, 1).map_err(map_err)?;
        let mut data = MiscData::new(&dims).map_err(map_err)?;
        {
            let grid = data.get_internal_data_mut();
            for (x, y, value) in cells {
                if x >= width || y >= height {
                    return Err(TrainerError::Config(format!(
                        "misc stream voxel ({x},{y}) is outside {width}x{height}x1"
                    )));
                }
                grid[(x as usize, y as usize, 0)] = value;
            }
        }
        let mut arrays = NeuronVoxelXYZPArrays::new();
        data.overwrite_neuron_data(&mut arrays, CorticalChannelIndex::from(0u32))
            .map_err(map_err)?;
        Ok(arrays)
    }
}

fn map_err<E: std::fmt::Display>(e: E) -> TrainerError {
    TrainerError::Config(e.to_string())
}

impl TickEncoder for MiscStreamEncoder {
    type Frame = CorticalMappedXYZPNeuronVoxels;

    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId("encoder.misc_data_stream".to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn encode_tick(
        &mut self,
        amplitudes: &[f64],
        class_ids: &[Option<u32>],
        profile: &EncoderBindingProfile,
        class_count: u32,
    ) -> Result<Self::Frame, TrainerError> {
        if !matches!(profile.scheme, EncodingScheme::Value) {
            return Err(TrainerError::Config(
                "misc stream encoder requires encoding scheme 'value'".to_string(),
            ));
        }
        let stream = profile.stream.as_ref().ok_or_else(|| {
            TrainerError::Config("misc stream encoder requires encoder_profile.stream".to_string())
        })?;
        stream.validate()?;
        if class_count == 0 {
            return Err(TrainerError::Config(
                "misc stream encoder requires class_count > 0".to_string(),
            ));
        }
        if amplitudes.len() != class_ids.len() {
            return Err(TrainerError::Config(format!(
                "amplitude count {} does not match class_id count {}",
                amplitudes.len(),
                class_ids.len()
            )));
        }
        if amplitudes.is_empty() {
            return Err(TrainerError::Config(
                "misc stream encoder requires at least one parallel slot".to_string(),
            ));
        }
        if amplitudes.len() as u32 > stream.parallel_width {
            return Err(TrainerError::Config(format!(
                "active slots {} exceed parallel_width {}",
                amplitudes.len(),
                stream.parallel_width
            )));
        }
        for (index, value) in amplitudes.iter().enumerate() {
            if !value.is_finite() {
                return Err(TrainerError::Config(format!(
                    "amplitude slot {index} = {value} is not finite"
                )));
            }
        }

        let width = stream.parallel_width;
        let mut amp_arrays = NeuronVoxelXYZPArrays::new();
        for (x, value) in amplitudes.iter().enumerate() {
            amp_arrays.push_raw(x as u32, 0, 0, *value as f32);
        }

        let teacher_cells = class_ids
            .iter()
            .enumerate()
            .filter_map(|(x, class)| class.map(|class_id| (x as u32, class_id, 1.0_f32)));
        for class in class_ids.iter().flatten() {
            if *class >= class_count {
                return Err(TrainerError::Config(format!(
                    "class id {class} is outside class_count {class_count}"
                )));
            }
        }
        let teacher_arrays = Self::write_area(width, class_count, teacher_cells)?;

        let mut frame = CorticalMappedXYZPNeuronVoxels::new();
        frame.insert(Self::misc_id(stream.amplitude_unit), amp_arrays);
        frame.insert(Self::misc_id(stream.teacher_unit), teacher_arrays);
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::profile::{StreamBinding, StreamMode};

    fn profile(width: u32) -> EncoderBindingProfile {
        EncoderBindingProfile {
            cortical_area_id: "misc_amp".to_string(),
            channels: width,
            scheme: EncodingScheme::Value,
            image_width: None,
            image_height: None,
            cortical_name: None,
            stream: Some(StreamBinding {
                parallel_width: width,
                amplitude_unit: 0,
                teacher_unit: 1,
                teacher_cortical_area_id: "misc_class".to_string(),
                teacher_cortical_name: None,
                mode: StreamMode::Train,
            }),
            teacher: None,
            segmentation_teacher: None,
        }
    }

    #[test]
    fn tick_writes_amplitude_on_x_and_class_on_y() {
        let mut encoder = MiscStreamEncoder::new();
        let frame = encoder
            .encode_tick(&[0.25, 0.75], &[Some(2), Some(0)], &profile(2), 5)
            .expect("tick");
        let amp = frame
            .get_neurons_of(&MiscStreamEncoder::misc_id(0))
            .expect("amp");
        let teacher = frame
            .get_neurons_of(&MiscStreamEncoder::misc_id(1))
            .expect("teacher");
        let amp_voxels: Vec<_> = amp
            .iter()
            .map(|n| {
                (
                    n.neuron_voxel_coordinate.x,
                    n.neuron_voxel_coordinate.y,
                    n.potential,
                )
            })
            .collect();
        assert!(amp_voxels.contains(&(0, 0, 0.25)));
        assert!(amp_voxels.contains(&(1, 0, 0.75)));
        let teacher_voxels: Vec<_> = teacher
            .iter()
            .map(|n| (n.neuron_voxel_coordinate.x, n.neuron_voxel_coordinate.y))
            .collect();
        assert!(teacher_voxels.contains(&(0, 2)));
        assert!(teacher_voxels.contains(&(1, 0)));
    }

    #[test]
    fn silent_teacher_writes_no_class_voxels() {
        let mut encoder = MiscStreamEncoder::new();
        let frame = encoder
            .encode_tick(&[0.5], &[None], &profile(1), 5)
            .expect("tick");
        let teacher = frame
            .get_neurons_of(&MiscStreamEncoder::misc_id(1))
            .expect("teacher");
        assert_eq!(teacher.len(), 0);
    }

    #[test]
    fn tick_writes_physical_amplitude() {
        let mut encoder = MiscStreamEncoder::new();
        let frame = encoder
            .encode_tick(&[-1.5, 2.25], &[Some(1), None], &profile(2), 5)
            .expect("tick");
        let amp = frame
            .get_neurons_of(&MiscStreamEncoder::misc_id(0))
            .expect("amp");
        let amp_voxels: Vec<_> = amp
            .iter()
            .map(|n| {
                (
                    n.neuron_voxel_coordinate.x,
                    n.neuron_voxel_coordinate.y,
                    n.potential,
                )
            })
            .collect();
        assert!(amp_voxels.contains(&(0, 0, -1.5)));
        assert!(amp_voxels.contains(&(1, 0, 2.25)));
    }
}
