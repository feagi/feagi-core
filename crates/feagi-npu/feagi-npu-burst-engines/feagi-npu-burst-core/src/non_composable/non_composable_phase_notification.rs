use crate::wrapped_values::EngineCorticalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// An engine response variant where the engine needs some specific attention
pub enum NonComposablePhaseNotification<FIQ: FeagiIndexQuantization> {
    /// Brain is killbinding. Cease operations
    BrainDeathTriggered {
        from_cortical_index: EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>,
    },
}

impl<FIQ: FeagiIndexQuantization> NonComposablePhaseNotification<FIQ> {
    pub const NUMBER_NON_COMPOSABLE_PHASE_NOTIFICATIONS: usize = 1;
    // TODO Update manually, but swap to mem::variant_count when available in rust!
}

