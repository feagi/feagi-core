use crate::engines::rayon::rayon_burst_engine::RayonBurstEngine;
use crate::engines_common::EditableEngine::EditableEngine;
use crate::visualization::CorticalAreaFireSnapshot;
use ahash::HashMap;
use feagi_data::collections::BiDirectionHashmap;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use feagi_data::values::quantizable::{QuantizedIndexCountTrait, WrappedQuantizedIndexCount};
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::connectome_requests::connectome_request::ConnectomeRequest;
use feagi_models::cortical_area::genome_compose::cortical_writer_by_model_quant::{CorticalWriterByModelQuant, FeagiAdvancedModelWriter};
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::model::FeagiAdvancedModel;
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use feagi_models::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer_by_model_quant::{
    CorticalMappingEntryWriterByModelQuant, UniformWriter,
};
use feagi_models::wrapped_index_collections::CorticalEngineIndex;

// TODO Genomic level quantization isnt really meant to be used, but we will use it here for now

type CorticalQuant = <FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::CorticalAreaIndexCountQuant;
type NeuronQuant = <FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant;

pub struct DynamicNPU {
    // TODO support multi burst engine setups
    rayon_burst_engine: RayonBurstEngine<FeagiIndexQuantizationGenomic>,
    cortical_id_engine_mapping: BiDirectionHashmap<CorticalID, CorticalEngineIndex<CorticalQuant>>,
    /// Cortical IDs positioned by their engine index. The engine hands out indexes sequentially
    /// from zero, so position is the index, which makes the reverse lookup a bounds-checked read
    /// instead of a hash. The per-burst visualization path walks this for every area.
    cortical_ids_by_engine_index: Vec<CorticalID>,
}

impl DynamicNPU {
    pub fn new() -> Self {
        Self {
            rayon_burst_engine: RayonBurstEngine::<FeagiIndexQuantizationGenomic>::new(),
            cortical_id_engine_mapping: BiDirectionHashmap::new(),
            cortical_ids_by_engine_index: Vec::new(),
        }
    }

    pub fn request(&mut self, request: ConnectomeRequest) {
        // TODO check indexing quant

        match request {
            ConnectomeRequest::CorticalAreaAdd { TEMP_adding_id, writer } => match writer {
                CorticalWriterByModelQuant::FeagiAdvanced(quant) => match quant {
                    FeagiAdvancedModelWriter::Standard(writer) => {
                        let index = self
                            .rayon_burst_engine
                            .add_cortical_area::<FeagiAdvancedModel<FeagiIndexQuantizationGenomic, FeagiAdvancedModelStandardQuant>>(writer);
                        self.cortical_id_engine_mapping.insert(TEMP_adding_id, index);
                        self.cortical_ids_by_engine_index.push(TEMP_adding_id);
                    }
                },
            },
            ConnectomeRequest::CorticalMappingEntryAdd {
                source_id,
                destination_id,
                mapping_writer,
            } => {
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

    /// The neurons that fired in the most recent burst, keyed by the cortical IDs the connectome
    /// knows rather than the engine indexes it assigns internally.
    ///
    /// Areas with no activity are absent. An engine index with no registered cortical ID is
    /// skipped, since there is nothing a consumer could attribute it to.
    pub fn fire_queue_snapshot(&self) -> Vec<(CorticalID, CorticalAreaFireSnapshot<FeagiIndexQuantizationGenomic>)> {
        self.rayon_burst_engine
            .fire_queue_snapshot()
            .into_iter()
            .filter_map(|snapshot| {
                let slot = snapshot.cortical_index.deref().quant_to_usize();
                self.cortical_ids_by_engine_index.get(slot).map(|cortical_id| (*cortical_id, snapshot))
            })
            .collect()
    }

    /// Every cortical area the NPU currently holds, in engine index order.
    pub fn cortical_areas(&self) -> &[CorticalID] {
        &self.cortical_ids_by_engine_index
    }
}
