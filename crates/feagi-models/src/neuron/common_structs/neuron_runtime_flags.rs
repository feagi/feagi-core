// NOTE: Force Off and Force Fire directly impact firing in the NPU, but are not meant to be
// controlled directly by the genome developer. Force Off takes priority and will stop any neuron
// regardless of anything else, while Force Fire will allow neuron dynamics to play out as normal
// but overwrite the boolean value of firing to true. However, the genome developer should be
// provided the system to disable neurons or to force fire them, with force firing taking
// priority. This is done this way for performance reasons, ergo translating this needs to be
// part of the npu allocators job

/// Describes various genome level flags for a neuron, as well as if a neuron is actively firing
/// as a result of dynamics
/// bit 0 - IsFiring -> Is the neuron currently firing as a result of dynamics>
/// bits 1-5 - Unused
/// bit 6 - force fire -> neuron is forced to fire unless force off is enabled
/// bit 7 - force off -> neuron is forced to not fire regardless of anything else
#[derive(Clone, Copy)]
pub struct NeuronRuntimeFlags(u8);

impl NeuronRuntimeFlags {
    const BITMASK_NEURON_FORCE_OFF: u8 = 1 << 0;
    const BITMASK_FORCE_FIRE: u8 = 1 << 1;
    const BITMASK_FIRING: u8 = 1 << 7;

    /// Prevents neuron from actually firing. Takes priority over `get_force_fire`
    pub fn get_force_off(&self) -> bool {
        (self.0 & Self::BITMASK_NEURON_FORCE_OFF) != 0
    }

    /// Forces neuron to fire regardless of output of the neuron model dynamics
    pub fn get_force_fire(&self) -> bool {
        (self.0 & Self::BITMASK_FORCE_FIRE) != 0
    }

    /// Is the neuron current firing as a result of neuron dynamics
    pub fn get_firing(&self) -> bool {
        (self.0 & Self::BITMASK_FIRING) != 0
    }

    // TODO SET commands

    pub fn set_firing(&mut self, firing: bool) {
        // TODO
    }

    pub fn new(force_off: bool, force_fire: bool) -> NeuronRuntimeFlags {
        let mut out = 0u8;
        if force_off {
            out |= Self::BITMASK_NEURON_FORCE_OFF;
        }
        if force_fire {
            out |= Self::BITMASK_FORCE_FIRE;
        }
        NeuronRuntimeFlags(out)
    }
}
