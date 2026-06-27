use core::marker::PhantomData;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use crate::neuron_models::feagi_standard::data::{FeagiStandardModelCorticalDataCPU, FeagiStandardModelNeuronDataCPU};
use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;
use crate::neuron_models::neuron_model_traits::neuron_model_processor::{NeuronModelProcessor, NeuronModelProcessorBase, NeuronModelProcessorBaseCPU, NeuronModelProcessorWithBurstHistoryCPU, NeuronModelProcessorWithHistory};

pub struct FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization, // fsm quant impl
{
    // No actual members
    _p: PhantomData<(FGQ, NMQ)>,
}


impl<FGQ, NMQ> NeuronModelProcessorBase<FGQ, NMQ, FeagiStandardModelCorticalDataCPU<NMQ>, FeagiStandardModelNeuronDataCPU<NMQ>> for FeagiStandardModelProcessorCPU<FGQ, NMQ> where FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization, {}

impl<FGQ, NMQ> NeuronModelProcessor<
    FGQ,
    NMQ,
    FeagiStandardModelCorticalDataCPU<NMQ>,
    FeagiStandardModelNeuronDataCPU<NMQ>,
> for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
}

impl<FGQ, NMQ> NeuronModelProcessorBaseCPU<FGQ, NMQ, FeagiStandardModelCorticalDataCPU<NMQ>, FeagiStandardModelNeuronDataCPU<NMQ>> for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{}

impl<FGQ, NMQ> NeuronModelProcessorWithHistory<FGQ, NMQ, FeagiStandardModelCorticalDataCPU<NMQ>, FeagiStandardModelNeuronDataCPU<NMQ>> for FeagiStandardModelProcessorCPU<FGQ, NMQ> where FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization, {}

impl<FGQ, NMQ> NeuronModelProcessorWithBurstHistoryCPU<
    FGQ,
    NMQ,
    FeagiStandardModelCorticalDataCPU<NMQ>,
    FeagiStandardModelNeuronDataCPU<NMQ>
> for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    fn process_neuron_potential_for_dimensional_cortical_configuration(
        incoming_potential: &NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUWrappedBurstEngineBurstIndex<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_activity: &NPUWrappedBurstEngineBurstIndex<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_firing: &NPUWrappedBurstEngineBurstIndex<FGQ::GlobalBurstIndexQuant>,
        cortical_layout_dimensional: &CorticalLayoutDimensionalCPU<FGQ>,
        cortical_area_data: &FeagiStandardModelCorticalDataCPU<NMQ>,
        neuron_model_data: &mut FeagiStandardModelNeuronDataCPU<NMQ>,
        this_neuron_potential: &mut NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>
    ) -> bool {

        *this_neuron_potential =
            this_neuron_potential + incoming_potential
                - (neuron_model_data.neuron_leak_coefficient * (this_neuron_potential - 0.0 )); // TODO right now subtracting 0, but this is the resting potential

        let should_fire: bool;



        /// If consecutive fire is disabled, its set to 0
        if cortical_area_data.consecutive_fire_limit == 0 {
            if this_neuron_potential > neuron_model_data.neuron_fire_threshold
            {
                if cortical_area_data.fire_threshold_limit == 0.0 || this_neuron_potential < cortical_area_data.fire_threshold_limit
                {
                    return true
                }
            }
            return false
        }

        return should_fire;
    }
}

