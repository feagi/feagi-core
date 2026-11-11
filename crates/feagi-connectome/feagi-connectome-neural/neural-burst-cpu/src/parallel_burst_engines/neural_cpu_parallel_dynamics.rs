use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::BurstIndex;
use neural_basis::neuron_model::{NeuralBurstEngineDynamics, NeuronModel};
use neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::shared_structs::{CorticalAreaUbiquitousFlags, InverseOutgoingConnectionCount, PerNeuronUbiquitousFlags};

pub trait NeuronCPUParallelDynamics<FIQ, NM>: NeuralBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// called per area. Called before neuron call
    fn process_model_cortical_area_dynamics(
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
            &mut NM::NeuronDataScratch,
            &NM::NeuronDataVariance
        ),
    ) -> bool; // true if firing


    /// called per neuron, calls `process_model_neuron_dynamics`, stores if the neuron is firing in
    /// its flags, and if it is, does the psp postprocessing upon it. The caller has already summed
    /// the incoming potentials into `processing_buffer`. DO NOT override this implementation!
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
            &mut NM::NeuronDataScratch,
            &NM::NeuronDataVariance
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

        // At this point processing_buffer holds the firing potential if the model fired, otherwise
        // the charge the neuron retained

        // This struct will apply any force fires / probe overrides and store firing state
        let is_firing = neuron_flags.process_neuron_firing(model_is_firing);

        // The modifiers of the outgoing potential only apply to a neuron that is firing. One that
        // is not keeps what the model left it, as that is the charge it starts its next burst with
        if !is_firing {
            return;
        }

        if !cortical_flags.is_psp_using_neuron_potential() {
            *processing_buffer = *cortical_level_psp;
        }

        // We divide out the outgoing potential when PSP Uniformity is DISABLED!!!!!!!!!!!
        if !cortical_flags.is_psp_uniform() {
            *processing_buffer *= CorticalAreaNeuronPotential::new(neuron_inversed_outgoing_connection_count.deref());
        }
    }
}