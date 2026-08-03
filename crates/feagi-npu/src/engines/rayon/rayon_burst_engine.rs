use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::connectome_requests::properties::UniversalCorticalAreaProperties;
use feagi_models::neuron::neuron_model::NeuronModel;
use feagi_models::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, MappingEntryEngineIndex};
use feagi_models::wrapped_indexes::BurstIndex;
use crate::engines::rayon::data::RayonEngineData;
use crate::engines::rayon::kernels_neurons;
use crate::engines::rayon::kernels_synapses;
use crate::engines_common::EditableEngine::EditableEngine;

pub struct RayonBurstEngine<FIQ: FeagiIndexQuantization> {
    data: RayonEngineData<FIQ>,
    // dyn stuff

    latest_cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    latest_mapping_entry_index: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>,


}

impl<FIQ: FeagiIndexQuantization> RayonBurstEngine<FIQ> {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            latest_cortical_index: CorticalEngineIndex::QUANT_ZERO,
            latest_mapping_entry_index: MappingEntryEngineIndex::QUANT_ZERO,
        }
    }

    pub fn set_sensor_data(&mut self, data: ()) {
        todo!()
    }

    pub fn get_motor_data(&self) {
        todo!()
    }

    pub fn get_visualization_data(&self) {
        todo!()
    }
    
    pub fn execute_single_burst(&mut self) {
        kernels_neurons::process_neurons(&self.data);
        kernels_synapses::process_synapses(&self.data);
        self.data.burst_index += BurstIndex::QUANT_ONE;
    }
}

impl<FIQ: FeagiIndexQuantization> EditableEngine<FIQ> for RayonBurstEngine<FIQ> {
    fn add_cortical_area<NMQ: NeuronModelQuantization, NM: NeuronModel<FIQ, NMQ>>(&mut self, number_neurons: FIQ::NeuronIndexQuant, cortical_flags: UniversalCorticalAreaProperties , cortical_data: NM::CorticalData, neuron_data_writer: ()) -> CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant> {




        let returning = self.latest_cortical_index.clone();
        self.latest_cortical_index += CorticalEngineIndex::QUANT_ONE;
        returning
    }

    fn edit_cortical_area_cortical_flags(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, cortical_flags: UniversalCorticalAreaProperties) {
        todo!()
    }

    fn edit_cortical_area_cortical_data<NMQ: NeuronModelQuantization, NM: NeuronModel<FIQ, NMQ>>(&mut self, new_cortical_data: NM::CorticalData) {
        todo!()
    }

    fn remove_cortical_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>) {
        todo!()
    }

    fn resize_dimensional_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>) {
        todo!()
    }

    fn add_mapping_entry(&mut self) -> MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant> {
        todo!()
    }

    fn remap_mapping_entry(&mut self, mapping_entry: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>) {
        todo!()
    }

    fn remove_mapping_entry(&mut self, mapping_entry: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>) {
        todo!()
    }

    /*
    fn probe_cortical_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, _: flags) {
        todo!()
    }

    fn probe_neurons(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, _: iterator) {
        todo!()
    }

    fn probe_mapping_entries(&mut self) {
        todo!()
    }

     */


}
