use feagi_structures::feagi_data::{create_quantized_decimal_wrapper, create_quantized_index_count_wrapper};


/// Defines a cortical area with an index unique in the entire NPU
create_quantized_index_count_wrapper!(NPUCorticalAreaIndexGlobal);


/// Defines a neuron with an index unique in the entire NPU
create_quantized_index_count_wrapper!(NPUNeuronIndexGlobal);

