use feagi_structures::feagi_data::feagi_ecs::element::FeagiECSElementDevice;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, FeagiGlobalIndexQuantization};



/// Root trait for all cortical data implementations
pub trait CorticalModelData<CAIQ, NMQ>:
FeagiECSElementDevice
where
    CAIQ: FeagiGlobalIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
{
    // nothing
}

