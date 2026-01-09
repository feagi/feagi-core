//! Unified decoder for ImageFilteringSettings (brightness, contrast, diff threshold).

use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::ImageFilteringSettings;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    decode_unsigned_percentage_from_fractional_exponential_neurons,
    decode_unsigned_percentage_from_linear_neurons,
};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

const CHANNEL_Y_HEIGHT: u32 = 1;

const BRIGHTNESS_CHANNEL_WIDTH: u32 = 1;
const CONTRAST_CHANNEL_WIDTH: u32 = 1;
const DIFF_CHANNEL_WIDTH: u32 = 2; // Lower and Upper thresholds

/// Decoder for ImageFilteringSettings (supports both linear and exponential interpolation).
///
/// Decodes brightness, contrast, and per-pixel diff threshold settings from neuron data.
#[derive(Debug)]
pub struct ImageFilteringSettingsNeuronVoxelXYZPDecoder {
    channel_brightness_dimensions: CorticalChannelDimensions,
    channel_contrast_dimensions: CorticalChannelDimensions,
    channel_diff_dimensions: CorticalChannelDimensions,
    channel_diff_image_dimensions: CorticalChannelDimensions,
    interpolation: PercentageNeuronPositioning,
    brightness_cortical_id: CorticalID,
    contrast_cortical_id: CorticalID,
    diff_cortical_id: CorticalID,
    z_depth_brightness_scratch_space: Vec<Vec<u32>>,
    z_depth_contrast_scratch_space: Vec<Vec<u32>>,
    z_depth_diff_scratch_space: Vec<Vec<u32>>,
    z_depth_image_diff_scratch_space: Vec<Vec<u32>>,
}

impl ImageFilteringSettingsNeuronVoxelXYZPDecoder {
    /// Create a new boxed decoder for ImageFilteringSettings.
    ///
    /// # Arguments
    ///
    /// * `brightness_cortical_id` - Cortical ID for brightness channel
    /// * `contrast_cortical_id` - Cortical ID for contrast channel
    /// * `diff_cortical_id` - Cortical ID for diff threshold channels (2 values: lower, upper)
    /// * `brightness_z_depth` - Z depth for brightness neurons
    /// * `contrast_z_depth` - Z depth for contrast neurons
    /// * `diff_z_depth` - Z depth for diff threshold neurons
    /// * `number_channels` - Number of channels
    /// * `interpolation` - Percentage neuron positioning mode
    #[allow(dead_code)]
    pub fn new_box(
        brightness_cortical_id: CorticalID,
        contrast_cortical_id: CorticalID,
        diff_cortical_id: CorticalID,
        brightness_z_depth: NeuronDepth,
        contrast_z_depth: NeuronDepth,
        diff_z_depth: NeuronDepth,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        let decoder = ImageFilteringSettingsNeuronVoxelXYZPDecoder {
            channel_brightness_dimensions: CorticalChannelDimensions::new(
                BRIGHTNESS_CHANNEL_WIDTH,
                CHANNEL_Y_HEIGHT,
                *brightness_z_depth,
            )?,
            channel_contrast_dimensions: CorticalChannelDimensions::new(
                CONTRAST_CHANNEL_WIDTH,
                CHANNEL_Y_HEIGHT,
                *contrast_z_depth,
            )?,
            channel_diff_dimensions: CorticalChannelDimensions::new(
                DIFF_CHANNEL_WIDTH,
                CHANNEL_Y_HEIGHT,
                *diff_z_depth,
            )?,
            channel_diff_image_dimensions: CorticalChannelDimensions::new(
                DIFF_CHANNEL_WIDTH,
                CHANNEL_Y_HEIGHT,
                *diff_z_depth,
            )?,
            brightness_cortical_id,
            contrast_cortical_id,
            diff_cortical_id,
            interpolation,
            z_depth_brightness_scratch_space: vec![
                Vec::new();
                *number_channels as usize * BRIGHTNESS_CHANNEL_WIDTH as usize
            ],
            z_depth_contrast_scratch_space: vec![
                Vec::new();
                *number_channels as usize * CONTRAST_CHANNEL_WIDTH as usize
            ],
            z_depth_diff_scratch_space: vec![
                Vec::new();
                *number_channels as usize * DIFF_CHANNEL_WIDTH as usize
            ],
            z_depth_image_diff_scratch_space: vec![
                Vec::new();
                *number_channels as usize * DIFF_CHANNEL_WIDTH as usize
            ],
        };
        Ok(Box::new(decoder))
    }
}

