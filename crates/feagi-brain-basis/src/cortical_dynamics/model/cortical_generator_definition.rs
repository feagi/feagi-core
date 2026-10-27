use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::cortical_dynamics::model::components::neuron_layout::NeuronLayout;
use crate::cortical_dynamics::model::cortical_dynamics_model::CorticalDynamicsModel;
use crate::cortical_dynamics::model::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use crate::cortical_dynamics::model::implementations::core_power::dynamics::CorticalDynamicsQuantizationCorePowerFloat32;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// TODO move to NPU




/// Defines generating a new cortical area, to be taken in by any burst engine type
pub trait CorticalGeneratorDefinition<FIQ, NL, CDQ>: Send
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>, // will likely be restricted by implementation
    CDQ: CorticalDynamicsQuantization
{
    /// What cortical model this will write tto
    type CorticalModel: CorticalDynamicsModel<CDQ>;

    /// Get the neuron layout that this cortical area is using
    fn get_neuron_layout(&self) -> &NL;

    /// Dump self and get the cortical level data needed for the cortical area
    fn get_cortical_level_data(self) -> (
        <Self::CorticalModel as CorticalDynamicsModel<CDQ>>::CorticalDataProperties,
        <Self::CorticalModel as CorticalDynamicsModel<CDQ>>::CorticalDataInternal,
        <Self::CorticalModel as CorticalDynamicsModel<CDQ>>::CorticalDataScratch,
    ); // Do not return function context as its not actually intended to be written, just used for neuron generation

    /// A function that returns what the neuron data should be at a given index. Also runs the function for
    /// the cortical level neuron modifier contexts
    fn generate_neuron_data_at(&self, index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>)
        -> PerNeuronData<Self::CorticalModel, CDQ>;

    //TODO into enum

    //region Default impls

    /// Number of neurons needed in the cortical area
    fn get_total_number_neurons_needed(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        let layout = self.get_neuron_layout();
        layout.get_neuron_count()
    }

    fn iter(&self)
        -> impl ExactSizeIterator<Item = PerNeuronData<Self::CorticalModel, CDQ>> + DoubleEndedIterator + '_
    {
        // NOTE: At this time rusts iter step trait is still nightly, hence this nonsense
        let range = 0 .. self.get_total_number_neurons_needed().quant_to_usize();
        range.map(move |i| self.generate_neuron_data_at(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i)))
    }

    /// Writes the generated neuron data into preallocated slices, one element per neuron
    fn write_into_slice(
        &self,
        internal: &mut [<Self::CorticalModel as CorticalDynamicsModel<CDQ>>::NeuronDataInternal],
        scratch: &mut [<Self::CorticalModel as CorticalDynamicsModel<CDQ>>::NeuronDataScratch],
    ) {
        let neuron_count = self.get_total_number_neurons_needed().quant_to_usize();
        assert_eq!(internal.len(), neuron_count);
        assert_eq!(scratch.len(), neuron_count);
        for (i, (internal_out, scratch_out)) in internal.iter_mut().zip(scratch.iter_mut()).enumerate() {
            (*internal_out, *scratch_out) = self.generate_neuron_data_at(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i));
        }
    }

    #[cfg(feature = "rayon")]
    fn par_iter(&self)
        -> impl rayon::iter::IndexedParallelIterator<Item = PerNeuronData<Self::CorticalModel, CDQ>> + '_
    where
        Self: Sync,
        PerNeuronData<Self::CorticalModel, CDQ>: Send,
    {
        use rayon::prelude::*;
        let range = 0 .. self.get_total_number_neurons_needed().quant_to_usize();
        range.into_par_iter().map(move |i| self.generate_neuron_data_at(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i)))
    }

    /// Parallel version of [`Self::write_into_slice`]
    #[cfg(feature = "rayon")]
    fn par_write_into_slice(
        &self,
        internal: &mut [<Self::CorticalModel as CorticalDynamicsModel<CDQ>>::NeuronDataInternal],
        scratch: &mut [<Self::CorticalModel as CorticalDynamicsModel<CDQ>>::NeuronDataScratch],
    )
    where
        Self: Sync,
        <Self::CorticalModel as CorticalDynamicsModel<CDQ>>::NeuronDataInternal: Send,
        <Self::CorticalModel as CorticalDynamicsModel<CDQ>>::NeuronDataScratch: Send,
    {
        use rayon::prelude::*;
        let neuron_count = self.get_total_number_neurons_needed().quant_to_usize();
        assert_eq!(internal.len(), neuron_count);
        assert_eq!(scratch.len(), neuron_count);
        internal.par_iter_mut().zip(scratch.par_iter_mut()).enumerate().for_each(|(i, (internal_out, scratch_out))| {
            (*internal_out, *scratch_out) = self.generate_neuron_data_at(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i));
        });
    }

    //endregion
}

/// The neuron data produced for each neuron
type PerNeuronData<CDM: CorticalDynamicsModel<CDQ>, CDQ: CorticalDynamicsQuantization> = (
    CDM::NeuronDataInternal,
    CDM::NeuronDataScratch,
);