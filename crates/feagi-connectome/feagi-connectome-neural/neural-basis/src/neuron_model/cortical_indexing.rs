use feagi_basis::create_wrapped_quantized_unsigned_integer;

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a cortical area within a burst engine
    pub CorticalAreaEngineIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a cortical area within a model and quantization
    pub NeuronModelCorticalQuantIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a neuron within a burst engine
    pub NeuronEngineIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a neuron within a model and quantization
    pub NeuronModelNeuronQuantIndex
);








