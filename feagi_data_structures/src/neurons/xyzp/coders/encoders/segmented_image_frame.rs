use crate::data::image_descriptors::SegmentedImageFrameProperties;
use crate::data::SegmentedImageFrame;
use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::CorticalChannelIndex;
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPEncoder};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub struct SegmentedImageFrameNeuronXYZPEncoder {
    segmented_image_properties: SegmentedImageFrameProperties,
    cortical_write_targets: [CorticalID; 9],
}

impl NeuronXYZPEncoder for SegmentedImageFrameNeuronXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        // Since changing Image Frame Properties often mean changing channel size, we shouldn't allow doing that
        WrappedIOType::SegmentedImageFrame(Some(self.segmented_image_properties))
    }

    fn write_neuron_data_single_channel(&self, wrapped_value: &WrappedIOData, cortical_channel: CorticalChannelIndex, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        // We are not doing any sort of verification checks here, other than ensuring data types
        let segmented_image: &SegmentedImageFrame = wrapped_value.try_into()?;
        segmented_image.write_as_neuron_xyzp_data(write_target, cortical_channel, &self.cortical_write_targets)?;
        Ok(())
    }
}

impl SegmentedImageFrameNeuronXYZPEncoder {
    pub fn new(cortical_write_targets: [CorticalID; 9], segmented_image_properties: SegmentedImageFrameProperties) -> Result<Self, FeagiDataError> {
        Ok(SegmentedImageFrameNeuronXYZPEncoder{
            segmented_image_properties: segmented_image_properties,
            cortical_write_targets
        })
    }
}