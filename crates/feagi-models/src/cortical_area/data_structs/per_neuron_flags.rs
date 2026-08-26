/// Contains bitpacked data on the state of the neuron.
pub struct PerNeuronFlags(u8);

impl PerNeuronFlags {
    /// Represents a nonexistant / completely dead neuron
    pub const NULL_NEURON: Self = Self(0);

    const MASK_NEURON_IS_FIRING: u8 = 1 << 4;
    const MASK_NEURON_IS_FORCED_TO_FIRE: u8 = 1 << 5;
    const MASK_NEURON_IS_FORCED_NOT_TO_FIRE: u8 = 1 << 6; // Setting this to true forces the neuron to not fire unless force fire is on (lower priority)
    const MASK_NEURON_IS_ALIVE: u8 = 1 << 7;

    /// Returns true if the neuron is currently firing.
    pub fn is_neuron_firing(&self) -> bool {
        self.get_bit(Self::MASK_NEURON_IS_FIRING)
    }

    pub fn set_neuron_firing(&mut self, value: bool) {
        self.set_bit(Self::MASK_NEURON_IS_FIRING, value);
    }

    pub fn is_neuron_forced_to_fire(&self) -> bool {
        self.get_bit(Self::MASK_NEURON_IS_FORCED_TO_FIRE)
    }

    pub fn set_neuron_forced_to_fire(&mut self, value: bool) {
        self.set_bit(Self::MASK_NEURON_IS_FORCED_TO_FIRE, value);
    }

    pub fn is_neuron_forced_not_to_fire(&self) -> bool {
        self.get_bit(Self::MASK_NEURON_IS_FORCED_NOT_TO_FIRE)
    }

    pub fn set_neuron_forced_not_to_fire(&mut self, value: bool) {
        self.set_bit(Self::MASK_NEURON_IS_FORCED_NOT_TO_FIRE, value);
    }

    pub fn is_neuron_alive(&self) -> bool {
        self.get_bit(Self::MASK_NEURON_IS_ALIVE)
    }

    pub fn set_neuron_alive(&mut self, value: bool) {
        self.set_bit(Self::MASK_NEURON_IS_ALIVE, value);
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