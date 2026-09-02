use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::wrapped_values::EngineCorticalIndex;


// Possible notifications: memory cortical area allocation request, brain death
#[cfg(feature = "growable")]
const MAX_NUMBER_NOTIFICATIONS: usize = 2;

// Only notification possible is brain death
#[cfg(not(feature = "growable"))]
const MAX_NUMBER_NOTIFICATIONS: usize = 1;

pub struct BurstPhaseOutput<FIQ: FeagiIndexQuantization> {
    output: heapless::Vec<BurstEngineAttentionNotification<FIQ>, MAX_NUMBER_NOTIFICATIONS>
}

impl<FIQ: FeagiIndexQuantization> BurstPhaseOutput<FIQ> {
    pub fn new() -> Self {
        Self {
            output: heapless::Vec::new()
        }
    }

    /// Returns true if there are no notifications
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    /// Adds a notification. MUST NOT EXCEED MAX_NUMBER_NOTIFICATIONS
    pub fn add_notification(&mut self, notification: BurstEngineAttentionNotification<FIQ>) {
        debug_assert_ne!(self.output.len(), MAX_NUMBER_NOTIFICATIONS);
        unsafe {
            self.output.push(notification).unwrap_unchecked();
        }
    }

    /// Returns an iterator that goes over all contained notifications. Calling this will empty the
    /// internal vector regardless of if the data is consumed or not
    pub fn drain_all(&mut self) -> heapless::vec::Drain<'_, BurstEngineAttentionNotification<FIQ>, usize>
    {
        self.output.drain(0..)
    }
}


/// An engine response variant where the engine needs some specific attention
pub enum BurstEngineAttentionNotification<FIQ: FeagiIndexQuantization> {
    /// Brain is killbinding. Cease operations
    BrainDeathTriggered{from_cortical_index: EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>},
    #[cfg(feature = "growable")]
    /// A memory cortical area needs to increase its allocation
    MemoryCorticalAreaNeedsAllocation(Vec<EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>>)
}


