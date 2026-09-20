//! Snapshot encoder onto Miscellaneous IPU.
//!
//! Population scheme: each `[0, 1]` feature fires one voxel at
//! `(x = feature, y = 0, z = bin)` with P = 1.0. Value scheme writes each analog
//! feature as graded P at `(x, 0, 0)` without clamping. ECG snapshot also holds
//! the class on Misc B (unit from `encoder_profile.teacher`, `1 × C × 1`).

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

use crate::binding::encoder::{EncoderPlugin, ObservationEncoder};
use crate::binding::encoding_scheme::{BinSpacing, EncodingScheme};
use crate::binding::profile::EncoderBindingProfile;
use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::ir_sample::{IRSample, Payload, TypedTarget};
use crate::error::TrainerError;

/// Sensory cortical unit index this encoder writes (Misc IPU unit 0).
const SNAPSHOT_MISC_UNIT: u16 = 0;

/// Stateless selector that encodes tabular features as population spikes on Misc IPU.
#[derive(Debug, Clone, Copy, Default)]
pub struct PopulationEncoder;

impl PopulationEncoder {
    /// Creates a new selector.
    pub fn new() -> Self {
        Self
    }

    /// Cortical ID for a Misc IPU unit. Must match ensure_io and FEAGI's little-endian unit index.
    pub fn misc_cortical_id(unit: u16) -> feagi_structures::genomic::cortical_area::CorticalID {
        SensoryCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
            FrameChangeHandling::Absolute,
            CorticalUnitIndex::from(unit),
        )[0]
    }

    /// Cortical ID the snapshot magnitude encoder writes. Must match ensure_io Misc unit 0.
    pub fn target_cortical_id() -> feagi_structures::genomic::cortical_area::CorticalID {
        Self::misc_cortical_id(SNAPSHOT_MISC_UNIT)
    }

    fn write_misc_volume(
        width: u32,
        height: u32,
        depth: u32,
        cells: impl Iterator<Item = (u32, u32, u32, f32)>,
    ) -> Result<NeuronVoxelXYZPArrays, TrainerError> {
        let dims = MiscDataDimensions::new(width, height, depth).map_err(map_err)?;
        let mut data = MiscData::new(&dims).map_err(map_err)?;
        {
            let grid = data.get_internal_data_mut();
            for (x, y, z, value) in cells {
                if x >= width || y >= height || z >= depth {
                    return Err(TrainerError::Config(format!(
                        "snapshot voxel ({x},{y},{z}) is outside {width}x{height}x{depth}"
                    )));
                }
                grid[(x as usize, y as usize, z as usize)] = value;
            }
        }
        let mut arrays = NeuronVoxelXYZPArrays::new();
        data.overwrite_neuron_data(&mut arrays, CorticalChannelIndex::from(0u32))
            .map_err(map_err)?;
        Ok(arrays)
    }

    /// Linear inverted Z: value 1.0 → z 0; value 0.0 → z bins-1.
    pub fn linear_z_index(value01: f64, bins: u32) -> Result<u32, TrainerError> {
        if bins == 0 {
            return Err(TrainerError::Config(
                "population encoder requires bins > 0".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&value01) {
            return Err(TrainerError::Config(format!(
                "feature = {value01} is not a normalized value in [0,1]"
            )));
        }
        let max_idx = bins.saturating_sub(1);
        let raw_idx = ((1.0 - value01) * f64::from(bins)).floor() as u32;
        Ok(raw_idx.min(max_idx))
    }

    /// Writes voxels without the MiscData [-1, 1] clamp so graded P stays physical-unit.
    fn write_raw_cells(
        cells: impl Iterator<Item = (u32, u32, u32, f32)>,
    ) -> Result<NeuronVoxelXYZPArrays, TrainerError> {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        for (x, y, z, value) in cells {
            if !value.is_finite() {
                return Err(TrainerError::Config(format!(
                    "snapshot feature at ({x},{y},{z}) is not finite"
                )));
            }
            arrays.push_raw(x, y, z, value);
        }
        Ok(arrays)
    }

    /// Shared core: encodes features onto Misc IPU unit 0.
    ///
    /// Value scheme writes each feature as graded P at `(x, 0, 0)`. Population scheme
    /// maps `[0, 1]` onto Z and writes P = 1.0.
    fn encode_features(
        &self,
        features: &[f64],
        profile: &EncoderBindingProfile,
    ) -> Result<CorticalMappedXYZPNeuronVoxels, TrainerError> {
        if features.len() != profile.channels as usize {
            return Err(TrainerError::Config(format!(
                "feature count {} does not match profile channels {}",
                features.len(),
                profile.channels
            )));
        }
        if profile.channels == 0 {
            return Err(TrainerError::Config(
                "population encoder requires channels > 0".to_string(),
            ));
        }

        let arrays = match &profile.scheme {
            EncodingScheme::Value => {
                let mut collected = Vec::with_capacity(features.len());
                for (channel, &feature) in features.iter().enumerate() {
                    if !feature.is_finite() {
                        return Err(TrainerError::Config(format!(
                            "feature[{channel}] = {feature} is not finite"
                        )));
                    }
                    collected.push((channel as u32, 0u32, 0u32, feature as f32));
                }
                Self::write_raw_cells(collected.into_iter())?
            }
            EncodingScheme::PopulationSingleSpike { .. } => {
                let resolved = profile.scheme.resolve()?;
                if resolved.spacing != BinSpacing::Linear {
                    return Err(TrainerError::Config(
                        "population encoder on Miscellaneous IPU supports linear bin spacing only"
                            .to_string(),
                    ));
                }
                let mut collected = Vec::with_capacity(features.len());
                for (channel, &feature) in features.iter().enumerate() {
                    let z = Self::linear_z_index(feature, resolved.bins)?;
                    collected.push((channel as u32, 0u32, z, 1.0_f32));
                }
                Self::write_misc_volume(profile.channels, 1, resolved.bins, collected.into_iter())?
            }
            other => {
                return Err(TrainerError::Config(format!(
                    "snapshot encoder does not support scheme '{}'",
                    other.name()
                )));
            }
        };
        let mut frame = CorticalMappedXYZPNeuronVoxels::new();
        frame.insert(Self::target_cortical_id(), arrays);
        Ok(frame)
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
            Payload::Tabular(values) => values.as_slice(),
            Payload::TimeSeries { samples, .. } => samples.as_slice(),
            other => {
                return Err(TrainerError::Config(format!(
                    "population encoder requires a tabular or time-series payload, got {other:?}"
                )))
            }
        };
        let mut frame = self.encode_features(features, profile)?;
        if let Some(teacher) = &profile.teacher {
            teacher.validate()?;
            let class_id = match &sample.target {
                Some(TypedTarget::Class { class_id, .. }) => *class_id,
                Some(other) => {
                    return Err(TrainerError::Config(format!(
                        "snapshot class teacher requires a class target, got {other:?}"
                    )))
                }
                None => {
                    return Err(TrainerError::Config(
                        "snapshot class teacher requires a labeled class target".to_string(),
                    ))
                }
            };
            if class_id >= teacher.class_count {
                return Err(TrainerError::Config(format!(
                    "class id {class_id} is outside teacher class_count {}",
                    teacher.class_count
                )));
            }
            let teacher_arrays = Self::write_misc_volume(
                1,
                teacher.class_count,
                1,
                std::iter::once((0u32, class_id, 0u32, 1.0_f32)),
            )?;
            frame.insert(Self::misc_cortical_id(teacher.unit), teacher_arrays);
        }
        Ok(frame)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::encoding_scheme::EncodingScheme;
    use crate::binding::profile::EncoderBindingProfile;

    fn profile(channels: u32, bins: u32) -> EncoderBindingProfile {
        EncoderBindingProfile {
            cortical_area_id: "misc_input".to_string(),
            channels,
            scheme: EncodingScheme::PopulationSingleSpike {
                bins,
                spacing: BinSpacing::Linear,
            },
            image_width: None,
            image_height: None,
            cortical_name: None,
            stream: None,
            teacher: None,
            segmentation_teacher: None,
        }
    }

    #[test]
    fn linear_z_is_inverted() {
        assert_eq!(PopulationEncoder::linear_z_index(1.0, 10).expect("z"), 0);
        assert_eq!(PopulationEncoder::linear_z_index(0.0, 10).expect("z"), 9);
        assert_eq!(PopulationEncoder::linear_z_index(0.5, 10).expect("z"), 5);
        assert_eq!(PopulationEncoder::linear_z_index(0.0, 1).expect("z"), 0);
    }

    #[test]
    fn writes_population_spikes_on_misc_ipu() {
        let encoder = PopulationEncoder::new();
        let frame = encoder
            .encode_features(&[1.0, 0.0], &profile(2, 4))
            .expect("encode");
        let neurons = frame
            .get_neurons_of(&PopulationEncoder::target_cortical_id())
            .expect("misc area");
        let voxels: Vec<_> = neurons
            .iter()
            .map(|n| {
                (
                    n.neuron_voxel_coordinate.x,
                    n.neuron_voxel_coordinate.y,
                    n.neuron_voxel_coordinate.z,
                    n.potential,
                )
            })
            .collect();
        assert!(voxels.contains(&(0, 0, 0, 1.0)));
        assert!(voxels.contains(&(1, 0, 3, 1.0)));
        assert_eq!(voxels.len(), 2);
    }

    #[test]
    fn rejects_fractional_spacing() {
        let profile = EncoderBindingProfile {
            cortical_area_id: "misc_input".to_string(),
            channels: 1,
            scheme: EncodingScheme::PopulationSingleSpike {
                bins: 4,
                spacing: BinSpacing::Fractional,
            },
            image_width: None,
            image_height: None,
            cortical_name: None,
            stream: None,
            teacher: None,
            segmentation_teacher: None,
        };
        let result = PopulationEncoder::new().encode_features(&[0.5], &profile);
        assert!(result.is_err());
    }

    #[test]
    fn writes_class_teacher_on_misc_b() {
        use crate::binding::profile::ClassTeacherBinding;
        use crate::contracts::common::{DatasetVersionId, Modality, OutputType, SampleId, Split};
        use crate::contracts::ir_sample::SCHEMA_VERSION;
        use std::collections::BTreeMap;

        let mut profile = profile(2, 1);
        profile.teacher = Some(ClassTeacherBinding {
            unit: 1,
            cortical_area_id: "misc_input_teacher".to_string(),
            cortical_name: Some("ECG Class Teacher IPU".to_string()),
            class_count: 5,
        });
        let sample = IRSample {
            schema_version: SCHEMA_VERSION,
            sample_id: SampleId("s".to_string()),
            dataset_version_id: DatasetVersionId("d".to_string()),
            split: Split::Train,
            modality: Modality::TimeSeries,
            payload: Payload::TimeSeries {
                sample_rate_hz: 360.0,
                channel_ids: vec!["lead_0".to_string()],
                samples: vec![1.0, 0.0],
                hold_ends: None,
            },
            target: Some(TypedTarget::Class {
                class_id: 2,
                label: None,
            }),
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        };
        let mut encoder = PopulationEncoder::new();
        let frame = encoder.encode(&sample, &profile).expect("encode");
        let teacher = frame
            .get_neurons_of(&PopulationEncoder::misc_cortical_id(1))
            .expect("teacher");
        let voxels: Vec<_> = teacher
            .iter()
            .map(|n| {
                (
                    n.neuron_voxel_coordinate.x,
                    n.neuron_voxel_coordinate.y,
                    n.potential,
                )
            })
            .collect();
        assert_eq!(voxels, vec![(0, 2, 1.0)]);
    }

    #[test]
    fn value_scheme_writes_physical_p() {
        let profile = EncoderBindingProfile {
            cortical_area_id: "misc_input".to_string(),
            channels: 3,
            scheme: EncodingScheme::Value,
            image_width: None,
            image_height: None,
            cortical_name: None,
            stream: None,
            teacher: None,
            segmentation_teacher: None,
        };
        let encoder = PopulationEncoder::new();
        let frame = encoder
            .encode_features(&[-1.5, 0.0, 2.25], &profile)
            .expect("encode");
        let neurons = frame
            .get_neurons_of(&PopulationEncoder::target_cortical_id())
            .expect("misc area");
        let voxels: Vec<_> = neurons
            .iter()
            .map(|n| {
                (
                    n.neuron_voxel_coordinate.x,
                    n.neuron_voxel_coordinate.y,
                    n.neuron_voxel_coordinate.z,
                    n.potential,
                )
            })
            .collect();
        assert_eq!(
            voxels,
            vec![(0, 0, 0, -1.5), (1, 0, 0, 0.0), (2, 0, 0, 2.25)]
        );
    }
}
