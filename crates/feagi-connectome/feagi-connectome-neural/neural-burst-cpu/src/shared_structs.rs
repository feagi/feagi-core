use core::cmp::max;
#[cfg(feature = "alloc")]
use feagi_basis::collections::bit_packed_bool::par_data::BitBatchParData;
#[cfg(feature = "alloc")]
use feagi_basis::collections::quant_indexed::quant_indexed::QuantIndexedVector;
use feagi_basis::feagi_basis_proc_common::bit_struct_builder;
#[cfg(feature = "alloc")]
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronPotential};
use feagi_basis::prelude::*;
#[cfg(feature = "alloc")]
use half::{bf16, f16};
#[cfg(feature = "alloc")]
use neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
#[cfg(feature = "alloc")]
use neural_basis::neuron_model::NeuronModel;
#[cfg(feature = "alloc")]
use neural_basis::neuron_model::wrapped_indexing::{CorticalAreaEngineIndex, NeuronEngineIndex, CorticalAreaQuantIndex, NeuronModelNeuronQuantIndex};
use neural_basis::quantization_levels::FeagiIndexQuantization;
#[cfg(feature = "alloc")]
use neural_basis::visualization::{BurstVisualizationData, NeuronVisualizationVoxelIndex, NeuronVisualizationWordIndex, VisualizationStructureChangeCounter, VoxelBitPacked};

//region Parallel Structs
// These hold heap allocated `QuantIndexedVector`s, so need alloc

//region Engine wide (independent of neuron model and quantization)

