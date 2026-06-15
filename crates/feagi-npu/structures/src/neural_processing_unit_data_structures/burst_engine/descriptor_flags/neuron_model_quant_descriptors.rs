
/// Used to hold the enums denoting the neuron model used, its quantization level,m
pub trait NeuronModelQuantDescriptors: Sized {
    // bits 0,1,2, are for Neuron Model Quantization (per neuron model basis), which can also
    // be converted to Membrane Potential Quantization
    // bits 3,4,5,6 are for Neuron Model
    // bit 7 is unused
}


#[repr(C)]
pub struct NeuronModelQuantDescriptorsCPU(u8);

impl NeuronModelQuantDescriptorsCPU
{
    // TODO
}








