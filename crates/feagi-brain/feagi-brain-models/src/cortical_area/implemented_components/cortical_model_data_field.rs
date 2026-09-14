use crate::cortical_area::implemented_components::cortical_area_quantization::CorticalAreaQuantization;

/// Various fields of cortical models just need some form of model quantized data
pub trait CorticalModelDataField<CAQ: CorticalAreaQuantization> {
    /// If true, then this field has no data and no attempt to maintain memory storage will be made
    const IS_NULL: bool = false;
}

/// A field that has no data.
pub struct NullCorticalModelDataField;

impl<CAQ: CorticalAreaQuantization> CorticalModelDataField<CAQ> for NullCorticalModelDataField
{
    /// The one exception
    const IS_NULL: bool = true;
}

