use crate::cortical_models::core_power::data::NeuronModelCorePower;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::NeuralBurstEngineDynamics;
use feagi_connectome_neural::neural_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
#[cfg(feature = "burst-engine-cpu")]
use feagi_basis::{neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential}, BurstIndex};
#[cfg(feature = "burst-engine-cpu")]
use feagi_connectome_neural::neural_basis::neuron_model::data::neuron_layout::implementations::voxels_default::NeuronLayoutVoxelDefaultData;

pub struct CorePowerBurstEngineDynamics<FIQ: FeagiIndexQuantization, NM: NeuronModelCorePower>(
    core::marker::PhantomData<(FIQ, NM)>,
);

//region Common
impl<FIQ, NM> CorePowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelCorePower,
{
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<FIQ, NM> Default for CorePowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelCorePower,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<FIQ, NM> NeuralBurstEngineDynamics<FIQ, NM> for CorePowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelCorePower,
{
}
//endregion

#[cfg(feature = "burst-engine-cpu")]
impl<FIQ, NM> feagi_connectome_neural::burst_engine_cpu::CorticalCPUParallelDynamics<FIQ, NM>
for CorePowerBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelCorePower,
{
    fn process_model_cortical_dynamics(
        _burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        _layout_context: &NeuronLayoutVoxelDefaultData<FIQ>,
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
        _layout_context: &NeuronLayoutVoxelDefaultData<FIQ>,
        _neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _cortical_data: (&NM::CorticalDataProperties, &NM::CorticalDataInternal, &NM::CorticalDataScratch),
        _neuron_data: (&mut NM::NeuronDataInternal, &mut NM::NeuronDataScratch),
    ) -> bool {
        // Power type always fires. We don't care about anything else
        // TODO we should enforce ! md-psp for this, which means this enforcement needs to exist
        true

    }
}


