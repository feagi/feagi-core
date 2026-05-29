use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalIndexQuantization;
use feagi_structures::neuron_descriptors::NeuronDimension;


pub trait NonPlasticDimensionalMorphologyMapper<FGIQ: FeagiGlobalIndexQuantization, SD> {

    fn generate_synapse_iterator(
        &self,
        source_area_dimensions: &NeuronDimension<FGIQ::NeuronIndexCountQuant>,
        destination_area_dimensions: &NeuronDimension<FGIQ::NeuronIndexCountQuant>,
    ) -> (impl Iterator<Item=()>)
    
}

