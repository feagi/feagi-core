use crate::engines::rayon::data::RayonEngineData;
use crate::engines::rayon::kernels_neurons;
use crate::engines::rayon::kernels_synapses;
use crate::engines_common::EditableEngine::EditableEngine;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::cortical_writer::NeuronModelCorticalWriter;
use feagi_models::neuron::models::feagi_advanced::{FeagiAdvancedModel, FeagiAdvancedModelStandardQuant};
use feagi_models::neuron::neuron_model::NeuronModel;
use feagi_models::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_models::neuron::properties::NeuronProperties;
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, MappingEntryEngineIndex};
use feagi_models::wrapped_indexes::BurstIndex;

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
    fn add_cortical_area<NMQ: NeuronModelQuantization, NM: NeuronModel<FIQ, NMQ>>(
        &mut self,
        neuron_data_writer: impl NeuronModelCorticalWriter<NMQ, NM::CorticalData, NM::NeuronData>,
    ) -> CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant> {
        let number_neurons = neuron_data_writer.number_neurons_needed().unwrap(); // TODO ERROR CHECKING
        let mut neuron_properties = vec![NeuronProperties::default(); number_neurons.quant_to_usize()];

        let (cortical_data, neuron_data) = self.data.neuron_model_data.allocate_for_new_area::<FeagiAdvancedModelStandardQuant, FeagiAdvancedModel<FIQ, FeagiAdvancedModelStandardQuant> >(number_neurons);

        let (layout, cortical_properties) = neuron_data_writer.write_to_cortical_area(cortical_data, neuron_data, neuron_properties.as_mut_slice()).unwrap(); // TODO ERROR HANDLING

        

    }

    fn edit_cortical_area_cortical_flags(
        &mut self,
        cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    ) {
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
