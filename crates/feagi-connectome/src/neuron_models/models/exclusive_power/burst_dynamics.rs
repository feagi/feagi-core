use crate::neuron_models::models::exclusive_power::NMExclusivePowerTrait;
use feagi_basis::{neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential}, BurstIndex};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::neuron_model::NeuralBurstEngineDynamics;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

pub struct ExclusivePowerBurstEngineDynamics<FIQ: FeagiIndexQuantization, NM: NMExclusivePowerTrait>(
    core::marker::PhantomData<(FIQ, NM)>,
);

//region Common
impl<FIQ, NM> ExclusivePowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusivePowerTrait,
{
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<FIQ, NM> Default for ExclusivePowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusivePowerTrait,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<FIQ, NM> NeuralBurstEngineDynamics<FIQ, NM> for ExclusivePowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusivePowerTrait,
{
}
//endregion


/*

#[cfg(feature = "burst-engine-cpu")]
impl<FIQ, NM> feagi_connectome_neural::burst_engine_cpu::NeuronCPUParallelDynamics<FIQ, NM>
for ExclusivePowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusivePowerTrait,
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
        // Power type always fires. We don't care about anything else
        // TODO we should enforce ! md-psp for this, which means this enforcement needs to exist
        true

    }
}




 */