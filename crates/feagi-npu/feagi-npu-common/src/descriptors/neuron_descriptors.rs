

/// Describes various metadata for a single neuron within a single byte
#[derive(Clone, Copy)]
pub struct NeuronDescriptors(u8);

impl NeuronDescriptors {
    const BITMASK_NEURON_ENABLED: u8 = 1 << 0;
    const BITMASK_FORCE_FIRE: u8 = 1 << 1;
    const BITMASK_KEEP_MP_CONSTANT: u8 = 1 << 2;
    
    pub fn get_enabled(self) -> bool {
        (self.0 & Self::BITMASK_NEURON_ENABLED) != 0
    }
    
    pub fn get_force_fire(self) -> bool {
        (self.0 & Self::BITMASK_FORCE_FIRE) != 0
    }
    
    pub fn get_keep_mp_constant(self) -> bool {
        (self.0 & Self::BITMASK_KEEP_MP_CONSTANT) != 0
    }
    
    pub fn new(
        is_enabled: bool,
        force_fire: bool,
        keep_mp_constant: bool,
    ) -> NeuronDescriptors
    {
        let mut out = 0u8;
        if is_enabled {
            out |= Self::BITMASK_NEURON_ENABLED;
        }
        if force_fire {
            out |= Self::BITMASK_FORCE_FIRE;
        }
        if keep_mp_constant {
            out |= Self::BITMASK_KEEP_MP_CONSTANT;
        }
        NeuronDescriptors(out)
    }
}




