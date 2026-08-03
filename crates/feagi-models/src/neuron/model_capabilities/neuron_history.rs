use crate::wrapped_indexes::BurstIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;

/// Defines the type of neuron history that a neuron model will use. Do not extend this trait
/// or try implementing it yourself, rather implement one of the implementations below
pub trait NeuronModelHistory<FIQ>: Clone
where
    FIQ: FeagiIndexQuantization,
{
    /// If this is true, the burst engine will automatically maintain a history of activity for
    /// all neurons of this model type
    const NEURON_MODEL_USES_HISTORY: bool = true;

    /// When using history, how far back to init the neuron history struct
    const INITIAL_BURST_INDEX_OFFSET: BurstIndex<FIQ::GlobalBurstIndexQuant> = BurstIndex::const_new(FIQ::GlobalBurstIndexQuant::QUANT_MAX_U8);

    fn new(current_burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>, offset: BurstIndex<FIQ::GlobalBurstIndexQuant>) -> Self;    
    
    fn update_activity_and_fire(&mut self, burst: BurstIndex<FIQ::GlobalBurstIndexQuant>);
}

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
