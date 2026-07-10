use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Describes various metadata of a cortical area in a single byte.
#[derive(Clone, Copy)]
pub struct CorticalAreaDescriptors(u8);
// bit 0 -> is cortical area enabled
// bit 1 -> is psp uniformity enabled
// bit 2 -> is mp driven psp enabled
// bit 3 -> unused
// bit 4 -> unused
// bits 5,6,7 -> cortical area layout
impl CorticalAreaDescriptors {
    const BITMASK_CORTICAL_AREA_ENABLED: u8 = 1 << 0;
    const BITMASK_PSP_UNIFORMITY: u8 = 1 << 1;
    const BITMASK_MP_DRIVEN_PSP: u8 = 1 << 2;

    pub const fn new(
        cortical_area_layout_type: CorticalAreaLayoutType,
        membrane_potential_driven_psp: bool,
        psp_uniform: bool,
        cortical_area_enabled: bool,
    ) -> Self {
        let mut out = cortical_area_layout_type as u8;
        if membrane_potential_driven_psp {
            out &= Self::BITMASK_MP_DRIVEN_PSP;
        }
        if psp_uniform {
            out &= Self::BITMASK_PSP_UNIFORMITY;
        }
        if cortical_area_enabled {
            out &= Self::BITMASK_CORTICAL_AREA_ENABLED;
        }
        CorticalAreaDescriptors(out)
    }

    pub fn get_membrane_driven_psp(self) -> bool {
        self.0 & Self::BITMASK_MP_DRIVEN_PSP != 0
    }

    pub fn get_psp_uniformity(self) -> bool {
        self.0 & Self::BITMASK_PSP_UNIFORMITY != 0
    }

    pub fn get_cortical_area_enabled(self) -> bool {
        self.0 & Self::BITMASK_CORTICAL_AREA_ENABLED != 0
    }

    pub fn get_cortical_area_layout(self) -> CorticalAreaLayoutType {
        let x = self.0 & CorticalAreaLayoutType::BITMASK;
        match x {
            (CorticalAreaLayoutType::BITS_DIMENSIONAL) => { CorticalAreaLayoutType::Dimensional }
            (CorticalAreaLayoutType::BITS_MEMORY) => { CorticalAreaLayoutType::Memory }
            _ => { panic!("Unexpected CorticalAreaLayoutType") } // TODO better error handling lol
        }
    }
}


/// Represents what type of cortical area layout is being used in a cortical_area area, within 3 bits
/// (limiting to only 8 options). Describes how the neurons of a cortical area area are
/// laid out and any other specific cortical area parameters for that layout
#[repr(u8)]
#[derive(Copy, Clone, Default)]
pub enum CorticalAreaLayoutType {
    #[default]
    Dimensional,
    Memory,
}

impl CorticalAreaLayoutType {
    pub const BITMASK: u8 = 0b1110_0000;
    pub const BITS_DIMENSIONAL: u8 = 0b0000_0000;
    pub const BITS_MEMORY: u8 = 0b0010_0000;
}



/// Describes a cortical area of neurons arranged in voxels with depth as 4d XYZD dimensions
#[derive(Copy, Clone)]
pub struct CorticalAreaLayoutDataDimensional<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    pub dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
}

/// Describes a cortical area of neurons arranged for memory formation
#[derive(Copy, Clone)]
pub struct CorticalAreaLayoutDataMemory<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    pub num_neurons: FIQ::NeuronIndexCountQuant, // TODO
}
