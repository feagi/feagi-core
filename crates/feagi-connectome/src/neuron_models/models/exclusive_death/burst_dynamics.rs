use crate::neuron_models::models::exclusive_death::NMExclusiveDeathTrait;
use feagi_basis::{neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential}, BurstIndex};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::neuron_model::NeuralBurstEngineDynamics;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
pub struct ExclusiveDeathBurstEngineDynamics<FIQ: FeagiIndexQuantization, NM: NMExclusiveDeathTrait>(
    core::marker::PhantomData<(FIQ, NM)>,
);

//region Common
impl<FIQ, NM> ExclusiveDeathBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusiveDeathTrait,
{
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<FIQ, NM> Default for ExclusiveDeathBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusiveDeathTrait,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<FIQ, NM> NeuralBurstEngineDynamics<FIQ, NM> for ExclusiveDeathBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusiveDeathTrait,
{
}
//endregion


/*
#[cfg(feature = "burst-engine-cpu")]
impl<FIQ, NM> feagi_connectome_neural::burst_engine_cpu::NeuronCPUParallelDynamics<FIQ, NM>
for ExclusiveDeathBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusiveDeathTrait,
{
    fn process_model_cortical_area_dynamics(
        _burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        _layout_context: &NeuronLayoutOf<FIQ, NM>,
        _cortical_data: (
            &NM::CorticalDataProperties,
            &mut NM::CorticalDataInternal,
            &mut NM::CorticalDataScratch,
        ),
    ) -> () {
        // Nothing! This model doesnt care about cortical state!
    }

    fn process_model_neuron_dynamics(
        _processing_buffer: &mut CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
        _burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        _layout_context: &NeuronLayoutOf<FIQ, NM>,
        _neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _cortical_data: (&NM::CorticalDataProperties, &NM::CorticalDataInternal, &NM::CorticalDataScratch),
        _neuron_data: (&mut NM::NeuronDataInternal, &mut NM::NeuronDataScratch, &NM::NeuronDataVariance),
    ) -> bool {
        // Death type is in a sense a motor in that it outputs, albiet a special channel
        false // TODO a bit better

    }
}


 */

