use ahash::HashMap;
use feagi_data::collections::BiDirectionHashmap;
use crate::engines::rayon::rayon_burst_engine::RayonBurstEngine;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::connectome_requests::connectome_request::{ConnectomeRequest};
use feagi_models::cortical_area::genome_compose::cortical_writer_by_model_quant::{CorticalWriterByModelQuant, FeagiAdvancedModelWriter};
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::model::FeagiAdvancedModel;
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use feagi_models::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer_by_model_quant::{CorticalMappingEntryWriterByModelQuant, UniformWriter};
use feagi_models::wrapped_index_collections::CorticalEngineIndex;
use crate::engines_common::EditableEngine::EditableEngine;

// TODO Genomic level quantization isnt really meant to be used, but we will use it here for now

type CorticalQuant = <FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::CorticalAreaIndexCountQuant;
type NeuronQuant = <FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant;

pub struct DynamicNPU {
    // TODO support multi burst engine setups
    rayon_burst_engine: RayonBurstEngine<FeagiIndexQuantizationGenomic>,
    cortical_id_engine_mapping: BiDirectionHashmap<CorticalID, CorticalEngineIndex<CorticalQuant>>,
}

impl DynamicNPU {
    pub fn new() -> Self {
        Self {
            rayon_burst_engine: RayonBurstEngine::<FeagiIndexQuantizationGenomic>::new(),
            cortical_id_engine_mapping: BiDirectionHashmap::new(),
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
                                let index = self.rayon_burst_engine.add_cortical_area::<
                                    FeagiAdvancedModel<FeagiIndexQuantizationGenomic, FeagiAdvancedModelStandardQuant>,
                                >(writer);
                                self.cortical_id_engine_mapping.insert(TEMP_adding_id, index);
                            }
                        }
                    }
                }
            },
            ConnectomeRequest::CorticalMappingEntryAdd { source_id, destination_id, mapping_writer } => {
                let source_index = self.cortical_id_engine_mapping.get_forward(&source_id).unwrap();
                let destination_index = self.cortical_id_engine_mapping.get_forward(&destination_id).unwrap();

                match mapping_writer {
                    CorticalMappingEntryWriterByModelQuant::Uniform(quant) => {
                        match quant {
                            UniformWriter::Standard(writer) => {
                                //self.rayon_burst_engine.add_mapping_entry();
                            }
                        }


                    }
                }
            }
        }



        
    }

    pub fn execute_single_burst(&mut self) {
        self.rayon_burst_engine.execute_single_burst();
    }
    
    
}
