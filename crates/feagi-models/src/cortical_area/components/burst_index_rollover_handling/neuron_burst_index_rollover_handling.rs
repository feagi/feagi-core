use crate::cortical_area::neuron::cortical_data::NeuronModelCorticalData;
use crate::cortical_area::neuron::neuron_data::NeuronModelNeuronData;
use crate::cortical_area::neuron::neuron_model_quantization::NeuronModelQuantization;
use crate::wrapped_indexes::BurstIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Allows configuring a special function for a neuron model in the case of the burst engine index
/// about to roll over (where it will be reset). Important for neuron models that store
/// time state in their per neuron data. Most of the time, the default
/// `NeuronModelNoSpecialBurstIndexRolloverHandling` implementation is enough since NeuronHistory
/// is automatically managed for roll over by the burst engine
pub trait NeuronModelBurstIndexRolloverHandling<FIQ, NMQ, NMCD, NMND>
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,       // NOTE: This should be extended for the given neuron model!
    NMCD: NeuronModelCorticalData<NMQ>, // NOTE: This should be extended for the given neuron model!
    NMND: NeuronModelNeuronData<NMQ>,
{
    /// If the neuron model has a special function that needs to be called if the burst index
    /// is being rolled over (to prevent overflow) set this to true and the accompanying function
    /// in this trait will be called. If its false, the neurons of this model will never be called
    /// to update their data when a burst index rollover occurs
    const NEURON_MODEL_HAS_SPECIAL_BURST_INDEX_ROLLOVER_FUNCTION: bool = true;

    /// The function that will be called per neuron of a cortical area if the model type needs
    /// to do something if there is about to be a burst index rollover. Use this function to
    /// update any values in the neuron to ensure a smooth transition. Note that we do not edit
    /// cortical level data, only neuron level!
    fn special_rollover_handling(
        neuron_data: &mut NMND,
        previous_burst: BurstIndex<FIQ::GlobalBurstIndexQuant>,
        upcoming_rollover_burst: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    );
}

