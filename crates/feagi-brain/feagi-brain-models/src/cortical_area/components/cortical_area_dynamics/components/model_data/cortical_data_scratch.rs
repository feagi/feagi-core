use crate::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;

/// Data for a cortical area that can be mutated during cortical processing, but cannot be viewed
/// or edited by genome developers and that is not saved by the connectome (is reset every load)
pub trait CorticalDataScratch<CAQ: CorticalAreaQuantization> {
    /// Should only be set to true in the null implementation, as this flag blocks memory allocation
    const IS_NULL: bool = false;
}

/// Represents no data in this field. Is empty/ Unused. No memory will be allocated
pub struct NullCorticalDataScratch;

impl<CAQ: CorticalAreaQuantization> CorticalDataScratch<CAQ> for NullCorticalDataScratch {
    // This is the only implementation where we set this to true
    const IS_NULL: bool = true;
}
