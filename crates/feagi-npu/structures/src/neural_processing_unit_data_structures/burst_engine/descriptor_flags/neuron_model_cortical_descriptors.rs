use crate::neural_processing_unit_data_structures::burst_engine::descriptor_flags::cortical_area_layout::CorticalAreaLayoutType;

/// Used to hold the enums denoting the cortical layout
pub trait NeuronModelCorticalDescriptors {
    // bits 0, 1, 2 are for cortical layout
    // bits 3, 4 are unused
    // bit 5 is if post synaptic potential is driven by membrane potential
    // bit 6 is if post synaptic potential should be uniform
    // bit 7 is if cortical area is alive
}


#[repr(C)]
pub struct NeuronModelCorticalDescriptorsCPU(u8);

impl NeuronModelCorticalDescriptorsCPU {

    const BITMASK_MEMBRANE_POTENTIAL_DRIVEN_PSP: u8 = 1u8 << 4;
    const BITMASK_PSP_UNIFORMITY: u8 = 1u8 << 5;
    const BITMASK_CORTICAL_AREA_ALIVE: u8 = 1u8 << 6;

    pub const fn new(layout: CorticalAreaLayoutType, membrane_potentiaL_driven_psp: bool, psp_uniform: bool, cortical_area_alive: bool) -> Self
    {
        let mut out = layout as u8;
        if membrane_potentiaL_driven_psp {out & Self::BITMASK_MEMBRANE_POTENTIAL_DRIVEN_PSP;}
        if psp_uniform {out & Self::BITMASK_PSP_UNIFORMITY;}
        if cortical_area_alive {out & Self::BITMASK_CORTICAL_AREA_ALIVE;}
        NeuronModelCorticalDescriptorsCPU(out)
    }
    
    pub fn get_cortical_area_layout(&self) -> CorticalAreaLayoutType
    {
        self.into()
    }

    pub fn get_membrane_driven_psp(self) -> bool
    {
        self.0 & Self::BITMASK_MEMBRANE_POTENTIAL_DRIVEN_PSP == Self::BITMASK_MEMBRANE_POTENTIAL_DRIVEN_PSP
    }

    pub fn get_psp_uniformity(self) -> bool
    {
        self.0 & Self::BITMASK_PSP_UNIFORMITY == Self::BITMASK_PSP_UNIFORMITY
    }

    pub fn get_cortical_area_alive(self) -> bool
    {
        self.0 & Self::BITMASK_CORTICAL_AREA_ALIVE == Self::BITMASK_CORTICAL_AREA_ALIVE
    }





}