/// Per cortical area data that does not depend on the area's neuron model. Every vector is indexed
/// by `CorticalAreaEngineIndex`
#[cfg(feature = "alloc")]
pub struct EnginePerCorticalData<FIQ: FeagiIndexQuantization> {
    pub cortical_flags: QuantIndexedVector<CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>, CorticalAreaUbiquitousFlags>,
    /// The engine index of the first neuron of the area. Its neurons are contiguous from there
    pub first_neuron_engine_index: QuantIndexedVector<CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>, NeuronEngineIndex<FIQ::NeuronIndexQuant>>,
    pub neuron_count: QuantIndexedVector<CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>, CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>>,
    /// The first visualization word of the area. It has one bit per neuron, padded to a whole word
    pub visualization_start_index:
        QuantIndexedVector<CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>, NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> EnginePerCorticalData<FIQ> {
    pub fn new() -> Self {
        Self {
            cortical_flags: QuantIndexedVector::default(),
            first_neuron_engine_index: QuantIndexedVector::default(),
            neuron_count: QuantIndexedVector::default(),
            visualization_start_index: QuantIndexedVector::default(),
        }
    }
}

/// Per neuron data that does not depend on the neuron's model. Every vector is indexed by
/// `NeuronEngineIndex`
#[cfg(feature = "alloc")]
pub struct EnginePerNeuronData<FIQ: FeagiIndexQuantization> {
    pub cortical_index: QuantIndexedVector<NeuronEngineIndex<FIQ::NeuronIndexQuant>, CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,
    /// Holds, among other flags, if the neuron is firing. Synapses only read the membrane
    /// potential of a neuron whose firing flag is set
    pub neuron_flags: QuantIndexedVector<NeuronEngineIndex<FIQ::NeuronIndexQuant>, PerNeuronUbiquitousFlags>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> EnginePerNeuronData<FIQ> {
    pub fn new() -> Self {
        Self {
            cortical_index: QuantIndexedVector::default(),
            neuron_flags: QuantIndexedVector::default(),
        }
    }
}

//endregion

//region Per membrane potential quantization (shared between neurons and synapses)

/// Where the incoming potentials of a single neuron are staged, as a run of slots of
/// `MembranePotentialQuantData::incoming_staging`. Each synapse leading to the neuron owns one slot
/// of the run, so synapses never write to the same place in parallel
#[cfg(feature = "alloc")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IncomingStagingRange<FIQ: FeagiIndexQuantization> {
    pub start: IncomingStagingIndex<FIQ::SynapseAggregatorIndexCountQuant>,
    pub length: FIQ::SynapseAggregatorIndexCountQuant,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> IncomingStagingRange<FIQ> {
    /// Nothing leads to the neuron
    pub const EMPTY: Self = Self {
        start: IncomingStagingIndex::QUANT_ZERO,
        length: <FIQ::SynapseAggregatorIndexCountQuant as QuantizedElementBase>::QUANT_ZERO,
    };
}

/// The membrane potentials of every neuron of one membrane potential quantization, whichever neuron
/// model they are of, and the staging their incoming potentials are summed from
#[cfg(feature = "alloc")]
pub struct MembranePotentialQuantData<FIQ: FeagiIndexQuantization, Q: QuantizedDecimalUnwrappedTrait> {
    /// One slot per neuron. Holds the outgoing potential of a neuron that is firing, otherwise the
    /// charge the neuron retained
    pub membrane_potentials: QuantIndexedVector<NeuronMembranePotentialIndex<FIQ::NeuronIndexQuant>, CorticalAreaNeuronPotential<Q>>,
    /// One per neuron, which slots of `incoming_staging` are summed into it
    pub incoming_staging_ranges: QuantIndexedVector<NeuronMembranePotentialIndex<FIQ::NeuronIndexQuant>, IncomingStagingRange<FIQ>>,
    /// One slot per incoming synapse, grouped by destination neuron. Written by the synapse phase,
    /// then summed and zeroed by the neuron phase
    pub incoming_staging: QuantIndexedVector<IncomingStagingIndex<FIQ::SynapseAggregatorIndexCountQuant>, CorticalAreaNeuronPotential<Q>>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization, Q: QuantizedDecimalUnwrappedTrait> MembranePotentialQuantData<FIQ, Q> {
    pub fn new() -> Self {
        Self {
            membrane_potentials: QuantIndexedVector::default(),
            incoming_staging_ranges: QuantIndexedVector::default(),
            incoming_staging: QuantIndexedVector::default(),
        }
    }
}

/// The membrane potentials of every neuron of the engine, separated by their quantization
#[cfg(feature = "alloc")]
pub struct MembranePotentialData<FIQ: FeagiIndexQuantization> {
    pub float_16: MembranePotentialQuantData<FIQ, f16>,
    pub b_float_16: MembranePotentialQuantData<FIQ, bf16>,
    pub float_32: MembranePotentialQuantData<FIQ, f32>,
    pub float_64: MembranePotentialQuantData<FIQ, f64>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> MembranePotentialData<FIQ> {
    pub fn new() -> Self {
        Self {
            float_16: MembranePotentialQuantData::new(),
            b_float_16: MembranePotentialQuantData::new(),
            float_32: MembranePotentialQuantData::new(),
            float_64: MembranePotentialQuantData::new(),
        }
    }
}

/// A membrane potential quantization the CPU engines hold neurons of. Picks its own data out of
/// `MembranePotentialData` at compile time, so no lookup happens while bursting
#[cfg(feature = "alloc")]
pub trait CpuMembranePotentialQuant: QuantizedDecimalUnwrappedTrait {
    fn data<FIQ: FeagiIndexQuantization>(all: &MembranePotentialData<FIQ>) -> &MembranePotentialQuantData<FIQ, Self>;
    fn data_mut<FIQ: FeagiIndexQuantization>(all: &mut MembranePotentialData<FIQ>) -> &mut MembranePotentialQuantData<FIQ, Self>;
}

#[cfg(feature = "alloc")]
macro_rules! impl_cpu_membrane_potential_quant {
    ($quant:ty, $field:ident) => {
        impl CpuMembranePotentialQuant for $quant {
            fn data<FIQ: FeagiIndexQuantization>(all: &MembranePotentialData<FIQ>) -> &MembranePotentialQuantData<FIQ, Self> {
                &all.$field
            }

            fn data_mut<FIQ: FeagiIndexQuantization>(all: &mut MembranePotentialData<FIQ>) -> &mut MembranePotentialQuantData<FIQ, Self> {
                &mut all.$field
            }
        }
    };
}

#[cfg(feature = "alloc")]
impl_cpu_membrane_potential_quant!(f16, float_16);
#[cfg(feature = "alloc")]
impl_cpu_membrane_potential_quant!(bf16, b_float_16);
#[cfg(feature = "alloc")]
impl_cpu_membrane_potential_quant!(f32, float_32);
#[cfg(feature = "alloc")]
impl_cpu_membrane_potential_quant!(f64, float_64);

//endregion

//region Per neuron model and quantization

/// Per cortical area data of one neuron model at one quantization. Every vector is indexed by
/// `CorticalAreaQuantIndex`
#[cfg(feature = "alloc")]
pub struct ModelQuantPerCorticalDataType<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    pub cortical_level_psp:
        QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, CorticalAreaNeuronPotential<NM::MembranePotentialQuant>>,
    pub model_properties: QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, NM::CorticalDataProperties>,
    pub model_internal: QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, NM::CorticalDataInternal>,
    pub model_scratch: QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, NM::CorticalDataScratch>,
    pub layouts: QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, NeuronLayoutOf<NM, FIQ>>,
    /// Which cortical area of the engine this is
    pub cortical_engine_index:
        QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,
    pub neuron_count: QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>>,
    /// The first neuron of the area within the `ModelQuantPerNeuronDataType` of this model
    pub first_neuron_model_index:
        QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, NeuronModelNeuronQuantIndex<FIQ::NeuronIndexQuant>>,
    /// The first neuron of the area within the `MembranePotentialQuantData` of this model's
    /// membrane potential quantization
    pub first_neuron_membrane_potential_index:
        QuantIndexedVector<CorticalAreaQuantIndex<FIQ::CorticalAreaIndexCountQuant>, NeuronMembranePotentialIndex<FIQ::NeuronIndexQuant>>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization, NM: NeuronModel> ModelQuantPerCorticalDataType<FIQ, NM> {
    pub fn new() -> Self {
        Self {
            cortical_level_psp: QuantIndexedVector::default(),
            model_properties: QuantIndexedVector::default(),
            model_internal: QuantIndexedVector::default(),
            model_scratch: QuantIndexedVector::default(),
            layouts: QuantIndexedVector::default(),
            cortical_engine_index: QuantIndexedVector::default(),
            neuron_count: QuantIndexedVector::default(),
            first_neuron_model_index: QuantIndexedVector::default(),
            first_neuron_membrane_potential_index: QuantIndexedVector::default(),
        }
    }
}

/// Per neuron data of one neuron model at one quantization. Every vector is indexed by
/// `NeuronModelNeuronQuantIndex`. Membrane potentials are not here, they are held per membrane
/// potential quantization in `MembranePotentialData`
#[cfg(feature = "alloc")]
pub struct ModelQuantPerNeuronDataType<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    pub inverse_connection_count:
        QuantIndexedVector<NeuronModelNeuronQuantIndex<FIQ::NeuronIndexQuant>, InverseOutgoingConnectionCount<NM::MembranePotentialQuant>>,
    pub model_internal: QuantIndexedVector<NeuronModelNeuronQuantIndex<FIQ::NeuronIndexQuant>, NM::NeuronDataInternal>,
    pub model_scratch: QuantIndexedVector<NeuronModelNeuronQuantIndex<FIQ::NeuronIndexQuant>, NM::NeuronDataScratch>,
    pub model_variance: QuantIndexedVector<NeuronModelNeuronQuantIndex<FIQ::NeuronIndexQuant>, NM::NeuronDataVariance>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization, NM: NeuronModel> ModelQuantPerNeuronDataType<FIQ, NM> {
    pub fn new() -> Self {
        Self {
            inverse_connection_count: QuantIndexedVector::default(),
            model_internal: QuantIndexedVector::default(),
            model_scratch: QuantIndexedVector::default(),
            model_variance: QuantIndexedVector::default(),
        }
    }
}

//endregion

//region Engine neuron data

/// Everything the CPU engines hold about neurons that does not depend on their neuron model. The
/// per neuron model data is held apart from this, so both can be borrowed while bursting
#[cfg(feature = "alloc")]
pub struct CpuEngineNeuronData<FIQ: FeagiIndexQuantization> {
    pub per_cortical: EnginePerCorticalData<FIQ>,
    pub per_neuron: EnginePerNeuronData<FIQ>,
    pub membrane_potentials: MembranePotentialData<FIQ>,
    /// One bit per neuron, set if it fired. Only up to date after `pack_visualization`
    pub visualization: BurstVisualizationData<FIQ>,
}

/// Where a cortical area added to a CPU engine ended up
#[cfg(feature = "alloc")]
pub struct CpuAddedCorticalArea<FIQ: FeagiIndexQuantization> {
    pub cortical_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub first_neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexQuant>,
    pub visualization_start_index: NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>,
}

/// Why a cortical area could not be added to a CPU engine. Nothing was changed
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CpuAddCorticalAreaError {
    #[error("The engine can not index another cortical area at its index quantization")]
    TooManyCorticalAreas,
    #[error("The engine can not index that many more neurons at its index quantization")]
    TooManyNeurons,
    #[error("The engine can not index that much more visualization at its index quantization")]
    TooMuchVisualization,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> CpuEngineNeuronData<FIQ> {
    /// How many neurons share a visualization word
    pub const NEURONS_PER_VISUALIZATION_WORD: usize = VoxelBitPacked::BITS as usize;

    pub fn new() -> Self {
        Self {
            per_cortical: EnginePerCorticalData::new(),
            per_neuron: EnginePerNeuronData::new(),
            membrane_potentials: MembranePotentialData::new(),
            visualization: BurstVisualizationData::new(),
        }
    }

    /// Allocates everything that does not depend on the neuron model for a new cortical area of
    /// `neuron_count` neurons: its engine wide indexes, neuron flags, and visualization words.
    /// Checks every index limit before changing anything
    pub fn register_cortical_area(
        &mut self,
        neuron_count: usize,
        cortical_flags: CorticalAreaUbiquitousFlags,
    ) -> Result<CpuAddedCorticalArea<FIQ>, CpuAddCorticalAreaError> {
        let cortical_index = CorticalAreaEngineIndex::quant_try_from_usize(self.per_cortical.cortical_flags.store().len())
            .map_err(|_| CpuAddCorticalAreaError::TooManyCorticalAreas)?;

        let first_neuron = self.per_neuron.neuron_flags.store().len();
        let neuron_total = first_neuron.checked_add(neuron_count).ok_or(CpuAddCorticalAreaError::TooManyNeurons)?;
        if neuron_total > FIQ::MAX_NEURON_COUNT {
            return Err(CpuAddCorticalAreaError::TooManyNeurons);
        }
        let first_neuron_engine_index = NeuronEngineIndex::quant_from_usize_unchecked(first_neuron);

        // Each area starts on its own word, so areas never share one
        let first_word = self.visualization.bit_data.store().len();
        let word_total = first_word + neuron_count.div_ceil(Self::NEURONS_PER_VISUALIZATION_WORD);
        let visualization_start_index =
            NeuronVisualizationWordIndex::quant_try_from_usize(first_word).map_err(|_| CpuAddCorticalAreaError::TooMuchVisualization)?;
        let bit_total = word_total
            .checked_mul(Self::NEURONS_PER_VISUALIZATION_WORD)
            .and_then(|bits| NeuronVisualizationVoxelIndex::quant_try_from_usize(bits).ok())
            .ok_or(CpuAddCorticalAreaError::TooMuchVisualization)?;

        // Nothing can fail from here

        self.per_cortical.cortical_flags.store_mut().push(cortical_flags);
        self.per_cortical.first_neuron_engine_index.store_mut().push(first_neuron_engine_index);
        self.per_cortical.neuron_count.store_mut().push(CorticalAreaNeuronCount::quant_from_usize_unchecked(neuron_count));
        self.per_cortical.visualization_start_index.store_mut().push(visualization_start_index);

        self.per_neuron.cortical_index.store_mut().resize(neuron_total, cortical_index);
        self.per_neuron.neuron_flags.store_mut().resize(neuron_total, PerNeuronUbiquitousFlags::default());

        // The padding bits between areas are counted as valid, as they are not trailing
        let mut words = core::mem::take(self.visualization.bit_data.store_mut());
        words.resize(word_total, 0);
        self.visualization.bit_data = BitBatchParData::from_vec(words, bit_total);
        // Listeners must look up where each area is again
        self.visualization.change_counter = VisualizationStructureChangeCounter(self.visualization.change_counter.0.wrapping_add(1));

        Ok(CpuAddedCorticalArea { cortical_index, first_neuron_engine_index, visualization_start_index })
    }
}

//endregion

//endregion

//region Singular Datas

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a neuron within the membrane potentials of its membrane potential
    /// quantization
    pub NeuronMembranePotentialIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a slot of the incoming potential staging of a membrane potential
    /// quantization
    pub IncomingStagingIndex
);

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
        let number_connections = max(FIQ::SynapseIndexCountQuant::QUANT_ONE, number_connections);
        let as_decimal: Q = Q::from_quantized_unsigned_integer(number_connections);
        Self(as_decimal.reciprocal())
    }
}

//endregion
