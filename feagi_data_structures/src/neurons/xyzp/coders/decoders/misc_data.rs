use crate::data::descriptors::MiscDataDimensions;
use crate::data::MiscData;
use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::{CorticalChannelIndex};
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData};
use crate::neurons::xyzp::coders::NeuronXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub struct MiscDataNeuronXYZPDecoder {
    misc_data_dimensions: MiscDataDimensions,
    cortical_read_target: CorticalID,
    number_elements: usize
}

impl NeuronXYZPDecoder for MiscDataNeuronXYZPDecoder {
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
        // TODO parallelize as soon as FEAGI has tests for this to avoid these errors
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
}

impl MiscDataNeuronXYZPDecoder {
    pub fn new(cortical_read_target: CorticalID, misc_data_dimensions: MiscDataDimensions) -> Result<Self, FeagiDataError> {
        Ok(MiscDataNeuronXYZPDecoder{
            misc_data_dimensions,
            cortical_read_target,
            number_elements: misc_data_dimensions.number_elements() as usize
        })
    }
}