use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::wrapped_indexes::BurstIndex;

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