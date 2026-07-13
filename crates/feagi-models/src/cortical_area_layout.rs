use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Represents what type of cortical area layout is being used in a cortical area, within 3 bits
/// (limiting to only 8 options). Describes how the neurons of a cortical area are
/// laid out and any other specific cortical area parameters for that layout
#[repr(u8)]
#[derive(Copy, Clone, Default)]
pub enum CorticalAreaLayoutType {
    #[default]
    Dimensional = 0b0000_0000,
    Memory = 0b0000_0001,
}

impl CorticalAreaLayoutType {
    /// Bitmask for this enum, as it has a packed descriptor
    pub const BITMASK: u8 = 0b0000_0111;

    /// given a byte value from 0-7, will output the cortical area layout type. If out of the
    /// range or the byte not valid, will cause undefined behavior
    pub unsafe fn from_three_bits(bits: u8) -> CorticalAreaLayoutType {
        core::mem::transmute(bits)
    }
}

/// Describes how a cortical area neurons are laid out.
pub trait CorticalAreaLayoutData<FIQ: FeagiIndexQuantization> {}

/// Describes a cortical area of neurons arranged in voxels with depth as 4d XYZD dimensions
#[derive(Copy, Clone)]
pub struct CorticalAreaLayoutDataDimensional<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    pub dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayoutData<FIQ> for CorticalAreaLayoutDataDimensional<FIQ> {}

/// Describes a cortical area of neurons arranged for memory formation
#[derive(Copy, Clone)]
pub struct CorticalAreaLayoutDataMemory<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    pub num_neurons: FIQ::NeuronIndexCountQuant, // TODO
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayoutData<FIQ> for CorticalAreaLayoutDataMemory<FIQ> {}
