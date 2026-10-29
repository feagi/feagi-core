use core::cmp::min;
use feagi_basis::prelude::*;
use feagi_basis::feagi_basis_proc_common::bit_struct_builder;
use neural_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

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
        let number_connections = min(FIQ::SynapseIndexCountQuant::QUANT_ONE, number_connections);
        let as_decimal: Q = Q::from_quantized_unsigned_integer(number_connections);
        Self(as_decimal.reciprocal())
    }
}


/// Within a single byte, stores various boolean flags about the configuration of post potential
/// neuron behavior, such as overrides of firing behavior and logic. It also carries a single
/// final result bit for if the neuron is firing, which is used to retain this context between
/// burst phases. This is written to by config change (rarely) and during neuron dynamics phase,
/// and read during cortical post processing phase
pub struct PerNeuronUbiquitousFlags(u8);

impl PerNeuronUbiquitousFlags {
    // Free flags for 0, 1, 2, 3, 4

    const MASK_PROBE_IS_PRESSING_FIRE: u8 = 5;
    const MASK_PROBE_IS_REPRESSING_FIRE: u8 = 6;
    const MASK_IS_FIRING_RESULT: u8 = 7;

    /// Sets the neuron to forcibly fire (despite all other parameters or contexts), but only
    /// lasts a single burst before being reset
    pub fn force_fire_once(&mut self) {
        self.turn_on_bit(Self::MASK_IS_FIRING_RESULT);
    }

    /// If set to true, any firing output from the neuron will be ignored and the neuron will always
    /// fire
    pub fn set_probe_is_pressing_fire(&mut self, probe_pressing_fire: bool) {
        if probe_pressing_fire {
            self.turn_on_bit(Self::MASK_PROBE_IS_PRESSING_FIRE)
        } else {
            self.turn_off_bit(Self::MASK_PROBE_IS_PRESSING_FIRE)
        }
    }

    /// If set to true, any firing output from the neuron will be ignored and the neuron will not
    /// fire. This is however overridden by both force_fire and probe_setting_fire
    pub fn set_probe_is_repressing_fire(&mut self, probe_repressing_fire: bool) {
        if probe_repressing_fire {
            self.turn_on_bit(Self::MASK_PROBE_IS_REPRESSING_FIRE)
        } else {
            self.turn_off_bit(Self::MASK_PROBE_IS_REPRESSING_FIRE)
        }
    }

    /// Given neuron model firing result, calculates if the final psp is of firing or not, stores 
    /// it and returns it as well
    pub fn process_neuron_firing(&mut self, is_neuron_model_firing: bool) -> bool {
        let mut is_firing = is_neuron_model_firing & !self.is_bit(Self::MASK_PROBE_IS_REPRESSING_FIRE);
        is_firing |= self.is_bit(Self::MASK_PROBE_IS_PRESSING_FIRE);
        if is_firing {
            self.turn_on_bit(Self::MASK_IS_FIRING_RESULT);
        }
        self.is_bit(Self::MASK_IS_FIRING_RESULT) // get final bit without resetting
    }

    /// Following the firing bit being reqd, turn it off to not let the neuron be stuck
    pub fn reset_is_firing_bit(&mut self) -> () {
        self.turn_off_bit(Self::MASK_IS_FIRING_RESULT);
    }

    /// Get if neuron is firing, resets the neuron firing state
    pub fn get_neuron_firing(&mut self) -> bool {
        let out = self.is_bit(Self::MASK_IS_FIRING_RESULT);
        self.turn_off_bit(Self::MASK_IS_FIRING_RESULT);
        out
    }

    #[inline(always)]
    fn is_bit(&self, bit: u8) -> bool {
        debug_assert!(bit < 8);
        (self.0 & bit) != 0
    }

    #[inline(always)]
    fn turn_off_bit(&mut self, bit: u8) {
        debug_assert!(bit < 8);
        self.0 &= !bit;
    }

    #[inline(always)]
    fn turn_on_bit(&mut self, bit: u8) {
        debug_assert!(bit < 8);
        self.0 |= bit;
    }
}