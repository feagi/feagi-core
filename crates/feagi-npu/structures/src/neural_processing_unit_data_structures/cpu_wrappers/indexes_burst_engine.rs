use feagi_structures::feagi_data::{create_quantized_decimal_wrapper, create_quantized_index_count_wrapper};


/// Defines a cortical area with an index unique to the burst engine
create_quantized_index_count_wrapper!(NPUWrappedCorticalAreaBurstEngineIndex);


/// Defines a neuron with an index unique to the burst engine. Relevant for both the FCL and
/// Membrane Potential
create_quantized_index_count_wrapper!(NPUWrappedNeuronIndexBurstEngineIndex);


