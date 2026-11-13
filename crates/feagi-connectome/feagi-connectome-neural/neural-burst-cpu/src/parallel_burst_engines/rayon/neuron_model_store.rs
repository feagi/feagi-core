//! The neurons of one neuron model at one quantization, as the rayon CPU engine holds and bursts
//! them, and the list of such stores the engine is built over

use crate::parallel_burst_engines::neural_cpu_parallel_dynamics::NeuronCPUParallelDynamics;
use crate::shared_structs::{
    CorticalAreaUbiquitousFlags, CpuAddCorticalAreaError, CpuAddedCorticalArea, CpuEngineNeuronData, CpuMembranePotentialQuant, IncomingStagingRange,
    InverseOutgoingConnectionCount, ModelQuantPerCorticalDataType, ModelQuantPerNeuronDataType, NeuronMembranePotentialIndex,
};
use ::rayon::prelude::*;
use core::marker::PhantomData;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::*;
use neural_basis::neuron_model::neurogenesis::NeurogenesisMethod;
use neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisStorageOrGenerator;
use neural_basis::neuron_model::neuron_layout::NeuronLayout;
use neural_basis::neuron_model::wrapped_indexing::NeuronModelNeuronQuantIndex;
use neural_basis::neuron_model::NeuronModel;
use neural_basis::quantization_levels::FeagiIndexQuantization;

/// Every neuron model store of a rayon CPU engine, one per neuron model and quantization it can
/// host. The engine does not know the neuron models, so whatever defines them lists a
/// `CpuNeuronModelStore` of each, and relays these calls to them
pub trait CpuNeuronModelStores<FIQ: FeagiIndexQuantization>: Sized {
    /// The neurogenesis of a cortical area, at whichever neuron model and quantization it uses
    type CorticalAreaNeurogenesisDispatch: Send;

    /// Every store, empty
    fn new() -> Self;

    /// Adds the cortical area to the store of its neuron model and quantization
    fn add_cortical_area(
        &mut self,
        neurogenesis: Self::CorticalAreaNeurogenesisDispatch,
        engine: &mut CpuEngineNeuronData<FIQ>,
    ) -> Result<CpuAddedCorticalArea<FIQ>, CpuAddCorticalAreaError>;

    /// Runs the neuron phase of every store
    fn run_neuron_phase(&mut self, burst_index: &BurstIndex<FIQ::BurstIndexQuant>, engine: &mut CpuEngineNeuronData<FIQ>);
}

/// Every cortical area and neuron of the neuron model `NM`, whose dynamics on the CPU are `D`.
/// Membrane potentials and firing flags are not held here but in the `CpuEngineNeuronData`
pub struct CpuNeuronModelStore<FIQ, NM, D>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    per_cortical: ModelQuantPerCorticalDataType<FIQ, NM>,
    per_neuron: ModelQuantPerNeuronDataType<FIQ, NM>,
    _dynamics: PhantomData<fn() -> D>,
}

