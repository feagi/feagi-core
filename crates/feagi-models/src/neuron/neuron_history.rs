use std::marker::PhantomData;
use crate::burst_index::BurstIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Describes the type of Neuron History used
pub trait NeuronHistoryBackend {
}

/// No data stored as history.
#[derive(Clone, Copy)]
pub struct NeuronHistoryNone<FIQ: FeagiIndexQuantization> {
    // It's nothing!
    // This is basically just a stub to satisfy compilation
    _p : PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> NeuronHistoryBackend for NeuronHistoryNone<FIQ> {}

/// Contains the burst indexes from when the neuron was last active / fired. Only neuron models
/// that require this data will get this data indexed
#[derive(Clone, Copy)]
pub struct NeuronHistoryFull<FIQ: FeagiIndexQuantization> {
    pub burst_last_active: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    pub burst_last_fired: BurstIndex<FIQ::GlobalBurstIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronHistoryBackend for NeuronHistoryFull<FIQ> {}
