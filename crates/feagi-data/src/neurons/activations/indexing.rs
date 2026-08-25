use crate::create_wrapped_quantized_unsigned_integer;

create_wrapped_quantized_unsigned_integer!(
    /// Index of the uint storing the bitpacked information of neuron activations
    pub NeuronActivationBitBatchIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Defines the number of BitBatched values encoding neuron activations. Likely has padding
    pub NeuronActivationBitBatchCount
);