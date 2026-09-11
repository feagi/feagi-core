use crate::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;

/// Data for a cortical area that is saved to the connectome, allows mutable access for cortical
/// level processing, but is not viewable or editable by genome developers
pub trait CorticalDataInternal<CAQ: CorticalAreaQuantization> {
    /// Should only be set to true in the null implementation, as this flag blocks memory allocation
    const IS_NULL: bool = false;
}

/// Represents no data in this field. Is empty/ Unused. No memory will be allocated
pub struct NullCorticalDataInternal;

impl<CAQ: CorticalAreaQuantization> CorticalDataInternal<CAQ> for NullCorticalDataInternal {
    // This is the only implementation where we set this to true
    const IS_NULL: bool = true;
}

