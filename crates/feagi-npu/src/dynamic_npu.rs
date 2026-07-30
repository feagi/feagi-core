use feagi_data::quantization_levels::feagi_index_quantization::{FeagiGlobalQuantizationAbsurd, FeagiGlobalQuantizationStandard, FeagiIndexQuantization};
use crate::editable::genome_engine_map::{GenomeEngineMapSingleEngine};

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
