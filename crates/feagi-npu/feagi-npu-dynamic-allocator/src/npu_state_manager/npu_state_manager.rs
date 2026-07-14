use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::npu_state_manager::burst_engine_context::burst_engine_context::BurstEngineContext;

pub struct NPUStateManager<FIQ: FeagiIndexQuantization>
{
    cortical_id_mapping: (),
    engine_contexts: BurstEngineContext<FIQ>
}