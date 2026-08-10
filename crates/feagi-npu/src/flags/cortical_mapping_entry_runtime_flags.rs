/// Per mapping entry flags that denote various properties
/// bit 0 - IsMappingEntryDisabled -> If disabled, all synapses under this mapping will not execute or fire
/// bit 1-7 - unused
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CorticalMappingEntryRuntimeFlags(u8);

impl CorticalMappingEntryRuntimeFlags {
    const BITMASK_IS_MAPPING_ENTRY_DISABLED: u8 = 1 << 7;

    pub fn new(is_mapping_entry_disabled: bool) -> CorticalMappingEntryRuntimeFlags {
        let mut out = 0u8;
        if is_mapping_entry_disabled {
            out |= Self::BITMASK_IS_MAPPING_ENTRY_DISABLED;
        }
        CorticalMappingEntryRuntimeFlags(out)
    }

    pub fn get_is_mapping_entry_disabled(&self) -> bool {
        self.0 & Self::BITMASK_IS_MAPPING_ENTRY_DISABLED != 0
    }

    pub fn set_is_mapping_entry_disabled(&mut self, is_mapping_entry_disabled: bool) {
        todo!()
    }
}
