pub mod cortical_neuron_writers {
    use core::marker::PhantomData;
    use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
    use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
    use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
    use feagi_models::neuron::shared::data::NeuronModelNeuronData;

    /// Writes neuron data within a dimensional cortical area
    pub trait DimensionalCorticalNeuronWriter {
        type CorticalModelAndQuant: CorticalPotentialQuantization;
        type NeuronData: NeuronModelNeuronData<Self::CorticalModelAndQuant>;

        /// This function will be called using some `DimensionalCorticalNeuronIterator` to modify
        /// a given set of neurons from a cortical area
        fn write_dimensional_neuron_data<FIQ: FeagiIndexQuantization>(
            &self,
            existing_neuron: &mut Self::NeuronData,
            index: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
            cortical_dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        );
    }

    /// Writes neuron data within a memory cortical area
    pub trait MemoryCorticalNeuronWriter {
        type CorticalModelAndQuant: CorticalPotentialQuantization;
        type NeuronData: NeuronModelNeuronData<Self::CorticalModelAndQuant>;

        /// This function will be called using some `MemoryCorticalNeuronIterator` to modify
        /// a given set of neurons from a cortical area
        fn write_memory_neuron_data<FIQ: FeagiIndexQuantization>(
            &self,
            existing_neuron: &mut Self::NeuronData,
            index: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
            total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        );
    }


}
