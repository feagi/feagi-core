use crate::wrapped_values::EngineCorticalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// An engine response variant where the engine needs some specific attention
pub enum ComposablePhaseNotification<FIQ: FeagiIndexQuantization> {
    /// Brain is killbinding. Cease operations
    BrainDeathTriggered {
        from_cortical_index: EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>,
    },
    /// A memory cortical area needs to increase its allocation
    MemoryCorticalAreaNeedsAllocation(Vec<EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>>),
}

impl<FIQ: FeagiIndexQuantization> ComposablePhaseNotification<FIQ> {
    pub const NUMBER_COMPOSABLE_PHASE_NOTIFICATIONS: usize = 2;
    // TODO Update manually, but swap to mem::variant_count when available in rust!
}