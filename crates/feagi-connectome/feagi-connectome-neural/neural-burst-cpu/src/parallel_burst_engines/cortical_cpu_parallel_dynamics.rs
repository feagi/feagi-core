use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::BurstIndex;
use neural_basis::neuron_model::neuron_model::NeuralBurstEngineDynamics;
use neural_basis::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use neural_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::shared_structs::{CorticalAreaUbiquitousFlags, InverseOutgoingConnectionCount, PerNeuronUbiquitousFlags};

pub trait CorticalCPUParallelDynamics<FIQ, NM>: NeuralBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// called per area. Called before neuron call
    fn process_model_cortical_dynamics(
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NeuronLayoutOf<NM, FIQ>,
        cortical_data: (
            &NM::CorticalDataProperties,
            &mut NM::CorticalDataInternal,
            &mut NM::CorticalDataScratch
        )
    ) -> (); // TODO return type?

    /// called per neuron, outputs the firing potential (to processing buffer) to be further post processed (or ignored) and if we are firing
    fn process_model_neuron_dynamics(
        // Inputs as the incoming potential, but is output as the firing potential
        processing_buffer: &mut CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NeuronLayoutOf<NM, FIQ>,
        neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        cortical_data: (
            &NM::CorticalDataProperties,
            &NM::CorticalDataInternal,
            &NM::CorticalDataScratch
        ),
        neuron_data: (
            &mut NM::NeuronDataInternal,
            &mut NM::NeuronDataScratch
        ),
    ) -> bool; // true if firing


    /// called per neuron, calls `process_model_neuron_dynamics` and does the psp postprocessing
    /// upon it. DO NOT override this implementation!
    fn process_neuron_dynamics_for_psp(
        // Inputs as the incoming potential, but is output as the firing potential
        processing_buffer: &mut CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NeuronLayoutOf<NM, FIQ>,
        neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        cortical_data: (
            &NM::CorticalDataProperties,
            &NM::CorticalDataInternal,
            &NM::CorticalDataScratch
        ),
        neuron_data: (
            &mut NM::NeuronDataInternal,
            &mut NM::NeuronDataScratch
        ),
        cortical_flags: &CorticalAreaUbiquitousFlags,
        cortical_level_psp: &CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
        neuron_inversed_outgoing_connection_count: &InverseOutgoingConnectionCount<NM::MembranePotentialQuant>,
        neuron_flags: &mut PerNeuronUbiquitousFlags
    ) {

        let model_is_firing = Self::process_model_neuron_dynamics(
            processing_buffer,
            burst_index,
            layout_context,
            neuron_linear_index,
            cortical_data,
            neuron_data
        );

        // At this point, the output neuron potential (regardless of firing state) was written
        // to processing_buffer

        // We should apply any modifiers to the outgoing neuron potential,
        // regardless of firing state
        if !cortical_flags.is_psp_using_membrane_potential() {
            *processing_buffer = *cortical_level_psp;
        }

        // We divide out the outgoing potential when PSP Uniformity is DISABLED!!!!!!!!!!!
        if !cortical_flags.is_psp_uniform() {
            *processing_buffer *= CorticalAreaNeuronPotential::new(neuron_inversed_outgoing_connection_count.deref());
        }

        // This struct will apply any force fires / probe overrides and store firing state
        _ = neuron_flags.process_neuron_firing(model_is_firing);
    }
}