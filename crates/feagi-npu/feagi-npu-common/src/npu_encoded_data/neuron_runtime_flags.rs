
// NOTE: Force Off and Force Fire directly impact firing in the NPU, but are not meant to be
// controlled directly by the genome developer. Force Off takes priority and will stop any neuron
// regardless of anything else, while Force Fire will allow neuron dynamics to play out as normal
// but overwrite the boolean value of firing to true. However, the genome developer should be
// provided the system to disable neurons or to force fire them, with force firing taking
// priority. This is done this way for performance reasons, ergo translating this needs to be
// part of the npu allocators job

/// Describes various genome level flags for a neuron
#[derive(Clone, Copy)]
pub struct NeuronRuntimeFlags(u8);

impl NeuronRuntimeFlags {
    const BITMASK_NEURON_FORCE_OFF: u8 = 1 << 0;
    const BITMASK_FORCE_FIRE: u8 = 1 << 1;
    const BITMASK_KEEP_MP_CONSTANT: u8 = 1 << 2;
    
    pub fn get_force_off(self) -> bool {
        (self.0 & Self::BITMASK_NEURON_FORCE_OFF) != 0
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
    ) -> NeuronRuntimeFlags
    {
        let mut out = 0u8;
        if is_enabled {
            out |= Self::BITMASK_NEURON_FORCE_OFF;
        }
        if force_fire {
            out |= Self::BITMASK_FORCE_FIRE;
        }
        if keep_mp_constant {
            out |= Self::BITMASK_KEEP_MP_CONSTANT;
        }
        NeuronRuntimeFlags(out)
    }
}




