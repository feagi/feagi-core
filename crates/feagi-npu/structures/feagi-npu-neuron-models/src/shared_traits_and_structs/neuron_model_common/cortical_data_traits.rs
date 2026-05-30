use feagi_structures::feagi_data::feagi_ecs::element::{FeagiECSElement};
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, FeagiGlobalIndexQuantization};



/// Root trait for all cortical data implementations
pub trait CorticalModelData<CAIQ, NMQ>:
FeagiECSElement
where
    CAIQ: FeagiGlobalIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
{
    // nothing
}

