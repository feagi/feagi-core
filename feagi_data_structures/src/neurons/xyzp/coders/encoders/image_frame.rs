use std::time::Instant;
use ndarray::parallel::prelude::IntoParallelIterator;
use crate::FeagiDataError;
use crate::data::descriptors::ImageFrameProperties;
use crate::genomic::CorticalID;
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPEncoder};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub struct ImageFrameNeuronXYZPEncoder {
    image_properties: ImageFrameProperties,
    cortical_write_target: CorticalID
}

impl NeuronXYZPEncoder for ImageFrameNeuronXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.image_properties))
    }

    fn write_neuron_data_multi_channel<'a, D, T>(&self, data_iterator: D, time_of_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError>
    where
        D: IntoParallelIterator<Item = &'a WrappedIOData>,
    {

        let data_par = data_iterator.into_par_iter();

        data_par
            .for_each(|wrapped_data| {




                // Process each (WrappedIOData, Instant) pair here
                // wrapped_data: &WrappedIOData
                // update_time: Instant

                // Your processing logic goes here
                // For example:
                // match wrapped_data {
                //     WrappedIOData::ImageFrame(image_frame) => {
                //         // Process the image frame with the update_time
                //     },
                //     _ => {
                //         // Handle other data types or error
                //     }
                // }
            });

        Ok(())


    }
}

impl ImageFrameNeuronXYZPEncoder {
    pub fn new(cortical_write_target: CorticalID, image_properties: &ImageFrameProperties) -> Result<Self, FeagiDataError> {
        Ok(ImageFrameNeuronXYZPEncoder{
            image_properties: image_properties.clone(),
            cortical_write_target
        })
    }
}