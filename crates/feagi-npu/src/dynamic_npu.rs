use ahash::HashMap;
use feagi_data::neurons::NeuronMembranePotential;
use crate::engines::rayon::rayon_burst_engine::RayonBurstEngine;
use feagi_data::quantization_levels::feagi_index_quantization::{
    FeagiIndexQuantizationAbsurd, FeagiIndexQuantization,
};
use feagi_data::values::quantizable::PercentageUnsigned;
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::connectome_requests::connectome_request::{ConnectomeRequest, ConnectomeRequestType};
use feagi_models::connectome_requests::connectome_request_builder::ConnectomeRequestBuilder;
use feagi_models::connectome_requests::properties::UniversalCorticalAreaProperties;
use feagi_models::neuron::models::feagi_advanced::{ConsecutiveFireLimit, DegeneracyConstant, FeagiAdvancedModel, FeagiAdvancedModelCorticalData, FeagiAdvancedModelStandardQuant, RefractoryPeriodLimit, SnoozePeriod};
use feagi_models::wrapped_index_collections::CorticalEngineIndex;
use crate::engines_common::EditableEngine::EditableEngine;

pub struct DynamicNPU {
    // TODO support multi burst engine setups
    rayon_burst_engine: RayonBurstEngine<FeagiIndexQuantizationAbsurd>,
    cortical_id_engine_mapping: HashMap<CorticalID, CorticalEngineIndex<u64>>,
}

impl DynamicNPU {
    pub fn new() -> Self {
        Self {
            rayon_burst_engine: RayonBurstEngine::<FeagiIndexQuantizationAbsurd>::new(),
            cortical_id_engine_mapping: Default::default(),
        }
    }
    
    pub fn request_builder(&self) -> ConnectomeRequestBuilder
    {
        ConnectomeRequestBuilder::new(FeagiIndexQuantizationAbsurd::QUANTIZATION_LEVEL)
    }
    
    pub fn request(&mut self, request: ConnectomeRequest) {
        
        // TODO check indexing quant
        
        match request.request_type {
            ConnectomeRequestType::CorticalAreaAddDimensional(cortical_id, dims) => {
                let cortical_id = CorticalID::try_from_u64(cortical_id).unwrap();
                let number_neurons = dims.number_contained_elements().deref();
                let cortical_props = UniversalCorticalAreaProperties {
                    non_mp_psp: 0.0,
                    probe_cortical_area_input_disabled: false,
                    probe_cortical_area_output_disabled: false,
                    is_psp_uniform: false,
                    is_psp_mp_driven: false,
                };
                // TODO derive cortical data from the request/genome instead of hardcoding "feagi advanced, standard quant"
                let cortical_data: FeagiAdvancedModelCorticalData<FeagiAdvancedModelStandardQuant> = FeagiAdvancedModelCorticalData::new(
                    PercentageUnsigned::HUNDRED_PERCENT,
                    RefractoryPeriodLimit::QUANT_ONE,
                    NeuronMembranePotential::QUANT_ONE,
                    ConsecutiveFireLimit::QUANT_ONE,
                    SnoozePeriod::QUANT_ONE,
                    DegeneracyConstant::QUANT_ONE
                );

                let cortical_engine_index = self.rayon_burst_engine.add_cortical_area::<
                    FeagiAdvancedModelStandardQuant,
                    FeagiAdvancedModel<FeagiIndexQuantizationAbsurd, FeagiAdvancedModelStandardQuant>,
                >(number_neurons, cortical_props, cortical_data, ());
                self.cortical_id_engine_mapping.insert(cortical_id, cortical_engine_index);
            }
            ConnectomeRequestType::CorticalAreaAddFormless(_) => {}
            ConnectomeRequestType::CorticalAreaDelete(_) => {}
            ConnectomeRequestType::MappingEntryAdd(source_id, destination_id) => {
                
            }
        }
        
    }

    pub fn execute_single_burst(&mut self) {
        self.rayon_burst_engine.execute_single_burst();
    }
    
    
}
