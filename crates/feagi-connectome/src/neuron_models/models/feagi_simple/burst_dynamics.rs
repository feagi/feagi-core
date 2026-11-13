use crate::neuron_models::models::feagi_simple::data::wrapped::Period;
use crate::neuron_models::models::feagi_simple::NMFeagiSimpleTrait;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::*;
use feagi_basis::BurstIndex;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::neuron_model::NeuralBurstEngineDynamics;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

pub struct FeagiSimpleBurstEngineDynamics<FIQ: FeagiIndexQuantization, NM: NMFeagiSimpleTrait>(core::marker::PhantomData<(FIQ, NM)>);

impl<FIQ, NM> Default for FeagiSimpleBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMFeagiSimpleTrait,
{
    fn default() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<FIQ, NM> NeuralBurstEngineDynamics<FIQ, NM> for FeagiSimpleBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMFeagiSimpleTrait,
{}

//endregion

/*
#[cfg(feature = "burst-engine-cpu")]
impl<FIQ, NM> feagi_connectome_neural::burst_engine_cpu::NeuronCPUParallelDynamics<FIQ, NM> for FeagiSimpleBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMFeagiSimpleTrait,
{
    fn process_model_cortical_area_dynamics(
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NeuronLayoutOf<FIQ, NM>,
        cortical_data: (&NM::CorticalDataProperties, &mut NM::CorticalDataInternal, &mut NM::CorticalDataScratch),
    ) -> () {
        // Unused
        ()
    }

    fn process_model_neuron_dynamics(
        processing_buffer: &mut CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
        _burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        _layout_context: &NeuronLayoutOf<FIQ, NM>,
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
        if neuron_internal.refractory_countdown > Period::QUANT_ONE {
            neuron_internal.refractory_countdown -= Period::QUANT_ONE;
            *processing_buffer = CorticalAreaNeuronPotential::QUANT_ZERO;
            return false;
        }
        
        let firing_threshold =
            cortical_properties.firing_threshold;
        if *processing_buffer >= firing_threshold {
            neuron_internal.refractory_countdown = cortical_properties.refractory_period;
            return true;
        }

        false
    }
}


 */