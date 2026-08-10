/// Per synapse flags that denote various properties
/// bit 0-6 - unused
/// bit 6 - debug disabled -> synapse has been disabled as per debug command
/// bit 7 - killed -> synapse was disabled as per synapse dynamics (plasticity) and can be trimmed
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SynapseRuntimeFlags(u8);

impl SynapseRuntimeFlags {
    const BITMASK_SYNAPSE_DISABLED: u8 = 1 << 1;
    const BITMASK_SYNAPSE_KILLED: u8 = 1 << 0;

    pub fn new(is_synapse_disabled: bool, is_synapse_killed: bool) -> SynapseRuntimeFlags {
        let mut out = 0u8;
        if is_synapse_disabled {
            out |= Self::BITMASK_SYNAPSE_DISABLED;
        }
        if is_synapse_killed {
            out |= Self::BITMASK_SYNAPSE_KILLED;
        }
        SynapseRuntimeFlags(out)
    }

    pub fn get_is_synapse_disabled(&self) -> bool {
        self.get_bit(Self::BITMASK_SYNAPSE_DISABLED)
    }

    pub fn get_is_synapse_killed(&self) -> bool {
        self.get_bit(Self::BITMASK_SYNAPSE_KILLED)
    }

    pub fn set_is_synapse_disabled(&mut self, is_synapse_disabled: bool) {
        self.set_bit(Self::BITMASK_SYNAPSE_DISABLED, is_synapse_disabled);
    }

    pub fn set_is_synapse_killed(&mut self, is_synapse_killed: bool) {
        self.set_bit(Self::BITMASK_SYNAPSE_KILLED, is_synapse_killed);
    }

    #[inline(always)]
    fn set_bit(&mut self, bitmask: u8, value: bool) {
        self.0 = (self.0 & !bitmask) | (bitmask * value as u8);
    }

    #[inline(always)]
    fn get_bit(&self, bitmask: u8) -> bool {
        (self.0 & bitmask) != 0
    }
}
