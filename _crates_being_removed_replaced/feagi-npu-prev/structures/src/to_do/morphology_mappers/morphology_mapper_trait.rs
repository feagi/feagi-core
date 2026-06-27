use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use feagi_structures::neuron_descriptors::NeuronDimension;


pub trait NonPlasticDimensionalMorphologyMapper<FGIQ: FeagiGlobalQuantization, SD> {

    fn generate_synapse_iterator(
        &self,
        source_area_dimensions: &NeuronDimension<FGIQ::NeuronIndexCountQuant>,
        destination_area_dimensions: &NeuronDimension<FGIQ::NeuronIndexCountQuant>,
    ) -> (impl Iterator<Item=()>)
    
}

