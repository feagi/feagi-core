use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::wrapped_values::EngineCorticalIndex;

#[derive(Default, Debug, Clone)]
pub enum BurstPhaseOutput<FIQ: FeagiIndexQuantization> {
    /// No action required
    #[default]
    NoFurtherActionNeeded,
    /// brain is killbinding
    BrainDeathTriggered(core::marker::PhantomData<FIQ>), // TODO phantom data only here due to needing something for the type
    #[cfg(feature = "composable")]
    BurstEngineNeedsAttention(BurstEngineNeedsAttention<FIQ>)
}

#[cfg(feature = "composable")]
/// An engine response variant where the engine needs some specific attention
#[derive(Debug, Clone)]
pub struct BurstEngineNeedsAttention<FIQ: FeagiIndexQuantization> {
    pub needs_allocation: Vec<ItemRequestingAllocationIncrease<FIQ>>, // TODO dynamic only
    // TODO other stuff
}

#[cfg(feature = "composable")]
/// Composable only, some item in the connectome of the burst engine needs more memory allocation
#[derive(Debug, Copy, Clone)]
pub enum ItemRequestingAllocationIncrease<FIQ: FeagiIndexQuantization> {
    MemoryCorticalArea(EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>)
}