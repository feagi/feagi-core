use feagi_structures::feagi_data::{create_quantized_decimal_wrapper, create_quantized_index_count_wrapper};


/// Defines a cortical area with an index unique in the entire NPU
create_quantized_index_count_wrapper!(NPUCorticalAreaIndexGlobal);


/// Defines a neuron with an index unique in the entire NPU
create_quantized_index_count_wrapper!(NPUNeuronIndexGlobal);


/// Defines a neuron with an index unique its membrane potential quantization
create_quantized_index_count_wrapper!(NPUNeuronIndexQuantizationLocal);


/// Defines a neuron with an index unique its neuron model type and membrane potential quantization
create_quantized_index_count_wrapper!(NPUNeuronIndexModelQuantizationLocal);

/// Points to a cortical area under a specific neuron model and quantization
create_quantized_index_count_wrapper!(NPUCorticalAreaModelQuantizationIndex);

/// Represents the potential of a neuron on its membrane
create_quantized_decimal_wrapper!(NPUNeuronMembranePotential);
