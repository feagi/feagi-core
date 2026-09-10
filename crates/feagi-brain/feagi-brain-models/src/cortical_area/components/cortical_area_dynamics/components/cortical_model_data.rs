use crate::models::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;


/// A generic struct used to represent quantized data for a specific cortical model.
pub trait CorticalModelQuantizedData<CAMQ: CorticalAreaQuantization> {}

/// Represents no data in this field, empty / unused.
pub struct CorticalModelQuantizedDataNull;

impl<CAMQ: CorticalAreaQuantization> CorticalModelQuantizedData<CAMQ> for CorticalModelQuantizedDataNull {}
