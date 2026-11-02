pub mod cortical_area_generator_neuron_uniform;
#[cfg(feature = "alloc")]
pub mod cortical_area_generator_neuron_from_vector;
// TODO from heapless vec? Array?


use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::neuron_layout::NeuronLayout;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub trait CorticalAreaGeneratorNeuron<FIQ, NM>: Send
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    /// Get the neuron layout that will be used
    fn get_neuron_layout(&self) -> &NeuronLayoutOf<NM, FIQ>;

    /// A function that returns what the neuron data should be at a given index. Also runs the function for
    /// the cortical level neuron modifier contexts
    fn generate_neuron_data_at(&self, index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, variation: &NM::CorticalFunctionContext)
        -> (NM::NeuronDataInternal, NM::NeuronDataScratch);


    //region Default impls

    /// Number of neurons needed in the cortical area
    fn get_total_number_neurons_needed(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        self.get_neuron_layout().get_neuron_count()
    }

    /// Use as an iterator to generate neuron values without needing to collect it all first
    fn iter<'a>(&'a self, variation: &'a NM::CorticalFunctionContext)
                -> impl ExactSizeIterator<Item = (NM::NeuronDataInternal, NM::NeuronDataScratch)> + DoubleEndedIterator + '_
    {
        // NOTE: At this time rusts iter step trait is still nightly, hence this nonsense
        let range = 0 .. self.get_total_number_neurons_needed().quant_to_usize();
        range.map(move |i| self.generate_neuron_data_at(& CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i), &variation))
    }

    /// Writes the generated neuron data into preallocated slices, one element per neuron
    fn write_into_slice<'a>(
        &'a self,
        internal: &mut [NM::NeuronDataInternal],
        scratch: &mut [NM::NeuronDataScratch],
        variation: &'a NM::CorticalFunctionContext
    ) {
        let neuron_count = self.get_total_number_neurons_needed().quant_to_usize();
        assert_eq!(internal.len(), neuron_count);
        assert_eq!(scratch.len(), neuron_count);
        for (i, (internal_out, scratch_out)) in internal.iter_mut().zip(scratch.iter_mut()).enumerate() {
            (*internal_out, *scratch_out) = self.generate_neuron_data_at(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i), variation);
        }
    }

    /* TODO update this
    #[cfg(feature = "rayon")]
    fn par_iter(&self)
        -> impl rayon::iter::IndexedParallelIterator<Item = (NM::NeuronDataInternal, NM::NeuronDataScratch)> + '_
    where
        Self: Sync,
        NM::NeuronDataInternal: Send,
        NM::NeuronDataScratch: Send,
    {
        use rayon::prelude::*;
        let range = 0 .. self.get_total_number_neurons_needed().quant_to_usize();
        range.into_par_iter().map(move |i| self.generate_neuron_data_at(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i)))
    }

    /// Parallel version of [`Self::write_into_slice`]
    #[cfg(feature = "rayon")]
    fn par_write_into_slice(
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
        let neuron_count = self.get_total_number_neurons_needed().quant_to_usize();
        assert_eq!(internal.len(), neuron_count);
        assert_eq!(scratch.len(), neuron_count);
        internal.par_iter_mut().zip(scratch.par_iter_mut()).enumerate().for_each(|(i, (internal_out, scratch_out))| {
            (*internal_out, *scratch_out) = self.generate_neuron_data_at(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i));
        });
    }

     */

    //endregion
}
