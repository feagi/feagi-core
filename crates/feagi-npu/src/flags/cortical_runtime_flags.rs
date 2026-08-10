use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_models::cortical_area::neuron::cortical_area_properties::{CorticalAreaProperties, PostCorticalPotential};

/// Per cortical area flags
/// bit 0 - IsCorticalAreaFrozenInput -> Cortical area will not respond to input, never running neuron dynamics and thus appearing in a frozen state
/// bit 1 - IsCorticalAreaFrozenOutput -> Cortical area will not output any firing or vizualiztion, appear off / disabled
/// bit 2 - IsPSPUniform -> Post Synaptic Potential is uniform across all outputs
/// bit 3 - IsPSPMembraneDriven -> PSP comes from the neuron, not a property of the cortical area
/// bit 4-7 - unused
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CorticalRuntimeFlags(u8);

impl CorticalRuntimeFlags {
    const BITMASK_CORTICAL_AREA_FROZEN_INPUT: u8 = 1 << 7;
    const BITMASK_CORTICAL_AREA_FROZEN_OUTPUT: u8 = 1 << 6;
    const BITMASK_PSP_UNIFORMITY: u8 = 1 << 5;
    const BITMASK_MP_DRIVEN_PSP: u8 = 1 << 4;

    pub fn get_membrane_driven_psp(self) -> bool {
        self.0 & Self::BITMASK_MP_DRIVEN_PSP != 0
    }

    pub fn get_psp_uniformity(self) -> bool {
        self.0 & Self::BITMASK_PSP_UNIFORMITY != 0
    }

    pub fn get_cortical_area_frozen_output(self) -> bool {
        self.0 & Self::BITMASK_CORTICAL_AREA_FROZEN_OUTPUT != 0
    }

    pub fn get_cortical_area_frozen_input(self) -> bool {
        self.0 & Self::BITMASK_CORTICAL_AREA_FROZEN_INPUT != 0
    }

    /// Packs the genome-level properties of a cortical area into the runtime flags the burst
    /// kernels read.
    ///
    /// `post_cortical_potential` decides membrane-driven PSP: `MembraneDriven` means the
    /// potential comes from the neuron, `Uniform` means it is a property of the cortical area.
    pub fn from_cortical_area_properties<MPQ: MembranePotentialQuantization>(properties: &CorticalAreaProperties<MPQ>) -> Self {
        let mut flags = 0u8;

        if properties.probe_cortical_area_input_disabled {
            flags |= Self::BITMASK_CORTICAL_AREA_FROZEN_INPUT;
        }
        if properties.probe_cortical_area_output_disabled {
            flags |= Self::BITMASK_CORTICAL_AREA_FROZEN_OUTPUT;
        }
        if properties.is_psp_uniform {
            flags |= Self::BITMASK_PSP_UNIFORMITY;
        }
        if matches!(properties.post_cortical_potential, PostCorticalPotential::MembraneDriven) {
            flags |= Self::BITMASK_MP_DRIVEN_PSP;
        }

        CorticalRuntimeFlags(flags)
    }
}
