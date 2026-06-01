use feagi_structures::feagi_data::shared_quantization_sets::{NeuronModelQuantization, FeagiGlobalQuantization};



pub struct NPUDimensionalCorticalAreaCPU<FGIQ, CAMQ>
where
    FGIQ: FeagiGlobalQuantization,
    CAMQ: NeuronModelQuantization,
{
    
    dimensional_neurons
}