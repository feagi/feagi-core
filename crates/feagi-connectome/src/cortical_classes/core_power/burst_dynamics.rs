use crate::cortical_classes::core_power::data::{CorticalDynamicsModelCorePower, CorticalDynamicsQuantizationCorePower};
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::BurstIndex;
use feagi_connectome_neural::neural_basis::cortical_area::cortical_burst_engine_dynamics::CorticalBurstEngineDynamics;
use feagi_connectome_neural::neural_basis::cortical_area::neuron_layout::implementations::voxels_default::NeuronLayoutVoxelDefault;
use feagi_connectome_neural::neural_basis::cortical_area::neuron_layout::NeuronLayout;
use feagi_connectome_neural::neural_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct CorePowerBurstEngineDynamics<FIQ: FeagiIndexQuantization, CDQ: CorticalDynamicsQuantizationCorePower>(
    core::marker::PhantomData<(FIQ, CDQ)>,
);

//region Common
impl<FIQ, CDQ> CorePowerBurstEngineDynamics<FIQ, CDQ>
where
    FIQ: FeagiIndexQuantization,
    CDQ: CorticalDynamicsQuantizationCorePower,
{
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<FIQ, CDQ> Default for CorePowerBurstEngineDynamics<FIQ, CDQ>
where
    FIQ: FeagiIndexQuantization,
    CDQ: CorticalDynamicsQuantizationCorePower,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<FIQ, CDQ> CorticalBurstEngineDynamics<FIQ, NeuronLayoutVoxelDefault<FIQ>, CDQ> for CorePowerBurstEngineDynamics<FIQ, CDQ>
where
    FIQ: FeagiIndexQuantization,
    CDQ: CorticalDynamicsQuantizationCorePower,
{
    type CorticalModel = CorticalDynamicsModelCorePower<FIQ, CDQ>;
}
//endregion

#[cfg(feature = "burst-engine-cpu")]
impl<FIQ, CDQ> feagi_connectome_neural::burst_engine_cpu::CorticalCPUParallelDynamics<FIQ, NeuronLayoutVoxelDefault<FIQ>, CDQ>
for CorePowerBurstEngineDynamics<FIQ, CDQ>
where
    FIQ: FeagiIndexQuantization,
    CDQ: CorticalDynamicsQuantizationCorePower,
{
    fn process_model_cortical_dynamics(
        _burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        _layout_context: &NeuronLayoutVoxelDefault<FIQ>,
        _cortical_data: (
            &CDQ::CorticalDataProperties,
            &mut CDQ::CorticalDataInternal,
            &mut CDQ::CorticalDataScratch,
        ),
    ) -> () {
        // Nothing! This model doesnt care about cortical state!
    }

    fn process_model_neuron_dynamics(
        _processing_buffer: &mut CorticalAreaNeuronPotential<CDQ::MembranePotentialQuant>,
        _burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        _layout_context: &NeuronLayoutVoxelDefault<FIQ>,
        _neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _cortical_data: (&CDQ::CorticalDataProperties, &CDQ::CorticalDataInternal, &CDQ::CorticalDataScratch),
        _neuron_data: (&mut CDQ::NeuronDataInternal, &mut CDQ::NeuronDataScratch),
    ) -> bool {
        // Power type always fires. We don't care about anything else
        // TODO we should enforce ! md-psp for this, which means this enforcement needs to exist
        true

    }
}


