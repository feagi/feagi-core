
/// Describes how a neuron collection is storing its data
#[derive(Clone, PartialEq)]
pub enum NeuronCollectionType {
    DenseFixedArray,
    DenseResizableVector,
    IndexedResizableVector,
}