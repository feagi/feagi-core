//! This structure is essentially the first one read in a burst. It points

use feagi_npu_neuron_models::NeuronModelTypeAndQuantizationFlat;
use feagi_structures::feagi_data::feagi_pdi::{PDICollection, PDIElement};
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::wrapped_indexing::{NPUCorticalAreaIndexModelQuantizationLocal, NPUNeuronChunkIndexGlobal};

/// Holds references to cortical area indexes, multiple times per chunk count of each area. This
/// Helps in paralleling neuron processing
pub trait GlobalNeuronChunkIndexTable<FGQ: FeagiGlobalQuantization>:
PDICollection
+ PDITagGenericDevice
{

    fn get_total_number_neuron_chunks(&self) -> NPUNeuronChunkIndexGlobal<FGQ>;
}

pub trait GlobalNeuronChunkIndexElement<FGQ: FeagiGlobalQuantization>:
PDIElement
+ PDITagGenericDevice
{}



//region CPU implementation

#[repr(C)]
pub struct GlobalNeuronChunkIndexTableCPU<FGQ: FeagiGlobalQuantization>{
    pub neuron_index_chunks: Vec<GlobalNeuronIndexChunkElementCPU<FGQ>>
}

impl<FGQ: FeagiGlobalQuantization> PDICollection for GlobalNeuronChunkIndexTableCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> PDITagGenericDevice for GlobalNeuronChunkIndexTableCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> PDITagCPU for GlobalNeuronChunkIndexTableCPU<FGQ> {}


impl<FGQ: FeagiGlobalQuantization> GlobalNeuronChunkIndexTable<FGQ> for GlobalNeuronChunkIndexTableCPU<FGQ> {
    fn get_total_number_neuron_chunks(&self) -> NPUNeuronChunkIndexGlobal<FGQ> {
        NPUNeuronChunkIndexGlobal::wrap(self.neuron_index_chunks.len())
    }
}


impl<FGQ: FeagiGlobalQuantization> GlobalNeuronChunkIndexTableCPU<FGQ> {
    pub fn new() -> Self {
        Self {
            neuron_index_chunks: vec![]
        }
    }
}

// TODO we will likely need different variants depending on quantization settings

#[repr(C)]
pub(crate) struct GlobalNeuronIndexChunkElementCPU<FGQ: FeagiGlobalQuantization>{
    pub neuron_cortical_area_index: NPUCorticalAreaIndexModelQuantizationLocal<FGQ>,
    pub neuron_cortical_area_type: NeuronModelTypeAndQuantizationFlat, // u8
    /// Most of the time, chunks are full and this will equal the number of neurons per chunk, but
    /// at the end of cortical areas, there will likely be one chunk that isnt full and thus
    /// this value will be smaller to avoid exceeding the boundaries of neuron storage vectors
    pub number_neurons_in_chunk: u8,
    _padding: [u8; 2],
}



//endregion