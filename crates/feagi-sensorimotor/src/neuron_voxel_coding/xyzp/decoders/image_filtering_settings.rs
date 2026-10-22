//! Decoder for one Image Enhancements area.
//!
//! The area is a row of sliders. Absolute layout is 3 columns wide:
//! X 0 diff, X 1 brightness, X 2 contrast. Incremental layout is 6 columns
//! wide: increase then decrease for each of those sliders. Z 0 is the top of
//! the range and the last Z index is the bottom. The cortical id's frame-change
//! flag selects the layout.

use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::{ImageFilteringSettings, Percentage};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    decode_unsigned_percentage_from_fractional_exponential_neurons,
    decode_unsigned_percentage_from_linear_neurons,
};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

const CHANNEL_Y_HEIGHT: u32 = 1;
const ABSOLUTE_WIDTH: u32 = 3;
const INCREMENTAL_WIDTH: u32 = 6;
const DIFF_SLIDER: usize = 0;
const BRIGHTNESS_SLIDER: usize = 1;
const CONTRAST_SLIDER: usize = 2;

/// Decodes the three image-enhancement sliders from one cortical area.
#[derive(Debug)]
pub struct ImageFilteringSettingsNeuronVoxelXYZPDecoder {
    channel_dimensions: CorticalChannelDimensions,
    interpolation: PercentageNeuronPositioning,
    frame_change_handling: FrameChangeHandling,
    cortical_id: CorticalID,
    /// One Z-index list per column. Absolute uses the first three. Incremental uses six.
    z_scratch: Vec<Vec<u32>>,
}

impl ImageFilteringSettingsNeuronVoxelXYZPDecoder {
    /// Create a decoder for the single Image Enhancements area.
    ///
    /// `number_channels` is accepted so registration matches other motor units.
    /// The three sliders are columns of that one area, not extra device channels.
    /// The area's cortical id selects absolute (3 columns) or incremental (6 columns).
    #[allow(dead_code)]
    pub fn new_box(
        cortical_id: CorticalID,
        z_depth: NeuronDepth,
        _number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        let (frame_change_handling, id_positioning) = match cortical_id.extract_io_data_flag()? {
            IOCorticalAreaConfigurationFlag::Percentage(frame, positioning) => (frame, positioning),
            other => {
                return Err(FeagiDataError::InternalError(format!(
                    "Image enhancements expected a Percentage cortical area flag, got {other}"
                )));
            }
        };
        if id_positioning != interpolation {
            return Err(FeagiDataError::InternalError(format!(
                "Image enhancements positioning {id_positioning} does not match {interpolation}"
            )));
        }
        let width = match frame_change_handling {
            FrameChangeHandling::Absolute => ABSOLUTE_WIDTH,
            FrameChangeHandling::Incremental => INCREMENTAL_WIDTH,
        };
        let decoder = ImageFilteringSettingsNeuronVoxelXYZPDecoder {
            channel_dimensions: CorticalChannelDimensions::new(width, CHANNEL_Y_HEIGHT, *z_depth)?,
            cortical_id,
            interpolation,
            frame_change_handling,
            z_scratch: vec![Vec::new(); INCREMENTAL_WIDTH as usize],
        };
        Ok(Box::new(decoder))
    }

    fn decode_column(&self, indexes: &Vec<u32>) -> Option<f32> {
        if indexes.is_empty() {
            return None;
        }
        let mut percentage = Percentage::new_zero();
        match self.interpolation {
            PercentageNeuronPositioning::Linear => {
                decode_unsigned_percentage_from_linear_neurons(
                    indexes,
                    self.channel_dimensions.depth,
                    &mut percentage,
                );
            }
            PercentageNeuronPositioning::Fractional => {
                decode_unsigned_percentage_from_fractional_exponential_neurons(
                    indexes,
                    &mut percentage,
                );
            }
        }
        Some(percentage.get_as_0_1())
    }

    fn write_percentage(target: &mut Percentage, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        *target = Percentage::new_from_0_1(clamped)
            .unwrap_or_else(|_| Percentage::new_from_0_1_unchecked(clamped));
    }

    /// Increase column then decrease column. The Z percentage is the nudge size.
    fn incremental_slider(
        increase: Option<f32>,
        decrease: Option<f32>,
        current: f32,
    ) -> Option<f32> {
        if increase.is_none() && decrease.is_none() {
            return None;
        }
        let delta = increase.unwrap_or(0.0) - decrease.unwrap_or(0.0);
        Some((current + delta).clamp(0.0, 1.0))
    }
}

