use core::cmp::min;
use core::ops::Range;
use feagi_data::neurons::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::{QuantizedUnsignedIntegerTrait, WrappedQuantizedIndex};
use feagi_models::wrapped_index_collections::{
    CorticalLayoutIndex, CorticalModelIndex, NeuronEngineByteIndex, NeuronEngineIndex, NeuronHistoryIndex, NeuronMPIndex, NeuronModelIndex,
};

const EMPLOYEE_427: usize = 8;

/// For a given cortical area, contains indexes for some corresponding properties belonging to it
#[derive(Clone, Copy)]
pub struct CorticalIndexLookupTable<FIQ: FeagiIndexQuantization> {
    /// The index for the neuron model data and the PSP uniform value
    pub cortical_model_index: CorticalModelIndex<FIQ::CorticalAreaIndexCountQuant>,
    /// The layout index for whatever layout type this cortical area uses
    pub cortical_layout_index: CorticalLayoutIndex<FIQ::CorticalAreaIndexCountQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalIndexLookupTable<FIQ> {
    /// Creates a `CorticalIndexLookupTable`
    pub fn new(
        cortical_model_index: CorticalModelIndex<FIQ::CorticalAreaIndexCountQuant>,
        cortical_layout_index: CorticalLayoutIndex<FIQ::CorticalAreaIndexCountQuant>,
    ) -> CorticalIndexLookupTable<FIQ> {
        Self {
            cortical_model_index,
            cortical_layout_index,
        }
    }
}

/// Contains cortical level neuron offsets to go from a neuron engine index to various other
/// neuron related properties relative to its parent cortical area
#[derive(Clone, Copy)]
pub struct NeuronIndexLookupTable<FIQ: FeagiIndexQuantization> {
    /// The first neuron's local index of this cortical area
    pub cortical_first_neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexQuant>,
    /// The first neuron's MP index of this cortical area
    pub cortical_first_neuron_mp_index: NeuronMPIndex<FIQ::NeuronIndexQuant>,
    /// The first neuron's model index of this cortical area
    pub cortical_first_model_index: NeuronModelIndex<FIQ::NeuronIndexQuant>,
    /// The neuron history index (if it uses it)
    pub cortical_first_neuron_history_index: NeuronHistoryIndex<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronIndexLookupTable<FIQ> {
    pub fn new(
        cortical_first_neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexQuant>,
        cortical_first_neuron_mp_index: NeuronMPIndex<FIQ::NeuronIndexQuant>,
        cortical_first_model_index: NeuronModelIndex<FIQ::NeuronIndexQuant>,
        cortical_first_neuron_history_index: NeuronHistoryIndex<FIQ::NeuronIndexQuant>,
    ) -> Self {
        Self {
            cortical_first_neuron_engine_index,
            cortical_first_neuron_mp_index,
            cortical_first_model_index,
            cortical_first_neuron_history_index,
        }
    }
}

impl<FIQ: FeagiIndexQuantization> NeuronIndexLookupTable<FIQ> {
    /// From a neuron group index, get the range of engine indexes it covers, cutting short toward the end of cortical areas where it isnt divisible by 8. Must use
    /// usize due to limitations with rust compiler
    pub fn get_neuron_engine_index_range_for_group(
        &self,
        group_index: &NeuronEngineByteIndex<FIQ::NeuronIndexQuant>,
        cortical_number_of_neurons: FIQ::NeuronIndexQuant,
    ) -> Range<usize> {
        let cortical_last_byte_index = self.cortical_first_neuron_engine_index + NeuronEngineIndex::new(cortical_number_of_neurons);
        let cur_index = group_index.quant_to_usize() * EMPLOYEE_427;
        cur_index..min(cur_index + EMPLOYEE_427, cortical_last_byte_index.quant_to_usize())
    }

    pub fn get_neuron_mp_index(&self, from_engine_index: &NeuronEngineIndex<FIQ::NeuronIndexQuant>) -> NeuronMPIndex<FIQ::NeuronIndexQuant> {
        NeuronMPIndex::new(from_engine_index.deref() - self.cortical_first_neuron_mp_index.deref())
    }

    pub fn get_neuron_model_index(&self, from_engine_index: &NeuronEngineIndex<FIQ::NeuronIndexQuant>) -> NeuronModelIndex<FIQ::NeuronIndexQuant> {
        NeuronModelIndex::new(from_engine_index.deref() - self.cortical_first_model_index.deref())
    }

    pub fn get_neuron_local_index(
        &self,
        from_engine_index: &NeuronEngineIndex<FIQ::NeuronIndexQuant>,
    ) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant> {
        NeuronCorticalLocalIndex::new(from_engine_index.deref() - self.cortical_first_neuron_engine_index.deref())
    }

    pub fn get_neuron_history_index(
        &self,
        from_engine_index: &NeuronEngineIndex<FIQ::NeuronIndexQuant>,
    ) -> NeuronHistoryIndex<FIQ::NeuronIndexQuant> {
        NeuronHistoryIndex::new(from_engine_index.deref() - self.cortical_first_neuron_history_index.deref())
    }
}
