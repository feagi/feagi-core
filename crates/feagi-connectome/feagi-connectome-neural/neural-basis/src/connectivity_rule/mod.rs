pub mod voxel_to_voxel;

use feagi_basis::create_wrapped_quantized_unsigned_integer;
use crate::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutTag};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


/// A genome storable rule for how a source area connects its neurons to a destination area.
pub trait ConnectivityRule<FIQ: FeagiIndexQuantization> {
    type MappingInputLayout: NeuronLayout<FIQ>;
    type MappingOutputLayout: NeuronLayout<FIQ>;
}


/// Gives an iterator of a known total length an exact size, for adapters such as `filter` and
/// `flat_map` that cannot work out their own. `remaining` must match what `inner` will yield.
pub struct ExactCountIterator<I: Iterator> {
    inner: I,
    remaining: usize,
}

impl<I: Iterator> ExactCountIterator<I> {
    pub(crate) fn new(inner: I, remaining: usize) -> Self {
        Self { inner, remaining }
    }
}

impl<I: Iterator> Iterator for ExactCountIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();
        if item.is_some() {
            self.remaining -= 1;
        } else {
            debug_assert_eq!(self.remaining, 0);
        }
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<I: Iterator> ExactSizeIterator for ExactCountIterator<I> {}



create_wrapped_quantized_unsigned_integer!(
    /// Index of a synapse relative to its Axon Bundle
    pub AxonBundleSynapseLocalIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Number of synapses within an Axon Bundle
    pub AxonBundleSynapseLocalCount
);

create_wrapped_quantized_unsigned_integer!(
    /// Number of synapses in general
    pub SynapseCount
);

//endregion