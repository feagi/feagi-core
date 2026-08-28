use crate::neuron_model::cortical_area::neuron_history::neuron_history::NeuronModelHistory;
use crate::wrapped_indexes::BurstIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// The neuron model stores per neuron the last time it received an input (active) and the last time
/// if fired
#[derive(Clone)]
pub struct NeuronModelFullNeuronHistory<FIQ: FeagiIndexQuantization> {
    pub burst_last_active: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    pub burst_last_fired: BurstIndex<FIQ::GlobalBurstIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronModelHistory<FIQ> for NeuronModelFullNeuronHistory<FIQ> {
    const NEURON_MODEL_USES_HISTORY: bool = true;

    fn new(current_burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>, offset: BurstIndex<FIQ::GlobalBurstIndexQuant>) -> Self {
        // TODO fix ord let index = current_burst_index - BurstIndex::min(current_burst_index, offset);
        let index = BurstIndex::QUANT_ZERO;
        Self {
            burst_last_active: index,
            burst_last_fired: index,
        }
    }

    fn update_activity_and_fire(&mut self, burst: BurstIndex<FIQ::GlobalBurstIndexQuant>) {
        self.burst_last_active = burst;
        self.burst_last_fired = burst;
    }
}
