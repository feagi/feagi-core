/// Contains bitpacked data on the state of the neuron.
pub struct PerNeuronFlags(u8);

impl PerNeuronFlags {
    /// Represents a nonexistant / completely dead neuron
    pub const NULL_NEURON: Self = Self(0);

    const MASK_NEURON_IS_PSP_UNIFORM_AND_NOT_DIVIDED: u8 = 1 << 2;
    const MASK_NEURON_IS_PSP_MEMBRANE_DRIVEN: u8 = 1 << 3;
    const MASK_NEURON_IS_FIRING: u8 = 1 << 4;
    const MASK_NEURON_IS_FORCED_TO_FIRE: u8 = 1 << 5;
    const MASK_NEURON_IS_FORCED_NOT_TO_FIRE: u8 = 1 << 6; // Setting this to true forces the neuron to not fire unless force fire is on (lower priority)
    const MASK_NEURON_IS_ALIVE: u8 = 1 << 7;

    const MASK_PSP_UNIFORMITY_AND_MEMBRANE_DRIVEN: u8 = Self::MASK_NEURON_IS_PSP_UNIFORM_AND_NOT_DIVIDED | Self::MASK_NEURON_IS_PSP_MEMBRANE_DRIVEN;

    pub fn set_cortical_area_psp_uniformity_and_membrane_driven(psp_uniformity_and_not_divided: bool, psp_membrane_driven: bool, flags: &mut [PerNeuronFlags]) {

        let mask = if psp_uniformity_and_not_divided {Self::MASK_NEURON_IS_PSP_UNIFORM_AND_NOT_DIVIDED} else {0}
            | if psp_membrane_driven {Self::MASK_NEURON_IS_PSP_MEMBRANE_DRIVEN} else {0};

        for flag in flags {
            flag.0 = (flag.0 & !Self::MASK_PSP_UNIFORMITY_AND_MEMBRANE_DRIVEN) | mask // TODO is this right
        }
    }

    pub fn is_neuron_psp_uniform_and_not_divided(&self) -> bool {
        self.get_bit(Self::MASK_NEURON_IS_PSP_UNIFORM_AND_NOT_DIVIDED)
    }

    pub fn set_neuron_psp_uniform_and_not_divided(&mut self, value: bool) {
        self.set_bit(Self::MASK_NEURON_IS_PSP_UNIFORM_AND_NOT_DIVIDED, value);
    }

    pub fn is_neuron_psp_is_membrane_driven(&self) -> bool {
        self.get_bit(Self::MASK_NEURON_IS_PSP_MEMBRANE_DRIVEN)
    }

    pub fn set_neuron_psp_is_membrane_driven(&mut self, value: bool) {
        self.set_bit(Self::MASK_NEURON_IS_PSP_MEMBRANE_DRIVEN, value);
    }

    /// Returns true if the neuron model labeled the neuron as firing (ignoring all other flags).
    pub fn is_neuron_firing_bit(&self) -> bool {
        self.get_bit(Self::MASK_NEURON_IS_FIRING)
    }

    pub fn set_neuron_firing_bit(&mut self, value: bool) {
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