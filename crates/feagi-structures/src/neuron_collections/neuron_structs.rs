//! Quantizxable data values for indexing and describing neuron data (Not Neuron Voxels!)


pub enum SingleCorticalNeuronCollectionType {
    DenseArray,
    DenseVector,
    IndexVector,
}


/// Neuron Potential of neuron_collections (not voxels!)
//region Neuron Membrane Potential

crate::define_quantizable_value_type_family!(NeuronMembranePotential);

//endregion

//region Neuron Index and Count

crate::define_quantizable_uint_type_family!(NeuronIndexCount);

//endregion

//region Neuron Density Per Voxel

pub use all_neuron_densities::NeuronDensityPerVoxel;

// We do this since we only want to expose the u8 level
mod all_neuron_densities {

    /// The number of neuron_collections that a single voxel represents. In most contexts this will be 1,
    /// but sometimes may be more, though never high, hence being locked to a u8. Cannot be 0
    pub type NeuronDensityPerVoxel = NeuronDensityPerVoxelAll<u8>;

    /// This defines multiple quantizations, which we dont care for
    crate::define_nonzero_count_family!(NeuronDensityPerVoxelAll);

}

//endregion