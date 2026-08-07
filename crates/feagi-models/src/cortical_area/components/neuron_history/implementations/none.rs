use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_area::components::neuron_history::neuron_history::NeuronModelHistory;
use crate::wrapped_indexes::BurstIndex;

/// The neuron model does not implement any neuron history at all
#[derive(Clone)]
pub struct NeuronModelNoNeuronHistory();

impl<FIQ: FeagiIndexQuantization> NeuronModelHistory<FIQ> for NeuronModelNoNeuronHistory {
    const NEURON_MODEL_USES_HISTORY: bool = false;

    fn new(_current_burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>, _offset: BurstIndex<FIQ::GlobalBurstIndexQuant>) -> Self {
        NeuronModelNoNeuronHistory {} // you arent supposed to init this
    }

    fn update_activity_and_fire(&mut self, burst: BurstIndex<FIQ::GlobalBurstIndexQuant>) {
        panic!("Cannot update neuron history of a NeuronModelNoNeuronHistory")
    }
}