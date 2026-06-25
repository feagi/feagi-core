use crate::neural_processing_unit_data_structures::burst_engine::descriptor_flags::cortical_area_layout::CorticalAreaLayoutType;

#[repr(C)]
pub struct NeuronModelCorticalDescriptorsCPU(u8);
// bits 0, 1, 2 are for cortical layout
// bit 3 is unused,
// bit 4 is if the neuron model uses burst index history
// bit 5 is if post synaptic potential is driven by membrane potential
// bit 6 is if post synaptic potential should be uniform
// bit 7 is if cortical area is alive


impl NeuronModelCorticalDescriptorsCPU {

    const BITMASK_USES_BURST_INDEX_HISTORY: u8 = 1u8 << 3;
    const BITMASK_MEMBRANE_POTENTIAL_DRIVEN_PSP: u8 = 1u8 << 4;
    const BITMASK_PSP_UNIFORMITY: u8 = 1u8 << 5;
    const BITMASK_CORTICAL_AREA_ALIVE: u8 = 1u8 << 6;

    pub const fn new(layout: CorticalAreaLayoutType, uses_burst_index_history: bool, membrane_potential_driven_psp: bool, psp_uniform: bool, cortical_area_alive: bool) -> Self
    {
        let mut out = layout as u8;
        if uses_burst_index_history {out & Self::BITMASK_USES_BURST_INDEX_HISTORY;}
        if membrane_potential_driven_psp {out & Self::BITMASK_MEMBRANE_POTENTIAL_DRIVEN_PSP;}
        if psp_uniform {out & Self::BITMASK_PSP_UNIFORMITY;}
        if cortical_area_alive {out & Self::BITMASK_CORTICAL_AREA_ALIVE;}
        NeuronModelCorticalDescriptorsCPU(out)
    }
    
    /*
    pub fn get_cortical_area_layout(&self) -> CorticalAreaLayoutType
    {
        self.into()
    }
    
     */

    pub fn get_uses_burst_index_history(&self) -> bool
    {
        self.0 & Self::BITMASK_USES_BURST_INDEX_HISTORY != 0
    }
    
    pub fn get_membrane_driven_psp(self) -> bool
    {
        self.0 & Self::BITMASK_MEMBRANE_POTENTIAL_DRIVEN_PSP != 0
    }

    pub fn get_psp_uniformity(self) -> bool
    {
        self.0 & Self::BITMASK_PSP_UNIFORMITY != 0
    }

    pub fn get_cortical_area_alive(self) -> bool
    {
        self.0 & Self::BITMASK_CORTICAL_AREA_ALIVE != 0
    }


}



