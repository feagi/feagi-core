use feagi_data::create_wrapped_quantized_unsigned_integer;

/// The index of a burst engine relative to all that are loaded
pub type EngineIndex = u16;

//region Engine level Indexing

create_wrapped_quantized_unsigned_integer!(
    /// Index of a cortical area relative to its burst engine
    pub EngineCorticalIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a (firing/established) neuron relative to its burst engine (not including transient neurons like memory short term)
    pub EngineNeuronIndex
);


//endregion