use feagi_data::index_range_managers::index_manager::IndexManager;
use feagi_data::index_range_managers::index_range_mapping_manager::IndexRangeMappingManager;
use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::genome_interface::cortical_area_spawners::DimensionalCorticalAreaSpawner;
use feagi_models::neuron::model_extensions::neuron_layout_implementations::DimensionalNeuronModel;
use feagi_models::neuron::neuron_model::NeuronModel;
use feagi_models::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, NeuronEngineIndex};

pub struct RayonAllocationManager<FIQ: FeagiIndexQuantization> {
    cortical_area: RayonCorticalAreaAllocationManager<FIQ>,
    //mapping_entry: RayonMappingEntryAllocationManager<FIQ>,
    //synapse: RayonSynapseAllocationManager<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> RayonAllocationManager<FIQ> {
    //pub fn add_cortical_area()
}

struct RayonCorticalAreaAllocationManager<FIQ: FeagiIndexQuantization> {
    engine_cortical_indexes: IndexRangeMappingManager<CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, NeuronEngineIndex<FIQ::NeuronIndexCountQuant>>,
}

impl<FIQ: FeagiIndexQuantization> RayonCorticalAreaAllocationManager<FIQ> {

    /// Gets an immutable ref to the cortical to neuron range mapping manager to easily get context info
    pub fn get_cortical_index_mappings(&self) -> &IndexRangeMappingManager<CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, NeuronEngineIndex<FIQ::NeuronIndexCountQuant>> {
        &self.engine_cortical_indexes
    }

    /// allocates for (if needed) and adds a cortical area with its neurons.
    pub fn add_dimensional_cortical_area<NMQ: NeuronModelQuantization, NM: DimensionalNeuronModel<FIQ, NMQ>>(
        &mut self,
        rayon_data: (),
        dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        spawner: impl DimensionalCorticalAreaSpawner<FIQ, NMQ, NM>,
    ) -> Result<(), ()> {
        let amount_neurons_needed = NeuronEngineIndex::new(dimensions.number_contained_elements().deref());
        let context = self.engine_cortical_indexes.allocate_for_length(amount_neurons_needed)?;
        
        
        
        todo!()
    }
}

/*
struct RayonMappingEntryAllocationManager<FIQ: FeagiIndexQuantization> {}

struct RayonSynapseAllocationManager<FIQ: FeagiIndexQuantization> {}


 */