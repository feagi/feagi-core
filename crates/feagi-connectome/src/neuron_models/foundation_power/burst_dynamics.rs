use crate::neuron_models::foundation_power::data::NeuronModelFoundationPower;
use feagi_basis::{neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential}, BurstIndex};
use feagi_connectome_neural::neural_basis::neuron_model::{NeuralBurstEngineDynamics, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

pub struct FoundationPowerBurstEngineDynamics<FIQ: FeagiIndexQuantization, NM: NeuronModelFoundationPower>(
    core::marker::PhantomData<(FIQ, NM)>,
);

//region Common
impl<FIQ, NM> FoundationPowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFoundationPower,
{
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<FIQ, NM> Default for FoundationPowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFoundationPower,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<FIQ, NM> NeuralBurstEngineDynamics<FIQ, NM> for FoundationPowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFoundationPower,
{
}
//endregion



#[cfg(feature = "burst-engine-cpu")]
impl<FIQ, NM> feagi_connectome_neural::burst_engine_cpu::CorticalCPUParallelDynamics<FIQ, NM>
for FoundationPowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFoundationPower,
{
    fn process_model_cortical_dynamics(
        _burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        _layout_context: &NeuronLayoutOf<NM, FIQ>,
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
        _layout_context: &NeuronLayoutOf<NM, FIQ>,
        _neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _cortical_data: (&NM::CorticalDataProperties, &NM::CorticalDataInternal, &NM::CorticalDataScratch),
        _neuron_data: (&mut NM::NeuronDataInternal, &mut NM::NeuronDataScratch, &NM::NeuronDataVariance),
    ) -> bool {
        // Power type always fires. We don't care about anything else
        // TODO we should enforce ! md-psp for this, which means this enforcement needs to exist
        true

    }
}


