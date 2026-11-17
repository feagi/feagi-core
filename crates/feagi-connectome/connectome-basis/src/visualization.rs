use serde::{Deserialize, Serialize};
#[cfg(feature = "alloc")]
use feagi_basis::collections::bit_packed_bool::par_data::BitBatchParDataVector;
use feagi_basis::create_wrapped_quantized_unsigned_integer;
#[cfg(feature = "alloc")]
use feagi_basis::parallelism::feagi_channel::recycling::Recyclable;
use crate::quantization_levels::FeagiIndexQuantization;

pub const VISUALIZATION_VERSION: u8 = 0;

/// Backed by a heap allocated `BitBatchParDataVector`, so requires alloc
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstVisualizationData<FIQ: FeagiIndexQuantization> {
    pub version: u8,
    pub change_counter: VisualizationStructureChangeCounter,
    pub bit_data: BitBatchParDataVector<
        NeuronVisualizationVoxelByteIndex<FIQ::NeuronIndexQuant>,
        NeuronVisualizationVoxelBitIndex<FIQ::NeuronIndexQuant>,
        VoxelBitPacked
    >
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> BurstVisualizationData<FIQ> {
    pub fn new() -> Self {
        Self {
            version: VISUALIZATION_VERSION,
            change_counter: VisualizationStructureChangeCounter(0),
            bit_data: BitBatchParDataVector::new_uniform(Default::default(),0),
        }
    }
}

/// Each channel slot keeps its allocation between bursts. Every burst sending visualization
/// overwrites it, so nothing is cleared on recycling
#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> Recyclable for BurstVisualizationData<FIQ> {
    fn new_for_recycler() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VisualizationStructureChangeCounter(pub u16);

/// Provides context for a cortical area to understand how to read visualization data
pub enum CorticalAreaVisualizationContext<FIQ: FeagiIndexQuantization> {
    /// Voxel data, cortical area starts reading at this byte
    DimensionalVoxelIndex(NeuronVisualizationVoxelByteIndex<FIQ::NeuronBitBatchIndexQuant>)
    // TODO memory
}


pub type VoxelBitPacked = u8;

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of the byte index on the neuron visualization array
    pub NeuronVisualizationVoxelByteIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of the bit index on the neuron visualization array
    pub NeuronVisualizationVoxelBitIndex
);