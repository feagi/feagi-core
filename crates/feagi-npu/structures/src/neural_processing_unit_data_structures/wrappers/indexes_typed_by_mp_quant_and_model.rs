//! These wrappers and their traits help ensure we are using the right neuron indexes in the right
//! spots. Has no direct function other than a programming aid for CPU code.

use feagi_structures::feagi_data::create_quantized_index_count_wrapper;



/// Defines a neuron with an index unique to the quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedNeuronMPQuantIndex);

/// Defines a FCLC Index (any level) with an index unique to the quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedFCLCMPQuantIndex);

/// Defines a neuron with an index unique to the quantization of a neuron membrane potential and
/// of the feagi standard model 
create_quantized_index_count_wrapper!(NPUWrappedNeuronNeuronModelMPQuantIndex);




// TODO belongs here?


create_quantized_index_count_wrapper!(NPUWrappedCorticalLayoutIndex);