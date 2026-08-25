use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

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
        self.get_bit(Self::BITMASK_MP_DRIVEN_PSP)
    }

    pub fn get_psp_uniformity(self) -> bool {
        self.get_bit(Self::BITMASK_PSP_UNIFORMITY)
    }

    pub fn get_cortical_area_frozen_output(self) -> bool {
        self.get_bit(Self::BITMASK_CORTICAL_AREA_FROZEN_OUTPUT)
    }

    pub fn get_cortical_area_frozen_input(self) -> bool {
        self.get_bit(Self::BITMASK_CORTICAL_AREA_FROZEN_INPUT)
    }

    /// Packs the genome-level properties of a cortical area into the runtime flags the burst
    /// kernels read
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

    #[inline(always)]
    fn set_bit(&mut self, bitmask: u8, value: bool) {
        self.0 = (self.0 & !bitmask) | (bitmask * value as u8);
    }

    #[inline(always)]
    fn get_bit(&self, bitmask: u8) -> bool {
        (self.0 & bitmask) != 0
    }
}
