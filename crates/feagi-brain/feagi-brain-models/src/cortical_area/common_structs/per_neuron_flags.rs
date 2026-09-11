pub struct PerNeuronFlags(u8);

impl PerNeuronFlags {
    // Free flags for 0, 1, 2, 3, 4
    const MASK_PROBE_IS_PRESSING_FIRE: u8 = 5;
    const MASK_PROBE_IS_REPRESSING_FIRE: u8 = 6;
    const MASK_IS_FIRING_RESULT: u8 = 7;


    /// Sets the neuron to forcibly fire (despite all other parameters or contexts), but only
    /// lasts a single burst before being reset
    pub fn force_fire(&mut self) {
        self.turn_on_bit(Self::MASK_IS_FIRING_RESULT);
    }

    /// If set to true, any firing output from the neuron will be ignored and the neuron will always
    /// fire
    pub fn set_probe_is_pressing_fire(&mut self, probe_pressing_fire: bool) {
        if probe_pressing_fire {
            self.turn_on_bit(Self::MASK_PROBE_IS_PRESSING_FIRE)
        } else {
            self.turn_off_bit(Self::MASK_PROBE_IS_PRESSING_FIRE)
        }
    }

    /// If set to true, any firing output from the neuron will be ignored and the neuron will not
    /// fire. This is however overridden by both force_fire and probe_setting_fire
    pub fn set_probe_is_repressing_fire(&mut self, probe_repressing_fire: bool) {
        if probe_repressing_fire {
            self.turn_on_bit(Self::MASK_PROBE_IS_REPRESSING_FIRE)
        } else {
            self.turn_off_bit(Self::MASK_PROBE_IS_REPRESSING_FIRE)
        }
    }

    /// Given neuron model firing result, calculates if the final psp is of firing or not, stores 
    /// it and returns it as well
    pub fn process_neuron_firing(&mut self, is_neuron_model_firing: bool) -> bool {
        let mut is_firing = is_neuron_model_firing & !self.is_bit(Self::MASK_PROBE_IS_REPRESSING_FIRE);
        is_firing |= self.is_bit(Self::MASK_PROBE_IS_PRESSING_FIRE);
        if is_firing {
            self.turn_on_bit(Self::MASK_IS_FIRING_RESULT);
        }
        self.is_bit(Self::MASK_IS_FIRING_RESULT) // get final bit without resetting
    }

    /// Get if neuron is firing, resets the neuron firing state
    pub fn get_neuron_firing(&mut self) -> bool {
        let out = self.is_bit(Self::MASK_IS_FIRING_RESULT);
        self.turn_off_bit(Self::MASK_IS_FIRING_RESULT);
        out
    }

    fn is_bit(&self, bit: u8) -> bool {
        debug_assert!(bit < 8);
        (self.0 & bit) != 0
    }

    fn turn_off_bit(&mut self, bit: u8) {
        debug_assert!(bit < 8);
        self.0 &= !bit;
    }

    fn turn_on_bit(&mut self, bit: u8) {
        debug_assert!(bit < 8);
        self.0 |= bit;
    }
}