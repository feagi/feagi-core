use feagi_structures_quantization::{define_quantized_index_count_wrapper_cpu, define_quantized_decimal_wrapper_cpu};


/// Linear indexing and counting of neurons
define_quantized_index_count_wrapper_cpu!(pub struct LinearNeuronIndexCount);

/// Neuron Membrane potential
define_quantized_decimal_wrapper_cpu!(pub struct NeuronMembranePotential);

