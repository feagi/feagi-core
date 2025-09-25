use crate::data::descriptors::MiscDataDimensions;
use crate::data::MiscData;
use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex};
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData};
use crate::neurons::xyzp::coders::NeuronXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub struct MiscDataNeuronXYZPAbsoluteDecoder {
    misc_data_dimensions: MiscDataDimensions,
    cortical_read_target: CorticalID,
    number_of_channels: CorticalChannelCount,
}

impl NeuronXYZPDecoder for MiscDataNeuronXYZPAbsoluteDecoder {
    fn get_decoded_data_type(&self) -> WrappedIOType {
        WrappedIOType::MiscData(Some(self.misc_data_dimensions))
    }

    fn read_neuron_data_multi_channel(&self, channel_value_target: &mut Vec<&mut WrappedIOData>, did_channel_change: &mut Vec<bool>, read_target: &CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        did_channel_change.fill(false);

        let neuron_array = read_target.get_neurons_of(&self.cortical_read_target);
        if neuron_array.is_none() {
            return Ok(()); // Cortical Area not in data. Nothing was changed
        }

        let neuron_array = neuron_array.unwrap();
        let (x_arr, y_arr, z_arr, p_arr) = neuron_array.borrow_xyzp_vectors();

        for neuron_index in 0..neuron_array.len() {
            if y_arr[neuron_index] >= self.misc_data_dimensions.height || z_arr[neuron_index] >= self.misc_data_dimensions.depth {
                return Err(FeagiDataError::NeuronError(format!("Feagi sent neuron {} which is out of bounds for corticalID {} with dimensions {}!",
                                                               neuron_array.get(neuron_index)?, self.cortical_read_target, self.misc_data_dimensions)))
            }

            let channel_index: CorticalChannelIndex = (x_arr[neuron_index] / self.misc_data_dimensions.width).into();
            if *channel_index >= *self.number_of_channels {
                return Err(FeagiDataError::NeuronError(format!("Feagi sent neuron {} which is out of bounds for corticalID {} with dimensions {}!",
                                                               neuron_array.get(neuron_index)?, self.cortical_read_target, self.misc_data_dimensions)))
            }
            let data: &mut MiscData = channel_value_target[*channel_index as usize].try_into()?;
            if !did_channel_change[*channel_index as usize] {
                data.blank_data();
            }
            let data_view = data.get_internal_data_mut();

            data_view[[
                (x_arr[neuron_index] % self.misc_data_dimensions.width) as usize,
                y_arr[neuron_index] as usize,
                z_arr[neuron_index] as usize,
                ]] = p_arr[neuron_index].clamp(-1.0, 1.0);
            did_channel_change[*channel_index as usize] = true;
        }
        Ok(())

    }

    fn get_number_of_channels(&self) -> CorticalChannelCount {
        self.number_of_channels
    }
}

impl MiscDataNeuronXYZPAbsoluteDecoder {
    pub fn new(cortical_read_target: CorticalID, misc_data_dimensions: MiscDataDimensions, number_of_channels: CorticalChannelCount) -> Result<Self, FeagiDataError> {
        Ok(MiscDataNeuronXYZPAbsoluteDecoder{
            misc_data_dimensions,
            cortical_read_target,
            number_of_channels,
        })
    }
}