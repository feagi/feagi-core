/// Per mapping entry flags that denote various properties
/// bit 0 - IsMappingEntryDisabled -> If disabled, all synapses under this mapping will not execute or fire
/// bit 1 - IsInhibitory -> Synapses under this mapping subtract from the destination potential instead of adding to it
/// bit 2-7 - unused
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CorticalMappingEntryRuntimeFlags(u8);

impl CorticalMappingEntryRuntimeFlags {
    const BITMASK_IS_MAPPING_ENTRY_DISABLED: u8 = 1 << 7;
    const BITMASK_IS_INHIBITORY: u8 = 1 << 6;

    pub fn new(is_mapping_entry_disabled: bool, is_inhibitory: bool) -> CorticalMappingEntryRuntimeFlags {
        let mut out = 0u8;
        if is_mapping_entry_disabled {
            out |= Self::BITMASK_IS_MAPPING_ENTRY_DISABLED;
        }
        if is_inhibitory {
            out |= Self::BITMASK_IS_INHIBITORY;
        }
        CorticalMappingEntryRuntimeFlags(out)
    }

    pub fn get_is_mapping_entry_disabled(&self) -> bool {
        self.0 & Self::BITMASK_IS_MAPPING_ENTRY_DISABLED != 0
    }

    pub fn set_is_mapping_entry_disabled(&mut self, is_mapping_entry_disabled: bool) {
        todo!()
    }

    pub fn get_is_inhibitory(&self) -> bool {
        self.0 & Self::BITMASK_IS_INHIBITORY != 0
    }

    /// The sign the destination accumulation applies, as a multiplier rather than a branch so the
    /// synapse kernel stays divergence free on wide SIMD and GPU backends.
    pub fn polarity_sign(&self) -> f32 {
        if self.get_is_inhibitory() {
            -1.0
        } else {
            1.0
        }
    }
}
