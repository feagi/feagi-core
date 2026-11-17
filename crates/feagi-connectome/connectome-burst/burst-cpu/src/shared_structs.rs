use std::cmp::max;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::create_wrapped_quantized_decimal;
use feagi_basis::feagi_basis_proc_common::bit_struct_builder;
use feagi_basis::prelude::{QuantizedDecimalUnwrappedTrait, QuantizedElementBase};

//region Cortical Area

bit_struct_builder! {
    u8,
    pub CorticalAreaUbiquitousFlags,
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
    /// Cortical Area Level Boolean Flags. Has genome properties, but also scratch data used during
    /// processing. Used during neuron dynamics, neuron post processing, intra-neuron-synapse phase.
    /// Only modified by configuration / genome logic
    {
        is_psp_uniform, // Cortical property, read during neuron processing phase,
        is_psp_using_neuron_potential, // Cortical property, read during neuron processing phase
        has_neuron_postprocessors, // set by calculation, read during neuron postprocessing
        has_neuron_spike_trains, // set by calculation, read during intra-neuron-synapse phase
        is_visualizer_active, // Cortical property or config set, read during intra-neuron-synapse phase
    }
}

//endregion

//region Neuron

bit_struct_builder! {
    u8,
    pub PerNeuronUbiquitousFlags,
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
    /// Within a single byte, stores various boolean flags about the configuration of post potential
    /// neuron behavior, such as overrides of firing behavior and logic. It also carries a single
    /// final result bit for if the neuron is firing, which is used to retain this context between
    /// burst phases. This is written to by config change (rarely) and during neuron dynamics phase,
    /// and read during cortical post processing phase
    {
        is_getting_incoming_fire, // 0
        is_firing, // 1
        is_probe_repressing_fire, // 2
        is_probe_forcing_fire, // 3
        is_psp_uniform, // 4
        is_psp_using_neuron_potential, // 5
    }
}

impl PerNeuronUbiquitousFlags {

    /// Given neuron model firing result, calculates if the final psp is of firing or not, stores
    /// it and returns it as well
    pub fn process_neuron_firing(&mut self, is_neuron_model_firing: bool) -> bool {
        let mut is_firing = is_neuron_model_firing & !self.is_probe_repressing_fire();
        is_firing |= self.is_probe_forcing_fire();
        self.set_is_firing(is_firing);
        is_firing
    }
}


create_wrapped_quantized_decimal!(
    /// When psp uniformity is DISABLED, we divide the membrane potential by the number of outgoing
    /// connection, for each connection. However, since storing the connection count as an int,
    /// then turning it into a float, then dividing with it is slow, here we precompute that into
    /// this struct. This is read during neuron dynamics phase, modified by external tuning
    // TODO This may actually be bad behavior, as synapses that slowly die will result in sudden
    // jumps in sharing ratios
    pub InverseOutgoingConnectionCount
);

impl<Q: QuantizedDecimalUnwrappedTrait> InverseOutgoingConnectionCount<Q> {
    /// Defines a new inverse count from a number of connections uint. If 0 is supplied, it rounds
    /// up to one
    pub fn new_from_count<FIQ: FeagiIndexQuantization>(number_connections: FIQ::SynapseIndexCountQuant) -> Self {
        // We don't want to end up dividing by 0
        let number_connections = max(FIQ::SynapseIndexCountQuant::QUANT_ONE, number_connections);
        let as_decimal: Q = Q::from_quantized_unsigned_integer(number_connections);
        Self(as_decimal.reciprocal())
    }
}

//endregion