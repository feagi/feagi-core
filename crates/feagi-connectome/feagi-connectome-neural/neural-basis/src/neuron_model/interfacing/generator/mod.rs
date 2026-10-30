use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::data::neuron_layout::NeuronLayout;
use crate::neuron_model::generators::cortical_storage_loader::CorticalScratchGenerator;
use crate::neuron_model::interfacing::shared::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronLayoutType, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


/// Contains the data and logic to generate a full cortical area
pub struct NeuronModelCorticalAreaGenerator<FIQ, NM, NDG>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NDG: NeuronModelNeuronDataGenerator<NM>
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub cortical_neuron_layout: NeuronLayoutOf<NM, FIQ>,
    
    /// Cortical level properties required by all neuron models
    pub universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: NM::CorticalDataProperties,
    /// Stateful data that must be kept but not exposed to the genome developer for editing.
    /// May update every burst
    pub cortical_internal: NM::CorticalDataInternal,
    /// Runtime cortical level data, not saved. May update every burst.
    pub cortical_scratch: NM::CorticalDataScratch,
    /// Not direct properties, but rather parameters used by functions to set per neuron variance
    pub neuron_variance_properties: NM::CorticalFunctionContext, // TODO
    
    /// Contains logic for generating the neuron data
    pub neuron_generator: NDG
}

/// Variable trait for generating neurons. There may be unique configs variant across the 
/// area, hence a trait to allow various logics
pub trait NeuronModelNeuronDataGenerator<NM>: Sized + Send
where
    NM: NeuronModel<LayoutType=Self::SupportedNeuronLayoutType>
{
    /// What Neuron Layout is supported
    type SupportedNeuronLayoutType: NeuronLayoutType;

    /// Get the neuron layout that will be used
    fn get_neuron_layout<FIQ: FeagiIndexQuantization>(&self)
        -> <Self::SupportedNeuronLayoutType as NeuronLayoutType>::Layout<FIQ>;
    
    /// A function that returns what the neuron data should be at a given index. Also runs the function for
    /// the cortical level neuron modifier contexts
    fn generate_neuron_data_at<FIQ: FeagiIndexQuantization>(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch);


    //region Default impls

    /// Number of neurons needed in the cortical area
    fn get_total_number_neurons_needed<FIQ: FeagiIndexQuantization>(&self)
        -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        self.get_neuron_layout::<FIQ>().get_neuron_count()
    }

    /// Use as an iterator to generate neuron values without needing to collect it all first
    fn iter<FIQ: FeagiIndexQuantization>(&self) 
        -> impl ExactSizeIterator<Item = (NM::NeuronDataInternal, NM::NeuronDataScratch)> + DoubleEndedIterator + '_
    {
        // NOTE: At this time rusts iter step trait is still nightly, hence this nonsense
        let range = 0 .. self.get_total_number_neurons_needed::<FIQ>().quant_to_usize();
        range.map(move |i| self.generate_neuron_data_at::<FIQ>(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i)))
    }

    /// Writes the generated neuron data into preallocated slices, one element per neuron
    fn write_into_slice<FIQ: FeagiIndexQuantization>(
        &self,
        internal: &mut [NM::NeuronDataInternal],
        scratch: &mut [NM::NeuronDataScratch],
    ) {
        let neuron_count = self.get_total_number_neurons_needed::<FIQ>().quant_to_usize();
        assert_eq!(internal.len(), neuron_count);
        assert_eq!(scratch.len(), neuron_count);
        for (i, (internal_out, scratch_out)) in internal.iter_mut().zip(scratch.iter_mut()).enumerate() {
            (*internal_out, *scratch_out) = self.generate_neuron_data_at::<FIQ>(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i));
        }
    }

    #[cfg(feature = "rayon")]
    fn par_iter<LSG: CorticalScratchGenerator<FIQ, NM>>(&self) 
        -> impl rayon::iter::IndexedParallelIterator<Item = (NM::NeuronDataInternal, NM::NeuronDataScratch)> + '_
    where
        Self: Sync,
        NM::NeuronDataInternal: Send,
        NM::NeuronDataScratch: Send,
    {
        use rayon::prelude::*;
        let range = 0 .. self.get_total_number_neurons_needed::<FIQ>().quant_to_usize();
        range.into_par_iter().map(move |i| self.generate_neuron_data_at::<LSG>(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i)))
    }

    /// Parallel version of [`Self::write_into_slice`]
    #[cfg(feature = "rayon")]
    fn par_write_into_slice<LSG: CorticalScratchGenerator<FIQ, NM>>(
        &self,
        internal: &mut [NM::NeuronDataInternal],
        scratch: &mut [NM::NeuronDataScratch],
    )
    where
        Self: Sync,
        NM::NeuronDataInternal: Send,
        NM::NeuronDataScratch: Send,
    {
        use rayon::prelude::*;
        let neuron_count = self.get_total_number_neurons_needed::<FIQ>().quant_to_usize();
        assert_eq!(internal.len(), neuron_count);
        assert_eq!(scratch.len(), neuron_count);
        internal.par_iter_mut().zip(scratch.par_iter_mut()).enumerate().for_each(|(i, (internal_out, scratch_out))| {
            (*internal_out, *scratch_out) = self.generate_neuron_data_at::<FIQ>(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i));
        });
    }

    //endregion
    
}