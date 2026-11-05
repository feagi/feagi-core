use serde::{Deserialize, Serialize};
use feagi_basis::collections::bit_packed_bool::par_data::BitBatchParDataVector;
use feagi_basis::create_wrapped_quantized_unsigned_integer;
use feagi_basis::parallelism::feagi_channel::recycling::Recyclable;
use crate::quantization_levels::FeagiIndexQuantization;

pub const VISUALIZATION_VERSION: u8 = 0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstVisualizationData<FIQ: FeagiIndexQuantization> {
    pub version: u8,
    pub change_counter: VisualizationStructureChangeCounter,
    pub bit_data: BitBatchParDataVector<
        NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>,
        NeuronVisualizationVoxelIndex<FIQ::NeuronIndexQuant>,
        VoxelBitPacked
    >
}

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
impl<FIQ: FeagiIndexQuantization> Recyclable for BurstVisualizationData<FIQ> {
    fn new_for_recycler() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VisualizationStructureChangeCounter(pub u16);




pub type VoxelBitPacked = u8;

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of the byte index on the neuron visualization array
    pub NeuronVisualizationWordIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of the bit index on the neuron visualization array
    pub NeuronVisualizationVoxelIndex
);