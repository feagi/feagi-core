use feagi_data::neurons::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_index_collections::{CorticalLayoutIndex, CorticalModelIndex, NeuronEngineIndex, NeuronHistoryIndex, NeuronMPIndex};

/// Contains cortical level neuron offsets to go from a neuron engine index to various other
/// neuron related properties relative to its parent cortical area
#[derive(Clone, Copy)]
pub struct CorticalNeuronOffsets<FIQ: FeagiIndexQuantization>
{
    /// The first neuron's local index of this cortical area
    pub cortical_first_neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexCountQuant>,
    /// The count of neurons in this area. Used for byte index -> index limiting
    pub cortical_number_of_neurons: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    /// The first neuron's MP index of this cortical area
    pub cortical_first_neuron_mp_index: NeuronMPIndex<FIQ::NeuronIndexCountQuant>,
    /// The neuron history index (if it uses it)
    pub cortical_first_neuron_history_index: NeuronHistoryIndex<FIQ::NeuronIndexCountQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalNeuronOffsets<FIQ> {

    pub fn get_neuron_local_index(&self, from_engine_index: &NeuronEngineIndex<FIQ::NeuronIndexCountQuant>) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>{
        NeuronCorticalLocalIndex::new(from_engine_index.deref() - self.cortical_first_neuron_engine_index.deref())
    }

    pub fn get_neuron_mp_index(&self, from_engine_index: &NeuronEngineIndex<FIQ::NeuronIndexCountQuant>) -> NeuronMPIndex<FIQ::NeuronIndexCountQuant>{
        NeuronMPIndex::new(from_engine_index.deref() - self.cortical_first_neuron_mp_index.deref())
    }
    
    pub fn get_neuron_history_index(&self,  from_engine_index: &NeuronEngineIndex<FIQ::NeuronIndexCountQuant>) -> NeuronHistoryIndex<FIQ::NeuronIndexCountQuant>{
        NeuronHistoryIndex::new(from_engine_index.deref() - self.cortical_first_neuron_history_index.deref())
    }

    pub fn get_number_of_neurons_in_neuron_byte(&self, from_local_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>) -> FIQ::NeuronIndexCountQuant{
        FIQ::NeuronIndexCountQuant::min(self.cortical_number_of_neurons - from_local_index.deref(), 8)
    }
    
}

