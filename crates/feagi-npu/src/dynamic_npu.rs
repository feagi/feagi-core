use ahash::HashMap;
use crate::engines::rayon::rayon_burst_engine::RayonBurstEngine;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::connectome_requests::connectome_request::{ConnectomeRequest};
use feagi_models::cortical_area::cortical_writer_by_model_quant::{CorticalWriterByModelQuant, FeagiAdvancedModelWriter};
use feagi_models::cortical_area::implementations::feagi_advanced::model::FeagiAdvancedModel;
use feagi_models::cortical_area::implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use feagi_models::wrapped_index_collections::CorticalEngineIndex;
use crate::engines_common::EditableEngine::EditableEngine;

// TODO Genomic level quantization isnt really meant to be used, but we will use it here for now

type CorticalQuant = <FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::CorticalAreaIndexCountQuant;
type NeuronQuant = <FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant;

pub struct DynamicNPU {
    // TODO support multi burst engine setups
    rayon_burst_engine: RayonBurstEngine<FeagiIndexQuantizationGenomic>,
    cortical_id_engine_mapping: HashMap<CorticalID, CorticalEngineIndex<CorticalQuant>>,
}

impl DynamicNPU {
    pub fn new() -> Self {
        Self {
            rayon_burst_engine: RayonBurstEngine::<FeagiIndexQuantizationGenomic>::new(),
            cortical_id_engine_mapping: Default::default(),
        }
    }
    
    pub fn request(&mut self, request: ConnectomeRequest) {

        // TODO check indexing quant

        match request {
            ConnectomeRequest::CorticalAreaAdd{ TEMP_adding_id, writer } => {
                match writer {
                    
                    CorticalWriterByModelQuant::FeagiAdvanced(quant) => {
                        match quant {
                            FeagiAdvancedModelWriter::Standard(writer) => {
                                self.rayon_burst_engine.add_cortical_area::<
                                    FeagiAdvancedModel<FeagiIndexQuantizationGenomic, FeagiAdvancedModelStandardQuant>,
                                >(writer);
                            }
                        }
                    }
                }
            },
            ConnectomeRequest::CorticalMappingEntryAdd { .. } => panic!("Not supported writer!")
        }



        
    }

    pub fn execute_single_burst(&mut self) {
        self.rayon_burst_engine.execute_single_burst();
    }
    
    
}
