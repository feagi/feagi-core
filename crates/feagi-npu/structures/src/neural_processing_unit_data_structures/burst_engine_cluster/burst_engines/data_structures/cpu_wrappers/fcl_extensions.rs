use feagi_structures::feagi_data::{create_quantized_index_count_wrapper};


/// Index for primary FCLC which reduces into the FCL. Membrane Potential Quant Level index
create_quantized_index_count_wrapper!(NPUPrimaryFCLCQuantizationLocalIndex);

// Not creating FCL values, as they are just neuron potentials