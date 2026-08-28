use crate::neuron_model::cortical_area::burst_index_rollover_handling::neuron_burst_index_rollover_handling::NeuronModelBurstIndexRolloverHandling;
use crate::neuron_model::cortical_area::cortical_data::NeuronModelCorticalData;
use crate::neuron_model::neuron::neuron_data::NeuronModelNeuronData;
use crate::neuron_model::neuron::neuron_model_quantization::NeuronModelQuantization;
use crate::wrapped_indexes::BurstIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// The case for many neuron models, since Neuron History is automatically rolled over
/// by the burst engine anyway
pub struct NeuronModelNoSpecialBurstIndexRolloverHandling;

impl<FIQ, NMQ, NMCD, NMND> NeuronModelBurstIndexRolloverHandling<FIQ, NMQ, NMCD, NMND> for NeuronModelNoSpecialBurstIndexRolloverHandling
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,       // NOTE: This should be extended for the given neuron model!
    NMCD: NeuronModelCorticalData<NMQ>, // NOTE: This should be extended for the given neuron model!
    NMND: NeuronModelNeuronData<NMQ>,   // NOTE: This should be extended for the given neuron model!
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
