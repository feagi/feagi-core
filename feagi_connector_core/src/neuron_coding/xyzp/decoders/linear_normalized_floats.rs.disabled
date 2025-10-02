use crate::error::{FeagiDataProcessingError, IODataError};
use crate::genomic_structures::{CorticalID, CorticalIOChannelIndex, SingleChannelDimensions};
use crate::io_data::{IOTypeData, IOTypeVariant, NormalizedM1To1F32};
use crate::neuron_data::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays};
use super::super::NeuronXYZPDecoder;

pub struct LinearNormalizedFloatSplitSignDividedNeuronXYZPDecoder {
    channel_dimensions: SingleChannelDimensions,
    cortical_read_source: [CorticalID; 1]
}

impl NeuronXYZPDecoder for LinearNormalizedFloatSplitSignDividedNeuronXYZPDecoder {
    fn get_decoded_data_type(&self) -> IOTypeVariant {
        IOTypeVariant::NormalizedM1to1F32
    }

    fn get_channel_dimensions(&self) -> &SingleChannelDimensions {
        &self.channel_dimensions
    }

    fn get_cortical_id_read_destinations(&self) -> &[CorticalID] {
        &self.cortical_read_source
    }

    fn read_neuron_data_single_channel(&self, cortical_channel: CorticalIOChannelIndex, read_from: &CorticalMappedXYZPNeuronData) -> Result<IOTypeData, FeagiDataProcessingError> {
        
        let cortical_id = self.cortical_read_source[0];
        if !read_from.contains(&cortical_id) {
            return Ok(NormalizedM1To1F32::new_zero().into());
        }
        const CHANNEL_WIDTH: u32 = 2;

        let neuron_data: &NeuronXYZPArrays = read_from.borrow(&cortical_id).unwrap();
        let positive_x_index: u32 = *cortical_channel * CHANNEL_WIDTH;
        let negative_x_index: u32 = positive_x_index + 1;
        let cortical_depth: f32 = self.channel_dimensions.get_z() as f32;
        
        let mut output: f32 = 0.0;
        let mut channel_neuron_count: usize = 0;
        for neuron in neuron_data.iter() {
            match neuron.x {
                positive_x_index => {
                    channel_neuron_count += 1;
                    let activation_feagi_index: f32 = (neuron.z + 1) as f32;
                    output += neuron.p * activation_feagi_index / cortical_depth;
                    continue;
                }
                negative_x_index => {
                    channel_neuron_count += 1;
                    let activation_feagi_index: f32 = (neuron.z + 1) as f32;
                    output -= neuron.p * activation_feagi_index / cortical_depth;
                    continue;
                }
                _ => {
                    // Do Nothing
                    continue;
                }
            }
        };
        output /= channel_neuron_count as f32;
        Ok(NormalizedM1To1F32::new_with_clamp(output)?.into())
    }
}

impl LinearNormalizedFloatSplitSignDividedNeuronXYZPDecoder {
    pub fn new(cortical_id_target: CorticalID, z_resolution: u32) -> Result<Self, FeagiDataProcessingError> {
        if z_resolution == 0 {
            return Err(IODataError::InvalidParameters("Resolution cannot be 0!".into()).into());
        }
        pub const CHANNEL_WIDTH: u32 = 2;
        pub const CHANNEL_HEIGHT: u32 = 1;
        Ok(LinearNormalizedFloatSplitSignDividedNeuronXYZPDecoder{
            channel_dimensions: SingleChannelDimensions::new(CHANNEL_WIDTH, CHANNEL_HEIGHT, z_resolution)?,
            cortical_read_source: [cortical_id_target; 1]
        })

    }
}


pub struct LinearNormalizedFloatPSPBirdirectionalDividedNeuronXYZPDecoder {
    channel_dimensions: SingleChannelDimensions,
    cortical_read_source: [CorticalID; 1]
}

// TODO right now the logic is a clone of the above, however we can optimize it specifically for this
impl NeuronXYZPDecoder for LinearNormalizedFloatPSPBirdirectionalDividedNeuronXYZPDecoder {
    fn get_decoded_data_type(&self) -> IOTypeVariant {
        IOTypeVariant::NormalizedM1to1F32
    }

    fn get_channel_dimensions(&self) -> &SingleChannelDimensions {
        &self.channel_dimensions
    }

    fn get_cortical_id_read_destinations(&self) -> &[CorticalID] {
        &self.cortical_read_source
    }

    fn read_neuron_data_single_channel(&self, cortical_channel: CorticalIOChannelIndex, read_from: &CorticalMappedXYZPNeuronData) -> Result<IOTypeData, FeagiDataProcessingError> {

        let cortical_id = self.cortical_read_source[0];
        if !read_from.contains(&cortical_id) {
            return Ok(NormalizedM1To1F32::new_zero().into());
        }
        const CHANNEL_WIDTH: u32 = 2;

        let neuron_data: &NeuronXYZPArrays = read_from.borrow(&cortical_id).unwrap();
        let positive_x_index: u32 = *cortical_channel * CHANNEL_WIDTH;
        let negative_x_index: u32 = positive_x_index + 1;
        let cortical_depth: f32 = self.channel_dimensions.get_z() as f32;

        let mut output: f32 = 0.0;
        let mut channel_neuron_count: usize = 0;
        for neuron in neuron_data.iter() {
            match neuron.x {
                positive_x_index => {
                    channel_neuron_count += 1;
                    let activation_feagi_index: f32 = (neuron.z + 1) as f32;
                    output += neuron.p * activation_feagi_index / cortical_depth;
                    continue;
                }
                negative_x_index => {
                    channel_neuron_count += 1;
                    let activation_feagi_index: f32 = (neuron.z + 1) as f32;
                    output -= neuron.p * activation_feagi_index / cortical_depth;
                    continue;
                }
                _ => {
                    // Do Nothing
                    continue;
                }
            }
        };
        output /= channel_neuron_count as f32;
        Ok(NormalizedM1To1F32::new_with_clamp(output)?.into())
    }
}

impl LinearNormalizedFloatPSPBirdirectionalDividedNeuronXYZPDecoder {
    pub fn new(cortical_id_target: CorticalID, z_resolution: u32) -> Result<Self, FeagiDataProcessingError> {
        if z_resolution == 0 {
            return Err(IODataError::InvalidParameters("Resolution cannot be 0!".into()).into());
        }
        pub const CHANNEL_WIDTH: u32 = 2;
        pub const CHANNEL_HEIGHT: u32 = 1;
        Ok(LinearNormalizedFloatPSPBirdirectionalDividedNeuronXYZPDecoder{
            channel_dimensions: SingleChannelDimensions::new(CHANNEL_WIDTH, CHANNEL_HEIGHT, z_resolution)?,
            cortical_read_source: [cortical_id_target; 1]
        })

    }
}