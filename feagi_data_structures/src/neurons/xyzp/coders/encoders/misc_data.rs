use std::time::Instant;
use crate::data::descriptors::MiscDataDimensions;
use crate::data::MiscData;
use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::{CorticalChannelIndex};
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays, NeuronXYZPEncoder};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub struct MiscDataNeuronXYZPEncoder {
    misc_data_dimensions: MiscDataDimensions,
    cortical_write_target: CorticalID,
    number_elements: usize
}

impl NeuronXYZPEncoder for MiscDataNeuronXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::MiscData(Some(self.misc_data_dimensions))
    }

    fn write_neuron_data_multi_channel<'a, D, T>(&self, data_iterator: D, update_time_iterator: T, time_of_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError>
    where
        D: IntoParallelIterator<Item = &'a WrappedIOData>,
        <D as IntoParallelIterator>::Iter: IndexedParallelIterator,
        T: IntoParallelIterator<Item = Instant>,
        <T as IntoParallelIterator>::Iter: IndexedParallelIterator,
    {
        
        Ok(())
    }

    /*
    fn write_neuron_data_single_channel(&self, wrapped_value: &WrappedIOData, cortical_channel: CorticalChannelIndex, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        const Y_OFFSET: u32 = 0;

        let value: MiscData = wrapped_value.try_into()?;
        let values = value.get_internal_data();

        let generated_neuron_data: &mut NeuronXYZPArrays = write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target, self.number_elements);
        let channel_offset: u32 = self.misc_data_dimensions.width * *cortical_channel;

        for ((x, y, z), value) in values.indexed_iter() {
            generated_neuron_data.push_raw(x as u32 + channel_offset, y as u32, z as u32, *value);
        }
        Ok(())
    }

     */

}

impl MiscDataNeuronXYZPEncoder {
    pub fn new(cortical_write_target: CorticalID, misc_data_dimensions: MiscDataDimensions) -> Result<Self, FeagiDataError> {
        Ok(MiscDataNeuronXYZPEncoder{
            misc_data_dimensions,
            cortical_write_target,
            number_elements: misc_data_dimensions.number_elements() as usize
        })
    }
}