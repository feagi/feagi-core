use std::time::Instant;
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

    fn write_neuron_data_multi_channel(&self, data_iterator: &dyn Iterator<Item=&WrappedIOData>, update_time_iterator: &dyn Iterator<Item=Instant>, time_of_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        todo!()
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