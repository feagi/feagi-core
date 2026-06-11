//! These wrappers and their traits help ensure we are using the right neuron indexes in the right
//! spots. Has no direct function other than a programming aid for CPU code.

use feagi_structures::feagi_data::create_quantized_index_count_wrapper;


//region Membrane Potential Quant Grouped

//region FCL

/// Defines a fcl with an index unique the f8 quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedFCLIndexF8Quant);

/// Defines a fcl with an index unique the f16 quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedFCLIndexF16Quant);

/// Defines a fcl with an index unique the f32 quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedFCLIndexF32Quant);

/// Defines a fcl with an index unique the f64 quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedFCLIndexF64Quant);

//endregion


//region Neuron 

/// Defines a neuron with an index unique the f8 quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedNeuronIndexF8Quant);

/// Defines a neuron with an index unique the f16 quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedNeuronIndexF16Quant);

/// Defines a neuron with an index unique the f32 quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedNeuronIndexF32Quant);

/// Defines a neuron with an index unique the f64 quantization of a neuron membrane potential
create_quantized_index_count_wrapper!(NPUWrappedNeuronIndexF64Quant);

//endregion

//endregion



//region Neuron Model + Neuron Model Quant Grouped

/// Defines a neuron with an index unique the f8 quantization of a neuron membrane potential and
/// of the feagi standard model 
create_quantized_index_count_wrapper!(NPUWrappedNeuronIndexF8QuantFeagiStandardModel);

/// Defines a neuron with an index unique the f16 quantization of a neuron membrane potential and
/// of the feagi standard model 
create_quantized_index_count_wrapper!(NPUWrappedNeuronIndexF16QuantFeagiStandardModel);

/// Defines a neuron with an index unique the f32 quantization of a neuron membrane potential and
/// of the feagi standard model 
create_quantized_index_count_wrapper!(NPUWrappedNeuronIndexF32QuantFeagiStandardModel);

/// Defines a neuron with an index unique the f64 quantization of a neuron membrane potential and
/// of the feagi standard model 
create_quantized_index_count_wrapper!(NPUWrappedNeuronIndexF64QuantFeagiStandardModel);



//endregion