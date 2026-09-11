use crate::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;

/// Per neuron data that is saved to the connectome
pub trait NeuronDataInternal<CAQ: CorticalAreaQuantization> {
    /// Should only be set to true in the null implementation, as this flag blocks memory allocation
    const IS_NULL: bool = false;
}

/// Represents no data in this field. Is empty/ Unused. No memory will be allocated
pub struct NullNeuronDataInternal;

impl<CAQ: CorticalAreaQuantization> NeuronDataInternal<CAQ> for NullNeuronDataInternal {
    // This is the only implementation where we set this to true
    const IS_NULL: bool = true;
}
