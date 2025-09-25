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

    fn read_neuron_data_single_channel(&self, read_target: &CorticalMappedXYZPNeuronData,  cortical_channel: CorticalChannelIndex, write_target: &mut WrappedIOData) -> Result<bool, FeagiDataError> {
        const Y_OFFSET: u32 = 0;

        let target: &mut MiscData = write_target.try_into()?;
        let values = target.get_internal_data_mut();
        let x_bound = (*cortical_channel * self.misc_data_dimensions.width)..((*cortical_channel + 1) * self.misc_data_dimensions.width);
        let y_bound = 0..self.misc_data_dimensions.height;
        let z_bound = 0..self.misc_data_dimensions.depth;

        let reading_neuron_data = read_target.get_neurons_of(&self.cortical_read_target);
        if reading_neuron_data.is_none() {
            return Ok(false); // No neuron data found, returning false to state that no update was made
        }
        let reading_neuron_data = reading_neuron_data.unwrap();
        let (x_arr, y_arr, z_arr, p_arr) = reading_neuron_data.borrow_xyzp_vectors();
        for i in 0..x_arr.len() {
            let x_val = x_arr[i];
            if !x_bound.contains(&x_val) {
                continue; // Assume it belongs to another channel
            }

            let y_val = y_arr[i];
            if !y_bound.contains(&y_val) {
                return Err(FeagiDataError::InternalError(format!("Feagi Returned Misc data outside of Misc Data bounds! Y bound: {:?}, Given Y index: {}, affected cortical ID: {}", y_bound, y_arr[i], self.cortical_read_target)));
            }

            let z_val = z_arr[i];
            if !z_bound.contains(&z_val) {
                return Err(FeagiDataError::InternalError(format!("Feagi Returned Misc data outside of Misc Data bounds! Z bound: {:?}, Given Z index: {}, affected cortical ID: {}", z_bound, z_arr[i], self.cortical_read_target)));
            }

            values[(x_val as usize, y_val as usize, z_val as usize)] = p_arr[i];
        }

        Ok(true)
    }

    fn fn read_neuron_data_multi_channel(&self, channel_value_target: &mut Vec<&WrappedIOData>, did_channel_change: &mut Vec<bool>, read_target: &CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
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
                                                               neuron_array[neuron_index], self.cortical_read_target, self.misc_data_dimensions)))
            }

            let channel_index: CorticalChannelIndex = (x_arr[neuron_index] / self.misc_data_dimensions.width).into();
            if *channel_index >= *self.number_of_channels {
                return Err(FeagiDataError::NeuronError(format!("Feagi sent neuron {} which is out of bounds for corticalID {} with dimensions {}!",
                                                               neuron_array[neuron_index], self.cortical_read_target, self.misc_data_dimensions)))
            }
            if !did_channel_change[*channel_index] {
                
            }


            did_channel_change[*channel_index] = true;




        }





        Ok(())

    }

    fn get_number_of_channels(&self) -> usize {
        self.has_channel_changed.len()
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