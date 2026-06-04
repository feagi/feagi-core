use feagi_structures::feagi_data::quantizable_spatial::index::SpatialIndexDimensions4D;
use feagi_structures::feagi_data::shared_quantization_sets::{NeuronModelQuantization, FeagiGlobalQuantization};



pub struct NPUDimensionalCorticalAreaCPU<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{
    neuron_model_index_offset: FGQ::NeuronIndexCountQuant,
    cortical_area_dimensions: SpatialIndexDimensions4D<FGQ::NeuronIndexCountQuant>,
    
}