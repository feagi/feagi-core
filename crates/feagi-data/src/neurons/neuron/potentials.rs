use std::ops::{Index, IndexMut, Range};
use crate::{create_quantized_contiguous_linear_collections, create_wrapped_quantized_decimal};
use crate::collections::linear::contiguous_data::{QuantizedContiguousMutTrait, QuantizedContiguousTrait, QuantizedContiguousVector};
use crate::neurons::neuron::indexing::{NeuronCount, NeuronLocalIndex};

use crate::values::quantizable::{QuantizedDecimalUnwrappedTrait, QuantizedUnsignedIntegerUnwrappedTrait};

create_wrapped_quantized_decimal!(
    /// The membrane potential of a single neuron (NOT VOXEL)
    pub NeuronPotential
);

pub struct NeuronPotentialLinearVector<I: QuantizedUnsignedIntegerUnwrappedTrait, Q: QuantizedDecimalUnwrappedTrait>
{
    data: QuantizedContiguousVector<NeuronLocalIndex<I>, NeuronCount<I>, NeuronPotential<Q>>
}

impl<I: QuantizedUnsignedIntegerUnwrappedTrait, Q: QuantizedDecimalUnwrappedTrait> QuantizedContiguousTrait<NeuronLocalIndex<I>, NeuronCount<I>, NeuronPotential<Q>> for NeuronPotentialLinearVector<I, Q>
{
    fn as_slice(&self) -> &[NeuronPotential<Q>] {
        self.data.as_slice()
    }
}

impl<I: QuantizedUnsignedIntegerUnwrappedTrait, Q: QuantizedDecimalUnwrappedTrait> QuantizedContiguousMutTrait<NeuronLocalIndex<I>, NeuronCount<I>, NeuronPotential<Q>> for NeuronPotentialLinearVector<I, Q>
{
    fn as_mut_slice(&mut self) -> &mut [NeuronPotential<Q>] {
        self.data.as_mut_slice()
    }
}


impl<I: QuantizedUnsignedIntegerUnwrappedTrait, Q: QuantizedDecimalUnwrappedTrait> Index<NeuronLocalIndex<I>, Output=NeuronPotential<Q>> for NeuronPotentialLinearVector<I, Q> {
    type Output = ();

    fn index(&self, index: NeuronLocalIndex<I>) -> &Self::Output {
        todo!()
    }
}

impl<I: QuantizedUnsignedIntegerUnwrappedTrait, Q: QuantizedDecimalUnwrappedTrait> Index<Range<NeuronLocalIndex<I>>> for NeuronPotentialLinearVector<I, Q> {
    type Output = ();

    fn index(&self, index: Range<NeuronLocalIndex<I>>) -> &Self::Output {
        todo!()
    }
}

impl<I: QuantizedUnsignedIntegerUnwrappedTrait, Q: QuantizedDecimalUnwrappedTrait> IndexMut<NeuronLocalIndex<I>> for NeuronPotentialLinearVector<I, Q> {
    fn index_mut(&mut self, index: NeuronLocalIndex<I>) -> &mut Self::Output {
        todo!()
    }
}

impl<I: QuantizedUnsignedIntegerUnwrappedTrait, Q: QuantizedDecimalUnwrappedTrait> IndexMut<Range<NeuronLocalIndex<I>>, Output=[NeuronPotential<Q>]> for NeuronPotentialLinearVector<I, Q> {
    fn index_mut(&mut self, index: Range<NeuronLocalIndex<I>>) -> &mut Self::Output {
        todo!()
    }
}


