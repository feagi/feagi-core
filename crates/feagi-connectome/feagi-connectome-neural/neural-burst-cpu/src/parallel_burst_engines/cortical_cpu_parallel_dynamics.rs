use feagi_basis::BurstIndex;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use neural_basis::cortical_area::neuron_layout::NeuronLayout;
use neural_basis::cortical_area::cortical_burst_engine_dynamics::CorticalBurstEngineDynamics;
use neural_basis::cortical_area::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use neural_basis::cortical_area::per_cortical_data::CorticalAreaUbiquitousFlags;
use neural_basis::cortical_area::per_neuron_data::{InverseOutgoingConnectionCount, PerNeuronUbiquitousFlags};
use neural_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use neural_basis::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

pub trait CorticalCPUParallelDynamics<FIQ, NL, CDQ>: CorticalBurstEngineDynamics<FIQ, NL, CDQ>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
{
    /// called per area. Called before neuron call
    fn process_model_cortical_dynamics(
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NL,
        cortical_data: (
            &CDQ::CorticalDataProperties,
            &mut CDQ::CorticalDataInternal,
            &mut CDQ::CorticalDataScratch
        )
    ) -> (); // TODO return type?

    /// called per neuron, outputs the firing potential (to processing buffer) to be further post processed (or ignored) and if we are firing
    fn process_model_neuron_dynamics(
        // Inputs as the incoming potential, but is output as the firing potential
        processing_buffer: &mut CorticalAreaNeuronPotential<CDQ::MembranePotentialQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NL,
        neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        cortical_data: (
            &CDQ::CorticalDataProperties,
            &CDQ::CorticalDataInternal,
            &CDQ::CorticalDataScratch
        ),
        neuron_data: (
            &mut CDQ::NeuronDataInternal,
            &mut CDQ::NeuronDataScratch
        ),
    ) -> bool; // true if firing


    /// called per neuron, calls `process_model_neuron_dynamics` and does the psp postprocessing
    /// upon it. DO NOT override this implementation!
    fn process_neuron_dynamics_for_psp(
        // Inputs as the incoming potential, but is output as the firing potential
        processing_buffer: &mut CorticalAreaNeuronPotential<CDQ::MembranePotentialQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        layout_context: &NL,
        neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        cortical_data: (
            &CDQ::CorticalDataProperties,
            &CDQ::CorticalDataInternal,
            &CDQ::CorticalDataScratch
        ),
        neuron_data: (
            &mut CDQ::NeuronDataInternal,
            &mut CDQ::NeuronDataScratch
        ),
        cortical_flags: &CorticalAreaUbiquitousFlags,
        cortical_level_psp: &CorticalAreaNeuronPotential<CDQ::MembranePotentialQuant>,
        neuron_inversed_outgoing_connection_count: &InverseOutgoingConnectionCount<CDQ::MembranePotentialQuant>,
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