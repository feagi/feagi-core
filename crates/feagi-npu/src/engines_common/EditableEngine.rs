use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::connectome_requests::properties::UniversalCorticalAreaProperties;
use feagi_models::neuron::neuron_model::NeuronModel;
use feagi_models::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, MappingEntryEngineIndex, NeuronEngineIndex};

/// When a burst engine is stopped, these function calls
pub trait EditableEngine<FIQ: FeagiIndexQuantization> {

    fn add_cortical_area<NMQ: NeuronModelQuantization, NM: NeuronModel<FIQ, NMQ>> (&mut self, number_neurons: FIQ::NeuronIndexQuant, cortical_flags: UniversalCorticalAreaProperties , cortical_data: NM::CorticalData, neuron_data_writer: impl NeuronDataWriter) -> CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>;

    fn edit_cortical_area_cortical_flags(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, cortical_flags: UniversalCorticalAreaProperties);

    fn edit_cortical_area_cortical_data<NMQ: NeuronModelQuantization, NM: NeuronModel<FIQ, NMQ>>(&mut self, new_cortical_data: NM::CorticalData);

    fn remove_cortical_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>);

    fn resize_dimensional_area(&mut self,cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>);

    fn add_mapping_entry(&mut self,) -> MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>;

    fn remap_mapping_entry(&mut self, mapping_entry: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>);

    fn remove_mapping_entry(&mut self, mapping_entry: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>);

    /*
    fn probe_cortical_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, flags);

    fn probe_neurons(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, iterator);

    fn probe_mapping_entries(&mut self);
    
     */


    // TODO defragging






}