use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::data::neuron_layout::NeuronLayout;
use crate::neuron_model::interfacing::shared::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronLayoutType, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Contains the data and logic to edit a cortical area
pub struct NeuronModelCorticalAreaEditor<FIQ, NM, NDG, >
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NDG: NeuronModelNeuronDataEditor<FIQ, NM>
{
    /// Details on how neurons are laid out (length, dimensions, etc).
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
}


/// Variable trait for editing neurons. There may be unique configs variant across the 
/// area, hence a trait to allow various logics
pub trait NeuronModelNeuronDataEditor<FIQ, NM>: Sized + Send
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel<LayoutType=Self::SupportedNeuronLayoutType>,
{
    /// What Neuron Layout is supported
    type SupportedNeuronLayoutType: NeuronLayout<FIQ>;
    
    /// Used to generate the indexes that will be edited
    type IndexesIterator: NeuronModelEditingIndexes<FIQ,NeuronLayout=Self::SupportedNeuronLayoutType>;

    /// Get the neuron layout that will be used
    fn get_neuron_layout(&self) 
        -> <Self::SupportedNeuronLayoutType as NeuronLayoutType>::Layout<FIQ>;
    
    
    fn edit_internal_neuron_data_inplace_at(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        existing: &mut NM::NeuronDataInternal
    );
    
    fn edit_internal_neuron_data_at(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        existing: &NM::NeuronDataInternal
    ) -> NM::NeuronDataInternal;
    
    
    fn edit_scratch_neuron_data_inplace_at(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        existing: &mut NM::NeuronDataScratch
    );
    
    fn edit_scratch_neuron_data_at(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        existing: &NM::NeuronDataScratch
    ) -> NM::NeuronDataScratch;
    

    //region Default impls


    
    fn iter_edit_internal_data(&self, editing_slice: &mut [NM::NeuronDataInternal]) // TODO error checking (iterator out of bounds?)
    {
        let index_iterator
            = Self::IndexesIterator::neuron_indexes_editing(&self.get_neuron_layout());
        _ = index_iterator.map(|index| {
            let current_data = &mut editing_slice[index];
            self.edit_internal_neuron_data_inplace_at(
                &CorticalAreaNeuronLocalIndex::<FIQ::NeuronIndexQuant>::quant_from_usize_unchecked(index),
                current_data
            );
        });
    }

    fn iter_edit_scratch_data(&self, editing_slice: &mut [NM::NeuronDataScratch]) // TODO error checking (iterator out of bounds?)
    {
        let index_iterator
            = Self::IndexesIterator::neuron_indexes_editing(&self.get_neuron_layout());
        _ = index_iterator.map(|index| {
            let current_data = &mut editing_slice[index];
            self.edit_scratch_neuron_data_inplace_at(
                &CorticalAreaNeuronLocalIndex::<FIQ::NeuronIndexQuant>::quant_from_usize_unchecked(index),
                current_data
            );
        });
    }


    // NOTE: No slice stuff as this may operate on sparse actions

    #[cfg(feature = "rayon")]
    fn par_iter(&self)
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
    
    //endregion

    // TODO extension with ExactSizeIterator that allows for rayon
}

/// Used for getting an iterator of what indexes we are editing. Zero Sized
pub trait NeuronModelEditingIndexes<FIQ: FeagiIndexQuantization>: Sized {
    
    type NeuronLayout: NeuronLayout<FIQ>;

    /// What neuron indexes are being edited TODO make a parameter
    fn neuron_indexes_editing(neuron_layout: &Self::NeuronLayout) 
        -> impl Iterator<Item=usize>; // Outputting using due to using this for indexing
}

// TODO extension with ExactSizeIterator that allows for rayon