impl<FIQ, NM, D> CpuNeuronModelStore<FIQ, NM, D>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NM::MembranePotentialQuant: CpuMembranePotentialQuant,
    D: NeuronCPUParallelDynamics<FIQ, NM>,
{
    pub fn new() -> Self {
        Self {
            per_cortical: ModelQuantPerCorticalDataType::new(),
            per_neuron: ModelQuantPerNeuronDataType::new(),
            _dynamics: PhantomData,
        }
    }

    /// How many cortical areas this store holds
    pub fn cortical_area_count(&self) -> usize {
        self.per_cortical.layouts.store().len()
    }

    /// Converts the neurogenesis of a cortical area into this store, and allocates its membrane
    /// potentials, flags and visualization in `engine`. Every neuron starts at rest, not firing,
    /// with nothing leading to or from it
    pub fn add_cortical_area(
        &mut self,
        neurogenesis: ValidatedNeurogenesisStorageOrGenerator<FIQ, NM>,
        engine: &mut CpuEngineNeuronData<FIQ>,
    ) -> Result<CpuAddedCorticalArea<FIQ>, CpuAddCorticalAreaError> {
        let layout = *neurogenesis.peek_layout();
        let universal_properties = neurogenesis.get_universal_properties();
        let neuron_count = neurogenesis.peek_layout().get_number_neurons_to_allocate_for().quant_to_usize();

        let mut cortical_flags = CorticalAreaUbiquitousFlags::default();
        cortical_flags.set_is_psp_uniform(universal_properties.is_psp_uniform);
        cortical_flags.set_is_psp_using_neuron_potential(universal_properties.is_psp_using_neuron_potential);

        // Checks every index limit of the engine. As neither this model nor its membrane potential
        // quantization can hold more neurons than the engine does, nothing below can exceed one
        let added = engine.register_cortical_area(neuron_count, cortical_flags)?;

        // Membrane potentials, per membrane potential quantization
        let membrane_potential_data = <NM::MembranePotentialQuant as CpuMembranePotentialQuant>::data_mut(&mut engine.membrane_potentials);
        let first_membrane_potential = membrane_potential_data.membrane_potentials.store().len();
        membrane_potential_data
            .membrane_potentials
            .store_mut()
            .resize(first_membrane_potential + neuron_count, CorticalAreaNeuronPotential::QUANT_ZERO);
        membrane_potential_data
            .incoming_staging_ranges
            .store_mut()
            .resize(first_membrane_potential + neuron_count, IncomingStagingRange::EMPTY);

        // Per neuron, per model
        let first_model_neuron = self.per_neuron.model_internal.store().len();
        self.per_neuron.model_internal.store_mut().reserve(neuron_count);
        self.per_neuron.model_scratch.store_mut().reserve(neuron_count);
        self.per_neuron.model_variance.store_mut().reserve(neuron_count);
        for (internal, scratch, variance) in neurogenesis.iter() {
            self.per_neuron.model_internal.store_mut().push(internal);
            self.per_neuron.model_scratch.store_mut().push(scratch);
            self.per_neuron.model_variance.store_mut().push(variance);
        }
        self.per_neuron.inverse_connection_count.store_mut().resize(
            first_model_neuron + neuron_count,
            InverseOutgoingConnectionCount::new_from_count::<FIQ>(<FIQ::SynapseIndexCountQuant as QuantizedElementBase>::QUANT_ZERO),
        );

        // Per cortical area, per model
        let (cortical_properties, cortical_internal) = neurogenesis.into_model_cortical_data();
        let cortical_scratch = NM::generate_cortical_scratch::<FIQ>(&layout, &cortical_properties, &cortical_internal);
        self.per_cortical.cortical_level_psp.store_mut().push(universal_properties.cortical_level_psp);
        self.per_cortical.model_properties.store_mut().push(cortical_properties);
        self.per_cortical.model_internal.store_mut().push(cortical_internal);
        self.per_cortical.model_scratch.store_mut().push(cortical_scratch);
        self.per_cortical.layouts.store_mut().push(layout);
        self.per_cortical.cortical_engine_index.store_mut().push(added.cortical_index);
        self.per_cortical.neuron_count.store_mut().push(CorticalAreaNeuronCount::quant_from_usize_unchecked(neuron_count));
        self.per_cortical
            .first_neuron_model_index
            .store_mut()
            .push(NeuronModelNeuronQuantIndex::quant_from_usize_unchecked(first_model_neuron));
        self.per_cortical
            .first_neuron_membrane_potential_index
            .store_mut()
            .push(NeuronMembranePotentialIndex::quant_from_usize_unchecked(first_membrane_potential));

        Ok(added)
    }

    /// Runs the neuron phase of every cortical area of this store, one area after another, the
    /// neurons of an area in parallel. Per neuron:
    ///
    /// 1. Starts from zero if it fired last burst, as its slot then holds the potential it sent
    ///    out. Otherwise starts from its slot, the charge it retained
    /// 2. Adds every incoming potential staged for it, and zeroes that staging
    /// 3. Runs the dynamics of the model, then stores if it is firing in its flags. Only a neuron
    ///    that is firing gets the outgoing potential modifiers applied to its slot
    pub fn run_neuron_phase(&mut self, burst_index: &BurstIndex<FIQ::BurstIndexQuant>, engine: &mut CpuEngineNeuronData<FIQ>) {
        let membrane_potential_data = <NM::MembranePotentialQuant as CpuMembranePotentialQuant>::data_mut(&mut engine.membrane_potentials);
        let incoming_staging = IncomingStagingAccess(membrane_potential_data.incoming_staging.store_mut().as_mut_ptr());

        for area in 0..self.cortical_area_count() {
            let layout = &self.per_cortical.layouts.store()[area];

            D::process_model_cortical_area_dynamics(
                burst_index,
                layout,
                (
                    &self.per_cortical.model_properties.store()[area],
                    &mut self.per_cortical.model_internal.store_mut()[area],
                    &mut self.per_cortical.model_scratch.store_mut()[area],
                ),
            );

            let cortical_data = (
                &self.per_cortical.model_properties.store()[area],
                &self.per_cortical.model_internal.store()[area],
                &self.per_cortical.model_scratch.store()[area],
            );
            let cortical_level_psp = self.per_cortical.cortical_level_psp.store()[area];
            let cortical_engine_index = self.per_cortical.cortical_engine_index.store()[area];
            let cortical_flags = engine.per_cortical.cortical_flags[cortical_engine_index];

            // The neurons of an area are contiguous in every vector holding them
            let neuron_count = self.per_cortical.neuron_count.store()[area].quant_to_usize();
            let model_neurons = {
                let first = self.per_cortical.first_neuron_model_index.store()[area].quant_to_usize();
                first..first + neuron_count
            };
            let membrane_potential_neurons = {
                let first = self.per_cortical.first_neuron_membrane_potential_index.store()[area].quant_to_usize();
                first..first + neuron_count
            };
            let engine_neurons = {
                let first = engine.per_cortical.first_neuron_engine_index[cortical_engine_index].quant_to_usize();
                first..first + neuron_count
            };

            (
                &mut membrane_potential_data.membrane_potentials.store_mut()[membrane_potential_neurons.clone()],
                &membrane_potential_data.incoming_staging_ranges.store()[membrane_potential_neurons],
                &mut self.per_neuron.model_internal.store_mut()[model_neurons.clone()],
                &mut self.per_neuron.model_scratch.store_mut()[model_neurons.clone()],
                &self.per_neuron.model_variance.store()[model_neurons.clone()],
                &self.per_neuron.inverse_connection_count.store()[model_neurons],
                &mut engine.per_neuron.neuron_flags.store_mut()[engine_neurons],
            )
                .into_par_iter()
                .enumerate()
                .for_each(
                    |(neuron, (membrane_potential, incoming_range, neuron_internal, neuron_scratch, neuron_variance, inverse_connection_count, neuron_flags))| {
                        let mut potential = match neuron_flags.is_firing() {
                            true => CorticalAreaNeuronPotential::QUANT_ZERO,
                            false => *membrane_potential,
                        };

                        let first_slot = incoming_range.start.quant_to_usize();
                        for slot in first_slot..first_slot + incoming_range.length.quant_to_usize() {
                            // SAFETY: the staging ranges of neurons never overlap, and are within
                            // the staging, so only this neuron reaches this slot. Nothing else
                            // borrows the staging while the neuron phase runs
                            let staged = unsafe { incoming_staging.slot(slot) };
                            potential += *staged;
                            *staged = CorticalAreaNeuronPotential::QUANT_ZERO;
                        }
                        *membrane_potential = potential;

                        D::process_neuron_dynamics_for_psp(
                            membrane_potential,
                            burst_index,
                            layout,
                            &CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(neuron),
                            cortical_data,
                            (neuron_internal, neuron_scratch, neuron_variance),
                            &cortical_flags,
                            &cortical_level_psp,
                            inverse_connection_count,
                            neuron_flags,
                        );
                    },
                );
        }
    }
}

impl<FIQ, NM, D> Default for CpuNeuronModelStore<FIQ, NM, D>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NM::MembranePotentialQuant: CpuMembranePotentialQuant,
    D: NeuronCPUParallelDynamics<FIQ, NM>,
{
    fn default() -> Self {
        Self::new()
    }
}

/// The incoming potential staging of a membrane potential quantization, reached by slot from the
/// neurons of an area in parallel
struct IncomingStagingAccess<Q: QuantizedDecimalUnwrappedTrait>(*mut CorticalAreaNeuronPotential<Q>);

// SAFETY: only hands out slots through `slot`, whose caller upholds that no slot is reached twice
unsafe impl<Q: QuantizedDecimalUnwrappedTrait> Sync for IncomingStagingAccess<Q> {}

impl<Q: QuantizedDecimalUnwrappedTrait> IncomingStagingAccess<Q> {
    /// SAFETY: `slot` must be within the staging, and not reached from anywhere else at this time
    #[allow(clippy::mut_from_ref)]
    unsafe fn slot(&self, slot: usize) -> &mut CorticalAreaNeuronPotential<Q> {
        &mut *self.0.add(slot)
    }
}
