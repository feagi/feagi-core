//! These structs are on all cortical areas

use feagi_basis::feagi_basis_proc_common::bit_struct_builder;



bit_struct_builder! {
    u8,
    pub CorticalAreaUbiquitousFlags,
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
    /// Cortical Area Level Boolean Flags. Has genome properties, but also scratch data used during
    /// processing. Used during neuron dynamics, neuron post processing, intra-neuron-synapse phase.
    /// Only modified by configuration / genome logic
    {
        is_psp_uniform, // Cortical property, read during neuron processing phase,
        is_psp_using_membrane_potential, // Cortical property, read during neuron processing phase
        has_neuron_postprocessors, // set by calculation, read during neuron postprocessing
        has_neuron_spike_trains, // set by calculation, read during intra-neuron-synapse phase
        is_visualizer_active, // Cortical property or config set, read during intra-neuron-synapse phase
    }
}