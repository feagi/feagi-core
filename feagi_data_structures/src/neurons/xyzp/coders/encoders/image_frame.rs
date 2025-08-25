use crate::data::image_descriptors::ImageFrameProperties;
use crate::data::ImageFrame;
use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::CorticalChannelIndex;
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPEncoder};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub struct ImageFrameNeuronXYZPEncoder {
    image_properties: ImageFrameProperties,
    cortical_write_target: CorticalID
}

impl NeuronXYZPEncoder for ImageFrameNeuronXYZPEncoder {

    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.image_properties))
    }

    fn write_neuron_data_single_channel(&self, wrapped_value: &WrappedIOData, cortical_channel: CorticalChannelIndex, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        // We are not doing any sort of verification checks here, other than ensuring data types
        let image: &ImageFrame = wrapped_value.try_into()?;
        image.write_as_neuron_xyzp_data(write_target, self.cortical_write_target, cortical_channel)?;
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