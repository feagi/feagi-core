use feagi_structures::feagi_data::feagi_ecs::element::FeagiECSElementOnDevice;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, CorticalAreasIndexQuantization};



/// Root trait for all cortical data implementations
pub trait CorticalModelData<CAIQ, NMQ>:
FeagiECSElementOnDevice
where
    CAIQ: CorticalAreasIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
{
    // nothing
}

