use core::cmp::min;
#[cfg(feature = "alloc")]
use feagi_basis::collections::quant_indexed::quant_indexed::QuantIndexedVector;
use feagi_basis::feagi_basis_proc_common::bit_struct_builder;
#[cfg(feature = "alloc")]
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_basis::prelude::*;
#[cfg(feature = "alloc")]
use neural_basis::neuron_model::NeuronModel;
#[cfg(feature = "alloc")]
use neural_basis::neuron_model::wrapped_indexing::{CorticalAreaEngineIndex, NeuronEngineIndex, CorticalAreaQuantIndex, NeuronModelNeuronQuantIndex};
use neural_basis::quantization_levels::FeagiIndexQuantization;

//region Parallel Structs
// These hold heap allocated `QuantIndexedVector`s, so need alloc

#[cfg(feature = "alloc")]
pub(crate) struct EnginePerCorticalData<FIQ: FeagiIndexQuantization> {
    pub cortical_flags: QuantIndexedVector<CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>, CorticalAreaUbiquitousFlags>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> EnginePerCorticalData<FIQ> {
    pub fn new() -> Self {
        Self {
            cortical_flags: QuantIndexedVector::default(),
        }
    }
}

#[cfg(feature = "alloc")]
pub(crate) struct ModelQuantPerCorticalDataType<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    pub cortical_level_psp:
        QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, CorticalAreaNeuronPotential<NM::MembranePotentialQuant>>,
    pub model_properties: QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, NM::CorticalDataProperties>,
    pub model_internal: QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, NM::CorticalDataInternal>,
    pub model_scratch: QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, NM::CorticalDataScratch>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization, NM: NeuronModel> ModelQuantPerCorticalDataType<FIQ, NM> {
    pub fn new() -> Self {
        Self {
            cortical_level_psp: QuantIndexedVector::default(),
            model_properties: QuantIndexedVector::default(),
            model_internal: QuantIndexedVector::default(),
            model_scratch: QuantIndexedVector::default(),
        }
    }
}

#[cfg(feature = "alloc")]
pub(crate) struct EnginePerNeuronData<FIQ: FeagiIndexQuantization> {
    pub neuron_model_and_quant: QuantIndexedVector<NeuronEngineIndex<FIQ::NeuronIndexQuant>, ()>,
    pub cortical_index: QuantIndexedVector<NeuronEngineIndex<FIQ::NeuronIndexQuant>, CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,
    pub neuron_flags: QuantIndexedVector<NeuronEngineIndex<FIQ::NeuronIndexQuant>, PerNeuronUbiquitousFlags>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> EnginePerNeuronData<FIQ> {
    pub fn new() -> Self {
        Self {
            neuron_model_and_quant: QuantIndexedVector::default(),
            cortical_index: QuantIndexedVector::default(),
            neuron_flags: QuantIndexedVector::default(),
        }
    }
}

#[cfg(feature = "alloc")]
pub(crate) struct ModelQuantPerNeuronDataType<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    pub membrane_potentials:
        QuantIndexedVector<NeuronModelNeuronQuantIndex<FIQ::NeuronIndexQuant>, CorticalAreaNeuronPotential<NM::MembranePotentialQuant>>,
    pub inverse_connection_count:
        QuantIndexedVector<NeuronModelNeuronQuantIndex<FIQ::NeuronIndexQuant>, InverseOutgoingConnectionCount<NM::MembranePotentialQuant>>,
    pub model_internal: QuantIndexedVector<NeuronModelNeuronQuantIndex<FIQ::NeuronIndexQuant>, NM::NeuronDataInternal>,
    pub model_scratch: QuantIndexedVector<NeuronModelNeuronQuantIndex<FIQ::NeuronIndexQuant>, NM::NeuronDataScratch>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization, NM: NeuronModel> ModelQuantPerNeuronDataType<FIQ, NM> {
    pub fn new() -> Self {
        Self {
            membrane_potentials: QuantIndexedVector::default(),
            inverse_connection_count: QuantIndexedVector::default(),
            model_internal: QuantIndexedVector::default(),
            model_scratch: QuantIndexedVector::default(),
        }
    }
}
//endregion

//region Singular Datas

//region Flags

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


//endregion

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

//endregion
