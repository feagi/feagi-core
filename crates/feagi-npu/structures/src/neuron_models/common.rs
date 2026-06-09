use feagi_structures::feagi_data::feagi_pdi::tag_device::PDIDeviceClass;
use crate::neuron_models::neuron_models::feagi_standard::FeagiStandardModelQuantizationAndDeviceMode;
// TODO these should probably be macro generated somehow


/// Standard way to define what type of quantization a neuron model is using
#[derive(Default)]
pub enum NeuronModelQuantizationAndDeviceClass {

    #[default]
    FeagiStandardModel(FeagiStandardModelQuantizationAndDeviceMode)
}

impl NeuronModelQuantizationAndDeviceClass {
    pub fn get_device_class(&self) -> PDIDeviceClass
    {
        PDIDeviceClass::CPU // TODO actual logic pls!
    }

}



#[repr(u8)]
#[derive(Default)]
pub enum NeuronChunkSize {
    #[default]
    Has8NeuronsPerChunk = 8,
    Has16NeuronsPerChunk = 16,
    Has32NeuronsPerChunk = 32,
    Has64NeuronsPerChunk = 64,
}


#[repr(u8)]
#[derive(Default)]
pub enum MembranePotentialNumberBits {
    #[default]
    Bit8 = 8,
    Bit16 = 16,
    Bit32 = 32,
    Bit64 = 64,
}