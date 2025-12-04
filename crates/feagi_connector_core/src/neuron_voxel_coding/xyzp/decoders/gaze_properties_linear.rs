use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_data_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelDimensions, NeuronDepth};
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::data_pipeline::PipelineStageRunner;
use crate::data_types::{GazeProperties, Percentage, Percentage2D};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{decode_unsigned_percentage_from_fractional_exponential_neurons, decode_unsigned_percentage_from_linear_neurons};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::WrappedIOType;
use super::{Percentage2DExponentialNeuronVoxelXYZPDecoder, PercentageExponentialNeuronVoxelXYZPDecoder};

const ECCENTRICITY_CHANNEL_WIDTH: u32 = 2;
const MODULARITY_CHANNEL_WIDTH: u32 = 1;

#[derive(Debug)]
pub struct GazePropertiesLinearNeuronVoxelXYZPDecoder {
    channel_eccentricity_dimensions: CorticalChannelDimensions,
    channel_modularity_dimensions: CorticalChannelDimensions,
    cortical_eccentricity_read_target: CorticalID,
    cortical_modularity_read_target: CorticalID,
    z_depth_eccentricity_scratch_space: Vec<Vec<u32>>,
    z_depth_modularity_scratch_space: Vec<Vec<u32>>,
}

impl NeuronVoxelXYZPDecoder for GazePropertiesLinearNeuronVoxelXYZPDecoder {
    fn get_decoded_data_type(&self) -> WrappedIOType {
        WrappedIOType::GazeProperties
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(&mut self, neurons_to_read: &CorticalMappedXYZPNeuronVoxels, time_of_read: Instant, pipelines_with_data_to_update: &mut Vec<PipelineStageRunner>, channel_changed: &mut Vec<bool>) -> Result<(), FeagiDataError> {

        const ONLY_ALLOWED_Y: u32 = 0; // This structure never has height

        let eccentricity_neuron_array = neurons_to_read.get_neurons_of(&self.cortical_eccentricity_read_target);
        let modularity_neuron_array = neurons_to_read.get_neurons_of(&self.cortical_modularity_read_target);

        if eccentricity_neuron_array.is_none() && modularity_neuron_array.is_none() {
            return Ok(());
        }

        let eccentricity_neuron_array = eccentricity_neuron_array.unwrap();
        let modularity_neuron_array = modularity_neuron_array.unwrap();
        if eccentricity_neuron_array.is_empty() && modularity_neuron_array.is_empty() {
            return Ok(());
        }

        let number_of_channels = pipelines_with_data_to_update.len() as u32;
        let eccentricity_z_depth: u32 = self.channel_modularity_dimensions.depth;
        let modularity_z_depth: u32 = self.channel_eccentricity_dimensions.depth;

        for neuron in eccentricity_neuron_array.iter() {

            // Ignoring any neuron_voxels that have no potential (if sent for some reason).
            if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                continue; // Something is wrong, but currently we will just skip these
            }

            if neuron.neuron_voxel_coordinate.x >= (number_of_channels * ECCENTRICITY_CHANNEL_WIDTH) || neuron.neuron_voxel_coordinate.z >= eccentricity_z_depth {
                continue; // Something is wrong, but currently we will just skip these
            }

            let z_row_vector = self.z_depth_eccentricity_scratch_space.get_mut(neuron.neuron_voxel_coordinate.x as usize).unwrap();
            z_row_vector.push(neuron.neuron_voxel_coordinate.z)
        };

        for neuron in modularity_neuron_array.iter() {

            // Ignoring any neuron_voxels that have no potential (if sent for some reason).
            if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                continue; // Something is wrong, but currently we will just skip these
            }

            if neuron.neuron_voxel_coordinate.x >= (number_of_channels * MODULARITY_CHANNEL_WIDTH) || neuron.neuron_voxel_coordinate.z >= modularity_z_depth {
                continue; // Something is wrong, but currently we will just skip these
            }

            let z_row_vector = self.z_depth_modularity_scratch_space.get_mut(neuron.neuron_voxel_coordinate.x as usize).unwrap();
            z_row_vector.push(neuron.neuron_voxel_coordinate.z)
        };

        // At this point, we have numbers in scratch space to average out
        for channel_index in 0..number_of_channels as usize { // Literally not worth making parallel... right?
            let eccentricity_z_row_a_index = channel_index * ECCENTRICITY_CHANNEL_WIDTH as usize;
            let eccentricity_z_row_b_index = eccentricity_z_row_a_index + 1;
            let modularity_z_row_index = channel_index;

            // We need to ensure if ANY of the numbers changed (as in they added anything to the vector for that row that only originally had 0), we update it and label it as such

            let mut eccentricity_z_a_vector = self.z_depth_eccentricity_scratch_space.get(eccentricity_z_row_a_index).unwrap();
            let mut eccentricity_z_b_vector = self.z_depth_eccentricity_scratch_space.get(eccentricity_z_row_b_index).unwrap();
            let mut modularity_z_vector = self.z_depth_modularity_scratch_space.get(modularity_z_row_index).unwrap();

            if eccentricity_z_a_vector.is_empty() && eccentricity_z_b_vector.is_empty() && modularity_z_vector.is_empty() {
                continue; // No data collected for this channel. Do not emit
            }
            channel_changed[channel_index] = true;
            let prev_gaze: &mut GazeProperties = pipelines_with_data_to_update.get_mut(channel_index).unwrap().get_cached_input_mut().try_into()?;

            if !eccentricity_z_a_vector.is_empty() {
                decode_unsigned_percentage_from_linear_neurons(&eccentricity_z_a_vector, self.channel_eccentricity_dimensions.depth, &mut prev_gaze.eccentricity_location_xy.a);
            }

            if !eccentricity_z_b_vector.is_empty() {
                decode_unsigned_percentage_from_linear_neurons(&eccentricity_z_b_vector, self.channel_eccentricity_dimensions.depth, &mut prev_gaze.eccentricity_location_xy.b);
            }

            if !modularity_z_vector.is_empty() {
                decode_unsigned_percentage_from_linear_neurons(&modularity_z_vector, self.channel_modularity_dimensions.depth, &mut prev_gaze.modulation_size);
            }

        }

        Ok(())


    }
}


impl GazePropertiesLinearNeuronVoxelXYZPDecoder {
    pub fn new_box(eccentricity_cortical_id: CorticalID, modularity_cortical_id: CorticalID, eccentricity_z_depth: NeuronDepth, modularity_z_depth: NeuronDepth, number_channels: CorticalChannelCount) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;

        let decoder = GazePropertiesLinearNeuronVoxelXYZPDecoder {
            channel_eccentricity_dimensions: CorticalChannelDimensions::new(ECCENTRICITY_CHANNEL_WIDTH, CHANNEL_Y_HEIGHT, *eccentricity_z_depth)?,
            channel_modularity_dimensions: CorticalChannelDimensions::new(MODULARITY_CHANNEL_WIDTH, CHANNEL_Y_HEIGHT, *modularity_z_depth)?,
            cortical_eccentricity_read_target: eccentricity_cortical_id,
            cortical_modularity_read_target: modularity_cortical_id,
            z_depth_eccentricity_scratch_space: vec![Vec::new(); *number_channels as usize * ECCENTRICITY_CHANNEL_WIDTH as usize],
            z_depth_modularity_scratch_space: vec![Vec::new(); *number_channels as usize * MODULARITY_CHANNEL_WIDTH as usize],
        };
        Ok(Box::new(decoder))
    }
}