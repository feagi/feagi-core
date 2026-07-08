use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use crate::wrapped_indexes::DimensionalCorticalAreaDimensions;

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
pub trait CorticalAreaLayoutDataBase<FGQ>
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