impl NeuronVoxelXYZPDecoder for ImageFilteringSettingsNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFilteringSettings
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::ImageFilteringSettings(
            NeuronDepth::new(self.channel_brightness_dimensions.depth).unwrap(),
            NeuronDepth::new(self.channel_contrast_dimensions.depth).unwrap(),
            NeuronDepth::new(self.channel_diff_dimensions.depth).unwrap(),
            self.interpolation,
        )
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        _time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
        const ONLY_ALLOWED_Y: u32 = 0;

        let brightness_neuron_array =
            neurons_to_read.get_neurons_of(&self.brightness_cortical_id);
        let contrast_neuron_array = neurons_to_read.get_neurons_of(&self.contrast_cortical_id);
        let diff_neuron_array = neurons_to_read.get_neurons_of(&self.diff_cortical_id);
        let diff_image_neuron_array = neurons_to_read.get_neurons_of(&self.diff_cortical_id);

        // Check if we have any data to process
        if brightness_neuron_array.is_none()
            && contrast_neuron_array.is_none()
            && diff_neuron_array.is_none()
            && diff_image_neuron_array.is_none()
        {
            return Ok(());
        }

        // Clear scratch spaces
        for scratch in self.z_depth_brightness_scratch_space.iter_mut() {
            scratch.clear();
        }
        for scratch in self.z_depth_contrast_scratch_space.iter_mut() {
            scratch.clear();
        }
        for scratch in self.z_depth_diff_scratch_space.iter_mut() {
            scratch.clear();
        }
        for scratch in self.z_depth_image_diff_scratch_space.iter_mut() {
            scratch.clear();
        }


        let number_of_channels = pipelines_with_data_to_update.len() as u32;
        let brightness_z_depth: u32 = self.channel_brightness_dimensions.depth;
        let contrast_z_depth: u32 = self.channel_contrast_dimensions.depth;
        let diff_z_depth: u32 = self.channel_diff_dimensions.depth;

        // Collect brightness neuron data
        if let Some(brightness_neurons) = brightness_neuron_array {
            for neuron in brightness_neurons.iter() {
                if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                    continue;
                }

                if neuron.neuron_voxel_coordinate.x
                    >= (number_of_channels * BRIGHTNESS_CHANNEL_WIDTH)
                    || neuron.neuron_voxel_coordinate.z >= brightness_z_depth
                {
                    continue;
                }

                let z_row_vector = self
                    .z_depth_brightness_scratch_space
                    .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                    .unwrap();
                z_row_vector.push(neuron.neuron_voxel_coordinate.z);
            }
        }

        // Collect contrast neuron data
        if let Some(contrast_neurons) = contrast_neuron_array {
            for neuron in contrast_neurons.iter() {
                if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                    continue;
                }

                if neuron.neuron_voxel_coordinate.x
                    >= (number_of_channels * CONTRAST_CHANNEL_WIDTH)
                    || neuron.neuron_voxel_coordinate.z >= contrast_z_depth
                {
                    continue;
                }

                let z_row_vector = self
                    .z_depth_contrast_scratch_space
                    .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                    .unwrap();
                z_row_vector.push(neuron.neuron_voxel_coordinate.z);
            }
        }

        // Collect diff threshold neuron data
        if let Some(diff_neurons) = diff_neuron_array {
            for neuron in diff_neurons.iter() {
                if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                    continue;
                }

                if neuron.neuron_voxel_coordinate.x >= (number_of_channels * DIFF_CHANNEL_WIDTH)
                    || neuron.neuron_voxel_coordinate.z >= diff_z_depth
                {
                    continue;
                }

                let z_row_vector = self
                    .z_depth_diff_scratch_space
                    .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                    .unwrap();
                z_row_vector.push(neuron.neuron_voxel_coordinate.z);
            }
        }

        // Collect image diff threshold neuron data
        if let Some(diff_neurons) = diff_image_neuron_array {
            for neuron in diff_neurons.iter() {
                if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                    continue;
                }

                if neuron.neuron_voxel_coordinate.x >= (number_of_channels * DIFF_CHANNEL_WIDTH)
                    || neuron.neuron_voxel_coordinate.z >= diff_z_depth
                {
                    continue;
                }

                let z_row_vector = self
                    .z_depth_diff_scratch_space
                    .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                    .unwrap();
                z_row_vector.push(neuron.neuron_voxel_coordinate.z);
            }
        }

        // Decode into pipeline caches
        for (channel_index, (pipeline, changed_flag)) in pipelines_with_data_to_update
            .iter_mut()
            .zip(channel_changed.iter_mut())
            .enumerate()
            .take(number_of_channels as usize)
        {
            let brightness_z_row_index = channel_index * BRIGHTNESS_CHANNEL_WIDTH as usize;
            let contrast_z_row_index = channel_index * CONTRAST_CHANNEL_WIDTH as usize;
            let diff_z_row_lower_index = channel_index * DIFF_CHANNEL_WIDTH as usize;
            let diff_z_row_upper_index = diff_z_row_lower_index + 1;
            let diff_image_z_row_lower_index = channel_index * DIFF_CHANNEL_WIDTH as usize;
            let diff_image_z_row_upper_index = diff_image_z_row_lower_index + 1;

            let brightness_z_vector = self
                .z_depth_brightness_scratch_space
                .get(brightness_z_row_index)
                .unwrap();
            let contrast_z_vector = self
                .z_depth_contrast_scratch_space
                .get(contrast_z_row_index)
                .unwrap();
            let diff_lower_z_vector = self
                .z_depth_diff_scratch_space
                .get(diff_z_row_lower_index)
                .unwrap();
            let diff_upper_z_vector = self
                .z_depth_diff_scratch_space
                .get(diff_z_row_upper_index)
                .unwrap();
            let diff_image_lower_z_vector = self
                .z_depth_image_diff_scratch_space
                .get(diff_image_z_row_lower_index)
                .unwrap();
            let diff_image_upper_z_vector = self
                .z_depth_image_diff_scratch_space
                .get(diff_image_z_row_upper_index)
                .unwrap();


            if brightness_z_vector.is_empty()
                && contrast_z_vector.is_empty()
                && diff_lower_z_vector.is_empty()
                && diff_upper_z_vector.is_empty()
                && diff_image_lower_z_vector.is_empty()
                && diff_image_upper_z_vector.is_empty()
            {
                continue;
            }

            *changed_flag = true;
            let prev_settings: &mut ImageFilteringSettings =
                pipeline.get_preprocessed_cached_value_mut().try_into()?;

            match self.interpolation {
                PercentageNeuronPositioning::Linear => {
                    if !brightness_z_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            brightness_z_vector,
                            self.channel_brightness_dimensions.depth,
                            prev_settings.brightness_mut(),
                        );
                    }
                    if !contrast_z_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            contrast_z_vector,
                            self.channel_contrast_dimensions.depth,
                            prev_settings.contrast_mut(),
                        );
                    }
                    if !diff_lower_z_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            diff_lower_z_vector,
                            self.channel_diff_dimensions.depth,
                            prev_settings.per_pixel_diff_threshold_mut().a_mut(),
                        );
                    }
                    if !diff_upper_z_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            diff_upper_z_vector,
                            self.channel_diff_dimensions.depth,
                            prev_settings.per_pixel_diff_threshold_mut().b_mut(),
                        );
                    }
                    if !diff_image_lower_z_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            diff_image_lower_z_vector,
                            self.channel_diff_image_dimensions.depth,
                            prev_settings.image_diff_threshold_mut().a_mut(),
                        );
                    }
                    if !diff_image_upper_z_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            diff_image_upper_z_vector,
                            self.channel_diff_image_dimensions.depth,
                            prev_settings.image_diff_threshold_mut().b_mut(),
                        );
                    }

                }
                PercentageNeuronPositioning::Fractional => {
                    if !brightness_z_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            brightness_z_vector,
                            prev_settings.brightness_mut(),
                        );
                    }
                    if !contrast_z_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            contrast_z_vector,
                            prev_settings.contrast_mut(),
                        );
                    }
                    if !diff_lower_z_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            diff_lower_z_vector,
                            prev_settings.per_pixel_diff_threshold_mut().a_mut(),
                        );
                    }
                    if !diff_upper_z_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            diff_upper_z_vector,
                            prev_settings.per_pixel_diff_threshold_mut().b_mut(),
                        );
                    }
                    if !diff_image_lower_z_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            diff_image_lower_z_vector,
                            prev_settings.image_diff_threshold_mut().a_mut(),
                        );
                    }
                    if !diff_image_upper_z_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            diff_image_upper_z_vector,
                            prev_settings.image_diff_threshold_mut().b_mut(),
                        );
                    }

                }
            }
        }

        Ok(())
    }
}
