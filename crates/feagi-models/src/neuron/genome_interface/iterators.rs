//! Common definitions of iterating over the neurons of a cortical area

use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// TODO Rayon?

/// Common trait that defines how to iterate the neurons over a dimensional cortical area
pub trait DimensionalCorticalNeuronIterator {
    /// Given the dimensions of the cortical area, define how they will be iterated through.
    /// Order is not relevant
    fn iterate_over_dimensional_neuron_indexes<FIQ: FeagiIndexQuantization>(
        &self,
        dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>>;

    /// Get number of neurons that will be iterated through. This default implementation simply runs through
    /// the iterator, which will always be accurate, but this can be replaced with a more
    /// efficient implementation if one is known.
    fn number_neurons_iterated<FIQ: FeagiIndexQuantization>(
        &self,
        dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) -> usize {
        Self::iterate_over_dimensional_neuron_indexes(self, dimensions,).count()
    }
}

/// Common trait that defines how to iterate the neurons over a formless layout cortical area
pub trait FormlessLayoutCorticalNeuronIterator {
    /// Given the size of the cortical area, define how they will be iterated through.
    /// Order is not relevant
    fn iterate_over_memory_neuron_indexes<FIQ: FeagiIndexQuantization>(
        total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>>;

    /// Get the number of neurons that will be iterated through. This default implementation simply runs through
    /// the iterator, which will always be accurate, but this can be replaced with a more
    /// efficient implementation if one is known.
    fn number_neurons_iterated<FIQ: FeagiIndexQuantization>(
        total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    ) -> usize {
        Self::iterate_over_memory_neuron_indexes(total_neuron_count).count()
    }
}
