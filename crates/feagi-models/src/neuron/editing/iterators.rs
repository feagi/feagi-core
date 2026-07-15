//! Common definitions of iterating over the neurons of a cortical area

use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Common trait that defines how to iterate the neurons over a dimensional cortical area
pub trait DimensionalCorticalNeuronIterator<FIQ: FeagiIndexQuantization> {
    /// Given the dimensions of the cortical area, define how they will be iterated through.
    /// Order is not relevant
    fn iterate_over_dimensional_neuron_indexes(
        &self,
        dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>>;

    /// Estimate the number of neurons that will be iterated through. This is often used for
    /// memory allocation, ergo while estimating too high isn't ideal, it is far better than
    /// estimating too low and causing a crash. This default implementation simply runs through
    /// the iterator, which will always be accurate, but this can be replaced with a more
    /// efficient implementation if one is known.
    fn estimate_number_neurons_iterated(
        &self,
        dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) -> usize {
        Self::iterate_over_dimensional_neuron_indexes(self, dimensions,).count()
    }
}

/// Common trait that defines how to iterate the neurons over a dimensional cortical area
pub trait MemoryCorticalNeuronIterator<FIQ: FeagiIndexQuantization> {
    /// Given the size of the cortical area, define how they will be iterated through.
    /// Order is not relevant
    fn iterate_over_memory_neuron_indexes(
        total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>>;

    /// Estimate the number of neurons that will be iterated through. This is often used for
    /// memory allocation, ergo while estimating too high isn't ideal, it is far better than
    /// estimating too low and causing a crash. This default implementation simply runs through
    /// the iterator, which will always be accurate, but this can be replaced with a more
    /// efficient implementation if one is known.
    fn estimate_number_neurons_iterated(
        total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    ) -> usize {
        Self::iterate_over_memory_neuron_indexes(total_neuron_count).count()
    }
}

/*
/// Simply iterates over every single neuron of a cortical area
pub struct AllNeuronIterator<FIQ: FeagiIndexQuantization> {
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> DimensionalCorticalNeuronIterator<FIQ> for AllNeuronIterator<FIQ> {
    fn iterate_over_dimensional_neuron_indexes(
        &self,
        dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>> {
        let total = dimensions.number_contained_elements();

        todo!()
    }

    fn estimate_number_neurons_iterated(
        &self,
        dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) -> usize {
        dimensions.number_contained_elements().to_usize()
    }
}

impl<FIQ: FeagiIndexQuantization> MemoryCorticalNeuronIterator<FIQ> for AllNeuronIterator<FIQ> {
    fn iterate_over_memory_neuron_indexes(
        total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>> {
        todo!()
    }

    fn estimate_number_neurons_iterated(
        total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    ) -> usize {
        total_neuron_count.to_usize() // lol
    }
}

 */