pub mod from_storage;
pub mod from_storage_or_by_generator;
pub mod by_generator;

use crate::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutTag};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;

/// Shared static generator for initing scratch and variance data
pub trait NeurogenesisScratchVarianceGenerator<FIQ, NM>: Clone + core::fmt::Debug + Send + Sync
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    fn static_get_cortical_scratch(
        layout: &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        cortical_properties: &NM::CorticalDataProperties,
        cortical_internal: &NM::CorticalDataInternal,
    ) -> NM::CorticalDataScratch;

    fn static_get_neuron_scratch(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout: &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        internal_neuron_data: &NM::NeuronDataInternal
    ) -> NM::NeuronDataScratch;

    fn static_get_neuron_variance(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout: &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        variance: &NM::CorticalDataVariance
    ) -> NM::NeuronDataVariance;
}



/// The most generic descriptor for describing how a cortical area gets spawnd
pub trait NeurogenesisMethod<FIQ, NM>: Clone + core::fmt::Debug + Send + Sync
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    
    fn get_layout(&self) -> &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>;

    /// Cortical level data used by all models
    fn get_universal_cortical_properties(&self) -> NeuronModelUniversalCorticalProperties<NM>;

    fn into_model_cortical_data(self) -> (
        NM::CorticalDataProperties,
        NM::CorticalDataInternal,
    );

    fn clone_model_cortical_data(&self)-> (NM::CorticalDataProperties, NM::CorticalDataInternal);

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance;

    fn spawn_neuron_data_at(
        &self, // NOTE: make sure the implementation makes use of variance!
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch);

    /// Number of neurons needed in the cortical area
    fn get_total_number_neurons_needed(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        self.get_layout().get_number_neurons_to_allocate_for()
    }

    /// Use as an iterator to spawn neuron values without needing to collect it all first
    fn iter(&self) -> impl ExactSizeIterator<Item = (NM::NeuronDataInternal, NM::NeuronDataScratch, NM::NeuronDataVariance)> + DoubleEndedIterator + '_ {
        // NOTE: At this time rusts iter step trait is still nightly, hence this nonsense
        let range = 0 .. self.get_total_number_neurons_needed().quant_to_usize();
        range.map(move |i| {
            let index = &CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i);
            let (internal, scratch)
                = self.spawn_neuron_data_at(index);
            let variance = NM::NeurogenesisDefaults::static_get_neuron_variance(
                index,
                &self.get_layout(),
                &self.get_model_cortical_variance()
            );
            (internal, scratch, variance)
        })
    }


    /// Writes the spawned neuron data into preallocated slices, one element per neuron
    fn write_into_slices(
        &self,
        internal: &mut [NM::NeuronDataInternal],
        scratch: &mut [NM::NeuronDataScratch],
        variance: &mut [NM::NeuronDataVariance],
    ) {
        let neuron_count = self.get_total_number_neurons_needed().quant_to_usize();
        assert_eq!(internal.len(), neuron_count);
        assert_eq!(scratch.len(), neuron_count);
        for (i, 
            (internal_out, 
            (scratch_out, variance_out)))
        in internal.iter_mut().zip(scratch.iter_mut().zip(variance.iter_mut())).enumerate() { 
            
            let index = CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i);
            let (internal, scratch)
                = self.spawn_neuron_data_at(&index);
            let variance = NM::NeurogenesisDefaults::static_get_neuron_variance(
                &index,
                &self.get_layout(),
                &self.get_model_cortical_variance()
            );
            (*internal_out, *scratch_out, *variance_out) = (internal, scratch, variance);
        }
    }

    // TODO parallel versions of the above
}


