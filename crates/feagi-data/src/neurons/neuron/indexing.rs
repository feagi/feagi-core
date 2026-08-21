use crate::create_wrapped_quantized_unsigned_integer;

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron relative to its parent cortical area
    pub NeuronLocalIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Defines a number of neurons
    pub NeuronCount
);


create_wrapped_quantized_unsigned_integer!(
    /// Index of the uint storing the bitpacked information of neuron activations
    pub NeuronActivationBitBatchIndex
);