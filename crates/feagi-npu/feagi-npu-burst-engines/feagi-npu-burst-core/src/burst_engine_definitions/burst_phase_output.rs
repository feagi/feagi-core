use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::wrapped_values::EngineCorticalIndex;

#[derive(Default, Debug, Clone)]
pub enum BurstPhaseOutput<FIQ: FeagiIndexQuantization> {
    /// No action required
    #[default]
    NoFurtherActionNeeded,
    /// brain is killbinding
    BrainDeathTriggered,
    /// allocation increase requests // TODO composable only
    MoreAllocationNeeded(Vec<ItemRequestingAllocationIncrease<FIQ>>)
}

#[derive(Debug, Copy, Clone)]
pub enum ItemRequestingAllocationIncrease<FIQ: FeagiIndexQuantization> {
    MemoryCorticalArea(EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>)
}