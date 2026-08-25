
/// Contains bitpacked data on the state of the neuron. All neurons are required to implement this somehow!
pub struct PerNeuronFlags(u8);

impl PerNeuronFlags {
    /// Represents a nonexistant / completely dead neuron
    pub const NULL_NEURON: Self = Self(0);

    const MASK_NEURON_IS_FIRING: u8 = 1 << 0;
    const MASK_NEURON_IS_FORCED_TO_FIRE: u8 = 1 << 1;
    const MASK_NEURON_IS_FORCED_NOT_TO_FIRE: u8 = 1 << 2; // Setting this to true forces the neuron to not fire unless force fire is on (lower priority)
    const MASK_NEURON_IS_ALIVE: u8 = 1 << 3;

    /// Returns true if the neuron is alive (and firing)
    pub fn is_neuron_firing(&self) -> bool {
        todo!()
    }

    // TODO functions for getting individual bits and setting them
}