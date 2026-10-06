// NOTE: Force Off and Force Fire directly impact firing in the NPU, but are not meant to be
// controlled directly by the genome developer. Force Off takes priority and will stop any neuron
// regardless of anything else, while Force Fire will allow neuron dynamics to play out as normal
// but overwrite the boolean value of firing to true. However, the genome developer should be
// provided the system to disable neurons or to force fire them, with force firing taking
// priority. This is done this way for performance reasons, ergo translating this needs to be
// part of the npu allocators job

/// Describes various genome level flags for a neuron, as well as if a neuron is actively firing
/// as a result of dynamics
/// bit 0 - Neuron Killed -> Neuron will not fire nor will it go through neuron dynamics. He's dead Jim.
/// bits 1,2,3,4 - Unused
/// bit 5 - Debug Force Off -> Neuron is forced to not fire regardless of anything else, but Neuron Dynamics still runs!
/// bit 6 - Debug Force Fire -> Neuron is forced to fire unless force off is enabled, but Neuron Dynamics still runs!
/// bit 7 - IsFiring -> Live Scratch Space, Is the neuron currently firing as a result of dynamics? 
/// This flag is used only by some burst engines, not all!
#[derive(Clone, Copy)]
pub struct NeuronRuntimeFlags(u8);

impl NeuronRuntimeFlags {
    const BITMASK_NEURON_KILLED: u8 = 1 << 0; // TODO
    const BITMASK_DEBUG_NEURON_FORCE_OFF: u8 = 1 << 5;
    const BITMASK_DEBUG_FORCE_FIRE: u8 = 1 << 6;
    const BITMASK_CACHE_FIRING: u8 = 1 << 7;

    pub fn new(force_off: bool, force_fire: bool) -> NeuronRuntimeFlags {
        let mut out = 0u8;
        if force_off {
            out |= Self::BITMASK_DEBUG_NEURON_FORCE_OFF;
        }
        if force_fire {
            out |= Self::BITMASK_DEBUG_FORCE_FIRE;
        }
        NeuronRuntimeFlags(out)
    }

    /// Prevents neuron from actually firing. Takes priority over `get_force_fire`
    pub fn get_debug_force_off(&self) -> bool {
        self.get_bit(Self::BITMASK_DEBUG_NEURON_FORCE_OFF)
    }

    /// Forces neuron to fire regardless of output of the neuron model dynamics
    pub fn get_debug_force_fire(&self) -> bool {
        self.get_bit(Self::BITMASK_DEBUG_FORCE_FIRE)
    }

    /// Is the neuron current firing as a result of neuron dynamics (NOT A FLAG, RUNTIME DATA!)
    pub fn get_firing(&self) -> bool {
        self.get_bit(Self::BITMASK_CACHE_FIRING)
    }

    pub fn set_debug_force_off(&mut self, value: bool) {
        self.set_bit(Self::BITMASK_DEBUG_NEURON_FORCE_OFF, value);
    }

    pub fn set_debug_force_fire(&mut self, value: bool) {
        self.set_bit(Self::BITMASK_DEBUG_FORCE_FIRE, value);
    }

    pub fn set_firing(&mut self, firing: bool) {
        self.set_bit(Self::BITMASK_CACHE_FIRING, firing);
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
