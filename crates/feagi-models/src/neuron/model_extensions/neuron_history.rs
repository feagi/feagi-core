use crate::wrapped_indexes::BurstIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Defines the type of neuron history that a neuron model will use. Do not extend this trait
/// or try implementing it yourself, rather implement one of the implementations below
pub trait NeuronModelHistory<FIQ>: Clone
where
    FIQ: FeagiIndexQuantization,
{
    /// If this is true, the burst engine will automatically maintain a history of activity for
    /// all neurons of this model type
    const NEURON_MODEL_USES_HISTORY: bool = true;

    fn update_activity_and_fire(&mut self, burst: BurstIndex<FIQ::GlobalBurstIndexQuant>);
}

/// The neuron model does not implement any neuron history at all
#[derive(Clone)]
pub struct NeuronModelNoNeuronHistory();

impl<FIQ: FeagiIndexQuantization> NeuronModelHistory<FIQ> for NeuronModelNoNeuronHistory {
    const NEURON_MODEL_USES_HISTORY: bool = false;

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

    fn update_activity_and_fire(&mut self, burst: BurstIndex<FIQ::GlobalBurstIndexQuant>) {
        self.burst_last_active = burst;
        self.burst_last_fired = burst;
    }
}
