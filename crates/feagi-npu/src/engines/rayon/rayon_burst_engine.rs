use crate::engines::rayon::data::RayonEngineData;
use crate::engines::rayon::kernels_neurons;
use crate::engines::rayon::kernels_synapses;
use crate::engines_common::EditableEngine::EditableEngine;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;
use feagi_models::cortical_area::cortical_writer::NeuronModelCorticalWriter;
use feagi_models::cortical_area::implementations::feagi_advanced::data::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData};
use feagi_models::cortical_area::implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use feagi_models::cortical_area::neuron::neuron_model::neuron_model::NeuronModel;
use feagi_models::cortical_area::neuron::neuron_model::quantization::NeuronModelQuantization;
use feagi_models::cortical_area::neuron::NeuronProperties;
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, MappingEntryEngineIndex, NeuronEngineIndexedVector};
use feagi_models::wrapped_indexes::BurstIndex;
use crate::flags::neuron_runtime_flags::NeuronRuntimeFlags;

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

    pub fn get_visualization_data(&self) -> &NeuronEngineIndexedVector<FIQ::NeuronIndexQuant, NeuronRuntimeFlags> {
        &self.data.neuron_runtime_flags
    }

    pub fn execute_single_burst(&mut self) {
        kernels_neurons::process_neurons(&self.data);
        kernels_synapses::process_synapses(&self.data);
        self.data.burst_index += BurstIndex::QUANT_ONE;
    }
}

impl<FIQ: FeagiIndexQuantization> EditableEngine<FIQ> for RayonBurstEngine<FIQ> {
    fn add_cortical_area<NM>(
        &mut self,
        neuron_data_writer: impl NeuronModelCorticalWriter<FeagiAdvancedModelStandardQuant, NM::CorticalData, NM::NeuronData>,
    ) -> CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>
    where
        NM: NeuronModel<
            FIQ,
            FeagiAdvancedModelStandardQuant,
            CorticalData = FeagiAdvancedModelCorticalData<FeagiAdvancedModelStandardQuant>,
            NeuronData = FeagiAdvancedModelNeuronData<FeagiAdvancedModelStandardQuant>,
        >,
    {
        let number_neurons: FIQ::NeuronIndexQuant = neuron_data_writer.number_neurons_needed::<FIQ>().unwrap(); // TODO ERROR CHECKING
        let mut neuron_properties = vec![NeuronProperties::default(); number_neurons.quant_to_usize()];

        // TODO for now fixate ona specific quantization
        let cortical_data = self.data.neuron_model_data.feagi_advanced.quantization_standard.cortical_data.append_single_mut(Default::default());
        let neuron_data = self.data.neuron_model_data.feagi_advanced.quantization_standard.neuron_data.extend_mut(number_neurons.into(), Default::default());

        let (layout, cortical_properties) = neuron_data_writer
            .write_to_cortical_area::<FIQ>(cortical_data, neuron_data, neuron_properties.as_mut_slice())
            .unwrap(); // TODO ERROR HANDLING
        let ret = self.latest_cortical_index.clone();
        self.latest_cortical_index += CorticalEngineIndex::QUANT_ONE;
        ret
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
