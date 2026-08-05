use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::engines::rayon::data::synapse::model_quantized_data::uniform_model::ModelUniform;

macro_rules! quant_default_without_synapses {
    ($struct_name:ident) => {
    impl<FIQ: FeagiIndexQuantization> Default for $struct_name<FIQ> {
        fn default() -> Self {
            Self {
                mapping_entry_data: MappingEntryModelIndexedVector::new_empty(),
            }
        }
    }
    };
}


pub struct SynapseModelData<FIQ: FeagiIndexQuantization> {
    pub uniform: ModelUniform<FIQ>,
    // TODO this should be macroized and expanded!
}


impl<FIQ: FeagiIndexQuantization> Default for SynapseModelData<FIQ> {
    fn default() -> Self {
        Self {
            uniform: Default::default(),
        }
    }
}




#[doc(hidden)]
mod uniform_model {
    use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
    use feagi_models::cortical_mapping_entry::synapse_model::models::uniform::{UniformSynapseModelCorticalMappingEntryData, UniformSynapseModelStandardQuant};
    use feagi_models::wrapped_index_collections::MappingEntryModelIndexedVector;

    #[doc(hidden)]
    pub struct ModelUniform<FIQ: FeagiIndexQuantization> {
        pub quantization_standard: QuantizationStandard<FIQ>
    }

    impl<FIQ: FeagiIndexQuantization> Default for ModelUniform<FIQ> {
        fn default() -> Self {
            Self {
                quantization_standard: Default::default()
            }
        }
    }



    #[doc(hidden)]
    pub struct QuantizationStandard<FIQ: FeagiIndexQuantization> {
        pub mapping_entry_data: MappingEntryModelIndexedVector<FIQ::CorticalMappingEntryIndexCountQuant, UniformSynapseModelCorticalMappingEntryData<UniformSynapseModelStandardQuant>>,
    }

    quant_default_without_synapses!(QuantizationStandard);

}