impl NeuronVoxelXYZPDecoder for ImageFilteringSettingsNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFilteringSettings
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        let depth = NeuronDepth::new(self.channel_dimensions.depth).unwrap();
        JSONDecoderProperties::ImageFilteringSettings(depth, depth, depth, self.interpolation)
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        _time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
        const ONLY_ALLOWED_Y: u32 = 0;

        let Some(neurons) = neurons_to_read.get_neurons_of(&self.cortical_id) else {
            return Ok(());
        };

        for scratch in self.z_scratch.iter_mut() {
            scratch.clear();
        }

        let z_depth = self.channel_dimensions.depth;
        let column_count = self.channel_dimensions.width as usize;
        for neuron in neurons.iter() {
            if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                continue;
            }
            let column = neuron.neuron_voxel_coordinate.x as usize;
            if column >= column_count || neuron.neuron_voxel_coordinate.z >= z_depth {
                continue;
            }
            self.z_scratch[column].push(neuron.neuron_voxel_coordinate.z);
        }

        let any_fired = self.z_scratch[..column_count]
            .iter()
            .any(|column| !column.is_empty());
        if !any_fired {
            return Ok(());
        }

        let Some((pipeline, changed_flag)) = pipelines_with_data_to_update
            .iter_mut()
            .zip(channel_changed.iter_mut())
            .next()
        else {
            return Ok(());
        };
        *changed_flag = true;
        let columns = self.z_scratch.clone();
        let settings: &mut ImageFilteringSettings =
            pipeline.get_preprocessed_cached_value_mut().try_into()?;

        match self.frame_change_handling {
            FrameChangeHandling::Absolute => {
                if let Some(diff) = self.decode_column(&columns[DIFF_SLIDER]) {
                    Self::write_percentage(settings.per_pixel_diff_threshold_mut().a_mut(), diff);
                    Self::write_percentage(settings.per_pixel_diff_threshold_mut().b_mut(), 1.0);
                }
                if let Some(brightness) = self.decode_column(&columns[BRIGHTNESS_SLIDER]) {
                    Self::write_percentage(settings.brightness_mut(), brightness);
                }
                if let Some(contrast) = self.decode_column(&columns[CONTRAST_SLIDER]) {
                    Self::write_percentage(settings.contrast_mut(), contrast);
                }
            }
            FrameChangeHandling::Incremental => {
                let diff_current = settings.per_pixel_diff_threshold().a.get_as_0_1();
                let brightness_current = settings.brightness().get_as_0_1();
                let contrast_current = settings.contrast().get_as_0_1();
                if let Some(diff) = Self::incremental_slider(
                    self.decode_column(&columns[DIFF_SLIDER * 2]),
                    self.decode_column(&columns[DIFF_SLIDER * 2 + 1]),
                    diff_current,
                ) {
                    Self::write_percentage(settings.per_pixel_diff_threshold_mut().a_mut(), diff);
                    Self::write_percentage(settings.per_pixel_diff_threshold_mut().b_mut(), 1.0);
                }
                if let Some(brightness) = Self::incremental_slider(
                    self.decode_column(&columns[BRIGHTNESS_SLIDER * 2]),
                    self.decode_column(&columns[BRIGHTNESS_SLIDER * 2 + 1]),
                    brightness_current,
                ) {
                    Self::write_percentage(settings.brightness_mut(), brightness);
                }
                if let Some(contrast) = Self::incremental_slider(
                    self.decode_column(&columns[CONTRAST_SLIDER * 2]),
                    self.decode_column(&columns[CONTRAST_SLIDER * 2 + 1]),
                    contrast_current,
                ) {
                    Self::write_percentage(settings.contrast_mut(), contrast);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
    use crate::wrapped_io_data::WrappedIOData;
    use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
    use feagi_structures::genomic::MotorCorticalUnit;
    use feagi_structures::neuron_voxels::xyzp::{
        CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays,
    };

    fn cortical_id(frame: FrameChangeHandling) -> CorticalID {
        MotorCorticalUnit::get_cortical_ids_array_for_dynamic_image_processing_with_parameters(
            frame,
            PercentageNeuronPositioning::Linear,
            CorticalUnitIndex::from(0u16),
        )[0]
    }

    fn decoder(frame: FrameChangeHandling) -> Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> {
        ImageFilteringSettingsNeuronVoxelXYZPDecoder::new_box(
            cortical_id(frame),
            NeuronDepth::new(10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap()
    }

    fn neurons(
        frame: FrameChangeHandling,
        voxels: &[(u32, u32, u32)],
    ) -> CorticalMappedXYZPNeuronVoxels {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        for &(x, y, z) in voxels {
            arrays.push(&NeuronVoxelXYZP::new(x, y, z, 1.0));
        }
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        map.insert(cortical_id(frame), arrays);
        map
    }

    fn decode(frame: FrameChangeHandling, voxels: &[(u32, u32, u32)]) -> ImageFilteringSettings {
        let mut decoder = decoder(frame);
        let map = neurons(frame, voxels);
        let mut pipelines =
            vec![
                MotorPipelineStageRunner::new(WrappedIOData::ImageFilteringSettings(
                    ImageFilteringSettings::default(),
                ))
                .unwrap(),
            ];
        let mut changed = vec![false];
        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &map,
                Instant::now(),
                &mut pipelines,
                &mut changed,
            )
            .unwrap();
        match pipelines[0].get_postprocessed_motor_value() {
            WrappedIOData::ImageFilteringSettings(settings) => *settings,
            other => panic!("expected ImageFilteringSettings, got {other:?}"),
        }
    }

    #[test]
    fn absolute_columns_are_diff_brightness_and_contrast() {
        let settings = decode(
            FrameChangeHandling::Absolute,
            &[(0, 0, 0), (1, 0, 9), (2, 0, 0)],
        );
        assert!((settings.per_pixel_diff_threshold().a.get_as_0_1() - 1.0).abs() < 1e-5);
        assert!((settings.per_pixel_diff_threshold().b.get_as_0_1() - 1.0).abs() < 1e-5);
        assert!((settings.brightness().get_as_0_1() - 0.0).abs() < 1e-5);
        assert!((settings.contrast().get_as_0_1() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn incremental_brightness_increase_nudges_the_cached_slider() {
        let settings = decode(FrameChangeHandling::Incremental, &[(2, 0, 0)]);
        assert!(
            (settings.brightness().get_as_0_1() - 1.0).abs() < 1e-5,
            "full increase from 0.5 must clamp at 1.0, got {}",
            settings.brightness().get_as_0_1()
        );
        assert!((settings.contrast().get_as_0_1() - 0.5).abs() < 1e-5);
    }
}
