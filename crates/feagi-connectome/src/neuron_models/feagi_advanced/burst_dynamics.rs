use crate::neuron_models::feagi_advanced::data::{NeuronModelFeagiAdvanced, Period};
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::QuantizedElementBase;
use feagi_basis::BurstIndex;
use feagi_connectome_neural::neural_basis::neuron_model::{NeuralBurstEngineDynamics, NeuronLayoutOf};
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
impl<FIQ, NM> feagi_connectome_neural::burst_engine_cpu::CorticalCPUParallelDynamics<FIQ, NM> for FeagiAdvancedBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFeagiAdvanced,
{
    fn process_model_cortical_dynamics(
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NeuronLayoutOf<NM, FIQ>,
        cortical_data: (&NM::CorticalDataProperties, &mut NM::CorticalDataInternal, &mut NM::CorticalDataScratch),
    ) -> () {
        // Unused
        ()
    }

    fn process_model_neuron_dynamics(
        processing_buffer: &mut CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
        _burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        _layout_context: &NeuronLayoutOf<NM, FIQ>,
        _neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        cortical_data: (&NM::CorticalDataProperties, &NM::CorticalDataInternal, &NM::CorticalDataScratch),
        neuron_data: (&mut NM::NeuronDataInternal, &mut NM::NeuronDataScratch, &NM::NeuronDataVariance),
    ) -> bool {
        // TODO extremely simple port, redo later

        let cortical_properties = cortical_data.0;
        let _cortical_internal = cortical_data.1;
        let _cortical_scratch = cortical_data.2;

        let neuron_internal = neuron_data.0;
        let _neuron_scratch = neuron_data.1;
        let neuron_variance = neuron_data.2;

        // A refractory neuron stays at rest, and any incoming charge is lost
        if neuron_internal.refractory_countdown > Period::QUANT_ZERO {
            neuron_internal.refractory_countdown -= Period::QUANT_ONE;
            *processing_buffer = CorticalAreaNeuronPotential::QUANT_ZERO;
            return false;
        }

        // The buffer holds the carried potential with this burst's input already integrated
        let potential = processing_buffer.deref();
        let firing_threshold =
            cortical_properties.firing_threshold + neuron_variance.firing_threshold_offset;
        if potential >= firing_threshold {
            neuron_internal.refractory_countdown = cortical_properties.refractory_period;
            return true;
        }

        // Only a neuron that did not fire keeps charge to leak
        let retained_fraction =
            NM::MembranePotentialQuant::QUANT_ONE - cortical_properties.leak_coefficient;
        *processing_buffer = CorticalAreaNeuronPotential::new(potential * retained_fraction);
        false
    }
}
