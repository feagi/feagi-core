use feagi_data::quantization_levels::feagi_index_quantization::{FeagiGlobalQuantizationAbsurd, FeagiGlobalQuantizationStandard, FeagiIndexQuantization};
use feagi_models::neuron::common_structs::cortical_area_layout::CorticalAreaLayoutDimensional;
use crate::editable::genome_engine_map::{GenomeEngineMapSingleEngine};
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions};

type StandardNeuronQuantization = <FeagiGlobalQuantizationAbsurd as FeagiIndexQuantization>::NeuronIndexCountQuant; // TODO swap from absurd

pub struct DynamicNPU {
    connectome_allocation_verifier: ConnectomeCacheWrapped,
}

impl DynamicNPU {
    pub fn new() -> Self {
        Self {
            connectome_allocation_verifier: ConnectomeCacheWrapped::StandardSingleEngine(GenomeEngineMapSingleEngine::new()),
        }
    }

}

enum ConnectomeCacheWrapped {
    StandardSingleEngine(GenomeEngineMapSingleEngine<FeagiGlobalQuantizationStandard>),
}
