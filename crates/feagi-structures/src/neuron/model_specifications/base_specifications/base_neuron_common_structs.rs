
/// Linear indexing and counting of neurons
crate::define_quantizable_uint_type_family!(LinearNeuronIndexCount);

/// Neuron Membrane potential
crate::define_quantizable_value_type_family!(NeuronMembranePotential);

/// Describes how a neuron collection is storing its data
#[derive(Clone, PartialEq)]
pub enum NeuronCollectionType {
    DenseFixedArray,
    DenseResizableVector,
    IndexedResizableVector,
}

