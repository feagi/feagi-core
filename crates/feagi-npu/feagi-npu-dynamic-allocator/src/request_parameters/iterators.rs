/// Common iterators that ae passed into parameters of NPU requests

pub mod cortical_neurons {
    use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
    use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

    /// Common trait that defines how to iterate the neurons over a dimensional cortical area
    pub trait DimensionalCorticalNeuronIterator {
        /// Given the dimensions of the cortical area, define how they will be iterated through.
        /// Order is not relevant
        fn iterate_over_dimensional_neuron_indexes<FIQ: FeagiIndexQuantization>(
            &self,
            dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>>;

        /// Estimate the number of neurons that will be iterated through. This is often used for
        /// memory allocation, ergo while estimating too high isn't ideal, it is far better than
        /// estimating too low and causing a crash. This default implementation simply runs through
        /// the iterator, which will always be accurate, but this can be replaced with a more
        /// efficient implementation if one is known.
        fn estimate_number_neurons_iterated<FIQ: FeagiIndexQuantization>(
            &self,
            dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        ) -> usize {
            Self::iterate_over_dimensional_neuron_indexes(self, dimensions,).count()
        }
    }

    /// Common trait that defines how to iterate the neurons over a dimensional cortical area
    pub trait MemoryCorticalNeuronIterator {
        /// Given the size of the cortical area, define how they will be iterated through.
        /// Order is not relevant
        fn iterate_over_memory_neuron_indexes<FIQ: FeagiIndexQuantization>(
            total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>>;

        /// Estimate the number of neurons that will be iterated through. This is often used for
        /// memory allocation, ergo while estimating too high isn't ideal, it is far better than
        /// estimating too low and causing a crash. This default implementation simply runs through
        /// the iterator, which will always be accurate, but this can be replaced with a more
        /// efficient implementation if one is known.
        fn estimate_number_neurons_iterated<FIQ: FeagiIndexQuantization>(
            total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        ) -> usize {
            Self::iterate_over_memory_neuron_indexes(total_neuron_count).count()
        }
    }

    /// Simply iterates over every single neuron of a cortical area
    pub struct AllNeuronIterator;

    impl DimensionalCorticalNeuronIterator for AllNeuronIterator {
        fn iterate_over_dimensional_neuron_indexes<FIQ: FeagiIndexQuantization>(
            &self,
            dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>> {
            let total = dimensions.number_contained_elements();

            todo!()
        }

        fn estimate_number_neurons_iterated<FIQ: FeagiIndexQuantization>(
            &self,
            dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        ) -> usize {
            dimensions.number_contained_elements().to_usize()
        }
    }

    impl MemoryCorticalNeuronIterator for AllNeuronIterator {
        fn iterate_over_memory_neuron_indexes<FIQ: FeagiIndexQuantization>(
            total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        ) -> impl Iterator<Item = NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>> {
            todo!()
        }

        fn estimate_number_neurons_iterated<FIQ: FeagiIndexQuantization>(
            total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        ) -> usize {
            total_neuron_count.to_usize() // lol
        }
    }
}
