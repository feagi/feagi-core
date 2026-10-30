use crate::neuron_models::feagi_advanced::data::NeuronModelFeagiAdvanced;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::BurstIndex;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::{NeuralBurstEngineDynamics, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct FeagiAdvancedBurstEngineDynamics<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiAdvanced>(core::marker::PhantomData<(FIQ, NM)>);

//region Common

impl<FIQ, NM> FeagiAdvancedBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFeagiAdvanced,
{
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<FIQ, NM> Default for FeagiAdvancedBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFeagiAdvanced,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<FIQ, NM> NeuralBurstEngineDynamics<FIQ, NM> for FeagiAdvancedBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFeagiAdvanced,
{
}

//endregion

#[cfg(feature = "burst-engine-cpu")]
impl<FIQ, NM> feagi_connectome_neural::burst_engine_cpu::CorticalCPUParallelDynamics<FIQ, NM>
for FeagiAdvancedBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFeagiAdvanced,
{
    fn process_model_cortical_dynamics(
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NeuronLayoutOf<NM, FIQ>,
        cortical_data: (&NM::CorticalDataProperties, &mut NM::CorticalDataInternal, &mut NM::CorticalDataScratch),
    ) -> () {
        todo!()
    }

    fn process_model_neuron_dynamics(
        processing_buffer: &mut CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NeuronLayoutOf<NM, FIQ>,
        neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        cortical_data: (&NM::CorticalDataProperties, &NM::CorticalDataInternal, &NM::CorticalDataScratch),
        neuron_data: (&mut NM::NeuronDataInternal, &mut NM::NeuronDataScratch),
    ) -> bool {
        todo!()
    }
}
