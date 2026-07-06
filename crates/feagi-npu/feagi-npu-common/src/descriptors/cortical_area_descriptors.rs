use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use crate::wrapped_values::DimensionalCorticalAreaDimensions;

#[repr(C)]
pub struct CorticalAreaDescriptor(u8);
// bits 0, 1, 2 are for cortical_area layout
// bit 3 is unused,
// bit 4 is if the neuron model uses burst index history
// bit 5 is if post synaptic potential is driven by membrane potential
// bit 6 is if post synaptic potential should be uniform
// bit 7 is if cortical_area area is alive


impl CorticalAreaDescriptor {

    const BITMASK_USES_BURST_INDEX_HISTORY: u8 = 0b0000_1000;
    const BITMASK_MEMBRANE_POTENTIAL_DRIVEN_PSP: u8 = 0b0000_0100;
    const BITMASK_PSP_UNIFORMITY: u8 = 0b0000_0010;
    const BITMASK_CORTICAL_AREA_ALIVE: u8 = 0b0000_0001;

    pub const fn new(layout: CorticalAreaLayoutType, uses_burst_index_history: bool, membrane_potential_driven_psp: bool, psp_uniform: bool, cortical_area_alive: bool) -> Self
    {
        let mut out = layout as u8;
        if uses_burst_index_history {out & Self::BITMASK_USES_BURST_INDEX_HISTORY;}
        if membrane_potential_driven_psp {out & Self::BITMASK_MEMBRANE_POTENTIAL_DRIVEN_PSP;}
        if psp_uniform {out & Self::BITMASK_PSP_UNIFORMITY;}
        if cortical_area_alive {out & Self::BITMASK_CORTICAL_AREA_ALIVE;}
        CorticalAreaDescriptor(out)
    }

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



//region Cortical Layout

/// Represents what type of cortical_area layout is being used in a cortical_area area, within 3 bits
/// (limiting to only 8 options)
#[repr(u8)]
#[derive(Copy, Clone, Default)]
pub enum CorticalAreaLayoutType {
    #[default]
    Dimensional = 0b0000_0000,
    Memory = 0b1000_0000,
}


/// Base trait for Cortical Area Layouts, which describes how the neurons of a cortical_area area are
/// laid out and any other specific cortical_area parameters for that layout
pub trait CorticalAreaLayoutDataBase<FGQ>:
where
    FGQ: FeagiGlobalQuantization,
{
    // Does NOT contain the base post synaptic potential! We dont want to deal with CBQ quantization!

    // Only contain usable data if on the CPU

    // Number of neurons contained should be accessible

    // NOTE: Do NOT use the base directly, use the one of the derived types (and the device)
}

/// Describes a cortical_area area of neurons arranged in voxels with depth as 4d XYZD dimensions
pub struct CorticalAreaLayoutDataDimensional<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{
    pub dimensions: DimensionalCorticalAreaDimensions<FGQ::NeuronIndexCountQuant>,
}

//endregion