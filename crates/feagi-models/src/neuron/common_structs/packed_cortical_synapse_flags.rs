


/// Per cortical area flags that are used during the synapse burst phase of a burst.
/// bit 0 - IsCorticalAreaFrozenOutput -> Cortical area will not output any firing or vizualiztion, appear off / disabled
/// bit 1 - IsPSPUniform -> Post Synaptic Potential is uniform across all outputs
/// bit 2 - IsPSPMembraneDriven -> PSP comes from the neuron, not a property of the cortical area
/// bit 3 - unused
/// bit 4-7 - STATIC -> Quantization of membrane potential
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PackedCorticalSynapseFlags(u8);

impl PackedCorticalSynapseFlags {
    const BITMASK_CORTICAL_AREA_FROZEN: u8 = 1 << 7;
    const BITMASK_PSP_UNIFORMITY: u8 = 1 << 6;
    const BITMASK_MP_DRIVEN_PSP: u8 = 1 << 5;

    pub fn get_membrane_driven_psp(self) -> bool {
        self.0 & Self::BITMASK_MP_DRIVEN_PSP != 0
    }

    pub fn get_psp_uniformity(self) -> bool {
        self.0 & Self::BITMASK_PSP_UNIFORMITY != 0
    }

    pub fn get_cortical_area_frozen(self) -> bool {
        self.0 & Self::BITMASK_CORTICAL_AREA_FROZEN != 0
    }
}
