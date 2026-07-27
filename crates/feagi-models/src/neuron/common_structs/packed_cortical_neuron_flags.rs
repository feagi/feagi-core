//! Per cortical are settings that is condensed to fit within a single byte

/// Per cortical area flags that are used during the neuron burst phase of a burst.
/// bit 0 - IsCorticalAreaFrozenInput -> Cortical area will not respond to input, state is frozen
/// bit 1 - unused
/// bit 2 - unused
/// bit 3 - unused
/// bit 4 - HasPerNeuronData -> STATIC - The neuron model of this area has per neuron data
/// bit 5 - HasNeuronHistory -> STATIC - does the neuron model have history
/// bits 6-7 - `PackedCorticalAreaLayoutType` -> STATIC - The way neurons are laid out
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PackedCorticalNeuronPhaseFlags(u8);


impl PackedCorticalNeuronPhaseFlags {
    
    pub fn get_is_cortical_area_frozen_input(&self) -> bool {
        todo!()
    }
    
    
}
