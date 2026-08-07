use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::cortical_area::cortical_writer::NeuronModelCorticalWriter;
use feagi_models::cortical_area::implementations::feagi_advanced::data::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData};
use feagi_models::cortical_area::implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use feagi_models::cortical_area::neuron::neuron_model::neuron_model::NeuronModel;
use feagi_models::cortical_area::neuron::neuron_model::quantization::NeuronModelQuantization;
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, MappingEntryEngineIndex, NeuronEngineIndex};

/// When a burst engine is stopped, these function calls
pub trait EditableEngine<FIQ: FeagiIndexQuantization> {
    // TODO: `neuron_data_writer` should become a real `NeuronDataWriter` trait bound once per-neuron
    // initial-value seeding is designed; for now callers just pass `()`.
    // TODO the engine only allocates `FeagiAdvanced` / standard quant storage for now, so the
    // quantization and the model data types are pinned to that pair. Restore the generic `NMQ` once
    // storage exists for every model / quantization combination.
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
        >;

    fn edit_cortical_area_cortical_flags(
        &mut self,
        cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    );

    fn edit_cortical_area_cortical_data<NMQ: NeuronModelQuantization, NM: NeuronModel<FIQ, NMQ>>(&mut self, new_cortical_data: NM::CorticalData);

    fn remove_cortical_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>);

    fn resize_dimensional_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>);

    fn add_mapping_entry(&mut self) -> MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>;

    fn remap_mapping_entry(&mut self, mapping_entry: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>);

    fn remove_mapping_entry(&mut self, mapping_entry: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>);

    /*
    fn probe_cortical_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, flags);

    fn probe_neurons(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, iterator);

    fn probe_mapping_entries(&mut self);

     */

    // TODO defragging
}
