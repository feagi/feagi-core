use crate::burst_index::BurstIndex;
use crate::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use crate::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Allows configuring a special function for a neuron model in the case of the burst engine index
/// about to roll over (where it will be reset). Important for neuron models that store
/// time state in their per neuron data. Most of the time, the default 
/// `NeuronModelNoSpecialBurstIndexRolloverHandling` implementation is enough since NeuronHistory
/// is automatically managed for roll over by the burst engine
pub trait NeuronModelBurstIndexRolloverHandling<FIQ, NMQ, NMCD, NMND>
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization, // NOTE: This should be extended for the given neuron model!
    NMCD: NeuronModelCorticalData<NMQ>, // NOTE: This should be extended for the given neuron model!
    NMND: NeuronModelNeuronData<NMQ>, // NOTE: This should be extended for the given neuron model!
{
    /// If the neuron model has a special function that needs to be called if the burst index
    /// is being rolled over (to prevent overflow) set this to true and the accompanying function
    /// in this trait will be called. If its false, the neurons of this model will never be called
    /// to update their data when a burst index rollover occurs
    const NEURON_MODEL_HAS_SPECIAL_BURST_INDEX_ROLLOVER_FUNCTION: bool = true;

    /// The function that will be called per neuron of a cortical area if the model type needs
    /// to do something if there is about to be a burst index rollover. Use this function to
    /// update any values in the neuron to ensure a smooth transition. Note that we do not
    fn special_rollover_handling(
        
        neuron_data: &mut NMND,
        previous_burst: BurstIndex<FIQ::GlobalBurstIndexQuant>,
        upcoming_rollover_burst: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    );
}

/// The case for many neuron models, since Neuron History is automatically rolled over
/// by the burst engine anyway
pub struct NeuronModelNoSpecialBurstIndexRolloverHandling;

impl<FIQ, NMQ, NMCD, NMND> NeuronModelBurstIndexRolloverHandling<FIQ, NMQ, NMCD, NMND> for NeuronModelNoSpecialBurstIndexRolloverHandling
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization, // NOTE: This should be extended for the given neuron model!
    NMCD: NeuronModelCorticalData<NMQ>, // NOTE: This should be extended for the given neuron model!
    NMND: NeuronModelNeuronData<NMQ>, // NOTE: This should be extended for the given neuron model!
{
    /// This case its explicitly not
    const NEURON_MODEL_HAS_SPECIAL_BURST_INDEX_ROLLOVER_FUNCTION: bool = false;

    fn special_rollover_handling(
        _neuron_data: &mut NMND,
        _previous_burst: BurstIndex<FIQ::GlobalBurstIndexQuant>,
        _upcoming_rollover_burst: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    ) {
        // This function should never be called!
        panic!("Burst Rollover handling called when not supported!")
    }
}
